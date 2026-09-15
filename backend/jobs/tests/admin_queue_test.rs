//! Jobs-console SQL integration tests (change: add-admin-jobs-console): the state
//! `jobs.admin_queue` gives each queued job, the cancellation outcomes including the
//! ordered-chain relink, the period figures, and what `jobs_admin_svc` may not do.
//!
//! Requires the dev infra with roles bootstrapped (`db/init/00-roles.sh` creates
//! `jobs_admin_svc`); runs the `jobs` migrations itself as worker_svc. Every test uses
//! its own job names and channels, so a worker running against the same database never
//! picks these jobs up (it has no handler for them) and tests do not see each other.
//!
//! Run: `cargo test -p cymbra-jobs --test admin_queue_test -- --ignored`

use std::collections::HashMap;
use std::time::Duration;

use cymbra_jobs::{AttemptStart, EnqueueRequest, begin_attempt, transactional_enqueue};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;

async fn connect(var: &str) -> PgPool {
    let url = std::env::var(var).unwrap_or_else(|_| panic!("{var} must be set"));
    PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .unwrap()
}

async fn worker() -> PgPool {
    let pool = connect("CYMBRA_WORKER_DATABASE_URL").await;
    cymbra_jobs::MIGRATOR.run(&pool).await.unwrap();
    pool
}

async fn console() -> PgPool {
    connect("CYMBRA_JOBS_ADMIN_DATABASE_URL").await
}

/// A job name and a channel nothing else uses.
fn unique(prefix: &str) -> (String, String) {
    let tag = Uuid::new_v4().simple().to_string();
    (format!("it_{prefix}_{tag}"), format!("it.{prefix}.{tag}"))
}

async fn enqueue(pool: &PgPool, name: &str, channel: &str, ordered: bool, delay: Duration) -> Uuid {
    let mut conn = pool.acquire().await.unwrap();
    transactional_enqueue(
        &mut conn,
        &EnqueueRequest {
            name: name.into(),
            channel_name: channel.into(),
            channel_args: String::new(),
            ordered,
            retries: 3,
            retry_backoff: Duration::from_secs(1),
            delay,
            // Shaped like a verification email: what the console must never surface.
            payload_json: r#"{"to":"someone@example.com","subject":"s","html":"h","text":"t"}"#
                .into(),
        },
    )
    .await
    .unwrap()
}

/// What sqlxmq does when a worker claims a job: one attempt spent, the lease set.
async fn claim(pool: &PgPool, id: Uuid) {
    sqlx::query(
        "UPDATE jobs.mq_msgs SET attempts = attempts - 1, attempt_at = NOW() + INTERVAL '1 hour' \
         WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await
    .unwrap();
}

async fn states(console: &PgPool, name: &str) -> HashMap<Uuid, String> {
    sqlx::query("SELECT id, state FROM jobs.admin_list_jobs(NULL, $1, 100, 0)")
        .bind(name)
        .fetch_all(console)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.get("id"), r.get("state")))
        .collect()
}

async fn cancel(console: &PgPool, id: Uuid, protected: &[&str]) -> String {
    sqlx::query_scalar("SELECT jobs.admin_cancel($1, 'it-admin', $2)")
        .bind(id)
        .bind(protected)
        .fetch_one(console)
        .await
        .unwrap()
}

/// Remove the test's jobs one at a time, in chain order (heads first), so an ordered
/// chain never has two heads at once.
async fn cleanup(pool: &PgPool, ids: &[Uuid]) {
    for id in ids {
        sqlx::query("SELECT jobs.mq_delete(ARRAY[$1]::uuid[])")
            .bind(id)
            .execute(pool)
            .await
            .unwrap();
    }
    sqlx::query("DELETE FROM jobs.job_attempts WHERE job_id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM jobs.cancellations WHERE job_id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with roles bootstrapped"]
async fn each_queued_job_gets_its_state() {
    let worker = worker().await;
    let console = console().await;
    let (name, channel) = unique("states");
    let now = Duration::ZERO;
    let later = Duration::from_secs(3600);

    let ready = enqueue(&worker, &name, &channel, false, now).await;
    let scheduled = enqueue(&worker, &name, &channel, false, later).await;

    // A failed attempt, then waiting for the backoff.
    let retry_wait = enqueue(&worker, &name, &channel, false, later).await;
    sqlx::query(
        "INSERT INTO jobs.job_attempts (job_id, job_name, channel_name, attempt, finished_at, outcome) \
         VALUES ($1, $2, $3, 1, NOW(), 'failed')",
    )
    .bind(retry_wait)
    .bind(&name)
    .bind(&channel)
    .execute(&worker)
    .await
    .unwrap();

    // Claimed and started: same `attempt_at` shape as `retry_wait`, told apart only by
    // the history — the reason it exists.
    let running = enqueue(&worker, &name, &channel, false, now).await;
    claim(&worker, running).await;
    assert!(matches!(
        begin_attempt(&worker, running, &name).await.unwrap(),
        AttemptStart::Started(_)
    ));

    let exhausted = enqueue(&worker, &name, &channel, false, now).await;
    sqlx::query("UPDATE jobs.mq_msgs SET attempts = 0, attempt_at = NULL WHERE id = $1")
        .bind(exhausted)
        .execute(&worker)
        .await
        .unwrap();

    let ordered = format!("{channel}.ordered");
    let head = enqueue(&worker, &name, &ordered, true, now).await;
    let blocked = enqueue(&worker, &name, &ordered, true, now).await;

    let got = states(&console, &name).await;
    assert_eq!(got.len(), 7);
    assert_eq!(got[&ready], "ready");
    assert_eq!(got[&scheduled], "scheduled");
    assert_eq!(got[&retry_wait], "retry_wait");
    assert_eq!(got[&running], "running");
    assert_eq!(got[&exhausted], "exhausted");
    assert_eq!(got[&head], "ready");
    assert_eq!(got[&blocked], "blocked");

    let counts: HashMap<String, i64> =
        sqlx::query("SELECT state, n FROM jobs.admin_queue_counts($1)")
            .bind(&name)
            .fetch_all(&console)
            .await
            .unwrap()
            .into_iter()
            .map(|r| (r.get("state"), r.get("n")))
            .collect();
    assert_eq!(counts["ready"], 2);
    assert_eq!(counts["running"], 1);
    assert_eq!(counts["blocked"], 1);

    let only_blocked: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM jobs.admin_list_jobs('blocked', $1, 100, 0)")
            .bind(&name)
            .fetch_all(&console)
            .await
            .unwrap();
    assert_eq!(only_blocked, vec![blocked]);

    cleanup(
        &worker,
        &[
            ready, scheduled, retry_wait, running, exhausted, head, blocked,
        ],
    )
    .await;
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with roles bootstrapped"]
async fn cancellation_refuses_running_and_protected_jobs_and_keeps_the_order() {
    let worker = worker().await;
    let console = console().await;
    let (name, channel) = unique("cancel");

    let a = enqueue(&worker, &name, &channel, true, Duration::ZERO).await;
    let b = enqueue(&worker, &name, &channel, true, Duration::ZERO).await;
    let c = enqueue(&worker, &name, &channel, true, Duration::ZERO).await;
    claim(&worker, a).await;
    begin_attempt(&worker, a, &name).await.unwrap();

    assert_eq!(cancel(&console, a, &[]).await, "running");

    // Cancelling the middle of the chain: C must now wait for A, not for nothing —
    // pointing it at nil would let it overtake the running head.
    assert_eq!(cancel(&console, b, &[]).await, "cancelled");
    let c_after: Option<Uuid> =
        sqlx::query_scalar("SELECT after_message_id FROM jobs.mq_msgs WHERE id = $1")
            .bind(c)
            .fetch_one(&worker)
            .await
            .unwrap();
    assert_eq!(c_after, Some(a));
    assert_eq!(states(&console, &name).await[&c], "blocked");
    let by: String =
        sqlx::query_scalar("SELECT cancelled_by FROM jobs.cancellations WHERE job_id = $1")
            .bind(b)
            .fetch_one(&worker)
            .await
            .unwrap();
    assert_eq!(by, "it-admin");
    assert_eq!(cancel(&console, b, &[]).await, "gone");

    // Once A's attempt has failed it is only waiting, so it can go; C becomes the head.
    sqlx::query(
        "UPDATE jobs.job_attempts SET outcome = 'failed', finished_at = NOW() \
         WHERE job_id = $1 AND outcome = 'running'",
    )
    .bind(a)
    .execute(&worker)
    .await
    .unwrap();
    assert_eq!(cancel(&console, a, &[]).await, "cancelled");
    assert_eq!(states(&console, &name).await[&c], "ready");

    let (protected_name, protected_channel) = unique("protected");
    let p = enqueue(
        &worker,
        &protected_name,
        &protected_channel,
        false,
        Duration::ZERO,
    )
    .await;
    assert_eq!(
        cancel(&console, p, &[protected_name.as_str()]).await,
        "protected"
    );
    assert!(states(&console, &protected_name).await.contains_key(&p));

    cleanup(&worker, &[c, p, a, b]).await;
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with roles bootstrapped"]
async fn period_figures_count_only_the_window() {
    let worker = worker().await;
    let console = console().await;
    let (name, channel) = unique("period");
    let (recent_ok, old_ok, failed, cancelled, dead) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );

    // One statement, so NOW() is the same instant everywhere and run times are exact.
    sqlx::query(
        "INSERT INTO jobs.job_attempts \
             (job_id, job_name, channel_name, attempt, started_at, finished_at, outcome) VALUES \
         ($1, $6, $7, 1, NOW() - INTERVAL '1 hour 2 seconds', NOW() - INTERVAL '1 hour', 'succeeded'), \
         ($2, $6, $7, 1, NOW() - INTERVAL '2 days 4 seconds', NOW() - INTERVAL '2 days', 'succeeded'), \
         ($3, $6, $7, 1, NOW() - INTERVAL '31 minutes', NOW() - INTERVAL '30 minutes', 'failed')",
    )
    .bind(recent_ok)
    .bind(old_ok)
    .bind(failed)
    .bind(cancelled)
    .bind(dead)
    .bind(&name)
    .bind(&channel)
    .execute(&worker)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO jobs.cancellations (job_id, job_name, channel_name, cancelled_by, cancelled_at) \
         VALUES ($1, $2, $3, 'it-admin', NOW() - INTERVAL '10 minutes')",
    )
    .bind(cancelled)
    .bind(&name)
    .bind(&channel)
    .execute(&worker)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO jobs.dead_letter (id, name, channel_name, attempts, dead_lettered_at) \
         VALUES ($1, $2, $3, 0, NOW() - INTERVAL '5 minutes')",
    )
    .bind(dead)
    .bind(&name)
    .bind(&channel)
    .execute(&worker)
    .await
    .unwrap();

    let day = sqlx::query(
        "SELECT * FROM jobs.admin_period_stats(NOW() - INTERVAL '24 hours', NOW(), $1)",
    )
    .bind(&name)
    .fetch_one(&console)
    .await
    .unwrap();
    assert_eq!(day.get::<i64, _>("completed"), 1);
    assert_eq!(day.get::<i64, _>("failed_attempts"), 1);
    assert_eq!(day.get::<i64, _>("cancelled"), 1);
    assert_eq!(day.get::<i64, _>("dead_lettered"), 1);
    assert_eq!(day.get::<Option<i64>, _>("avg_run_ms"), Some(2000));
    assert!(
        day.get::<Option<chrono::DateTime<chrono::Utc>>, _>("history_since")
            .is_some()
    );

    let week =
        sqlx::query("SELECT * FROM jobs.admin_period_stats(NOW() - INTERVAL '7 days', NOW(), $1)")
            .bind(&name)
            .fetch_one(&console)
            .await
            .unwrap();
    assert_eq!(week.get::<i64, _>("completed"), 2);
    assert_eq!(week.get::<Option<i64>, _>("avg_run_ms"), Some(3000));

    sqlx::query("DELETE FROM jobs.job_attempts WHERE job_name = $1")
        .bind(&name)
        .execute(&worker)
        .await
        .unwrap();
    sqlx::query("DELETE FROM jobs.cancellations WHERE job_name = $1")
        .bind(&name)
        .execute(&worker)
        .await
        .unwrap();
    sqlx::query("DELETE FROM jobs.dead_letter WHERE name = $1")
        .bind(&name)
        .execute(&worker)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with roles bootstrapped"]
async fn the_console_role_can_only_call_the_admin_functions() {
    let _worker = worker().await;
    let console = console().await;

    for sql in [
        "SELECT count(*) FROM jobs.mq_payloads",
        "SELECT count(*) FROM jobs.mq_msgs",
        "SELECT count(*) FROM jobs.job_attempts",
        "SELECT count(*) FROM jobs.admin_queue",
        "DELETE FROM jobs.mq_msgs WHERE false",
        "SELECT jobs.enqueue('x', 'it.x', '', false, 0, INTERVAL '1 second', INTERVAL '0', '{}')",
    ] {
        let err = sqlx::query(sql).execute(&console).await.expect_err(sql);
        let code = err
            .as_database_error()
            .and_then(|e| e.code())
            .map(|c| c.into_owned());
        assert_eq!(code.as_deref(), Some("42501"), "{sql}: {err}");
    }

    sqlx::query("SELECT * FROM jobs.admin_queue_counts(NULL)")
        .fetch_all(&console)
        .await
        .unwrap();
    sqlx::query("SELECT * FROM jobs.admin_period_stats(NOW() - INTERVAL '1 hour', NOW(), NULL)")
        .fetch_one(&console)
        .await
        .unwrap();
}
