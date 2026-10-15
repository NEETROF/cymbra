//! The anonymous usage counters against live Postgres (change: add-lingua-usage-counts,
//! task 2.4): the tables hold counters and nothing that could identify a request, the
//! upserts stop at the ceiling, the console reads them back by installation day (leaving
//! answers by week), and the retention delete removes old days and weeks.
//!
//! Run: `CYMBRA_LINGUA_DATABASE_URL=… cargo test -p cymbra-lingua --test pg_usage_counts_it -- --ignored`

use cymbra_lingua::pg_usage_counts::PgUsageCountsRepo;
use cymbra_lingua::usage_counts::UsageCountsRepo;
use cymbra_lingua::usage_counts_core::monday_of;
use cymbra_lingua::{LinguaAdminRepo, PgLinguaAdminRepo};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

async fn pool() -> PgPool {
    let url = std::env::var("CYMBRA_LINGUA_DATABASE_URL").expect("CYMBRA_LINGUA_DATABASE_URL");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect");
    cymbra_lingua::MIGRATOR.run(&pool).await.expect("migrate");
    pool
}

/// A Monday of our own, before the epoch, where no real counter lives: the retention delete
/// below then removes nothing but this test's rows. Installation counters use its days, the
/// leaving answers its week.
fn own_week() -> i32 {
    monday_of(-100_000 - (uuid::Uuid::new_v4().as_u128() % 10_000) as i32)
}

async fn columns(pool: &PgPool, table: &str) -> Vec<String> {
    sqlx::query(
        "SELECT column_name FROM information_schema.columns \
         WHERE table_schema = 'lingua' AND table_name = $1 ORDER BY ordinal_position",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .expect("columns")
    .iter()
    .map(|r| r.get::<String, _>("column_name"))
    .collect()
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with the lingua role"]
async fn the_counter_tables_have_no_room_for_anything_identifying() {
    let pool = pool().await;
    assert_eq!(
        columns(&pool, "usage_counts_daily").await,
        ["install_day", "step", "browser", "version", "count"]
    );
    assert_eq!(
        columns(&pool, "leave_reasons_weekly").await,
        ["week", "reason", "browser", "version", "count"]
    );
    for table in ["usage_counts_daily", "leave_reasons_weekly"] {
        for column in columns(&pool, table).await {
            for forbidden in [
                "ip",
                "addr",
                "header",
                "agent",
                "origin",
                "time",
                "at",
                "user",
                "account",
                "device",
                "install_id",
                "report",
            ] {
                assert!(
                    column != forbidden && !column.ends_with(&format!("_{forbidden}")),
                    "{table}.{column} looks like a column for {forbidden}"
                );
            }
        }
    }
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with the lingua role"]
async fn counters_grow_to_the_ceiling_and_are_read_back_by_day() {
    let pool = pool().await;
    let repo = PgUsageCountsRepo::new(pool.clone());
    let admin = PgLinguaAdminRepo::new(pool.clone());
    let week = own_week();
    // An installation day: the Wednesday of our week.
    let day = week + 2;

    // Two reports of one installation day on Firefox 1.10, one on Firefox 1.9, one on Safari.
    repo.count_steps(day, "firefox", "1.10", &["installed", "set_up"], 5_000)
        .await
        .unwrap();
    repo.count_steps(day, "firefox", "1.10", &["installed"], 5_000)
        .await
        .unwrap();
    repo.count_steps(day, "firefox", "1.9", &["installed"], 5_000)
        .await
        .unwrap();
    repo.count_steps(day, "safari", "1.10", &["installed"], 5_000)
        .await
        .unwrap();
    // A ceiling of 3 holds for the day's Firefox "installed", versions summed (2 + 1):
    // neither 1.10 nor a version with no row of its own yet grows past it.
    repo.count_steps(day, "firefox", "1.10", &["installed"], 3)
        .await
        .unwrap();
    repo.count_steps(day, "firefox", "1.11", &["installed"], 3)
        .await
        .unwrap();
    // Safari's own counter is not held by Firefox's.
    repo.count_steps(day, "safari", "1.10", &["installed"], 3)
        .await
        .unwrap();
    // The next day has a ceiling of its own.
    repo.count_steps(day + 1, "firefox", "1.10", &["installed"], 3)
        .await
        .unwrap();
    // A ceiling of 0 creates nothing.
    repo.count_steps(day, "firefox", "1.10", &["first_analysis"], 0)
        .await
        .unwrap();

    let firefox = admin
        .usage_funnel(day, day, Some("firefox"), 3)
        .await
        .unwrap();
    let get = |rows: &[cymbra_lingua::admin_core::FunnelRow], step: &str| {
        rows.iter()
            .find(|r| r.step == step)
            .map(|r| (r.count, r.capped))
    };
    // Versions summed: 2 (1.10) + 1 (1.9), and capped — the mark is true of the sum shown.
    assert_eq!(get(&firefox, "installed"), Some((3, true)));
    let versions: Vec<String> = sqlx::query_scalar(
        "SELECT version FROM lingua.usage_counts_daily \
         WHERE install_day = $1 AND browser = 'firefox' AND step = 'installed' ORDER BY version",
    )
    .bind(day)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(versions, ["1.10", "1.9"]);
    // Every browser at once: capped when one browser's sum reached the ceiling.
    let capped_all = admin.usage_funnel(day, day, None, 3).await.unwrap();
    assert_eq!(get(&capped_all, "installed"), Some((5, true)));
    assert_eq!(get(&firefox, "set_up"), Some((1, false)));
    assert_eq!(get(&firefox, "first_analysis"), None);
    let all = admin.usage_funnel(day, day, None, 5_000).await.unwrap();
    assert_eq!(get(&all, "installed"), Some((5, false)));
    // One row per installation day and step, the days apart.
    let week_rows = admin
        .usage_funnel(week, week + 6, Some("firefox"), 5_000)
        .await
        .unwrap();
    let installed: Vec<(i32, i64)> = week_rows
        .iter()
        .filter(|r| r.step == "installed")
        .map(|r| (r.day, r.count))
        .collect();
    assert_eq!(installed, [(day, 3), (day + 1, 1)]);
    // Outside the window, nothing.
    assert!(
        admin
            .usage_funnel(week + 7, week + 13, None, 5_000)
            .await
            .unwrap()
            .is_empty()
    );

    // Leaving reasons, the same way.
    repo.count_leave(week, "slow", "firefox", "1.10", 5_000)
        .await
        .unwrap();
    repo.count_leave(week, "slow", "unknown", "unknown", 5_000)
        .await
        .unwrap();
    // The reason's ceiling holds across versions too.
    repo.count_leave(week, "slow", "firefox", "other", 1)
        .await
        .unwrap();
    let reasons = admin
        .leave_reasons(week, week + 6, None, 5_000)
        .await
        .unwrap();
    assert_eq!(reasons.len(), 1);
    assert_eq!((reasons[0].reason.as_str(), reasons[0].count), ("slow", 2));
    let unknown = admin
        .leave_reasons(week, week + 6, Some("unknown"), 1)
        .await
        .unwrap();
    assert_eq!((unknown[0].count, unknown[0].capped), (1, true));

    // Retention: every day and week before the week after is deleted.
    repo.delete_before(week + 7).await.unwrap();
    assert!(
        admin
            .usage_funnel(week, week + 6, None, 5_000)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        admin
            .leave_reasons(week, week + 6, None, 5_000)
            .await
            .unwrap()
            .is_empty()
    );
}
