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

//! Postgres-backed [`UserCollectionRepo`] — thin I/O glue (exercised by the
//! integration tests, not the unit gate).
//!
//! Runtime `sqlx::query(...).bind(...)`, fully-qualified table names, every
//! statement owner-scoped — same shape as [`crate::pg_user_scores`]. The
//! case-insensitive name uniqueness is the DB's (`lower(name)` functional unique
//! index): a violation is translated here into [`AppError::AlreadyExists`] so the
//! app localises "that name is taken" instead of surfacing a constraint error.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::user_collections::{Collection, UserCollectionRepo};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("user_collections db: {e}"))
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .map(|d| d.is_unique_violation())
        .unwrap_or(false)
}

/// A malformed id is "not found" rather than a 500: ids come from clients.
fn parse_uuid(s: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(s).map_err(|_| AppError::NotFound("collection not found".into()))
}

fn name_taken() -> AppError {
    AppError::AlreadyExists("collection name taken".into())
}

/// Postgres [`UserCollectionRepo`].
pub struct PgUserCollectionRepo {
    pool: PgPool,
}

impl PgUserCollectionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Whether the collection exists AND belongs to the owner. Every membership and
    /// read path goes through this first, so a foreign collection is indistinguishable
    /// from a missing one.
    async fn owns(&self, collection_id: uuid::Uuid, owner: uuid::Uuid) -> Result<bool> {
        let found = sqlx::query(
            "SELECT 1 FROM music.user_score_collections WHERE id = $1 AND owner_id = $2",
        )
        .bind(collection_id)
        .bind(owner)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        Ok(found.is_some())
    }
}

#[async_trait]
impl UserCollectionRepo for PgUserCollectionRepo {
    async fn create(&self, c: &Collection) -> Result<()> {
        let created: DateTime<Utc> =
            DateTime::from_timestamp(c.created_at, 0).unwrap_or_else(Utc::now);
        sqlx::query(
            "INSERT INTO music.user_score_collections (id, owner_id, name, created_at) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(parse_uuid(&c.id)?)
        .bind(parse_uuid(&c.owner_id)?)
        .bind(&c.name)
        .bind(created)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                name_taken()
            } else {
                internal(e)
            }
        })?;
        Ok(())
    }

    async fn rename(&self, id: &str, owner_id: &str, name: &str) -> Result<()> {
        let (id, owner) = (parse_uuid(id)?, parse_uuid(owner_id)?);
        let res = sqlx::query(
            "UPDATE music.user_score_collections SET name = $3 WHERE id = $1 AND owner_id = $2",
        )
        .bind(id)
        .bind(owner)
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                name_taken()
            } else {
                internal(e)
            }
        })?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("collection not found".into()));
        }
        Ok(())
    }

    async fn delete(&self, id: &str, owner_id: &str) -> Result<()> {
        // Memberships go with it through the FK cascade; no score is touched.
        let res =
            sqlx::query("DELETE FROM music.user_score_collections WHERE id = $1 AND owner_id = $2")
                .bind(parse_uuid(id)?)
                .bind(parse_uuid(owner_id)?)
                .execute(&self.pool)
                .await
                .map_err(internal)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("collection not found".into()));
        }
        Ok(())
    }

    async fn list(&self, owner_id: &str) -> Result<Vec<Collection>> {
        let rows = sqlx::query(
            "SELECT id, owner_id, name, created_at FROM music.user_score_collections \
             WHERE owner_id = $1 ORDER BY created_at DESC, id DESC",
        )
        .bind(parse_uuid(owner_id)?)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .iter()
            .map(|r| Collection {
                id: r.get::<uuid::Uuid, _>("id").to_string(),
                owner_id: r.get::<uuid::Uuid, _>("owner_id").to_string(),
                name: r.get("name"),
                created_at: r.get::<DateTime<Utc>, _>("created_at").timestamp(),
            })
            .collect())
    }

    async fn add_item(&self, collection_id: &str, score_id: &str, owner_id: &str) -> Result<()> {
        let (cid, sid, owner) = (
            parse_uuid(collection_id)?,
            parse_uuid(score_id)?,
            parse_uuid(owner_id)?,
        );
        if !self.owns(cid, owner).await? {
            return Err(AppError::NotFound("collection not found".into()));
        }
        // The score half of the ownership check rides in the INSERT itself: the SELECT
        // yields no row unless the score is the caller's, so a foreign score inserts
        // nothing rather than leaking that it exists.
        let res = sqlx::query(
            "INSERT INTO music.user_score_collection_items (collection_id, user_score_id) \
             SELECT $1, id FROM music.user_scores WHERE id = $2 AND owner_id = $3 \
             ON CONFLICT DO NOTHING",
        )
        .bind(cid)
        .bind(sid)
        .bind(owner)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        if res.rows_affected() == 0 {
            // Either the score is not the caller's (refuse) or the membership already
            // existed (idempotent success) — distinguish with one cheap lookup.
            let already = sqlx::query(
                "SELECT 1 FROM music.user_score_collection_items \
                 WHERE collection_id = $1 AND user_score_id = $2",
            )
            .bind(cid)
            .bind(sid)
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
            if already.is_none() {
                return Err(AppError::NotFound("score not found".into()));
            }
        }
        Ok(())
    }

    async fn remove_item(&self, collection_id: &str, score_id: &str, owner_id: &str) -> Result<()> {
        let (cid, owner) = (parse_uuid(collection_id)?, parse_uuid(owner_id)?);
        if !self.owns(cid, owner).await? {
            return Err(AppError::NotFound("collection not found".into()));
        }
        // Idempotent: removing an absent membership affects no row and is a success.
        sqlx::query(
            "DELETE FROM music.user_score_collection_items \
             WHERE collection_id = $1 AND user_score_id = $2",
        )
        .bind(cid)
        .bind(parse_uuid(score_id)?)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn list_score_ids(&self, collection_id: &str, owner_id: &str) -> Result<Vec<String>> {
        let (cid, owner) = (parse_uuid(collection_id)?, parse_uuid(owner_id)?);
        if !self.owns(cid, owner).await? {
            return Err(AppError::NotFound("collection not found".into()));
        }
        let rows = sqlx::query(
            "SELECT user_score_id FROM music.user_score_collection_items \
             WHERE collection_id = $1 ORDER BY added_at DESC, user_score_id DESC",
        )
        .bind(cid)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .iter()
            .map(|r| r.get::<uuid::Uuid, _>("user_score_id").to_string())
            .collect())
    }
}
