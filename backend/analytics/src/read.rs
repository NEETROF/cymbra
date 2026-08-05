//! The back-office reporting reads (change: add-feature-usage-analytics, task 7.2,
//! design D6). Behind a trait so the reporting handlers are unit-testable with a
//! `mockall`-generated mock.
//!
//! **Hybrid batch + speed layer (design D6).** Each read merges two disjoint
//! ranges so recent activity is live with zero rollup lag:
//! - **closed UTC days** (`day < today`) come from the permanent aggregates
//!   (`usage_action_daily` / `usage_user_daily`) — cheap, exact, available beyond
//!   the raw retention window;
//! - the **current UTC day** comes straight from raw `usage_events` — the moment an
//!   event is ingested it shows, no daily rollup needed.
//!
//! The split boundary is the start of the current UTC day, so the two ranges never
//! overlap and figures never double-count. (Distinct-user counts are computed over
//! the *union* of presence rows — you can never sum daily distinct counts.)

use std::collections::{BTreeMap, HashSet};

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

/// Which dimension a time series is split by (one curve per key). The dimension
/// also decides the metric: distinct users per day for platform/device, event
/// count per day for action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesDim {
    Platform,
    DeviceClass,
    Action,
}

/// One point of a per-day time series.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesPoint {
    pub day: String, // ISO yyyy-mm-dd (UTC)
    pub series: String,
    pub value: i64,
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
    /// A per-day time series split into one curve per key of `dim` (hybrid: closed
    /// days from the aggregates + the current day live from raw).
    async fn usage_series(
        &self,
        q: &UsageQuery,
        dim: SeriesDim,
    ) -> anyhow::Result<Vec<SeriesPoint>>;
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

/// Aggregate-side window (the batch layer): the day range, capped **strictly
/// before today** (UTC) so it never overlaps the raw current-day range. Pushed onto
/// a builder that already opened its `WHERE`.
fn push_agg_window(qb: &mut QueryBuilder<'_, sqlx::Postgres>, q: &UsageQuery, with_action: bool) {
    qb.push(" day >= ").push_bind(q.from_day);
    qb.push(" AND day <= ").push_bind(q.to_day);
    qb.push(" AND day < (now() AT TIME ZONE 'UTC')::date");
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

/// Raw-side window (the speed layer): only the **current** UTC day (and later),
/// still constrained to the requested range. Empty when the range ends before today.
fn push_raw_window(qb: &mut QueryBuilder<'_, sqlx::Postgres>, q: &UsageQuery, with_action: bool) {
    qb.push(" (occurred_at AT TIME ZONE 'UTC') >= date_trunc('day', now() AT TIME ZONE 'UTC')");
    qb.push(" AND (occurred_at AT TIME ZONE 'UTC')::date >= ")
        .push_bind(q.from_day);
    qb.push(" AND (occurred_at AT TIME ZONE 'UTC')::date <= ")
        .push_bind(q.to_day);
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
        // Presence set = closed days (aggregate) ∪ current day (raw, live). The
        // action filter does not apply here (per-user presence has no action
        // dimension). Distinct counts are computed over the DEDUPED union — daily
        // distinct counts can never be summed.
        let mut qb = QueryBuilder::new(
            "SELECT user_bucket, platform, device_class \
             FROM analytics.usage_user_daily WHERE",
        );
        push_agg_window(&mut qb, q, false);
        qb.push(
            " UNION SELECT DISTINCT user_bucket, platform, device_class \
             FROM analytics.usage_events WHERE",
        );
        push_raw_window(&mut qb, q, false);
        let rows = qb.build().fetch_all(&self.pool).await?;

        let mut all: HashSet<String> = HashSet::new();
        let mut by_p: BTreeMap<String, HashSet<String>> = BTreeMap::new();
        let mut by_d: BTreeMap<String, HashSet<String>> = BTreeMap::new();
        for r in &rows {
            let bucket: String = r.get("user_bucket");
            all.insert(bucket.clone());
            by_p.entry(r.get("platform"))
                .or_default()
                .insert(bucket.clone());
            by_d.entry(r.get("device_class"))
                .or_default()
                .insert(bucket);
        }
        Ok(UsersSummary {
            total_users: all.len() as i64,
            by_platform: by_p
                .into_iter()
                .map(|(platform, s)| PlatformUsers {
                    platform,
                    users: s.len() as i64,
                })
                .collect(),
            by_device_class: by_d
                .into_iter()
                .map(|(device_class, s)| DeviceClassUsers {
                    device_class,
                    users: s.len() as i64,
                })
                .collect(),
        })
    }

    async fn action_breakdown(&self, q: &UsageQuery) -> anyhow::Result<Vec<ActionCount>> {
        // Closed days' pre-aggregated counts UNION ALL the current day counted live
        // from raw; the two ranges are disjoint, so summing never double-counts.
        let mut qb = QueryBuilder::new("SELECT action, variant, sum(cnt)::bigint AS events FROM (");
        qb.push(
            "SELECT action, variant, event_count AS cnt \
             FROM analytics.usage_action_daily WHERE",
        );
        push_agg_window(&mut qb, q, true);
        qb.push(
            " UNION ALL SELECT action, COALESCE(variant, '') AS variant, count(*) AS cnt \
             FROM analytics.usage_events WHERE",
        );
        push_raw_window(&mut qb, q, true);
        qb.push(" GROUP BY action, COALESCE(variant, '')");
        qb.push(") s GROUP BY action, variant ORDER BY events DESC, action, variant");
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
        // Union the aggregates with the current day's raw actions so a brand-new
        // action emitted today appears in the filter before the next rollup.
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT action FROM ( \
               SELECT action FROM analytics.usage_action_daily \
               UNION \
               SELECT action FROM analytics.usage_events \
               WHERE (occurred_at AT TIME ZONE 'UTC') >= date_trunc('day', now() AT TIME ZONE 'UTC') \
             ) s ORDER BY action",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn usage_series(
        &self,
        q: &UsageQuery,
        dim: SeriesDim,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        // The split column is chosen by the `dim` enum (never user input), so
        // interpolating it into the SQL is injection-safe.
        let rows = match dim {
            SeriesDim::Platform | SeriesDim::DeviceClass => {
                let col = if matches!(dim, SeriesDim::DeviceClass) {
                    "device_class"
                } else {
                    "platform"
                };
                // Distinct users PER DAY per key: closed days (aggregate) UNION ALL
                // the current day (raw). Disjoint days ⇒ no key appears twice.
                let mut qb = QueryBuilder::new(format!(
                    "SELECT to_char(day, 'YYYY-MM-DD') AS d, {col} AS series, \
                     count(DISTINCT user_bucket) AS value \
                     FROM analytics.usage_user_daily WHERE"
                ));
                push_agg_window(&mut qb, q, false);
                qb.push(format!(
                    " GROUP BY day, {col} \
                     UNION ALL SELECT to_char((occurred_at AT TIME ZONE 'UTC')::date, 'YYYY-MM-DD'), \
                     {col}, count(DISTINCT user_bucket) \
                     FROM analytics.usage_events WHERE"
                ));
                push_raw_window(&mut qb, q, false);
                qb.push(format!(" GROUP BY 1, {col}"));
                qb.build().fetch_all(&self.pool).await?
            }
            SeriesDim::Action => {
                // Events PER DAY per action: closed (summed) + current day (raw count).
                let mut qb = QueryBuilder::new(
                    "SELECT to_char(day, 'YYYY-MM-DD') AS d, action AS series, \
                     sum(event_count)::bigint AS value \
                     FROM analytics.usage_action_daily WHERE",
                );
                push_agg_window(&mut qb, q, true);
                qb.push(
                    " GROUP BY day, action \
                     UNION ALL SELECT to_char((occurred_at AT TIME ZONE 'UTC')::date, 'YYYY-MM-DD'), \
                     action, count(*) \
                     FROM analytics.usage_events WHERE",
                );
                push_raw_window(&mut qb, q, true);
                qb.push(" GROUP BY 1, action");
                qb.build().fetch_all(&self.pool).await?
            }
        };
        Ok(rows
            .into_iter()
            .map(|r| SeriesPoint {
                day: r.get("d"),
                series: r.get("series"),
                value: r.get("value"),
            })
            .collect())
    }
}
