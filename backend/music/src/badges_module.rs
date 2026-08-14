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

//! Badge awarding + projection (change: add-achievement-badges).
//!
//! [`BadgesModule`] composes the pure registry ([`crate::badges_core`]) over the
//! storage port ([`BadgeRepo`]) into the one behaviour the feature needs:
//! **grant-on-read** (design D2). A badge read fetches every counter once,
//! evaluates the whole registry, inserts a grant for each newly-due badge, and
//! projects the result.
//!
//! That is deliberately the only awarding path. Hooking every domain ingest (a
//! session recorded, a best raised, a proposal accepted) would spread badge
//! knowledge across five modules and still give no retroactivity for a badge
//! defined later; a periodic job would need an all-users scan and delay the grant
//! for no gain, since the only consumer of a grant is the screen that would have
//! triggered it. The cost — a user who never opens the screen holds no grant row —
//! is harmless because `earned` is the union of granted and over-threshold
//! (design D3), so the badge is simply awarded the first time they look.

use std::sync::Arc;

use cymbra_platform::Result;

use crate::badges::BadgeRepo;
use crate::badges_core::{BadgeStanding, evaluate, newly_due};

/// The badge service over the counter + grant repo.
pub struct BadgesModule {
    repo: Arc<dyn BadgeRepo>,
}

impl BadgesModule {
    pub fn new(repo: Arc<dyn BadgeRepo>) -> Self {
        Self { repo }
    }

    /// The signed-in user's standing on **every** badge in the registry, earned or
    /// not, in registry order — granting anything newly due first so the surface
    /// is always current and a badge defined since the last read is awarded
    /// retroactively with no backfill.
    ///
    /// Idempotent: the grant insert is `DO NOTHING` on the (user, key) primary
    /// key, so a repeated evaluation neither creates a second row nor moves the
    /// recorded moment. The grants are re-read only when something was actually
    /// inserted, so a steady-state read costs exactly two queries.
    pub async fn achievements(&self, user_id: &str) -> Result<Vec<BadgeStanding>> {
        let counters = self.repo.counters(user_id).await?.fold();
        let granted = self.repo.granted_badges(user_id).await?;
        let standings = evaluate(&counters, &granted);

        let due = newly_due(&standings);
        if due.is_empty() {
            return Ok(standings);
        }
        for key in due {
            self.repo.insert_grant(user_id, key).await?;
        }
        // Re-read so the freshly granted badges report the moment the store
        // recorded, rather than a clock this module would have to guess at.
        let granted = self.repo.granted_badges(user_id).await?;
        Ok(evaluate(&counters, &granted))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::badges::{MockBadgeRepo, RawBadgeCounters};
    use crate::badges_core::{BadgeFamily, REGISTRY};
    use std::collections::HashMap;
    use std::sync::Mutex;

    const U: &str = "u1";

    fn standing<'a>(all: &'a [BadgeStanding], key: &str) -> &'a BadgeStanding {
        all.iter()
            .find(|s| s.def.key == key)
            .unwrap_or_else(|| panic!("no badge {key}"))
    }

    /// A `MockBadgeRepo` whose grant store is a real (shared) map, so the
    /// insert-once semantics of `curation_grants` are exercised rather than
    /// asserted away: `insert_grant` stamps a moment on the first call only.
    fn repo_with(counters: RawBadgeCounters, seed: &[(&str, i64)]) -> (Arc<MockBadgeRepo>, Grants) {
        let grants: Grants = Arc::new(Mutex::new(
            seed.iter().map(|(k, at)| (k.to_string(), *at)).collect(),
        ));
        let mut repo = MockBadgeRepo::new();
        repo.expect_counters()
            .returning(move |_| Ok(counters.clone()));
        let read = grants.clone();
        repo.expect_granted_badges()
            .returning(move |_| Ok(read.lock().unwrap().clone()));
        let write = grants.clone();
        repo.expect_insert_grant().returning(move |_, key| {
            let mut g = write.lock().unwrap();
            if g.contains_key(key) {
                return Ok(false); // ON CONFLICT DO NOTHING
            }
            // A distinct, stable moment per grant so a moved one is visible.
            let at = 1_700_000_000_000 + g.len() as i64;
            g.insert(key.to_string(), at);
            Ok(true)
        });
        (Arc::new(repo), grants)
    }

    type Grants = Arc<Mutex<HashMap<String, i64>>>;

    fn module(repo: Arc<MockBadgeRepo>) -> BadgesModule {
        BadgesModule::new(repo)
    }

    #[tokio::test]
    async fn a_fresh_account_earns_nothing_and_grants_nothing() {
        let (repo, grants) = repo_with(RawBadgeCounters::default(), &[]);
        let all = module(repo).achievements(U).await.unwrap();
        // The whole registry is still projected — the grid shows every locked badge.
        assert_eq!(all.len(), REGISTRY.len());
        assert!(all.iter().all(|s| !s.earned));
        assert!(grants.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn crossing_a_threshold_grants_on_read_with_a_moment() {
        let (repo, grants) = repo_with(
            RawBadgeCounters {
                session_count: 25,
                ..Default::default()
            },
            &[],
        );
        let all = module(repo).achievements(U).await.unwrap();
        let s = standing(&all, "performer_1");
        assert!(s.earned);
        // The re-read means the freshly granted badge carries its date immediately.
        assert!(s.granted_at_ms.is_some());
        assert_eq!(s.value, 25);
        assert!(grants.lock().unwrap().contains_key("performer_1"));
        // `first_performance` (1 session) came along in the same pass.
        assert!(standing(&all, "first_performance").earned);
    }

    #[tokio::test]
    async fn re_evaluation_is_idempotent_and_does_not_move_the_moment() {
        let (repo, grants) = repo_with(
            RawBadgeCounters {
                courses_completed: 10,
                ..Default::default()
            },
            &[],
        );
        let m = module(repo);
        let first = m.achievements(U).await.unwrap();
        let granted_at = standing(&first, "student_2").granted_at_ms;
        assert!(granted_at.is_some());
        let count_after_first = grants.lock().unwrap().len();

        // Two more reads: no second row, no new moment.
        let second = m.achievements(U).await.unwrap();
        let third = m.achievements(U).await.unwrap();
        assert_eq!(grants.lock().unwrap().len(), count_after_first);
        assert_eq!(standing(&second, "student_2").granted_at_ms, granted_at);
        assert_eq!(standing(&third, "student_2").granted_at_ms, granted_at);
    }

    #[tokio::test]
    async fn a_badge_the_user_already_qualifies_for_is_awarded_retroactively() {
        // The registry entry is new to them; they hold no grant row but have long
        // satisfied it. No backfill runs — the next read awards it.
        let (repo, grants) = repo_with(
            RawBadgeCounters {
                accepted_proposals: 12,
                accepted_soundfonts: 1,
                ..Default::default()
            },
            &[],
        );
        let all = module(repo).achievements(U).await.unwrap();
        assert!(standing(&all, "publisher_1").earned);
        assert!(standing(&all, "publisher_2").earned);
        assert!(standing(&all, "sound_donor").earned);
        let g = grants.lock().unwrap();
        assert!(g.contains_key("publisher_1"));
        assert!(g.contains_key("publisher_2"));
        assert!(g.contains_key("sound_donor"));
    }

    #[tokio::test]
    async fn an_earned_badge_survives_its_counter_dropping_to_zero() {
        // The catalog piece was purged and the board rows cascaded away; the badge
        // was granted long ago and stays earned, with its original moment.
        let (repo, _) = repo_with(
            RawBadgeCounters::default(),
            &[("podium_1", 1_690_000_000_000)],
        );
        let all = module(repo).achievements(U).await.unwrap();
        let s = standing(&all, "podium_1");
        assert!(s.earned);
        assert_eq!(s.granted_at_ms, Some(1_690_000_000_000));
        // ...and it reports full progress rather than 0/1.
        assert_eq!(s.value, s.def.threshold);
    }

    #[tokio::test]
    async fn the_consistency_fold_runs_over_the_repo_s_local_days() {
        use chrono::NaiveDate;
        let day = |d: u32| NaiveDate::from_ymd_opt(2026, 4, d).unwrap();
        let (repo, _) = repo_with(
            RawBadgeCounters {
                // 9 distinct days (the 3rd is played twice), longest run 4 (11→14).
                play_days: vec![
                    day(1),
                    day(3),
                    day(3),
                    day(11),
                    day(12),
                    day(13),
                    day(14),
                    day(20),
                    day(21),
                    day(28),
                ],
                ..Default::default()
            },
            &[],
        );
        let all = module(repo).achievements(U).await.unwrap();
        // 9 days < 10 → `regular_1` still locked, but its progress reads 9 (the
        // day played twice counts once).
        let regular = standing(&all, "regular_1");
        assert!(!regular.earned);
        assert_eq!(regular.value, 9);
        // A run of 4 clears `streak_1` (3) but not `streak_2` (7).
        assert!(standing(&all, "streak_1").earned);
        assert!(!standing(&all, "streak_2").earned);
        assert_eq!(standing(&all, "streak_2").value, 4);
    }

    #[tokio::test]
    async fn a_player_who_only_drills_measure_ranges_still_builds_a_streak() {
        use chrono::NaiveDate;
        let day = |d: u32| NaiveDate::from_ymd_opt(2026, 4, d).unwrap();
        // Never finishes a scored run — only loops over a measure range, every
        // day for a week.
        let (repo, _) = repo_with(
            RawBadgeCounters {
                practice_days: (1..=7).map(day).collect(),
                ..Default::default()
            },
            &[],
        );
        let all = module(repo).achievements(U).await.unwrap();
        // Consistency sees the work: 7 days, a run of 7.
        assert!(standing(&all, "streak_1").earned); // 3 in a row
        assert!(standing(&all, "streak_2").earned); // 7 in a row
        assert_eq!(standing(&all, "regular_1").value, 7); // 7 of 10 days
        // Play does NOT: a selective run has no sub-score to claim.
        assert!(!standing(&all, "first_performance").earned);
        assert_eq!(standing(&all, "performer_1").value, 0);
        assert_eq!(standing(&all, "virtuoso_1").value, 0);
    }

    #[tokio::test]
    async fn curation_grants_written_before_this_change_are_re_read_unchanged() {
        // The migration contract: rows keyed by the seven original badge keys keep
        // their earned state and their date, with no counter behind them any more.
        let seeded: Vec<(&str, i64)> = vec![
            ("first_note", 1_600_000_000_000),
            ("curator_1", 1_600_000_000_001),
            ("sharp_ear_1", 1_600_000_000_002),
        ];
        let (repo, _) = repo_with(RawBadgeCounters::default(), &seeded);
        let all = module(repo).achievements(U).await.unwrap();
        for (key, at) in seeded {
            let s = standing(&all, key);
            assert!(s.earned, "{key} lost");
            assert_eq!(s.granted_at_ms, Some(at), "{key} moment moved");
        }
        // The curation badges they had NOT earned stay locked.
        assert!(!standing(&all, "curator_3").earned);
    }

    #[tokio::test]
    async fn every_family_is_projected_so_the_grid_can_group() {
        let (repo, _) = repo_with(RawBadgeCounters::default(), &[]);
        let all = module(repo).achievements(U).await.unwrap();
        for f in [
            BadgeFamily::Play,
            BadgeFamily::Consistency,
            BadgeFamily::Ranking,
            BadgeFamily::Contribution,
            BadgeFamily::Curation,
            BadgeFamily::Learning,
        ] {
            assert!(
                all.iter().any(|s| s.def.family == f),
                "{} absent from the projection",
                f.as_str()
            );
        }
    }
}
