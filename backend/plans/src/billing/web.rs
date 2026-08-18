//! Web channel (design D9): one merchant-of-record — Paddle Billing — hosted
//! checkout, hosted customer portal, cancellation on erasure, and HMAC-signed
//! webhooks. Cymbra keeps the provider customer / subscription identifiers and
//! nothing else about the buyer. [`verify_signature`] and [`map_subscription`]
//! are the pure cores.

use crate::billing::{IngestOutcome, ProviderEvent, ingest, payload_digest};
use crate::model::{EntitlementStatus, EventProvider, Source};
use crate::ports::{EntitlementWrite, WebBillingProvider, WebSubscriptionCanceller};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cymbra_platform::{AppError, Result};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;

// ------------------------------------------------------------- signature

/// Verify a `Paddle-Signature: ts=<unix>;h1=<hex>` header over `ts:body` with
/// the endpoint secret; `max_skew_secs` bounds replay.
pub fn verify_signature(
    header: &str,
    body: &[u8],
    secret: &str,
    now_unix: i64,
    max_skew_secs: i64,
) -> Result<()> {
    let mut ts: Option<i64> = None;
    let mut h1: Vec<String> = Vec::new();
    for part in header.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("ts=") {
            ts = v.parse().ok();
        } else if let Some(v) = part.strip_prefix("h1=") {
            h1.push(v.to_string());
        }
    }
    let ts = ts.ok_or_else(|| AppError::Unauthenticated("web webhook: missing ts".into()))?;
    if (now_unix - ts).abs() > max_skew_secs {
        return Err(AppError::Unauthenticated(
            "web webhook: stale timestamp".into(),
        ));
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::Config("web webhook secret".into()))?;
    mac.update(ts.to_string().as_bytes());
    mac.update(b":");
    mac.update(body);
    let expected: String = mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if h1
        .iter()
        .any(|h| constant_time_eq(h.as_bytes(), expected.as_bytes()))
    {
        Ok(())
    } else {
        Err(AppError::Unauthenticated(
            "web webhook: bad signature".into(),
        ))
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

// ------------------------------------------------------------- payloads

/// A Paddle subscription entity (the fields the ledger needs).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct WebSubscription {
    pub id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub customer_id: Option<String>,
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub canceled_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub paused_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub current_billing_period: Option<BillingPeriod>,
    #[serde(default)]
    pub scheduled_change: Option<ScheduledChange>,
    #[serde(default)]
    pub custom_data: Option<CustomData>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct BillingPeriod {
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct ScheduledChange {
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub effective_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct CustomData {
    #[serde(default)]
    pub user_id: Option<String>,
}

/// Pure mapper: a Paddle subscription → the ledger write, or `None` when it
/// carries no plan state.
pub fn map_subscription(
    user_id: &str,
    sub: &WebSubscription,
    now: DateTime<Utc>,
) -> Option<EntitlementWrite> {
    let period_end = sub.current_billing_period.as_ref().and_then(|p| p.ends_at);
    let scheduled_cancel = sub
        .scheduled_change
        .as_ref()
        .is_some_and(|c| c.action.as_deref() == Some("cancel"));
    let (status, ends_at) = match sub.status.as_deref() {
        Some("active") | Some("trialing") if scheduled_cancel => (
            EntitlementStatus::Cancelled,
            sub.scheduled_change
                .as_ref()
                .and_then(|c| c.effective_at)
                .or(period_end),
        ),
        Some("active") | Some("trialing") => (EntitlementStatus::Active, period_end),
        Some("past_due") => (EntitlementStatus::BillingRetry, period_end),
        Some("paused") => (EntitlementStatus::Ended, sub.paused_at.or(Some(now))),
        Some("canceled") => (
            EntitlementStatus::Ended,
            sub.canceled_at.or(period_end).or(Some(now)),
        ),
        _ => return None,
    };
    Some(EntitlementWrite {
        user_id: user_id.to_string(),
        source: Source::Web,
        provider_ref: sub.id.clone(),
        campaign_id: None,
        starts_at: sub.started_at.unwrap_or(now),
        ends_at: ends_at.or(Some(now)),
        status,
    })
}

/// A refund adjustment ends the row now.
pub fn map_refund(user_id: &str, subscription_id: &str, now: DateTime<Utc>) -> EntitlementWrite {
    EntitlementWrite {
        user_id: user_id.to_string(),
        source: Source::Web,
        provider_ref: subscription_id.to_string(),
        campaign_id: None,
        starts_at: now,
        ends_at: Some(now),
        status: EntitlementStatus::Refunded,
    }
}

// -------------------------------------------------------------- webhook

#[derive(Debug, Deserialize)]
struct WebhookEnvelope {
    event_id: String,
    event_type: String,
    #[serde(default)]
    data: serde_json::Value,
}

/// Handle one (signature-verified) Paddle webhook body: idempotent by
/// `event_id`; subscription events re-map the entity; refund adjustments end the
/// row; everything else is acknowledged as a no-op.
pub async fn handle_webhook(
    svc: &crate::service::PlanService,
    body: &[u8],
    now: DateTime<Utc>,
) -> Result<IngestOutcome> {
    let env: WebhookEnvelope = serde_json::from_slice(body)
        .map_err(|_| AppError::InvalidArgument("bad webhook body".into()))?;
    let mut writes = Vec::new();
    if env.event_type.starts_with("subscription.") {
        let sub: WebSubscription = serde_json::from_value(env.data.clone())
            .map_err(|_| AppError::InvalidArgument("bad subscription entity".into()))?;
        let uid = match sub.custom_data.as_ref().and_then(|c| c.user_id.clone()) {
            Some(u) => Some(u),
            None => svc.user_for_ref(Source::Web, &sub.id).await?,
        };
        match uid {
            Some(uid) => {
                if let Some(w) = map_subscription(&uid, &sub, now) {
                    writes.push(w);
                }
            }
            None => tracing::warn!(
                subscription = %sub.id,
                "web webhook for an unknown subscription (no user id) — acknowledged"
            ),
        }
    } else if env.event_type == "adjustment.created"
        && env.data.get("action").and_then(|a| a.as_str()) == Some("refund")
        && let Some(sub_id) = env.data.get("subscription_id").and_then(|s| s.as_str())
        && let Some(uid) = svc.user_for_ref(Source::Web, sub_id).await?
    {
        writes.push(map_refund(&uid, sub_id, now));
    }
    ingest(
        svc,
        ProviderEvent {
            provider: EventProvider::Web,
            event_id: env.event_id,
            payload_digest: payload_digest(body),
            writes,
        },
    )
    .await
}

// -------------------------------------------------------------- provider

/// Paddle Billing API client (checkout / portal / cancel).
pub struct PaddleProvider {
    api_key: String,
    base: String,
    /// The hosted checkout page (the site page carrying Paddle.js) that receives
    /// `_ptxn=<transaction id>`; also sent as the transaction's `checkout.url` so
    /// Paddle binds the transaction to that approved page. The success URL is a
    /// client-side setting of the page (`/checkout/done`), not a transaction field.
    checkout_page: String,
    http: reqwest::Client,
}

impl PaddleProvider {
    /// `sandbox` selects the sandbox API host.
    pub fn new(api_key: String, sandbox: bool, checkout_page: String) -> Self {
        Self {
            api_key,
            base: if sandbox {
                "https://sandbox-api.paddle.com".into()
            } else {
                "https://api.paddle.com".into()
            },
            checkout_page,
            http: reqwest::Client::new(),
        }
    }

    async fn post(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        self.http
            .post(format!("{}{path}", self.base))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("paddle: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("paddle status: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("paddle body: {e}")))
    }

    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        self.http
            .get(format!("{}{path}", self.base))
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("paddle: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("paddle status: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("paddle body: {e}")))
    }

    /// Read a subscription (reconciliation).
    pub async fn subscription(&self, id: &str) -> Result<WebSubscription> {
        let v = self.get(&format!("/subscriptions/{id}")).await?;
        serde_json::from_value(v["data"].clone())
            .map_err(|_| AppError::Internal(anyhow::anyhow!("paddle subscription shape")))
    }
}

#[async_trait]
impl WebBillingProvider for PaddleProvider {
    async fn create_checkout(&self, user_id: &str, product_id: &str) -> Result<String> {
        // `product_id` is the Paddle price id configured in `plans.premium.products`.
        let body = serde_json::json!({
            "items": [{ "price_id": product_id, "quantity": 1 }],
            "custom_data": { "user_id": user_id },
            // Paddle's `checkout.url` is the page hosting Paddle.js (an approved
            // domain), never the success URL.
            "checkout": { "url": self.checkout_page },
        });
        let v = self.post("/transactions", body).await?;
        let txn = v["data"]["id"]
            .as_str()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("paddle transaction id")))?;
        // Hosted checkout: the checkout page receives the transaction id.
        Ok(format!("{}?_ptxn={txn}", self.checkout_page))
    }

    async fn portal_url(&self, subscription_ref: &str) -> Result<String> {
        let sub = self.subscription(subscription_ref).await?;
        let customer = sub
            .customer_id
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("paddle customer id")))?;
        let v = self
            .post(
                &format!("/customers/{customer}/portal-sessions"),
                serde_json::json!({ "subscription_ids": [subscription_ref] }),
            )
            .await?;
        v["data"]["urls"]["general"]["overview"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("paddle portal url")))
    }

    async fn cancel(&self, subscription_ref: &str) -> Result<()> {
        self.post(
            &format!("/subscriptions/{subscription_ref}/cancel"),
            serde_json::json!({ "effective_from": "immediately" }),
        )
        .await
        .map(|_| ())
    }
}

#[async_trait]
impl WebSubscriptionCanceller for PaddleProvider {
    async fn cancel(&self, subscription_ref: &str) -> Result<()> {
        WebBillingProvider::cancel(self, subscription_ref).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn sign(body: &[u8], secret: &str, ts: i64) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(ts.to_string().as_bytes());
        mac.update(b":");
        mac.update(body);
        let h: String = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        format!("ts={ts};h1={h}")
    }

    #[test]
    fn signature_verifies_and_rejects_bad_or_stale() {
        let body = br#"{"event_id":"evt_1"}"#;
        let now = 1_700_000_000;
        let header = sign(body, "secret", now);
        assert!(verify_signature(&header, body, "secret", now, 300).is_ok());
        assert!(verify_signature(&header, body, "other", now, 300).is_err());
        assert!(verify_signature(&header, b"tampered", "secret", now, 300).is_err());
        assert!(verify_signature(&header, body, "secret", now + 1000, 300).is_err());
        assert!(verify_signature("h1=abc", body, "secret", now, 300).is_err());
    }

    fn sub(status: &str, days: i64) -> WebSubscription {
        WebSubscription {
            id: "sub_1".into(),
            status: Some(status.into()),
            customer_id: Some("ctm_1".into()),
            started_at: Some(Utc::now() - Duration::days(10)),
            current_billing_period: Some(BillingPeriod {
                starts_at: Some(Utc::now() - Duration::days(10)),
                ends_at: Some(Utc::now() + Duration::days(days)),
            }),
            custom_data: Some(CustomData {
                user_id: Some("u1".into()),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn mapper_table() {
        let now = Utc::now();
        let m = |s: &WebSubscription| map_subscription("u1", s, now);
        assert_eq!(
            m(&sub("active", 20)).unwrap().status,
            EntitlementStatus::Active
        );
        assert_eq!(
            m(&sub("trialing", 20)).unwrap().status,
            EntitlementStatus::Active
        );
        assert_eq!(
            m(&sub("past_due", 20)).unwrap().status,
            EntitlementStatus::BillingRetry
        );
        assert_eq!(
            m(&sub("paused", 20)).unwrap().status,
            EntitlementStatus::Ended
        );
        let mut cancelled = sub("canceled", 20);
        cancelled.canceled_at = Some(now);
        assert_eq!(m(&cancelled).unwrap().status, EntitlementStatus::Ended);
        // scheduled cancel at period end: cancelled, active until then
        let mut scheduled = sub("active", 20);
        scheduled.scheduled_change = Some(ScheduledChange {
            action: Some("cancel".into()),
            effective_at: Some(now + Duration::days(20)),
        });
        let w = m(&scheduled).unwrap();
        assert_eq!(w.status, EntitlementStatus::Cancelled);
        assert!(w.ends_at.unwrap() > now + Duration::days(19));
        assert!(m(&sub("weird", 20)).is_none());
        assert_eq!(m(&sub("active", 20)).unwrap().provider_ref, "sub_1");
        assert_eq!(
            map_refund("u1", "sub_1", now).status,
            EntitlementStatus::Refunded
        );
    }
}
