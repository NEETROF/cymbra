// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Pure practice-streak logic (change: add-practice-streak, design D1/D2).
//!
//! No I/O, no clock: every function takes the player's LOCAL day (computed from
//! the client's UTC offset by [`crate::play_core::local_day`], the same day
//! convention as the activity heatmap) and returns a decision the module
//! persists. Two rules live here and are exhaustively unit-tested:
//!
//! * [`advance`] — the day-transition: same-day replay is a no-op, the next day
//!   increments, a gap restarts the run at 1, and `longest` never decreases.
//! * [`recover_decision`] — the freeze: is the streak actually broken, is the
//!   break still inside the grace window, and can the user afford the cost?
//!   Deciding is separate from spending, so the "no silent debit" rule is
//!   provable without a database.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use cymbra_platform::email_template::SupportedLocale;

/// A user's streak state: the running count, the monotonic all-time best, and
/// the LOCAL day of their most recent play (`None` = never played).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StreakState {
    pub current: i64,
    pub longest: i64,
    pub last_played: Option<NaiveDate>,
}

impl StreakState {
    /// Whether the user has already played on `today` (their local day) — the
    /// "not at risk" test the reminder and the in-app nudge both use.
    pub fn played_on(&self, today: NaiveDate) -> bool {
        self.last_played == Some(today)
    }

    /// Whether the run stored here is **still alive** on `today`: it exists and
    /// the last play was today or yesterday.
    ///
    /// The stored `current` is a plain number that nothing ever decays — the row
    /// is only written by a play or a granted freeze — so a run abandoned months
    /// ago still reads as `current = 7`. Liveness is therefore a property of
    /// `(state, today)`, never of the row alone: every surface that shows or
    /// defends a streak asks this rather than `current > 0`.
    pub fn is_live(&self, today: NaiveDate) -> bool {
        self.current > 0 && self.days_since(today).is_some_and(|d| (0..=1).contains(&d))
    }

    /// Whether the run is broken on `today` — it existed and the last play is two
    /// or more days behind. Whether it can still be bought back is
    /// [`recover_decision`]'s call; this only says there is something to decide.
    pub fn is_broken(&self, today: NaiveDate) -> bool {
        self.current > 0 && self.days_since(today).is_some_and(|d| d >= 2)
    }

    /// Whether a live streak is at risk **right now**: the last play was
    /// yesterday and today is not secured yet.
    ///
    /// Bounded to a *live* run on purpose. Left open-ended it also matched every
    /// long-dead row, and the reminder job then pushed "keep your 1-day streak"
    /// to players whose streak had been gone for weeks.
    pub fn at_risk(&self, today: NaiveDate) -> bool {
        self.current > 0 && self.days_since(today) == Some(1)
    }

    /// Whole days elapsed since the last play (`None` = never played). `0` is a
    /// same-day replay, `1` is a consecutive next-day play, `>= 2` is a gap.
    fn days_since(&self, today: NaiveDate) -> Option<i64> {
        self.last_played.map(|last| (today - last).num_days())
    }
}

/// Tunable streak configuration (task 2.1). Both values are back-office feature
/// flags at the call site — held here so the pure decision stays testable and the
/// defaults live with the rule they parameterise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreakConfig {
    /// Points a confirmed freeze costs (`streak_freeze_cost`).
    pub freeze_cost: i64,
    /// How many MISSED days a break may still be recovered from
    /// (`streak_grace_days`). `1` = "you can rescue yesterday's break today".
    pub grace_days: i64,
}

/// Default freeze cost in points — a meaningful but reachable daily sink, sized
/// against the coverage curve (a handful of ratings).
pub const DEFAULT_FREEZE_COST: i64 = 30;

/// Default grace window in missed days: exactly one. A streak broken by missing
/// yesterday can be bought back today; miss two days and it is gone.
pub const DEFAULT_GRACE_DAYS: i64 = 1;

impl Default for StreakConfig {
    fn default() -> Self {
        Self {
            freeze_cost: DEFAULT_FREEZE_COST,
            grace_days: DEFAULT_GRACE_DAYS,
        }
    }
}

/// The streak after a play on `today` (the player's local day).
///
/// * never played → the run starts at 1;
/// * `today == last_played` → **unchanged** (a second play the same day never
///   advances a streak);
/// * `today == last_played + 1` → consecutive: the run grows by one;
/// * `today > last_played + 1` → the run was broken: a new one starts at 1;
/// * `today < last_played` → a late/replayed session from an earlier day: left
///   untouched (a streak never moves backwards).
///
/// `longest` is `max(longest, current)` in every branch, so it only ever rises.
pub fn advance(state: &StreakState, today: NaiveDate) -> StreakState {
    let current = match state.days_since(today) {
        None => 1,                       // first-ever play
        Some(0) => state.current.max(1), // same day: already counted
        Some(1) => state.current + 1,    // consecutive
        Some(d) if d > 1 => 1,           // gap: a fresh run starts today
        // Negative: a session dated BEFORE the last recorded day (clock skew, a
        // late outbox delivery). Nothing to advance, and nothing to lose.
        Some(_) => return *state,
    };
    StreakState {
        current,
        longest: state.longest.max(current),
        last_played: Some(today),
    }
}

/// Why a recovery offer is (not) available — the freeze decision (design D2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverDecision {
    /// The streak is broken, inside the grace window, and affordable: spending
    /// `cost` restores `restored` days.
    Allow { restored: i64, cost: i64 },
    /// Nothing to recover: no streak yet, or it is still intact (played today or
    /// yesterday), so nothing has been lost.
    Intact,
    /// The break is older than the grace window — the streak is gone for good.
    GraceElapsed,
    /// Broken and recoverable, but the user cannot pay for it.
    InsufficientPoints { needed: i64, available: i64 },
}

impl RecoverDecision {
    /// Whether the app should offer the recovery (the only variant that spends).
    pub fn is_allowed(&self) -> bool {
        matches!(self, RecoverDecision::Allow { .. })
    }
}

/// Whether `state` may be recovered on `today` for `balance` spendable points.
///
/// A break is only recoverable while the user has NOT played since it happened —
/// once they play, [`advance`] has restarted the run and the pre-break count is
/// gone (there is nothing left to restore). `grace_days` counts MISSED days: with
/// the default `1`, a play on day D is recoverable up to and including D+2 (one
/// whole missed day), and lost from D+3 on.
pub fn recover_decision(
    state: &StreakState,
    today: NaiveDate,
    balance: i64,
    cfg: &StreakConfig,
) -> RecoverDecision {
    let Some(gap) = state.days_since(today) else {
        return RecoverDecision::Intact; // never played: no streak to lose
    };
    // `gap <= 1` is a live streak (played today, or yesterday and still in time);
    // a non-positive current streak means there is nothing to buy back either.
    if state.current <= 0 || gap <= 1 {
        return RecoverDecision::Intact;
    }
    // Days actually missed: the gap minus the "yesterday still counts" day.
    if gap - 1 > cfg.grace_days.max(0) {
        return RecoverDecision::GraceElapsed;
    }
    if balance < cfg.freeze_cost {
        return RecoverDecision::InsufficientPoints {
            needed: cfg.freeze_cost,
            available: balance,
        };
    }
    RecoverDecision::Allow {
        restored: state.current,
        cost: cfg.freeze_cost,
    }
}

/// The state a granted recovery writes: the pre-break run is kept and stamped as
/// played today, so tomorrow's play continues it. `longest` is re-maxed for the
/// (rare) case where the restored run is the user's best.
pub fn recovered(state: &StreakState, today: NaiveDate) -> StreakState {
    StreakState {
        current: state.current,
        longest: state.longest.max(state.current),
        last_played: Some(today),
    }
}

// --- reminder candidates (task 3.2) -----------------------------------------

/// One row of the reminder sweep: a live streak plus the account's IANA zone and
/// locale. The zone is what makes "has not played **today**" answerable —
/// `last_played` is a LOCAL day, so comparing it against a UTC date would nudge
/// every player west of Greenwich on the evening they actually practised. The
/// locale is needed because the push platform sends copy, not keys: it does not
/// translate, so the sender must supply the finished sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderCandidate {
    pub user_id: String,
    pub state: StreakState,
    /// The account's stored IANA zone. Empty/unparsable falls back to `fallback_tz`.
    pub timezone: String,
    /// The account's stored locale tag (`fr`, `fr-FR`, …). Empty = English.
    pub locale: String,
}

/// One dispatchable batch: the users who share a locale AND a streak length, so
/// a single already-rendered message is correct for all of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderGroup {
    pub locale: SupportedLocale,
    /// The streak every user in this group stands to lose.
    pub streak: i64,
    pub user_ids: Vec<String>,
}

/// The user ids to remind: everyone whose live streak has no play on **their own**
/// current local day. Users who already played today are filtered out here — the
/// push platform never sees them, so no consent or hour gate can let one slip
/// through. Consent, the kill-switch and platform selection stay the platform's
/// job; this is only the candidate set.
pub fn at_risk_user_ids(
    rows: &[ReminderCandidate],
    now: chrono::DateTime<chrono::Utc>,
    fallback_tz: &str,
) -> Vec<String> {
    rows.iter()
        .filter(|c| c.state.at_risk(local_today(&c.timezone, fallback_tz, now)))
        .map(|c| c.user_id.clone())
        .collect()
}

/// The at-risk users batched by `(locale, streak)` — one batch per distinct
/// message, since the platform ships finished copy rather than a template.
/// Deterministically ordered (by locale then streak) so a re-delivered job sends
/// the same batches in the same order.
pub fn reminder_groups(
    rows: &[ReminderCandidate],
    now: chrono::DateTime<chrono::Utc>,
    fallback_tz: &str,
) -> Vec<ReminderGroup> {
    let mut by_key: BTreeMap<(String, i64), Vec<String>> = BTreeMap::new();
    for c in rows {
        if !c.state.at_risk(local_today(&c.timezone, fallback_tz, now)) {
            continue;
        }
        by_key
            .entry((locale_key(&c.locale).to_string(), c.state.current))
            .or_default()
            .push(c.user_id.clone());
    }
    by_key
        .into_iter()
        .map(|((locale, streak), mut user_ids)| {
            user_ids.sort();
            ReminderGroup {
                locale: SupportedLocale::parse(Some(&locale)),
                streak,
                user_ids,
            }
        })
        .collect()
}

/// The canonical tag a locale groups under, so `fr`, `fr-FR` and `FR_ca` all land
/// in one batch.
fn locale_key(tag: &str) -> &'static str {
    match SupportedLocale::parse(Some(tag)) {
        SupportedLocale::Fr => "fr",
        SupportedLocale::Es => "es",
        SupportedLocale::It => "it",
        SupportedLocale::En => "en",
    }
}

/// The reminder's `(title, body)` for a `streak`-day run, in `locale` — already
/// localized, because the push platform transports copy and never translates it.
/// Loss-framed on purpose (that is the mechanic) but kept warm and short, and the
/// whole category is opt-out per user.
pub fn reminder_copy(locale: SupportedLocale, streak: i64) -> (String, String) {
    match locale {
        SupportedLocale::Fr => (
            "Ta série continue ?".into(),
            format!("Il te reste peu de temps pour garder ta série de {streak} jours."),
        ),
        SupportedLocale::Es => (
            "¿Sigue tu racha?".into(),
            format!("Te queda poco tiempo para mantener tu racha de {streak} días."),
        ),
        SupportedLocale::It => (
            "La tua serie continua?".into(),
            format!("Ti resta poco tempo per mantenere la tua serie di {streak} giorni."),
        ),
        SupportedLocale::En => (
            "Keep your streak?".into(),
            format!("Not long left to keep your {streak}-day streak going."),
        ),
    }
}

/// `now` as a calendar date in `zone`, falling back to `fallback` and finally to
/// UTC when neither parses — a garbage client-supplied zone must never abort the
/// sweep, only cost that one user a slightly-off day boundary.
fn local_today(zone: &str, fallback: &str, now: chrono::DateTime<chrono::Utc>) -> NaiveDate {
    match zone
        .parse::<chrono_tz::Tz>()
        .ok()
        .or_else(|| fallback.parse::<chrono_tz::Tz>().ok())
    {
        Some(tz) => now.with_timezone(&tz).date_naive(),
        None => now.date_naive(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn state(current: i64, longest: i64, last: Option<NaiveDate>) -> StreakState {
        StreakState {
            current,
            longest,
            last_played: last,
        }
    }

    #[test]
    fn first_play_starts_the_streak() {
        let s = advance(&StreakState::default(), ymd(2026, 8, 12));
        assert_eq!(s.current, 1);
        assert_eq!(s.longest, 1);
        assert_eq!(s.last_played, Some(ymd(2026, 8, 12)));
    }

    #[test]
    fn same_day_replay_does_not_advance() {
        let before = state(4, 9, Some(ymd(2026, 8, 12)));
        let after = advance(&before, ymd(2026, 8, 12));
        assert_eq!(after, before, "a second play the same day changes nothing");
    }

    #[test]
    fn next_day_play_increments_and_tracks_longest() {
        let before = state(4, 4, Some(ymd(2026, 8, 12)));
        let after = advance(&before, ymd(2026, 8, 13));
        assert_eq!(after.current, 5);
        assert_eq!(after.longest, 5, "a new personal best is recorded");
        assert_eq!(after.last_played, Some(ymd(2026, 8, 13)));
    }

    #[test]
    fn longest_is_retained_when_the_run_is_shorter() {
        let before = state(2, 30, Some(ymd(2026, 8, 12)));
        let after = advance(&before, ymd(2026, 8, 13));
        assert_eq!(after.current, 3);
        assert_eq!(after.longest, 30, "longest never decreases");
    }

    #[test]
    fn a_gap_resets_the_run_but_keeps_the_record() {
        let before = state(12, 12, Some(ymd(2026, 8, 12)));
        // Skipped the 13th entirely.
        let after = advance(&before, ymd(2026, 8, 14));
        assert_eq!(after.current, 1, "a broken run restarts at one");
        assert_eq!(after.longest, 12);
        // ...and a much longer gap behaves the same.
        let far = advance(&before, ymd(2026, 12, 25));
        assert_eq!(far.current, 1);
        assert_eq!(far.longest, 12);
    }

    #[test]
    fn a_backdated_session_never_moves_the_streak() {
        // A late outbox delivery (or a client clock jumping back) must not reset
        // a healthy streak — the day is older than what is already recorded.
        let before = state(6, 6, Some(ymd(2026, 8, 12)));
        assert_eq!(advance(&before, ymd(2026, 8, 10)), before);
    }

    #[test]
    fn same_day_replay_repairs_a_zero_run() {
        // Defensive: a row left at current = 0 with today's date (e.g. seeded)
        // still reads as a one-day streak rather than zero.
        let before = state(0, 3, Some(ymd(2026, 8, 12)));
        assert_eq!(advance(&before, ymd(2026, 8, 12)).current, 1);
    }

    // --- recovery / freeze ---------------------------------------------------

    #[test]
    fn an_intact_streak_has_nothing_to_recover() {
        let cfg = StreakConfig::default();
        // Played today.
        let today_played = state(5, 5, Some(ymd(2026, 8, 12)));
        assert_eq!(
            recover_decision(&today_played, ymd(2026, 8, 12), 1000, &cfg),
            RecoverDecision::Intact
        );
        // Played yesterday: still live, today is not over.
        assert_eq!(
            recover_decision(&today_played, ymd(2026, 8, 13), 1000, &cfg),
            RecoverDecision::Intact
        );
        // Never played at all.
        assert_eq!(
            recover_decision(&StreakState::default(), ymd(2026, 8, 13), 1000, &cfg),
            RecoverDecision::Intact
        );
    }

    #[test]
    fn a_break_inside_the_grace_window_is_offered() {
        let cfg = StreakConfig::default(); // grace = 1 missed day, cost = 30
        let broken = state(7, 7, Some(ymd(2026, 8, 12)));
        assert_eq!(
            recover_decision(&broken, ymd(2026, 8, 14), 30, &cfg),
            RecoverDecision::Allow {
                restored: 7,
                cost: 30
            }
        );
    }

    #[test]
    fn the_grace_boundary_is_exact() {
        let cfg = StreakConfig::default(); // 1 missed day
        let broken = state(7, 7, Some(ymd(2026, 8, 12)));
        // Missed the 13th → recoverable on the 14th.
        assert!(recover_decision(&broken, ymd(2026, 8, 14), 999, &cfg).is_allowed());
        // Missed the 13th AND the 14th → gone on the 15th.
        assert_eq!(
            recover_decision(&broken, ymd(2026, 8, 15), 999, &cfg),
            RecoverDecision::GraceElapsed
        );
    }

    #[test]
    fn a_wider_grace_window_extends_the_offer() {
        let cfg = StreakConfig {
            freeze_cost: 30,
            grace_days: 3,
        };
        let broken = state(7, 7, Some(ymd(2026, 8, 12)));
        assert!(recover_decision(&broken, ymd(2026, 8, 16), 999, &cfg).is_allowed());
        assert_eq!(
            recover_decision(&broken, ymd(2026, 8, 17), 999, &cfg),
            RecoverDecision::GraceElapsed
        );
    }

    #[test]
    fn an_unaffordable_recovery_is_refused_before_it_is_offered() {
        let cfg = StreakConfig::default();
        let broken = state(7, 7, Some(ymd(2026, 8, 12)));
        assert_eq!(
            recover_decision(&broken, ymd(2026, 8, 14), 29, &cfg),
            RecoverDecision::InsufficientPoints {
                needed: 30,
                available: 29
            }
        );
        // Exactly the cost is enough (the boundary spends the whole balance).
        assert!(recover_decision(&broken, ymd(2026, 8, 14), 30, &cfg).is_allowed());
    }

    #[test]
    fn a_zero_grace_window_forbids_every_recovery() {
        let cfg = StreakConfig {
            freeze_cost: 0,
            grace_days: 0,
        };
        let broken = state(7, 7, Some(ymd(2026, 8, 12)));
        assert_eq!(
            recover_decision(&broken, ymd(2026, 8, 14), 999, &cfg),
            RecoverDecision::GraceElapsed
        );
    }

    #[test]
    fn recovering_keeps_the_run_and_stamps_today() {
        let broken = state(7, 7, Some(ymd(2026, 8, 12)));
        let after = recovered(&broken, ymd(2026, 8, 14));
        assert_eq!(after.current, 7, "the pre-break run is restored");
        assert_eq!(after.last_played, Some(ymd(2026, 8, 14)));
        // ...and it continues normally the next day.
        assert_eq!(advance(&after, ymd(2026, 8, 15)).current, 8);
    }

    #[test]
    fn recovering_can_set_a_new_record() {
        let broken = state(7, 3, Some(ymd(2026, 8, 12)));
        assert_eq!(recovered(&broken, ymd(2026, 8, 14)).longest, 7);
    }

    // --- at-risk / played-today ---------------------------------------------

    #[test]
    fn at_risk_is_a_live_streak_without_a_play_today() {
        let s = state(3, 3, Some(ymd(2026, 8, 12)));
        assert!(s.at_risk(ymd(2026, 8, 13)));
        assert!(!s.at_risk(ymd(2026, 8, 12)), "already played today");
        assert!(s.played_on(ymd(2026, 8, 12)));
        // No streak → nothing at risk, whatever the date.
        assert!(!StreakState::default().at_risk(ymd(2026, 8, 13)));
        assert!(!state(0, 9, Some(ymd(2026, 8, 1))).at_risk(ymd(2026, 8, 13)));
    }

    #[test]
    fn a_broken_run_is_never_at_risk() {
        // The row still says "3", but the run died two days ago: there is nothing
        // left to defend, so no nudge and no reminder — the message would be
        // "keep your 3-day streak" about a streak the player no longer has.
        let s = state(3, 3, Some(ymd(2026, 8, 12)));
        assert!(!s.at_risk(ymd(2026, 8, 14)), "broken, not at risk");
        assert!(
            !s.at_risk(ymd(2026, 12, 25)),
            "long dead, still not at risk"
        );
    }

    #[test]
    fn liveness_is_a_property_of_the_state_and_the_day() {
        let s = state(3, 3, Some(ymd(2026, 8, 12)));
        assert!(s.is_live(ymd(2026, 8, 12)), "played today");
        assert!(
            s.is_live(ymd(2026, 8, 13)),
            "played yesterday, today is open"
        );
        assert!(!s.is_live(ymd(2026, 8, 14)), "a whole day was missed");
        assert!(!s.is_live(ymd(2026, 12, 25)));
        // A stored zero and a never-played row are never live.
        assert!(!state(0, 9, Some(ymd(2026, 8, 12))).is_live(ymd(2026, 8, 12)));
        assert!(!StreakState::default().is_live(ymd(2026, 8, 12)));
    }

    #[test]
    fn broken_is_the_complement_of_live_once_a_run_exists() {
        let s = state(3, 3, Some(ymd(2026, 8, 12)));
        for day in [ymd(2026, 8, 12), ymd(2026, 8, 13)] {
            assert!(!s.is_broken(day), "{day} is still live");
        }
        for day in [ymd(2026, 8, 14), ymd(2026, 12, 25)] {
            assert!(s.is_broken(day), "{day} is past the break");
        }
        // Nothing to break without a run.
        assert!(!StreakState::default().is_broken(ymd(2026, 8, 14)));
    }

    // --- reminder candidates -------------------------------------------------

    fn candidate(user: &str, current: i64, last: NaiveDate, tz: &str) -> ReminderCandidate {
        ReminderCandidate {
            user_id: user.into(),
            state: state(current, current, Some(last)),
            timezone: tz.into(),
            locale: String::new(),
        }
    }

    fn in_locale(mut c: ReminderCandidate, locale: &str) -> ReminderCandidate {
        c.locale = locale.into();
        c
    }

    /// 2026-08-12T22:30:00Z — already the 13th in Paris (+02), still the 12th in
    /// New York (-04). The two accounts below disagree about what "today" is.
    fn late_evening_utc() -> chrono::DateTime<chrono::Utc> {
        "2026-08-12T22:30:00Z".parse().unwrap()
    }

    #[test]
    fn at_risk_uses_each_players_own_local_day() {
        let rows = vec![
            // Played on the 12th. In Paris it is already the 13th → at risk.
            candidate("paris", 5, ymd(2026, 8, 12), "Europe/Paris"),
            // Played on the 12th. In New York it is STILL the 12th → already
            // played today, must not be nudged.
            candidate("ny", 5, ymd(2026, 8, 12), "America/New_York"),
        ];
        assert_eq!(
            at_risk_user_ids(&rows, late_evening_utc(), "Europe/Paris"),
            vec!["paris".to_string()]
        );
    }

    #[test]
    fn users_without_a_streak_are_never_candidates() {
        let rows = vec![
            candidate("live", 3, ymd(2026, 8, 12), "Europe/Paris"),
            candidate("none", 0, ymd(2026, 7, 1), "Europe/Paris"),
            ReminderCandidate {
                user_id: "never".into(),
                state: StreakState::default(),
                timezone: "Europe/Paris".into(),
                locale: String::new(),
            },
        ];
        assert_eq!(
            at_risk_user_ids(&rows, late_evening_utc(), "Europe/Paris"),
            vec!["live".to_string()]
        );
    }

    #[test]
    fn an_unusable_timezone_falls_back_instead_of_dropping_the_user() {
        // A blank or garbage stored zone still yields a decision (via the
        // fallback), rather than silently excluding the account from reminders.
        let rows = vec![
            candidate("blank", 5, ymd(2026, 8, 12), ""),
            candidate("garbage", 5, ymd(2026, 8, 12), "Mars/Olympus_Mons"),
        ];
        // Fallback Paris → already the 13th → both are at risk.
        assert_eq!(
            at_risk_user_ids(&rows, late_evening_utc(), "Europe/Paris"),
            vec!["blank".to_string(), "garbage".to_string()]
        );
        // An unusable FALLBACK degrades to UTC rather than panicking: at 22:30Z
        // it is still the 12th in UTC, so both count as having played today and
        // nobody is nudged — a wrong-by-a-few-hours boundary, never a crash.
        assert!(at_risk_user_ids(&rows, late_evening_utc(), "Nowhere/At_All").is_empty());
    }

    #[test]
    fn an_empty_sweep_reminds_nobody() {
        assert!(at_risk_user_ids(&[], late_evening_utc(), "Europe/Paris").is_empty());
        assert!(reminder_groups(&[], late_evening_utc(), "Europe/Paris").is_empty());
    }

    #[test]
    fn groups_batch_one_message_per_locale_and_streak() {
        let last = ymd(2026, 8, 12);
        let rows = vec![
            in_locale(candidate("fr-a", 5, last, "Europe/Paris"), "fr-FR"),
            in_locale(candidate("fr-b", 5, last, "Europe/Paris"), "fr"),
            // Same locale, different streak → its own message.
            in_locale(candidate("fr-c", 9, last, "Europe/Paris"), "fr"),
            // Same streak, different locale → its own message.
            in_locale(candidate("en-a", 5, last, "Europe/Paris"), "en-GB"),
            // Unknown locale collapses into English.
            in_locale(candidate("xx-a", 5, last, "Europe/Paris"), "tlh"),
        ];
        let groups = reminder_groups(&rows, late_evening_utc(), "Europe/Paris");
        assert_eq!(groups.len(), 3);
        assert_eq!(
            groups[0],
            ReminderGroup {
                locale: SupportedLocale::En,
                streak: 5,
                user_ids: vec!["en-a".into(), "xx-a".into()],
            }
        );
        assert_eq!(
            groups[1],
            ReminderGroup {
                locale: SupportedLocale::Fr,
                streak: 5,
                user_ids: vec!["fr-a".into(), "fr-b".into()],
            }
        );
        assert_eq!(groups[2].streak, 9);
        assert_eq!(groups[2].user_ids, vec!["fr-c".to_string()]);
    }

    #[test]
    fn groups_apply_the_same_already_played_filter() {
        // The New-York player has still not reached tomorrow, so they are not in
        // any batch — the platform never sees them.
        let rows = vec![
            in_locale(
                candidate("paris", 5, ymd(2026, 8, 12), "Europe/Paris"),
                "fr",
            ),
            in_locale(
                candidate("ny", 5, ymd(2026, 8, 12), "America/New_York"),
                "en",
            ),
        ];
        let groups = reminder_groups(&rows, late_evening_utc(), "Europe/Paris");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].user_ids, vec!["paris".to_string()]);
    }

    #[test]
    fn the_copy_names_the_streak_in_every_supported_locale() {
        for locale in [
            SupportedLocale::En,
            SupportedLocale::Fr,
            SupportedLocale::Es,
            SupportedLocale::It,
        ] {
            let (title, body) = reminder_copy(locale, 12);
            assert!(!title.is_empty());
            assert!(body.contains("12"), "{locale:?} body must name the streak");
        }
        // The French copy is the design's wording.
        let (_, fr) = reminder_copy(SupportedLocale::Fr, 12);
        assert!(fr.contains("série de 12 jours"), "{fr}");
    }
}
