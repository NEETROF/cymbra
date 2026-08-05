//! Daily rollup + retention purge over the analytics store (change:
//! add-feature-usage-analytics, tasks 5.1 + 5.2, design D3/D4). Thin SQL I/O run
//! by the `cymbra-worker` scheduled jobs as `admin_svc` (the only worker actor
//! allowed to write across schemas); exercised by the `#[ignore]` DB integration
//! tests in `tests/`.
//!
//! Ordering guarantee (design D4): the worker runs [`rollup_closed_days`] before
//! [`purge_expired`], and the retention window is many months, so purge only ever
//! deletes raw for days already folded into the permanent aggregates.

use sqlx::PgPool;

/// Fold every **closed** UTC day still present in `analytics.usage_events` into
/// both permanent aggregates via idempotent upserts (design D3):
///
/// * `usage_action_daily` — event counts grouped by the frozen grain incl.
///   `variant` (coalesced to `''`); the count is **recomputed** from raw
///   (`DO UPDATE SET event_count = EXCLUDED.event_count`), so a re-run for a day
///   whose raw is intact reproduces identical rows (idempotent, never doubles).
/// * `usage_user_daily` — per-user daily presence (`DO NOTHING`).
///
/// A "closed day" is any `occurred_at` strictly before the start of the current
/// UTC day. `subject_id` is high-cardinality and deliberately NOT rolled up
/// (raw-only, design D8). Days whose raw has already been purged are simply absent
/// from the scan, so their aggregate rows are left untouched.
pub async fn rollup_closed_days(admin_pool: &PgPool) -> anyhow::Result<()> {
    // Action volume aggregate.
    sqlx::query(
        "INSERT INTO analytics.usage_action_daily \
           (day, action, variant, platform, device_class, app_version, locale, event_count) \
         SELECT (e.occurred_at AT TIME ZONE 'UTC')::date, e.action, COALESCE(e.variant, ''), \
                e.platform, e.device_class, e.app_version, e.locale, count(*) \
         FROM analytics.usage_events e \
         WHERE (e.occurred_at AT TIME ZONE 'UTC') < date_trunc('day', now() AT TIME ZONE 'UTC') \
         GROUP BY 1, e.action, COALESCE(e.variant, ''), e.platform, e.device_class, \
                  e.app_version, e.locale \
         ON CONFLICT (day, action, variant, platform, device_class, app_version, locale) \
         DO UPDATE SET event_count = EXCLUDED.event_count",
    )
    .execute(admin_pool)
    .await?;

    // Per-user daily presence (exact distinct-user counts over any window).
    sqlx::query(
        "INSERT INTO analytics.usage_user_daily (day, user_bucket, platform, device_class) \
         SELECT DISTINCT (e.occurred_at AT TIME ZONE 'UTC')::date, e.user_bucket, e.platform, \
                e.device_class \
         FROM analytics.usage_events e \
         WHERE (e.occurred_at AT TIME ZONE 'UTC') < date_trunc('day', now() AT TIME ZONE 'UTC') \
         ON CONFLICT (day, user_bucket, platform, device_class) DO NOTHING",
    )
    .execute(admin_pool)
    .await?;

    Ok(())
}

/// Delete raw events older than the retention window (design D4). Returns the
/// number of rows purged. The permanent aggregates are untouched — the rollup ran
/// first and this only removes already-aggregated raw. Idempotent: a re-run simply
/// finds nothing older than the window.
pub async fn purge_expired(admin_pool: &PgPool, retention_days: i64) -> anyhow::Result<u64> {
    let res = sqlx::query(
        "DELETE FROM analytics.usage_events \
         WHERE occurred_at < now() - make_interval(days => $1)",
    )
    .bind(retention_days as i32)
    .execute(admin_pool)
    .await?;
    Ok(res.rows_affected())
}
