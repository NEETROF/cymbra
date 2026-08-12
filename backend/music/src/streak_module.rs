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

//! Practice-streak domain logic (change: add-practice-streak).
//!
//! Orchestrates the pure rules of [`crate::streak_core`] over a [`StreakRepo`]:
//! advance the streak on each ingested play, read the standing the app-bar chip
//! renders, spend points to recover a broken streak, and resolve the reminder
//! job's at-risk set.
//!
//! The **server** is the source of truth (design D1): the client sends only its
//! UTC offset, from which the module derives the player's local day exactly like
//! the activity heatmap. A modified client can shift its own day boundary — that
//! is a soft product mechanic, not a security boundary — but it cannot assert a
//! streak count, and `longest_streak` is monotonic, so the blast radius is
//! capped.

use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};
use cymbra_platform::{AppError, Result};

use crate::play_core::local_day;
use crate::streak::StreakRepo;
use crate::streak_core::{
    RecoverDecision, ReminderGroup, StreakConfig, StreakState, advance, at_risk_user_ids,
    recover_decision, recovered, reminder_groups,
};

/// What the app needs to render the streak: the counts, whether today is already
/// secured, and the recovery offer (if any).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreakStanding {
    pub state: StreakState,
    /// Whether the user has already played on their current local day.
    pub played_today: bool,
    /// The freeze decision as of today — `Allow` is the only variant the app
    /// turns into an offer.
    pub recover: RecoverDecision,
}

impl StreakStanding {
    /// The streak the chip shows. A run broken but still recoverable keeps
    /// displaying its pre-break value — it has not been lost yet.
    pub fn display_streak(&self) -> i64 {
        self.state.current
    }
}

/// Where the freeze cost + grace window come from at **call** time (task 2.1).
///
/// A trait rather than a stored [`StreakConfig`] so the values stay
/// hot-reloadable: the server implements this over the feature-flag service, and
/// an operator retuning `streak.freeze_cost` in the back office changes the next
/// offer with no redeploy. The music module stays free of a flag dependency.
pub trait StreakConfigSource: Send + Sync {
    fn streak_config(&self) -> StreakConfig;
}

/// Streak maintenance + the freeze, over an owner-scoped [`StreakRepo`].
pub struct StreakModule {
    repo: Arc<dyn StreakRepo>,
    config: StreakConfig,
    source: Option<Arc<dyn StreakConfigSource>>,
}

impl StreakModule {
    pub fn new(repo: Arc<dyn StreakRepo>) -> Self {
        Self {
            repo,
            config: StreakConfig::default(),
            source: None,
        }
    }

    /// Fix the freeze cost / grace window (tests, or a deployment with no flag
    /// store). A [`Self::with_config_source`] takes precedence.
    pub fn with_config(mut self, config: StreakConfig) -> Self {
        self.config = config;
        self
    }

    /// Resolve the freeze cost / grace window per call, so back-office changes
    /// apply without a redeploy.
    pub fn with_config_source(mut self, source: Arc<dyn StreakConfigSource>) -> Self {
        self.source = Some(source);
        self
    }

    /// The configuration in force right now.
    pub fn config(&self) -> StreakConfig {
        match &self.source {
            Some(s) => s.streak_config(),
            None => self.config,
        }
    }

    /// Advance `user_id`'s streak for a play that ended at `played_at_ms` with
    /// the client offset `tz_offset_minutes`. A second play the same local day
    /// writes nothing, so the ingest path stays cheap for a practising user, and
    /// a retried delivery of the same session is naturally a no-op.
    pub async fn record_play(
        &self,
        user_id: &str,
        played_at_ms: i64,
        tz_offset_minutes: i32,
    ) -> Result<StreakState> {
        let today = local_day(played_at_ms, tz_offset_minutes);
        let before = self.repo.get(user_id).await?;
        let after = advance(&before, today);
        if after != before {
            self.repo.put(user_id, &after).await?;
        }
        Ok(after)
    }

    /// `user_id`'s standing on `today` (their local day), including whether a
    /// recovery is on offer and what it would cost.
    pub async fn standing(&self, user_id: &str, today: NaiveDate) -> Result<StreakStanding> {
        let state = self.repo.get(user_id).await?;
        // The balance is only needed to decide affordability; skip the read when
        // the streak is not broken at all (the overwhelmingly common case).
        let balance = if state.at_risk(today) {
            self.repo.spendable_points(user_id).await?
        } else {
            0
        };
        Ok(StreakStanding {
            state,
            played_today: state.played_on(today),
            recover: recover_decision(&state, today, balance, &self.config()),
        })
    }

    /// Spend points to restore `user_id`'s broken streak on `today`. This is the
    /// **confirmed** side of the freeze: the app only calls it after the user
    /// accepts, and nothing here charges anyone who did not.
    ///
    /// Refuses an intact streak, a break past the grace window, and an
    /// unaffordable one — each with a distinct precondition failure so the app
    /// can say why. The debit and the restore are one atomic repo operation, and
    /// the balance is re-checked inside it, so a concurrent double-confirm
    /// charges exactly once.
    pub async fn recover(&self, user_id: &str, today: NaiveDate) -> Result<StreakStanding> {
        let state = self.repo.get(user_id).await?;
        let balance = self.repo.spendable_points(user_id).await?;
        let cost = match recover_decision(&state, today, balance, &self.config()) {
            RecoverDecision::Allow { cost, .. } => cost,
            RecoverDecision::Intact => {
                return Err(AppError::FailedPrecondition(
                    "there is no broken streak to recover".into(),
                ));
            }
            RecoverDecision::GraceElapsed => {
                return Err(AppError::FailedPrecondition(
                    "this streak can no longer be recovered".into(),
                ));
            }
            RecoverDecision::InsufficientPoints { needed, available } => {
                return Err(AppError::FailedPrecondition(format!(
                    "not enough points: {needed} needed, {available} available"
                )));
            }
        };
        let restored = recovered(&state, today);
        if !self
            .repo
            .spend_and_restore(user_id, &restored, cost)
            .await?
        {
            // The balance moved between the decision and the charge (a parallel
            // spend). Nothing was written; report it like any other refusal.
            return Err(AppError::FailedPrecondition(
                "not enough points to recover this streak".into(),
            ));
        }
        Ok(StreakStanding {
            state: restored,
            played_today: true,
            recover: RecoverDecision::Intact,
        })
    }

    /// The users to remind at `now`: those holding a streak who have not played
    /// on **their own** current local day (task 3.2). Users who already played
    /// never reach the push platform.
    pub async fn reminder_candidates(
        &self,
        now: DateTime<Utc>,
        fallback_tz: &str,
    ) -> Result<Vec<String>> {
        let rows = self.repo.live_streaks().await?;
        Ok(at_risk_user_ids(&rows, now, fallback_tz))
    }

    /// The same at-risk set, batched by `(locale, streak)` — one batch per
    /// distinct message, since the push platform transports finished copy rather
    /// than a template (task 3.3).
    pub async fn reminder_groups(
        &self,
        now: DateTime<Utc>,
        fallback_tz: &str,
    ) -> Result<Vec<ReminderGroup>> {
        let rows = self.repo.live_streaks().await?;
        Ok(reminder_groups(&rows, now, fallback_tz))
    }

    /// `user_id`'s all-time best run — the counter the practice-streak badges are
    /// measured against (task 4.2).
    pub async fn longest_streak(&self, user_id: &str) -> Result<i64> {
        Ok(self.repo.get(user_id).await?.longest)
    }

    /// `user_id`'s spendable points — what a freeze is paid from. Surfaced so the
    /// recovery RPC can report the fresh balance the way the reward shop does.
    pub async fn spendable_points(&self, user_id: &str) -> Result<i64> {
        self.repo.spendable_points(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::streak::FakeStreakRepo;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// Epoch millis for local midday on `date` at UTC+0 (so the local day of a
    /// play is exactly `date`).
    fn noon(date: NaiveDate) -> i64 {
        date.and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_millis()
    }

    fn module(repo: Arc<FakeStreakRepo>) -> StreakModule {
        StreakModule::new(repo)
    }

    #[tokio::test]
    async fn a_first_play_starts_a_streak_and_a_replay_does_not_advance_it() {
        let repo = Arc::new(FakeStreakRepo::default());
        let m = module(repo.clone());
        let day = ymd(2026, 8, 12);
        let s = m.record_play("u1", noon(day), 0).await.unwrap();
        assert_eq!((s.current, s.longest), (1, 1));
        // A second session the same local day leaves it at one.
        let s = m.record_play("u1", noon(day) + 3_600_000, 0).await.unwrap();
        assert_eq!(s.current, 1);
    }

    #[tokio::test]
    async fn consecutive_days_build_the_streak_and_a_gap_restarts_it() {
        let repo = Arc::new(FakeStreakRepo::default());
        let m = module(repo.clone());
        for d in 12..=14 {
            m.record_play("u1", noon(ymd(2026, 8, d)), 0).await.unwrap();
        }
        let s = repo.get("u1").await.unwrap();
        assert_eq!((s.current, s.longest), (3, 3));
        // Skip the 15th; play the 16th.
        let s = m
            .record_play("u1", noon(ymd(2026, 8, 16)), 0)
            .await
            .unwrap();
        assert_eq!(s.current, 1);
        assert_eq!(s.longest, 3, "the record survives the break");
    }

    #[tokio::test]
    async fn the_day_boundary_follows_the_clients_offset() {
        let repo = Arc::new(FakeStreakRepo::default());
        let m = module(repo.clone());
        // 2026-08-12T23:30Z is already the 13th at +60 minutes (CET).
        let late = ymd(2026, 8, 12).and_hms_opt(23, 30, 0).unwrap();
        let ms = late.and_utc().timestamp_millis();
        m.record_play("u1", ms, 60).await.unwrap();
        assert_eq!(
            repo.get("u1").await.unwrap().last_played,
            Some(ymd(2026, 8, 13))
        );
    }

    #[tokio::test]
    async fn standing_reports_today_and_no_offer_for_a_live_streak() {
        let repo = Arc::new(FakeStreakRepo::default());
        let m = module(repo.clone());
        m.record_play("u1", noon(ymd(2026, 8, 12)), 0)
            .await
            .unwrap();
        let st = m.standing("u1", ymd(2026, 8, 12)).await.unwrap();
        assert!(st.played_today);
        assert_eq!(st.recover, RecoverDecision::Intact);
        assert_eq!(st.display_streak(), 1);
    }

    /// A user with a 7-day streak last played on the 12th, and 100 points.
    fn broken_user(repo: &FakeStreakRepo) {
        repo.seed(
            "u1",
            StreakState {
                current: 7,
                longest: 7,
                last_played: Some(ymd(2026, 8, 12)),
            },
        );
        repo.seed_balance("u1", 100);
    }

    #[tokio::test]
    async fn a_break_inside_grace_is_offered_with_its_cost() {
        let repo = Arc::new(FakeStreakRepo::default());
        broken_user(&repo);
        let m = module(repo.clone());
        let st = m.standing("u1", ymd(2026, 8, 14)).await.unwrap();
        assert!(!st.played_today);
        assert_eq!(
            st.recover,
            RecoverDecision::Allow {
                restored: 7,
                cost: 30
            }
        );
        // Reading the offer must NEVER move points (design: no silent debit).
        assert!(repo.debits().is_empty());
        assert_eq!(repo.spendable_points("u1").await.unwrap(), 100);
    }

    #[tokio::test]
    async fn confirming_the_recovery_debits_once_and_restores_the_run() {
        let repo = Arc::new(FakeStreakRepo::default());
        broken_user(&repo);
        let m = module(repo.clone());
        let st = m.recover("u1", ymd(2026, 8, 14)).await.unwrap();
        assert_eq!(st.state.current, 7);
        assert!(st.played_today);
        assert_eq!(repo.debits(), vec![("u1".to_string(), 30)]);
        assert_eq!(repo.spendable_points("u1").await.unwrap(), 70);
        // The restored streak continues normally the next day.
        let s = m
            .record_play("u1", noon(ymd(2026, 8, 15)), 0)
            .await
            .unwrap();
        assert_eq!(s.current, 8);
    }

    #[tokio::test]
    async fn recovering_twice_is_refused_the_second_time() {
        let repo = Arc::new(FakeStreakRepo::default());
        broken_user(&repo);
        let m = module(repo.clone());
        m.recover("u1", ymd(2026, 8, 14)).await.unwrap();
        // The streak is live again, so there is nothing left to buy.
        assert!(matches!(
            m.recover("u1", ymd(2026, 8, 14)).await,
            Err(AppError::FailedPrecondition(_))
        ));
        assert_eq!(repo.debits().len(), 1, "charged exactly once");
    }

    #[tokio::test]
    async fn an_unaffordable_recovery_writes_nothing() {
        let repo = Arc::new(FakeStreakRepo::default());
        broken_user(&repo);
        repo.seed_balance("u1", 29);
        let m = module(repo.clone());
        let err = m.recover("u1", ymd(2026, 8, 14)).await.unwrap_err();
        assert!(matches!(err, AppError::FailedPrecondition(_)));
        assert!(repo.debits().is_empty());
        // The streak is untouched — still broken, still 7 pending recovery.
        assert_eq!(
            repo.get("u1").await.unwrap().last_played,
            Some(ymd(2026, 8, 12))
        );
        assert_eq!(repo.spendable_points("u1").await.unwrap(), 29);
    }

    #[tokio::test]
    async fn past_the_grace_window_there_is_no_offer_and_no_recovery() {
        let repo = Arc::new(FakeStreakRepo::default());
        broken_user(&repo);
        let m = module(repo.clone());
        let st = m.standing("u1", ymd(2026, 8, 15)).await.unwrap();
        assert_eq!(st.recover, RecoverDecision::GraceElapsed);
        assert!(matches!(
            m.recover("u1", ymd(2026, 8, 15)).await,
            Err(AppError::FailedPrecondition(_))
        ));
        assert!(repo.debits().is_empty());
    }

    #[tokio::test]
    async fn an_intact_streak_cannot_be_recovered() {
        let repo = Arc::new(FakeStreakRepo::default());
        broken_user(&repo);
        let m = module(repo.clone());
        assert!(matches!(
            m.recover("u1", ymd(2026, 8, 12)).await,
            Err(AppError::FailedPrecondition(_))
        ));
        assert!(repo.debits().is_empty());
    }

    #[tokio::test]
    async fn the_freeze_cost_and_grace_window_are_configurable() {
        let repo = Arc::new(FakeStreakRepo::default());
        broken_user(&repo);
        let m = module(repo.clone()).with_config(StreakConfig {
            freeze_cost: 5,
            grace_days: 3,
        });
        // Still on offer three missed days later, at the configured price.
        let st = m.standing("u1", ymd(2026, 8, 16)).await.unwrap();
        assert_eq!(
            st.recover,
            RecoverDecision::Allow {
                restored: 7,
                cost: 5
            }
        );
        m.recover("u1", ymd(2026, 8, 16)).await.unwrap();
        assert_eq!(repo.debits(), vec![("u1".to_string(), 5)]);
    }

    /// A config source whose values can change between calls (what the flag
    /// service does when an operator edits the back office).
    #[derive(Default)]
    struct MutableConfig(Mutex<StreakConfig>);

    impl StreakConfigSource for MutableConfig {
        fn streak_config(&self) -> StreakConfig {
            *self.0.lock().unwrap()
        }
    }

    #[tokio::test]
    async fn a_config_source_is_re_read_on_every_call() {
        let repo = Arc::new(FakeStreakRepo::default());
        broken_user(&repo);
        let cfg = Arc::new(MutableConfig(Mutex::new(StreakConfig {
            freeze_cost: 30,
            grace_days: 1,
        })));
        let m = module(repo.clone()).with_config_source(cfg.clone());
        assert_eq!(
            m.standing("u1", ymd(2026, 8, 14)).await.unwrap().recover,
            RecoverDecision::Allow {
                restored: 7,
                cost: 30
            }
        );
        // An operator retunes the price in the back office: the very next offer
        // uses it, with no restart.
        *cfg.0.lock().unwrap() = StreakConfig {
            freeze_cost: 200,
            grace_days: 1,
        };
        assert_eq!(
            m.standing("u1", ymd(2026, 8, 14)).await.unwrap().recover,
            RecoverDecision::InsufficientPoints {
                needed: 200,
                available: 100
            }
        );
    }

    #[tokio::test]
    async fn reminder_candidates_exclude_users_who_already_played() {
        let repo = Arc::new(FakeStreakRepo::default());
        let m = module(repo.clone());
        // 2026-08-12T22:30Z → the 13th in Paris, still the 12th in New York.
        let now: DateTime<Utc> = "2026-08-12T22:30:00Z".parse().unwrap();
        for user in ["paris", "ny"] {
            repo.seed(
                user,
                StreakState {
                    current: 4,
                    longest: 4,
                    last_played: Some(ymd(2026, 8, 12)),
                },
            );
        }
        repo.seed_timezone("paris", "Europe/Paris");
        repo.seed_timezone("ny", "America/New_York");
        // ...plus somebody with no streak at all.
        repo.seed("idle", StreakState::default());
        assert_eq!(
            m.reminder_candidates(now, "Europe/Paris").await.unwrap(),
            vec!["paris".to_string()]
        );
    }

    #[tokio::test]
    async fn longest_streak_is_the_badge_counter() {
        let repo = Arc::new(FakeStreakRepo::default());
        repo.seed(
            "u1",
            StreakState {
                current: 1,
                longest: 42,
                last_played: Some(ymd(2026, 8, 12)),
            },
        );
        let m = module(repo);
        assert_eq!(m.longest_streak("u1").await.unwrap(), 42);
        // An unknown user has no streak, not an error.
        assert_eq!(m.longest_streak("nobody").await.unwrap(), 0);
    }
}
