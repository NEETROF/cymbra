// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Postgres adapter for [`UsageCountsRepo`] (change: add-lingua-usage-counts): upserts that
//! stop at the ceiling, and the retention delete. Thin sqlx glue — coverage-excluded; what
//! reaches it is decided and tested in `usage_counts`, and it is run against Postgres by
//! `tests/pg_usage_counts_it.rs`. Runs on the `lingua_svc` pool (`search_path = lingua`),
//! every name schema-qualified.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::PgPool;

use crate::usage_counts::UsageCountsRepo;

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("lingua db: {e}"))
}

pub struct PgUsageCountsRepo {
    pool: PgPool,
}

impl PgUsageCountsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UsageCountsRepo for PgUsageCountsRepo {
    async fn count_steps(
        &self,
        week: i32,
        browser: &str,
        version: &str,
        steps: &[&'static str],
        ceiling: i64,
    ) -> Result<()> {
        let steps: Vec<String> = steps.iter().map(|s| s.to_string()).collect();
        sqlx::query(
            "INSERT INTO lingua.usage_counts_weekly (install_week, step, browser, version, count) \
             SELECT $1, s, $3, $4, 1 FROM unnest($2::text[]) AS s WHERE $5 > 0 \
             ON CONFLICT (install_week, step, browser, version) DO UPDATE \
               SET count = lingua.usage_counts_weekly.count + 1 \
               WHERE lingua.usage_counts_weekly.count < $5",
        )
        .bind(week)
        .bind(&steps)
        .bind(browser)
        .bind(version)
        .bind(ceiling)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn count_leave(
        &self,
        week: i32,
        reason: &str,
        browser: &str,
        version: &str,
        ceiling: i64,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO lingua.leave_reasons_weekly (week, reason, browser, version, count) \
             SELECT $1, $2, $3, $4, 1 WHERE $5 > 0 \
             ON CONFLICT (week, reason, browser, version) DO UPDATE \
               SET count = lingua.leave_reasons_weekly.count + 1 \
               WHERE lingua.leave_reasons_weekly.count < $5",
        )
        .bind(week)
        .bind(reason)
        .bind(browser)
        .bind(version)
        .bind(ceiling)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn delete_before(&self, week: i32) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(internal)?;
        sqlx::query("DELETE FROM lingua.usage_counts_weekly WHERE install_week < $1")
            .bind(week)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
        sqlx::query("DELETE FROM lingua.leave_reasons_weekly WHERE week < $1")
            .bind(week)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
        tx.commit().await.map_err(internal)?;
        Ok(())
    }
}
