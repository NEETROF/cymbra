//! Provider notification routes + channel wiring (change: add-premium-subscription
//! D9 for the web merchant-of-record; swap-store-billing-to-revenuecat D4 for the
//! store aggregator). Two public HTTP endpoints, each authenticated by its
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
pub use cymbra_plans::billing::env::{BillingChannels, BillingEnv, RevenueCatEnv, WebEnv};
use cymbra_plans::billing::revenuecat::WebhookOutcome;
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
    if state.channels.revenuecat.is_some() {
        r = r.route("/billing/revenuecat/webhook", post(revenuecat_webhook));
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

/// The store aggregator's webhook: shared-secret `Authorization` header
/// (constant-time), event id idempotency, per-store flag inside the handler.
/// Past authentication the answer is always 200 — RevenueCat retries on anything
/// else, and a skip is a decision, not a failure.
async fn revenuecat_webhook(
    State(s): State<BillingState>,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    let Some(cfg) = s.channels.revenuecat.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let authorization = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    match cymbra_plans::billing::revenuecat::handle_webhook(
        &s.svc,
        cfg,
        s.paywall.as_ref(),
        s.channels.rc_customers.as_deref(),
        authorization,
        &body,
        Utc::now(),
    )
    .await
    {
        Ok(outcome) => {
            match &outcome {
                WebhookOutcome::ChannelDisabled(src) => tracing::info!(
                    source = src.as_str(),
                    "revenuecat event ignored (billing.<channel>.enabled off)"
                ),
                WebhookOutcome::Skipped(reason) => {
                    tracing::info!(?reason, "revenuecat event skipped")
                }
                other => tracing::info!(?other, "revenuecat webhook"),
            }
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "revenuecat webhook refused");
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
