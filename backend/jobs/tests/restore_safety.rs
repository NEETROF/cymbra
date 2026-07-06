//! Regression guard for cold-restore safety (migration 0009).
//!
//! `pg_dump` restores with an EMPTY search_path and dumps function bodies
//! verbatim, so anything a function body references must be schema-qualified.
//! The vendored sqlxmq `mq_uuid_exists()` called `uuid_nil()` unqualified; since
//! uuid-ossp lives in schema `jobs`, a cold `pg_dump | psql` into a fresh database
//! failed to recreate the `mq_msgs` polling partial index (its predicate inlines
//! `mq_uuid_exists`), silently dropping the index. This test reproduces that exact
//! condition — calling the function under `search_path = ''` — so re-vendoring an
//! unqualified `uuid_nil()` fails here instead of in a future restore.
//!
//! Requires the dev infra with per-module roles bootstrapped (`db/init/00-roles.sh`).
//! Run: `cargo test -p cymbra-jobs --test restore_safety -- --ignored`

use sqlx::Row;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn mq_uuid_exists_resolves_under_empty_search_path() {
    let worker_url = std::env::var("CYMBRA_WORKER_DATABASE_URL")
        .expect("CYMBRA_WORKER_DATABASE_URL must be set");

    let worker = PgPoolOptions::new()
        .max_connections(1)
        .connect(&worker_url)
        .await
        .unwrap();
    cymbra_jobs::MIGRATOR.run(&worker).await.unwrap();

    // Emulate pg_dump's restore context: no schema on the search_path. Use one
    // pinned connection so the SET applies to the following queries (it overrides
    // worker_svc's role-level `search_path = jobs`).
    let mut conn = worker.acquire().await.unwrap();
    sqlx::query("SET search_path = ''")
        .execute(&mut *conn)
        .await
        .unwrap();

    // Before 0009 this errored with "function uuid_nil() does not exist" instead
    // of returning a boolean. Nil UUID → false; a non-nil UUID → true.
    let nil_exists: bool =
        sqlx::query("SELECT jobs.mq_uuid_exists('00000000-0000-0000-0000-000000000000'::uuid)")
            .fetch_one(&mut *conn)
            .await
            .expect("mq_uuid_exists must resolve under empty search_path (migration 0009)")
            .get(0);
    assert!(!nil_exists, "nil UUID must report as not-exists");

    let some_exists: bool =
        sqlx::query("SELECT jobs.mq_uuid_exists('11111111-1111-1111-1111-111111111111'::uuid)")
            .fetch_one(&mut *conn)
            .await
            .unwrap()
            .get(0);
    assert!(some_exists, "non-nil UUID must report as exists");
}
