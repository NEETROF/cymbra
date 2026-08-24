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

//! Leaderboard personal-best store (change: add-play-leaderboards): the
//! per-(player, piece, mode) best derived from ingested sessions, plus the
//! ranking DTOs. The best is maintained **monotonically** on #5's
//! `RecordPlaySession` path — a new result raises a best only when it is strictly
//! better, so ingest is idempotent under at-least-once delivery — and boards read
//! from these durable rows (not raw sessions), so a best survives detail pruning.
//!
//! The visibility/eligibility gate that decides who is *listed* to others is NOT
//! here: it lives in the `user_account` schema and is delegated to the
//! [`cymbra_user_port::UserPort`] (as the play-activity read already does).

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};

use crate::play::PlaySession;

/// Which board a result feeds. `Tempo` is the free-run synchronization sub-score;
/// `Reaction` is the Wait-Mode sub-score. A `mixed` run feeds both; a pure run one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Tempo,
    Reaction,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Tempo => "tempo",
            Mode::Reaction => "reaction",
        }
    }

    /// Parse the wire/stored form; unknown values are rejected (fail-closed).
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "tempo" => Ok(Mode::Tempo),
            "reaction" => Ok(Mode::Reaction),
            other => Err(AppError::InvalidArgument(format!("unknown mode {other:?}"))),
        }
    }
}

/// A per-mode result derived from one ingested session, ready to be considered as
/// a new personal best. `tiebreak_metric` is normalised so **smaller is better**
/// (|mean tempo offset| on the tempo board, mean reaction ms on the reaction
/// board); `achieved_at_ms` is when the session ended (the final tie-break).
#[derive(Debug, Clone, PartialEq)]
pub struct BestCandidate {
    pub mode: Mode,
    pub subscore: f32,
    pub tiebreak_metric: f32,
    pub achieved_at_ms: i64,
}

/// A personal best to persist (the monotonic-upsert input).
#[derive(Debug, Clone, PartialEq)]
pub struct LeaderboardBest {
    pub user_id: String,
    pub catalog_score_id: String,
    pub mode: Mode,
    pub subscore: f32,
    pub tiebreak_metric: f32,
    pub achieved_at_ms: i64,
}

/// A stored best as read back for ranking. No sensitive fields: the display
/// handle/name are resolved separately through the [`cymbra_user_port::UserPort`].
#[derive(Debug, Clone, PartialEq)]
pub struct StoredBest {
    pub user_id: String,
    pub subscore: f32,
    pub tiebreak_metric: f32,
    pub achieved_at_ms: i64,
}

/// Storage surface for leaderboard bests.
#[async_trait]
pub trait LeaderboardRepo: Send + Sync {
    /// Whether `score_id` is an **accepted catalog score** — only these have shared
    /// boards (user uploads / pending / rejected pieces do not).
    async fn is_accepted_catalog(&self, score_id: &str) -> Result<bool>;

    /// Which of `score_ids` are **percussion** pieces (change: add-drum-scoring).
    ///
    /// The instrument lookup lives here, on the boards' own store, because a board
    /// is keyed by catalog piece: answering "this piece has a board" is itself an
    /// existence oracle, so the read gate needs the instrument of the very ids it
    /// was asked about — batched, since the card read asks about many at once. Ids
    /// that are not catalog pieces (bundled slugs, uploads) are simply absent from
    /// the answer: they have no instrument and no board.
    async fn percussion_pieces(&self, score_ids: &[String]) -> Result<HashSet<String>>;

    /// **Monotonic** upsert: raise the stored best for `(user, piece, mode)` only
    /// when `best` is strictly better under the ranking order (higher sub-score;
    /// ties by smaller tie-break metric, then earlier `achieved_at`). A worse or
    /// replayed result is a no-op — so this is idempotent under at-least-once
    /// ingest.
    async fn upsert_best(&self, best: &LeaderboardBest) -> Result<()>;

    /// Every stored best for one board `(score_id, mode)`, in ranking order
    /// (sub-score desc, tie-break asc, achieved_at asc). The visibility filter is
    /// applied by the caller (module) via the user port — this returns all owners.
    async fn board_bests(&self, score_id: &str, mode: Mode) -> Result<Vec<StoredBest>>;
}

/// The hook #5's play-session ingest path calls after a session is persisted, to
/// maintain the leaderboard bests (change: add-play-leaderboards). A narrow seam
/// so [`crate::play_module::PlayModule`] depends only on this — not the whole
/// leaderboard module — and can be left `None` where leaderboards are not wired.
#[async_trait]
pub trait LeaderboardSink: Send + Sync {
    /// Consider `session` for the boards: monotonically upsert the per-mode
    /// best(s) for an accepted catalog piece, after the integrity checks. A no-op
    /// (never an error that would fail the session ack) for a non-catalog piece,
    /// an unscored session, or an integrity-rejected result.
    async fn ingest_session(&self, session: &PlaySession) -> Result<()>;
}

// --- In-memory fake (tests) -------------------------------------------------

/// In-memory [`LeaderboardRepo`] for unit tests (no Postgres). Enforces the same
/// monotonic-upsert and ranking semantics as the Postgres adapter.
#[derive(Default)]
pub struct FakeLeaderboardRepo {
    /// (user_id, score_id, mode) -> stored best.
    bests: Mutex<HashMap<(String, String, String), LeaderboardBest>>,
    /// score_ids that count as accepted catalog scores.
    accepted: Mutex<Vec<String>>,
    /// score_ids that are percussion pieces (change: add-drum-scoring).
    percussion: Mutex<HashSet<String>>,
}

impl FakeLeaderboardRepo {
    /// Mark `score_id` as an accepted catalog score (so ingest admits it).
    pub fn accept(&self, score_id: &str) {
        self.accepted.lock().unwrap().push(score_id.to_string());
    }

    /// Mark `score_id` as a percussion piece (so the read gate withholds it from
    /// a caller outside the drum audience).
    pub fn mark_percussion(&self, score_id: &str) {
        self.percussion.lock().unwrap().insert(score_id.to_string());
    }

    /// Test helper: the stored best for `(user, piece, mode)`, if any.
    pub fn best_for(&self, user_id: &str, score_id: &str, mode: Mode) -> Option<LeaderboardBest> {
        self.bests
            .lock()
            .unwrap()
            .get(&(
                user_id.to_string(),
                score_id.to_string(),
                mode.as_str().to_string(),
            ))
            .cloned()
    }
}

#[async_trait]
impl LeaderboardRepo for FakeLeaderboardRepo {
    async fn is_accepted_catalog(&self, score_id: &str) -> Result<bool> {
        Ok(self.accepted.lock().unwrap().iter().any(|s| s == score_id))
    }

    async fn percussion_pieces(&self, score_ids: &[String]) -> Result<HashSet<String>> {
        let drums = self.percussion.lock().unwrap();
        Ok(score_ids
            .iter()
            .filter(|id| drums.contains(*id))
            .cloned()
            .collect())
    }

    async fn upsert_best(&self, best: &LeaderboardBest) -> Result<()> {
        let key = (
            best.user_id.clone(),
            best.catalog_score_id.clone(),
            best.mode.as_str().to_string(),
        );
        let mut map = self.bests.lock().unwrap();
        let raise = match map.get(&key) {
            None => true,
            Some(cur) => crate::leaderboard_core::is_better(
                best.subscore,
                best.tiebreak_metric,
                best.achieved_at_ms,
                cur.subscore,
                cur.tiebreak_metric,
                cur.achieved_at_ms,
            ),
        };
        if raise {
            map.insert(key, best.clone());
        }
        Ok(())
    }

    async fn board_bests(&self, score_id: &str, mode: Mode) -> Result<Vec<StoredBest>> {
        let map = self.bests.lock().unwrap();
        let mut rows: Vec<StoredBest> = map
            .values()
            .filter(|b| b.catalog_score_id == score_id && b.mode == mode)
            .map(|b| StoredBest {
                user_id: b.user_id.clone(),
                subscore: b.subscore,
                tiebreak_metric: b.tiebreak_metric,
                achieved_at_ms: b.achieved_at_ms,
            })
            .collect();
        rows.sort_by(|a, b| {
            crate::leaderboard_core::rank_cmp(
                a.subscore,
                a.tiebreak_metric,
                a.achieved_at_ms,
                b.subscore,
                b.tiebreak_metric,
                b.achieved_at_ms,
            )
        });
        Ok(rows)
    }
}
