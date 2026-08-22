//! RevenueCat — the store aggregator in front of the App Store (iOS + macOS),
//! Google Play and, once routed through it, Paddle (change:
//! swap-store-billing-to-revenuecat, design D1–D4, D6).
//!
//! Two ways store facts reach the ledger, both ending in [`PlanService::apply`]:
//! - **push**: the RevenueCat webhook ([`handle_webhook`]) — shared-secret
//!   `Authorization` header compared in constant time before the body is read,
//!   event id as the idempotency key, per-store `billing.<channel>.enabled`
//!   kill-switch (disabled ⇒ acknowledged and ignored), and a pure mapper
//!   ([`map_event`]) from the event type to a ledger transition;
//! - **pull**: [`sync_customer`] — the calling account's subscriptions read from
//!   the RevenueCat customer API ([`StoreCustomerSource`]) and mapped by
//!   [`map_customer`]; used by `SyncStorePlan` after a purchase/restore and by the
//!   reconciliation job. Reads only the given account, so no account can claim
//!   another one's subscription.
//!
//! Ledger rows keep the **store** as `source` (`APP_STORE`/`MAC_APP_STORE` →
//! `apple`, `PLAY_STORE` → `google`, `PADDLE` → `web`); `provider_ref` is the
//! store's original transaction id as RevenueCat reports it. Only products in the
//! premium set grant; other stores/products, sandbox events in production and
//! informational event types are acknowledged as counted no-ops. Nothing about
//! amounts is stored (D5). The HTTP client lives in `rc_client.rs` (thin glue,
//! coverage-excluded); the customer-API payload projection is here and tested.

use crate::billing::{IngestOutcome, ProviderEvent, ingest, payload_digest};
use crate::model::{EntitlementStatus, EventProvider, Source};
use crate::ports::{
    Channel, EntitlementWrite, PaywallConfigSource, StoreCustomerSource, StoreSubscription,
};
use crate::service::PlanService;
use chrono::{DateTime, Utc};
use cymbra_platform::{AppError, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

// ------------------------------------------------------------------ config

/// The aggregator's configuration (from the environment; see `env.rs`).
#[derive(Debug, Clone)]
pub struct RcConfig {
    /// The value RevenueCat sends in the webhook `Authorization` header.
    pub webhook_secret: String,
    /// Secret (v1) API key for the customer API.
    pub api_key: String,
    /// Project id (dashboard deep links only).
    pub project_id: Option<String>,
    /// Apply `SANDBOX` events / subscriptions (staging only).
    pub allow_sandbox: bool,
}

/// RevenueCat store name → ledger source. Accepts the webhook casing
/// (`APP_STORE`) and the v1 customer API casing (`app_store`). Stores Cymbra does
/// not sell through map to `None`.
pub fn store_to_source(store: &str) -> Option<Source> {
    match store.to_ascii_uppercase().as_str() {
        "APP_STORE" | "MAC_APP_STORE" => Some(Source::Apple),
        "PLAY_STORE" => Some(Source::Google),
        "PADDLE" => Some(Source::Web),
        _ => None,
    }
}

fn channel_of(source: Source) -> Option<Channel> {
    match source {
        Source::Apple => Some(Channel::Apple),
        Source::Google => Some(Channel::Google),
        Source::Web => Some(Channel::Web),
        Source::Code | Source::Admin => None,
    }
}

fn ms(v: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp_millis(v)
}

fn is_premium_product(products: &[String], product: &str) -> bool {
    // An empty product set would grant nothing; treat it as "any" and let the
    // paywall config own the list. Play subscriptions under the base-plans model
    // are reported as `subscription_id:base_plan_id` (e.g.
    // `premium_monthly:monthly`): the part before `:` is the store product id,
    // so both spellings match the configured set (`:` is not a legal store id
    // character, the split is unambiguous).
    let base = product.split(':').next().unwrap_or(product);
    products.is_empty() || products.iter().any(|p| p == product || p == base)
}

/// Google order ids gain a `..N` suffix on every renewal (`GPA.x-x-x-x..0`,
/// `..1`, …) while the base id names the subscription for its whole life.
/// RevenueCat is not consistent about which spelling an event carries (observed
/// in 7.4: `INITIAL_PURCHASE`/`RENEWAL` carry the base id, `CANCELLATION` the
/// suffixed one), and a ledger keyed on the raw value splits one subscription
/// across `(source, provider_ref)` rows. Strip the suffix for `google` refs;
/// Apple/web ids never use this spelling.
fn normalize_provider_ref(source: Source, provider_ref: String) -> String {
    if source != Source::Google {
        return provider_ref;
    }
    match provider_ref.rfind("..") {
        Some(i)
            if i + 2 < provider_ref.len()
                && provider_ref[i + 2..].bytes().all(|b| b.is_ascii_digit()) =>
        {
            provider_ref[..i].to_string()
        }
        _ => provider_ref,
    }
}

// ------------------------------------------------------------- event shape

/// A RevenueCat webhook body: `{ "api_version": "1.0", "event": { … } }`.
#[derive(Debug, Clone, Deserialize)]
pub struct RcWebhookBody {
    pub api_version: Option<String>,
    pub event: RcEvent,
}

/// The fields of a RevenueCat event the mapper reads. Tolerant: unknown fields
/// and unknown types are accepted (they map to no-ops).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RcEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub app_user_id: String,
    pub store: Option<String>,
    pub environment: Option<String>,
    pub product_id: Option<String>,
    pub new_product_id: Option<String>,
    pub original_transaction_id: Option<String>,
    pub transaction_id: Option<String>,
    pub purchased_at_ms: Option<i64>,
    pub expiration_at_ms: Option<i64>,
    pub grace_period_expiration_at_ms: Option<i64>,
    pub cancel_reason: Option<String>,
    pub expiration_reason: Option<String>,
    #[serde(default)]
    pub transferred_from: Vec<String>,
    #[serde(default)]
    pub transferred_to: Vec<String>,
}

/// Why an event produced no write (counted by the caller).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// Informational or unknown event type.
    Informational(String),
    /// A store Cymbra does not sell through (`STRIPE`, `PROMOTIONAL`, …).
    UnmappedStore(String),
    /// A product outside `plans.premium.products`.
    ProductNotPremium(String),
    /// `environment = SANDBOX` while sandbox acceptance is off.
    Sandbox,
    /// `app_user_id` is not a Cymbra account id (anonymous SDK id, garbage).
    MalformedUser,
    /// The event lacks the fields its transition needs.
    MissingFields(&'static str),
}

/// What an event maps to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mapped {
    Writes(Vec<EntitlementWrite>),
    /// `TRANSFER`: end the source accounts' rows of that store, re-read the
    /// destination accounts (needs the service — handled by the caller).
    Transfer {
        source: Source,
        from: Vec<String>,
        to: Vec<String>,
    },
    Skip(SkipReason),
}

/// Pure mapper: one webhook event → the ledger transition of design D4.
pub fn map_event(
    ev: &RcEvent,
    premium_products: &[String],
    now: DateTime<Utc>,
    allow_sandbox: bool,
) -> Mapped {
    use EntitlementStatus as S;
    let kind = ev.kind.as_str();
    // Informational types first: they need no other field.
    match kind {
        "INITIAL_PURCHASE"
        | "RENEWAL"
        | "UNCANCELLATION"
        | "SUBSCRIPTION_EXTENDED"
        | "REFUND_REVERSED"
        | "TEMPORARY_ENTITLEMENT_GRANT"
        | "PRODUCT_CHANGE"
        | "BILLING_ISSUE"
        | "CANCELLATION"
        | "EXPIRATION"
        | "SUBSCRIPTION_PAUSED"
        | "TRANSFER" => {}
        other => return Mapped::Skip(SkipReason::Informational(other.to_string())),
    }
    if ev
        .environment
        .as_deref()
        .is_some_and(|e| e.eq_ignore_ascii_case("SANDBOX"))
        && !allow_sandbox
    {
        return Mapped::Skip(SkipReason::Sandbox);
    }
    let Some(source) = ev.store.as_deref().and_then(store_to_source) else {
        return Mapped::Skip(SkipReason::UnmappedStore(
            ev.store.clone().unwrap_or_default(),
        ));
    };
    if kind == "TRANSFER" {
        return Mapped::Transfer {
            source,
            from: ev.transferred_from.clone(),
            to: ev.transferred_to.clone(),
        };
    }
    if Uuid::parse_str(&ev.app_user_id).is_err() {
        return Mapped::Skip(SkipReason::MalformedUser);
    }
    let product = match kind {
        "PRODUCT_CHANGE" => ev.new_product_id.as_deref().or(ev.product_id.as_deref()),
        _ => ev.product_id.as_deref(),
    };
    let Some(product) = product else {
        return Mapped::Skip(SkipReason::MissingFields("product_id"));
    };
    if !is_premium_product(premium_products, product) {
        return Mapped::Skip(SkipReason::ProductNotPremium(product.to_string()));
    }
    let Some(provider_ref) = ev
        .original_transaction_id
        .clone()
        .or_else(|| ev.transaction_id.clone())
        .map(|r| normalize_provider_ref(source, r))
    else {
        return Mapped::Skip(SkipReason::MissingFields("original_transaction_id"));
    };
    let expires = ev.expiration_at_ms.and_then(ms);
    let starts_at = ev.purchased_at_ms.and_then(ms).unwrap_or(now);
    let (status, ends_at) = match kind {
        "INITIAL_PURCHASE"
        | "RENEWAL"
        | "UNCANCELLATION"
        | "SUBSCRIPTION_EXTENDED"
        | "REFUND_REVERSED"
        | "TEMPORARY_ENTITLEMENT_GRANT"
        | "PRODUCT_CHANGE" => (S::Active, expires),
        "BILLING_ISSUE" => (
            S::BillingRetry,
            ev.grace_period_expiration_at_ms.and_then(ms).or(expires),
        ),
        "CANCELLATION" => {
            let refund = ev
                .cancel_reason
                .as_deref()
                .is_some_and(|r| r == "CUSTOMER_SUPPORT")
                || expires.is_some_and(|e| e <= now);
            if refund {
                (S::Refunded, Some(expires.map_or(now, |e| e.min(now))))
            } else {
                (S::Cancelled, expires)
            }
        }
        "EXPIRATION" | "SUBSCRIPTION_PAUSED" => (S::Ended, expires),
        _ => unreachable!("filtered above"),
    };
    let Some(ends_at) = ends_at else {
        return Mapped::Skip(SkipReason::MissingFields("expiration_at_ms"));
    };
    Mapped::Writes(vec![EntitlementWrite {
        user_id: ev.app_user_id.clone(),
        source,
        provider_ref,
        campaign_id: None,
        starts_at,
        ends_at: Some(ends_at),
        status,
    }])
}

/// Pure mapper: one account's subscriptions (customer API) → ledger writes,
/// same rules as [`map_event`]. Unmapped stores, non-premium products and
/// sandbox rows (in production) are dropped.
pub fn map_customer(
    user_id: &str,
    subs: &[StoreSubscription],
    premium_products: &[String],
    now: DateTime<Utc>,
    allow_sandbox: bool,
) -> Vec<EntitlementWrite> {
    use EntitlementStatus as S;
    let mut out = Vec::new();
    for s in subs {
        let Some(source) = store_to_source(&s.store) else {
            continue;
        };
        if (s.is_sandbox && !allow_sandbox)
            || !is_premium_product(premium_products, &s.product_id)
            || s.provider_ref.is_empty()
        {
            continue;
        }
        let (status, ends_at) = if let Some(r) = s.refunded_at {
            (S::Refunded, Some(r))
        } else if s.billing_issues_detected_at.is_some() && s.expires_at.is_some_and(|e| e <= now) {
            (S::BillingRetry, s.grace_period_expires_at.or(s.expires_at))
        } else if s.unsubscribe_detected_at.is_some() {
            (S::Cancelled, s.expires_at)
        } else if s.expires_at.is_some_and(|e| e <= now) {
            (S::Ended, s.expires_at)
        } else {
            (S::Active, s.expires_at)
        };
        let Some(ends_at) = ends_at else {
            continue;
        };
        out.push(EntitlementWrite {
            user_id: user_id.to_string(),
            source,
            provider_ref: normalize_provider_ref(source, s.provider_ref.clone()),
            campaign_id: None,
            starts_at: s.purchase_at.unwrap_or(now),
            ends_at: Some(ends_at),
            status,
        });
    }
    out
}

// ---------------------------------------------------------------- webhook

/// What the webhook route logs (it always answers 200 past authentication).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebhookOutcome {
    Ingested(IngestOutcome),
    Skipped(SkipReason),
    /// The store's `billing.<channel>.enabled` flag is off: acknowledged, ignored.
    ChannelDisabled(Source),
    /// A `TRANSFER` was applied (rows ended, destinations re-read).
    Transferred {
        ended: u64,
        resynced: u64,
    },
}

/// Constant-time equality (length leaks; contents do not).
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Authenticate the request. `Err(Unauthenticated)` ⇒ 401, no side effect.
pub fn authenticate(cfg: &RcConfig, authorization: Option<&str>) -> Result<()> {
    let given = authorization.unwrap_or("");
    if given.is_empty() || !ct_eq(given.as_bytes(), cfg.webhook_secret.as_bytes()) {
        return Err(AppError::Unauthenticated(
            "revenuecat webhook: bad authorization".into(),
        ));
    }
    Ok(())
}

/// Handle one webhook delivery: authenticate → parse → map → per-store
/// kill-switch → ingest exactly once. `customers` (the customer API) is only
/// needed for `TRANSFER` destinations; without it they are logged and left to
/// the next reconciliation.
pub async fn handle_webhook(
    svc: &PlanService,
    cfg: &RcConfig,
    paywall: &dyn PaywallConfigSource,
    customers: Option<&dyn StoreCustomerSource>,
    authorization: Option<&str>,
    body: &[u8],
    now: DateTime<Utc>,
) -> Result<WebhookOutcome> {
    authenticate(cfg, authorization)?;
    let parsed: RcWebhookBody = serde_json::from_slice(body)
        .map_err(|e| AppError::InvalidArgument(format!("revenuecat webhook body: {e}")))?;
    let ev = parsed.event;
    let products = paywall.products();
    let mapped = map_event(&ev, &products, now, cfg.allow_sandbox);
    let digest = payload_digest(body);
    match mapped {
        Mapped::Skip(reason) => {
            // Record it (replays become duplicates) but write nothing.
            let out = ingest(
                svc,
                ProviderEvent {
                    provider: EventProvider::Revenuecat,
                    event_id: ev.id.clone(),
                    payload_digest: digest,
                    writes: vec![],
                },
            )
            .await?;
            Ok(match out {
                IngestOutcome::Duplicate => WebhookOutcome::Ingested(out),
                _ => WebhookOutcome::Skipped(reason),
            })
        }
        Mapped::Writes(writes) => {
            let source = writes[0].source;
            if channel_of(source).is_some_and(|c| !paywall.channel_enabled(c)) {
                return Ok(WebhookOutcome::ChannelDisabled(source));
            }
            Ok(WebhookOutcome::Ingested(
                ingest(
                    svc,
                    ProviderEvent {
                        provider: EventProvider::Revenuecat,
                        event_id: ev.id.clone(),
                        payload_digest: digest,
                        writes,
                    },
                )
                .await?,
            ))
        }
        Mapped::Transfer { source, from, to } => {
            if channel_of(source).is_some_and(|c| !paywall.channel_enabled(c)) {
                return Ok(WebhookOutcome::ChannelDisabled(source));
            }
            // Idempotency gate first; the moves below are idempotent anyway.
            let fresh = svc
                .record_event(EventProvider::Revenuecat, &ev.id, None, &digest)
                .await?;
            if !fresh {
                return Ok(WebhookOutcome::Ingested(IngestOutcome::Duplicate));
            }
            let mut ended = 0u64;
            for uid in from.iter().filter(|u| Uuid::parse_str(u).is_ok()) {
                let rows = svc.rows_for_user(uid).await?;
                for row in rows
                    .iter()
                    .filter(|r| r.source == source && !r.status.is_terminal())
                {
                    svc.apply(EntitlementWrite {
                        user_id: uid.clone(),
                        source,
                        provider_ref: row.provider_ref.clone(),
                        campaign_id: None,
                        starts_at: row.starts_at,
                        ends_at: Some(now),
                        status: EntitlementStatus::Refunded,
                    })
                    .await?;
                    ended += 1;
                }
            }
            let mut resynced = 0u64;
            match customers {
                Some(c) => {
                    for uid in to.iter().filter(|u| Uuid::parse_str(u).is_ok()) {
                        resynced +=
                            sync_customer(svc, c, uid, &products, cfg.allow_sandbox, now).await?;
                    }
                }
                None if !to.is_empty() => tracing::warn!(
                    event = %ev.id,
                    "revenuecat TRANSFER: no customer API configured; destinations left to reconciliation"
                ),
                None => {}
            }
            svc.mark_event_applied(EventProvider::Revenuecat, &ev.id)
                .await?;
            Ok(WebhookOutcome::Transferred { ended, resynced })
        }
    }
}

// ------------------------------------------------------------------- pull

/// Read the account's subscriptions from the aggregator and re-apply them
/// (forward-only). Returns how many rows were written. An aggregator error is
/// returned as is — never turned into a grant.
pub async fn sync_customer(
    svc: &PlanService,
    customers: &dyn StoreCustomerSource,
    user_id: &str,
    premium_products: &[String],
    allow_sandbox: bool,
    now: DateTime<Utc>,
) -> Result<u64> {
    let subs = customers.subscriptions(user_id).await?;
    let writes = map_customer(user_id, &subs, premium_products, now, allow_sandbox);
    let n = writes.len() as u64;
    for w in writes {
        svc.apply(w).await?;
    }
    Ok(n)
}

// --------------------------------------------------- customer API payload

#[derive(Debug, Deserialize)]
pub struct SubscriberResponse {
    pub subscriber: Subscriber,
}

#[derive(Debug, Default, Deserialize)]
pub struct Subscriber {
    #[serde(default)]
    pub subscriptions: BTreeMap<String, SubscriptionV1>,
}

/// One entry of the v1 `subscriber.subscriptions` map (keyed by product id).
#[derive(Debug, Default, Deserialize)]
pub struct SubscriptionV1 {
    pub store: Option<String>,
    pub expires_date: Option<DateTime<Utc>>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub original_purchase_date: Option<DateTime<Utc>>,
    pub unsubscribe_detected_at: Option<DateTime<Utc>>,
    pub billing_issues_detected_at: Option<DateTime<Utc>>,
    pub grace_period_expires_date: Option<DateTime<Utc>>,
    pub refunded_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub is_sandbox: bool,
    pub store_transaction_id: Option<String>,
    pub original_transaction_id: Option<String>,
}

/// Project the v1 subscriber payload onto the neutral shape. The ledger key is
/// the original transaction id when RevenueCat exposes it, else the store
/// transaction id (verified equal to the webhook's `original_transaction_id` in
/// sandbox — design D1).
pub fn project_subscriber(sub: &Subscriber) -> Vec<StoreSubscription> {
    sub.subscriptions
        .iter()
        .map(|(product_id, s)| StoreSubscription {
            store: s.store.clone().unwrap_or_default(),
            product_id: product_id.clone(),
            provider_ref: s
                .original_transaction_id
                .clone()
                .or_else(|| s.store_transaction_id.clone())
                .unwrap_or_default(),
            purchase_at: s.original_purchase_date.or(s.purchase_date),
            expires_at: s.expires_date,
            unsubscribe_detected_at: s.unsubscribe_detected_at,
            billing_issues_detected_at: s.billing_issues_detected_at,
            grace_period_expires_at: s.grace_period_expires_date,
            refunded_at: s.refunded_at,
            is_sandbox: s.is_sandbox,
        })
        .collect()
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::billing::tests::service;
    use crate::model::EntitlementStatus as S;
    use crate::ports::{FixedPaywallConfig, MockStoreCustomerSource};
    use chrono::Duration;

    const U1: &str = "018f0000-0000-7000-8000-000000000001";
    const U2: &str = "018f0000-0000-7000-8000-000000000002";

    fn products() -> Vec<String> {
        vec!["premium_monthly".into(), "premium_yearly".into()]
    }

    fn fixture(name: &str) -> String {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/revenuecat/");
        std::fs::read_to_string(format!("{dir}{name}.json"))
            .unwrap_or_else(|e| panic!("fixture {name}: {e}"))
    }

    fn event(name: &str) -> RcEvent {
        serde_json::from_str::<RcWebhookBody>(&fixture(name))
            .unwrap()
            .event
    }

    fn cfg(allow_sandbox: bool) -> RcConfig {
        RcConfig {
            webhook_secret: "s3cret".into(),
            api_key: "sk_test".into(),
            project_id: None,
            allow_sandbox,
        }
    }

    fn paywall(apple: bool, google: bool) -> FixedPaywallConfig {
        FixedPaywallConfig {
            apple,
            google,
            web: true,
            products: products(),
        }
    }

    /// The fixture events all happen around this instant.
    fn now() -> DateTime<Utc> {
        ms(1_700_000_000_000).unwrap()
    }

    #[test]
    fn store_names_map_in_both_casings_and_unknown_is_none() {
        assert_eq!(store_to_source("APP_STORE"), Some(Source::Apple));
        assert_eq!(store_to_source("mac_app_store"), Some(Source::Apple));
        assert_eq!(store_to_source("PLAY_STORE"), Some(Source::Google));
        assert_eq!(store_to_source("play_store"), Some(Source::Google));
        assert_eq!(store_to_source("PADDLE"), Some(Source::Web));
        for s in [
            "STRIPE",
            "PROMOTIONAL",
            "AMAZON",
            "RC_BILLING",
            "TEST_STORE",
            "",
        ] {
            assert_eq!(store_to_source(s), None, "{s}");
        }
    }

    #[test]
    fn google_provider_refs_are_normalized_to_the_base_order_id() {
        for (raw, want) in [
            ("GPA.3390-0929-2914-92454..0", "GPA.3390-0929-2914-92454"),
            ("GPA.3390-0929-2914-92454..12", "GPA.3390-0929-2914-92454"),
            ("GPA.3390-0929-2914-92454", "GPA.3390-0929-2914-92454"),
            ("GPA.3390..", "GPA.3390.."), // nothing after `..` — untouched
            ("GPA.3390..x1", "GPA.3390..x1"), // non-numeric suffix — untouched
        ] {
            assert_eq!(
                normalize_provider_ref(Source::Google, raw.into()),
                want,
                "{raw}"
            );
        }
        // Only google refs are rewritten.
        assert_eq!(
            normalize_provider_ref(Source::Apple, "otx..0".into()),
            "otx..0"
        );
        // End to end: a Play CANCELLATION carrying the renewal-suffixed order id
        // (observed live in 7.4) keys the same row as the INITIAL_PURCHASE did.
        let ev = RcEvent {
            id: "e-cancel-google".into(),
            kind: "CANCELLATION".into(),
            app_user_id: U1.into(),
            store: Some("PLAY_STORE".into()),
            product_id: Some("premium_monthly:monthly".into()),
            original_transaction_id: Some("GPA.3390-0929-2914-92454..0".into()),
            purchased_at_ms: Some(1_700_000_000_000),
            expiration_at_ms: Some(1_700_000_300_000),
            ..Default::default()
        };
        let Mapped::Writes(ws) = map_event(&ev, &products(), now(), true) else {
            panic!("expected writes");
        };
        assert_eq!(ws[0].provider_ref, "GPA.3390-0929-2914-92454");
        assert_eq!(ws[0].status, S::Cancelled);
    }

    #[test]
    fn map_event_table() {
        let p = products();
        let n = now();
        // (fixture, expected status, ends_at = expiration?) — the D4 table.
        let cases: &[(&str, S, Source)] = &[
            ("initial_purchase", S::Active, Source::Apple),
            ("renewal", S::Active, Source::Apple),
            ("uncancellation", S::Active, Source::Google),
            ("product_change", S::Active, Source::Apple),
            ("cancellation_unsubscribe", S::Cancelled, Source::Google),
            ("expiration", S::Ended, Source::Apple),
            ("subscription_paused", S::Ended, Source::Google),
        ];
        for (name, status, source) in cases {
            let ev = event(name);
            match map_event(&ev, &p, n, true) {
                Mapped::Writes(ws) => {
                    assert_eq!(ws.len(), 1, "{name}");
                    let w = &ws[0];
                    assert_eq!(w.status, *status, "{name}");
                    assert_eq!(w.source, *source, "{name}");
                    assert_eq!(w.user_id, U1, "{name}");
                    assert_eq!(
                        w.ends_at,
                        ev.expiration_at_ms.and_then(ms),
                        "{name}: ends_at = expiration"
                    );
                    assert_eq!(
                        w.provider_ref,
                        ev.original_transaction_id.clone().unwrap(),
                        "{name}"
                    );
                }
                other => panic!("{name}: expected writes, got {other:?}"),
            }
        }
    }

    #[test]
    fn product_change_moves_to_the_new_product_only_if_premium() {
        let ev = event("product_change");
        assert!(matches!(
            map_event(&ev, &products(), now(), true),
            Mapped::Writes(_)
        ));
        // new product outside the premium set ⇒ skip
        let mut ev2 = ev.clone();
        ev2.new_product_id = Some("coins_100".into());
        assert_eq!(
            map_event(&ev2, &products(), now(), true),
            Mapped::Skip(SkipReason::ProductNotPremium("coins_100".into()))
        );
    }

    #[test]
    fn refund_ends_now_and_billing_issue_opens_grace() {
        let n = now();
        let ev = event("cancellation_refund");
        let Mapped::Writes(ws) = map_event(&ev, &products(), n, true) else {
            panic!("refund should write");
        };
        assert_eq!(ws[0].status, S::Refunded);
        assert!(ws[0].ends_at.unwrap() <= n);
        // a CANCELLATION whose expiration is already past also ends now
        let mut past = event("cancellation_unsubscribe");
        past.expiration_at_ms = Some((n - Duration::hours(1)).timestamp_millis());
        let Mapped::Writes(ws) = map_event(&past, &products(), n, true) else {
            panic!()
        };
        assert_eq!(ws[0].status, S::Refunded);
        assert_eq!(ws[0].ends_at, past.expiration_at_ms.and_then(ms));
        // billing issue: grace end governs
        let bi = event("billing_issue");
        let Mapped::Writes(ws) = map_event(&bi, &products(), n, true) else {
            panic!()
        };
        assert_eq!(ws[0].status, S::BillingRetry);
        assert_eq!(ws[0].ends_at, bi.grace_period_expiration_at_ms.and_then(ms));
        assert!(ws[0].ends_at > bi.expiration_at_ms.and_then(ms));
    }

    #[test]
    fn skips_are_typed() {
        let p = products();
        let n = now();
        assert_eq!(
            map_event(&event("test"), &p, n, true),
            Mapped::Skip(SkipReason::Informational("TEST".into()))
        );
        for t in [
            "NON_RENEWING_PURCHASE",
            "INVOICE_ISSUANCE",
            "VIRTUAL_CURRENCY_TRANSACTION",
            "PAYWALL_IMPRESSION",
            "EXPERIMENT_ENROLLMENT",
            "PURCHASE_REDEEMED",
            "PRICE_INCREASE_CONSENT_REQUIRED",
            "SUBSCRIBER_ALIAS",
            "SOMETHING_NEW",
        ] {
            let mut ev = event("renewal");
            ev.kind = t.into();
            assert_eq!(
                map_event(&ev, &p, n, true),
                Mapped::Skip(SkipReason::Informational(t.into())),
                "{t}"
            );
        }
        // sandbox in production
        assert_eq!(
            map_event(&event("initial_purchase"), &p, n, false),
            Mapped::Skip(SkipReason::Sandbox)
        );
        // unmapped store
        let mut ev = event("renewal");
        ev.store = Some("STRIPE".into());
        assert_eq!(
            map_event(&ev, &p, n, true),
            Mapped::Skip(SkipReason::UnmappedStore("STRIPE".into()))
        );
        // anonymous / malformed user id
        let mut ev = event("renewal");
        ev.app_user_id = "$RCAnonymousID:abc".into();
        assert_eq!(
            map_event(&ev, &p, n, true),
            Mapped::Skip(SkipReason::MalformedUser)
        );
        // non-premium product
        let mut ev = event("renewal");
        ev.product_id = Some("coins_100".into());
        assert_eq!(
            map_event(&ev, &p, n, true),
            Mapped::Skip(SkipReason::ProductNotPremium("coins_100".into()))
        );
        // missing expiration
        let mut ev = event("renewal");
        ev.expiration_at_ms = None;
        assert_eq!(
            map_event(&ev, &p, n, true),
            Mapped::Skip(SkipReason::MissingFields("expiration_at_ms"))
        );
        // empty product set = any product
        let ev = event("renewal");
        assert!(matches!(map_event(&ev, &[], n, true), Mapped::Writes(_)));
    }

    #[test]
    fn play_base_plan_suffix_still_matches_the_premium_set() {
        // RevenueCat reports Play base-plan subscriptions as `id:base_plan`.
        let mut ev = event("uncancellation"); // PLAY_STORE fixture
        ev.product_id = Some("premium_yearly:yearly".into());
        assert!(matches!(
            map_event(&ev, &products(), now(), true),
            Mapped::Writes(_)
        ));
        ev.product_id = Some("coins_100:monthly".into());
        assert_eq!(
            map_event(&ev, &products(), now(), true),
            Mapped::Skip(SkipReason::ProductNotPremium("coins_100:monthly".into()))
        );
    }

    #[test]
    fn transfer_maps_to_a_move() {
        let ev = event("transfer");
        assert_eq!(
            map_event(&ev, &products(), now(), true),
            Mapped::Transfer {
                source: Source::Apple,
                from: vec![U1.into()],
                to: vec![U2.into()],
            }
        );
    }

    #[test]
    fn customer_projection_and_key_consistency_with_the_webhook() {
        // The v1 subscriber fixture describes the same subscription as
        // `renewal.json`: the ledger key and the state must agree (D1).
        let body: SubscriberResponse = serde_json::from_str(&fixture("subscriber_v1")).unwrap();
        let subs = project_subscriber(&body.subscriber);
        assert_eq!(subs.len(), 2);
        let n = now();
        let ws = map_customer(U1, &subs, &products(), n, true);
        // the play_store row is not premium (coins) ⇒ dropped
        assert_eq!(ws.len(), 1);
        let renewal = event("renewal");
        let Mapped::Writes(from_hook) = map_event(&renewal, &products(), n, true) else {
            panic!()
        };
        assert_eq!(ws[0].source, from_hook[0].source);
        assert_eq!(ws[0].provider_ref, from_hook[0].provider_ref);
        assert_eq!(ws[0].ends_at, from_hook[0].ends_at);
        assert_eq!(ws[0].status, from_hook[0].status);
        assert_eq!(ws[0].user_id, U1);
    }

    #[test]
    fn customer_states() {
        let n = now();
        let base = StoreSubscription {
            store: "APP_STORE".into(),
            product_id: "premium_monthly".into(),
            provider_ref: "otx-1".into(),
            purchase_at: Some(n - Duration::days(10)),
            expires_at: Some(n + Duration::days(20)),
            unsubscribe_detected_at: None,
            billing_issues_detected_at: None,
            grace_period_expires_at: None,
            refunded_at: None,
            is_sandbox: false,
        };
        let one = |s: StoreSubscription| map_customer(U1, &[s], &products(), n, false);
        assert_eq!(one(base.clone())[0].status, S::Active);
        assert_eq!(
            one(StoreSubscription {
                unsubscribe_detected_at: Some(n),
                ..base.clone()
            })[0]
                .status,
            S::Cancelled
        );
        let refunded = one(StoreSubscription {
            refunded_at: Some(n - Duration::hours(1)),
            ..base.clone()
        });
        assert_eq!(refunded[0].status, S::Refunded);
        assert_eq!(refunded[0].ends_at, Some(n - Duration::hours(1)));
        let ended = one(StoreSubscription {
            expires_at: Some(n - Duration::days(1)),
            ..base.clone()
        });
        assert_eq!(ended[0].status, S::Ended);
        let grace = one(StoreSubscription {
            expires_at: Some(n - Duration::days(1)),
            billing_issues_detected_at: Some(n - Duration::days(1)),
            grace_period_expires_at: Some(n + Duration::days(15)),
            ..base.clone()
        });
        assert_eq!(grace[0].status, S::BillingRetry);
        assert_eq!(grace[0].ends_at, Some(n + Duration::days(15)));
        // sandbox in production, unmapped store, missing ref ⇒ dropped
        assert!(
            one(StoreSubscription {
                is_sandbox: true,
                ..base.clone()
            })
            .is_empty()
        );
        assert!(
            one(StoreSubscription {
                store: "STRIPE".into(),
                ..base.clone()
            })
            .is_empty()
        );
        assert!(
            one(StoreSubscription {
                provider_ref: String::new(),
                ..base
            })
            .is_empty()
        );
    }

    #[test]
    fn authentication_is_exact() {
        let c = cfg(true);
        assert!(authenticate(&c, Some("s3cret")).is_ok());
        for bad in [
            None,
            Some(""),
            Some("s3cre"),
            Some("s3cret "),
            Some("S3CRET"),
        ] {
            assert!(matches!(
                authenticate(&c, bad),
                Err(AppError::Unauthenticated(_))
            ));
        }
    }

    #[tokio::test]
    async fn webhook_applies_once_and_replays_are_duplicates() {
        let (svc, writes, _) = service();
        let body = fixture("initial_purchase");
        let pw = paywall(true, true);
        let out = handle_webhook(
            &svc,
            &cfg(true),
            &pw,
            None,
            Some("s3cret"),
            body.as_bytes(),
            now(),
        )
        .await
        .unwrap();
        assert_eq!(out, WebhookOutcome::Ingested(IngestOutcome::Applied));
        {
            let ws = writes.lock().unwrap();
            assert_eq!(ws.len(), 1);
            assert_eq!(ws[0].source, Source::Apple);
            assert_eq!(ws[0].status, S::Active);
            assert_eq!(ws[0].user_id, U1);
        }
        let again = handle_webhook(
            &svc,
            &cfg(true),
            &pw,
            None,
            Some("s3cret"),
            body.as_bytes(),
            now(),
        )
        .await
        .unwrap();
        assert_eq!(again, WebhookOutcome::Ingested(IngestOutcome::Duplicate));
        assert_eq!(writes.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn webhook_refuses_bad_auth_without_side_effect() {
        let (svc, writes, events) = service();
        let body = fixture("initial_purchase");
        let pw = paywall(true, true);
        let r = handle_webhook(
            &svc,
            &cfg(true),
            &pw,
            None,
            Some("wrong"),
            body.as_bytes(),
            now(),
        )
        .await;
        assert!(matches!(r, Err(AppError::Unauthenticated(_))));
        assert!(writes.lock().unwrap().is_empty());
        assert!(events.lock().unwrap().is_empty());
        // garbage body after good auth: invalid argument, nothing recorded
        let r = handle_webhook(
            &svc,
            &cfg(true),
            &pw,
            None,
            Some("s3cret"),
            b"not json",
            now(),
        )
        .await;
        assert!(matches!(r, Err(AppError::InvalidArgument(_))));
        assert!(events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn webhook_disabled_channel_acknowledges_and_ignores() {
        let (svc, writes, events) = service();
        let pw = paywall(true, false); // google off
        let out = handle_webhook(
            &svc,
            &cfg(true),
            &pw,
            None,
            Some("s3cret"),
            fixture("uncancellation").as_bytes(), // PLAY_STORE
            now(),
        )
        .await
        .unwrap();
        assert_eq!(out, WebhookOutcome::ChannelDisabled(Source::Google));
        assert!(writes.lock().unwrap().is_empty());
        // not recorded either: once the channel opens, reconciliation re-reads
        assert!(events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn webhook_skips_are_recorded_no_ops() {
        let (svc, writes, events) = service();
        let pw = paywall(true, true);
        let out = handle_webhook(
            &svc,
            &cfg(true),
            &pw,
            None,
            Some("s3cret"),
            fixture("test").as_bytes(),
            now(),
        )
        .await
        .unwrap();
        assert_eq!(
            out,
            WebhookOutcome::Skipped(SkipReason::Informational("TEST".into()))
        );
        assert!(writes.lock().unwrap().is_empty());
        assert_eq!(events.lock().unwrap().len(), 1);
        // sandbox event in production: skipped, recorded
        let out = handle_webhook(
            &svc,
            &cfg(false),
            &pw,
            None,
            Some("s3cret"),
            fixture("renewal").as_bytes(),
            now(),
        )
        .await
        .unwrap();
        assert_eq!(out, WebhookOutcome::Skipped(SkipReason::Sandbox));
        assert!(writes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn webhook_refund_and_transfer() {
        let (svc, writes, _) = service();
        let pw = paywall(true, true);
        let c = cfg(true);
        // purchase then refund on the same subscription
        handle_webhook(
            &svc,
            &c,
            &pw,
            None,
            Some("s3cret"),
            fixture("initial_purchase").as_bytes(),
            now(),
        )
        .await
        .unwrap();
        let out = handle_webhook(
            &svc,
            &c,
            &pw,
            None,
            Some("s3cret"),
            fixture("cancellation_refund").as_bytes(),
            now(),
        )
        .await
        .unwrap();
        assert_eq!(out, WebhookOutcome::Ingested(IngestOutcome::Applied));
        {
            let ws = writes.lock().unwrap();
            assert_eq!(ws[1].status, S::Refunded);
            assert_eq!(ws[1].provider_ref, ws[0].provider_ref);
        }
        // transfer U1 → U2: U1's apple rows end; U2 is re-read from the customer API
        let mut customers = MockStoreCustomerSource::new();
        customers.expect_subscriptions().returning(|_| {
            Ok(vec![StoreSubscription {
                store: "APP_STORE".into(),
                product_id: "premium_monthly".into(),
                provider_ref: "otx-1001".into(),
                purchase_at: None,
                expires_at: Some(now() + Duration::days(30)),
                unsubscribe_detected_at: None,
                billing_issues_detected_at: None,
                grace_period_expires_at: None,
                refunded_at: None,
                is_sandbox: true,
            }])
        });
        let out = handle_webhook(
            &svc,
            &c,
            &pw,
            Some(&customers),
            Some("s3cret"),
            fixture("transfer").as_bytes(),
            now(),
        )
        .await
        .unwrap();
        // U1's rows are already terminal (refunded) ⇒ nothing more to end
        assert_eq!(
            out,
            WebhookOutcome::Transferred {
                ended: 0,
                resynced: 1
            }
        );
        {
            let ws = writes.lock().unwrap();
            let last = ws.last().unwrap();
            assert_eq!(last.user_id, U2);
            assert_eq!(last.status, S::Active);
        }
        // replayed transfer is a duplicate
        let out = handle_webhook(
            &svc,
            &c,
            &pw,
            Some(&customers),
            Some("s3cret"),
            fixture("transfer").as_bytes(),
            now(),
        )
        .await
        .unwrap();
        assert_eq!(out, WebhookOutcome::Ingested(IngestOutcome::Duplicate));
    }

    #[tokio::test]
    async fn sync_customer_writes_only_mapped_rows_and_propagates_errors() {
        let (svc, writes, _) = service();
        let mut customers = MockStoreCustomerSource::new();
        customers
            .expect_subscriptions()
            .times(1)
            .returning(|_| Ok(vec![]));
        assert_eq!(
            sync_customer(&svc, &customers, U1, &products(), true, now())
                .await
                .unwrap(),
            0
        );
        let mut failing = MockStoreCustomerSource::new();
        failing
            .expect_subscriptions()
            .returning(|_| Err(AppError::Internal(anyhow::anyhow!("down"))));
        assert!(matches!(
            sync_customer(&svc, &failing, U1, &products(), true, now()).await,
            Err(AppError::Internal(_))
        ));
        assert!(writes.lock().unwrap().is_empty());
        let mut ok = MockStoreCustomerSource::new();
        ok.expect_subscriptions().returning(|_| {
            Ok(vec![StoreSubscription {
                store: "PLAY_STORE".into(),
                product_id: "premium_yearly".into(),
                provider_ref: "GPA.1".into(),
                purchase_at: None,
                expires_at: Some(now() + Duration::days(300)),
                unsubscribe_detected_at: None,
                billing_issues_detected_at: None,
                grace_period_expires_at: None,
                refunded_at: None,
                is_sandbox: false,
            }])
        });
        assert_eq!(
            sync_customer(&svc, &ok, U1, &products(), false, now())
                .await
                .unwrap(),
            1
        );
        assert_eq!(writes.lock().unwrap()[0].source, Source::Google);
    }
}
