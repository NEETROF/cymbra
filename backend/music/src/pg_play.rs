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

//! Postgres-backed [`PlayRepo`] (change: add-play-activity-profile) — thin I/O
//! glue (excluded from the coverage gate; exercised by the integration tests).
//! Ingestion is idempotent by the client session id (`ON CONFLICT (id) DO
//! NOTHING`).

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::play::{PlayRepo, PlaySession, SessionPoint};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("play db: {e}"))
}

fn parse_uuid(s: &str, what: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(s).map_err(|_| AppError::InvalidArgument(format!("invalid {what}")))
}

/// Postgres implementation over the `music_svc` pool (search_path = `music`).
pub struct PgPlayRepo {
    pool: PgPool,
}

impl PgPlayRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlayRepo for PgPlayRepo {
    async fn record(&self, s: &PlaySession) -> Result<()> {
        let id = parse_uuid(&s.session_id, "session id")?;
        let user_id = parse_uuid(&s.user_id, "user id")?;
        let played_at = chrono::DateTime::from_timestamp_millis(s.played_at_ms)
            .ok_or_else(|| AppError::InvalidArgument("invalid played_at".into()))?;
        // Empty detail ⇒ SQL NULL (no heavy tier to store/prune).
        let detail: Option<&str> = if s.session_result_json.is_empty() {
            None
        } else {
            Some(&s.session_result_json)
        };
        sqlx::query(
            "INSERT INTO music.play_sessions \
             (id, user_id, score_id, played_at, tz_offset_minutes, overall_sync_pct, session_result) \
             VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(user_id)
        .bind(&s.score_id)
        .bind(played_at)
        .bind(s.tz_offset_minutes)
        .bind(s.overall_sync_pct)
        .bind(detail)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn session_points(&self, user_id: &str) -> Result<Vec<SessionPoint>> {
        let uid = parse_uuid(user_id, "user id")?;
        let rows = sqlx::query(
            "SELECT (extract(epoch FROM played_at) * 1000)::bigint AS played_at_ms, \
             tz_offset_minutes, overall_sync_pct \
             FROM music.play_sessions WHERE user_id = $1 ORDER BY played_at",
        )
        .bind(uid)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| SessionPoint {
                played_at_ms: r.get::<i64, _>("played_at_ms"),
                tz_offset_minutes: r.get("tz_offset_minutes"),
                overall_sync_pct: r.get("overall_sync_pct"),
            })
            .collect())
    }
}
