// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Postgres adapter for [`LinguaAdminRepo`]. Thin sqlx glue — coverage-excluded; the
//! window parse + series formatting are host-tested in `admin_core`. Every query is an
//! aggregate over `lingua.daily_stats` grouped by day and studied language — a
//! `COUNT(DISTINCT user_id)` / `SUM(...)` — and returns no account identifier, honouring
//! the privacy allow-list. "Active accounts" = accounts that synced a daily stat in the
//! window (the sync-only bias the console labels on screen).

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::admin::LinguaAdminRepo;
use crate::admin_core::{LanguageUsage, SeriesMetric, Usage};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("lingua db: {e}"))
}

pub struct PgLinguaAdminRepo {
    pool: PgPool,
}

impl PgLinguaAdminRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LinguaAdminRepo for PgLinguaAdminRepo {
    async fn usage(&self, from_day: i32, to_day: i32) -> Result<Usage> {
        // Tiles: distinct synced accounts + summed learning counts across the window.
        let tiles = sqlx::query(
            "SELECT COUNT(DISTINCT user_id) AS active, \
                    COALESCE(SUM(words_learned), 0)::bigint AS words, \
                    COALESCE(SUM(reviews_done), 0)::bigint AS reviews \
             FROM lingua.daily_stats WHERE day >= $1 AND day <= $2",
        )
        .bind(from_day)
        .bind(to_day)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        // Breakdown by studied language (counts only, ordered for a stable display).
        let rows = sqlx::query(
            "SELECT language, \
                    COUNT(DISTINCT user_id) AS active, \
                    COALESCE(SUM(words_learned), 0)::bigint AS words, \
                    COALESCE(SUM(reviews_done), 0)::bigint AS reviews \
             FROM lingua.daily_stats WHERE day >= $1 AND day <= $2 \
             GROUP BY language ORDER BY language",
        )
        .bind(from_day)
        .bind(to_day)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;

        Ok(Usage {
            active_accounts: tiles.get::<i64, _>("active"),
            words_learned: tiles.get::<i64, _>("words"),
            reviews: tiles.get::<i64, _>("reviews"),
            by_language: rows
                .iter()
                .map(|r| LanguageUsage {
                    language: r.get("language"),
                    active_accounts: r.get::<i64, _>("active"),
                    words_learned: r.get::<i64, _>("words"),
                    reviews: r.get::<i64, _>("reviews"),
                })
                .collect(),
        })
    }

    async fn series(
        &self,
        from_day: i32,
        to_day: i32,
        metric: SeriesMetric,
        language: Option<&str>,
    ) -> Result<Vec<(i32, i64)>> {
        // The aggregate column is chosen from the enum, never from client input.
        let agg = match metric {
            SeriesMetric::WordsLearned => "COALESCE(SUM(words_learned), 0)::bigint",
            SeriesMetric::Reviews => "COALESCE(SUM(reviews_done), 0)::bigint",
            SeriesMetric::Exposures => "COALESCE(SUM(exposures), 0)::bigint",
        };
        let sql = format!(
            "SELECT day, {agg} AS value FROM lingua.daily_stats \
             WHERE day >= $1 AND day <= $2 AND ($3::text IS NULL OR language = $3) \
             GROUP BY day ORDER BY day"
        );
        let rows = sqlx::query(&sql)
            .bind(from_day)
            .bind(to_day)
            .bind(language)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        Ok(rows
            .iter()
            .map(|r| (r.get::<i32, _>("day"), r.get::<i64, _>("value")))
            .collect())
    }
}
