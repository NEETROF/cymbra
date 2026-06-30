//! Scheduler integration test (task 6.3): a recurring job (the orphan reaper) is
//! enqueued **exactly once per occurrence** even when several worker replicas
//! evaluate the same schedule — the idempotent time-bucket dedup (design D5).
//!
//! Requires the dev infra up; runs the `jobs` migrations itself (as worker_svc).
//! Run: `cargo test -p cymbra-jobs --test scheduler_test -- --ignored`

use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::postgres::PgPoolOptions;

fn utc(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn scheduled_reaper_enqueues_exactly_once_across_replicas() {
    let url = std::env::var("CYMBRA_WORKER_DATABASE_URL").expect("CYMBRA_WORKER_DATABASE_URL");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .unwrap();
    cymbra_jobs::MIGRATOR.run(&pool).await.unwrap();

    // Fresh, isolated test schedule for the reaper kind.
    let t0 = utc("2026-06-01T10:15:00Z");
    let now = utc("2026-06-01T11:45:00Z"); // window contains exactly one 11:00 occurrence
    sqlx::query("DELETE FROM jobs.schedule_occurrences WHERE dedup_key LIKE 'test_reap:%'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM jobs.schedules WHERE name = 'test_reap'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO jobs.schedules \
            (name, module, kind, cron_expr, timezone, enabled, missed_run_policy, last_evaluated_at) \
         VALUES ('test_reap','user','orphan_reap','0 * * * *','UTC',TRUE,'skip',$1)",
    )
    .bind(t0)
    .execute(&pool)
    .await
    .unwrap();

    // First replica evaluates → enqueues the single due occurrence.
    cymbra_jobs::run_scheduler_tick(&pool, now).await.unwrap();

    // Second replica that still has the old cursor re-evaluates the same window.
    sqlx::query("UPDATE jobs.schedules SET last_evaluated_at = $1 WHERE name = 'test_reap'")
        .bind(t0)
        .execute(&pool)
        .await
        .unwrap();
    cymbra_jobs::run_scheduler_tick(&pool, now).await.unwrap();

    // Exactly one occurrence was recorded (the dedup made the second a no-op).
    let occurrences: i64 = sqlx::query(
        "SELECT COUNT(*) AS n FROM jobs.schedule_occurrences WHERE dedup_key LIKE 'test_reap:%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap()
    .get::<i64, _>("n");
    assert_eq!(occurrences, 1, "exactly one occurrence across replicas");

    // Clean up.
    sqlx::query("DELETE FROM jobs.schedules WHERE name = 'test_reap'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM jobs.schedule_occurrences WHERE dedup_key LIKE 'test_reap:%'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("SELECT jobs.mq_clear(ARRAY['user.reap'])")
        .execute(&pool)
        .await
        .unwrap();
}
