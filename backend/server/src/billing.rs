//! Provider notification routes + channel wiring (change: add-premium-subscription,
//! design D7–D9). Three public HTTP endpoints, each authenticated by its
//! provider's own mechanism BEFORE any side effect, idempotent by event id, and
//! **acknowledge-and-ignore** while the channel's flag is off (a disabled channel
//! must not make the provider retry forever). Configuration comes from the
//! environment (`cymbra_plans::billing::env`); a channel with no configuration is
//! simply not mounted.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Router, response::IntoResponse};
use chrono::Utc;
pub use cymbra_plans::billing::env::{AppleEnv, BillingChannels, BillingEnv, GoogleEnv, WebEnv};
use cymbra_plans::billing::web::verify_signature;
use cymbra_plans::{AppError, Channel, PaywallConfigSource, PlanService};

#[derive(Clone)]
pub struct BillingState {
    pub svc: Arc<PlanService>,
    pub paywall: Arc<dyn PaywallConfigSource>,
    pub channels: BillingChannels,
}

/// The provider notification routes; unconfigured channels are not mounted.
pub fn billing_router(state: BillingState) -> Router {
    let mut r = Router::new();
    if state.channels.apple.is_some() {
        r = r.route("/billing/apple/notifications", post(apple_notifications));
    }
    if state.channels.google_api.is_some() {
        r = r.route("/billing/google/rtdn", post(google_rtdn));
    }
    if state.channels.web.is_some() {
        r = r.route("/billing/web/webhook", post(web_webhook));
    }
    r.with_state(state)
}

fn status_for(e: &AppError) -> StatusCode {
    match e {
        AppError::Unauthenticated(_) | AppError::PermissionDenied(_) => StatusCode::UNAUTHORIZED,
        AppError::InvalidArgument(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn apple_notifications(
    State(s): State<BillingState>,
    body: Bytes,
) -> axum::response::Response {
    if !s.paywall.channel_enabled(Channel::Apple) {
        tracing::info!("apple notification ignored (billing.apple.enabled off)");
        return StatusCode::OK.into_response();
    }
    let Some(cfg) = s.channels.apple.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match cymbra_plans::billing::apple::handle_notification(&s.svc, cfg, &body, Utc::now()).await {
        Ok(outcome) => {
            tracing::info!(?outcome, "apple notification");
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "apple notification refused");
            status_for(&e).into_response()
        }
    }
}

async fn google_rtdn(
    State(s): State<BillingState>,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    if !s.paywall.channel_enabled(Channel::Google) {
        tracing::info!("google rtdn ignored (billing.google.enabled off)");
        return StatusCode::OK.into_response();
    }
    let (Some(api), Some(auth), Some(pkg)) = (
        s.channels.google_api.as_ref(),
        s.channels.google_push_auth.as_ref(),
        s.channels.google_package.as_deref(),
    ) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // Authenticate the push (Google-signed OIDC token) BEFORE any side effect.
    let bearer = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    if let Err(e) = auth.verify(bearer).await {
        tracing::warn!(error = %e, "google rtdn refused");
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match cymbra_plans::billing::google::handle_rtdn(&s.svc, api.as_ref(), pkg, &body, Utc::now())
        .await
    {
        Ok(outcome) => {
            tracing::info!(?outcome, "google rtdn");
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "google rtdn failed");
            status_for(&e).into_response()
        }
    }
}

async fn web_webhook(
    State(s): State<BillingState>,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    if !s.paywall.channel_enabled(Channel::Web) {
        tracing::info!("web webhook ignored (billing.web.enabled off)");
        return StatusCode::OK.into_response();
    }
    let Some(secret) = s.channels.web_secret.as_deref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let sig = headers
        .get("paddle-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if let Err(e) = verify_signature(sig, &body, secret, Utc::now().timestamp(), 300) {
        tracing::warn!(error = %e, "web webhook refused");
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match cymbra_plans::billing::web::handle_webhook(&s.svc, &body, Utc::now()).await {
        Ok(outcome) => {
            tracing::info!(?outcome, "web webhook");
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "web webhook failed");
            status_for(&e).into_response()
        }
    }
}
