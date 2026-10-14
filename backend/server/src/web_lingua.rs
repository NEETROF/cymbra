//! The Lingua leaving page's one JSON route (change: add-lingua-usage-counts, spec
//! `lingua-usage-counts`, D12): `POST /web/lingua/leave` with `{ reason, browser, version }`.
//!
//! No authentication and no cookie: the answer is counted, the same anonymous way as the
//! count reports, under the UTC week it arrived in — through the same module, so the same
//! flag, flood guard, ceiling and retention apply. The handler reads the body and the
//! forwarded address (for the flood guard alone) and nothing else; it logs nothing about the
//! request. CORS allows only the site's origins (`CYMBRA_WEB_ORIGINS`), without credentials.
//! Mounted only when the Lingua module is wired.
//!
//! Answers: `200 {}` whether counted or not, `400` for anything but one of the six answers
//! (no free text), `429` past the flood limit, `503` when the counter could not be written.

use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::header::CONTENT_TYPE;
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
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Serialize)]
struct Empty {}

/// Build the router (CORS-wrapped), ready to `.merge()` into the HTTP server. `origins` are
/// the exact browser origins allowed (`CYMBRA_WEB_ORIGINS`).
pub fn web_lingua_router(module: Arc<UsageCountsModule>, origins: Vec<String>) -> Router {
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
        .with_state(module)
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

async fn leave(
    State(module): State<Arc<UsageCountsModule>>,
    headers: HeaderMap,
    body: Result<Json<LeaveBody>, JsonRejection>,
) -> Response {
    let Ok(Json(body)) = body else {
        return error_response(&AppError::InvalidArgument(
            "the answer is not one of the six".into(),
        ));
    };
    let addr = client_addr(&headers);
    match module
        .leave(
            &addr,
            &Leave {
                reason: body.reason,
                browser: body.browser,
                version: body.version,
            },
        )
        .await
    {
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
        repo.expect_delete_before().returning(|_| Ok(()));
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
        repo.expect_delete_before().returning(|_| Ok(()));
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
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_delete_before().returning(|_| Ok(()));
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

        let captured = Captured::default();
        let writer = captured.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(move || writer.clone())
            .with_ansi(false)
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);

        let mut repo = MockUsageCountsRepo::new();
        repo.expect_delete_before().returning(|_| Ok(()));
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
