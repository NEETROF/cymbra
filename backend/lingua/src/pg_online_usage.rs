// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Postgres adapter for [`UsageRepo`]: one upsert into the day's totals. Thin sqlx glue —
//! coverage-excluded; what goes into an event is tested in `online_translation`.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::PgPool;

use crate::online_translation::{UsageEvent, UsageRepo};

pub struct PgOnlineUsageRepo {
    pool: PgPool,
}

impl PgOnlineUsageRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UsageRepo for PgOnlineUsageRepo {
    async fn record(&self, e: &UsageEvent) -> Result<()> {
        sqlx::query(
            "INSERT INTO lingua.online_translation_daily \
               (day, provider, pair, outcome, requests, input_tokens, output_tokens, cost_micro_usd) \
             VALUES ($1, $2, $3, $4, 1, $5, $6, $7) \
             ON CONFLICT (day, provider, pair, outcome) DO UPDATE SET \
               requests = lingua.online_translation_daily.requests + 1, \
               input_tokens = lingua.online_translation_daily.input_tokens + EXCLUDED.input_tokens, \
               output_tokens = lingua.online_translation_daily.output_tokens + EXCLUDED.output_tokens, \
               cost_micro_usd = lingua.online_translation_daily.cost_micro_usd + EXCLUDED.cost_micro_usd",
        )
        .bind(e.day)
        .bind(&e.provider)
        .bind(&e.pair)
        .bind(e.outcome.as_str())
        .bind(e.input_tokens)
        .bind(e.output_tokens)
        .bind(e.cost_micro_usd)
        .execute(&self.pool)
        .await
        .map_err(|err| AppError::Internal(anyhow::anyhow!("lingua db: {err}")))?;
        Ok(())
    }
}
