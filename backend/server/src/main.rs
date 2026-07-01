//! Cymbra ID — composition root (binary `cymbra-server`).
//!
//! The **only** place modules are wired: connect Postgres (per-module roles) +
//! Redis, run migrations, build the user + auth modules and their adapters,
//! install the internal-token interceptors, and serve gRPC + the Axum JWKS/health
//! surface. Contains no business logic.

use std::net::SocketAddr;
use std::sync::Arc;

use cymbra_auth::{AuthConfig, AuthGrpc, AuthModule, PgCredentialRepo, RealOidcVerifier};
use cymbra_auth::{CredentialRepo, OidcProviderCfg, OidcVerifier};
use cymbra_auth_port::proto::auth_service_server::AuthServiceServer;
use cymbra_platform::cache::{Cache, RedisCache};
use cymbra_platform::config::Config;
use cymbra_platform::email::{EmailSender, SmtpSender};
use cymbra_platform::interceptor::{AuthInterceptor, OptionalAuthInterceptor};
use cymbra_platform::{db, metrics, telemetry};
use cymbra_user::{PgUserRepo, UserGrpc, UserModule};
use cymbra_user_port::UserPort;
use cymbra_user_port::proto::user_service_server::UserServiceServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load `backend/.env` (repo-root run) or `.env` (run from backend/) if present.
    // Real environment variables always win over the file.
    let _ = dotenvy::from_filename("backend/.env").or_else(|_| dotenvy::dotenv());
    let cfg = Config::from_env()?;
    let telemetry = telemetry::init(
        "cymbra-server",
        cfg.otlp_enabled,
        cfg.otlp_endpoint.as_deref(),
    )?;
    metrics::install_resource_metrics();
    let red = std::sync::Arc::new(metrics::RedMetrics::new());

    // --- Postgres (per-module roles) + migrations ---
    let auth_pool = db::connect(&cfg.auth_database_url, 5).await?;
    let user_pool = db::connect(&cfg.user_database_url, 5).await?;
    let ready_pool = user_pool.clone(); // for the readiness probe
    cymbra_auth::MIGRATOR.run(&auth_pool).await?;
    cymbra_user::MIGRATOR.run(&user_pool).await?;

    // --- Redis (sessions, rate-limit) ---
    let cache: Arc<dyn Cache> = Arc::new(RedisCache::connect(&cfg.redis_url).await?);

    // --- user module ---
    let user_concrete = Arc::new(UserModule::new(PgUserRepo::new(user_pool)));
    let user_dyn: Arc<dyn UserPort> = user_concrete.clone();

    // --- auth module ---
    let creds: Arc<dyn CredentialRepo> = Arc::new(PgCredentialRepo::new(auth_pool));
    let providers: Vec<OidcProviderCfg> = cfg
        .oidc_providers
        .iter()
        .map(|p| OidcProviderCfg {
            provider: p.provider.clone(),
            issuer: p.issuer.clone(),
            audience: p.audience.clone(),
            jwks_uri: p.jwks_uri.clone(),
        })
        .collect();
    let oidc: Arc<dyn OidcVerifier> = Arc::new(RealOidcVerifier::new(providers));
    let email: Arc<dyn EmailSender> = Arc::new(SmtpSender::new(&cfg.smtp_url, &cfg.smtp_from)?);
    let auth_cfg = AuthConfig::new(
        cfg.token.access_ttl,
        cfg.token.refresh_ttl,
        cfg.allowed_audiences.clone(),
        cfg.password_min_length,
        cfg.signin_max_attempts,
        cfg.signin_lockout,
        cfg.email_max,
        cfg.email_window,
        cfg.verify_ttl,
        cfg.reset_ttl,
    );
    let auth = Arc::new(AuthModule::new(
        user_dyn,
        creds,
        cache.clone(),
        email,
        oidc,
        &cfg.token.signing_key_pem,
        &cfg.token.kid,
        auth_cfg,
    )?);

    // The orphan reaper no longer runs in-process here: it is a scheduled job
    // (`orphan_reap`) executed by cymbra-worker (change: add-job-infrastructure).
    // cymbra-worker MUST be deployed for handle-less accounts to be purged.

    // --- interceptors (strict for user; optional for auth's public methods) ---
    let keys = cymbra_server::interceptor_keys(&cfg)?;
    let strict = AuthInterceptor::new(keys.clone(), cfg.allowed_audiences.clone());
    let optional = OptionalAuthInterceptor::new(keys, cfg.allowed_audiences.clone());

    let user_svc = UserServiceServer::with_interceptor(UserGrpc::new(user_concrete), strict);
    let auth_svc = AuthServiceServer::with_interceptor(AuthGrpc::new(auth), optional);

    // --- HTTP surface (JWKS + health) ---
    let jwks = cymbra_server::jwks_value(&cfg)?;
    let http = cymbra_server::http_router(jwks, ready_pool, cache.clone());

    let grpc_addr: SocketAddr = cfg.grpc_addr.parse()?;
    let http_addr: SocketAddr = cfg.http_addr.parse()?;
    tracing::info!(%grpc_addr, %http_addr, "cymbra-server serving");

    let grpc = Server::builder()
        .layer(metrics::ObserveLayer::new(red))
        .add_service(user_svc)
        .add_service(auth_svc)
        .serve(grpc_addr);
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    let http_srv = axum::serve(listener, http.into_make_service());

    let result = tokio::try_join!(async { grpc.await.map_err(anyhow::Error::from) }, async {
        http_srv.await.map_err(anyhow::Error::from)
    });
    telemetry.shutdown();
    result?;
    Ok(())
}
