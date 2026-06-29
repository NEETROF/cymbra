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

use std::time::Duration;

use async_trait::async_trait;
use sqlx::postgres::types::PgInterval;
use sqlx::{PgConnection, PgPool, Row};
use uuid::Uuid;

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

    tx.commit().await?;
    Ok(moved as u64)
}
