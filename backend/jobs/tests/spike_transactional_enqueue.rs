//! Spike (tasks 1.1–1.2): transactional enqueue against the shared `jobs` schema
//! under a confined module role, and the least-privilege grant (design D3 + the
//! recorded spike refinement). Requires the dev infra up
//! (`backend/docker-compose.yml`) with the schemas/roles bootstrapped; the test
//! runs the `jobs` migrations itself (as `worker_svc`).
//!
//! Run: `cargo test -p cymbra-jobs --test spike_transactional_enqueue -- --ignored`

use cymbra_jobs::{EnqueueRequest, JobSpec, RetryPolicy, transactional_enqueue};
use sqlx::Row;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

fn worker_url() -> String {
    std::env::var("CYMBRA_WORKER_DATABASE_URL").expect("CYMBRA_WORKER_DATABASE_URL must be set")
}
fn auth_url() -> String {
    std::env::var("CYMBRA_AUTH_DATABASE_URL").expect("CYMBRA_AUTH_DATABASE_URL must be set")
}

/// A spec whose channel_args we override per-test for isolation.
fn spec() -> JobSpec {
    JobSpec::new(
        "spike_email",
        cymbra_jobs::Channel::parallel("auth", "email"),
        RetryPolicy::new(3, Duration::from_secs(1), Duration::from_secs(60)),
    )
}

async fn count_msgs(worker: &sqlx::PgPool, channel_args: &str) -> i64 {
    sqlx::query("SELECT COUNT(*) AS n FROM jobs.mq_msgs WHERE channel_args = $1")
        .bind(channel_args)
        .fetch_one(worker)
        .await
        .unwrap()
        .get::<i64, _>("n")
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn enqueue_is_atomic_with_the_producer_transaction() {
    // Apply the jobs migrations as worker_svc (idempotent).
    let worker = PgPoolOptions::new()
        .max_connections(2)
        .connect(&worker_url())
        .await
        .expect("connect worker_svc");
    cymbra_jobs::MIGRATOR
        .run(&worker)
        .await
        .expect("migrate jobs");

    // Producer connects as the confined auth_svc role.
    let auth = PgPoolOptions::new()
        .max_connections(2)
        .connect(&auth_url())
        .await
        .expect("connect auth_svc");

    let tag = uuid::Uuid::new_v4().to_string();
    let mut req =
        EnqueueRequest::for_job(&spec(), &serde_json::json!({"to": "a@x.dev"}), None).unwrap();
    req.channel_args = tag.clone();

    // (a) Rollback drops the job: enqueue inside a tx, then roll back.
    let mut tx = auth.begin().await.unwrap();
    transactional_enqueue(&mut tx, &req)
        .await
        .expect("enqueue in tx");
    tx.rollback().await.unwrap();
    assert_eq!(
        count_msgs(&worker, &tag).await,
        0,
        "rolled-back job must not persist"
    );

    // (b) Commit keeps the job: same enqueue, committed this time.
    let mut tx = auth.begin().await.unwrap();
    transactional_enqueue(&mut tx, &req)
        .await
        .expect("enqueue in tx");
    tx.commit().await.unwrap();
    assert_eq!(
        count_msgs(&worker, &tag).await,
        1,
        "committed job must persist exactly once"
    );

    // Clean up.
    sqlx::query("SELECT jobs.mq_clear(ARRAY['auth.email'])")
        .execute(&worker)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn module_role_can_enqueue_but_not_read_queue_or_other_schema() {
    let worker = PgPoolOptions::new()
        .max_connections(1)
        .connect(&worker_url())
        .await
        .expect("connect worker_svc");
    cymbra_jobs::MIGRATOR
        .run(&worker)
        .await
        .expect("migrate jobs");

    let auth = PgPoolOptions::new()
        .max_connections(1)
        .connect(&auth_url())
        .await
        .expect("connect auth_svc");

    // CAN enqueue (EXECUTE on the SECURITY DEFINER wrapper).
    let req = EnqueueRequest::for_job(&spec(), &serde_json::json!({}), None).unwrap();
    let mut conn = auth.acquire().await.unwrap();
    transactional_enqueue(&mut conn, &req)
        .await
        .expect("module role may enqueue via jobs.enqueue");

    // CANNOT read the queue tables directly (no table privileges).
    assert!(
        sqlx::query("SELECT 1 FROM jobs.mq_msgs LIMIT 1")
            .execute(&auth)
            .await
            .is_err(),
        "module role must NOT be able to SELECT mq_msgs"
    );
    // CANNOT read another module's schema (D0 preserved).
    assert!(
        sqlx::query("SELECT 1 FROM user_account.users LIMIT 1")
            .execute(&auth)
            .await
            .is_err(),
        "module role must NOT be able to read user_account"
    );

    sqlx::query("SELECT jobs.mq_clear(ARRAY['auth.email'])")
        .execute(&worker)
        .await
        .unwrap();
}
