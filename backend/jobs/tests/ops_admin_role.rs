//! Ops admin-role integration test (change: add-ops-db-access): `admin_svc` reads
//! AND writes across schemas via the predefined `pg_read_all_data` /
//! `pg_write_all_data` roles, but cannot run DDL (pure DML, owns nothing). The
//! complementary D0 assertion (a module role stays confined to its own schema) is
//! covered by `cymbra-user`'s `db_isolation` test.
//!
//! Requires the dev infra with roles bootstrapped (`db/init/00-roles.sh`).
//! Run: `cargo test -p cymbra-jobs --test ops_admin_role -- --ignored`

use sqlx::Row;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with roles bootstrapped"]
async fn admin_role_reads_writes_all_schemas_but_not_ddl() {
    let admin_url =
        std::env::var("CYMBRA_ADMIN_DATABASE_URL").expect("CYMBRA_ADMIN_DATABASE_URL must be set");
    let worker_url = std::env::var("CYMBRA_WORKER_DATABASE_URL")
        .expect("CYMBRA_WORKER_DATABASE_URL must be set");

    // Ensure the jobs schema/tables exist (migrate as worker_svc).
    let worker = PgPoolOptions::new()
        .max_connections(1)
        .connect(&worker_url)
        .await
        .unwrap();
    cymbra_jobs::MIGRATOR.run(&worker).await.unwrap();

    let admin = PgPoolOptions::new()
        .max_connections(1)
        .connect(&admin_url)
        .await
        .expect("connect admin_svc");

    // Holds read + write on ALL data → every schema, present and future.
    let read: bool = sqlx::query("SELECT pg_has_role('pg_read_all_data', 'MEMBER')")
        .fetch_one(&admin)
        .await
        .unwrap()
        .get(0);
    let write: bool = sqlx::query("SELECT pg_has_role('pg_write_all_data', 'MEMBER')")
        .fetch_one(&admin)
        .await
        .unwrap()
        .get(0);
    assert!(
        read && write,
        "admin_svc must hold pg_read_all_data + pg_write_all_data"
    );

    // Read works (jobs here; the membership guarantees auth/user_account too).
    let _: i64 = sqlx::query("SELECT count(*) FROM jobs.schedules")
        .fetch_one(&admin)
        .await
        .unwrap()
        .get(0);

    // Write works: insert + delete a throwaway row.
    sqlx::query(
        "INSERT INTO jobs.schedules (name, module, kind, cron_expr) \
         VALUES ('admin_it_probe', 'x', 'x', '0 0 * * *')",
    )
    .execute(&admin)
    .await
    .expect("admin_svc must be able to write");
    sqlx::query("DELETE FROM jobs.schedules WHERE name = 'admin_it_probe'")
        .execute(&admin)
        .await
        .unwrap();

    // DDL is denied — admin owns nothing, holds data privileges only.
    let ddl = sqlx::query("CREATE TABLE jobs.admin_it_ddl (x int)")
        .execute(&admin)
        .await;
    assert!(ddl.is_err(), "admin_svc must NOT be able to run DDL");
}
