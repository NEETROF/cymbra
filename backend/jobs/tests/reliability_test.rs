//! Reliability integration tests (task 4.3): bounded retry → success, retry
//! exhaustion → dead-letter, and an ordered channel advancing after a poison job
//! is dead-lettered. Drives the real sqlxmq runner against live Postgres.
//!
//! Requires the dev infra up; runs the `jobs` migrations itself (as worker_svc).
//! Run: `cargo test -p cymbra-jobs --test reliability_test -- --ignored`

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use cymbra_jobs::{
    Channel, EnqueueRequest, JobSpec, RetryPolicy, dead_letter_sweep, transactional_enqueue,
};
use sqlx::Row;
use sqlx::postgres::PgPoolOptions;
use sqlxmq::{CurrentJob, JobRegistry, JobRunnerHandle};

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Per-job attempt counter; lets a handler fail a configured number of times.
#[derive(Default)]
struct TestCtx {
    attempts: Mutex<HashMap<uuid::Uuid, u32>>,
}

#[derive(serde::Deserialize)]
struct Plan {
    fail_times: u32,
}

/// Fails `fail_times` then completes. Tracks attempts per job id.
#[sqlxmq::job("spike_flaky")]
async fn flaky(mut job: CurrentJob, ctx: Arc<TestCtx>) -> Result<(), BoxError> {
    let plan: Plan = job.json()?.ok_or("missing payload")?;
    let n = {
        let mut m = ctx.attempts.lock().unwrap();
        let e = m.entry(job.id()).or_insert(0);
        *e += 1;
        *e
    };
    if n <= plan.fail_times {
        return Err(format!("intentional failure {n}").into());
    }
    job.complete().await?;
    Ok(())
}

async fn connect(var: &str) -> sqlx::PgPool {
    let url = std::env::var(var).unwrap_or_else(|_| panic!("{var} must be set"));
    PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .unwrap()
}

/// Start a runner restricted to a single channel (so parallel tests don't steal
/// each other's jobs).
async fn runner_on(pool: &sqlx::PgPool, channel: &str, ctx: Arc<TestCtx>) -> JobRunnerHandle {
    let mut registry = JobRegistry::new(&[flaky]);
    registry.set_context(ctx);
    registry
        .runner(pool)
        .set_channel_names(&[channel])
        .set_concurrency(1, 4)
        .run()
        .await
        .unwrap()
}

fn spec(channel: Channel, retries: u32) -> JobSpec {
    JobSpec::new(
        "spike_flaky",
        channel,
        RetryPolicy::new(
            retries + 1,
            Duration::from_millis(50),
            Duration::from_secs(5),
        ),
    )
}

async fn channel_count(pool: &sqlx::PgPool, channel: &str) -> i64 {
    sqlx::query(
        "SELECT COUNT(*) AS n FROM jobs.mq_msgs WHERE channel_name = $1 AND id != jobs.uuid_nil()",
    )
    .bind(channel)
    .fetch_one(pool)
    .await
    .unwrap()
    .get::<i64, _>("n")
}

/// Dead-letters scoped to a channel, so concurrent tests (which share the
/// `spike_flaky` job name and the global `jobs.dead_letter` table) don't
/// contaminate each other's assertions — each test uses a unique channel.
async fn dead_letter_count(pool: &sqlx::PgPool, channel: &str) -> i64 {
    sqlx::query("SELECT COUNT(*) AS n FROM jobs.dead_letter WHERE channel_name = $1")
        .bind(channel)
        .fetch_one(pool)
        .await
        .unwrap()
        .get::<i64, _>("n")
}

/// Poll `f` until it returns true or the timeout elapses.
async fn wait_until<F, Fut>(timeout: Duration, mut f: F) -> bool
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if f().await {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn bounded_retry_then_success() {
    let pool = connect("CYMBRA_WORKER_DATABASE_URL").await;
    cymbra_jobs::MIGRATOR.run(&pool).await.unwrap();
    let ch = Channel::parallel("test", "retry");
    let name = ch.name();
    sqlx::query("SELECT jobs.mq_clear(ARRAY[$1])")
        .bind(&name)
        .execute(&pool)
        .await
        .unwrap();

    let ctx = Arc::new(TestCtx::default());
    let _runner = runner_on(&pool, &name, ctx.clone()).await;

    // 3 retries allowed; fail twice then succeed.
    let req = EnqueueRequest::for_job(
        &spec(ch.clone(), 3),
        &serde_json::json!({"fail_times": 2}),
        None,
    )
    .unwrap();
    let mut c = pool.acquire().await.unwrap();
    transactional_enqueue(&mut c, &req).await.unwrap();

    assert!(
        wait_until(Duration::from_secs(15), || async {
            channel_count(&pool, &name).await == 0
        })
        .await,
        "job should complete after bounded retries"
    );
    assert_eq!(dead_letter_count(&pool, &name).await, 0);
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn exhaustion_moves_to_dead_letter() {
    let pool = connect("CYMBRA_WORKER_DATABASE_URL").await;
    cymbra_jobs::MIGRATOR.run(&pool).await.unwrap();
    let ch = Channel::parallel("test", "exhaust");
    let name = ch.name();
    sqlx::query("SELECT jobs.mq_clear(ARRAY[$1])")
        .bind(&name)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM jobs.dead_letter WHERE channel_name = $1")
        .bind(&name)
        .execute(&pool)
        .await
        .unwrap();

    // Enqueue a job, then force the terminal state the runner leaves after the
    // last failed attempt (attempts spent, never to retry: attempts=0,
    // attempt_at NULL). Done via SQL rather than a live runner so this test of
    // the *sweep* is deterministic — natural retry exhaustion is exercised by
    // `bounded_retry_then_success`.
    let req = EnqueueRequest::for_job(&spec(ch.clone(), 0), &serde_json::json!({}), None).unwrap();
    let mut c = pool.acquire().await.unwrap();
    transactional_enqueue(&mut c, &req).await.unwrap();
    sqlx::query(
        "UPDATE jobs.mq_msgs SET attempts = 0, attempt_at = NULL \
         WHERE channel_name = $1 AND id != jobs.uuid_nil()",
    )
    .bind(&name)
    .execute(&pool)
    .await
    .unwrap();

    // Sweep moves it to the dead-letter store and out of the queue. Assert on
    // this channel's resulting state, not the global `moved` count (a concurrent
    // test's sweep may move the row first — the row still lands here).
    dead_letter_sweep(&pool).await.unwrap();
    assert!(
        wait_until(Duration::from_secs(5), || async {
            dead_letter_count(&pool, &name).await == 1 && channel_count(&pool, &name).await == 0
        })
        .await,
        "exhausted job should be dead-lettered and removed from the queue"
    );
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn poison_job_does_not_freeze_ordered_channel() {
    let pool = connect("CYMBRA_WORKER_DATABASE_URL").await;
    cymbra_jobs::MIGRATOR.run(&pool).await.unwrap();
    let ch = Channel::ordered("test", "ordered");
    let name = ch.name();
    sqlx::query("SELECT jobs.mq_clear(ARRAY[$1])")
        .bind(&name)
        .execute(&pool)
        .await
        .unwrap();

    let ctx = Arc::new(TestCtx::default());
    let _runner = runner_on(&pool, &name, ctx.clone()).await;

    // Poison first (always fails, 0 retries), then a healthy follow-up on the
    // SAME ordered channel — blocked behind the poison until it is dead-lettered.
    let poison = EnqueueRequest::for_job(
        &spec(ch.clone(), 0),
        &serde_json::json!({"fail_times": 999}),
        None,
    )
    .unwrap();
    let good = EnqueueRequest::for_job(
        &spec(ch.clone(), 0),
        &serde_json::json!({"fail_times": 0}),
        None,
    )
    .unwrap();
    let mut c = pool.acquire().await.unwrap();
    transactional_enqueue(&mut c, &poison).await.unwrap();
    transactional_enqueue(&mut c, &good).await.unwrap();

    // Wait until the poison is exhausted, then sweep it to the dead-letter store.
    assert!(
        wait_until(Duration::from_secs(15), || async {
            sqlx::query(
                "SELECT COUNT(*) AS n FROM jobs.mq_msgs WHERE channel_name=$1 AND attempts<=0 AND attempt_at IS NULL",
            )
            .bind(&name)
            .fetch_one(&pool)
            .await
            .unwrap()
            .get::<i64, _>("n")
                >= 1
        })
        .await,
        "poison should exhaust"
    );
    dead_letter_sweep(&pool).await.unwrap();

    // The follow-up should now drain — channel empties.
    assert!(
        wait_until(Duration::from_secs(15), || async {
            channel_count(&pool, &name).await == 0
        })
        .await,
        "successor must proceed once the poison is dead-lettered"
    );
}
