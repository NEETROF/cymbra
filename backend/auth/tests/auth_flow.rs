//! Integration test (task 7.5): the local auth lifecycle against real Postgres +
//! Redis (the auth/user modules wired with their Pg repos + RedisCache; OIDC +
//! email faked). Requires the dev infra up.
//!
//! Run: `cargo test -p cymbra-auth --test auth_flow -- --ignored`

use std::sync::Arc;
use std::time::Duration;

use cymbra_auth::{AuthConfig, AuthModule, FakeOidcVerifier, PgCredentialRepo};
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
    // cymbra-id enqueues. Mirror that here so sign-up's transactional enqueue
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
    let m = AuthModule::new(user, creds, cache, email, oidc, PRIV, "k1", cfg)?;

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
