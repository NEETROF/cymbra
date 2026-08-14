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

//! Postgres-backed [`BadgeRepo`] (change: add-achievement-badges) — thin I/O glue
//! (excluded from the coverage gate; the registry + awarding semantics are proven
//! against `MockBadgeRepo` in [`crate::badges_module`]).
//!
//! Two things are worth knowing about the SQL here:
//!
//! * **The local-day lists** mirror [`crate::play_core::local_day`] exactly:
//!   `played_at AT TIME ZONE 'UTC'` drops to a naive UTC timestamp, the session's
//!   recorded `tz_offset_minutes` shifts it, and only then is it truncated to a
//!   date. Casting the `timestamptz` directly would silently bucket by the
//!   *server's* TimeZone setting instead of the player's. Scored sessions and
//!   scoreless practice are read as two lists, kept apart here and unioned by the
//!   fold — consistency counts both, play counts only the scored ones.
//! * **Placement** is a correlated "how many are strictly better" probe rather
//!   than a window function, so it reads through
//!   `leaderboard_bests_board_idx (catalog_score_id, mode, best_subscore DESC)`
//!   per board instead of ranking every board in the table. Ties are generous: a
//!   three-way tie for first leaves all three inside the top 3.
//!
//! The curation counters are NOT re-derived here — they come from
//! [`CurationRewardsRepo::curator_metrics`], the one place that already knows what
//! "aligned" and "first rater" mean.

use std::collections::HashMap;

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::badges::{BadgeRepo, RawBadgeCounters};
use crate::badges_core::{HIGH_ACCURACY_PCT, TOP_PLACEMENT};
use crate::curation_rewards::{CurationRewardsRepo, GrantKind};
use crate::curation_rewards_core::{RewardConfig, coverage_base};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("badges db: {e}"))
}

fn uid(user_id: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(user_id).map_err(|_| AppError::InvalidArgument("invalid user id".into()))
}

/// Postgres implementation over the `music_svc` pool (search_path = `music`).
pub struct PgBadgeRepo {
    pool: PgPool,
    /// The curation counters' existing home — reused rather than re-queried, so
    /// there is only ever one definition of an "aligned" or "first rater" rating.
    curation: std::sync::Arc<dyn CurationRewardsRepo>,
    /// Classifies aligned vs misaligned settled ratings and identifies first-rater
    /// coverage, exactly as `CurationRewardsModule::metrics` does.
    config: RewardConfig,
}

impl PgBadgeRepo {
    pub fn new(pool: PgPool, curation: std::sync::Arc<dyn CurationRewardsRepo>) -> Self {
        Self {
            pool,
            curation,
            config: RewardConfig::default(),
        }
    }

    /// Override the reward configuration, so the curation counters are classified
    /// with the same tuning the rewards module runs with.
    pub fn with_config(mut self, config: RewardConfig) -> Self {
        self.config = config;
        self
    }
}

#[async_trait]
impl BadgeRepo for PgBadgeRepo {
    async fn counters(&self, user_id: &str) -> Result<RawBadgeCounters> {
        let id = uid(user_id)?;

        // --- curation: the existing metrics read, not a second definition ---
        let metrics = self
            .curation
            .curator_metrics(
                user_id,
                self.config.honesty_floor,
                coverage_base(0, &self.config),
            )
            .await?;

        // --- play: one pass over the user's sessions ---
        let play = sqlx::query(
            "SELECT count(*)::bigint AS sessions, \
             count(DISTINCT score_id)::bigint AS pieces, \
             count(*) FILTER (WHERE overall_sync_pct >= $2)::bigint AS accurate \
             FROM music.play_sessions WHERE user_id = $1",
        )
        .bind(id)
        .bind(HIGH_ACCURACY_PCT as f32)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        // --- consistency: the player's distinct LOCAL days (folded in Rust) ---
        let day_rows = sqlx::query(
            "SELECT DISTINCT \
             ((played_at AT TIME ZONE 'UTC') + make_interval(mins => tz_offset_minutes))::date \
             AS local_day \
             FROM music.play_sessions WHERE user_id = $1",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        let play_days = day_rows
            .iter()
            .map(|r| r.get::<chrono::NaiveDate, _>("local_day"))
            .collect();

        // Scoreless practice (a measure-range run) is read SEPARATELY and stays
        // separate on the wire home: the fold unions it into the consistency
        // counters only. Same local-day shift, so a day both played and drilled
        // collapses to one.
        let practice_rows = sqlx::query(
            "SELECT DISTINCT \
             ((practiced_at AT TIME ZONE 'UTC') + make_interval(mins => tz_offset_minutes))::date \
             AS local_day \
             FROM music.practice_sessions WHERE user_id = $1",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        let practice_days = practice_rows
            .iter()
            .map(|r| r.get::<chrono::NaiveDate, _>("local_day"))
            .collect();

        // --- ranking: boards entered, and how many of them are podiums ---
        let boards = sqlx::query(
            "SELECT count(*)::bigint AS entered, \
             count(*) FILTER (WHERE better < $2)::bigint AS podiums FROM ( \
               SELECT (SELECT count(*) FROM music.leaderboard_bests o \
                       WHERE o.catalog_score_id = b.catalog_score_id \
                         AND o.mode = b.mode \
                         AND o.best_subscore > b.best_subscore) AS better \
               FROM music.leaderboard_bests b WHERE b.user_id = $1 \
             ) ranked",
        )
        .bind(id)
        .bind(TOP_PLACEMENT)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        let season_podiums: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM ( \
               SELECT (SELECT count(*) FROM music.global_season_snapshots o \
                       WHERE o.season_id = s.season_id AND o.mode = s.mode \
                         AND o.global_score > s.global_score) AS better \
               FROM music.global_season_snapshots s WHERE s.user_id = $1 \
             ) standings WHERE better < $2",
        )
        .bind(id)
        .bind(TOP_PLACEMENT)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        // --- contribution: accepted only, in both catalogs ---
        let accepted_proposals: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM music.catalog_scores \
             WHERE proposed_by = $1 AND moderation_status = 'accepted'",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        let accepted_soundfonts: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM music.soundfonts \
             WHERE uploaded_by = $1 AND moderation_status = 'accepted'",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        // --- learning: completions, not merely started courses ---
        let courses_completed: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM music.course_progress \
             WHERE user_id = $1 AND completed_at IS NOT NULL",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        Ok(RawBadgeCounters {
            rating_count: metrics.total_ratings,
            aligned_count: metrics.aligned_count,
            first_rater_count: metrics.first_rater_count,
            session_count: play.get("sessions"),
            distinct_pieces: play.get("pieces"),
            high_accuracy_sessions: play.get("accurate"),
            play_days,
            practice_days,
            ranked_boards: boards.get("entered"),
            top_three_finishes: boards.get("podiums"),
            season_podiums,
            accepted_proposals,
            accepted_soundfonts,
            courses_completed,
        })
    }

    async fn granted_badges(&self, user_id: &str) -> Result<HashMap<String, i64>> {
        let rows = sqlx::query(
            "SELECT key, (extract(epoch FROM granted_at) * 1000)::bigint AS granted_at_ms \
             FROM music.curation_grants WHERE user_id = $1 AND grant_kind = 'badge'",
        )
        .bind(uid(user_id)?)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .iter()
            .map(|r| (r.get::<String, _>("key"), r.get::<i64, _>("granted_at_ms")))
            .collect())
    }

    async fn insert_grant(&self, user_id: &str, key: &str) -> Result<bool> {
        // The (user_id, key) primary key IS the idempotency guard: a repeated
        // evaluation neither inserts a second row nor moves `granted_at`.
        let res = sqlx::query(
            "INSERT INTO music.curation_grants (user_id, grant_kind, key) VALUES ($1, $2, $3) \
             ON CONFLICT (user_id, key) DO NOTHING",
        )
        .bind(uid(user_id)?)
        .bind(GrantKind::Badge.as_str())
        .bind(key)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(res.rows_affected() > 0)
    }
}
