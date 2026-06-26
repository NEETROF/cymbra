//! Cymbra ID composition-root helpers (group 5).
//!
//! Keeps the wiring testable: `main` connects to real infra and calls these.

use std::collections::HashMap;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::{Router, extract::State, routing::get};
use cymbra_platform::cache::Cache;
use cymbra_platform::config::Config;
use cymbra_platform::{Result, db, jwks};
use jsonwebtoken::DecodingKey;
use serde_json::Value;
use sqlx::PgPool;

/// Liveness/readiness logic (pure; the HTTP/gRPC surfaces apply it).
pub mod health {
    /// Ready only when every critical dependency is reachable.
    pub fn ready(db_ok: bool, redis_ok: bool) -> bool {
        db_ok && redis_ok
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn requires_all_deps() {
            assert!(ready(true, true));
            assert!(!ready(false, true));
            assert!(!ready(true, false));
        }
    }
}

/// Build the JWKS document from the configured signing public key.
pub fn jwks_value(cfg: &Config) -> Result<Value> {
    jwks::jwks(&cfg.token.public_key_pem, &cfg.token.kid)
}

/// Decoding keys (by `kid`) the interceptor validates internal tokens against.
pub fn interceptor_keys(cfg: &Config) -> Result<HashMap<String, DecodingKey>> {
    let key = cymbra_platform::token::decoding_key(&cfg.token.public_key_pem)?;
    Ok(HashMap::from([(cfg.token.kid.clone(), key)]))
}

/// State for the Axum HTTP surface (JWKS + health/readiness).
#[derive(Clone)]
pub struct HttpState {
    jwks: Arc<Value>,
    pool: PgPool,
    cache: Arc<dyn Cache>,
}

/// Axum router: `/.well-known/jwks.json`, `/healthz` (liveness), `/readyz`
/// (readiness — pings Postgres + Redis).
pub fn http_router(jwks: Value, pool: PgPool, cache: Arc<dyn Cache>) -> Router {
    Router::new()
        .route("/.well-known/jwks.json", get(jwks_handler))
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(readyz_handler))
        .with_state(HttpState {
            jwks: Arc::new(jwks),
            pool,
            cache,
        })
}

async fn jwks_handler(State(s): State<HttpState>) -> axum::Json<Value> {
    axum::Json((*s.jwks).clone())
}

async fn readyz_handler(State(s): State<HttpState>) -> StatusCode {
    let db_ok = db::ping(&s.pool).await;
    let redis_ok = s.cache.ping().await;
    if health::ready(db_ok, redis_ok) {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

#[cfg(test)]
mod build_tests {
    use super::*;
    use cymbra_auth::{AuthConfig, AuthGrpc, AuthModule, FakeCredentialRepo, FakeOidcVerifier};
    use cymbra_auth::{CredentialRepo, OidcVerifier};
    use cymbra_platform::cache::FakeCache;
    use cymbra_platform::email::{EmailSender, FakeEmail};
    use cymbra_user::{FakeUserRepo, UserGrpc, UserModule};
    use cymbra_user_port::UserPort;
    use std::time::Duration;

    const PRIV: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIPlT7JHCc7NTTIZVmlCgVeNNEkqsENhAZoscpnG+jSSw\n-----END PRIVATE KEY-----\n";

    #[test]
    fn both_services_mount() {
        // user service builds
        let user_for_grpc = Arc::new(UserModule::new(FakeUserRepo::default()));
        let _user_server = UserGrpc::new(user_for_grpc).into_server();

        // auth service builds over the user port + fakes
        let user_for_auth: Arc<dyn UserPort> = Arc::new(UserModule::new(FakeUserRepo::default()));
        let creds: Arc<dyn CredentialRepo> = Arc::new(FakeCredentialRepo::default());
        let cache: Arc<dyn Cache> = Arc::new(FakeCache::default());
        let email: Arc<dyn EmailSender> = Arc::new(FakeEmail::default());
        let oidc: Arc<dyn OidcVerifier> = Arc::new(FakeOidcVerifier::default());
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
        let auth = Arc::new(
            AuthModule::new(user_for_auth, creds, cache, email, oidc, PRIV, "k1", cfg).unwrap(),
        );
        let _auth_server = AuthGrpc::new(auth).into_server();
    }
}
