//! Attempt-tracking integration tests (change: add-admin-jobs-console, design D1/D6):
//! `tracked` records each outcome and never runs a job cancelled before its start, a
//! reclaim closes the crashed attempt, the dead-letter sweep spares a running final
//! attempt until its grace ends, and the history prune keeps only the retention.
//!
//! A separate test binary from `admin_queue_test` on purpose: the sweep here acts on
//! the whole queue, and cargo runs test binaries one after the other, so it can never
//! dead-letter the other file's fixtures mid-assertion.
//!
//! Requires the dev infra up; runs the `jobs` migrations itself (as worker_svc).
//! Run: `cargo test -p cymbra-jobs --test attempt_tracking_test -- --ignored`

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use cymbra_jobs::{
    AttemptOutcome, AttemptStart, EnqueueRequest, begin_attempt, dead_letter_sweep, finish_attempt,
    prune_history, tracked, transactional_enqueue,
};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

async fn worker() -> PgPool {
    let url = std::env::var("CYMBRA_WORKER_DATABASE_URL")
        .expect("CYMBRA_WORKER_DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .unwrap();
    cymbra_jobs::MIGRATOR.run(&pool).await.unwrap();
    pool
}

fn unique(prefix: &str) -> (String, String) {
    let tag = Uuid::new_v4().simple().to_string();
    (format!("it_{prefix}_{tag}"), format!("it.{prefix}.{tag}"))
}

async fn enqueue(pool: &PgPool, name: &str, channel: &str) -> Uuid {
    let mut conn = pool.acquire().await.unwrap();
    transactional_enqueue(
        &mut conn,
        &EnqueueRequest {
            name: name.into(),
            channel_name: channel.into(),
            channel_args: String::new(),
            ordered: false,
            retries: 3,
            retry_backoff: Duration::from_secs(1),
            delay: Duration::ZERO,
            payload_json: "{}".into(),
        },
    )
    .await
    .unwrap()
}

async fn outcomes(pool: &PgPool, job: Uuid) -> Vec<(i32, String)> {
    sqlx::query_as(
        "SELECT attempt, outcome FROM jobs.job_attempts WHERE job_id = $1 ORDER BY attempt",
    )
    .bind(job)
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn in_queue(pool: &PgPool, job: Uuid) -> bool {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM jobs.mq_msgs WHERE id = $1)")
        .bind(job)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn dead_lettered(pool: &PgPool, job: Uuid) -> bool {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM jobs.dead_letter WHERE id = $1)")
        .bind(job)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn forget(pool: &PgPool, jobs: &[Uuid]) {
    sqlx::query("SELECT jobs.mq_delete($1)")
        .bind(jobs)
        .execute(pool)
        .await
        .unwrap();
    for table in [
        "DELETE FROM jobs.job_attempts WHERE job_id = ANY($1)",
        "DELETE FROM jobs.cancellations WHERE job_id = ANY($1)",
        "DELETE FROM jobs.dead_letter WHERE id = ANY($1)",
    ] {
        sqlx::query(table).bind(jobs).execute(pool).await.unwrap();
    }
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn tracked_records_each_outcome_and_skips_a_cancelled_job() {
    let pool = worker().await;
    let (name, channel) = unique("tracked");
    let ok = enqueue(&pool, &name, &channel).await;
    let flaky = enqueue(&pool, &name, &channel).await;
    let cancelled = enqueue(&pool, &name, &channel).await;

    tracked::<_, BoxError>(&pool, ok, &name, async { Ok(()) })
        .await
        .unwrap();
    let failure =
        tracked::<_, BoxError>(&pool, flaky, &name, async { Err("smtp down".into()) }).await;
    assert!(failure.is_err(), "the handler's error must reach sqlxmq");
    tracked::<_, BoxError>(&pool, flaky, &name, async { Ok(()) })
        .await
        .unwrap();

    assert_eq!(outcomes(&pool, ok).await, vec![(1, "succeeded".into())]);
    assert_eq!(
        outcomes(&pool, flaky).await,
        vec![(1, "failed".into()), (2, "succeeded".into())]
    );

    // Cancelled between the claim and the start: the body must never run.
    let word: String = sqlx::query_scalar("SELECT jobs.admin_cancel($1, 'it', NULL)")
        .bind(cancelled)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(word, "cancelled");
    let ran = AtomicBool::new(false);
    tracked::<_, BoxError>(&pool, cancelled, &name, async {
        ran.store(true, Ordering::SeqCst);
        Ok(())
    })
    .await
    .unwrap();
    assert!(!ran.load(Ordering::SeqCst));
    assert!(outcomes(&pool, cancelled).await.is_empty());

    forget(&pool, &[ok, flaky, cancelled]).await;
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn a_reclaimed_job_closes_the_crashed_attempt() {
    let pool = worker().await;
    let (name, channel) = unique("reclaim");
    let job = enqueue(&pool, &name, &channel).await;

    begin_attempt(&pool, job, &name).await.unwrap();
    // The first worker died; the lease expired and another worker starts the job.
    begin_attempt(&pool, job, &name).await.unwrap();

    assert_eq!(
        outcomes(&pool, job).await,
        vec![(1, "abandoned".into()), (2, "running".into())]
    );
    forget(&pool, &[job]).await;
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn the_sweep_spares_a_running_final_attempt_until_its_grace_ends() {
    let pool = worker().await;
    let (name, channel) = unique("grace");
    let job = enqueue(&pool, &name, &channel).await;

    // sqlxmq claiming the LAST attempt: no attempts left and no next attempt time.
    sqlx::query("UPDATE jobs.mq_msgs SET attempts = 0, attempt_at = NULL WHERE id = $1")
        .bind(job)
        .execute(&pool)
        .await
        .unwrap();
    let AttemptStart::Started(attempt) = begin_attempt(&pool, job, &name).await.unwrap() else {
        panic!("the job is in the queue");
    };

    dead_letter_sweep(&pool).await.unwrap();
    assert!(
        in_queue(&pool, job).await,
        "a running final attempt is not dead"
    );
    assert!(!dead_lettered(&pool, job).await);

    // Past the grace it is presumed crashed.
    sqlx::query(
        "UPDATE jobs.job_attempts SET started_at = NOW() - INTERVAL '2 hours' WHERE job_id = $1",
    )
    .bind(job)
    .execute(&pool)
    .await
    .unwrap();
    dead_letter_sweep(&pool).await.unwrap();
    assert!(!in_queue(&pool, job).await);
    assert!(dead_lettered(&pool, job).await);
    assert_eq!(outcomes(&pool, job).await, vec![(1, "abandoned".into())]);

    // A worker that was only slow still has the last word on its own attempt.
    finish_attempt(&pool, attempt, AttemptOutcome::Succeeded)
        .await
        .unwrap();
    assert_eq!(outcomes(&pool, job).await, vec![(1, "succeeded".into())]);

    forget(&pool, &[job]).await;
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn the_prune_keeps_only_the_retention_window() {
    let pool = worker().await;
    let (name, channel) = unique("prune");
    let (old, recent) = (Uuid::new_v4(), Uuid::new_v4());

    sqlx::query(
        "INSERT INTO jobs.job_attempts \
             (job_id, job_name, channel_name, attempt, started_at, finished_at, outcome) VALUES \
         ($1, $3, $4, 1, NOW() - INTERVAL '91 days', NOW() - INTERVAL '91 days', 'succeeded'), \
         ($2, $3, $4, 1, NOW() - INTERVAL '1 day', NOW() - INTERVAL '1 day', 'succeeded')",
    )
    .bind(old)
    .bind(recent)
    .bind(&name)
    .bind(&channel)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO jobs.cancellations (job_id, job_name, channel_name, cancelled_by, cancelled_at) VALUES \
         ($1, $3, $4, 'it', NOW() - INTERVAL '91 days'), ($2, $3, $4, 'it', NOW() - INTERVAL '1 day')",
    )
    .bind(old)
    .bind(recent)
    .bind(&name)
    .bind(&channel)
    .execute(&pool)
    .await
    .unwrap();

    assert!(prune_history(&pool).await.unwrap() >= 2);
    assert!(outcomes(&pool, old).await.is_empty());
    assert_eq!(outcomes(&pool, recent).await.len(), 1);
    let cancellations: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM jobs.cancellations WHERE job_name = $1")
            .bind(&name)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cancellations, 1);

    forget(&pool, &[old, recent]).await;
}
