//! Database glue for the job substrate (coverage-excluded; exercised by the
//! `#[ignore]` integration tests against live Postgres). Everything here is thin
//! I/O over the `jobs` schema:
//!
//! * [`transactional_enqueue`] — the design's headline property: enqueue a job
//!   **inside the producer's own transaction** by calling the `SECURITY DEFINER`
//!   `jobs.enqueue(...)` wrapper on the caller's connection (design D1/D3).
//! * [`PgEnqueuer`] — a pool-backed [`Enqueuer`] for producers that don't need to
//!   share a transaction.
//! * [`dead_letter_sweep`] — moves retry-exhausted messages out of `mq_msgs` into
//!   `jobs.dead_letter` and removes them from the queue (design D6).
//! * [`tracked`] — runs one job attempt with its start and outcome recorded in
//!   `jobs.job_attempts` (change: add-admin-jobs-console, design D1); every worker
//!   handler goes through it. [`prune_history`] keeps that history bounded.

use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;
use sqlx::postgres::types::PgInterval;
use sqlx::{PgConnection, PgPool, Row};
use uuid::Uuid;

use crate::attempt::{AttemptOutcome, AttemptStart, HISTORY_RETENTION_DAYS};
use crate::enqueue::{EnqueueRequest, Enqueuer};
use crate::error::{JobError, Result};
use crate::retry::RetryPolicy;

fn interval(d: Duration) -> Result<PgInterval> {
    PgInterval::try_from(d).map_err(|e| {
        JobError::Engine(anyhow::anyhow!(
            "duration not representable as interval: {e}"
        ))
    })
}

/// Convert a Postgres `INTERVAL` to a `Duration` (months approximated at 30d —
/// fine for retry backoffs, which are seconds/minutes/hours).
fn interval_to_duration(i: &PgInterval) -> Duration {
    let micros = (i.months as i64) * 30 * 86_400 * 1_000_000
        + (i.days as i64) * 86_400 * 1_000_000
        + i.microseconds;
    Duration::from_micros(micros.max(0) as u64)
}

/// Read the runtime-tunable retry policy for `(module, kind)` from
/// `jobs.retry_policy` (task 4.1). Returns `None` when no row exists, so the
/// caller falls back to the job type's built-in default. The producer passes the
/// result to [`EnqueueRequest::for_job`], which maps it onto the enqueued job.
pub async fn load_retry_policy(
    conn: &mut PgConnection,
    module: &str,
    kind: &str,
) -> Result<Option<RetryPolicy>> {
    let row = sqlx::query(
        "SELECT max_attempts, base_backoff, max_backoff \
         FROM jobs.retry_policy WHERE module = $1 AND kind = $2",
    )
    .bind(module)
    .bind(kind)
    .fetch_optional(conn)
    .await?;

    Ok(row.map(|r| {
        let max_attempts: i32 = r.get("max_attempts");
        let base: PgInterval = r.get("base_backoff");
        let max: PgInterval = r.get("max_backoff");
        RetryPolicy::new(
            max_attempts.max(1) as u32,
            interval_to_duration(&base),
            interval_to_duration(&max),
        )
    }))
}

/// Enqueue a job inside the caller's transaction. `conn` is typically
/// `&mut *tx` for an in-progress `sqlx::Transaction`, so the job exists iff that
/// transaction commits (transactional enqueue, no dual-write).
pub async fn transactional_enqueue(conn: &mut PgConnection, req: &EnqueueRequest) -> Result<Uuid> {
    let id: Uuid = sqlx::query_scalar("SELECT jobs.enqueue($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind(&req.name)
        .bind(&req.channel_name)
        .bind(&req.channel_args)
        .bind(req.ordered)
        .bind(req.retries)
        .bind(interval(req.retry_backoff)?)
        .bind(interval(req.delay)?)
        .bind(&req.payload_json)
        .fetch_one(conn)
        .await?;
    Ok(id)
}

/// A pool-backed [`Enqueuer`]. Each call runs on its own pooled connection, so it
/// does **not** join a producer's transaction — use [`transactional_enqueue`]
/// with `&mut *tx` when atomicity with a business write is required.
pub struct PgEnqueuer {
    pool: PgPool,
}

impl PgEnqueuer {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Enqueuer for PgEnqueuer {
    async fn enqueue(&self, req: EnqueueRequest) -> Result<Uuid> {
        let mut conn = self.pool.acquire().await?;
        transactional_enqueue(&mut conn, &req).await
    }
}

/// Move every retry-exhausted message (sqlxmq leaves them with `attempts <= 0`
/// and `attempt_at IS NULL`) into `jobs.dead_letter` and delete it from the
/// queue. Returns the number of newly dead-lettered jobs (the caller raises an
/// alert / increments a metric when it is non-zero). Idempotent via
/// `ON CONFLICT (id) DO NOTHING`.
#[tracing::instrument(skip_all, name = "dlq.sweep")]
pub async fn dead_letter_sweep(pool: &PgPool) -> Result<u64> {
    let mut tx = pool.begin().await?;

    let moved: i64 = sqlx::query_scalar(
        r#"
        WITH dead AS (
            SELECT m.id, p.name, m.channel_name, m.channel_args, p.payload_json, m.attempts
            FROM jobs.mq_msgs m
            JOIN jobs.mq_payloads p ON p.id = m.id
            WHERE m.id != jobs.uuid_nil()
              AND m.attempt_at IS NULL
              AND m.attempts <= 0
              -- sqlxmq nulls `attempt_at` the moment it CLAIMS the last attempt, so a
              -- final attempt still running looks exhausted. Leave it alone until it
              -- ends or outlives the grace (change: add-admin-jobs-console, design D6);
              -- otherwise a long last attempt is dead-lettered mid-run and then succeeds.
              AND NOT EXISTS (
                  SELECT 1 FROM jobs.job_attempts a
                  WHERE a.job_id = m.id
                    AND a.outcome = 'running'
                    AND jobs.attempt_holds_lease(a.started_at, m.attempt_at)
              )
        ),
        moved AS (
            INSERT INTO jobs.dead_letter
                (id, name, channel_name, channel_args, payload_json, attempts)
            SELECT id, name, channel_name, channel_args, payload_json, attempts FROM dead
            ON CONFLICT (id) DO NOTHING
            RETURNING id
        )
        SELECT COUNT(*) FROM moved
        "#,
    )
    .fetch_one(&mut *tx)
    .await?;

    // Remove the dead-lettered messages from the queue (also fires the channel
    // NOTIFY so an ordered channel advances to its next message).
    sqlx::query(
        r#"
        SELECT jobs.mq_delete(COALESCE(ARRAY(
            SELECT id FROM jobs.dead_letter
            WHERE id IN (SELECT id FROM jobs.mq_msgs WHERE attempt_at IS NULL AND attempts <= 0)
        ), '{}'::uuid[]))
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // An attempt whose message has left the queue cannot finish on its own any more:
    // the job was just dead-lettered, or its worker died between completing the job and
    // recording the outcome. Close it so nothing stays "running" forever. A worker that
    // does report late still wins (see [`finish_attempt`]), so a completion racing this
    // sweep is not lost.
    sqlx::query(
        r#"
        UPDATE jobs.job_attempts a
        SET outcome = 'abandoned', finished_at = NOW()
        WHERE a.outcome = 'running'
          AND NOT EXISTS (SELECT 1 FROM jobs.mq_msgs m WHERE m.id = a.job_id)
        "#,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(moved as u64)
}

/// Rows deleted per table by one [`prune_history`] call, so a first prune after a long
/// outage never holds a large delete in one statement; the next tick takes the rest.
const PRUNE_BATCH: i64 = 5_000;

/// Delete attempt history and cancellation records older than
/// [`HISTORY_RETENTION_DAYS`] (change: add-admin-jobs-console, design D6). Run on the
/// dead-letter sweep's cadence. Returns the number of rows deleted.
#[tracing::instrument(skip_all, name = "jobs.history_prune")]
pub async fn prune_history(pool: &PgPool) -> Result<u64> {
    let days = HISTORY_RETENTION_DAYS as i32;
    let attempts = sqlx::query(
        "DELETE FROM jobs.job_attempts WHERE id IN ( \
             SELECT id FROM jobs.job_attempts \
             WHERE started_at < NOW() - make_interval(days => $1) LIMIT $2)",
    )
    .bind(days)
    .bind(PRUNE_BATCH)
    .execute(pool)
    .await?
    .rows_affected();
    let cancellations = sqlx::query(
        "DELETE FROM jobs.cancellations WHERE job_id IN ( \
             SELECT job_id FROM jobs.cancellations \
             WHERE cancelled_at < NOW() - make_interval(days => $1) LIMIT $2)",
    )
    .bind(days)
    .bind(PRUNE_BATCH)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(attempts + cancellations)
}

/// Record the start of an attempt of `job_id` (change: add-admin-jobs-console, design
/// D1). Locks the message first — the same lock `jobs.admin_cancel` takes — so a claim
/// and a cancellation serialise: if the message is gone, an operator cancelled it
/// between the claim and now, and the handler must not run. A `running` attempt left
/// by a crashed worker is closed as `abandoned` before the new one is recorded.
pub async fn begin_attempt(pool: &PgPool, job_id: Uuid, job_name: &str) -> Result<AttemptStart> {
    let mut tx = pool.begin().await?;
    let channel: Option<String> =
        sqlx::query_scalar("SELECT channel_name FROM jobs.mq_msgs WHERE id = $1 FOR UPDATE")
            .bind(job_id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some(channel) = channel else {
        tx.rollback().await?;
        return Ok(AttemptStart::Gone);
    };
    sqlx::query(
        "UPDATE jobs.job_attempts SET outcome = 'abandoned', finished_at = NOW() \
         WHERE job_id = $1 AND outcome = 'running'",
    )
    .bind(job_id)
    .execute(&mut *tx)
    .await?;
    let attempt: i64 = sqlx::query_scalar(
        "INSERT INTO jobs.job_attempts (job_id, job_name, channel_name, attempt) \
         SELECT $1, $2, $3, (COUNT(*) + 1)::INT FROM jobs.job_attempts WHERE job_id = $1 \
         RETURNING id",
    )
    .bind(job_id)
    .bind(job_name)
    .bind(&channel)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(AttemptStart::Started(attempt))
}

/// Record how an attempt ended. Overrides `abandoned` as well as `running`: the sweep
/// only guesses that a worker died, and the worker's own report is the truth.
pub async fn finish_attempt(pool: &PgPool, attempt: i64, outcome: AttemptOutcome) -> Result<()> {
    sqlx::query(
        "UPDATE jobs.job_attempts SET outcome = $2, finished_at = NOW() \
         WHERE id = $1 AND outcome IN ('running', 'abandoned')",
    )
    .bind(attempt)
    .bind(outcome.as_db())
    .execute(pool)
    .await?;
    Ok(())
}

/// Run one attempt of a job with its history recorded (change: add-admin-jobs-console,
/// design D1). Every worker handler wraps its body in this: `body` is not polled until
/// the attempt is recorded, and is dropped unpolled when the job was cancelled first.
///
/// Recording the start is part of running the job — if it fails, the attempt fails and
/// sqlxmq retries it. Recording the outcome is not: a failure there is logged and the
/// handler's own result stands, so a history hiccup never turns a sent email into a
/// retry that sends it twice.
pub async fn tracked<Fut, E>(
    pool: &PgPool,
    job_id: Uuid,
    job_name: &str,
    body: Fut,
) -> std::result::Result<(), E>
where
    Fut: Future<Output = std::result::Result<(), E>>,
    E: From<JobError>,
{
    let AttemptStart::Started(attempt) = begin_attempt(pool, job_id, job_name).await? else {
        tracing::info!(%job_id, job = job_name, "job left the queue before its attempt started; not running it");
        return Ok(());
    };
    let result = body.await;
    if let Err(e) = finish_attempt(pool, attempt, AttemptOutcome::of(&result)).await {
        tracing::warn!(error = %e, %job_id, job = job_name, "could not record the attempt outcome");
    }
    result
}
