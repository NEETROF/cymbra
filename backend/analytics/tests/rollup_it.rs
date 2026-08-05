//! Analytics rollup + purge integration tests (change: add-feature-usage-analytics,
//! tasks 4.3 + 5.4). Drives the real `PgUsageEventRepo` insert and the
//! `rollup_closed_days` / `purge_expired` SQL against live Postgres.
//!
//! Requires the dev infra with the `analytics` role/schema bootstrapped
//! (`db/init/00-roles.sh`) reachable via `CYMBRA_ANALYTICS_DATABASE_URL`.
//! Run: `cargo test -p cymbra-analytics --test rollup_it -- --ignored`

use chrono::{Duration, Utc};
use cymbra_analytics::{PgUsageEventRepo, UsageEventRepo, UsageRow, ValidEvent};
use sqlx::{PgPool, Row};

async fn pool() -> PgPool {
    let url = std::env::var("CYMBRA_ANALYTICS_DATABASE_URL")
        .expect("CYMBRA_ANALYTICS_DATABASE_URL must be set");
    let p = cymbra_analytics::connect(&url, 2)
        .await
        .expect("connect analytics");
    cymbra_analytics::MIGRATOR.run(&p).await.expect("migrate");
    p
}

fn event(action: &str, bucket: &str, days_ago: i64) -> UsageRow {
    UsageRow {
        user_bucket: bucket.to_string(),
        event: ValidEvent {
            action: action.to_string(),
            variant: None,
            subject_id: None,
            platform: "ios".into(),
            device_class: "phone".into(),
            app_version: "1.0.0".into(),
            locale: "fr".into(),
            occurred_at: Utc::now() - Duration::days(days_ago),
        },
    }
}

#[tokio::test]
#[ignore = "requires live Postgres with the analytics schema"]
async fn rollup_is_idempotent_and_purge_keeps_aggregates() {
    let p = pool().await;
    // Unique, shape-valid action tag so assertions isolate this run's rows.
    let tag = format!("test_{}", uuid::Uuid::now_v7().simple());
    let a = format!("bkt_{}_a", uuid::Uuid::now_v7().simple());
    let b = format!("bkt_{}_b", uuid::Uuid::now_v7().simple());

    let repo = PgUsageEventRepo::new(p.clone());
    // Day D-3: A×2 + B×1; Day D-2: A×1. A is present on two days (distinct = 2).
    repo.insert_batch(&[
        event(&tag, &a, 3),
        event(&tag, &a, 3),
        event(&tag, &b, 3),
        event(&tag, &a, 2),
    ])
    .await
    .unwrap();

    // Rollup twice — must be identical (idempotent recompute).
    cymbra_analytics::rollup_closed_days(&p).await.unwrap();
    cymbra_analytics::rollup_closed_days(&p).await.unwrap();

    // Action counts for this tag: 3 on D-3, 1 on D-2 (summed = 4), unchanged by the
    // second run.
    let total_events: i64 =
        sqlx::query("SELECT COALESCE(sum(event_count), 0)::bigint FROM analytics.usage_action_daily WHERE action = $1")
            .bind(&tag)
            .fetch_one(&p)
            .await
            .unwrap()
            .get(0);
    assert_eq!(total_events, 4, "action rollup must recompute, not double");

    let action_days: i64 =
        sqlx::query("SELECT count(*) FROM analytics.usage_action_daily WHERE action = $1")
            .bind(&tag)
            .fetch_one(&p)
            .await
            .unwrap()
            .get(0);
    assert_eq!(action_days, 2, "two closed days aggregated");

    // Distinct users over the window from usage_user_daily: A counted once → 2.
    let distinct_users: i64 = sqlx::query(
        "SELECT count(DISTINCT user_bucket) FROM analytics.usage_user_daily \
         WHERE user_bucket IN ($1, $2)",
    )
    .bind(&a)
    .bind(&b)
    .fetch_one(&p)
    .await
    .unwrap()
    .get(0);
    assert_eq!(distinct_users, 2, "A present on two days counts once");

    // The reporting reads are a HYBRID (design D6): closed days from the aggregates
    // + the current day live from raw. Insert two raw TODAY events (NOT rolled up)
    // and assert they are counted on top of the 4 aggregated closed-day events —
    // proving the speed layer works with zero rollup lag.
    use cymbra_analytics::{PgUsageReadRepo, UsageQuery, UsageReadRepo};
    repo.insert_batch(&[event(&tag, &a, 0), event(&tag, &b, 0)])
        .await
        .unwrap();

    let read = PgUsageReadRepo::new(p.clone());
    let q = UsageQuery {
        from_day: (Utc::now() - Duration::days(4)).date_naive(),
        to_day: Utc::now().date_naive(),
        platform: None,
        device_class: None,
        action: Some(tag.clone()),
    };
    let breakdown = read.action_breakdown(&q).await.unwrap();
    let my_events: i64 = breakdown
        .iter()
        .filter(|r| r.action == tag)
        .map(|r| r.events)
        .sum();
    assert_eq!(
        my_events, 6,
        "hybrid: 4 aggregated closed-day events + 2 live raw today events (no rollup)"
    );
    assert!(
        read.list_actions().await.unwrap().contains(&tag),
        "list_actions is data-driven and includes today's raw actions"
    );

    // The per-day series (line charts) is the same hybrid, split by day: its points
    // sum to the same total (4 aggregated + 2 live raw today).
    let series = read
        .usage_series(&q, cymbra_analytics::SeriesDim::Action)
        .await
        .unwrap();
    let series_total: i64 = series
        .iter()
        .filter(|pt| pt.series == tag)
        .map(|pt| pt.value)
        .sum();
    assert_eq!(
        series_total, 6,
        "usage_series per-day points sum to the hybrid total"
    );

    // Clean up the today events so the purge assertions below stay about closed days.
    sqlx::query(
        "DELETE FROM analytics.usage_events WHERE action = $1 \
         AND (occurred_at AT TIME ZONE 'UTC') >= date_trunc('day', now() AT TIME ZONE 'UTC')",
    )
    .bind(&tag)
    .execute(&p)
    .await
    .unwrap();

    // Purge everything older than 1 day (all seeds are ≥2 days old).
    let purged = cymbra_analytics::purge_expired(&p, 1).await.unwrap();
    assert!(purged >= 4, "the seeded raw rows are purged");

    // Raw gone…
    let raw_left: i64 =
        sqlx::query("SELECT count(*) FROM analytics.usage_events WHERE action = $1")
            .bind(&tag)
            .fetch_one(&p)
            .await
            .unwrap()
            .get(0);
    assert_eq!(raw_left, 0);

    // …but the aggregates survive the purge.
    let agg_left: i64 =
        sqlx::query("SELECT count(*) FROM analytics.usage_action_daily WHERE action = $1")
            .bind(&tag)
            .fetch_one(&p)
            .await
            .unwrap()
            .get(0);
    assert_eq!(agg_left, 2, "aggregates are permanent, unaffected by purge");

    // Cleanup this run's aggregate rows (raw already purged).
    sqlx::query("DELETE FROM analytics.usage_action_daily WHERE action = $1")
        .bind(&tag)
        .execute(&p)
        .await
        .unwrap();
    sqlx::query("DELETE FROM analytics.usage_user_daily WHERE user_bucket IN ($1, $2)")
        .bind(&a)
        .bind(&b)
        .execute(&p)
        .await
        .unwrap();
}
