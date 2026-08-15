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

//! Postgres-backed [`CatalogDayAccessRepo`] (change: add-score-daily-access-
//! rewards) — thin I/O glue (excluded from the coverage gate; the rules it serves
//! are proven in `catalog_daily_access_core` / `catalog_daily_access` against the
//! in-memory fake).
//!
//! [`PgCatalogDayAccessRepo::spend_day_slot`] is ONE transaction shaped like the
//! streak freeze: it serialises the user's spends (there is no per-user row to
//! `FOR UPDATE`, so a transaction-scoped advisory lock on the user id plays that
//! role), re-checks the ledger balance under that lock, then writes both the
//! negative `curation_points` entry — `ON CONFLICT (user_id, award_key) DO
//! NOTHING`, the charge-once guard — and the paid day row. A crash mid-way leaves
//! neither write.

use async_trait::async_trait;
use chrono::NaiveDate;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::catalog_daily_access::{CatalogDayAccessRepo, DAY_SLOT_REWARD_KEY};
use crate::catalog_daily_access_core::DayState;

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("catalog day access db: {e}"))
}

fn parse_uuid(s: &str, what: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(s).map_err(|_| AppError::InvalidArgument(format!("invalid {what}")))
}

/// Postgres implementation over a pool with write access to the `music` schema.
pub struct PgCatalogDayAccessRepo {
    pool: PgPool,
}

impl PgCatalogDayAccessRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatalogDayAccessRepo for PgCatalogDayAccessRepo {
    async fn day_state(&self, user_id: &str, day: NaiveDate) -> Result<DayState> {
        let uid = parse_uuid(user_id, "user id")?;
        let rows = sqlx::query(
            "SELECT catalog_id::text AS catalog_id, paid \
             FROM music.catalog_day_access WHERE user_id = $1 AND day = $2",
        )
        .bind(uid)
        .bind(day)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        let mut state = DayState::default();
        for r in rows {
            let id: String = r.get("catalog_id");
            if r.get::<bool, _>("paid") {
                state.paid.insert(id.clone());
            }
            state.opened.insert(id);
        }
        Ok(state)
    }

    async fn record_open(&self, user_id: &str, catalog_id: &str, day: NaiveDate) -> Result<()> {
        let uid = parse_uuid(user_id, "user id")?;
        let cid = parse_uuid(catalog_id, "catalog id")?;
        // A re-open is a no-op; an existing PAID row is never downgraded.
        sqlx::query(
            "INSERT INTO music.catalog_day_access (user_id, catalog_id, day, paid) \
             VALUES ($1, $2, $3, FALSE) \
             ON CONFLICT (user_id, catalog_id, day) DO NOTHING",
        )
        .bind(uid)
        .bind(cid)
        .bind(day)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn spendable_balance(&self, user_id: &str) -> Result<i64> {
        let uid = parse_uuid(user_id, "user id")?;
        let balance: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0)::bigint FROM music.curation_points WHERE user_id = $1",
        )
        .bind(uid)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(balance)
    }

    async fn spend_day_slot(
        &self,
        user_id: &str,
        catalog_id: &str,
        day: NaiveDate,
        cost: i64,
        award_key: &str,
    ) -> Result<bool> {
        let uid = parse_uuid(user_id, "user id")?;
        let cid = parse_uuid(catalog_id, "catalog id")?;
        let mut tx = self.pool.begin().await.map_err(internal)?;
        // Serialise concurrent confirms of the same user: the second one blocks
        // here, then re-reads a balance that already reflects the first charge
        // (and is refused if it no longer covers the cost). Released at commit.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
        let balance: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0)::bigint FROM music.curation_points WHERE user_id = $1",
        )
        .bind(uid)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;
        if balance < cost {
            // Nothing written; the transaction is dropped without committing.
            return Ok(false);
        }
        // `DO NOTHING` against the partial unique index on (user_id, award_key)
        // (change: add-play-rewards): the SECOND confirmation of the same piece +
        // day writes no row. The lock above only re-checks the balance; without
        // this key two racing confirmations would each pass the check and charge
        // twice for one slot. `piece_id` names the piece for the activity feed.
        sqlx::query(
            "INSERT INTO music.curation_points \
             (user_id, award_kind, amount, catalog_score_id, reward_key, award_key, piece_id) \
             VALUES ($1, 'redeem', $2, $3, $4, $5, $6) \
             ON CONFLICT (user_id, award_key) WHERE award_key IS NOT NULL DO NOTHING",
        )
        .bind(uid)
        .bind(-(cost as i32))
        .bind(cid)
        .bind(DAY_SLOT_REWARD_KEY)
        .bind(award_key)
        .bind(catalog_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
        sqlx::query(
            "INSERT INTO music.catalog_day_access (user_id, catalog_id, day, paid) \
             VALUES ($1, $2, $3, TRUE) \
             ON CONFLICT (user_id, catalog_id, day) DO UPDATE SET paid = TRUE",
        )
        .bind(uid)
        .bind(cid)
        .bind(day)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
        tx.commit().await.map_err(internal)?;
        Ok(true)
    }

    async fn prune_before(&self, before: NaiveDate) -> Result<u64> {
        let done = sqlx::query("DELETE FROM music.catalog_day_access WHERE day < $1")
            .bind(before)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(done.rows_affected())
    }
}
