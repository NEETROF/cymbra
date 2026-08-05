//! Minimal feature-flag service for the worker (change: add-feature-usage-
//! analytics, D4). The usage-purge job reads the raw-event retention window from
//! `cymbra-feature-flags` so an operator can retune it in the back office without a
//! redeploy. The worker only *reads* config values (no admin edits, no per-user
//! rollout, no invalidation bus) — so it uses a no-op admin resolver + no-op bus,
//! and the purge handler refreshes the L1 snapshot on demand rather than running a
//! background refresher. Migrating the `feature_flags` schema stays the server's
//! job; a read failure just serves code defaults (fail-safe).

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_feature_flags::{
    AdminScopeResolver, FlagService, FlagStore, NoopBus, PgFlagStore, Registry,
};
use cymbra_platform::db;

/// The worker never evaluates admin-scoped rollout, so platform-admin resolution
/// is irrelevant — always `false`.
struct NoopResolver;

#[async_trait]
impl AdminScopeResolver for NoopResolver {
    async fn is_platform_admin(&self, _user_id: &str) -> cymbra_platform::Result<bool> {
        Ok(false)
    }
}

/// Build a read-only flag service: a Postgres override store when the flags DB is
/// configured (defaults-only otherwise). Does a best-effort initial load so the
/// first purge reads current overrides.
pub async fn build_flag_service(
    flags_database_url: Option<&str>,
) -> anyhow::Result<Arc<FlagService>> {
    let store: Option<Arc<dyn FlagStore>> = match flags_database_url {
        Some(url) => {
            let pool = db::connect(url, 2).await?;
            Some(Arc::new(PgFlagStore::new(pool)))
        }
        None => {
            tracing::info!(
                "worker feature flags in defaults-only mode (CYMBRA_FLAGS_DATABASE_URL unset)"
            );
            None
        }
    };
    let service = Arc::new(FlagService::new(
        Registry::default(),
        store,
        Arc::new(NoopBus),
        Arc::new(NoopResolver),
    ));
    if let Err(e) = service.refresh().await {
        tracing::warn!("initial worker flag load failed (serving defaults): {e}");
    }
    Ok(service)
}
