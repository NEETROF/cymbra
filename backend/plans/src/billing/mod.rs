//! Purchase-channel adapters: the store aggregator (RevenueCat — App Store,
//! Play, later Paddle; change: swap-store-billing-to-revenuecat) and the direct
//! web merchant-of-record (Paddle; add-premium-subscription D9). Each turns a
//! provider's authenticated events into [`EntitlementWrite`]s and hands them to
//! [`PlanService::apply`] behind the idempotency gate. The pure parts — payload →
//! transition mappers — are host-tested; the HTTP/webhook glue is thin.
//!
//! Shared rules: a provider event is applied at most once (`billing_events`),
//! an event that cannot be mapped to a user is acknowledged and logged (never
//! retried forever), and a disabled channel acknowledges without applying.

pub mod env;
pub mod rc_client;
pub mod reconcile;
pub mod revenuecat;
pub mod web;

use crate::model::EventProvider;
use crate::ports::EntitlementWrite;
use crate::service::PlanService;
use cymbra_platform::Result;
use sha2::{Digest, Sha256};

/// A provider notification reduced to what the ledger needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEvent {
    /// Who delivered it — the `billing_events` idempotency scope.
    pub provider: EventProvider,
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
            event.provider,
            &event.event_id,
            user_hint.as_deref(),
            &event.payload_digest,
        )
        .await?;
    if !fresh {
        return Ok(IngestOutcome::Duplicate);
    }
    if event.writes.is_empty() {
        svc.mark_event_applied(event.provider, &event.event_id)
            .await?;
        return Ok(IngestOutcome::NoOp);
    }
    for w in event.writes {
        svc.apply(w).await?;
    }
    svc.mark_event_applied(event.provider, &event.event_id)
        .await?;
    Ok(IngestOutcome::Applied)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::{EntitlementRow, EntitlementStatus, Source};
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
    pub(crate) type Ledger = Arc<Mutex<Vec<EntitlementWrite>>>;
    pub(crate) type EventLog = Arc<Mutex<Vec<String>>>;

    fn row_of(w: &EntitlementWrite) -> EntitlementRow {
        EntitlementRow {
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
        }
    }

    /// The "table" view of the write log: the latest write per
    /// `(source, provider_ref)` wins, like the real upsert.
    fn table(writes: &[EntitlementWrite]) -> Vec<EntitlementRow> {
        let mut latest: Vec<EntitlementWrite> = Vec::new();
        for w in writes {
            if let Some(slot) = latest
                .iter_mut()
                .find(|x| x.source == w.source && x.provider_ref == w.provider_ref)
            {
                *slot = w.clone();
            } else {
                latest.push(w.clone());
            }
        }
        latest.iter().map(row_of).collect()
    }

    pub(crate) fn service() -> (PlanService, Ledger, EventLog) {
        let writes = Arc::new(Mutex::new(Vec::<EntitlementWrite>::new()));
        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        let mut ent = MockEntitlementRepo::new();
        let w1 = writes.clone();
        ent.expect_upsert().returning(move |w| {
            w1.lock().unwrap().push(w.clone());
            Ok(row_of(&w))
        });
        let w2 = writes.clone();
        ent.expect_find_by_provider_ref().returning(move |src, r| {
            Ok(table(&w2.lock().unwrap())
                .into_iter()
                .find(|w| w.source == src && w.provider_ref == r))
        });
        let w3 = writes.clone();
        ent.expect_list_for_user().returning(move |u| {
            Ok(table(&w3.lock().unwrap())
                .into_iter()
                .filter(|w| w.user_id == u)
                .collect())
        });
        ent.expect_mark_withdrawn().returning(|_, _| Ok(0));
        let w4 = writes.clone();
        ent.expect_list_ending_between()
            .returning(move |from, to, sources| {
                Ok(table(&w4.lock().unwrap())
                    .into_iter()
                    .filter(|w| sources.contains(&w.source))
                    .filter(|w| w.ends_at.is_some_and(|e| e >= from && e < to))
                    .collect())
            });
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
            provider: EventProvider::Web,
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
            provider: EventProvider::Web,
            event_id: "evt_2".into(),
            payload_digest: "d".into(),
            writes: vec![],
        };
        assert_eq!(ingest(&svc, empty).await.unwrap(), IngestOutcome::NoOp);
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
