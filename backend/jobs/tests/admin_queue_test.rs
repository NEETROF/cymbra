//! Jobs-console SQL integration tests (change: add-admin-jobs-console): the state
//! `jobs.admin_queue` gives each queued job, the cancellation outcomes including the
//! ordered-chain relink, the period figures, and what `jobs_admin_svc` may not do.
//! Change add-jobs-console-history adds the finished-attempt history, the per-kind
//! figures and the schedule list.
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
use sqlx::{Column, PgPool, Row};
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
        "SELECT payload_json FROM jobs.schedules",
        "SELECT last_error FROM jobs.dead_letter",
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
    for sql in [
        "SELECT * FROM jobs.admin_list_attempts(NOW() - INTERVAL '1 hour', NOW(), NULL, NULL, 1, 0)",
        "SELECT jobs.admin_count_attempts(NOW() - INTERVAL '1 hour', NOW(), NULL, NULL)",
        "SELECT * FROM jobs.admin_period_stats_by_kind(NOW() - INTERVAL '1 hour', NOW(), NULL)",
        "SELECT * FROM jobs.admin_list_schedules()",
    ] {
        sqlx::query(sql).fetch_all(&console).await.expect(sql);
    }
}

/// Insert finished (or running) attempts for `name`, each `minutes_ago` before one shared
/// `NOW()`, with a 2-second run time.
async fn attempts(pool: &PgPool, name: &str, channel: &str, rows: &[(Uuid, i32, &str, i64)]) {
    for (job_id, attempt, outcome, minutes_ago) in rows {
        let finished = (*outcome != "running").then_some(*minutes_ago);
        sqlx::query(
            "INSERT INTO jobs.job_attempts \
                 (job_id, job_name, channel_name, attempt, started_at, finished_at, outcome) \
             VALUES ($1, $2, $3, $4, \
                     NOW() - make_interval(mins => $5::int) - INTERVAL '2 seconds', \
                     NOW() - make_interval(mins => $6::int), $7)",
        )
        .bind(job_id)
        .bind(name)
        .bind(channel)
        .bind(attempt)
        .bind(*minutes_ago as i32)
        .bind(finished.map(|m| m as i32))
        .bind(outcome)
        .execute(pool)
        .await
        .unwrap();
    }
}

async fn purge_history(pool: &PgPool, name: &str) {
    for sql in [
        "DELETE FROM jobs.job_attempts WHERE job_name = $1",
        "DELETE FROM jobs.cancellations WHERE job_name = $1",
        "DELETE FROM jobs.dead_letter WHERE name = $1",
    ] {
        sqlx::query(sql).bind(name).execute(pool).await.unwrap();
    }
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with roles bootstrapped"]
async fn history_lists_finished_attempts_newest_first_and_pages() {
    let worker = worker().await;
    let console = console().await;
    let (name, channel) = unique("history");
    let (retried, ok, crashed, live) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    attempts(
        &worker,
        &name,
        &channel,
        &[
            (retried, 1, "failed", 50),
            (retried, 2, "succeeded", 40),
            (ok, 1, "succeeded", 30),
            (crashed, 1, "abandoned", 20),
            (live, 1, "running", 10),
            // Outside a 1-hour window.
            (ok, 1, "succeeded", 120),
        ],
    )
    .await;

    let page = |outcome: Option<&'static str>, limit: i32, offset: i32| {
        let console = console.clone();
        let name = name.clone();
        async move {
            sqlx::query(
                "SELECT job_id, attempt, outcome FROM jobs.admin_list_attempts(\
                 NOW() - INTERVAL '1 hour', NOW(), $1, $2, $3, $4)",
            )
            .bind(name)
            .bind(outcome)
            .bind(limit)
            .bind(offset)
            .fetch_all(&console)
            .await
            .unwrap()
            .into_iter()
            .map(|r| {
                (
                    r.get::<Uuid, _>("job_id"),
                    r.get::<i32, _>("attempt"),
                    r.get::<String, _>("outcome"),
                )
            })
            .collect::<Vec<_>>()
        }
    };
    let count = |outcome: Option<&'static str>| {
        let console = console.clone();
        let name = name.clone();
        async move {
            sqlx::query_scalar::<_, i64>(
                "SELECT jobs.admin_count_attempts(NOW() - INTERVAL '1 hour', NOW(), $1, $2)",
            )
            .bind(name)
            .bind(outcome)
            .fetch_one(&console)
            .await
            .unwrap()
        }
    };

    // Newest first, `running` never listed, the old attempt outside the window.
    assert_eq!(
        page(None, 10, 0).await,
        vec![
            (crashed, 1, "abandoned".to_owned()),
            (ok, 1, "succeeded".to_owned()),
            (retried, 2, "succeeded".to_owned()),
            (retried, 1, "failed".to_owned()),
        ]
    );
    assert_eq!(count(None).await, 4);

    // Paging continues the same order; a page past the end is empty but the total stays.
    assert_eq!(
        page(None, 2, 2).await,
        vec![
            (retried, 2, "succeeded".to_owned()),
            (retried, 1, "failed".to_owned()),
        ]
    );
    assert!(page(None, 2, 10).await.is_empty());

    // Outcome filter.
    assert_eq!(
        page(Some("failed"), 10, 0).await,
        vec![(retried, 1, "failed".to_owned())]
    );
    assert_eq!(count(Some("succeeded")).await, 2);
    assert_eq!(count(Some("running")).await, 0);

    // Kind filter: another kind's attempts are not listed.
    let (other, other_channel) = unique("history_other");
    attempts(
        &worker,
        &other,
        &other_channel,
        &[(Uuid::new_v4(), 1, "succeeded", 5)],
    )
    .await;
    assert_eq!(count(None).await, 4);

    purge_history(&worker, &name).await;
    purge_history(&worker, &other).await;
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with roles bootstrapped"]
async fn per_kind_figures_add_up_to_the_period_totals() {
    let worker = worker().await;
    let console = console().await;
    let (a, a_channel) = unique("bykind_a");
    let (b, b_channel) = unique("bykind_b");
    let (c, c_channel) = unique("bykind_c");

    attempts(
        &worker,
        &a,
        &a_channel,
        &[
            (Uuid::new_v4(), 1, "succeeded", 10),
            (Uuid::new_v4(), 1, "succeeded", 20),
            (Uuid::new_v4(), 1, "failed", 30),
        ],
    )
    .await;
    attempts(
        &worker,
        &b,
        &b_channel,
        &[(Uuid::new_v4(), 1, "abandoned", 15)],
    )
    .await;
    // `c` only has a dead letter and a cancellation: no attempt at all.
    let (dead, gone) = (Uuid::new_v4(), Uuid::new_v4());
    sqlx::query(
        "INSERT INTO jobs.dead_letter (id, name, channel_name, attempts, payload_json, last_error, \
             dead_lettered_at) \
         VALUES ($1, $2, $3, 0, '{\"to\":\"someone@example.com\"}', 'boom', NOW() - INTERVAL '5 minutes')",
    )
    .bind(dead)
    .bind(&c)
    .bind(&c_channel)
    .execute(&worker)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO jobs.cancellations (job_id, job_name, channel_name, cancelled_by, cancelled_at) \
         VALUES ($1, $2, $3, 'it-admin', NOW() - INTERVAL '6 minutes')",
    )
    .bind(gone)
    .bind(&c)
    .bind(&c_channel)
    .execute(&worker)
    .await
    .unwrap();

    let by_kind = |name: String| {
        let console = console.clone();
        async move {
            sqlx::query(
                "SELECT * FROM jobs.admin_period_stats_by_kind(NOW() - INTERVAL '1 hour', NOW(), $1)",
            )
            .bind(name)
            .fetch_all(&console)
            .await
            .unwrap()
        }
    };
    let total = |name: String| {
        let console = console.clone();
        async move {
            sqlx::query(
                "SELECT * FROM jobs.admin_period_stats(NOW() - INTERVAL '1 hour', NOW(), $1)",
            )
            .bind(name)
            .fetch_one(&console)
            .await
            .unwrap()
        }
    };

    for name in [&a, &b, &c] {
        let rows = by_kind(name.clone()).await;
        assert_eq!(rows.len(), 1, "{name}: one row per kind with activity");
        let (row, sum) = (&rows[0], total(name.clone()).await);
        assert_eq!(row.get::<String, _>("job_name"), *name);
        for col in ["completed", "failed_attempts", "dead_lettered", "cancelled"] {
            assert_eq!(
                row.get::<i64, _>(col),
                sum.get::<i64, _>(col),
                "{name}.{col}"
            );
        }
        assert_eq!(
            row.get::<Option<i64>, _>("avg_run_ms"),
            sum.get::<Option<i64>, _>("avg_run_ms"),
            "{name}.avg_run_ms"
        );
    }

    let a_row = &by_kind(a.clone()).await[0];
    assert_eq!(a_row.get::<i64, _>("completed"), 2);
    assert_eq!(a_row.get::<i64, _>("failed_attempts"), 1);
    assert_eq!(a_row.get::<Option<i64>, _>("avg_run_ms"), Some(2000));
    assert!(
        a_row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_finished_at")
            .is_some()
    );

    // Only an abandoned attempt: no figure, but it ran — listed, with its finish time.
    let b_row = &by_kind(b.clone()).await[0];
    assert_eq!(b_row.get::<i64, _>("completed"), 0);
    assert_eq!(b_row.get::<Option<i64>, _>("avg_run_ms"), None);
    assert!(
        b_row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_finished_at")
            .is_some()
    );

    let c_row = &by_kind(c.clone()).await[0];
    assert_eq!(c_row.get::<i64, _>("dead_lettered"), 1);
    assert_eq!(c_row.get::<i64, _>("cancelled"), 1);
    assert_eq!(
        c_row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_finished_at"),
        None
    );

    // Unfiltered, every test kind is there, in kind order.
    let names: Vec<String> = sqlx::query_scalar(
        "SELECT job_name FROM jobs.admin_period_stats_by_kind(NOW() - INTERVAL '1 hour', NOW(), NULL)",
    )
    .fetch_all(&console)
    .await
    .unwrap();
    let ours: Vec<&String> = names.iter().filter(|n| [&a, &b, &c].contains(n)).collect();
    let mut sorted = ours.clone();
    sorted.sort();
    assert_eq!(ours.len(), 3);
    assert_eq!(ours, sorted);

    // A window with nothing in it returns no row for these kinds.
    assert!(
        sqlx::query(
            "SELECT * FROM jobs.admin_period_stats_by_kind(NOW() - INTERVAL '3 hours', \
             NOW() - INTERVAL '2 hours', $1)",
        )
        .bind(&a)
        .fetch_all(&console)
        .await
        .unwrap()
        .is_empty()
    );

    for name in [&a, &b, &c] {
        purge_history(&worker, name).await;
    }
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with roles bootstrapped"]
async fn schedules_are_listed_without_their_payload() {
    let worker = worker().await;
    let console = console().await;
    let (name, _) = unique("schedule");
    sqlx::query(
        "INSERT INTO jobs.schedules (name, module, kind, cron_expr, timezone, enabled, payload_json) \
         VALUES ($1, 'it', $2, '0 */6 * * 1-5', 'Europe/Paris', false, '{\"secret\":true}')",
    )
    .bind(&name)
    .bind(format!("{name}_kind"))
    .execute(&worker)
    .await
    .unwrap();

    let rows = sqlx::query("SELECT * FROM jobs.admin_list_schedules()")
        .fetch_all(&console)
        .await
        .unwrap();
    let ours = rows
        .iter()
        .find(|r| r.get::<String, _>("name") == name)
        .expect("the test schedule is listed");
    assert_eq!(ours.get::<String, _>("kind"), format!("{name}_kind"));
    assert_eq!(ours.get::<String, _>("cron_expr"), "0 */6 * * 1-5");
    assert_eq!(ours.get::<String, _>("timezone"), "Europe/Paris");
    assert!(!ours.get::<bool, _>("enabled"));
    // The columns are exactly these: no payload, no scheduler bookkeeping.
    let columns: Vec<&str> = ours.columns().iter().map(|c| c.name()).collect();
    assert_eq!(
        columns,
        ["name", "kind", "cron_expr", "timezone", "enabled"]
    );
    // The seeded schedules are there too.
    assert!(
        rows.iter()
            .any(|r| r.get::<String, _>("name") == "orphan_reap_hourly")
    );

    sqlx::query("DELETE FROM jobs.schedules WHERE name = $1")
        .bind(&name)
        .execute(&worker)
        .await
        .unwrap();
}
