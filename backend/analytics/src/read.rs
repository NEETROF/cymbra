//! The back-office reporting reads over the permanent aggregates (change:
//! add-feature-usage-analytics, task 7.2, design D6). Behind a trait so the
//! reporting handlers are unit-testable with a `mockall`-generated mock. Queries
//! hit only `analytics.usage_action_daily` / `analytics.usage_user_daily`, so they
//! answer any window (aggregates are permanent) and never read raw.

use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::{PgPool, QueryBuilder, Row};

/// A composable query over the aggregates. All filters are ANDed; `None` means no
/// filter on that dimension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageQuery {
    pub from_day: NaiveDate,
    pub to_day: NaiveDate,
    pub platform: Option<String>,
    pub device_class: Option<String>,
    pub action: Option<String>,
}

/// Distinct users on one platform over the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformUsers {
    pub platform: String,
    pub users: i64,
}

/// Distinct users on one device class over the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceClassUsers {
    pub device_class: String,
    pub users: i64,
}

/// Exact distinct users over the window, split by platform and device class.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UsersSummary {
    pub total_users: i64,
    pub by_platform: Vec<PlatformUsers>,
    pub by_device_class: Vec<DeviceClassUsers>,
}

/// One (action, variant) volume row over the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionCount {
    pub action: String,
    pub variant: String,
    pub events: i64,
}

/// Reporting reads for the back-office "Usage" console.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UsageReadRepo: Send + Sync {
    /// Exact distinct-user counts over the window (from `usage_user_daily`), split
    /// by platform and device class. The `action` filter does not apply here (the
    /// per-user presence table has no action dimension).
    async fn users_summary(&self, q: &UsageQuery) -> anyhow::Result<UsersSummary>;
    /// Action-volume breakdown over the window (from `usage_action_daily`) under the
    /// applied filters, grouped by (action, variant), busiest first.
    async fn action_breakdown(&self, q: &UsageQuery) -> anyhow::Result<Vec<ActionCount>>;
    /// The distinct actions present in the aggregates — the data-driven filter list.
    async fn list_actions(&self) -> anyhow::Result<Vec<String>>;
}

/// Postgres-backed [`UsageReadRepo`] (role `analytics_svc`).
pub struct PgUsageReadRepo {
    pool: PgPool,
}

impl PgUsageReadRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Push `day BETWEEN $from AND $to` plus the optional platform/device filters onto
/// a builder that already opened its `WHERE`.
fn push_common_filters(
    qb: &mut QueryBuilder<'_, sqlx::Postgres>,
    q: &UsageQuery,
    with_action: bool,
) {
    qb.push(" day >= ").push_bind(q.from_day);
    qb.push(" AND day <= ").push_bind(q.to_day);
    if let Some(p) = &q.platform {
        qb.push(" AND platform = ").push_bind(p.clone());
    }
    if let Some(d) = &q.device_class {
        qb.push(" AND device_class = ").push_bind(d.clone());
    }
    if with_action && let Some(a) = &q.action {
        qb.push(" AND action = ").push_bind(a.clone());
    }
}

#[async_trait]
impl UsageReadRepo for PgUsageReadRepo {
    async fn users_summary(&self, q: &UsageQuery) -> anyhow::Result<UsersSummary> {
        // Total distinct.
        let mut qb = QueryBuilder::new(
            "SELECT count(DISTINCT user_bucket) FROM analytics.usage_user_daily WHERE",
        );
        push_common_filters(&mut qb, q, false);
        let total_users: i64 = qb.build_query_scalar().fetch_one(&self.pool).await?;

        // By platform.
        let mut qb = QueryBuilder::new(
            "SELECT platform, count(DISTINCT user_bucket) AS users \
             FROM analytics.usage_user_daily WHERE",
        );
        push_common_filters(&mut qb, q, false);
        qb.push(" GROUP BY platform ORDER BY platform");
        let by_platform = qb
            .build()
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| PlatformUsers {
                platform: r.get("platform"),
                users: r.get("users"),
            })
            .collect();

        // By device class.
        let mut qb = QueryBuilder::new(
            "SELECT device_class, count(DISTINCT user_bucket) AS users \
             FROM analytics.usage_user_daily WHERE",
        );
        push_common_filters(&mut qb, q, false);
        qb.push(" GROUP BY device_class ORDER BY device_class");
        let by_device_class = qb
            .build()
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| DeviceClassUsers {
                device_class: r.get("device_class"),
                users: r.get("users"),
            })
            .collect();

        Ok(UsersSummary {
            total_users,
            by_platform,
            by_device_class,
        })
    }

    async fn action_breakdown(&self, q: &UsageQuery) -> anyhow::Result<Vec<ActionCount>> {
        let mut qb = QueryBuilder::new(
            "SELECT action, variant, sum(event_count)::bigint AS events \
             FROM analytics.usage_action_daily WHERE",
        );
        push_common_filters(&mut qb, q, true);
        qb.push(" GROUP BY action, variant ORDER BY events DESC, action, variant");
        let rows = qb
            .build()
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| ActionCount {
                action: r.get("action"),
                variant: r.get("variant"),
                events: r.get("events"),
            })
            .collect();
        Ok(rows)
    }

    async fn list_actions(&self) -> anyhow::Result<Vec<String>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT action FROM analytics.usage_action_daily ORDER BY action",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
