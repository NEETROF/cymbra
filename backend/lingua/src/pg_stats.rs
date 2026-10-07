// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Postgres adapter for [`StatsRepo`]. Thin sqlx glue — coverage-excluded; the
//! SUM-across-devices consolidation is host-tested in `stats_core`. Upserts replace by
//! (user, day, language, device), never add; the device's native language is a value of
//! the row the last upsert sets (add-lingua-native-language-server), `DEFAULT 'fr'` for
//! every row written before migration 0006.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::stats::{DailyStat, StatsRepo};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("lingua db: {e}"))
}

fn uid(user: &str) -> Result<Uuid> {
    Uuid::parse_str(user).map_err(|_| AppError::InvalidArgument("invalid user id".into()))
}

pub struct PgStatsRepo {
    pool: PgPool,
}

impl PgStatsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StatsRepo for PgStatsRepo {
    async fn upsert(&self, user: &str, stat: &DailyStat) -> Result<()> {
        sqlx::query(
            "INSERT INTO lingua.daily_stats \
               (user_id, day, language, device_id, exposures, words_learned, reviews_done, \
                unknown_seen, native_language) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (user_id, day, language, device_id) DO UPDATE SET \
               exposures = excluded.exposures, words_learned = excluded.words_learned, \
               reviews_done = excluded.reviews_done, unknown_seen = excluded.unknown_seen, \
               native_language = excluded.native_language",
        )
        .bind(uid(user)?)
        .bind(stat.day)
        .bind(&stat.language)
        .bind(&stat.device_id)
        .bind(stat.exposures as i32)
        .bind(stat.words_learned as i32)
        .bind(stat.reviews_done as i32)
        .bind(stat.unknown_seen.unwrap_or(0) as i32)
        .bind(&stat.native_language)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn range(
        &self,
        user: &str,
        from_day: i32,
        to_day: i32,
        language: Option<&str>,
    ) -> Result<Vec<DailyStat>> {
        let rows = sqlx::query(
            "SELECT day, language, device_id, exposures, words_learned, reviews_done, \
                    unknown_seen, native_language \
             FROM lingua.daily_stats \
             WHERE user_id = $1 AND day >= $2 AND day <= $3 \
               AND ($4::text IS NULL OR language = $4)",
        )
        .bind(uid(user)?)
        .bind(from_day)
        .bind(to_day)
        .bind(language)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .iter()
            .map(|r| DailyStat {
                day: r.get("day"),
                language: r.get("language"),
                device_id: r.get("device_id"),
                exposures: r.get::<i32, _>("exposures") as u32,
                words_learned: r.get::<i32, _>("words_learned") as u32,
                reviews_done: r.get::<i32, _>("reviews_done") as u32,
                unknown_seen: Some(r.get::<i32, _>("unknown_seen") as u32),
                native_language: r.get("native_language"),
            })
            .collect())
    }
}
