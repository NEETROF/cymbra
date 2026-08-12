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

//! The badge storage port (change: add-achievement-badges).
//!
//! [`BadgeRepo`] is deliberately narrow — **one** call gathers every counter a
//! badge read needs (design D4), instead of one query per badge — plus the two
//! grant operations. The grant store is the SAME `music.curation_grants`
//! (`grant_kind = 'badge'`) the curation rewards introduced, so every badge
//! already earned keeps working with no data migration.
//!
//! The raw read hands the consistency counters over as a **list of local days**
//! rather than pre-aggregated numbers: distinct-days and longest-consecutive-run
//! are timezone-sensitive rules that belong with the badge semantics, so
//! [`RawBadgeCounters::fold`] computes them through the pure
//! [`crate::badges_core`] helpers instead of a SQL window function.
//!
//! The trait is `#[automock]`ed (the rust-testing default), so the module tests
//! drive a `MockBadgeRepo` and never touch a database.

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::NaiveDate;
use cymbra_platform::Result;

use crate::badges_core::{BadgeCounters, distinct_days, longest_streak};

/// Every counter for one user, as the repo reads them: scalar aggregates plus the
/// raw local-day list the consistency fold needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RawBadgeCounters {
    // curation — the same three counters `CuratorMetrics` already exposes.
    pub rating_count: i64,
    pub aligned_count: i64,
    pub first_rater_count: i64,
    // play
    pub session_count: i64,
    pub distinct_pieces: i64,
    pub high_accuracy_sessions: i64,
    /// One entry per recorded session: the player's LOCAL day for it (`played_at`
    /// shifted by the `tz_offset_minutes` captured with the session — the same
    /// bucketing the activity heatmap uses). Unordered, duplicates expected.
    pub play_days: Vec<NaiveDate>,
    // ranking
    pub ranked_boards: i64,
    pub top_three_finishes: i64,
    pub season_podiums: i64,
    // contribution
    pub accepted_proposals: i64,
    pub accepted_soundfonts: i64,
    // learning
    pub courses_completed: i64,
}

impl RawBadgeCounters {
    /// Fold the raw read into the metric-addressable counters the registry is
    /// evaluated against, computing the two consistency values from
    /// [`Self::play_days`].
    pub fn fold(&self) -> BadgeCounters {
        BadgeCounters {
            rating_count: self.rating_count,
            aligned_count: self.aligned_count,
            first_rater_count: self.first_rater_count,
            session_count: self.session_count,
            distinct_pieces: self.distinct_pieces,
            high_accuracy_sessions: self.high_accuracy_sessions,
            days_played: distinct_days(&self.play_days),
            longest_streak: longest_streak(&self.play_days),
            ranked_boards: self.ranked_boards,
            top_three_finishes: self.top_three_finishes,
            season_podiums: self.season_podiums,
            accepted_proposals: self.accepted_proposals,
            accepted_soundfonts: self.accepted_soundfonts,
            courses_completed: self.courses_completed,
        }
    }
}

/// Storage port for the badge registry. Keyed by the plain `user_id` string, like
/// every other `music` port (no cross-schema FK). Implemented by
/// [`crate::PgBadgeRepo`] (coverage-excluded I/O glue) and mocked in tests.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BadgeRepo: Send + Sync {
    /// EVERY counter for `user_id`, in one call (design D4).
    async fn counters(&self, user_id: &str) -> Result<RawBadgeCounters>;

    /// The user's badge grants: `key` → when it was granted, in unix millis. This
    /// is the durable memory of the *date*; whether a badge is earned is the union
    /// of this and the live counters (design D3).
    async fn granted_badges(&self, user_id: &str) -> Result<HashMap<String, i64>>;

    /// Record a badge grant once — `ON CONFLICT (user_id, key) DO NOTHING` against
    /// the `curation_grants` primary key. Returns `true` iff this call inserted
    /// it, which is the idempotency guard: a repeated evaluation neither creates a
    /// second row nor moves the recorded moment.
    async fn insert_grant(&self, user_id: &str, key: &str) -> Result<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn fold_derives_the_consistency_counters_and_passes_the_rest_through() {
        let raw = RawBadgeCounters {
            rating_count: 3,
            aligned_count: 2,
            first_rater_count: 1,
            session_count: 6,
            distinct_pieces: 4,
            high_accuracy_sessions: 2,
            // Two days played, one of them twice → 2 distinct days, run of 2.
            play_days: vec![
                ymd(2026, 5, 2),
                ymd(2026, 5, 1),
                ymd(2026, 5, 2),
                ymd(2026, 5, 1),
                ymd(2026, 5, 2),
                ymd(2026, 5, 1),
            ],
            ranked_boards: 7,
            top_three_finishes: 5,
            season_podiums: 1,
            accepted_proposals: 8,
            accepted_soundfonts: 9,
            courses_completed: 10,
        };
        let c = raw.fold();
        assert_eq!(c.days_played, 2);
        assert_eq!(c.longest_streak, 2);
        // Everything else is carried across unchanged.
        assert_eq!(c.rating_count, 3);
        assert_eq!(c.aligned_count, 2);
        assert_eq!(c.first_rater_count, 1);
        assert_eq!(c.session_count, 6);
        assert_eq!(c.distinct_pieces, 4);
        assert_eq!(c.high_accuracy_sessions, 2);
        assert_eq!(c.ranked_boards, 7);
        assert_eq!(c.top_three_finishes, 5);
        assert_eq!(c.season_podiums, 1);
        assert_eq!(c.accepted_proposals, 8);
        assert_eq!(c.accepted_soundfonts, 9);
        assert_eq!(c.courses_completed, 10);
    }

    #[test]
    fn folding_an_empty_read_is_all_zeroes() {
        assert_eq!(RawBadgeCounters::default().fold(), BadgeCounters::default());
    }
}
