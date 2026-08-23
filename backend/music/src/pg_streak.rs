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

//! Postgres-backed [`StreakRepo`] (change: add-practice-streak) — thin I/O glue
//! (excluded from the coverage gate; the rules it serves are proven in
//! `streak_core` / `streak_module` against the in-memory fake).
//!
//! Two things are worth knowing about this adapter:
//!
//! * [`PgStreakRepo::spend_and_restore`] is ONE transaction: it locks the user's
//!   streak row, re-checks the ledger balance under that lock, then writes both
//!   the negative `curation_points` entry and the restored streak. Concurrent
//!   confirms serialise on the row lock, so a double-submit charges once and a
//!   crash mid-way leaves neither write.
//! * [`PgStreakRepo::live_streaks`] joins `user_account.users` for the account's
//!   timezone, which the `music_svc` role cannot read. It is the **worker's**
//!   reminder sweep only, and the worker builds this repo on the `admin_svc`
//!   pool (the same actor that already runs the consensus sweep and the account
//!   purge across schemas). The request-path instance never calls it.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::streak::{FREEZE_REWARD_KEY, StreakRepo};
use crate::streak_core::{ReminderCandidate, StreakState};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("streak db: {e}"))
}

fn parse_uuid(s: &str, what: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(s).map_err(|_| AppError::InvalidArgument(format!("invalid {what}")))
}

/// Postgres implementation over a pool with write access to the `music` schema.
pub struct PgStreakRepo {
    pool: PgPool,
}

impl PgStreakRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StreakRepo for PgStreakRepo {
    async fn get(&self, user_id: &str) -> Result<StreakState> {
        let uid = parse_uuid(user_id, "user id")?;
        let row = sqlx::query(
            "SELECT current_streak, longest_streak, last_played_date \
             FROM music.practice_streaks WHERE user_id = $1",
        )
        .bind(uid)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        // No row = never played. That is an empty streak, not an error.
        Ok(row.map(row_to_state).unwrap_or_default())
    }

    async fn put(&self, user_id: &str, state: &StreakState) -> Result<()> {
        let uid = parse_uuid(user_id, "user id")?;
        sqlx::query(
            "INSERT INTO music.practice_streaks \
             (user_id, current_streak, longest_streak, last_played_date, updated_at) \
             VALUES ($1, $2, $3, $4, now()) \
             ON CONFLICT (user_id) DO UPDATE SET \
               current_streak = EXCLUDED.current_streak, \
               longest_streak = GREATEST(music.practice_streaks.longest_streak, EXCLUDED.longest_streak), \
               last_played_date = EXCLUDED.last_played_date, \
               updated_at = now()",
        )
        .bind(uid)
        .bind(state.current as i32)
        .bind(state.longest as i32)
        .bind(state.last_played)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn spendable_points(&self, user_id: &str) -> Result<i64> {
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

    async fn spend_and_restore(
        &self,
        user_id: &str,
        state: &StreakState,
        cost: i64,
        award_key: &str,
    ) -> Result<bool> {
        let uid = parse_uuid(user_id, "user id")?;
        let mut tx = self.pool.begin().await.map_err(internal)?;
        // Serialise concurrent confirms on the user's own streak row: the second
        // one blocks here, then re-reads a balance that already reflects the
        // first charge (and is refused if it no longer covers the cost).
        sqlx::query("SELECT 1 FROM music.practice_streaks WHERE user_id = $1 FOR UPDATE")
            .bind(uid)
            .fetch_optional(&mut *tx)
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
        // (change: add-play-rewards): the SECOND confirmation of the same day's
        // freeze writes no row. The row lock above only re-checks the balance, so
        // without this key two racing confirmations would each pass the check and
        // charge the user twice for one restored streak. The index predicate is
        // repeated because Postgres infers a partial index only from a matching
        // one. A conflict is not an error: the streak restore below still runs,
        // which makes the whole operation idempotent per local day.
        sqlx::query(
            "INSERT INTO music.curation_points \
             (user_id, award_kind, amount, reward_key, award_key) \
             VALUES ($1, 'redeem', $2, $3, $4) \
             ON CONFLICT (user_id, award_key) WHERE award_key IS NOT NULL DO NOTHING",
        )
        .bind(uid)
        .bind(-(cost as i32))
        .bind(FREEZE_REWARD_KEY)
        .bind(award_key)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
        sqlx::query(
            "INSERT INTO music.practice_streaks \
             (user_id, current_streak, longest_streak, last_played_date, updated_at) \
             VALUES ($1, $2, $3, $4, now()) \
             ON CONFLICT (user_id) DO UPDATE SET \
               current_streak = EXCLUDED.current_streak, \
               longest_streak = GREATEST(music.practice_streaks.longest_streak, EXCLUDED.longest_streak), \
               last_played_date = EXCLUDED.last_played_date, \
               updated_at = now()",
        )
        .bind(uid)
        .bind(state.current as i32)
        .bind(state.longest as i32)
        .bind(state.last_played)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
        tx.commit().await.map_err(internal)?;
        Ok(true)
    }

    async fn live_streaks(&self) -> Result<Vec<ReminderCandidate>> {
        // LEFT JOIN: an account with no stored timezone/locale still yields a row
        // (with empty strings), and the at-risk core falls back rather than
        // dropping the user from the sweep.
        //
        // The date bound is a coarse PRE-filter, not the decision: only a run
        // whose last play was the player's own yesterday is at risk, and the core
        // settles that per timezone. Two UTC days back covers every offset (a
        // player at UTC-11 is a whole day behind the server's date), while
        // keeping the sweep off the rows of everyone who stopped playing months
        // ago — `current_streak` alone never stops being positive.
        let rows = sqlx::query(
            "SELECT s.user_id, s.current_streak, s.longest_streak, s.last_played_date, \
                    COALESCE(u.timezone, '') AS timezone, \
                    COALESCE(u.locale, '') AS locale \
             FROM music.practice_streaks s \
             LEFT JOIN user_account.users u ON u.id = s.user_id \
             WHERE s.current_streak > 0 \
               AND s.last_played_date >= CURRENT_DATE - 2 \
             ORDER BY s.user_id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| ReminderCandidate {
                user_id: r.get::<uuid::Uuid, _>("user_id").to_string(),
                timezone: r.get::<String, _>("timezone"),
                locale: r.get::<String, _>("locale"),
                state: row_to_state(r),
            })
            .collect())
    }
}

/// Map a `practice_streaks` row (or the streak columns of a join) to the state.
fn row_to_state(r: sqlx::postgres::PgRow) -> StreakState {
    StreakState {
        current: r.get::<i32, _>("current_streak") as i64,
        longest: r.get::<i32, _>("longest_streak") as i64,
        last_played: r.get::<Option<chrono::NaiveDate>, _>("last_played_date"),
    }
}
