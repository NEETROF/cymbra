//! Purchase-channel adapters (design D7–D9): each turns a provider's signed
//! events into [`EntitlementWrite`]s and hands them to
//! [`PlanService::apply`] behind the idempotency gate. The pure parts —
//! signature verification cores, payload → transition mappers — are host-tested;
//! the HTTP/webhook glue is thin.
//!
//! Shared rules: a provider event is applied at most once (`billing_events`),
//! an event that cannot be mapped to a user is acknowledged and logged (never
//! retried forever), and a disabled channel acknowledges without applying.

pub mod apple;
pub mod env;
pub mod google;
pub mod reconcile;
pub mod web;

use crate::model::Source;
use crate::ports::EntitlementWrite;
use crate::service::PlanService;
use cymbra_platform::Result;
use sha2::{Digest, Sha256};

/// A provider notification reduced to what the ledger needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEvent {
    pub source: Source,
    /// The provider's event id (idempotency key).
    pub event_id: String,
    /// Short digest of the raw payload (never the payload itself).
    pub payload_digest: String,
    /// The ledger writes; empty when the event carries no plan change.
    pub writes: Vec<EntitlementWrite>,
}

/// What happened to an ingested event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestOutcome {
    /// New event, writes applied.
    Applied,
    /// Already seen — nothing done.
    Duplicate,
    /// New event but nothing to write (informational, or unmapped user).
    NoOp,
}

/// SHA-256 hex of a raw payload — what `billing_events.payload_ref` stores.
pub fn payload_digest(raw: &[u8]) -> String {
    Sha256::digest(raw)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Apply an event exactly once: record → apply writes → mark applied.
pub async fn ingest(svc: &PlanService, event: ProviderEvent) -> Result<IngestOutcome> {
    let user_hint = event.writes.first().map(|w| w.user_id.clone());
    let fresh = svc
        .record_event(
            event.source,
            &event.event_id,
            user_hint.as_deref(),
            &event.payload_digest,
        )
        .await?;
    if !fresh {
        return Ok(IngestOutcome::Duplicate);
    }
    if event.writes.is_empty() {
        svc.mark_event_applied(event.source, &event.event_id)
            .await?;
        return Ok(IngestOutcome::NoOp);
    }
    for w in event.writes {
        svc.apply(w).await?;
    }
    svc.mark_event_applied(event.source, &event.event_id)
        .await?;
    Ok(IngestOutcome::Applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EntitlementRow, EntitlementStatus};
    use crate::ports::{
        MockAccessCodeRepo, MockAuditRepo, MockBillingEventRepo, MockCampaignRepo, MockClock,
        MockEntitlementRepo, MockMembershipRepo, MockPlanConfigSource, PlanConfig,
    };
    use crate::service::PlanDeps;
    use chrono::{Duration, Utc};
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    /// A service whose ledger is an in-memory map keyed by (source, provider_ref)
    /// and whose event log de-duplicates by (provider, event_id).
    type Ledger = Arc<Mutex<Vec<EntitlementWrite>>>;
    type EventLog = Arc<Mutex<Vec<String>>>;

    fn service() -> (PlanService, Ledger, EventLog) {
        let writes = Arc::new(Mutex::new(Vec::<EntitlementWrite>::new()));
        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        let mut ent = MockEntitlementRepo::new();
        let w1 = writes.clone();
        ent.expect_upsert().returning(move |w| {
            w1.lock().unwrap().push(w.clone());
            Ok(EntitlementRow {
                id: Uuid::new_v4(),
                user_id: w.user_id,
                source: w.source,
                provider_ref: w.provider_ref,
                campaign_id: None,
                starts_at: w.starts_at,
                ends_at: w.ends_at,
                status: w.status,
                revoked_at: None,
                withdrawn_at: None,
            })
        });
        let w2 = writes.clone();
        ent.expect_find_by_provider_ref().returning(move |src, r| {
            Ok(w2
                .lock()
                .unwrap()
                .iter()
                .find(|w| w.source == src && w.provider_ref == r)
                .map(|w| EntitlementRow {
                    id: Uuid::new_v4(),
                    user_id: w.user_id.clone(),
                    source: w.source,
                    provider_ref: w.provider_ref.clone(),
                    campaign_id: None,
                    starts_at: w.starts_at,
                    ends_at: w.ends_at,
                    status: w.status,
                    revoked_at: None,
                    withdrawn_at: None,
                }))
        });
        let w3 = writes.clone();
        ent.expect_list_for_user().returning(move |u| {
            Ok(w3
                .lock()
                .unwrap()
                .iter()
                .filter(|w| w.user_id == u)
                .map(|w| EntitlementRow {
                    id: Uuid::new_v4(),
                    user_id: w.user_id.clone(),
                    source: w.source,
                    provider_ref: w.provider_ref.clone(),
                    campaign_id: None,
                    starts_at: w.starts_at,
                    ends_at: w.ends_at,
                    status: w.status,
                    revoked_at: None,
                    withdrawn_at: None,
                })
                .collect())
        });
        ent.expect_mark_withdrawn().returning(|_, _| Ok(0));
        let mut billing = MockBillingEventRepo::new();
        let e1 = events.clone();
        billing
            .expect_record_if_new()
            .returning(move |p, id, _, _| {
                let key = format!("{}:{id}", p.as_str());
                let mut g = e1.lock().unwrap();
                if g.contains(&key) {
                    Ok(false)
                } else {
                    g.push(key);
                    Ok(true)
                }
            });
        billing.expect_mark_applied().returning(|_, _| Ok(()));
        let mut config = MockPlanConfigSource::new();
        config.expect_plan_config().returning(|| PlanConfig {
            enabled: true,
            grace_days: 3,
        });
        let mut clock = MockClock::new();
        clock.expect_now().returning(Utc::now);
        let mut audit = MockAuditRepo::new();
        audit.expect_record().returning(|_| Ok(()));
        let svc = PlanService::new(PlanDeps {
            entitlements: Arc::new(ent),
            campaigns: Arc::new(MockCampaignRepo::new()),
            memberships: Arc::new(MockMembershipRepo::new()),
            codes: Arc::new(MockAccessCodeRepo::new()),
            billing_events: Arc::new(billing),
            audit: Arc::new(audit),
            config: Arc::new(config),
            clock: Arc::new(clock),
            rotator: None,
        });
        (svc, writes, events)
    }

    #[tokio::test]
    async fn ingest_applies_once_and_replays_are_no_ops() {
        let (svc, writes, _) = service();
        let ev = ProviderEvent {
            source: Source::Web,
            event_id: "evt_1".into(),
            payload_digest: "d".into(),
            writes: vec![EntitlementWrite {
                user_id: "u1".into(),
                source: Source::Web,
                provider_ref: "sub_1".into(),
                campaign_id: None,
                starts_at: Utc::now(),
                ends_at: Some(Utc::now() + Duration::days(30)),
                status: EntitlementStatus::Active,
            }],
        };
        assert_eq!(
            ingest(&svc, ev.clone()).await.unwrap(),
            IngestOutcome::Applied
        );
        assert_eq!(ingest(&svc, ev).await.unwrap(), IngestOutcome::Duplicate);
        assert_eq!(writes.lock().unwrap().len(), 1);
        let empty = ProviderEvent {
            source: Source::Web,
            event_id: "evt_2".into(),
            payload_digest: "d".into(),
            writes: vec![],
        };
        assert_eq!(ingest(&svc, empty).await.unwrap(), IngestOutcome::NoOp);
    }

    #[tokio::test]
    async fn apple_notification_is_verified_mapped_and_idempotent() {
        use crate::billing::apple::{AppleConfig, handle_notification, tests::*};
        let (svc, writes, _) = service();
        let (root, leaf, key) = test_chain();
        let mut cfg = AppleConfig::new("com.cymbra.music", true);
        cfg.roots = vec![root.clone()];
        let now = Utc::now();
        let tx = serde_json::json!({
            "originalTransactionId": "otx-77",
            "productId": "premium_monthly",
            "bundleId": "com.cymbra.music",
            "environment": "Sandbox",
            "purchaseDate": (now - Duration::days(1)).timestamp_millis(),
            "expiresDate": (now + Duration::days(29)).timestamp_millis(),
            "appAccountToken": "u1",
            "type": "Auto-Renewable Subscription",
        });
        let tx_jws = sign_jws(&tx, &[&leaf, &root], &key);
        let renewal_jws = sign_jws(
            &serde_json::json!({"autoRenewStatus": 1}),
            &[&leaf, &root],
            &key,
        );
        let payload = serde_json::json!({
            "notificationType": "DID_RENEW",
            "notificationUUID": "n-1",
            "data": {
                "bundleId": "com.cymbra.music",
                "environment": "Sandbox",
                "signedTransactionInfo": tx_jws,
                "signedRenewalInfo": renewal_jws,
            }
        });
        let signed = sign_jws(&payload, &[&leaf, &root], &key);
        let body = serde_json::to_vec(&serde_json::json!({"signedPayload": signed})).unwrap();
        assert_eq!(
            handle_notification(&svc, &cfg, &body, now).await.unwrap(),
            IngestOutcome::Applied
        );
        assert_eq!(writes.lock().unwrap()[0].provider_ref, "otx-77");
        assert_eq!(writes.lock().unwrap()[0].status, EntitlementStatus::Active);
        // replay
        assert_eq!(
            handle_notification(&svc, &cfg, &body, now).await.unwrap(),
            IngestOutcome::Duplicate
        );
        // a REFUND for the same subscription WITHOUT the account token resolves
        // the user through the existing row
        let tx2 = serde_json::json!({
            "originalTransactionId": "otx-77",
            "productId": "premium_monthly",
            "bundleId": "com.cymbra.music",
            "environment": "Sandbox",
            "expiresDate": (now + Duration::days(29)).timestamp_millis(),
            "revocationDate": now.timestamp_millis(),
            "type": "Auto-Renewable Subscription",
        });
        let payload2 = serde_json::json!({
            "notificationType": "REFUND",
            "notificationUUID": "n-2",
            "data": { "bundleId": "com.cymbra.music", "environment": "Sandbox",
                      "signedTransactionInfo": sign_jws(&tx2, &[&leaf, &root], &key) }
        });
        let body2 = serde_json::to_vec(&serde_json::json!({
            "signedPayload": sign_jws(&payload2, &[&leaf, &root], &key)
        }))
        .unwrap();
        assert_eq!(
            handle_notification(&svc, &cfg, &body2, now).await.unwrap(),
            IngestOutcome::Applied
        );
        assert_eq!(
            writes.lock().unwrap()[1].status,
            EntitlementStatus::Refunded
        );
        assert_eq!(writes.lock().unwrap()[1].user_id, "u1");
        // an unsigned / tampered body is refused with no side effect
        let bad = serde_json::to_vec(&serde_json::json!({"signedPayload": "a.b.c"})).unwrap();
        assert!(handle_notification(&svc, &cfg, &bad, now).await.is_err());
        assert_eq!(writes.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn google_rtdn_rereads_state_and_is_idempotent() {
        use crate::billing::google::{
            ExternalAccountIdentifiers, LineItem, MockPlayApi, SubscriptionV2, handle_rtdn,
        };
        use base64::Engine as _;
        let (svc, writes, _) = service();
        let mut api = MockPlayApi::new();
        api.expect_get_subscription().returning(|_| {
            Ok(SubscriptionV2 {
                subscription_state: Some("SUBSCRIPTION_STATE_ACTIVE".into()),
                start_time: Some(Utc::now()),
                line_items: vec![LineItem {
                    product_id: Some("premium_monthly".into()),
                    expiry_time: Some(Utc::now() + Duration::days(30)),
                }],
                linked_purchase_token: Some("old-tok".into()),
                acknowledgement_state: None,
                external_account_identifiers: Some(ExternalAccountIdentifiers {
                    obfuscated_external_account_id: Some("u1".into()),
                }),
                canceled_state_context: None,
            })
        });
        let data = serde_json::json!({
            "version": "1.0",
            "packageName": "com.cymbra.music",
            "subscriptionNotification": { "version": "1.0", "notificationType": 2, "purchaseToken": "new-tok", "subscriptionId": "premium_monthly" }
        });
        let body = serde_json::to_vec(&serde_json::json!({
            "message": {
                "data": base64::engine::general_purpose::STANDARD.encode(serde_json::to_vec(&data).unwrap()),
                "messageId": "m-1"
            },
            "subscription": "projects/x/subscriptions/y"
        }))
        .unwrap();
        let now = Utc::now();
        assert_eq!(
            handle_rtdn(&svc, &api, "com.cymbra.music", &body, now)
                .await
                .unwrap(),
            IngestOutcome::Applied
        );
        // the superseded token is ended, the new one active
        {
            let ws = writes.lock().unwrap();
            assert_eq!(ws.len(), 2);
            assert_eq!(ws[0].provider_ref, "old-tok");
            assert_eq!(ws[0].status, EntitlementStatus::Ended);
            assert_eq!(ws[1].provider_ref, "new-tok");
            assert_eq!(ws[1].status, EntitlementStatus::Active);
        }
        assert_eq!(
            handle_rtdn(&svc, &api, "com.cymbra.music", &body, now)
                .await
                .unwrap(),
            IngestOutcome::Duplicate
        );
        assert!(
            handle_rtdn(&svc, &api, "com.other", &body, now)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn web_webhook_maps_and_refunds_resolve_the_user() {
        use crate::billing::web::handle_webhook;
        let (svc, writes, _) = service();
        let now = Utc::now();
        let created = serde_json::json!({
            "event_id": "evt_a",
            "event_type": "subscription.activated",
            "data": {
                "id": "sub_9", "status": "active", "customer_id": "ctm_1",
                "started_at": now.to_rfc3339(),
                "current_billing_period": { "starts_at": now.to_rfc3339(), "ends_at": (now + Duration::days(30)).to_rfc3339() },
                "custom_data": { "user_id": "u1" }
            }
        });
        let body = serde_json::to_vec(&created).unwrap();
        assert_eq!(
            handle_webhook(&svc, &body, now).await.unwrap(),
            IngestOutcome::Applied
        );
        assert_eq!(
            handle_webhook(&svc, &body, now).await.unwrap(),
            IngestOutcome::Duplicate
        );
        let refund = serde_json::json!({
            "event_id": "evt_b",
            "event_type": "adjustment.created",
            "data": { "action": "refund", "subscription_id": "sub_9" }
        });
        assert_eq!(
            handle_webhook(&svc, &serde_json::to_vec(&refund).unwrap(), now)
                .await
                .unwrap(),
            IngestOutcome::Applied
        );
        {
            let ws = writes.lock().unwrap();
            assert_eq!(ws[1].status, EntitlementStatus::Refunded);
            assert_eq!(ws[1].user_id, "u1");
        }
        // unrelated event: acknowledged as a no-op
        let other =
            serde_json::json!({"event_id": "evt_c", "event_type": "transaction.paid", "data": {}});
        assert_eq!(
            handle_webhook(&svc, &serde_json::to_vec(&other).unwrap(), now)
                .await
                .unwrap(),
            IngestOutcome::NoOp
        );
    }

    #[test]
    fn digest_is_hex_sha256() {
        let d = payload_digest(b"hello");
        assert_eq!(d.len(), 64);
        assert_eq!(
            d,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
