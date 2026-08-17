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

use crate::offline_secret::OfflineSecretRepo;
use crate::pg::{META_COLS, bind_meta, meta_from_row};
use crate::user_library::UserLibraryRepo;
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

fn is_foreign_key_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .map(|d| d.is_foreign_key_violation())
        .unwrap_or(false)
}

/// The `user_scores` columns: owner/rights/lifecycle first, then the shared
/// [`META_COLS`] block — matching `row_to_score` / the insert bind order.
fn user_cols() -> String {
    format!(
        "id, owner_id, level, rights_basis, rights_ack, sha256, size_bytes, \
         object_key, created_at, favorite, proposed_catalog_id, {META_COLS}"
    )
}

fn row_to_score(r: &PgRow) -> UserScore {
    UserScore {
        id: r.get::<uuid::Uuid, _>("id").to_string(),
        owner_id: r.get::<uuid::Uuid, _>("owner_id").to_string(),
        level: r.get("level"),
        rights_basis: r.get("rights_basis"),
        rights_ack: r.get("rights_ack"),
        sha256: r.get("sha256"),
        size_bytes: r.get("size_bytes"),
        object_key: r.get("object_key"),
        created_at: r.get::<DateTime<Utc>, _>("created_at").timestamp(),
        favorite: r.get("favorite"),
        proposed_catalog_id: r
            .get::<Option<uuid::Uuid>, _>("proposed_catalog_id")
            .map(|u| u.to_string()),
        meta: meta_from_row(r),
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
        // Owner/rights/lifecycle columns ($1..$10), then the shared ScoreMeta
        // block ($11..$28) via `bind_meta`.
        let proposed = s
            .proposed_catalog_id
            .as_deref()
            .and_then(|p| uuid::Uuid::parse_str(p).ok());
        let sql = format!(
            "INSERT INTO music.user_scores ({}) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,\
                $19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29)",
            user_cols()
        );
        let q = sqlx::query(&sql)
            .bind(id)
            .bind(owner)
            .bind(&s.level)
            .bind(&s.rights_basis)
            .bind(s.rights_ack)
            .bind(&s.sha256)
            .bind(s.size_bytes)
            .bind(&s.object_key)
            .bind(created)
            .bind(s.favorite)
            .bind(proposed);
        let res = bind_meta(q, &s.meta).execute(&self.pool).await;
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
            "SELECT {} FROM music.user_scores WHERE owner_id = $1 ORDER BY created_at DESC",
            user_cols()
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
            "SELECT {} FROM music.user_scores WHERE id = $1 AND owner_id = $2",
            user_cols()
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
            "DELETE FROM music.user_scores WHERE id = $1 AND owner_id = $2 RETURNING {}",
            user_cols()
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

    async fn count_library(&self, owner_id: &str) -> Result<i64> {
        let owner = parse_uuid(owner_id)?;
        // Uploads accepted into the public catalog no longer count against the
        // private library cap (they became public content).
        let row = sqlx::query(
            "SELECT count(*) AS n FROM music.user_scores s \
             WHERE s.owner_id = $1 AND NOT EXISTS ( \
               SELECT 1 FROM music.catalog_scores c \
               WHERE c.id = s.proposed_catalog_id AND c.moderation_status = 'accepted')",
        )
        .bind(owner)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(row.get::<i64, _>("n"))
    }

    async fn set_favorite(&self, id: &str, owner_id: &str, favorite: bool) -> Result<()> {
        let (id, owner) = (parse_uuid(id)?, parse_uuid(owner_id)?);
        let res = sqlx::query(
            "UPDATE music.user_scores SET favorite = $3 WHERE id = $1 AND owner_id = $2",
        )
        .bind(id)
        .bind(owner)
        .bind(favorite)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("score not found".into()));
        }
        Ok(())
    }

    async fn set_proposed_catalog_id(
        &self,
        id: &str,
        owner_id: &str,
        catalog_id: &str,
    ) -> Result<()> {
        let (id, owner) = (parse_uuid(id)?, parse_uuid(owner_id)?);
        let catalog = uuid::Uuid::parse_str(catalog_id)
            .map_err(|_| AppError::Internal(anyhow::anyhow!("bad catalog id")))?;
        let res = sqlx::query(
            "UPDATE music.user_scores SET proposed_catalog_id = $3 WHERE id = $1 AND owner_id = $2",
        )
        .bind(id)
        .bind(owner)
        .bind(catalog)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("score not found".into()));
        }
        Ok(())
    }
}

/// Postgres-backed [`UserLibraryRepo`] over the `music_svc` pool — the per-user
/// saved-catalog library (change: score-hub-search). Every statement is
/// owner-scoped; `save`/`remove` are idempotent.
pub struct PgUserLibraryRepo {
    pool: PgPool,
}

impl PgUserLibraryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserLibraryRepo for PgUserLibraryRepo {
    async fn save(&self, owner_id: &str, catalog_id: &str) -> Result<()> {
        let owner = parse_uuid(owner_id)?;
        // A malformed catalog id can never reference a real row → not found.
        let catalog = uuid::Uuid::parse_str(catalog_id)
            .map_err(|_| AppError::NotFound("catalog score not found".into()))?;
        let res = sqlx::query(
            "INSERT INTO music.user_library (owner_id, catalog_id) \
             VALUES ($1, $2) ON CONFLICT (owner_id, catalog_id) DO NOTHING",
        )
        .bind(owner)
        .bind(catalog)
        .execute(&self.pool)
        .await;
        match res {
            Ok(_) => Ok(()),
            // A foreign-key violation means the catalog id doesn't exist.
            Err(e) if is_foreign_key_violation(&e) => {
                Err(AppError::NotFound("catalog score not found".into()))
            }
            Err(e) => Err(internal(e)),
        }
    }

    async fn remove(&self, owner_id: &str, catalog_id: &str) -> Result<()> {
        let owner = parse_uuid(owner_id)?;
        let Ok(catalog) = uuid::Uuid::parse_str(catalog_id) else {
            return Ok(()); // malformed id was never saved → no-op success
        };
        sqlx::query("DELETE FROM music.user_library WHERE owner_id = $1 AND catalog_id = $2")
            .bind(owner)
            .bind(catalog)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }

    async fn list_ids(&self, owner_id: &str) -> Result<Vec<String>> {
        let owner = parse_uuid(owner_id)?;
        let rows = sqlx::query(
            "SELECT catalog_id FROM music.user_library \
             WHERE owner_id = $1 ORDER BY created_at DESC, catalog_id",
        )
        .bind(owner)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .iter()
            .map(|r| r.get::<uuid::Uuid, _>("catalog_id").to_string())
            .collect())
    }
}

/// Postgres-backed [`OfflineSecretRepo`] over the `music_svc` pool — the per-user
/// offline-cache secret (change: add-offline-score-cache). Every statement is
/// owner-scoped by `user_id`; the secret value is never logged (sqlx errors are
/// funneled through the generic [`internal`] without binding the value into any
/// message).
pub struct PgOfflineSecretRepo {
    pool: PgPool,
}

impl PgOfflineSecretRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OfflineSecretRepo for PgOfflineSecretRepo {
    async fn get(&self, user_id: &str) -> Result<Option<Vec<u8>>> {
        let uid = parse_uuid(user_id)?;
        let row = sqlx::query("SELECT secret FROM music.offline_cache_secrets WHERE user_id = $1")
            .bind(uid)
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        Ok(row.map(|r| r.get::<Vec<u8>, _>("secret")))
    }

    async fn create_if_absent(&self, user_id: &str, candidate: &[u8]) -> Result<Vec<u8>> {
        let uid = parse_uuid(user_id)?;
        // Atomic get-or-create: insert the candidate, but on a pre-existing row do
        // nothing; the `RETURNING` from the INSERT is empty on conflict, so a second
        // SELECT resolves the value that actually won (existing on a race). One
        // round-trip in the common (create) path.
        let inserted = sqlx::query_scalar::<_, Vec<u8>>(
            "INSERT INTO music.offline_cache_secrets (user_id, secret) VALUES ($1, $2) \
             ON CONFLICT (user_id) DO NOTHING RETURNING secret",
        )
        .bind(uid)
        .bind(candidate)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        if let Some(secret) = inserted {
            return Ok(secret);
        }
        // Conflict: a secret already existed — return the stored value unchanged.
        self.get(user_id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("offline secret vanished on race")))
    }

    async fn rotate(&self, user_id: &str, secret: &[u8]) -> Result<()> {
        let uid = parse_uuid(user_id)?;
        sqlx::query(
            "INSERT INTO music.offline_cache_secrets (user_id, secret) VALUES ($1, $2) \
             ON CONFLICT (user_id) DO UPDATE SET secret = EXCLUDED.secret, rotated_at = now()",
        )
        .bind(uid)
        .bind(secret)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }
}
