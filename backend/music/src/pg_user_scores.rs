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

//! Postgres-backed [`UserScoreRepo`] — thin I/O glue (exercised by the
//! integration tests, not the unit gate).
//!
//! Runtime `sqlx::query(...).bind(...)` API (not the compile-time macros),
//! matching `PgCatalogRepo`/`PgUserRepo`. Table names are fully qualified
//! (`music.user_scores`) so they resolve regardless of the connecting role's
//! `search_path`. Every statement is owner-scoped.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row, postgres::PgRow};

use crate::user_scores::{UserScore, UserScoreRepo};

/// Maps a sqlx error to an internal `AppError` (no detail leaked to clients).
fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("user_scores db: {e}"))
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .map(|d| d.is_unique_violation())
        .unwrap_or(false)
}

const COLS: &str = "id, owner_id, level, rights_basis, rights_ack, title, composer, \
     title_norm, work_key, key_fifths, time_sig, measure_count, is_piano, sha256, \
     size_bytes, object_key, created_at";

fn row_to_score(r: &PgRow) -> UserScore {
    UserScore {
        id: r.get::<uuid::Uuid, _>("id").to_string(),
        owner_id: r.get::<uuid::Uuid, _>("owner_id").to_string(),
        level: r.get("level"),
        rights_basis: r.get("rights_basis"),
        rights_ack: r.get("rights_ack"),
        title: r.get("title"),
        composer: r.get("composer"),
        title_norm: r.get("title_norm"),
        work_key: r.get("work_key"),
        key_fifths: r.get("key_fifths"),
        time_sig: r.get("time_sig"),
        measure_count: r.get("measure_count"),
        is_piano: r.get("is_piano"),
        sha256: r.get("sha256"),
        size_bytes: r.get("size_bytes"),
        object_key: r.get("object_key"),
        created_at: r.get::<DateTime<Utc>, _>("created_at").timestamp(),
    }
}

/// Parse an id/owner text UUID; a malformed id is a "not found", never a 500.
fn parse_uuid(s: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(s).map_err(|_| AppError::NotFound("score not found".into()))
}

/// Postgres implementation over the `music_svc` pool.
pub struct PgUserScoreRepo {
    pool: PgPool,
}

impl PgUserScoreRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserScoreRepo for PgUserScoreRepo {
    async fn insert(&self, s: &UserScore) -> Result<()> {
        let id = parse_uuid(&s.id)?;
        let owner = parse_uuid(&s.owner_id)?;
        let created = DateTime::from_timestamp(s.created_at, 0)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("bad created_at")))?;
        let res = sqlx::query(&format!(
            "INSERT INTO music.user_scores ({COLS}) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)"
        ))
        .bind(id)
        .bind(owner)
        .bind(&s.level)
        .bind(&s.rights_basis)
        .bind(s.rights_ack)
        .bind(&s.title)
        .bind(&s.composer)
        .bind(&s.title_norm)
        .bind(&s.work_key)
        .bind(s.key_fifths)
        .bind(&s.time_sig)
        .bind(s.measure_count)
        .bind(s.is_piano)
        .bind(&s.sha256)
        .bind(s.size_bytes)
        .bind(&s.object_key)
        .bind(created)
        .execute(&self.pool)
        .await;
        match res {
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => {
                Err(AppError::AlreadyExists("score already uploaded".into()))
            }
            Err(e) => Err(internal(e)),
        }
    }

    async fn list_by_owner(&self, owner_id: &str) -> Result<Vec<UserScore>> {
        let owner = parse_uuid(owner_id)?;
        let rows = sqlx::query(&format!(
            "SELECT {COLS} FROM music.user_scores WHERE owner_id = $1 ORDER BY created_at DESC"
        ))
        .bind(owner)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows.iter().map(row_to_score).collect())
    }

    async fn get_owned(&self, id: &str, owner_id: &str) -> Result<Option<UserScore>> {
        let (id, owner) = (parse_uuid(id)?, parse_uuid(owner_id)?);
        let row = sqlx::query(&format!(
            "SELECT {COLS} FROM music.user_scores WHERE id = $1 AND owner_id = $2"
        ))
        .bind(id)
        .bind(owner)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        Ok(row.as_ref().map(row_to_score))
    }

    async fn delete_owned(&self, id: &str, owner_id: &str) -> Result<Option<UserScore>> {
        let (id, owner) = (parse_uuid(id)?, parse_uuid(owner_id)?);
        let row = sqlx::query(&format!(
            "DELETE FROM music.user_scores WHERE id = $1 AND owner_id = $2 RETURNING {COLS}"
        ))
        .bind(id)
        .bind(owner)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        Ok(row.as_ref().map(row_to_score))
    }

    async fn count_recent(&self, owner_id: &str, window_days: u32) -> Result<i64> {
        let owner = parse_uuid(owner_id)?;
        let row = sqlx::query(
            "SELECT count(*) AS n FROM music.user_scores \
             WHERE owner_id = $1 AND created_at >= now() - make_interval(days => $2)",
        )
        .bind(owner)
        .bind(window_days as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(row.get::<i64, _>("n"))
    }
}
