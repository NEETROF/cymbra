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

//! GLOBAL leaderboard orchestration (change: add-global-leaderboard) over a
//! [`GlobalLeaderboardRepo`] and the [`UserPort`] visibility gate — the three
//! sides of the feature:
//!
//! * **Ingest** ([`GlobalSeasonSink`]): chained off #6's per-piece ingest hook, so
//!   it sees exactly the candidates that already passed the accepted-catalog and
//!   integrity checks. Each is monotonically upserted as the player's season best
//!   for the season it was *achieved* in — idempotent under at-least-once ingest.
//! * **Read** ([`GlobalLeaderboardModule::get_board`]): rank one `(mode, season)`,
//!   always returning the caller their **own** global rank + score among the public
//!   entries, even when the caller is private or under-age. Viewing itself is open
//!   to any authenticated user. No sensitive fields leave here. Which listing gate
//!   applies depends on the season:
//!   * a **live** season is gated on the CURRENT profile — public AND age-eligible
//!     (the #5 gate, fail-closed);
//!   * an **archived** season replays the consent frozen when it closed and
//!     re-checks only the AGE safeguard live, so a past ranking stays stable when a
//!     player goes private afterwards, while anyone who must not be listed at all
//!     still disappears from it.
//! * **Rollover** ([`GlobalLeaderboardModule::run_season_snapshot`]): freezes a
//!   closed season's final standings — and each player's consent to be listed — into
//!   the hall of fame. Idempotent, so the scheduled worker job is safe under
//!   at-least-once delivery.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cymbra_platform::Result;
use cymbra_user_port::{PlayerProfile, UserPort};

use crate::global_leaderboard::{
    GlobalLeaderboardRepo, GlobalScore, GlobalSeasonBest, GlobalSeasonSink,
};
use crate::global_leaderboard_core::{self, GlobalConfig};
use crate::leaderboard::{BestCandidate, Mode};

/// Server-side clamp on a global-board page size, so a client cannot ask for an
/// unbounded listing. A non-positive request falls back to [`DEFAULT_PAGE`].
const MAX_PAGE: i64 = 100;
const DEFAULT_PAGE: i64 = 50;

/// One ranked global-board entry (public players) or the caller's own standing.
/// Carries only NON-SENSITIVE fields — the public handle/display name, never an
/// email, curator alignment/reliability, or moderation state.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEntry {
    pub rank: i32,
    pub user_id: String,
    pub handle: Option<String>,
    pub display_name: Option<String>,
    /// The difficulty-weighted best-N global season score.
    pub score: f64,
    pub contributing_pieces: i32,
    pub reached_at_ms: i64,
}

/// A page of one global board plus the caller's own standing.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBoard {
    /// The season actually read (the current one when none was requested).
    pub season_id: String,
    /// The requested page of PUBLIC + eligible entries, in ranking order.
    pub entries: Vec<GlobalEntry>,
    /// Total number of public entries on this board (for paging).
    pub total: i32,
    /// The caller's own score + rank among the public entries, present whenever
    /// they scored this season — even if they are private/ineligible.
    pub own: Option<GlobalEntry>,
}

/// One page request into a board's public listing. A non-positive `limit` falls
/// back to [`DEFAULT_PAGE`]; anything above [`MAX_PAGE`] is clamped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    pub offset: i64,
    pub limit: i64,
}

/// The seasons a selector may offer: the live one plus every snapshotted past
/// season, most recent first.
#[derive(Debug, Clone, PartialEq)]
pub struct Seasons {
    pub current_season_id: String,
    pub past_season_ids: Vec<String>,
}

/// Global-board reads, the season-best ingest hook, and the end-of-season
/// snapshot, over the season stores and the user-port visibility gate.
pub struct GlobalLeaderboardModule {
    repo: Arc<dyn GlobalLeaderboardRepo>,
    user: Arc<dyn UserPort>,
    cfg: GlobalConfig,
}

impl GlobalLeaderboardModule {
    pub fn new(repo: Arc<dyn GlobalLeaderboardRepo>, user: Arc<dyn UserPort>) -> Self {
        Self {
            repo,
            user,
            cfg: GlobalConfig::default(),
        }
    }

    /// Override the tunable aggregation/season configuration (design D1/D3).
    pub fn with_config(mut self, cfg: GlobalConfig) -> Self {
        self.cfg = cfg;
        self
    }

    pub fn config(&self) -> &GlobalConfig {
        &self.cfg
    }

    /// Maintain the season bests from the candidates of one ingested session (the
    /// [`GlobalSeasonSink`] body, exposed directly for tests). Each candidate is
    /// bucketed into the season it was ACHIEVED in and monotonically upserted.
    pub async fn maintain_from_candidates(
        &self,
        user_id: &str,
        score_id: &str,
        candidates: &[BestCandidate],
    ) -> Result<()> {
        for candidate in candidates {
            let season = self.cfg.season_at(candidate.achieved_at_ms);
            self.repo
                .upsert_season_best(&GlobalSeasonBest {
                    user_id: user_id.to_string(),
                    season_id: season.id,
                    catalog_score_id: score_id.to_string(),
                    mode: candidate.mode,
                    subscore: candidate.subscore,
                    achieved_at_ms: candidate.achieved_at_ms,
                })
                .await?;
        }
        Ok(())
    }

    /// The seasons a selector may offer at `now_ms`: the live season plus the
    /// snapshotted past ones (most recent first).
    pub async fn seasons(&self, now_ms: i64) -> Result<Seasons> {
        let current = self.cfg.season_at(now_ms);
        let past: Vec<String> = self
            .repo
            .snapshotted_seasons()
            .await?
            .into_iter()
            .filter(|id| *id != current.id)
            .collect();
        Ok(Seasons {
            current_season_id: current.id,
            past_season_ids: past,
        })
    }

    /// Read one global board as seen by `viewer_id`: the requested page of public
    /// entries (private/ineligible players are never listed), plus the caller's own
    /// standing. `season_id` is `None` for the live season. `now` (UTC) both
    /// resolves the live season and drives the deterministic eligibility check.
    pub async fn get_board(
        &self,
        viewer_id: &str,
        mode: Mode,
        season_id: Option<&str>,
        page: Page,
        now: DateTime<Utc>,
    ) -> Result<GlobalBoard> {
        let Page { offset, limit } = page;
        let today = now.date_naive();
        let season = season_id
            .map(str::to_string)
            .unwrap_or_else(|| self.cfg.season_at(now.timestamp_millis()).id);
        let (all, frozen) = self.standings(&season, mode).await?;
        let ids: Vec<String> = all.iter().map(|s| s.user_id.clone()).collect();

        // Which gate applies depends on where the standings came from (design D5):
        //
        // * a LIVE season is gated on the CURRENT profile — public AND age-eligible;
        // * a FROZEN (archived) season replays the consent recorded when it closed,
        //   `was_listable`, and re-checks only the AGE safeguard live. That keeps a
        //   past ranking stable when a player goes private afterwards, while still
        //   removing anyone who must not be listed at all.
        //
        // Either way the map supplies the non-sensitive display fields.
        let profiles: HashMap<String, PlayerProfile> = if frozen {
            self.user.age_eligible_profiles(&ids, today).await?
        } else {
            self.user.listable_profiles(&ids, today).await?
        }
        .into_iter()
        .map(|p| (p.user_id.clone(), p))
        .collect();

        let public: Vec<&GlobalScore> = all
            .iter()
            .filter(|s| profiles.contains_key(&s.user_id) && (!frozen || s.was_listable))
            .collect();
        let total = public.len() as i32;

        // Page the public listing (clamped page size, offset past the end ⇒ empty).
        let page_size = if limit <= 0 {
            DEFAULT_PAGE
        } else {
            limit.min(MAX_PAGE)
        } as usize;
        let start = offset.max(0) as usize;
        let end = start.saturating_add(page_size).min(public.len());
        let entries: Vec<GlobalEntry> = if start >= public.len() {
            Vec::new()
        } else {
            public[start..end]
                .iter()
                .enumerate()
                .map(|(i, s)| entry((start + i) as i32 + 1, s, profiles.get(&s.user_id)))
                .collect()
        };

        // The caller's own standing — always, even when not publicly listed.
        let own = match all.iter().find(|s| s.user_id == viewer_id) {
            Some(mine) => {
                let public_owned: Vec<GlobalScore> = public.iter().map(|s| (*s).clone()).collect();
                let rank = global_leaderboard_core::own_rank(&public_owned, mine);
                // The owner always sees their own profile (whatever the visibility).
                let me = self
                    .user
                    .get_player_profile(viewer_id, viewer_id, today)
                    .await
                    .ok();
                Some(entry(rank, mine, me.as_ref()))
            }
            None => None,
        };

        Ok(GlobalBoard {
            season_id: season,
            entries,
            total,
            own,
        })
    }

    /// Freeze the season that has just CLOSED at `now` into the hall of fame
    /// (task 2.2) — see [`snapshot_closed_season`].
    pub async fn run_season_snapshot(&self, now: DateTime<Utc>) -> Result<u64> {
        snapshot_closed_season(self.repo.as_ref(), self.user.as_ref(), &self.cfg, now).await
    }

    /// The standings for one `(season, mode)` in ranking order, plus whether they
    /// came from a **frozen** snapshot: the archive when the season has one, else a
    /// live aggregation of its season bests (which also covers a closed-but-not-
    /// yet-snapshotted season).
    ///
    /// The flag matters because the two carry different consent: a live aggregation
    /// is gated on the CURRENT visibility, while a frozen one already recorded who
    /// had consented when the season closed.
    async fn standings(&self, season_id: &str, mode: Mode) -> Result<(Vec<GlobalScore>, bool)> {
        let snapshot = self.repo.snapshot_standings(season_id, mode).await?;
        if !snapshot.is_empty() {
            let mut ranked = snapshot;
            ranked.sort_by(global_leaderboard_core::rank_cmp);
            return Ok((ranked, true));
        }
        let rows = self.repo.season_bests(season_id, mode).await?;
        Ok((global_leaderboard_core::aggregate(&rows, &self.cfg), false))
    }
}

/// Freeze the season that has just CLOSED at `now_ms` into the hall of fame
/// (task 2.2, design D3). For each mode, aggregates that season's bests and writes
/// the final standings; a season already snapshotted is left untouched, so the
/// scheduled rollover job is **idempotent** under at-least-once delivery.
///
/// Per-piece all-time bests (#6) are never touched — only the per-season
/// accumulation rolls over, and the new season starts empty simply because its
/// bests are keyed by its own season id. Returns how many standings were newly
/// frozen.
///
/// A free function (rather than only a module method) so the `cymbra-worker` job
/// can run it with just a repo — the rollover needs no visibility gate, since the
/// snapshot stores raw aggregates and the gate is re-applied on read.
pub async fn snapshot_closed_season(
    repo: &dyn GlobalLeaderboardRepo,
    user: &dyn UserPort,
    cfg: &GlobalConfig,
    now: DateTime<Utc>,
) -> Result<u64> {
    let season = cfg.previous_season(now.timestamp_millis());
    let today = now.date_naive();
    let mut written = 0;
    for mode in [Mode::Tempo, Mode::Reaction] {
        // Already snapshotted ⇒ nothing to do (idempotent re-delivery).
        if !repo.snapshot_standings(&season.id, mode).await?.is_empty() {
            continue;
        }
        let rows = repo.season_bests(&season.id, mode).await?;
        let mut standings = global_leaderboard_core::aggregate(&rows, cfg);
        if standings.is_empty() {
            continue;
        }
        // FREEZE each player's consent to be listed as it stands now, at the close
        // of the season (design D5). This is what keeps the archive's ranking
        // stable when someone later goes private. The age safeguard is NOT frozen
        // with it — `get_board` re-checks it live on every archive read.
        let ids: Vec<String> = standings.iter().map(|s| s.user_id.clone()).collect();
        let listable: HashSet<String> = user
            .listable_profiles(&ids, today)
            .await?
            .into_iter()
            .map(|p| p.user_id)
            .collect();
        for s in &mut standings {
            s.was_listable = listable.contains(&s.user_id);
        }
        written += repo.write_snapshot(&season.id, mode, &standings).await?;
    }
    Ok(written)
}

/// Build a non-sensitive board entry from a standing + the profile fields the
/// user port allowed.
fn entry(rank: i32, s: &GlobalScore, profile: Option<&PlayerProfile>) -> GlobalEntry {
    GlobalEntry {
        rank,
        user_id: s.user_id.clone(),
        handle: profile.and_then(|p| p.handle.clone()),
        display_name: profile.and_then(|p| p.display_name.clone()),
        score: s.score,
        contributing_pieces: s.contributing_pieces,
        reached_at_ms: s.reached_at_ms,
    }
}

#[async_trait]
impl GlobalSeasonSink for GlobalLeaderboardModule {
    async fn ingest_candidates(
        &self,
        user_id: &str,
        score_id: &str,
        candidates: &[BestCandidate],
    ) -> Result<()> {
        self.maintain_from_candidates(user_id, score_id, candidates)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_leaderboard::FakeGlobalLeaderboardRepo;
    use cymbra_user_port::MockUserPort;

    /// The UTC instant `ms`, as the board read wants it.
    fn at(ms: i64) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(ms).expect("valid instant")
    }

    /// An instant inside season 0 of the default config.
    fn t0() -> i64 {
        GlobalConfig::default().season_anchor_ms + 1_000
    }

    /// An instant inside season 1 (one window later).
    fn t1() -> i64 {
        let cfg = GlobalConfig::default();
        cfg.season_anchor_ms + cfg.season_length_days * 86_400_000 + 1_000
    }

    fn candidate(mode: Mode, sub: f32, at: i64) -> BestCandidate {
        BestCandidate {
            mode,
            subscore: sub,
            tiebreak_metric: 5.0,
            achieved_at_ms: at,
        }
    }

    fn profile(id: &str) -> PlayerProfile {
        PlayerProfile {
            user_id: id.to_string(),
            handle: Some(format!("@{id}")),
            display_name: Some(id.to_string()),
            visibility: cymbra_user_port::Visibility::Public,
        }
    }

    /// A module whose user port lists exactly `public` (each with a handle) and
    /// resolves any own-profile self-read. Everyone is age-eligible.
    fn module(
        repo: Arc<FakeGlobalLeaderboardRepo>,
        public: &'static [&'static str],
    ) -> GlobalLeaderboardModule {
        module_with_age(repo, public, None)
    }

    /// As [`module`], but `age_ok` (when `Some`) restricts who passes the MINIMUM-AGE
    /// safeguard — the half of the gate that stays live even on a frozen season.
    fn module_with_age(
        repo: Arc<FakeGlobalLeaderboardRepo>,
        public: &'static [&'static str],
        age_ok: Option<&'static [&'static str]>,
    ) -> GlobalLeaderboardModule {
        let mut user = MockUserPort::new();
        user.expect_listable_profiles().returning(move |ids, _| {
            Ok(ids
                .iter()
                .filter(|id| public.contains(&id.as_str()))
                .map(|id| profile(id))
                .collect())
        });
        // Visibility-blind: returns a now-private player too, gated on age only.
        user.expect_age_eligible_profiles()
            .returning(move |ids, _| {
                Ok(ids
                    .iter()
                    .filter(|id| age_ok.is_none_or(|ok| ok.contains(&id.as_str())))
                    .map(|id| profile(id))
                    .collect())
            });
        user.expect_get_player_profile().returning(|_, target, _| {
            Ok(PlayerProfile {
                user_id: target.to_string(),
                handle: Some(format!("@{target}")),
                display_name: Some(target.to_string()),
                visibility: cymbra_user_port::Visibility::Private,
            })
        });
        GlobalLeaderboardModule::new(repo, Arc::new(user))
    }

    /// Record one season best for `user` on `piece` at `at`.
    async fn play(m: &GlobalLeaderboardModule, user: &str, piece: &str, sub: f32, at: i64) {
        m.maintain_from_candidates(user, piece, &[candidate(Mode::Tempo, sub, at)])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn ingest_buckets_by_season_and_is_monotonic() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let m = module(repo.clone(), &[]);
        let cfg = GlobalConfig::default();
        let s0 = cfg.season_at(t0()).id;
        let s1 = cfg.season_at(t1()).id;

        play(&m, "u", "p", 80.0, t0()).await;
        assert_eq!(
            repo.best_for("u", &s0, "p", Mode::Tempo).unwrap().subscore,
            80.0
        );
        // A WORSE in-season result never lowers the season best (monotonic).
        play(&m, "u", "p", 60.0, t0() + 5).await;
        assert_eq!(
            repo.best_for("u", &s0, "p", Mode::Tempo).unwrap().subscore,
            80.0
        );
        // An identical REPLAY is a no-op, not a duplicate (idempotent).
        play(&m, "u", "p", 80.0, t0()).await;
        assert_eq!(
            repo.best_for("u", &s0, "p", Mode::Tempo).unwrap().subscore,
            80.0
        );
        // A better one raises it.
        play(&m, "u", "p", 90.0, t0() + 9).await;
        assert_eq!(
            repo.best_for("u", &s0, "p", Mode::Tempo).unwrap().subscore,
            90.0
        );
        // The NEXT season is a separate row — season 0's best is untouched.
        play(&m, "u", "p", 55.0, t1()).await;
        assert_eq!(
            repo.best_for("u", &s1, "p", Mode::Tempo).unwrap().subscore,
            55.0
        );
        assert_eq!(
            repo.best_for("u", &s0, "p", Mode::Tempo).unwrap().subscore,
            90.0
        );
    }

    #[tokio::test]
    async fn ingest_feeds_both_modes_from_a_mixed_run() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let m = module(repo.clone(), &[]);
        let s0 = GlobalConfig::default().season_at(t0()).id;
        m.maintain_from_candidates(
            "u",
            "p",
            &[
                candidate(Mode::Tempo, 70.0, t0()),
                candidate(Mode::Reaction, 88.0, t0()),
            ],
        )
        .await
        .unwrap();
        assert_eq!(
            repo.best_for("u", &s0, "p", Mode::Tempo).unwrap().subscore,
            70.0
        );
        assert_eq!(
            repo.best_for("u", &s0, "p", Mode::Reaction)
                .unwrap()
                .subscore,
            88.0
        );
    }

    #[tokio::test]
    async fn board_lists_public_only_and_ranks_by_global_score() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        repo.set_level("easy", Some("beginner"));
        repo.set_level("hard", Some("advanced"));
        let m = module(repo.clone(), &["a1", "a2"]);
        // a1 plays an ADVANCED piece well; a2 the same score on a beginner one.
        play(&m, "a1", "hard", 80.0, t0()).await;
        play(&m, "a2", "easy", 80.0, t0()).await;
        play(&m, "priv", "hard", 99.0, t0()).await;

        let board = m
            .get_board(
                "a1",
                Mode::Tempo,
                None,
                Page {
                    offset: 0,
                    limit: 50,
                },
                at(t0()),
            )
            .await
            .unwrap();
        assert_eq!(board.season_id, GlobalConfig::default().season_at(t0()).id);
        assert_eq!(board.total, 2);
        // The harder piece outranks the same sub-score on an easy one.
        assert_eq!(board.entries[0].user_id, "a1");
        assert_eq!(board.entries[0].rank, 1);
        assert_eq!(board.entries[0].handle.as_deref(), Some("@a1"));
        assert_eq!(board.entries[1].user_id, "a2");
        // The private player is never listed to others.
        assert!(board.entries.iter().all(|e| e.user_id != "priv"));
    }

    #[tokio::test]
    async fn private_caller_sees_own_rank_but_is_not_listed() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let m = module(repo.clone(), &["a1", "a2"]);
        play(&m, "a1", "p1", 95.0, t0()).await;
        play(&m, "a2", "p2", 70.0, t0()).await;
        play(&m, "priv", "p3", 88.0, t0()).await;

        let board = m
            .get_board(
                "priv",
                Mode::Tempo,
                None,
                Page {
                    offset: 0,
                    limit: 50,
                },
                at(t0()),
            )
            .await
            .unwrap();
        assert!(board.entries.iter().all(|e| e.user_id != "priv"));
        let own = board.own.expect("own standing present");
        // 0.88 slots between a1 (0.95) and a2 (0.70) → rank 2 among the public.
        assert_eq!(own.rank, 2);
        assert!((own.score - 0.88).abs() < 1e-9);
    }

    #[tokio::test]
    async fn caller_without_a_season_score_has_no_own_standing() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let m = module(repo.clone(), &["a1"]);
        play(&m, "a1", "p", 95.0, t0()).await;
        let board = m
            .get_board(
                "newcomer",
                Mode::Tempo,
                None,
                Page {
                    offset: 0,
                    limit: 50,
                },
                at(t0()),
            )
            .await
            .unwrap();
        assert!(board.own.is_none());
    }

    #[tokio::test]
    async fn board_paging_slices_the_public_listing() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let m = module(repo.clone(), &["a", "b", "c"]);
        for (u, sub) in [("a", 90.0), ("b", 80.0), ("c", 70.0)] {
            play(&m, u, &format!("p-{u}"), sub, t0()).await;
        }
        let page = m
            .get_board(
                "a",
                Mode::Tempo,
                None,
                Page {
                    offset: 1,
                    limit: 1,
                },
                at(t0()),
            )
            .await
            .unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.entries[0].user_id, "b");
        assert_eq!(page.entries[0].rank, 2);
        // An offset past the end is an empty page, not an error.
        let past = m
            .get_board(
                "a",
                Mode::Tempo,
                None,
                Page {
                    offset: 99,
                    limit: 10,
                },
                at(t0()),
            )
            .await
            .unwrap();
        assert!(past.entries.is_empty());
        assert_eq!(past.total, 3);
    }

    #[tokio::test]
    async fn rollover_snapshots_the_closed_season_and_starts_fresh() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let m = module(repo.clone(), &["a"]);
        play(&m, "a", "p", 90.0, t0()).await;

        // At an instant inside season 1, the snapshot freezes season 0.
        let written = m.run_season_snapshot(at(t1())).await.unwrap();
        assert_eq!(written, 1);
        let s0 = GlobalConfig::default().season_at(t0()).id;
        assert_eq!(
            m.seasons(t1()).await.unwrap().past_season_ids,
            vec![s0.clone()]
        );

        // The NEW season starts fresh — nobody is ranked in it yet...
        let live = m
            .get_board(
                "a",
                Mode::Tempo,
                None,
                Page {
                    offset: 0,
                    limit: 50,
                },
                at(t1()),
            )
            .await
            .unwrap();
        assert_eq!(live.total, 0);
        assert!(live.own.is_none());
        // ...while the snapshotted season still shows its final standings.
        let past = m
            .get_board(
                "a",
                Mode::Tempo,
                Some(&s0),
                Page {
                    offset: 0,
                    limit: 50,
                },
                at(t1()),
            )
            .await
            .unwrap();
        assert_eq!(past.total, 1);
        assert_eq!(past.entries[0].user_id, "a");
        // The per-season bests themselves are untouched by the rollover.
        assert!(repo.best_for("a", &s0, "p", Mode::Tempo).is_some());
    }

    #[tokio::test]
    async fn rollover_is_idempotent() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let m = module(repo.clone(), &["a"]);
        play(&m, "a", "p", 90.0, t0()).await;
        assert_eq!(m.run_season_snapshot(at(t1())).await.unwrap(), 1);
        // A re-delivered job writes nothing more (and does not duplicate).
        assert_eq!(m.run_season_snapshot(at(t1())).await.unwrap(), 0);
        let s0 = GlobalConfig::default().season_at(t0()).id;
        assert_eq!(
            repo.snapshot_standings(&s0, Mode::Tempo)
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn rollover_of_an_empty_season_writes_nothing() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let m = module(repo.clone(), &[]);
        assert_eq!(m.run_season_snapshot(at(t1())).await.unwrap(), 0);
        assert!(m.seasons(t1()).await.unwrap().past_season_ids.is_empty());
    }

    /// Seed a closed season with `a` (90) and `b` (95) and freeze it, with `public`
    /// deciding who had consented to be listed AT THE CLOSE. Returns the season id.
    async fn frozen_season(m: &GlobalLeaderboardModule) -> String {
        play(m, "a", "p1", 90.0, t0()).await;
        play(m, "b", "p2", 95.0, t0()).await;
        m.run_season_snapshot(at(t1())).await.unwrap();
        GlobalConfig::default().season_at(t0()).id
    }

    async fn past_board(m: &GlobalLeaderboardModule, viewer: &str, season: &str) -> GlobalBoard {
        m.get_board(
            viewer,
            Mode::Tempo,
            Some(season),
            Page {
                offset: 0,
                limit: 50,
            },
            at(t1()),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn a_past_season_keeps_a_player_who_has_since_gone_private() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        // Both were public when the season closed…
        let m = module(repo.clone(), &["a", "b"]);
        let s0 = frozen_season(&m).await;

        // …then "b" goes private. The archive must NOT re-rank: history is frozen.
        let m = module(repo.clone(), &["a"]);
        let past = past_board(&m, "a", &s0).await;
        assert_eq!(past.total, 2);
        assert_eq!(past.entries[0].user_id, "b"); // still #1, still listed
        assert_eq!(past.entries[1].user_id, "a");
    }

    #[tokio::test]
    async fn a_past_season_never_lists_someone_who_was_private_when_it_closed() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        // Only "a" had consented at the close; "b" was private then.
        let m = module(repo.clone(), &["a"]);
        let s0 = frozen_season(&m).await;

        // "b" goes public afterwards — that must NOT retro-add them to the archive.
        let m = module(repo.clone(), &["a", "b"]);
        let past = past_board(&m, "a", &s0).await;
        assert_eq!(past.total, 1);
        assert_eq!(past.entries[0].user_id, "a");
    }

    #[tokio::test]
    async fn a_past_season_still_drops_someone_no_longer_age_eligible() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        // Both consented at the close, so both are frozen as listable…
        let m = module(repo.clone(), &["a", "b"]);
        let s0 = frozen_season(&m).await;

        // …but the AGE safeguard stays live: "b" no longer passes it, so the frozen
        // consent must not pin them in the archive.
        let m = module_with_age(repo.clone(), &["a", "b"], Some(&["a"]));
        let past = past_board(&m, "a", &s0).await;
        assert_eq!(past.total, 1);
        assert_eq!(past.entries[0].user_id, "a");
    }

    #[tokio::test]
    async fn a_past_season_shows_a_private_caller_their_own_standing() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        // Only "a" consented at the close; "b" was private then and still is.
        let m = module(repo.clone(), &["a"]);
        let s0 = frozen_season(&m).await;

        let past = past_board(&m, "b", &s0).await;
        // Not listed to others…
        assert!(past.entries.iter().all(|e| e.user_id != "b"));
        // …but still sees their own past standing (95 beats a's 90 → rank 1).
        assert_eq!(past.own.expect("own standing").rank, 1);
    }

    #[tokio::test]
    async fn seasons_lists_the_current_one_plus_snapshots() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let m = module(repo.clone(), &["a"]);
        let cfg = GlobalConfig::default();
        let seasons = m.seasons(t0()).await.unwrap();
        assert_eq!(seasons.current_season_id, cfg.season_at(t0()).id);
        assert!(seasons.past_season_ids.is_empty());

        play(&m, "a", "p", 90.0, t0()).await;
        m.run_season_snapshot(at(t1())).await.unwrap();
        let seasons = m.seasons(t1()).await.unwrap();
        assert_eq!(seasons.current_season_id, cfg.season_at(t1()).id);
        assert_eq!(seasons.past_season_ids, vec![cfg.season_at(t0()).id]);
    }
}
