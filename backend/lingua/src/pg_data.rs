// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Postgres adapter for [`DataRepo`] and [`ErasureMarks`]. Thin sqlx glue —
//! coverage-excluded; the drop rules are host-tested in `data_core` and the modules.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::data::{DataRepo, ErasureMarks};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("lingua db: {e}"))
}

fn uid(user: &str) -> Result<Uuid> {
    Uuid::parse_str(user).map_err(|_| AppError::InvalidArgument("invalid user id".into()))
}

/// The user's Lingua tables. The same list as backend/worker's `purge_user` block (which
/// also drops `lingua.data_erasures`); names MUST match the migrations.
const USER_TABLES: [&str; 4] = [
    "lingua.word_statuses",
    "lingua.declared_levels",
    "lingua.cards",
    "lingua.daily_stats",
];

pub struct PgDataRepo {
    pool: PgPool,
}

impl PgDataRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

async fn read_mark(pool: &PgPool, user: &str) -> Result<i64> {
    let row = sqlx::query("SELECT erased_at FROM lingua.data_erasures WHERE user_id = $1")
        .bind(uid(user)?)
        .fetch_optional(pool)
        .await
        .map_err(internal)?;
    Ok(row.map(|r| r.get::<i64, _>("erased_at")).unwrap_or(0))
}

#[async_trait]
impl DataRepo for PgDataRepo {
    async fn erase(&self, user: &str, now: i64) -> Result<i64> {
        let user = uid(user)?;
        let mut tx = self.pool.begin().await.map_err(internal)?;
        for table in USER_TABLES {
            sqlx::query(&format!("DELETE FROM {table} WHERE user_id = $1"))
                .bind(user)
                .execute(&mut *tx)
                .await
                .map_err(internal)?;
        }
        let row = sqlx::query(
            "INSERT INTO lingua.data_erasures (user_id, erased_at) VALUES ($1, $2) \
             ON CONFLICT (user_id) DO UPDATE SET \
               erased_at = GREATEST(lingua.data_erasures.erased_at, excluded.erased_at) \
             RETURNING erased_at",
        )
        .bind(user)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;
        tx.commit().await.map_err(internal)?;
        Ok(row.get::<i64, _>("erased_at"))
    }

    async fn erased_at(&self, user: &str) -> Result<i64> {
        read_mark(&self.pool, user).await
    }
}

#[async_trait]
impl ErasureMarks for PgDataRepo {
    async fn erased_at(&self, user: &str) -> Result<i64> {
        read_mark(&self.pool, user).await
    }
}
