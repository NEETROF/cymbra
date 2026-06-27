//! Integration test (task 7.2): per-module DB roles confine each module to its
//! own schema. Requires the dev infra up (`backend/docker-compose.yml`) with the
//! schemas/roles bootstrapped and migrations applied.
//!
//! Run: `cargo test -p cymbra-user --test db_isolation -- --ignored`

use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "needs docker compose (Postgres) up with per-module roles"]
async fn auth_role_cannot_read_user_schema() {
    let auth_url =
        std::env::var("CYMBRA_AUTH_DATABASE_URL").expect("CYMBRA_AUTH_DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&auth_url)
        .await
        .expect("connect as auth_svc");

    // The `auth_svc` role has privileges only on the `auth` schema; reaching into
    // `user_account` must be rejected by Postgres.
    let res = sqlx::query("SELECT 1 FROM user_account.users LIMIT 1")
        .execute(&pool)
        .await;

    assert!(
        res.is_err(),
        "auth role must NOT be able to read the user_account schema"
    );
}
