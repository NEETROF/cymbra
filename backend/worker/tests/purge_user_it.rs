//! Account-purge integration tests (task group 5, change: complete-account-deletion).
//! Drives the real `cymbra_worker::purge_user` erasure against live Postgres as
//! `admin_svc`, seeding across the `user_account` and `auth` schemas.
//!
//! Requires the dev infra with per-module roles bootstrapped (`db/init/00-roles.sh`)
//! and reachable via the `CYMBRA_*_DATABASE_URL` env vars.
//! Run: `cargo test -p cymbra-worker --test purge_user_it -- --ignored`

use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

async fn connect(var: &str) -> PgPool {
    let url = std::env::var(var).unwrap_or_else(|_| panic!("{var} must be set"));
    PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .unwrap_or_else(|e| panic!("connect {var}: {e}"))
}

/// Ensure the `user_account` + `auth` tables exist (idempotent). Migrations run as
/// their owning module roles — `admin_svc` holds DML only, not DDL.
async fn migrate() {
    let user = connect("CYMBRA_USER_DATABASE_URL").await;
    cymbra_user::MIGRATOR.run(&user).await.unwrap();
    let auth = connect("CYMBRA_AUTH_DATABASE_URL").await;
    cymbra_auth::MIGRATOR.run(&auth).await.unwrap();
}

/// Seed a user + one identity via `admin_svc` (crosses schemas in one place, which
/// is exactly what the purge does). Returns the new user id.
async fn seed_user(admin: &PgPool, provider: &str, subject: &str) -> uuid::Uuid {
    let uid = uuid::Uuid::now_v7();
    sqlx::query("INSERT INTO user_account.users (id) VALUES ($1)")
        .bind(uid)
        .execute(admin)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO user_account.user_identities (id, user_id, provider, subject) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(uid)
    .bind(provider)
    .bind(subject)
    .execute(admin)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO user_account.user_roles (user_id, scope, role) VALUES ($1, 'global', 'user')",
    )
    .bind(uid)
    .execute(admin)
    .await
    .unwrap();
    uid
}

async fn seed_local_credential(admin: &PgPool, email: &str) {
    sqlx::query("INSERT INTO auth.local_credentials (email, password_hash) VALUES ($1, $2)")
        .bind(email)
        .bind("argon2-hash")
        .execute(admin)
        .await
        .unwrap();
}

async fn seed_session(admin: &PgPool, user_id: &str) {
    sqlx::query(
        "INSERT INTO auth.sessions (id, user_id, audience, current_rt_hash, expires_at) \
         VALUES ($1, $2, 'music', $3, now() + interval '1 day')",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(user_id)
    .bind(format!("hash-{user_id}"))
    .execute(admin)
    .await
    .unwrap();
}

async fn count(admin: &PgPool, sql: &str, bind: &str) -> i64 {
    sqlx::query(sql)
        .bind(bind)
        .fetch_one(admin)
        .await
        .unwrap()
        .get::<i64, _>(0)
}

async fn users_count(admin: &PgPool, uid: uuid::Uuid) -> i64 {
    sqlx::query("SELECT count(*) FROM user_account.users WHERE id = $1::uuid")
        .bind(uid)
        .fetch_one(admin)
        .await
        .unwrap()
        .get::<i64, _>(0)
}

/// 5.1 / 5.2 / 5.3 — a full local account is erased across both schemas; the email
/// is registrable again and the sessions (refresh tokens) are gone.
#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn purge_local_account_erases_everything_and_frees_email() {
    migrate().await;
    let admin = connect("CYMBRA_ADMIN_DATABASE_URL").await;

    let email = format!("purge-{}@x.dev", uuid::Uuid::now_v7());
    let uid = seed_user(&admin, "local", &email).await;
    seed_local_credential(&admin, &email).await;
    seed_session(&admin, &uid.to_string()).await;

    cymbra_worker::purge_user(&admin, &uid.to_string())
        .await
        .expect("purge should succeed");

    // user_account row + cascaded identities/roles gone.
    assert_eq!(users_count(&admin, uid).await, 0, "users row must be gone");
    assert_eq!(
        count(
            &admin,
            "SELECT count(*) FROM user_account.user_identities WHERE user_id = $1::uuid",
            &uid.to_string(),
        )
        .await,
        0,
        "identities must cascade-delete"
    );
    // auth.local_credentials gone → email free to register again (5.2).
    assert_eq!(
        count(
            &admin,
            "SELECT count(*) FROM auth.local_credentials WHERE email = $1",
            &email,
        )
        .await,
        0,
        "credential must be gone"
    );
    seed_local_credential(&admin, &email).await; // re-registration succeeds
    // auth.sessions gone → any refresh with those tokens is rejected (5.3).
    assert_eq!(
        count(
            &admin,
            "SELECT count(*) FROM auth.sessions WHERE user_id = $1",
            &uid.to_string(),
        )
        .await,
        0,
        "sessions must be gone"
    );

    // cleanup
    sqlx::query("DELETE FROM auth.local_credentials WHERE email = $1")
        .bind(&email)
        .execute(&admin)
        .await
        .unwrap();
}

/// 5.4 — an OIDC-only account (no local credential) purges cleanly; the missing
/// credential is not an error.
#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn purge_oidc_only_account_succeeds() {
    migrate().await;
    let admin = connect("CYMBRA_ADMIN_DATABASE_URL").await;

    let uid = seed_user(
        &admin,
        "google",
        &format!("google-sub-{}", uuid::Uuid::now_v7()),
    )
    .await;
    seed_session(&admin, &uid.to_string()).await; // OIDC users still have sessions

    cymbra_worker::purge_user(&admin, &uid.to_string())
        .await
        .expect("OIDC-only purge must succeed (no local credential to remove)");

    assert_eq!(users_count(&admin, uid).await, 0);
    assert_eq!(
        count(
            &admin,
            "SELECT count(*) FROM auth.sessions WHERE user_id = $1",
            &uid.to_string(),
        )
        .await,
        0
    );
}

/// 5.5 — running the purge twice for the same user is a successful no-op.
#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn purge_is_idempotent() {
    migrate().await;
    let admin = connect("CYMBRA_ADMIN_DATABASE_URL").await;

    let email = format!("idem-{}@x.dev", uuid::Uuid::now_v7());
    let uid = seed_user(&admin, "local", &email).await;
    seed_local_credential(&admin, &email).await;
    seed_session(&admin, &uid.to_string()).await;

    cymbra_worker::purge_user(&admin, &uid.to_string())
        .await
        .expect("first purge succeeds");
    // Second run: everything is already gone → still succeeds, deletes nothing.
    cymbra_worker::purge_user(&admin, &uid.to_string())
        .await
        .expect("second purge is a no-op success");

    assert_eq!(users_count(&admin, uid).await, 0);
}
