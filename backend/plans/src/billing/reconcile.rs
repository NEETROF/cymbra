//! Reconciliation (spec `music-subscription-billing`, "Restore and reconciliation
//! keep the ledger truthful"): for paid rows nearing their end, re-read the
//! account's subscriptions from the store aggregator and re-apply them, so a
//! missed notification never leaves a paying user degraded nor a refunded user
//! active. One code path with the app-triggered plan sync
//! ([`crate::billing::revenuecat::sync_customer`]). Runs daily from the worker,
//! before the withdrawal sweep.

use crate::billing::revenuecat::sync_customer;
use crate::model::Source;
use crate::ports::{PaywallConfigSource, StoreCustomerSource};
use crate::service::PlanService;
use chrono::{DateTime, Duration, Utc};
use cymbra_platform::Result;
use std::collections::BTreeSet;
use std::sync::Arc;

/// What the sweep needs; `customers = None` makes it inert (aggregator not
/// configured).
#[derive(Default)]
pub struct Reconciler {
    pub customers: Option<Arc<dyn StoreCustomerSource>>,
    /// Apply `SANDBOX` subscriptions (staging only).
    pub allow_sandbox: bool,
}

/// Re-read every account holding a store row (`apple` / `google` / `web`) that
/// ends within `horizon` and re-apply the aggregator's current state
/// (forward-only). Per-account failures are logged and skipped (the next run
/// retries); returns how many rows were re-applied.
pub async fn reconcile(
    svc: &PlanService,
    r: &Reconciler,
    paywall: &dyn PaywallConfigSource,
    now: DateTime<Utc>,
    horizon: Duration,
) -> Result<u64> {
    let Some(customers) = r.customers.as_ref() else {
        return Ok(0);
    };
    let rows = svc
        .rows_ending_between(now - Duration::days(1), now + horizon)
        .await?;
    // One read per account, whatever the number of rows.
    let users: BTreeSet<String> = rows
        .into_iter()
        .filter(|row| matches!(row.source, Source::Apple | Source::Google | Source::Web))
        .map(|row| row.user_id)
        .collect();
    let products = paywall.products();
    let mut applied = 0u64;
    for uid in users {
        match sync_customer(
            svc,
            customers.as_ref(),
            &uid,
            &products,
            r.allow_sandbox,
            now,
        )
        .await
        {
            Ok(n) => applied += n,
            Err(e) => tracing::warn!(
                error = %e,
                user_id = %uid,
                "reconciliation read failed; will retry next run"
            ),
        }
    }
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::billing::tests::service;
    use crate::model::EntitlementStatus;
    use crate::ports::{
        EntitlementWrite, FixedPaywallConfig, MockStoreCustomerSource, StoreSubscription,
    };
    use cymbra_platform::AppError;

    const U1: &str = "018f0000-0000-7000-8000-000000000001";

    fn paywall() -> FixedPaywallConfig {
        FixedPaywallConfig {
            apple: true,
            google: true,
            web: true,
            products: vec!["premium_monthly".into()],
        }
    }

    async fn seed(svc: &PlanService, now: DateTime<Utc>, ends_in: Duration) {
        svc.apply(EntitlementWrite {
            user_id: U1.into(),
            source: Source::Apple,
            provider_ref: "otx-1".into(),
            campaign_id: None,
            starts_at: now - Duration::days(29),
            ends_at: Some(now + ends_in),
            status: EntitlementStatus::Active,
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn inert_without_customers() {
        let (svc, _, _) = service();
        let n = reconcile(
            &svc,
            &Reconciler::default(),
            &paywall(),
            Utc::now(),
            Duration::days(3),
        )
        .await
        .unwrap();
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn missed_renewal_is_repaired_and_refund_is_caught() {
        let (svc, writes, _) = service();
        let now = Utc::now();
        seed(&svc, now, Duration::days(1)).await;
        let mut customers = MockStoreCustomerSource::new();
        customers
            .expect_subscriptions()
            .withf(|u| u == U1)
            .times(1)
            .returning(move |_| {
                Ok(vec![StoreSubscription {
                    store: "APP_STORE".into(),
                    product_id: "premium_monthly".into(),
                    provider_ref: "otx-1".into(),
                    purchase_at: Some(now - Duration::days(29)),
                    // renewed: a month further
                    expires_at: Some(now + Duration::days(31)),
                    unsubscribe_detected_at: None,
                    billing_issues_detected_at: None,
                    grace_period_expires_at: None,
                    refunded_at: None,
                    is_sandbox: false,
                }])
            });
        let r = Reconciler {
            customers: Some(Arc::new(customers)),
            allow_sandbox: false,
        };
        let n = reconcile(&svc, &r, &paywall(), now, Duration::days(3))
            .await
            .unwrap();
        assert_eq!(n, 1);
        {
            let ws = writes.lock().unwrap();
            let last = ws.last().unwrap();
            assert_eq!(last.provider_ref, "otx-1");
            assert_eq!(last.ends_at, Some(now + Duration::days(31)));
            assert_eq!(last.status, EntitlementStatus::Active);
        }
        // refund without an event: caught by the next sweep
        let mut refunded = MockStoreCustomerSource::new();
        refunded.expect_subscriptions().returning(move |_| {
            Ok(vec![StoreSubscription {
                store: "APP_STORE".into(),
                product_id: "premium_monthly".into(),
                provider_ref: "otx-1".into(),
                purchase_at: None,
                expires_at: Some(now + Duration::days(31)),
                unsubscribe_detected_at: None,
                billing_issues_detected_at: None,
                grace_period_expires_at: None,
                refunded_at: Some(now),
                is_sandbox: false,
            }])
        });
        let r = Reconciler {
            customers: Some(Arc::new(refunded)),
            allow_sandbox: false,
        };
        // the row now ends in 31 days: widen the horizon so it is selected
        let n = reconcile(&svc, &r, &paywall(), now, Duration::days(40))
            .await
            .unwrap();
        assert_eq!(n, 1);
        assert_eq!(
            writes.lock().unwrap().last().unwrap().status,
            EntitlementStatus::Refunded
        );
    }

    #[tokio::test]
    async fn aggregator_error_leaves_the_row_untouched() {
        let (svc, writes, _) = service();
        let now = Utc::now();
        seed(&svc, now, Duration::days(1)).await;
        let before = writes.lock().unwrap().len();
        let mut failing = MockStoreCustomerSource::new();
        failing
            .expect_subscriptions()
            .returning(|_| Err(AppError::Internal(anyhow::anyhow!("down"))));
        let r = Reconciler {
            customers: Some(Arc::new(failing)),
            allow_sandbox: false,
        };
        let n = reconcile(&svc, &r, &paywall(), now, Duration::days(3))
            .await
            .unwrap();
        assert_eq!(n, 0);
        assert_eq!(writes.lock().unwrap().len(), before);
    }
}
