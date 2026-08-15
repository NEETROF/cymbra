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

//! Pure rules of the catalog daily-access gate (change: add-score-daily-access-
//! rewards, design D2) — the freemium quota on catalog player-opens.
//!
//! No I/O, no clock: every function takes the day's state and the configuration
//! in force and returns a decision, so each branch is unit-tested here without a
//! database. The module ([`crate::catalog_daily_access::CatalogDailyAccess`])
//! reads the state, calls in, and writes the outcome; the Postgres adapter is
//! I/O glue.
//!
//! The day is the **server (UTC) day** — the clock the play-award daily cap uses,
//! and for the same reason: a client-offset day would hand out N more free opens
//! per device-clock change (design D1). The quota is a product limit, not a
//! security boundary: the egress cap stays `catalog-access-limits`.

use std::collections::BTreeSet;

use chrono::{DateTime, NaiveDate, Utc};

/// The gate's configuration in force for one decision (read per call from the
/// feature flags through [`crate::catalog_daily_access::DailyAccessConfigSource`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DailyAccessConfig {
    /// Kill-switch. `false` = no gate at all: every open serves as before.
    pub enabled: bool,
    /// Distinct catalog pieces a user may open for free per server day.
    pub free_quota: u32,
    /// Points one extra piece for the day costs. `0` = a free unlock (no
    /// effective gate — the app can still show the flow, nothing is charged).
    pub day_slot_cost: i64,
}

impl Default for DailyAccessConfig {
    /// The design's starting values (mirrors the flag registry defaults).
    fn default() -> Self {
        Self {
            enabled: false,
            free_quota: 3,
            day_slot_cost: 20,
        }
    }
}

/// A user's access state for one server day.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DayState {
    /// Every catalog piece open for the user today (free opens AND paid slots).
    pub opened: BTreeSet<String>,
    /// The subset bought with points (day-slots).
    pub paid: BTreeSet<String>,
}

impl DayState {
    /// How many of today's free slots are used: opened pieces that were not paid
    /// for. Derived, so there is no counter to keep in sync with the set.
    pub fn free_used(&self) -> u32 {
        self.opened.difference(&self.paid).count() as u32
    }
}

/// Who is opening the piece — decides whether the quota applies at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallerKind {
    /// A normal signed-in player: the quota applies.
    Regular,
    /// Back-office audience or music-scope moderator/admin — reviewing is work,
    /// not consumption. Never quota'd, never counted.
    Exempt,
    /// `has_active_subscription` — unlimited opens (billing seam, design D6).
    Subscriber,
    /// The proposer of an accepted user-proposed piece opening their own
    /// contribution from the catalog: never pays for it.
    Contributor,
}

/// The gate's answer to one open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Open {
    /// Serve without touching the quota: the gate is off, the caller is outside
    /// it, or the piece is already open today (a re-open is free).
    Serve,
    /// Serve and consume one free slot (record the piece into today's set).
    ServeFree,
    /// Refuse the MusicXML: the free quota is used up and the piece is not open
    /// today. `cost` is the day-slot price; `upsell` marks the moment the future
    /// subscription offer would be shown.
    Locked { cost: i64, upsell: bool },
}

/// Decide one catalog player-open (design D2).
///
/// ```text
/// !cfg.enabled | caller ∈ {Exempt, Subscriber, Contributor} -> Serve
/// piece ∈ day.opened                                        -> Serve       (re-open, free)
/// day.free_used() < cfg.free_quota                          -> ServeFree   (consume a slot)
/// else                                                      -> Locked
/// ```
pub fn decide_open(
    piece: &str,
    day: &DayState,
    cfg: &DailyAccessConfig,
    caller: CallerKind,
) -> Open {
    if !cfg.enabled || caller != CallerKind::Regular {
        return Open::Serve;
    }
    if day.opened.contains(piece) {
        return Open::Serve;
    }
    if day.free_used() < cfg.free_quota {
        return Open::ServeFree;
    }
    Open::Locked {
        cost: cfg.day_slot_cost,
        upsell: true,
    }
}

/// Whether recording a served open should write a day row: only a `ServeFree`
/// consumes a slot; a plain `Serve` is either outside the gate or already
/// recorded. (Kept as a function so the module never re-derives the rule.)
pub fn records_open(open: Open) -> bool {
    matches!(open, Open::ServeFree)
}

/// The day-slot purchase decision (design D4), taken before the atomic spend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaySlot {
    /// Nothing to buy: the gate is off / the caller is outside it / the piece is
    /// already open today. Report success without a debit.
    AlreadyOpen,
    /// Charge `cost` under the day-slot key, then record the paid row.
    Charge { cost: i64 },
    /// The spendable balance does not cover the cost: refuse, write nothing.
    Insufficient { cost: i64, balance: i64 },
}

/// Decide a day-slot purchase for `piece`.
pub fn day_slot_decision(
    piece: &str,
    day: &DayState,
    cfg: &DailyAccessConfig,
    caller: CallerKind,
    balance: i64,
) -> DaySlot {
    if !cfg.enabled || caller != CallerKind::Regular || day.opened.contains(piece) {
        return DaySlot::AlreadyOpen;
    }
    let cost = cfg.day_slot_cost.max(0);
    if balance < cost {
        return DaySlot::Insufficient { cost, balance };
    }
    DaySlot::Charge { cost }
}

/// The ledger `reward_key` a day-slot spend is recorded under, so it is
/// identifiable in the activity feed (never confused with a shop redemption or a
/// streak freeze).
pub const DAY_SLOT_REWARD_KEY: &str = "score_day_slot";

/// The ledger **idempotency key** for a day-slot on `piece` for `day` — the same
/// `<kind>:<discriminator>` shape the play awards and the streak freeze use. Two
/// confirmations for the same piece and day therefore charge once: the unique
/// index `curation_points_award_key_once_idx` is the guard, not the read-then-
/// write decision.
pub fn day_slot_award_key(piece: &str, day: NaiveDate) -> String {
    format!("{DAY_SLOT_REWARD_KEY}:{piece}:{day}")
}

/// The server day of `now` (UTC).
pub fn server_day(now: DateTime<Utc>) -> NaiveDate {
    now.date_naive()
}

/// The instant the day containing `now` rolls over (next UTC midnight), in epoch
/// milliseconds — what the app counts down to.
pub fn resets_at_ms(now: DateTime<Utc>) -> i64 {
    let next = server_day(now)
        .succ_opt()
        .expect("date within range")
        .and_hms_opt(0, 0, 0)
        .expect("midnight")
        .and_utc();
    next.timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn cfg(enabled: bool, quota: u32, cost: i64) -> DailyAccessConfig {
        DailyAccessConfig {
            enabled,
            free_quota: quota,
            day_slot_cost: cost,
        }
    }

    fn day(opened: &[&str], paid: &[&str]) -> DayState {
        DayState {
            opened: opened.iter().map(|s| s.to_string()).collect(),
            paid: paid.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn gate_off_serves_everything() {
        let d = day(&["a", "b", "c"], &[]);
        assert_eq!(
            decide_open("z", &d, &cfg(false, 0, 20), CallerKind::Regular),
            Open::Serve
        );
    }

    #[test]
    fn exempt_subscriber_and_contributor_bypass_the_quota() {
        let d = day(&["a", "b", "c"], &[]);
        for k in [
            CallerKind::Exempt,
            CallerKind::Subscriber,
            CallerKind::Contributor,
        ] {
            assert_eq!(decide_open("z", &d, &cfg(true, 3, 20), k), Open::Serve);
        }
    }

    #[test]
    fn first_opens_are_free_up_to_the_quota() {
        let c = cfg(true, 3, 20);
        assert_eq!(
            decide_open("a", &day(&[], &[]), &c, CallerKind::Regular),
            Open::ServeFree
        );
        assert_eq!(
            decide_open("c", &day(&["a", "b"], &[]), &c, CallerKind::Regular),
            Open::ServeFree
        );
        assert_eq!(
            decide_open("d", &day(&["a", "b", "c"], &[]), &c, CallerKind::Regular),
            Open::Locked {
                cost: 20,
                upsell: true
            }
        );
    }

    #[test]
    fn re_opening_a_piece_already_open_today_is_free() {
        let d = day(&["a", "b", "c"], &[]);
        assert_eq!(
            decide_open("b", &d, &cfg(true, 3, 20), CallerKind::Regular),
            Open::Serve
        );
    }

    #[test]
    fn paid_slots_do_not_consume_free_quota() {
        // 3 free + 1 paid open today: a 4th free open still fits? No — quota is 3
        // free; the paid one is not counted against it, so free_used = 3.
        let d = day(&["a", "b", "c", "p"], &["p"]);
        assert_eq!(d.free_used(), 3);
        // A 2-free-quota day with one paid: one free slot left.
        let d2 = day(&["a", "p"], &["p"]);
        assert_eq!(d2.free_used(), 1);
        assert_eq!(
            decide_open("z", &d2, &cfg(true, 2, 20), CallerKind::Regular),
            Open::ServeFree
        );
    }

    #[test]
    fn a_paid_piece_reopens_free_the_same_day() {
        let d = day(&["a", "b", "c", "p"], &["p"]);
        assert_eq!(
            decide_open("p", &d, &cfg(true, 3, 20), CallerKind::Regular),
            Open::Serve
        );
    }

    #[test]
    fn next_day_is_an_empty_state_so_a_paid_piece_costs_again() {
        // The next day's state is simply empty (the repo reads by day).
        let tomorrow = day(&[], &[]);
        let c = cfg(true, 0, 20);
        assert_eq!(
            decide_open("p", &tomorrow, &c, CallerKind::Regular),
            Open::Locked {
                cost: 20,
                upsell: true
            }
        );
    }

    #[test]
    fn quota_zero_locks_every_new_piece() {
        assert_eq!(
            decide_open("a", &day(&[], &[]), &cfg(true, 0, 5), CallerKind::Regular),
            Open::Locked {
                cost: 5,
                upsell: true
            }
        );
    }

    #[test]
    fn only_a_free_serve_records_a_new_row() {
        assert!(records_open(Open::ServeFree));
        assert!(!records_open(Open::Serve));
        assert!(!records_open(Open::Locked {
            cost: 1,
            upsell: true
        }));
    }

    #[test]
    fn day_slot_charges_when_affordable() {
        let d = day(&["a", "b", "c"], &[]);
        assert_eq!(
            day_slot_decision("z", &d, &cfg(true, 3, 20), CallerKind::Regular, 20),
            DaySlot::Charge { cost: 20 }
        );
    }

    #[test]
    fn day_slot_refuses_when_insufficient() {
        let d = day(&["a", "b", "c"], &[]);
        assert_eq!(
            day_slot_decision("z", &d, &cfg(true, 3, 20), CallerKind::Regular, 19),
            DaySlot::Insufficient {
                cost: 20,
                balance: 19
            }
        );
    }

    #[test]
    fn day_slot_is_a_no_op_when_already_open_or_outside_the_gate() {
        let d = day(&["z"], &[]);
        assert_eq!(
            day_slot_decision("z", &d, &cfg(true, 3, 20), CallerKind::Regular, 0),
            DaySlot::AlreadyOpen
        );
        assert_eq!(
            day_slot_decision("q", &d, &cfg(false, 3, 20), CallerKind::Regular, 0),
            DaySlot::AlreadyOpen
        );
        assert_eq!(
            day_slot_decision("q", &d, &cfg(true, 3, 20), CallerKind::Exempt, 0),
            DaySlot::AlreadyOpen
        );
    }

    #[test]
    fn zero_cost_day_slot_charges_zero() {
        assert_eq!(
            day_slot_decision(
                "q",
                &day(&[], &[]),
                &cfg(true, 0, 0),
                CallerKind::Regular,
                0
            ),
            DaySlot::Charge { cost: 0 }
        );
    }

    #[test]
    fn award_key_names_piece_and_day() {
        let d = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
        assert_eq!(
            day_slot_award_key("11111111-1111-7111-8111-111111111111", d),
            "score_day_slot:11111111-1111-7111-8111-111111111111:2026-08-15"
        );
    }

    #[test]
    fn server_day_and_reset_are_utc() {
        let now = Utc.with_ymd_and_hms(2026, 8, 15, 23, 30, 0).unwrap();
        assert_eq!(
            server_day(now),
            NaiveDate::from_ymd_opt(2026, 8, 15).unwrap()
        );
        let reset = Utc.with_ymd_and_hms(2026, 8, 16, 0, 0, 0).unwrap();
        assert_eq!(resets_at_ms(now), reset.timestamp_millis());
    }
}
