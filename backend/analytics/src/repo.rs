//! The raw-event store behind a trait (change: add-feature-usage-analytics, task
//! 4.3). Behind a trait so the ingestion handler is unit-testable with a
//! `mockall`-generated `MockUsageEventRepo` (rust-testing default) — no DB needed.

use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder};

use crate::usage_core::ValidEvent;

/// One validated row ready to persist: the pseudonymous `user_bucket` (design D2)
/// plus the validated event dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageRow {
    pub user_bucket: String,
    pub event: ValidEvent,
}

/// Batch insert of raw usage events into `analytics.usage_events`.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UsageEventRepo: Send + Sync {
    /// Insert a batch of validated rows; returns the number of rows written.
    async fn insert_batch(&self, rows: &[UsageRow]) -> anyhow::Result<u64>;
}

/// Postgres-backed [`UsageEventRepo`] over `analytics.usage_events` (role
/// `analytics_svc`).
pub struct PgUsageEventRepo {
    pool: PgPool,
}

impl PgUsageEventRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UsageEventRepo for PgUsageEventRepo {
    async fn insert_batch(&self, rows: &[UsageRow]) -> anyhow::Result<u64> {
        if rows.is_empty() {
            return Ok(0);
        }
        // `id` is a server-generated UUID v7; `received_at` defaults to now().
        let mut qb = QueryBuilder::new(
            "INSERT INTO analytics.usage_events \
             (id, user_bucket, action, variant, subject_id, platform, device_class, \
              app_version, locale, occurred_at) ",
        );
        qb.push_values(rows, |mut b, row| {
            b.push_bind(uuid::Uuid::now_v7())
                .push_bind(&row.user_bucket)
                .push_bind(&row.event.action)
                .push_bind(&row.event.variant)
                .push_bind(&row.event.subject_id)
                .push_bind(&row.event.platform)
                .push_bind(&row.event.device_class)
                .push_bind(&row.event.app_version)
                .push_bind(&row.event.locale)
                .push_bind(row.event.occurred_at);
        });
        let res = qb.build().execute(&self.pool).await?;
        Ok(res.rows_affected())
    }
}
