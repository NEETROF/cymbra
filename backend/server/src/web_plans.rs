//! Browser-only **plan JSON routes** for the public site (change:
//! add-site-account-pages, spec `music-plan-web-api`).
//!
//! Four bearer-authenticated endpoints, each a one-line delegation to the shared
//! `cymbra_plans::web` operations (the same rules as the gRPC `PlanService`):
//!
//! - `GET  /web/plans/me`        — the caller's plan for platform `web`
//! - `POST /web/plans/redeem`    — `{ code }` → campaign (web-only, throttled)
//! - `POST /web/plans/checkout`  — `{ product_id }` → `{ checkout_url }`
//! - `GET  /web/plans/portal`    — `{ portal_url }` for an active web subscription
//!
//! The bearer is the short-lived access token the web sign-in hands the browser
//! (`/web/auth/*`); it is verified through the same seam as the SoundFont routes.
//! A missing/invalid token is `401` before any read. Errors are `{ "error": ... }`
//! with the RPC's neutral wording. CORS allows only the configured web origins.

use std::sync::Arc;

use axum::extract::State;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use cymbra_plans::web;
use cymbra_plans::{PaywallConfigSource, PlanService, Platform, WebBillingProvider};
use cymbra_platform::AppError;
use cymbra_platform::cache::Cache;
use cymbra_platform::identity::AuthIdentity;
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::soundfont::SoundfontAuth;
use crate::web_auth::{http_status, safe_message};

/// Router state: the plan service + paywall config + throttle cache + the
/// (optional) web billing provider, and the bearer auth seam.
#[derive(Clone)]
pub struct WebPlansState {
    pub svc: Arc<PlanService>,
    pub paywall: Arc<dyn PaywallConfigSource>,
    pub cache: Arc<dyn Cache>,
    pub web: Option<Arc<dyn WebBillingProvider>>,
    pub auth: Arc<dyn SoundfontAuth>,
}

#[derive(Deserialize)]
struct RedeemBody {
    code: String,
}

#[derive(Deserialize)]
struct CheckoutBody {
    product_id: String,
}

#[derive(Serialize)]
struct CheckoutUrl {
    checkout_url: String,
}

#[derive(Serialize)]
struct PortalUrl {
    portal_url: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

/// Build the router (CORS-wrapped), ready to `.merge()` into the HTTP server.
/// `origins` are the exact browser origins allowed (`CYMBRA_WEB_ORIGINS`).
pub fn web_plans_router(state: WebPlansState, origins: Vec<String>) -> Router {
    let origins: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);
    Router::new()
        .route("/web/plans/me", get(me))
        .route("/web/plans/redeem", post(redeem))
        .route("/web/plans/checkout", post(checkout))
        .route("/web/plans/portal", get(portal))
        .layer(cors)
        .with_state(state)
}

/// The caller's identity from the bearer, or `None` (→ `401`).
fn caller(state: &WebPlansState, headers: &HeaderMap) -> Option<AuthIdentity> {
    state.auth.identify_admin(headers)
}

fn unauthenticated() -> Response {
    error_response(&AppError::Unauthenticated("sign in required".into()))
}

/// Best-effort client address for the per-address throttle: the first
/// `X-Forwarded-For` hop (set by the reverse proxy), else `X-Real-IP`, else
/// `unknown` — the per-account limit is the primary guard.
fn client_addr(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|s| !s.is_empty())
        })
        .unwrap_or("unknown")
        .to_string()
}

fn error_response(e: &AppError) -> Response {
    (
        http_status(e),
        Json(ErrorBody {
            error: safe_message(e),
        }),
    )
        .into_response()
}

fn ok<T: Serialize>(body: T) -> Response {
    (StatusCode::OK, Json(body)).into_response()
}

async fn me(State(s): State<WebPlansState>, headers: HeaderMap) -> Response {
    let Some(id) = caller(&s, &headers) else {
        return unauthenticated();
    };
    match web::my_plan(&s.svc, s.paywall.as_ref(), &id.user_id, Some(Platform::Web)).await {
        Ok(v) => ok(v),
        Err(e) => error_response(&e),
    }
}

async fn redeem(
    State(s): State<WebPlansState>,
    headers: HeaderMap,
    body: Json<RedeemBody>,
) -> Response {
    let Some(id) = caller(&s, &headers) else {
        return unauthenticated();
    };
    let addr = client_addr(&headers);
    match web::redeem(
        &s.svc,
        s.cache.as_ref(),
        &id.user_id,
        &id.audience,
        &addr,
        &body.0.code,
    )
    .await
    {
        Ok(v) => ok(v),
        Err(e) => error_response(&e),
    }
}

async fn checkout(
    State(s): State<WebPlansState>,
    headers: HeaderMap,
    body: Json<CheckoutBody>,
) -> Response {
    let Some(id) = caller(&s, &headers) else {
        return unauthenticated();
    };
    match web::create_checkout(
        &s.svc,
        s.paywall.as_ref(),
        s.web.as_ref(),
        &id.user_id,
        &body.0.product_id,
    )
    .await
    {
        Ok(checkout_url) => ok(CheckoutUrl { checkout_url }),
        Err(e) => error_response(&e),
    }
}

async fn portal(State(s): State<WebPlansState>, headers: HeaderMap) -> Response {
    let Some(id) = caller(&s, &headers) else {
        return unauthenticated();
    };
    match web::portal_url(&s.svc, s.web.as_ref(), &id.user_id).await {
        Ok(portal_url) => ok(PortalUrl { portal_url }),
        Err(e) => error_response(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use chrono::{DateTime, TimeZone, Utc};
    use cymbra_plans::model::{EntitlementRow, EntitlementStatus, Source};
    use cymbra_plans::ports::{
        FixedPaywallConfig, MockAccessCodeRepo, MockAuditRepo, MockBillingEventRepo,
        MockCampaignRepo, MockClock, MockEntitlementRepo, MockMembershipRepo, MockPlanConfigSource,
        MockWebBillingProvider,
    };
    use cymbra_plans::{PlanConfig, PlanDeps};
    use cymbra_platform::cache::FakeCache;
    use std::collections::BTreeMap;
    use tower::ServiceExt;
    use uuid::Uuid;

    const ORIGIN: &str = "https://cymbra.app";

    fn t(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, day, 12, 0, 0).unwrap()
    }

    fn row(source: Source) -> EntitlementRow {
        EntitlementRow {
            id: Uuid::new_v4(),
            user_id: "u1".into(),
            source,
            provider_ref: format!("{}-ref", source.as_str()),
            campaign_id: None,
            starts_at: t(1),
            ends_at: Some(t(30)),
            status: EntitlementStatus::Active,
            revoked_at: None,
            withdrawn_at: None,
        }
    }

    fn service(rows: Vec<EntitlementRow>) -> Arc<PlanService> {
        let mut config = MockPlanConfigSource::new();
        config.expect_plan_config().returning(|| PlanConfig {
            enabled: true,
            grace_days: 3,
        });
        let mut clock = MockClock::new();
        clock.expect_now().returning(|| t(5));
        let mut audit = MockAuditRepo::new();
        audit.expect_record().returning(|_| Ok(()));
        let mut entitlements = MockEntitlementRepo::new();
        entitlements
            .expect_list_for_user()
            .returning(move |_| Ok(rows.clone()));
        let mut memberships = MockMembershipRepo::new();
        memberships
            .expect_list_for_user()
            .returning(|_| Ok(Vec::new()));
        let mut codes = MockAccessCodeRepo::new();
        codes.expect_find_by_hash().returning(|_| Ok(None));
        Arc::new(PlanService::new(PlanDeps {
            entitlements: Arc::new(entitlements),
            campaigns: Arc::new(MockCampaignRepo::new()),
            memberships: Arc::new(memberships),
            codes: Arc::new(codes),
            billing_events: Arc::new(MockBillingEventRepo::new()),
            audit: Arc::new(audit),
            config: Arc::new(config),
            clock: Arc::new(clock),
            rotator: None,
        }))
    }

    /// Auth double: `Bearer web` → a `web`-audience user, `Bearer music` → the
    /// store-build audience, anything else → unauthenticated.
    struct FixedAuth;

    impl SoundfontAuth for FixedAuth {
        fn identify(&self, headers: &HeaderMap) -> Option<String> {
            self.identify_admin(headers).map(|i| i.user_id)
        }
        fn identify_admin(&self, headers: &HeaderMap) -> Option<AuthIdentity> {
            let aud = headers
                .get(AUTHORIZATION)?
                .to_str()
                .ok()?
                .strip_prefix("Bearer ")?;
            if aud != "web" && aud != "music" {
                return None;
            }
            Some(AuthIdentity {
                user_id: "u1".into(),
                audience: aud.into(),
                roles: vec![],
                roles_by_scope: BTreeMap::new(),
            })
        }
    }

    fn app(rows: Vec<EntitlementRow>, web_enabled: bool, with_web: bool) -> Router {
        let mut web = MockWebBillingProvider::new();
        web.expect_create_checkout()
            .returning(|_, _| Ok("https://cymbra.app/checkout?_ptxn=txn_1".into()));
        web.expect_portal_url()
            .returning(|_| Ok("https://portal.example/s".into()));
        let state = WebPlansState {
            svc: service(rows),
            paywall: Arc::new(FixedPaywallConfig {
                apple: true,
                google: true,
                web: web_enabled,
                products: vec!["premium_monthly".into()],
            }),
            cache: Arc::new(FakeCache::default()),
            web: with_web.then(|| Arc::new(web) as Arc<dyn WebBillingProvider>),
            auth: Arc::new(FixedAuth),
        };
        web_plans_router(state, vec![ORIGIN.into()])
    }

    fn req(method: Method, uri: &str, bearer: Option<&str>, json: Option<&str>) -> Request<Body> {
        let mut b = Request::builder().method(method).uri(uri);
        if let Some(t) = bearer {
            b = b.header(AUTHORIZATION, format!("Bearer {t}"));
        }
        match json {
            Some(j) => b
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(j.to_string()))
                .unwrap(),
            None => b.body(Body::empty()).unwrap(),
        }
    }

    async fn json_of(resp: Response) -> serde_json::Value {
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn missing_bearer_is_401_before_any_read() {
        let resp = app(vec![], true, true)
            .oneshot(req(Method::GET, "/web/plans/me", None, None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert!(json_of(resp).await.get("error").is_some());
        let resp = app(vec![], true, true)
            .oneshot(req(Method::GET, "/web/plans/me", Some("nope"), None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn me_mirrors_the_rpc_for_the_web_platform() {
        let resp = app(vec![row(Source::Web)], true, true)
            .oneshot(req(Method::GET, "/web/plans/me", Some("web"), None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let v = json_of(resp).await;
        assert_eq!(v["plan"], "premium");
        assert_eq!(v["managed_on"], "web");
        assert_eq!(v["can_purchase_here"], false);
        assert!(!v["unlocks"].as_array().unwrap().is_empty());

        // Free on the web platform with the channel open → purchasable here.
        let resp = app(vec![], true, true)
            .oneshot(req(Method::GET, "/web/plans/me", Some("web"), None))
            .await
            .unwrap();
        let v = json_of(resp).await;
        assert_eq!(v["plan"], "free");
        assert_eq!(v["can_purchase_here"], true);
        assert_eq!(v["purchase_channel"], "web");
        assert_eq!(v["products"][0], "premium_monthly");
    }

    #[tokio::test]
    async fn redeem_neutral_refusal_music_audience_refused_and_throttled() {
        let app = app(vec![], true, true);
        let body = r#"{"code":"ABCD-EFGH"}"#;
        // Unknown code → neutral 4xx with an error message, no crash.
        let resp = app
            .clone()
            .oneshot(req(
                Method::POST,
                "/web/plans/redeem",
                Some("web"),
                Some(body),
            ))
            .await
            .unwrap();
        assert!(resp.status().is_client_error());
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let msg = json_of(resp).await["error"].as_str().unwrap().to_string();
        assert!(!msg.is_empty());

        // Store-build audience → refused (precondition), nothing consumed.
        let resp = app
            .clone()
            .oneshot(req(
                Method::POST,
                "/web/plans/redeem",
                Some("music"),
                Some(body),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);

        // Past the per-account limit → 429 (same neutral shape).
        for _ in 0..web::REDEEM_MAX_PER_USER {
            let _ = app
                .clone()
                .oneshot(req(
                    Method::POST,
                    "/web/plans/redeem",
                    Some("web"),
                    Some(body),
                ))
                .await
                .unwrap();
        }
        let resp = app
            .oneshot(req(
                Method::POST,
                "/web/plans/redeem",
                Some("web"),
                Some(body),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn checkout_gated_then_returns_the_url() {
        let body = r#"{"product_id":"premium_monthly"}"#;
        // Channel disabled.
        let resp = app(vec![], false, true)
            .oneshot(req(
                Method::POST,
                "/web/plans/checkout",
                Some("web"),
                Some(body),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);
        // Subscribed elsewhere.
        let resp = app(vec![row(Source::Apple)], true, true)
            .oneshot(req(
                Method::POST,
                "/web/plans/checkout",
                Some("web"),
                Some(body),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);
        assert!(
            json_of(resp).await["error"]
                .as_str()
                .unwrap()
                .contains("another channel")
        );
        // Not configured → 500-class generic message (no provider detail).
        let resp = app(vec![], true, false)
            .oneshot(req(
                Method::POST,
                "/web/plans/checkout",
                Some("web"),
                Some(body),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json_of(resp).await["error"], "internal error");
        // Happy path.
        let resp = app(vec![], true, true)
            .oneshot(req(
                Method::POST,
                "/web/plans/checkout",
                Some("web"),
                Some(body),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(
            json_of(resp).await["checkout_url"]
                .as_str()
                .unwrap()
                .contains("_ptxn=")
        );
    }

    #[tokio::test]
    async fn portal_only_for_web_rows() {
        let resp = app(vec![row(Source::Web)], true, true)
            .oneshot(req(Method::GET, "/web/plans/portal", Some("web"), None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            json_of(resp).await["portal_url"],
            "https://portal.example/s"
        );

        let resp = app(vec![row(Source::Apple)], true, true)
            .oneshot(req(Method::GET, "/web/plans/portal", Some("web"), None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);
    }

    #[tokio::test]
    async fn cors_allows_the_site_origin_only() {
        let preflight = |origin: &str| {
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/web/plans/me")
                .header("origin", origin)
                .header("access-control-request-method", "GET")
                .header("access-control-request-headers", "authorization")
                .body(Body::empty())
                .unwrap()
        };
        let resp = app(vec![], true, true)
            .oneshot(preflight(ORIGIN))
            .await
            .unwrap();
        assert_eq!(
            resp.headers().get("access-control-allow-origin").unwrap(),
            ORIGIN
        );
        let resp = app(vec![], true, true)
            .oneshot(preflight("https://evil.example"))
            .await
            .unwrap();
        assert!(resp.headers().get("access-control-allow-origin").is_none());
    }

    #[test]
    fn client_addr_prefers_the_first_forwarded_hop() {
        let mut h = HeaderMap::new();
        assert_eq!(client_addr(&h), "unknown");
        h.insert("x-real-ip", "10.0.0.9".parse().unwrap());
        assert_eq!(client_addr(&h), "10.0.0.9");
        h.insert("x-forwarded-for", "203.0.113.5, 10.0.0.1".parse().unwrap());
        assert_eq!(client_addr(&h), "203.0.113.5");
    }
}
