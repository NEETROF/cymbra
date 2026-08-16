//! Reconciliation (spec `music-subscription-billing`, "Restore and reconciliation
//! keep the ledger truthful"): for paid rows nearing their end, re-read the
//! provider state and re-apply, so a missed notification never leaves a paying
//! user degraded nor a refunded user active. Runs daily from the worker, before
//! the withdrawal sweep.

use crate::billing::apple::{AppStoreServerApi, map_transaction};
use crate::billing::google::{PlayApi, map_subscription as map_google};
use crate::billing::web::{PaddleProvider, map_subscription as map_web};
use crate::model::Source;
use crate::service::PlanService;
use chrono::{DateTime, Duration, Utc};
use cymbra_platform::Result;
use std::sync::Arc;

/// The provider clients the sweep may use; a missing one skips its rows.
#[derive(Default)]
pub struct Reconcilers {
    pub apple: Option<Arc<AppStoreServerApi>>,
    pub google: Option<Arc<dyn PlayApi>>,
    pub web: Option<Arc<PaddleProvider>>,
}

/// Re-read every paid row ending within `horizon` and re-apply the provider's
/// current state. Per-row failures are logged and skipped (the next run
/// retries); returns how many rows were re-applied.
pub async fn reconcile(
    svc: &PlanService,
    clients: &Reconcilers,
    now: DateTime<Utc>,
    horizon: Duration,
) -> Result<u64> {
    let rows = svc
        .rows_ending_between(now - Duration::days(1), now + horizon)
        .await?;
    let mut applied = 0u64;
    for row in rows {
        let write = match row.source {
            Source::Apple => match &clients.apple {
                Some(api) => api
                    .latest_transactions(&row.provider_ref, now)
                    .await
                    .map(|txs| {
                        txs.into_iter().find_map(|(tx, renewal)| {
                            map_transaction(&row.user_id, &tx, renewal.as_ref(), None, None, now)
                        })
                    }),
                None => continue,
            },
            Source::Google => match &clients.google {
                Some(api) => api
                    .get_subscription(&row.provider_ref)
                    .await
                    .map(|sub| map_google(&row.user_id, &row.provider_ref, &sub, now)),
                None => continue,
            },
            Source::Web => match &clients.web {
                Some(api) => api
                    .subscription(&row.provider_ref)
                    .await
                    .map(|sub| map_web(&row.user_id, &sub, now)),
                None => continue,
            },
            Source::Code | Source::Admin => continue,
        };
        match write {
            Ok(Some(w)) => {
                svc.apply(w).await?;
                applied += 1;
            }
            Ok(None) => {}
            Err(e) => tracing::warn!(
                error = %e,
                source = row.source.as_str(),
                provider_ref = %row.provider_ref,
                "reconciliation read failed; will retry next run"
            ),
        }
    }
    Ok(applied)
}
