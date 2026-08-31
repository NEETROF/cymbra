//! Browser-only **account routes** for the public site (change:
//! add-site-account-pages, spec `web-auth-session`):
//!
//! - `GET  /web/account/me`     — the caller's handle, display name, locale and
//!   linked sign-in methods.
//! - `POST /web/account/delete` — erase the caller's own account.
//!
//! Bearer-authenticated through the same seam as the plan routes, CORS restricted
//! to the web origins. The `local` identity's subject is the e-mail and is returned
//! as `email`; the OIDC subjects (Google / Apple) are **never** exposed — only the
//! provider name.
//!
//! The delete route exists because Google Play requires a **web** path to account
//! deletion for any app that offers sign-up: the in-app screen alone does not
//! satisfy it. It carries no privilege the app's `DeleteAccount` RPC does not — the
//! bearer names the only account it can erase.

use std::sync::Arc;

use axum::extract::State;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use cymbra_platform::AppError;
use cymbra_user_port::UserPort;
use serde::Serialize;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::soundfont::SoundfontAuth;
use crate::web_auth::{http_status, safe_message};

/// Router state: the identity port + the bearer auth seam.
#[derive(Clone)]
pub struct WebAccountState {
    pub users: Arc<dyn UserPort>,
    pub auth: Arc<dyn SoundfontAuth>,
}

/// One linked sign-in method. `email` only for the `local` provider.
#[derive(Serialize)]
struct IdentityView {
    provider: String,
    email: Option<String>,
    linked_at: i64,
}

#[derive(Serialize)]
struct AccountView {
    handle: Option<String>,
    display_name: Option<String>,
    locale: Option<String>,
    identities: Vec<IdentityView>,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

/// Answer of the delete route. A body (rather than `204 No Content`) because the
/// site's `fetchJson` parses every success as JSON.
#[derive(Serialize)]
struct DeletedView {
    deleted: bool,
}

/// Build the router (CORS-wrapped), ready to `.merge()` into the HTTP server.
pub fn web_account_router(state: WebAccountState, origins: Vec<String>) -> Router {
    let origins: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);
    Router::new()
        .route("/web/account/me", get(me))
        .route("/web/account/delete", post(delete_me))
        .layer(cors)
        .with_state(state)
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

/// Map a port identity to its public view: the e-mail for `local`, provider only otherwise.
fn identity_view(provider: String, subject: String, linked_at: i64) -> IdentityView {
    let email = (provider == "local").then_some(subject);
    IdentityView {
        provider,
        email,
        linked_at,
    }
}

async fn me(State(s): State<WebAccountState>, headers: HeaderMap) -> Response {
    let Some(id) = s.auth.identify_admin(&headers) else {
        return error_response(&AppError::Unauthenticated("sign in required".into()));
    };
    let account = match s.users.get_account(&id.user_id).await {
        Ok(a) => a,
        Err(e) => return error_response(&e),
    };
    let identities = match s.users.list_identities(&id.user_id).await {
        Ok(v) => v,
        Err(e) => return error_response(&e),
    };
    let view = AccountView {
        handle: account.handle,
        display_name: account.display_name,
        locale: account.locale,
        identities: identities
            .into_iter()
            .map(|i| identity_view(i.provider, i.subject, i.linked_at))
            .collect(),
    };
    (StatusCode::OK, Json(view)).into_response()
}

/// Erase the caller's own account. The port owns what "erase" means — with the job
/// queue wired it enqueues the cross-schema `purge_user`, so this returns once the
/// erasure is scheduled, exactly like the app's RPC.
async fn delete_me(State(s): State<WebAccountState>, headers: HeaderMap) -> Response {
    let Some(id) = s.auth.identify_admin(&headers) else {
        return error_response(&AppError::Unauthenticated("sign in required".into()));
    };
    if let Err(e) = s.users.delete_account(&id.user_id).await {
        return error_response(&e);
    }
    (StatusCode::OK, Json(DeletedView { deleted: true })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use cymbra_platform::identity::AuthIdentity;
    use cymbra_user_port::{Account, Identity, MockUserPort};
    use std::collections::BTreeMap;
    use tower::ServiceExt;

    const ORIGIN: &str = "https://cymbra.app";

    struct FixedAuth;
    impl SoundfontAuth for FixedAuth {
        fn identify(&self, headers: &HeaderMap) -> Option<String> {
            self.identify_admin(headers).map(|i| i.user_id)
        }
        fn identify_admin(&self, headers: &HeaderMap) -> Option<AuthIdentity> {
            let tok = headers
                .get(AUTHORIZATION)?
                .to_str()
                .ok()?
                .strip_prefix("Bearer ")?;
            (tok == "web").then(|| AuthIdentity {
                user_id: "u1".into(),
                audience: "web".into(),
                roles: vec![],
                roles_by_scope: BTreeMap::new(),
            })
        }
    }

    fn app(users: MockUserPort) -> Router {
        web_account_router(
            WebAccountState {
                users: Arc::new(users),
                auth: Arc::new(FixedAuth),
            },
            vec![ORIGIN.into()],
        )
    }

    fn req(bearer: Option<&str>) -> Request<Body> {
        let mut b = Request::builder()
            .method(Method::GET)
            .uri("/web/account/me");
        if let Some(t) = bearer {
            b = b.header(AUTHORIZATION, format!("Bearer {t}"));
        }
        b.body(Body::empty()).unwrap()
    }

    fn delete_req(bearer: Option<&str>) -> Request<Body> {
        let mut b = Request::builder()
            .method(Method::POST)
            .uri("/web/account/delete")
            .header(CONTENT_TYPE, "application/json");
        if let Some(t) = bearer {
            b = b.header(AUTHORIZATION, format!("Bearer {t}"));
        }
        b.body(Body::from("{}")).unwrap()
    }

    async fn json_of(resp: Response) -> serde_json::Value {
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn missing_bearer_is_401_before_any_read() {
        let mut users = MockUserPort::new();
        users.expect_get_account().never();
        users.expect_list_identities().never();
        let resp = app(users).oneshot(req(None)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert!(json_of(resp).await.get("error").is_some());
    }

    #[tokio::test]
    async fn summary_exposes_the_local_email_but_never_an_oidc_subject() {
        let mut users = MockUserPort::new();
        users.expect_get_account().returning(|_| {
            Ok(Account {
                user_id: "u1".into(),
                display_name: Some("Guillaume".into()),
                preferences: "{}".into(),
                version: 1,
                updated_at: 0,
                handle: Some("neetrof".into()),
                locale: Some("fr".into()),
            })
        });
        users.expect_list_identities().returning(|_| {
            Ok(vec![
                Identity {
                    provider: "google".into(),
                    subject: "1046789-opaque".into(),
                    linked_at: 10,
                },
                Identity {
                    provider: "local".into(),
                    subject: "g@example.org".into(),
                    linked_at: 20,
                },
            ])
        });
        let resp = app(users).oneshot(req(Some("web"))).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let v = json_of(resp).await;
        assert_eq!(v["handle"], "neetrof");
        assert_eq!(v["display_name"], "Guillaume");
        assert_eq!(v["locale"], "fr");
        let ids = v["identities"].as_array().unwrap();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0]["provider"], "google");
        assert!(ids[0]["email"].is_null());
        assert_eq!(ids[1]["provider"], "local");
        assert_eq!(ids[1]["email"], "g@example.org");
        assert!(!v.to_string().contains("1046789-opaque"));
    }

    #[tokio::test]
    async fn delete_without_bearer_is_401_and_erases_nothing() {
        let mut users = MockUserPort::new();
        users.expect_delete_account().never();
        let resp = app(users).oneshot(delete_req(None)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert!(json_of(resp).await.get("error").is_some());
    }

    #[tokio::test]
    async fn delete_erases_the_bearers_own_account_only() {
        let mut users = MockUserPort::new();
        // The route never reads a user id from the request: `u1` is the bearer's.
        users
            .expect_delete_account()
            .withf(|id| id == "u1")
            .times(1)
            .returning(|_| Ok(()));
        let resp = app(users).oneshot(delete_req(Some("web"))).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(json_of(resp).await["deleted"], true);
    }

    #[tokio::test]
    async fn delete_reports_a_port_failure_as_a_safe_error() {
        let mut users = MockUserPort::new();
        users
            .expect_delete_account()
            .returning(|_| Err(AppError::FailedPrecondition("queue down".into())));
        let resp = app(users).oneshot(delete_req(Some("web"))).await.unwrap();
        assert_ne!(resp.status(), StatusCode::OK);
        assert!(json_of(resp).await.get("error").is_some());
    }

    #[tokio::test]
    async fn cors_allows_the_site_origin_only() {
        let preflight = |origin: &str| {
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/web/account/me")
                .header("origin", origin)
                .header("access-control-request-method", "GET")
                .header("access-control-request-headers", "authorization")
                .body(Body::empty())
                .unwrap()
        };
        let resp = app(MockUserPort::new())
            .oneshot(preflight(ORIGIN))
            .await
            .unwrap();
        assert_eq!(
            resp.headers().get("access-control-allow-origin").unwrap(),
            ORIGIN
        );
        let resp = app(MockUserPort::new())
            .oneshot(preflight("https://evil.example"))
            .await
            .unwrap();
        assert!(resp.headers().get("access-control-allow-origin").is_none());
    }

    #[tokio::test]
    async fn cors_preflights_the_delete_post() {
        let preflight = Request::builder()
            .method(Method::OPTIONS)
            .uri("/web/account/delete")
            .header("origin", ORIGIN)
            .header("access-control-request-method", "POST")
            .header("access-control-request-headers", "authorization")
            .body(Body::empty())
            .unwrap();
        let resp = app(MockUserPort::new()).oneshot(preflight).await.unwrap();
        assert_eq!(
            resp.headers().get("access-control-allow-origin").unwrap(),
            ORIGIN
        );
    }
}
