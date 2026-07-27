//! Browser-only **web-auth HTTP surface** (change: add-web-auth-cookies).
//!
//! A thin gateway over the auth module that carries the long-lived refresh token in
//! an `HttpOnly; Secure; SameSite=Strict` cookie instead of exposing it to page
//! JavaScript. Three JSON endpoints, mounted on the existing Axum HTTP server:
//!
//! - `POST /web/auth/signin`  — local or OIDC sign-in; sets the refresh cookie,
//!   returns `{ accessToken }` (the refresh token never reaches the body).
//! - `POST /web/auth/refresh` — reads the cookie, rotates it via the auth module,
//!   re-sets the rotated cookie, returns `{ accessToken }`.
//! - `POST /web/auth/logout`  — revokes the session and clears the cookie.
//!
//! The gRPC `AuthService` (native `TokenPair` flow) is unchanged; this surface is
//! additive and web-only. CSRF is defended in depth: the cookie is `SameSite=Strict`,
//! every endpoint requires a custom non-simple header (`X-Cymbra-Web`, which forces a
//! CORS preflight a cross-site `<form>` cannot satisfy), and CORS echoes an exact
//! allowed origin with credentials (never `*`).

use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::header::{CONTENT_TYPE, COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use cymbra_auth_port::AuthPort;
use cymbra_platform::AppError;
use serde::Serialize;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Name of the refresh-token cookie. Opaque to the browser (JS can't read it).
pub const COOKIE_NAME: &str = "cymbra_refresh";

/// The custom request header that forces a CORS preflight (CSRF defence). A simple
/// cross-site `<form>` submission cannot set it, so it can never drive these endpoints.
const CSRF_HEADER: &str = "x-cymbra-web";

/// Per-environment cookie + CORS settings for the web-auth surface.
#[derive(Clone)]
pub struct WebAuthConfig {
    /// `Domain` attribute; `None` scopes the cookie to the exact API host.
    pub cookie_domain: Option<String>,
    /// `Secure` attribute; `false` only for plain-HTTP localhost dev.
    pub cookie_secure: bool,
    /// `Path` the cookie is scoped to (so it isn't sent to every request).
    pub cookie_path: String,
    /// Cookie `Max-Age` — the refresh-token lifetime (reused from auth config).
    pub refresh_ttl: Duration,
    /// Exact browser origins allowed to make credentialed calls (never `*`).
    pub allowed_origins: Vec<String>,
}

impl WebAuthConfig {
    /// Default cookie path: scoped to the auth endpoints only.
    pub const DEFAULT_PATH: &'static str = "/web/auth";
}

#[derive(Clone)]
struct WebAuthState {
    auth: Arc<dyn AuthPort>,
    cfg: Arc<WebAuthConfig>,
}

/// Sign-in request: internally tagged so one endpoint serves both local and OIDC.
#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum SignInBody {
    Local {
        email: String,
        password: String,
        audience: String,
    },
    Oidc {
        #[serde(rename = "idToken")]
        id_token: String,
        audience: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccessTokenBody {
    access_token: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

/// Build the web-auth router (CORS-wrapped), ready to `.merge()` into the HTTP server.
pub fn web_auth_router(auth: Arc<dyn AuthPort>, cfg: WebAuthConfig) -> Router {
    let origins: Vec<HeaderValue> = cfg
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    // Credentialed CORS: echo an exact allowed origin (tower-http never emits `*`
    // once `allow_credentials(true)` is set) and permit the CSRF + content-type
    // headers so the browser preflight succeeds.
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_credentials(true)
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, HeaderName::from_static(CSRF_HEADER)]);

    Router::new()
        .route("/web/auth/signin", post(signin))
        .route("/web/auth/refresh", post(refresh))
        .route("/web/auth/logout", post(logout))
        .layer(cors)
        .with_state(WebAuthState {
            auth,
            cfg: Arc::new(cfg),
        })
}

/// True when the preflight-forcing CSRF header is present.
fn csrf_ok(headers: &HeaderMap) -> bool {
    headers.contains_key(CSRF_HEADER)
}

/// Extract a cookie value by name from the request `Cookie` header (pure).
fn cookie_value<'a>(cookies: &'a str, name: &str) -> Option<&'a str> {
    cookies.split(';').find_map(|pair| {
        let (k, v) = pair.trim().split_once('=')?;
        (k == name).then_some(v)
    })
}

/// Read the refresh cookie off the request headers.
fn refresh_cookie(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(COOKIE)?.to_str().ok()?;
    cookie_value(raw, COOKIE_NAME).map(str::to_string)
}

/// `Set-Cookie` value that stores the rotated refresh token.
fn set_cookie(cfg: &WebAuthConfig, value: &str) -> String {
    build_cookie(cfg, value, cfg.refresh_ttl.as_secs())
}

/// `Set-Cookie` value that immediately expires the refresh cookie (Max-Age=0). Uses
/// the same Path/Domain so the browser matches and drops the stored cookie.
fn clear_cookie(cfg: &WebAuthConfig) -> String {
    build_cookie(cfg, "", 0)
}

fn build_cookie(cfg: &WebAuthConfig, value: &str, max_age: u64) -> String {
    let mut c = format!(
        "{COOKIE_NAME}={value}; Path={}; Max-Age={max_age}; HttpOnly; SameSite=Strict",
        cfg.cookie_path,
    );
    if cfg.cookie_secure {
        c.push_str("; Secure");
    }
    if let Some(domain) = &cfg.cookie_domain {
        c.push_str("; Domain=");
        c.push_str(domain);
    }
    c
}

/// Map an auth-module error to an HTTP status. 4xx messages are safe to surface
/// (per `AppError`); 5xx collapse to a generic message.
fn http_status(e: &AppError) -> StatusCode {
    match e {
        AppError::InvalidArgument(_) => StatusCode::BAD_REQUEST,
        AppError::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
        AppError::PermissionDenied(_) => StatusCode::FORBIDDEN,
        AppError::NotFound(_) => StatusCode::NOT_FOUND,
        AppError::AlreadyExists(_) | AppError::Aborted(_) => StatusCode::CONFLICT,
        AppError::FailedPrecondition(_) => StatusCode::PRECONDITION_FAILED,
        AppError::ResourceExhausted(_) => StatusCode::TOO_MANY_REQUESTS,
        AppError::Config(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Client-safe message: never leak an internal/config cause.
fn safe_message(e: &AppError) -> String {
    match e {
        AppError::Config(_) | AppError::Internal(_) => "internal error".to_string(),
        other => other.to_string(),
    }
}

/// A JSON response, optionally carrying one `Set-Cookie` header.
fn json_response<T: Serialize>(status: StatusCode, cookie: Option<String>, body: T) -> Response {
    let mut resp = (status, Json(body)).into_response();
    if let Some(c) = cookie
        && let Ok(v) = HeaderValue::from_str(&c)
    {
        resp.headers_mut().insert(SET_COOKIE, v);
    }
    resp
}

fn error_response(status: StatusCode, msg: &str, cookie: Option<String>) -> Response {
    json_response(status, cookie, ErrorBody { error: msg.into() })
}

async fn signin(
    State(s): State<WebAuthState>,
    headers: HeaderMap,
    body: Json<SignInBody>,
) -> Response {
    if !csrf_ok(&headers) {
        return error_response(StatusCode::FORBIDDEN, "missing csrf header", None);
    }
    let result = match body.0 {
        SignInBody::Local {
            email,
            password,
            audience,
        } => s.auth.sign_in_local(&email, &password, &audience).await,
        SignInBody::Oidc { id_token, audience } => s.auth.sign_in_oidc(&id_token, &audience).await,
    };
    match result {
        Ok(pair) => json_response(
            StatusCode::OK,
            Some(set_cookie(&s.cfg, &pair.refresh_token)),
            AccessTokenBody {
                access_token: pair.access_token,
            },
        ),
        // Failed sign-in sets no cookie and returns no token.
        Err(e) => error_response(http_status(&e), &safe_message(&e), None),
    }
}

async fn refresh(State(s): State<WebAuthState>, headers: HeaderMap) -> Response {
    if !csrf_ok(&headers) {
        return error_response(StatusCode::FORBIDDEN, "missing csrf header", None);
    }
    let Some(token) = refresh_cookie(&headers) else {
        // No cookie → treat as ended; clear anything stale.
        return error_response(
            StatusCode::UNAUTHORIZED,
            "no session",
            Some(clear_cookie(&s.cfg)),
        );
    };
    match s.auth.refresh(&token).await {
        Ok(pair) => json_response(
            StatusCode::OK,
            Some(set_cookie(&s.cfg, &pair.refresh_token)),
            AccessTokenBody {
                access_token: pair.access_token,
            },
        ),
        // Invalid / expired / reused (rotation reuse detection lives in the auth
        // module): 401 and clear the cookie so the client falls back to sign-in.
        Err(_) => error_response(
            StatusCode::UNAUTHORIZED,
            "session ended",
            Some(clear_cookie(&s.cfg)),
        ),
    }
}

async fn logout(State(s): State<WebAuthState>, headers: HeaderMap) -> Response {
    if !csrf_ok(&headers) {
        return error_response(StatusCode::FORBIDDEN, "missing csrf header", None);
    }
    // Best-effort revoke: clearing the cookie is what actually ends the browser
    // session, so a revoke failure must not fail logout.
    if let Some(token) = refresh_cookie(&headers) {
        let _ = s.auth.logout(&token).await;
    }
    let mut resp = StatusCode::NO_CONTENT.into_response();
    if let Ok(v) = HeaderValue::from_str(&clear_cookie(&s.cfg)) {
        resp.headers_mut().insert(SET_COOKIE, v);
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use cymbra_auth::{FakeSessionStore, SessionStore};
    use cymbra_auth_port::TokenPair;
    use cymbra_platform::Result;
    use tower::ServiceExt;

    // Not a secret — a canned password for the auth double below.
    const PW: &str = "correct-horse-battery-staple";
    const ORIGIN: &str = "https://bo.cymbra.app";

    fn cfg() -> WebAuthConfig {
        WebAuthConfig {
            cookie_domain: Some("cymbra.app".into()),
            cookie_secure: true,
            cookie_path: WebAuthConfig::DEFAULT_PATH.into(),
            refresh_ttl: Duration::from_secs(2_592_000),
            allowed_origins: vec![ORIGIN.into()],
        }
    }

    /// Minimal [`AuthPort`] test double: real rotation + reuse detection via
    /// [`FakeSessionStore`], with a canned access token (these handler tests exercise
    /// cookie handling, not JWT contents). Avoids wiring the whole auth module — and
    /// therefore any signing key — into a transport-level test.
    struct FakeAuth {
        sessions: FakeSessionStore,
        email: String,
        password: String,
    }

    #[async_trait]
    impl AuthPort for FakeAuth {
        async fn sign_in_local(
            &self,
            email: &str,
            password: &str,
            audience: &str,
        ) -> Result<TokenPair> {
            if email != self.email || password != self.password {
                return Err(AppError::Unauthenticated("invalid credentials".into()));
            }
            let refresh_token = self.sessions.create(email, audience).await?;
            Ok(TokenPair {
                access_token: "test.access.token".into(),
                refresh_token,
            })
        }
        async fn sign_in_oidc(&self, _id_token: &str, audience: &str) -> Result<TokenPair> {
            let refresh_token = self.sessions.create(&self.email, audience).await?;
            Ok(TokenPair {
                access_token: "test.access.token".into(),
                refresh_token,
            })
        }
        async fn refresh(&self, refresh_token: &str) -> Result<TokenPair> {
            let rotated = self.sessions.rotate(refresh_token).await?;
            Ok(TokenPair {
                access_token: "test.access.token".into(),
                refresh_token: rotated.refresh_token,
            })
        }
        async fn logout(&self, refresh_token: &str) -> Result<()> {
            self.sessions.revoke(refresh_token).await
        }
        async fn list_sessions(
            &self,
            user_id: &str,
        ) -> Result<Vec<cymbra_auth_port::SessionSummary>> {
            Ok(self
                .sessions
                .list_for_user(user_id)
                .await?
                .into_iter()
                .map(|s| cymbra_auth_port::SessionSummary {
                    id: s.id,
                    audience: s.audience,
                })
                .collect())
        }
        async fn revoke_session(&self, user_id: &str, session_id: &str) -> Result<()> {
            self.sessions.revoke_by_id(user_id, session_id).await
        }
        async fn revoke_all_sessions(&self, user_id: &str) -> Result<()> {
            self.sessions.revoke_all(user_id).await
        }
        // The web-auth surface never calls the methods below.
        async fn sign_up_local(&self, _email: &str, _password: &str) -> Result<()> {
            unreachable!()
        }
        async fn verify_email(&self, _token: &str) -> Result<()> {
            unreachable!()
        }
        async fn resend_verification(&self, _email: &str) -> Result<()> {
            unreachable!()
        }
        async fn request_password_reset(&self, _email: &str) -> Result<()> {
            unreachable!()
        }
        async fn reset_password(&self, _token: &str, _new_password: &str) -> Result<()> {
            unreachable!()
        }
        async fn link_identity(&self, _user_id: &str, _id_token: &str) -> Result<()> {
            unreachable!()
        }
        async fn unlink_identity(
            &self,
            _user_id: &str,
            _provider: &str,
            _subject: &str,
        ) -> Result<()> {
            unreachable!()
        }
    }

    /// A signed-up, verified user backed by the fake session store.
    fn auth_with_verified_user(email: &str) -> Arc<dyn AuthPort> {
        Arc::new(FakeAuth {
            sessions: FakeSessionStore::default(),
            email: email.into(),
            password: PW.into(),
        })
    }

    fn router(auth: Arc<dyn AuthPort>) -> Router {
        web_auth_router(auth, cfg())
    }

    async fn body_string(resp: Response) -> String {
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    fn signin_req(json: &str, csrf: bool) -> Request<Body> {
        let mut b = Request::builder()
            .method(Method::POST)
            .uri("/web/auth/signin")
            .header(CONTENT_TYPE, "application/json");
        if csrf {
            b = b.header(CSRF_HEADER, "1");
        }
        b.body(Body::from(json.to_string())).unwrap()
    }

    // Pull the `cymbra_refresh` cookie value out of a Set-Cookie header.
    fn cookie_from(resp: &Response) -> Option<String> {
        let sc = resp.headers().get(SET_COOKIE)?.to_str().ok()?;
        cookie_value(sc, COOKIE_NAME).map(str::to_string)
    }

    #[tokio::test]
    async fn signin_sets_httponly_cookie_and_returns_access_no_refresh() {
        let auth = auth_with_verified_user("a@x.dev");
        let json =
            format!(r#"{{"kind":"local","email":"a@x.dev","password":"{PW}","audience":"music"}}"#);
        let resp = router(auth).oneshot(signin_req(&json, true)).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let set = resp
            .headers()
            .get(SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(set.contains("cymbra_refresh="));
        assert!(set.contains("HttpOnly"));
        assert!(set.contains("SameSite=Strict"));
        assert!(set.contains("Secure"));
        assert!(set.contains("Path=/web/auth"));
        assert!(set.contains("Domain=cymbra.app"));

        // Body carries the access token; the refresh token is nowhere in it.
        let refresh = cookie_value(&set, COOKIE_NAME).unwrap().to_string();
        let body = body_string(resp).await;
        assert!(body.contains("accessToken"));
        assert!(!body.contains("refresh"));
        assert!(!body.contains(&refresh));
    }

    #[tokio::test]
    async fn signin_bad_credentials_401_and_no_cookie() {
        let auth = auth_with_verified_user("a@x.dev");
        let json = r#"{"kind":"local","email":"a@x.dev","password":"wrong-passphrase","audience":"music"}"#;
        let resp = router(auth).oneshot(signin_req(json, true)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert!(resp.headers().get(SET_COOKIE).is_none());
    }

    #[tokio::test]
    async fn signin_without_csrf_header_is_forbidden() {
        let auth = auth_with_verified_user("a@x.dev");
        let json =
            format!(r#"{{"kind":"local","email":"a@x.dev","password":"{PW}","audience":"music"}}"#);
        let resp = router(auth)
            .oneshot(signin_req(&json, false))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert!(resp.headers().get(SET_COOKIE).is_none());
    }

    #[tokio::test]
    async fn refresh_reads_cookie_rotates_and_resets() {
        let auth = auth_with_verified_user("a@x.dev");
        let app = router(auth);
        let json =
            format!(r#"{{"kind":"local","email":"a@x.dev","password":"{PW}","audience":"music"}}"#);
        let signed = app.clone().oneshot(signin_req(&json, true)).await.unwrap();
        let cookie = cookie_from(&signed).unwrap();

        let req = Request::builder()
            .method(Method::POST)
            .uri("/web/auth/refresh")
            .header(CSRF_HEADER, "1")
            .header(COOKIE, format!("{COOKIE_NAME}={cookie}"))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let rotated = cookie_from(&resp).unwrap();
        assert_ne!(rotated, cookie, "refresh must rotate the cookie");
        assert!(body_string(resp).await.contains("accessToken"));
    }

    #[tokio::test]
    async fn refresh_without_cookie_401_and_clears() {
        let auth = auth_with_verified_user("a@x.dev");
        let req = Request::builder()
            .method(Method::POST)
            .uri("/web/auth/refresh")
            .header(CSRF_HEADER, "1")
            .body(Body::empty())
            .unwrap();
        let resp = router(auth).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let set = resp.headers().get(SET_COOKIE).unwrap().to_str().unwrap();
        assert!(set.contains("Max-Age=0"));
    }

    #[tokio::test]
    async fn refresh_with_reused_cookie_401_and_clears() {
        let auth = auth_with_verified_user("a@x.dev");
        let app = router(auth);
        let json =
            format!(r#"{{"kind":"local","email":"a@x.dev","password":"{PW}","audience":"music"}}"#);
        let signed = app.clone().oneshot(signin_req(&json, true)).await.unwrap();
        let cookie = cookie_from(&signed).unwrap();
        let refresh_req = || {
            Request::builder()
                .method(Method::POST)
                .uri("/web/auth/refresh")
                .header(CSRF_HEADER, "1")
                .header(COOKIE, format!("{COOKIE_NAME}={cookie}"))
                .body(Body::empty())
                .unwrap()
        };
        // First rotation succeeds; replaying the now-rotated cookie is reuse → 401.
        assert_eq!(
            app.clone().oneshot(refresh_req()).await.unwrap().status(),
            StatusCode::OK
        );
        let resp = app.oneshot(refresh_req()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert!(
            resp.headers()
                .get(SET_COOKIE)
                .unwrap()
                .to_str()
                .unwrap()
                .contains("Max-Age=0")
        );
    }

    #[tokio::test]
    async fn logout_clears_cookie_and_revokes_session() {
        let auth = auth_with_verified_user("a@x.dev");
        let app = router(auth);
        let json =
            format!(r#"{{"kind":"local","email":"a@x.dev","password":"{PW}","audience":"music"}}"#);
        let signed = app.clone().oneshot(signin_req(&json, true)).await.unwrap();
        let cookie = cookie_from(&signed).unwrap();

        let logout_req = Request::builder()
            .method(Method::POST)
            .uri("/web/auth/logout")
            .header(CSRF_HEADER, "1")
            .header(COOKIE, format!("{COOKIE_NAME}={cookie}"))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(logout_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert!(
            resp.headers()
                .get(SET_COOKIE)
                .unwrap()
                .to_str()
                .unwrap()
                .contains("Max-Age=0")
        );

        // The revoked session can no longer refresh.
        let after = Request::builder()
            .method(Method::POST)
            .uri("/web/auth/refresh")
            .header(CSRF_HEADER, "1")
            .header(COOKIE, format!("{COOKIE_NAME}={cookie}"))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.oneshot(after).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn credentialed_cors_echoes_exact_origin_never_wildcard() {
        let auth = auth_with_verified_user("a@x.dev");
        // A CORS preflight for the refresh endpoint from an allowed origin.
        let preflight = Request::builder()
            .method(Method::OPTIONS)
            .uri("/web/auth/refresh")
            .header("origin", ORIGIN)
            .header("access-control-request-method", "POST")
            .header("access-control-request-headers", CSRF_HEADER)
            .body(Body::empty())
            .unwrap();
        let resp = router(auth).oneshot(preflight).await.unwrap();
        let acao = resp
            .headers()
            .get("access-control-allow-origin")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(acao, ORIGIN);
        assert_ne!(acao, "*");
        assert_eq!(
            resp.headers()
                .get("access-control-allow-credentials")
                .unwrap(),
            "true"
        );
    }

    #[test]
    fn cookie_value_parses_from_a_multi_cookie_header() {
        assert_eq!(
            cookie_value("other=1; cymbra_refresh=abc123; x=y", COOKIE_NAME),
            Some("abc123")
        );
        assert_eq!(cookie_value("nope=1", COOKIE_NAME), None);
    }

    #[test]
    fn clear_cookie_expires_with_matching_scope() {
        let c = clear_cookie(&cfg());
        assert!(c.contains("Max-Age=0"));
        assert!(c.contains("Path=/web/auth"));
        assert!(c.contains("Domain=cymbra.app"));
        assert!(c.contains("HttpOnly"));
    }

    #[test]
    fn insecure_dev_cookie_omits_secure_and_domain() {
        let dev = WebAuthConfig {
            cookie_domain: None,
            cookie_secure: false,
            cookie_path: WebAuthConfig::DEFAULT_PATH.into(),
            refresh_ttl: Duration::from_secs(60),
            allowed_origins: vec![],
        };
        let c = set_cookie(&dev, "tok");
        assert!(!c.contains("Secure"));
        assert!(!c.contains("Domain="));
        assert!(c.contains("Max-Age=60"));
    }
}
