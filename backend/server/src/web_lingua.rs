//! The Lingua leaving page's one route (change: add-lingua-usage-counts, spec
//! `lingua-usage-counts`, D12): `POST /web/lingua/leave` with `{ reason, browser, version }`
//! — as JSON from the page's script, or as a plain HTML form when the page runs without
//! JavaScript.
//!
//! No authentication and no cookie: the answer is counted, the same anonymous way as the
//! count reports, under the UTC week it arrived in — through the same module, so the same
//! flag, flood guard, ceiling and version bound apply. The handler reads the body and the
//! forwarded address (for the flood guard alone) and nothing else; it logs nothing about the
//! request. CORS allows only the site's origins (`CYMBRA_WEB_ORIGINS`), without credentials.
//! Mounted only when the Lingua module is wired.
//!
//! Answers to JSON: `200 {}` whether counted or not, `400` for anything but one of the six
//! answers (no free text), `429` past the flood limit, `503` when the counter could not be
//! written. The flood guard counts every request first, a malformed one included.
//!
//! Answers to the form: `303` back to the page's thanks (`#merci`, `#thanks`, `#gracias`, in
//! the form's `lang`, on the site's origin — the first of `CYMBRA_WEB_ORIGINS`, never an
//! address the request names), whatever happened to the answer, as the script thanks the
//! reader whatever the backend says; `400` only for an answer that is not one of the six.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::header::{CONTENT_TYPE, LOCATION};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use cymbra_lingua::usage_counts::{Leave, UsageCountsModule};
use cymbra_lingua::usage_counts_core::UNKNOWN;
use cymbra_platform::AppError;
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::web_auth::{http_status, safe_message};

/// The largest body accepted: one answer is a few dozen bytes.
const MAX_BODY: usize = 1024;

fn unknown() -> String {
    UNKNOWN.to_string()
}

#[derive(Deserialize)]
struct LeaveBody {
    reason: String,
    #[serde(default = "unknown")]
    browser: String,
    #[serde(default = "unknown")]
    version: String,
    /// The form's page language, for the thanks it is sent back to; never counted.
    #[serde(default)]
    lang: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Serialize)]
struct Empty {}

#[derive(Clone)]
struct LeaveState {
    module: Arc<UsageCountsModule>,
    /// The site's origin, where a form is sent back to (the first allowed web origin).
    site: Option<String>,
}

/// The leaving page's thanks in each site language: the page itself, at the block its CSS
/// shows when targeted (`apps/site/src/pages/**/lingua/*`). A fixed table: no address from
/// the request is ever redirected to.
fn thanks_path(lang: &str) -> &'static str {
    match lang {
        "en" => "/en/lingua/goodbye/#thanks",
        "es" => "/es/lingua/adios/#gracias",
        _ => "/lingua/au-revoir/#merci",
    }
}

/// Build the router (CORS-wrapped), ready to `.merge()` into the HTTP server. `origins` are
/// the exact browser origins allowed (`CYMBRA_WEB_ORIGINS`), the site's first.
pub fn web_lingua_router(module: Arc<UsageCountsModule>, origins: Vec<String>) -> Router {
    let site = origins.first().map(|o| o.trim_end_matches('/').to_string());
    let origins: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE]);
    Router::new()
        .route("/web/lingua/leave", post(leave))
        .layer(DefaultBodyLimit::max(MAX_BODY))
        .layer(cors)
        .with_state(LeaveState { module, site })
}

/// The address the flood guard keys, resolved like every rate limit (Caddy's first
/// `X-Forwarded-For` hop, else `X-Real-IP`, else `unknown`). Passed to the guard alone.
fn client_addr(headers: &HeaderMap) -> String {
    cymbra_platform::client_addr::resolve(
        |name| {
            headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        },
        None,
    )
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

/// Whether the body is a plain HTML form's (the page without JavaScript).
fn is_form(headers: &HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/x-www-form-urlencoded"))
}

/// A form sent back to the page's thanks on the site; with no site configured, a plain
/// "thank you" in the three languages.
fn thanks(site: Option<&str>, lang: &str) -> Response {
    match site.and_then(|s| HeaderValue::from_str(&format!("{s}{}", thanks_path(lang))).ok()) {
        Some(location) => (StatusCode::SEE_OTHER, [(LOCATION, location)]).into_response(),
        None => (StatusCode::OK, "Merci. Thank you. Gracias.").into_response(),
    }
}

async fn leave(State(state): State<LeaveState>, headers: HeaderMap, body: Bytes) -> Response {
    let addr = client_addr(&headers);
    let form = is_form(&headers);
    let parsed: Option<LeaveBody> = if form {
        serde_urlencoded::from_bytes(&body).ok()
    } else {
        serde_json::from_slice(&body).ok()
    };
    // The guard counts a body it cannot read too (the module's `refuse_malformed`).
    let Some(body) = parsed else {
        return error_response(&state.module.refuse_malformed(&addr));
    };
    let lang = body.lang;
    let outcome = state
        .module
        .leave(
            &addr,
            &Leave {
                reason: body.reason,
                browser: body.browser,
                version: body.version,
            },
        )
        .await;
    match outcome {
        Err(e @ AppError::InvalidArgument(_)) => error_response(&e),
        _ if form => thanks(state.site.as_deref(), &lang),
        Ok(()) => (StatusCode::OK, Json(Empty {})).into_response(),
        Err(e) => error_response(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use cymbra_lingua::proto::ReportRequest;
    use cymbra_lingua::proto::usage_counts_service_server::UsageCountsService;
    use cymbra_lingua::usage_counts::{
        FloodGuard, MockClock, MockUsageCountsPolicy, MockUsageCountsRepo,
    };
    use cymbra_lingua::usage_counts_core::monday_of;
    use cymbra_lingua::usage_counts_grpc::UsageCountsGrpc;
    use std::sync::Mutex;
    use tower::ServiceExt;

    const SITE: &str = "https://cymbra.app";
    /// 2026-10-21 (a Wednesday), noon UTC.
    const NOW: u64 = 20_747 * 86_400 + 43_200;

    fn module(repo: MockUsageCountsRepo, enabled: bool, limit: u32) -> Arc<UsageCountsModule> {
        let mut policy = MockUsageCountsPolicy::new();
        policy.expect_enabled().return_const(enabled);
        policy.expect_ceiling().return_const(5_000i64);
        policy
            .expect_latest_version()
            .return_const("1.10".to_string());
        let mut clock = MockClock::new();
        clock.expect_now_secs().return_const(NOW);
        Arc::new(UsageCountsModule::new(
            Arc::new(repo),
            Arc::new(policy),
            Arc::new(clock),
            FloodGuard::with_key([5; 32], limit, 1_000),
        ))
    }

    fn counting(times: usize) -> MockUsageCountsRepo {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_leave()
            .withf(|week, reason, browser, version, _| {
                *week == monday_of(20_747)
                    && reason == "slow"
                    && browser == "firefox"
                    && version == "1.10"
            })
            .times(times)
            .returning(|_, _, _, _, _| Ok(()));
        repo
    }

    fn post_leave(body: &str, origin: &str) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri("/web/lingua/leave")
            .header(CONTENT_TYPE, "application/json")
            .header("origin", origin)
            .header("x-forwarded-for", "203.0.113.5")
            .header(
                "user-agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Firefox/140.0",
            )
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    const SLOW: &str = r#"{"reason":"slow","browser":"firefox","version":"1.10"}"#;

    #[tokio::test]
    async fn a_reason_is_counted_and_answered_empty() {
        let app = web_lingua_router(module(counting(1), true, 60), vec![SITE.into()]);
        let resp = app.oneshot(post_leave(SLOW, SITE)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get("access-control-allow-origin").unwrap(),
            SITE
        );
        assert!(
            resp.headers()
                .get("access-control-allow-credentials")
                .is_none()
        );
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"{}");
    }

    #[tokio::test]
    async fn absent_parameters_are_counted_as_unknown() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_leave()
            .withf(|_, reason, browser, version, _| {
                reason == "other" && browser == "unknown" && version == "unknown"
            })
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));
        let app = web_lingua_router(module(repo, true, 60), vec![SITE.into()]);
        let resp = app
            .oneshot(post_leave(r#"{"reason":"other"}"#, SITE))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// The review's nit: malformed JSON was refused before the flood guard saw it, so
    /// garbage could be sent without limit. Now it spends the network's budget.
    #[tokio::test]
    async fn malformed_bodies_go_through_the_flood_guard_first() {
        let app = web_lingua_router(
            module(MockUsageCountsRepo::new(), true, 2),
            vec![SITE.into()],
        );
        for _ in 0..2 {
            let resp = app
                .clone()
                .oneshot(post_leave("not json", SITE))
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }
        let resp = app.oneshot(post_leave("not json", SITE)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    fn post_form(body: &str) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri("/web/lingua/leave")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header("origin", SITE)
            .header("x-forwarded-for", "203.0.113.5")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn location(resp: &Response) -> &str {
        resp.headers().get(LOCATION).unwrap().to_str().unwrap()
    }

    /// The page without JavaScript (review item 9): its buttons are a plain form, counted
    /// like the script's answer and sent back to the page's thanks in its language.
    #[tokio::test]
    async fn a_plain_form_is_counted_and_sent_back_to_the_thanks() {
        let app = web_lingua_router(module(counting(1), true, 60), vec![SITE.into()]);
        let resp = app
            .oneshot(post_form(
                "reason=slow&browser=firefox&version=1.10&lang=en",
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            location(&resp),
            "https://cymbra.app/en/lingua/goodbye/#thanks"
        );
    }

    #[tokio::test]
    async fn a_form_without_parameters_is_counted_as_unknown_in_its_language() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_leave()
            .withf(|_, reason, browser, version, _| {
                reason == "unclear" && browser == "unknown" && version == "unknown"
            })
            .times(3)
            .returning(|_, _, _, _, _| Ok(()));
        let app = web_lingua_router(
            module(repo, true, 60),
            vec![format!("{SITE}/"), "https://bo.cymbra.app".into()],
        );
        for (lang, thanks) in [
            ("es", "https://cymbra.app/es/lingua/adios/#gracias"),
            ("fr", "https://cymbra.app/lingua/au-revoir/#merci"),
            // A language the site does not have: French, never an address from the request.
            (
                "https://evil.example",
                "https://cymbra.app/lingua/au-revoir/#merci",
            ),
        ] {
            let body =
                serde_urlencoded::to_string([("reason", "unclear"), ("lang", lang)]).unwrap();
            let resp = app.clone().oneshot(post_form(&body)).await.unwrap();
            assert_eq!(resp.status(), StatusCode::SEE_OTHER);
            assert_eq!(location(&resp), thanks);
        }
    }

    /// The tests that make the module log "counter not written" run one at a time: `tracing`
    /// caches a call site's interest when it is first hit, and a first hit on another thread
    /// while `nothing_identifying_reaches_the_logs` installs its capture can leave that call
    /// site silent for the capture (seen under `cargo llvm-cov`), which would make the
    /// capture test fail — or, worse, pass on an empty log.
    static LOGGING_TESTS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[tokio::test]
    async fn a_form_is_thanked_whatever_happened_to_the_answer_but_free_text() {
        let _one_at_a_time = LOGGING_TESTS.lock().await;
        // Past the flood limit, or with the counter down, the reader is still thanked —
        // as the script thanks them; only an answer outside the six is refused.
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_leave()
            .times(1)
            .returning(|_, _, _, _, _| Err(AppError::Internal(anyhow::anyhow!("db down"))));
        let app = web_lingua_router(module(repo, true, 1), vec![SITE.into()]);
        for _ in 0..2 {
            let resp = app
                .clone()
                .oneshot(post_form("reason=slow&lang=fr"))
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        }
        let app = web_lingua_router(
            module(MockUsageCountsRepo::new(), true, 60),
            vec![SITE.into()],
        );
        let resp = app
            .clone()
            .oneshot(post_form("reason=call+me+at+0612345678"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let resp = app.oneshot(post_form("%%%")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        // With no site configured, a plain thank-you.
        let app = web_lingua_router(module(counting(1), true, 60), vec![]);
        let resp = app
            .oneshot(post_form("reason=slow&browser=firefox&version=1.10"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn free_text_and_malformed_bodies_are_400() {
        for body in [
            r#"{"reason":"too slow, write to me at reader@example.com","browser":"firefox","version":"1.10"}"#,
            r#"{"reason":"slow","browser":"firefox","version":"1.10.0"}"#,
            r#"{"reason":"slow","browser":"netscape","version":"1.10"}"#,
            r#"{"browser":"firefox"}"#,
            "not json",
        ] {
            // No expectation: a counter write would panic.
            let app = web_lingua_router(
                module(MockUsageCountsRepo::new(), true, 60),
                vec![SITE.into()],
            );
            let resp = app.oneshot(post_leave(body, SITE)).await.unwrap();
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{body}");
        }
        // A body far larger than an answer is refused before it is read.
        let app = web_lingua_router(
            module(MockUsageCountsRepo::new(), true, 60),
            vec![SITE.into()],
        );
        let huge = format!(r#"{{"reason":"{}"}}"#, "x".repeat(4_096));
        let resp = app.oneshot(post_leave(&huge, SITE)).await.unwrap();
        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn a_disallowed_origin_gets_no_cors_grant() {
        let app = web_lingua_router(
            module(MockUsageCountsRepo::new(), true, 60),
            vec![SITE.into()],
        );
        let preflight = Request::builder()
            .method(Method::OPTIONS)
            .uri("/web/lingua/leave")
            .header("origin", "https://evil.example")
            .header("access-control-request-method", "POST")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(preflight).await.unwrap();
        assert!(resp.headers().get("access-control-allow-origin").is_none());
        let preflight = Request::builder()
            .method(Method::OPTIONS)
            .uri("/web/lingua/leave")
            .header("origin", SITE)
            .header("access-control-request-method", "POST")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(preflight).await.unwrap();
        assert_eq!(
            resp.headers().get("access-control-allow-origin").unwrap(),
            SITE
        );
    }

    #[tokio::test]
    async fn the_flag_off_acknowledges_and_counts_nothing() {
        let app = web_lingua_router(
            module(MockUsageCountsRepo::new(), false, 60),
            vec![SITE.into()],
        );
        let resp = app.oneshot(post_leave(SLOW, SITE)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn the_61st_request_of_the_hour_is_429() {
        let app = web_lingua_router(module(counting(60), true, 60), vec![SITE.into()]);
        for _ in 0..60 {
            let resp = app.clone().oneshot(post_leave(SLOW, SITE)).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
        let resp = app.oneshot(post_leave(SLOW, SITE)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn a_counter_not_written_is_503() {
        let _one_at_a_time = LOGGING_TESTS.lock().await;
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_leave()
            .returning(|_, _, _, _, _| Err(AppError::Internal(anyhow::anyhow!("pool timed out"))));
        let app = web_lingua_router(module(repo, true, 60), vec![SITE.into()]);
        let resp = app.oneshot(post_leave(SLOW, SITE)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    /// A `tracing` writer into a shared buffer.
    #[derive(Clone, Default)]
    struct Captured(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for Captured {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Task 2.8 (design D11): across a counted, a dropped and a refused report, a write that
    /// failed, and leaving answers, with the identifying headers set, the server's tracing
    /// output — every level — holds neither the address, the `Origin` nor the `User-Agent`.
    #[tokio::test]
    async fn nothing_identifying_reaches_the_logs() {
        const ADDRESS: &str = "203.0.113.77";
        const ORIGIN: &str = "moz-extension://4b1d7a3e-2f0c-4b8e-9d55-0c6f1a2b3c4d";
        const AGENT: &str = "Mozilla/5.0 (Macintosh; rv:140.0) Gecko/20100101 Firefox/140.0";

        let _one_at_a_time = LOGGING_TESTS.lock().await;
        let captured = Captured::default();
        let writer = captured.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(move || writer.clone())
            .with_ansi(false)
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);
        // A call site first hit before this capture existed is asked again.
        tracing::callsite::rebuild_interest_cache();

        let mut repo = MockUsageCountsRepo::new();
        // The first counted report is written; the next write fails (logged).
        let mut writes = 0;
        repo.expect_count_steps().returning(move |_, _, _, _, _| {
            writes += 1;
            if writes == 1 {
                Ok(())
            } else {
                Err(AppError::Internal(anyhow::anyhow!("lingua db: timed out")))
            }
        });
        repo.expect_count_leave().returning(|_, _, _, _, _| Ok(()));
        let module = module(repo, true, 4);
        let grpc = UsageCountsGrpc::new(module.clone());
        let report = |week: i32, steps: Vec<i32>| {
            let mut req = tonic::Request::new(ReportRequest {
                install_week: week,
                browser: 2,
                version: "1.10".into(),
                steps,
            });
            let md = req.metadata_mut();
            md.insert("x-forwarded-for", ADDRESS.parse().unwrap());
            md.insert("origin", ORIGIN.parse().unwrap());
            md.insert("user-agent", AGENT.parse().unwrap());
            md.insert("authorization", "Bearer eyJ.reader.bearer".parse().unwrap());
            req
        };
        let monday = monday_of(20_747);
        // Counted, then a write that fails, then dropped (week too old), then refused.
        grpc.report(report(monday, vec![1, 2])).await.unwrap();
        assert!(grpc.report(report(monday, vec![3])).await.is_err());
        grpc.report(report(monday_of(20_747 - 200), vec![1]))
            .await
            .unwrap();
        assert!(grpc.report(report(monday + 1, vec![1])).await.is_err());
        // The guard (4 an hour) now refuses the address: a flood refusal too.
        assert!(grpc.report(report(monday, vec![1])).await.is_err());

        let app = web_lingua_router(module, vec![SITE.into()]);
        for body in [SLOW, r#"{"reason":"free text"}"#] {
            let req = Request::builder()
                .method(Method::POST)
                .uri("/web/lingua/leave")
                .header(CONTENT_TYPE, "application/json")
                .header("origin", ORIGIN)
                .header("x-forwarded-for", "198.51.100.23")
                .header("user-agent", AGENT)
                .body(Body::from(body))
                .unwrap();
            let _ = app.clone().oneshot(req).await.unwrap();
        }

        let out = String::from_utf8(captured.0.lock().unwrap().clone()).unwrap();
        // The capture is live: the failed write was logged.
        assert!(out.contains("counter not written"), "{out}");
        for needle in [
            ADDRESS,
            "198.51.100.23",
            ORIGIN,
            "4b1d7a3e",
            AGENT,
            "Firefox/140.0",
            "eyJ.reader.bearer",
        ] {
            assert!(
                !out.contains(needle),
                "a request value leaked into the logs: {needle}"
            );
        }
    }
}
