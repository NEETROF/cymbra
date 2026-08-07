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

//! The GLOBAL leaderboard's season stores (change: add-global-leaderboard): the
//! per-(player, season, piece, mode) season best maintained on ingest, the
//! end-of-season snapshot (hall of fame), and their DTOs.
//!
//! Like the per-piece boards (#6), the season best is raised **monotonically** —
//! only a strictly better in-season result wins — so ingest stays idempotent under
//! #5's at-least-once delivery. The difficulty-weighted best-N aggregation that
//! turns these rows into a global season score lives in
//! [`crate::global_leaderboard_core`]; the visibility/eligibility gate that decides
//! who is *listed* lives in the `user_account` schema and is delegated to the
//! [`cymbra_user_port::UserPort`], exactly as the per-piece board read does.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_platform::Result;

use crate::leaderboard::{BestCandidate, Mode};

/// A season best to persist (the monotonic-upsert input): one player's best
/// per-mode sub-score on one piece **within one season**.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSeasonBest {
    pub user_id: String,
    pub season_id: String,
    pub catalog_score_id: String,
    pub mode: Mode,
    pub subscore: f32,
    pub achieved_at_ms: i64,
}

/// A stored season best as read back for aggregation, joined with the piece's
/// catalog `level` (the difficulty-weight input). No sensitive fields: display
/// handles are resolved separately through the [`cymbra_user_port::UserPort`].
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonBestRow {
    pub user_id: String,
    pub catalog_score_id: String,
    pub mode: Mode,
    /// The piece's catalog level (`beginner`/`intermediate`/`advanced`), or `None`
    /// for an unleveled piece → the neutral difficulty weight.
    pub level: Option<String>,
    pub subscore: f32,
    pub achieved_at_ms: i64,
}

/// One player's aggregated standing in a season+mode — the difficulty-weighted
/// best-N total plus its tie-break inputs. Produced by
/// [`crate::global_leaderboard_core::aggregate`] for a live season and stored
/// verbatim by the end-of-season snapshot for a past one.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalScore {
    pub user_id: String,
    /// The global season score (difficulty-weighted best-N sum). Rank primary, desc.
    pub score: f64,
    /// How many pieces fed the score (≤ the configured N). First tie-break, desc.
    pub contributing_pieces: i32,
    /// When the score was reached (its latest contribution). Last tie-break, asc.
    pub reached_at_ms: i64,
    /// Whether the player had consented to being listed **when the season closed**
    /// — only meaningful on a SNAPSHOT row, where it freezes past consent so the
    /// archive's ranking stays stable (design D5). A live aggregation leaves it
    /// `true`: there, the live [`cymbra_user_port::UserPort`] gate decides and this
    /// field is never consulted.
    pub was_listable: bool,
}

/// Storage surface for the global-leaderboard season data.
#[async_trait]
pub trait GlobalLeaderboardRepo: Send + Sync {
    /// **Monotonic** upsert: raise the stored season best for
    /// `(user, season, piece, mode)` only when `best` has a strictly higher
    /// sub-score (ties broken by an earlier `achieved_at`). A worse or replayed
    /// result is a no-op — so this is idempotent under at-least-once ingest.
    async fn upsert_season_best(&self, best: &GlobalSeasonBest) -> Result<()>;

    /// Every stored season best for one `(season, mode)`, joined with each piece's
    /// catalog level. Order is irrelevant — the aggregation sorts.
    async fn season_bests(&self, season_id: &str, mode: Mode) -> Result<Vec<SeasonBestRow>>;

    /// The snapshotted final standings of a past `(season, mode)`, empty when that
    /// season has not been snapshotted. Raw aggregates, NOT ranks: the ranking is
    /// recomputed on read, and each row carries the `was_listable` consent frozen
    /// when the season closed (the age safeguard is re-checked live).
    async fn snapshot_standings(&self, season_id: &str, mode: Mode) -> Result<Vec<GlobalScore>>;

    /// Freeze `standings` as the final hall-of-fame rows for `(season, mode)`.
    /// **Idempotent**: an already-snapshotted season keeps its existing rows.
    /// Returns how many rows were newly written.
    async fn write_snapshot(
        &self,
        season_id: &str,
        mode: Mode,
        standings: &[GlobalScore],
    ) -> Result<u64>;

    /// Every season id that has a snapshot, most recent first — the past seasons a
    /// season selector may offer.
    async fn snapshotted_seasons(&self) -> Result<Vec<String>>;
}

/// The hook the per-piece leaderboard ingest calls to ALSO maintain the global
/// season bests (change: add-global-leaderboard, task 1.2). A narrow seam so
/// [`crate::leaderboard_module::LeaderboardModule`] depends only on this — not the
/// whole global module — and can be left `None` where the global board is not
/// wired. It receives the candidates that already passed #6's accepted-catalog and
/// integrity checks, so both boards admit exactly the same results.
#[async_trait]
pub trait GlobalSeasonSink: Send + Sync {
    /// Consider `candidates` (one per mode, already integrity-checked) as season
    /// bests for `user_id` on the accepted catalog piece `score_id`. The season is
    /// derived from each candidate's achievement time, so a late delivery lands in
    /// the season it was played in.
    async fn ingest_candidates(
        &self,
        user_id: &str,
        score_id: &str,
        candidates: &[BestCandidate],
    ) -> Result<()>;
}

// --- In-memory fake (tests) -------------------------------------------------

/// In-memory [`GlobalLeaderboardRepo`] for unit tests (no Postgres). Enforces the
/// same monotonic-upsert and snapshot-idempotency semantics as the Postgres
/// adapter.
#[derive(Default)]
pub struct FakeGlobalLeaderboardRepo {
    /// (user_id, season_id, score_id, mode) -> stored season best.
    bests: Mutex<HashMap<(String, String, String, String), GlobalSeasonBest>>,
    /// (season_id, mode) -> frozen final standings.
    snapshots: Mutex<HashMap<(String, String), Vec<GlobalScore>>>,
    /// score_id -> catalog level, for the difficulty-weight join.
    levels: Mutex<HashMap<String, Option<String>>>,
}

impl FakeGlobalLeaderboardRepo {
    /// Declare `score_id`'s catalog level (what the SQL join would supply).
    /// Undeclared pieces read back as unleveled.
    pub fn set_level(&self, score_id: &str, level: Option<&str>) {
        self.levels
            .lock()
            .unwrap()
            .insert(score_id.to_string(), level.map(str::to_string));
    }

    /// Test helper: the stored season best for `(user, season, piece, mode)`.
    pub fn best_for(
        &self,
        user_id: &str,
        season_id: &str,
        score_id: &str,
        mode: Mode,
    ) -> Option<GlobalSeasonBest> {
        self.bests
            .lock()
            .unwrap()
            .get(&(
                user_id.to_string(),
                season_id.to_string(),
                score_id.to_string(),
                mode.as_str().to_string(),
            ))
            .cloned()
    }
}

#[async_trait]
impl GlobalLeaderboardRepo for FakeGlobalLeaderboardRepo {
    async fn upsert_season_best(&self, best: &GlobalSeasonBest) -> Result<()> {
        let key = (
            best.user_id.clone(),
            best.season_id.clone(),
            best.catalog_score_id.clone(),
            best.mode.as_str().to_string(),
        );
        let mut map = self.bests.lock().unwrap();
        let raise = match map.get(&key) {
            None => true,
            Some(cur) => {
                best.subscore > cur.subscore
                    || (best.subscore == cur.subscore && best.achieved_at_ms < cur.achieved_at_ms)
            }
        };
        if raise {
            map.insert(key, best.clone());
        }
        Ok(())
    }

    async fn season_bests(&self, season_id: &str, mode: Mode) -> Result<Vec<SeasonBestRow>> {
        let levels = self.levels.lock().unwrap();
        Ok(self
            .bests
            .lock()
            .unwrap()
            .values()
            .filter(|b| b.season_id == season_id && b.mode == mode)
            .map(|b| SeasonBestRow {
                user_id: b.user_id.clone(),
                catalog_score_id: b.catalog_score_id.clone(),
                mode: b.mode,
                level: levels.get(&b.catalog_score_id).cloned().flatten(),
                subscore: b.subscore,
                achieved_at_ms: b.achieved_at_ms,
            })
            .collect())
    }

    async fn snapshot_standings(&self, season_id: &str, mode: Mode) -> Result<Vec<GlobalScore>> {
        Ok(self
            .snapshots
            .lock()
            .unwrap()
            .get(&(season_id.to_string(), mode.as_str().to_string()))
            .cloned()
            .unwrap_or_default())
    }

    async fn write_snapshot(
        &self,
        season_id: &str,
        mode: Mode,
        standings: &[GlobalScore],
    ) -> Result<u64> {
        let mut map = self.snapshots.lock().unwrap();
        let key = (season_id.to_string(), mode.as_str().to_string());
        // Idempotent: an existing snapshot is never overwritten.
        if map.contains_key(&key) {
            return Ok(0);
        }
        map.insert(key, standings.to_vec());
        Ok(standings.len() as u64)
    }

    async fn snapshotted_seasons(&self) -> Result<Vec<String>> {
        let map = self.snapshots.lock().unwrap();
        let mut seasons: Vec<String> = map.keys().map(|(s, _)| s.clone()).collect();
        seasons.sort();
        seasons.dedup();
        seasons.reverse(); // most recent first (ids sort chronologically)
        Ok(seasons)
    }
}
