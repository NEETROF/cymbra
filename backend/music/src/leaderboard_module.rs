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

//! Leaderboard orchestration (change: add-play-leaderboards): the two sides of
//! the feature over a [`LeaderboardRepo`] and the [`UserPort`] visibility gate.
//!
//! * **Ingest** ([`LeaderboardSink`]): on #5's `RecordPlaySession` path, for an
//!   **accepted catalog piece** only, derive the per-mode candidates (with the
//!   basic integrity checks), and monotonically upsert each as a personal best. A
//!   non-catalog piece, an unscored session, a malformed record, or an integrity
//!   failure contributes nothing — and never fails the session ack.
//! * **Read** ([`LeaderboardModule::get_board`]): rank a `(piece, mode)` board,
//!   listing only **public + age-eligible** players (the #5 gate, fail-closed) but
//!   always returning the caller their **own** best + rank among the public
//!   entries, even when the caller is private/ineligible. No sensitive fields
//!   leave here — entries carry only the public handle/display name resolved
//!   through the user port.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;
use cymbra_platform::Result;
use cymbra_user_port::{PlayerProfile, UserPort};

use crate::leaderboard::{LeaderboardBest, LeaderboardRepo, LeaderboardSink, Mode, StoredBest};
use crate::leaderboard_core;
use crate::play::PlaySession;

/// Server-side clamp on a board page size, so a client cannot ask for an unbounded
/// listing. A non-positive request falls back to [`DEFAULT_PAGE`].
const MAX_PAGE: i64 = 100;
const DEFAULT_PAGE: i64 = 50;

/// One ranked board entry (public players) or the caller's own standing. Carries
/// only NON-SENSITIVE fields — the public handle/display name, never email,
/// curator alignment/reliability, or moderation state.
#[derive(Debug, Clone, PartialEq)]
pub struct BoardEntry {
    pub rank: i32,
    pub user_id: String,
    pub handle: Option<String>,
    pub display_name: Option<String>,
    pub subscore: f32,
    pub tiebreak_metric: f32,
    pub achieved_at_ms: i64,
}

/// A page of a board plus the caller's own standing.
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    /// The requested page of PUBLIC + eligible entries, in ranking order.
    pub entries: Vec<BoardEntry>,
    /// Total number of public entries on this board (for paging).
    pub total: i32,
    /// The caller's own best + rank among the public entries, present whenever the
    /// caller has scored on this board — even if the caller is private/ineligible.
    pub own: Option<BoardEntry>,
}

/// A piece's score-card standing (change: add-play-leaderboards). `rank > 0` is the
/// caller's BEST rank across the two modes (the smaller wins); `rank == 0` means the
/// caller is not ranked but the board has public entries (a bare-trophy badge).
/// `mode` is the board a tap should open.
#[derive(Debug, Clone, PartialEq)]
pub struct MyStanding {
    pub score_id: String,
    pub rank: i32,
    pub subscore: f32,
    pub mode: Mode,
}

/// Leaderboard reads + the ingest maintenance hook, over the bests store and the
/// user-port visibility gate.
pub struct LeaderboardModule {
    repo: Arc<dyn LeaderboardRepo>,
    user: Arc<dyn UserPort>,
}

impl LeaderboardModule {
    pub fn new(repo: Arc<dyn LeaderboardRepo>, user: Arc<dyn UserPort>) -> Self {
        Self { repo, user }
    }

    /// Maintain the boards from one persisted session (the [`LeaderboardSink`]
    /// body, exposed directly for tests). See the module docs for the gates.
    pub async fn maintain_from_session(&self, session: &PlaySession) -> Result<()> {
        // No piece ⇒ nothing to rank (the heatmap still counted it via #5).
        let Some(score_id) = session.score_id.as_deref() else {
            return Ok(());
        };
        // Only accepted catalog pieces have shared boards (D1); a user upload,
        // pending, or rejected piece is skipped.
        if !self.repo.is_accepted_catalog(score_id).await? {
            return Ok(());
        }
        let candidates = match leaderboard_core::candidates_from_result(
            &session.session_result_json,
            session.played_at_ms,
        ) {
            Ok(c) => c,
            Err(why) => {
                // A malformed record is stored by #5 but never reaches a board.
                tracing::warn!(
                    session_id = %session.session_id,
                    reason = %why,
                    "leaderboard: skipping session with unparseable result"
                );
                return Ok(());
            }
        };
        // Integrity-rejected modes are logged and kept off the boards (design D5).
        for reason in &candidates.rejected {
            tracing::warn!(
                session_id = %session.session_id,
                score_id = %score_id,
                reason = %reason,
                "leaderboard: result failed integrity check, excluded from board"
            );
        }
        for candidate in candidates.accepted {
            self.repo
                .upsert_best(&LeaderboardBest {
                    user_id: session.user_id.clone(),
                    catalog_score_id: score_id.to_string(),
                    mode: candidate.mode,
                    subscore: candidate.subscore,
                    tiebreak_metric: candidate.tiebreak_metric,
                    achieved_at_ms: candidate.achieved_at_ms,
                })
                .await?;
        }
        Ok(())
    }

    /// Read one board `(score_id, mode)` as seen by `viewer_id`: the requested page
    /// of public entries (private/ineligible players are never listed), plus the
    /// caller's own standing. `today` (UTC) drives the eligibility gate.
    pub async fn get_board(
        &self,
        viewer_id: &str,
        score_id: &str,
        mode: Mode,
        offset: i64,
        limit: i64,
        today: NaiveDate,
    ) -> Result<Board> {
        // All stored bests for the board, already in ranking order.
        let all = self.repo.board_bests(score_id, mode).await?;
        let ids: Vec<String> = all.iter().map(|b| b.user_id.clone()).collect();

        // The public + age-eligible subset, with their non-sensitive display fields.
        let profiles: HashMap<String, PlayerProfile> = self
            .user
            .listable_profiles(&ids, today)
            .await?
            .into_iter()
            .map(|p| (p.user_id.clone(), p))
            .collect();

        // Public entries in rank order (all was already sorted by the repo).
        let public: Vec<&StoredBest> = all
            .iter()
            .filter(|b| profiles.contains_key(&b.user_id))
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
        let entries: Vec<BoardEntry> = if start >= public.len() {
            Vec::new()
        } else {
            public[start..end]
                .iter()
                .enumerate()
                .map(|(i, b)| self.entry((start + i) as i32 + 1, b, profiles.get(&b.user_id)))
                .collect()
        };

        // The caller's own standing — always, even when not publicly listed.
        let own = match all.iter().find(|b| b.user_id == viewer_id) {
            Some(own_best) => {
                let public_stored: Vec<StoredBest> = public.iter().map(|b| (*b).clone()).collect();
                let rank = leaderboard_core::own_rank(&public_stored, own_best);
                // The owner always sees their own profile (whatever the visibility).
                let me = self
                    .user
                    .get_player_profile(viewer_id, viewer_id, today)
                    .await
                    .ok();
                Some(BoardEntry {
                    rank,
                    user_id: viewer_id.to_string(),
                    handle: me.as_ref().and_then(|p| p.handle.clone()),
                    display_name: me.as_ref().and_then(|p| p.display_name.clone()),
                    subscore: own_best.subscore,
                    tiebreak_metric: own_best.tiebreak_metric,
                    achieved_at_ms: own_best.achieved_at_ms,
                })
            }
            None => None,
        };

        Ok(Board {
            entries,
            total,
            own,
        })
    }

    /// Batch: for each of `score_ids`, the standing that drives its score-card
    /// badge. Fetches every board once and resolves visibility with a **single**
    /// `listable_profiles` call. A piece is returned when it has a **badge-worthy**
    /// board:
    /// * the caller is ranked ⇒ `rank` = their BEST rank across the two modes
    ///   (`> 0`), `mode` = the board it came from;
    /// * else the public listing is non-empty ⇒ `rank == 0` (a bare-trophy badge
    ///   that still opens a populated board), `mode` = a board that has entries.
    ///
    /// A piece with an **empty** board (no public entries and the caller not
    /// ranked) is **omitted** — so its card shows no badge (no click "for
    /// nothing"). `today` (UTC) drives the eligibility gate, like `get_board`.
    pub async fn my_standings(
        &self,
        viewer_id: &str,
        score_ids: &[String],
        today: NaiveDate,
    ) -> Result<Vec<MyStanding>> {
        use std::collections::HashSet;
        const MODES: [Mode; 2] = [Mode::Tempo, Mode::Reaction];

        // Fetch each (piece, mode) board once; gather every owner id for one gate.
        let mut boards: HashMap<(String, Mode), Vec<StoredBest>> = HashMap::new();
        let mut all_ids: HashSet<String> = HashSet::new();
        for score_id in score_ids {
            for mode in MODES {
                let bests = self.repo.board_bests(score_id, mode).await?;
                for b in &bests {
                    all_ids.insert(b.user_id.clone());
                }
                boards.insert((score_id.clone(), mode), bests);
            }
        }
        let ids: Vec<String> = all_ids.into_iter().collect();
        let listable: HashSet<String> = self
            .user
            .listable_profiles(&ids, today)
            .await?
            .into_iter()
            .map(|p| p.user_id)
            .collect();

        let mut out = Vec::new();
        for score_id in score_ids {
            let mut best: Option<MyStanding> = None;
            // The first mode whose PUBLIC listing is non-empty — so a not-ranked
            // caller still gets a bare-trophy badge that opens a real board.
            let mut public_mode: Option<Mode> = None;
            for mode in MODES {
                let bests = boards
                    .get(&(score_id.clone(), mode))
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                let public: Vec<StoredBest> = bests
                    .iter()
                    .filter(|b| listable.contains(&b.user_id))
                    .cloned()
                    .collect();
                if !public.is_empty() && public_mode.is_none() {
                    public_mode = Some(mode);
                }
                if let Some(own) = bests.iter().find(|b| b.user_id == viewer_id) {
                    let rank = leaderboard_core::own_rank(&public, own);
                    // Keep the smaller rank across the two modes (ties keep first).
                    let better = match &best {
                        None => true,
                        Some(b) => rank < b.rank,
                    };
                    if better {
                        best = Some(MyStanding {
                            score_id: score_id.clone(),
                            rank,
                            subscore: own.subscore,
                            mode,
                        });
                    }
                }
            }
            if let Some(standing) = best {
                out.push(standing);
            } else if let Some(mode) = public_mode {
                // Not ranked, but the board has public entries: emit a `rank == 0`
                // marker so the card shows a bare trophy opening a populated board.
                // A piece with an empty board is OMITTED entirely (no badge).
                out.push(MyStanding {
                    score_id: score_id.clone(),
                    rank: 0,
                    subscore: 0.0,
                    mode,
                });
            }
        }
        Ok(out)
    }

    fn entry(&self, rank: i32, best: &StoredBest, profile: Option<&PlayerProfile>) -> BoardEntry {
        BoardEntry {
            rank,
            user_id: best.user_id.clone(),
            handle: profile.and_then(|p| p.handle.clone()),
            display_name: profile.and_then(|p| p.display_name.clone()),
            subscore: best.subscore,
            tiebreak_metric: best.tiebreak_metric,
            achieved_at_ms: best.achieved_at_ms,
        }
    }
}

#[async_trait]
impl LeaderboardSink for LeaderboardModule {
    async fn ingest_session(&self, session: &PlaySession) -> Result<()> {
        self.maintain_from_session(session).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leaderboard::FakeLeaderboardRepo;
    use cymbra_user_port::MockUserPort;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()
    }

    fn session(user: &str, score: &str, json: &str, at: i64) -> PlaySession {
        PlaySession {
            session_id: uuid::Uuid::now_v7().to_string(),
            user_id: user.into(),
            score_id: Some(score.into()),
            played_at_ms: at,
            tz_offset_minutes: 0,
            overall_sync_pct: 80.0,
            session_result_json: json.into(),
        }
    }

    /// A tempo-only result JSON with the given sub-score + mean offset.
    fn tempo_json(sub: f64, offset: f64) -> String {
        format!(r#"{{"freeSyncPct": {sub}, "freeOnsetCount": 6, "avgFreeOffsetMs": {offset}}}"#)
    }

    /// A module whose user port lists exactly `public` (each with a handle) and
    /// resolves any own-profile self-read.
    fn module(
        repo: Arc<FakeLeaderboardRepo>,
        public: &'static [&'static str],
    ) -> LeaderboardModule {
        let mut user = MockUserPort::new();
        user.expect_listable_profiles().returning(move |ids, _| {
            Ok(ids
                .iter()
                .filter(|id| public.contains(&id.as_str()))
                .map(|id| PlayerProfile {
                    user_id: id.clone(),
                    handle: Some(format!("@{id}")),
                    display_name: Some(id.clone()),
                    visibility: cymbra_user_port::Visibility::Public,
                })
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
        LeaderboardModule::new(repo, Arc::new(user))
    }

    #[tokio::test]
    async fn ingest_upserts_only_for_accepted_catalog() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        repo.accept("piece-ok");
        let m = module(repo.clone(), &[]);
        // Accepted piece → a tempo best is recorded.
        m.maintain_from_session(&session("u1", "piece-ok", &tempo_json(80.0, 10.0), 5))
            .await
            .unwrap();
        assert!(repo.best_for("u1", "piece-ok", Mode::Tempo).is_some());
        // Non-accepted piece (user upload / pending) → nothing recorded.
        m.maintain_from_session(&session("u1", "piece-private", &tempo_json(99.0, 1.0), 6))
            .await
            .unwrap();
        assert!(repo.best_for("u1", "piece-private", Mode::Tempo).is_none());
    }

    #[tokio::test]
    async fn ingest_is_monotonic_and_idempotent() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        repo.accept("p");
        let m = module(repo.clone(), &[]);
        m.maintain_from_session(&session("u1", "p", &tempo_json(80.0, 10.0), 1))
            .await
            .unwrap();
        // A worse result does not lower the best.
        m.maintain_from_session(&session("u1", "p", &tempo_json(60.0, 1.0), 2))
            .await
            .unwrap();
        assert_eq!(
            repo.best_for("u1", "p", Mode::Tempo).unwrap().subscore,
            80.0
        );
        // A better result raises it.
        m.maintain_from_session(&session("u1", "p", &tempo_json(90.0, 20.0), 3))
            .await
            .unwrap();
        assert_eq!(
            repo.best_for("u1", "p", Mode::Tempo).unwrap().subscore,
            90.0
        );
    }

    #[tokio::test]
    async fn ingest_tolerates_missing_piece_and_bad_json() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        repo.accept("p");
        let m = module(repo.clone(), &[]);
        // No score_id → no-op success.
        let mut s = session("u1", "p", &tempo_json(80.0, 10.0), 1);
        s.score_id = None;
        m.maintain_from_session(&s).await.unwrap();
        // Malformed result on an accepted piece → no-op success, nothing recorded.
        m.maintain_from_session(&session("u1", "p", "not json", 1))
            .await
            .unwrap();
        assert!(repo.best_for("u1", "p", Mode::Tempo).is_none());
    }

    #[tokio::test]
    async fn board_lists_public_only_and_ranks() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        repo.accept("p");
        // Three players, only a1 + a2 are public.
        let m = module(repo.clone(), &["a1", "a2"]);
        for (u, sub) in [("a1", 95.0), ("a2", 80.0), ("priv", 99.0)] {
            m.maintain_from_session(&session(u, "p", &tempo_json(sub, 5.0), 1))
                .await
                .unwrap();
        }
        // A public viewer sees only the two public players, ranked by sub-score.
        let board = m
            .get_board("a1", "p", Mode::Tempo, 0, 50, today())
            .await
            .unwrap();
        assert_eq!(board.total, 2);
        assert_eq!(board.entries.len(), 2);
        assert_eq!(board.entries[0].user_id, "a1");
        assert_eq!(board.entries[0].rank, 1);
        assert_eq!(board.entries[0].handle.as_deref(), Some("@a1"));
        assert_eq!(board.entries[1].user_id, "a2");
        // The private player is never listed to others.
        assert!(board.entries.iter().all(|e| e.user_id != "priv"));
    }

    #[tokio::test]
    async fn private_caller_sees_own_rank_but_is_not_listed() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        repo.accept("p");
        let m = module(repo.clone(), &["a1", "a2"]);
        for (u, sub) in [("a1", 95.0), ("a2", 70.0), ("priv", 88.0)] {
            m.maintain_from_session(&session(u, "p", &tempo_json(sub, 5.0), 1))
                .await
                .unwrap();
        }
        // The private caller: not in the listing, but sees their own standing —
        // 88 slots between a1 (95) and a2 (70) → rank 2 among the public players.
        let board = m
            .get_board("priv", "p", Mode::Tempo, 0, 50, today())
            .await
            .unwrap();
        assert!(board.entries.iter().all(|e| e.user_id != "priv"));
        let own = board.own.expect("own standing present");
        assert_eq!(own.subscore, 88.0);
        assert_eq!(own.rank, 2);
    }

    #[tokio::test]
    async fn caller_without_a_best_has_no_own_standing() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        repo.accept("p");
        let m = module(repo.clone(), &["a1"]);
        m.maintain_from_session(&session("a1", "p", &tempo_json(95.0, 5.0), 1))
            .await
            .unwrap();
        let board = m
            .get_board("newcomer", "p", Mode::Tempo, 0, 50, today())
            .await
            .unwrap();
        assert!(board.own.is_none());
    }

    #[tokio::test]
    async fn board_paging_slices_the_public_listing() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        repo.accept("p");
        let m = module(repo.clone(), &["a", "b", "c"]);
        for (u, sub) in [("a", 90.0), ("b", 80.0), ("c", 70.0)] {
            m.maintain_from_session(&session(u, "p", &tempo_json(sub, 5.0), 1))
                .await
                .unwrap();
        }
        let page = m
            .get_board("a", "p", Mode::Tempo, 1, 1, today())
            .await
            .unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.entries[0].user_id, "b");
        assert_eq!(page.entries[0].rank, 2);
    }

    /// Seed a stored best directly (bypassing ingest) for the batch-standing tests.
    async fn seed(
        repo: &FakeLeaderboardRepo,
        user: &str,
        score: &str,
        mode: Mode,
        sub: f32,
        at: i64,
    ) {
        repo.upsert_best(&LeaderboardBest {
            user_id: user.into(),
            catalog_score_id: score.into(),
            mode,
            subscore: sub,
            tiebreak_metric: 5.0,
            achieved_at_ms: at,
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn my_standings_takes_the_best_rank_across_modes() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        let m = module(repo.clone(), &["a", "me"]);
        // Tempo: a=95 (#1), me=80 (#2). Reaction: me=99 (#1).
        seed(&repo, "a", "p", Mode::Tempo, 95.0, 1).await;
        seed(&repo, "me", "p", Mode::Tempo, 80.0, 1).await;
        seed(&repo, "me", "p", Mode::Reaction, 99.0, 1).await;
        let out = m.my_standings("me", &["p".into()], today()).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].score_id, "p");
        // The reaction #1 beats the tempo #2.
        assert_eq!(out[0].rank, 1);
        assert_eq!(out[0].mode, Mode::Reaction);
    }

    #[tokio::test]
    async fn my_standings_ranks_a_private_caller_among_public_players() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        // Only "a" is public; the caller "me" is private (not listable).
        let m = module(repo.clone(), &["a"]);
        seed(&repo, "a", "p", Mode::Tempo, 95.0, 1).await;
        seed(&repo, "me", "p", Mode::Tempo, 88.0, 1).await;
        let out = m.my_standings("me", &["p".into()], today()).await.unwrap();
        // Ranked #2 behind the public player, even though not listed to others.
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].rank, 2);
        assert_eq!(out[0].mode, Mode::Tempo);
    }

    #[tokio::test]
    async fn my_standings_flags_populated_boards_and_omits_empty_ones() {
        use std::collections::HashMap;
        let repo = Arc::new(FakeLeaderboardRepo::default());
        let m = module(repo.clone(), &["a"]); // only "a" is public
        seed(&repo, "a", "p1", Mode::Tempo, 95.0, 1).await; // caller absent, public board
        seed(&repo, "me", "p2", Mode::Tempo, 70.0, 1).await; // caller ranked
        // p3: no bests at all (empty board).
        let out = m
            .my_standings("me", &["p1".into(), "p2".into(), "p3".into()], today())
            .await
            .unwrap();
        let by_id: HashMap<&str, &MyStanding> =
            out.iter().map(|s| (s.score_id.as_str(), s)).collect();
        // p1: a populated public board the caller is not on → bare trophy (rank 0).
        assert_eq!(by_id["p1"].rank, 0);
        // p2: the caller is ranked → a real rank.
        assert!(by_id["p2"].rank > 0);
        // p3: an empty board is omitted entirely (its card shows no badge).
        assert!(!by_id.contains_key("p3"));
        assert_eq!(out.len(), 2);
    }
}
