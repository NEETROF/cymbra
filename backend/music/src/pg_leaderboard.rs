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

//! Postgres-backed [`LeaderboardRepo`] (change: add-play-leaderboards) — thin I/O
//! glue (excluded from the coverage gate; exercised by the integration tests). The
//! monotonic-upsert and ranking semantics tested against the fake are mirrored in
//! SQL: the upsert raises a best only via a `WHERE` that encodes the ranking order,
//! and the board read returns bests already ordered by it.

use std::collections::HashSet;

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::leaderboard::{LeaderboardBest, LeaderboardRepo, Mode, StoredBest};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("leaderboard db: {e}"))
}

fn to_datetime(ms: i64) -> Result<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::from_timestamp_millis(ms)
        .ok_or_else(|| AppError::InvalidArgument("invalid achieved_at".into()))
}

/// Postgres implementation over the `music_svc` pool (search_path = `music`).
pub struct PgLeaderboardRepo {
    pool: PgPool,
}

impl PgLeaderboardRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LeaderboardRepo for PgLeaderboardRepo {
    async fn is_accepted_catalog(&self, score_id: &str) -> Result<bool> {
        // A non-UUID id (or a user-upload id) is simply not an accepted catalog
        // piece — no board, no error.
        let Ok(id) = uuid::Uuid::parse_str(score_id) else {
            return Ok(false);
        };
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM music.catalog_scores \
             WHERE id = $1 AND moderation_status = 'accepted')",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(exists)
    }

    async fn percussion_pieces(&self, score_ids: &[String]) -> Result<HashSet<String>> {
        // Non-UUID ids (bundled slugs, upload ids) are not catalog pieces: no
        // instrument, no board, nothing to withhold.
        let ids: Vec<uuid::Uuid> = score_ids
            .iter()
            .filter_map(|s| uuid::Uuid::parse_str(s).ok())
            .collect();
        if ids.is_empty() {
            return Ok(HashSet::new());
        }
        let rows = sqlx::query(
            "SELECT id::text AS id FROM music.catalog_scores \
             WHERE id = ANY($1) AND instrument = 'percussion'",
        )
        .bind(&ids)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows.into_iter().map(|r| r.get::<String, _>("id")).collect())
    }

    async fn upsert_best(&self, best: &LeaderboardBest) -> Result<()> {
        let user_id = uuid::Uuid::parse_str(&best.user_id)
            .map_err(|_| AppError::InvalidArgument("invalid user id".into()))?;
        let catalog_score_id = uuid::Uuid::parse_str(&best.catalog_score_id)
            .map_err(|_| AppError::InvalidArgument("invalid catalog score id".into()))?;
        let achieved_at = to_datetime(best.achieved_at_ms)?;
        // Monotonic: raise the best only when the new result is STRICTLY better
        // under the ranking order (higher sub-score; ties by smaller tie-break,
        // then earlier achieved_at). A worse/equal replay is a no-op — idempotent.
        sqlx::query(
            "INSERT INTO music.leaderboard_bests \
             (user_id, catalog_score_id, mode, best_subscore, tiebreak_metric, achieved_at, \
              best_overall_sync_pct) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (user_id, catalog_score_id, mode) DO UPDATE \
             SET best_subscore = CASE WHEN better THEN EXCLUDED.best_subscore \
                                      ELSE music.leaderboard_bests.best_subscore END, \
                 tiebreak_metric = CASE WHEN better THEN EXCLUDED.tiebreak_metric \
                                        ELSE music.leaderboard_bests.tiebreak_metric END, \
                 achieved_at = CASE WHEN better THEN EXCLUDED.achieved_at \
                                    ELSE music.leaderboard_bests.achieved_at END, \
                 best_overall_sync_pct = GREATEST(music.leaderboard_bests.best_overall_sync_pct, \
                                                  EXCLUDED.best_overall_sync_pct) \
             FROM (SELECT ($4::real > b.best_subscore \
                        OR ($4::real = b.best_subscore AND $5::real < b.tiebreak_metric) \
                        OR ($4::real = b.best_subscore AND $5::real = b.tiebreak_metric \
                            AND $6::timestamptz < b.achieved_at)) AS better \
                   FROM music.leaderboard_bests b \
                   WHERE b.user_id = $1 AND b.catalog_score_id = $2 AND b.mode = $3) AS cmp",
        )
        .bind(user_id)
        .bind(catalog_score_id)
        .bind(best.mode.as_str())
        .bind(best.subscore)
        .bind(best.tiebreak_metric)
        .bind(achieved_at)
        .bind(best.overall_sync_pct)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn board_bests(&self, score_id: &str, mode: Mode) -> Result<Vec<StoredBest>> {
        let Ok(id) = uuid::Uuid::parse_str(score_id) else {
            return Ok(Vec::new());
        };
        let rows = sqlx::query(
            "SELECT user_id::text AS user_id, best_subscore, tiebreak_metric, \
             (extract(epoch FROM achieved_at) * 1000)::bigint AS achieved_at_ms, \
             best_overall_sync_pct \
             FROM music.leaderboard_bests \
             WHERE catalog_score_id = $1 AND mode = $2 \
             ORDER BY best_subscore DESC, tiebreak_metric ASC, achieved_at ASC",
        )
        .bind(id)
        .bind(mode.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| StoredBest {
                user_id: r.get::<String, _>("user_id"),
                subscore: r.get("best_subscore"),
                tiebreak_metric: r.get("tiebreak_metric"),
                achieved_at_ms: r.get::<i64, _>("achieved_at_ms"),
                best_overall_sync_pct: r.get::<Option<f32>, _>("best_overall_sync_pct"),
            })
            .collect())
    }
}
