//! Integration test (task 7.5): the local auth lifecycle against real Postgres +
//! Redis (the auth/user modules wired with their Pg repos + RedisCache; OIDC +
//! email faked). Requires the dev infra up.
//!
//! Run: `cargo test -p cymbra-auth --test auth_flow -- --ignored`

use std::sync::Arc;
use std::time::Duration;

use cymbra_auth::{
    AuthConfig, AuthModule, FakeOidcVerifier, PgCredentialRepo, PgSessionStore, SessionStore,
};
use cymbra_auth_port::AuthPort;
use cymbra_platform::cache::{Cache, RedisCache};
use cymbra_platform::email::{EmailSender, FakeEmail};
use cymbra_platform::{AppError, Result};
use cymbra_user::{PgUserRepo, UserModule};
use cymbra_user_port::UserPort;
use sqlx::postgres::PgPoolOptions;

const PRIV: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIPlT7JHCc7NTTIZVmlCgVeNNEkqsENhAZoscpnG+jSSw\n-----END PRIVATE KEY-----\n";
const PW: &str = "a-strong-passphrase";

#[tokio::test]
#[ignore = "needs docker compose (Postgres + Redis) up"]
async fn local_lifecycle_signup_verify_signin_refresh_reuse() -> Result<()> {
    let auth_url = std::env::var("CYMBRA_AUTH_DATABASE_URL").unwrap();
    let user_url = std::env::var("CYMBRA_USER_DATABASE_URL").unwrap();
    let redis_url =
        std::env::var("CYMBRA_REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());

    // The worker owns the `jobs` schema; in production it applies these
    // migrations (creating `jobs.enqueue` + granting auth_svc EXECUTE) before
    // cymbra-server enqueues. Mirror that here so sign-up's transactional enqueue
    // works — run them as worker_svc.
    let worker_url = std::env::var("CYMBRA_WORKER_DATABASE_URL").unwrap();
    let worker_pool = PgPoolOptions::new().connect(&worker_url).await.unwrap();
    cymbra_jobs::MIGRATOR.run(&worker_pool).await.unwrap();

    let auth_pool = PgPoolOptions::new().connect(&auth_url).await.unwrap();
    let user_pool = PgPoolOptions::new().connect(&user_url).await.unwrap();
    cymbra_auth::MIGRATOR.run(&auth_pool).await.unwrap();
    cymbra_user::MIGRATOR.run(&user_pool).await.unwrap();

    let user: Arc<dyn UserPort> = Arc::new(UserModule::new(PgUserRepo::new(user_pool)));
    let creds = Arc::new(PgCredentialRepo::new(auth_pool.clone()));
    let cache: Arc<dyn Cache> = Arc::new(RedisCache::connect(&redis_url).await?);
    let email: Arc<dyn EmailSender> = Arc::new(FakeEmail::default());
    let oidc = Arc::new(FakeOidcVerifier::default());
    let cfg = AuthConfig::new(
        Duration::from_secs(900),
        Duration::from_secs(2_592_000),
        vec!["music".into()],
        12,
        3,
        Duration::from_secs(60),
        5,
        Duration::from_secs(3600),
        Duration::from_secs(86_400),
        Duration::from_secs(3600),
    );
    let sessions: Arc<dyn SessionStore> =
        Arc::new(PgSessionStore::new(auth_pool.clone(), cfg.refresh_ttl));
    let m = AuthModule::new(user, creds, cache, email, oidc, sessions, PRIV, "k1", cfg)?;

    // Unique email per run.
    let email_addr = format!("it-{}@x.dev", uuid::Uuid::new_v4());

    m.sign_up_local(&email_addr, PW).await?;
    // Pull the verification token straight from the auth schema.
    let token: String =
        sqlx::query_scalar("SELECT verification_token FROM local_credentials WHERE email = $1")
            .bind(&email_addr)
            .fetch_one(&auth_pool)
            .await
            .unwrap();
    m.verify_email(&token).await?;

    let pair = m.sign_in_local(&email_addr, PW, "music").await?;
    let rotated = m.refresh(&pair.refresh_token).await?;

    // Replaying the original (now rotated) refresh token is reuse → rejected.
    assert!(matches!(
        m.refresh(&pair.refresh_token).await,
        Err(AppError::Unauthenticated(_))
    ));
    // The whole family is revoked, so the rotated token is dead too.
    assert!(matches!(
        m.refresh(&rotated.refresh_token).await,
        Err(AppError::Unauthenticated(_))
    ));
    Ok(())
}

/// Durable session store directly: rotate/replay, revoke, revoke_all, listing,
/// expiry, and the reap. (change: durable-sessions-postgres, task 6.1)
#[tokio::test]
#[ignore = "needs docker compose (Postgres) up"]
async fn pg_session_store_lifecycle_and_reap() -> Result<()> {
    let auth_url = std::env::var("CYMBRA_AUTH_DATABASE_URL").unwrap();
    let auth_pool = PgPoolOptions::new().connect(&auth_url).await.unwrap();
    cymbra_auth::MIGRATOR.run(&auth_pool).await.unwrap();

    let store = PgSessionStore::new(auth_pool.clone(), Duration::from_secs(3600));
    let uid = format!("u-{}", uuid::Uuid::new_v4());

    // create → rotate → replay revokes the family.
    let rt = store.create(&uid, "music").await?;
    assert_eq!(store.list_for_user(&uid).await?.len(), 1);
    let rot = store.rotate(&rt).await?;
    assert_eq!(rot.user_id, uid);
    assert_eq!(rot.audience, "music");
    assert!(matches!(
        store.rotate(&rt).await,
        Err(AppError::Unauthenticated(_))
    ));
    assert!(matches!(
        store.rotate(&rot.refresh_token).await,
        Err(AppError::Unauthenticated(_))
    ));
    assert!(store.list_for_user(&uid).await?.is_empty());

    // revoke (logout).
    let rt2 = store.create(&uid, "music").await?;
    store.revoke(&rt2).await?;
    assert!(matches!(
        store.rotate(&rt2).await,
        Err(AppError::Unauthenticated(_))
    ));

    // revoke_all ends every session for the user.
    let _a = store.create(&uid, "music").await?;
    let _b = store.create(&uid, "live").await?;
    assert_eq!(store.list_for_user(&uid).await?.len(), 2);
    store.revoke_all(&uid).await?;
    assert!(store.list_for_user(&uid).await?.is_empty());

    // concurrent rotate of the same token: exactly one succeeds.
    let rt3 = store.create(&uid, "music").await?;
    let (r1, r2) = tokio::join!(store.rotate(&rt3), store.rotate(&rt3));
    let wins = [r1.is_ok(), r2.is_ok()].iter().filter(|b| **b).count();
    assert_eq!(wins, 1, "exactly one concurrent rotate wins");

    // expired session is rejected on use and not listed, then reaped.
    let expired = PgSessionStore::new(auth_pool.clone(), Duration::from_secs(0));
    let ert = expired.create(&uid, "music").await?;
    assert!(matches!(
        expired.rotate(&ert).await,
        Err(AppError::Unauthenticated(_))
    ));
    assert!(store.list_for_user(&uid).await?.is_empty());
    let reaped = cymbra_auth::reap_expired_sessions(&auth_pool).await?;
    assert!(reaped >= 1, "reap deletes at least the expired row");
    Ok(())
}
