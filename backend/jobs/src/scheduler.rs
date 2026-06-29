//! Scheduler DB glue (design D5; tasks 5.1–5.3 wiring). Coverage-excluded — the
//! decision logic it drives (cron/timezone occurrences, missed-run policy,
//! dedup key) is the host-tested [`crate::schedule`] core; this module is the
//! thin loop that reads `jobs.schedules`, and for each due occurrence enqueues a
//! singleton job inside one transaction (`ON CONFLICT DO NOTHING` on
//! `jobs.schedule_occurrences`) so exactly one job is created across replicas.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::Row;

use crate::engine::transactional_enqueue;
use crate::enqueue::EnqueueRequest;
use crate::error::Result;
use crate::registry;
use crate::schedule::{MissedRun, Schedule, bucket, dedup_key};

/// One row of `jobs.schedules`.
struct ScheduleRow {
    name: String,
    kind: String,
    cron_expr: String,
    timezone: String,
    enabled: bool,
    missed_run_policy: String,
    payload_json: String,
    last_evaluated_at: Option<DateTime<Utc>>,
}

/// Evaluate every schedule once and enqueue any due occurrences. Returns the
/// number of jobs enqueued this tick. Called on an interval by `cymbra-worker`.
pub async fn run_scheduler_tick(pool: &PgPool, now: DateTime<Utc>) -> Result<u64> {
    let rows = sqlx::query(
        "SELECT name, kind, cron_expr, timezone, enabled, missed_run_policy, \
                payload_json, last_evaluated_at \
         FROM jobs.schedules",
    )
    .fetch_all(pool)
    .await?;

    let mut enqueued = 0u64;
    for row in rows {
        let r = ScheduleRow {
            name: row.try_get("name")?,
            kind: row.try_get("kind")?,
            cron_expr: row.try_get("cron_expr")?,
            timezone: row.try_get("timezone")?,
            enabled: row.try_get("enabled")?,
            missed_run_policy: row.try_get("missed_run_policy")?,
            payload_json: row.try_get("payload_json")?,
            last_evaluated_at: row.try_get("last_evaluated_at")?,
        };
        enqueued += evaluate_one(pool, &r, now).await?;
        // Advance the evaluation cursor even when nothing was due, so a `skip`
        // schedule does not re-collapse the same window next tick.
        sqlx::query("UPDATE jobs.schedules SET last_evaluated_at = $1 WHERE name = $2")
            .bind(now)
            .bind(&r.name)
            .execute(pool)
            .await?;
    }
    Ok(enqueued)
}

async fn evaluate_one(pool: &PgPool, r: &ScheduleRow, now: DateTime<Utc>) -> Result<u64> {
    let missed = MissedRun::parse(&r.missed_run_policy)?;
    let sched = Schedule::parse(&r.name, &r.cron_expr, &r.timezone, r.enabled, missed)?;
    // First evaluation has no cursor → start from now (no historical backfill).
    let after = r.last_evaluated_at.unwrap_or(now);
    let occurrences = sched.occurrences_to_enqueue(after, now);

    let Some(spec) = registry::spec(&r.kind) else {
        tracing::warn!(schedule = %r.name, kind = %r.kind, "no JobSpec for schedule kind; skipping");
        return Ok(0);
    };
    let payload: serde_json::Value = serde_json::from_str(&r.payload_json)?;

    let mut count = 0u64;
    for occ in occurrences {
        let key = dedup_key(&r.name, bucket(occ));
        let mut tx = pool.begin().await?;
        let inserted: Option<i32> = sqlx::query_scalar(
            "INSERT INTO jobs.schedule_occurrences (dedup_key) VALUES ($1) \
             ON CONFLICT DO NOTHING RETURNING 1",
        )
        .bind(&key)
        .fetch_optional(&mut *tx)
        .await?;
        if inserted.is_some() {
            let req = EnqueueRequest::for_job(&spec, &payload, None)?;
            transactional_enqueue(&mut tx, &req).await?;
            count += 1;
        }
        tx.commit().await?;
    }
    Ok(count)
}
