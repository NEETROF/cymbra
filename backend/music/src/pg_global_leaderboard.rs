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

//! Postgres-backed [`GlobalLeaderboardRepo`] (change: add-global-leaderboard) —
//! thin I/O glue (excluded from the coverage gate; exercised by the integration
//! tests). The monotonic-upsert and snapshot-idempotency semantics tested against
//! the fake are mirrored in SQL: the season-best upsert raises a row only via a
//! `WHERE` that encodes the ranking order, and the snapshot insert is
//! `ON CONFLICT DO NOTHING` so a re-delivered rollover never rewrites history.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::global_leaderboard::{
    GlobalLeaderboardRepo, GlobalScore, GlobalSeasonBest, SeasonBestRow,
};
use crate::leaderboard::Mode;

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("global leaderboard db: {e}"))
}

fn to_datetime(ms: i64) -> Result<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::from_timestamp_millis(ms)
        .ok_or_else(|| AppError::InvalidArgument("invalid achieved_at".into()))
}

/// Postgres implementation over the `music_svc` pool (search_path = `music`).
pub struct PgGlobalLeaderboardRepo {
    pool: PgPool,
}

impl PgGlobalLeaderboardRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GlobalLeaderboardRepo for PgGlobalLeaderboardRepo {
    async fn upsert_season_best(&self, best: &GlobalSeasonBest) -> Result<()> {
        let user_id = uuid::Uuid::parse_str(&best.user_id)
            .map_err(|_| AppError::InvalidArgument("invalid user id".into()))?;
        let catalog_score_id = uuid::Uuid::parse_str(&best.catalog_score_id)
            .map_err(|_| AppError::InvalidArgument("invalid catalog score id".into()))?;
        let achieved_at = to_datetime(best.achieved_at_ms)?;
        // Monotonic: raise the season best only when the new result is STRICTLY
        // better (higher sub-score; ties by an earlier achieved_at). A worse/equal
        // replay is a no-op — idempotent under at-least-once ingest.
        sqlx::query(
            "INSERT INTO music.global_season_bests \
             (user_id, season_id, catalog_score_id, mode, best_subscore, achieved_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (user_id, season_id, catalog_score_id, mode) DO UPDATE \
             SET best_subscore = EXCLUDED.best_subscore, \
                 achieved_at = EXCLUDED.achieved_at \
             WHERE EXCLUDED.best_subscore > music.global_season_bests.best_subscore \
                OR (EXCLUDED.best_subscore = music.global_season_bests.best_subscore \
                    AND EXCLUDED.achieved_at < music.global_season_bests.achieved_at)",
        )
        .bind(user_id)
        .bind(&best.season_id)
        .bind(catalog_score_id)
        .bind(best.mode.as_str())
        .bind(best.subscore)
        .bind(achieved_at)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn season_bests(&self, season_id: &str, mode: Mode) -> Result<Vec<SeasonBestRow>> {
        // Join the piece's catalog `level` here so the difficulty weighting is a
        // pure computation on the aggregation side (no per-row lookup).
        let rows = sqlx::query(
            "SELECT b.user_id::text AS user_id, b.catalog_score_id::text AS catalog_score_id, \
             c.level, b.best_subscore, \
             (extract(epoch FROM b.achieved_at) * 1000)::bigint AS achieved_at_ms \
             FROM music.global_season_bests b \
             JOIN music.catalog_scores c ON c.id = b.catalog_score_id \
             WHERE b.season_id = $1 AND b.mode = $2",
        )
        .bind(season_id)
        .bind(mode.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| SeasonBestRow {
                user_id: r.get::<String, _>("user_id"),
                catalog_score_id: r.get::<String, _>("catalog_score_id"),
                mode,
                level: r.get::<Option<String>, _>("level"),
                subscore: r.get("best_subscore"),
                achieved_at_ms: r.get::<i64, _>("achieved_at_ms"),
            })
            .collect())
    }

    async fn snapshot_standings(&self, season_id: &str, mode: Mode) -> Result<Vec<GlobalScore>> {
        let rows = sqlx::query(
            "SELECT user_id::text AS user_id, global_score, contributing_pieces, \
             was_listable, \
             (extract(epoch FROM reached_at) * 1000)::bigint AS reached_at_ms \
             FROM music.global_season_snapshots \
             WHERE season_id = $1 AND mode = $2 \
             ORDER BY global_score DESC, contributing_pieces DESC, reached_at ASC",
        )
        .bind(season_id)
        .bind(mode.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| GlobalScore {
                user_id: r.get::<String, _>("user_id"),
                score: r.get::<f32, _>("global_score") as f64,
                contributing_pieces: r.get::<i32, _>("contributing_pieces"),
                reached_at_ms: r.get::<i64, _>("reached_at_ms"),
                was_listable: r.get::<bool, _>("was_listable"),
            })
            .collect())
    }

    async fn write_snapshot(
        &self,
        season_id: &str,
        mode: Mode,
        standings: &[GlobalScore],
    ) -> Result<u64> {
        let mut written = 0;
        // `DO NOTHING` makes the whole rollover idempotent: a re-delivered job (or
        // one that crashed halfway) never rewrites an already-frozen standing.
        for s in standings {
            let user_id = uuid::Uuid::parse_str(&s.user_id)
                .map_err(|_| AppError::InvalidArgument("invalid user id".into()))?;
            let res = sqlx::query(
                "INSERT INTO music.global_season_snapshots \
                 (season_id, mode, user_id, global_score, contributing_pieces, reached_at, \
                  was_listable) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7) \
                 ON CONFLICT (season_id, mode, user_id) DO NOTHING",
            )
            .bind(season_id)
            .bind(mode.as_str())
            .bind(user_id)
            .bind(s.score as f32)
            .bind(s.contributing_pieces)
            .bind(to_datetime(s.reached_at_ms)?)
            .bind(s.was_listable)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
            written += res.rows_affected();
        }
        Ok(written)
    }

    async fn snapshotted_seasons(&self) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT DISTINCT season_id FROM music.global_season_snapshots \
             ORDER BY season_id DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>("season_id"))
            .collect())
    }
}
