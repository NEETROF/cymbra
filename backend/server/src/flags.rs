//! Feature-flag wiring for the composition root (change: add-runtime-feature-flags).
//!
//! Builds the shared [`FlagService`] from config: a Postgres override store (role
//! `flags_svc`) + a Redis invalidation bus when `CYMBRA_FLAGS_DATABASE_URL` is set,
//! else defaults-only mode. The platform-admin check reads the user account's
//! scoped roles on the user-owned pool (the isolated `flags_svc` role cannot),
//! keeping the flags crate app-agnostic.

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_feature_flags::{
    AdminScopeResolver, DEFAULT_CHANNEL, FlagService, FlagStore, InvalidationBus, NoopBus,
    PgFlagStore, RedisInvalidationBus, Registry,
};
use cymbra_platform::config::Config;
use cymbra_platform::error::AppError;
use cymbra_platform::{Result, db};
use sqlx::PgPool;

/// Resolves whether an account is a platform (`global`) admin by reading its
/// scoped roles from `user_account.user_roles` on the user pool.
pub struct PgAdminScopeResolver {
    user_pool: PgPool,
}

impl PgAdminScopeResolver {
    pub fn new(user_pool: PgPool) -> Self {
        Self { user_pool }
    }
}

#[async_trait]
impl AdminScopeResolver for PgAdminScopeResolver {
    async fn is_platform_admin(&self, user_id: &str) -> Result<bool> {
        let uid = uuid::Uuid::parse_str(user_id)
            .map_err(|_| AppError::InvalidArgument(format!("invalid uuid: {user_id}")))?;
        let is_admin: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM user_roles \
             WHERE user_id = $1 AND scope = 'global' AND role = 'admin')",
        )
        .bind(uid)
        .fetch_one(&self.user_pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("platform-admin check: {e}")))?;
        Ok(is_admin)
    }
}

/// Build the shared flag service. Runs the flags migrations and does a best-effort
/// initial L1 load so flags are hot before serving. `user_pool` backs the
/// platform-admin resolver.
pub async fn build_flag_service(cfg: &Config, user_pool: PgPool) -> Result<Arc<FlagService>> {
    let (store, bus): (Option<Arc<dyn FlagStore>>, Arc<dyn InvalidationBus>) = match &cfg
        .flags_database_url
    {
        Some(url) => {
            let pool = db::connect(url, 5).await?;
            cymbra_feature_flags::MIGRATOR
                .run(&pool)
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("flags migrate: {e}")))?;
            let store: Arc<dyn FlagStore> = Arc::new(PgFlagStore::new(pool));
            // The bus is best-effort: a down Redis just means edits propagate via the
            // L1 TTL backstop instead of within milliseconds.
            let bus: Arc<dyn InvalidationBus> =
                match RedisInvalidationBus::connect(&cfg.redis_url, DEFAULT_CHANNEL).await {
                    Ok(b) => Arc::new(b),
                    Err(e) => {
                        tracing::warn!("feature-flag Redis bus unavailable, TTL-only: {e}");
                        Arc::new(NoopBus)
                    }
                };
            (Some(store), bus)
        }
        None => {
            tracing::info!("feature flags in defaults-only mode (CYMBRA_FLAGS_DATABASE_URL unset)");
            (None, Arc::new(NoopBus))
        }
    };

    let resolver = Arc::new(PgAdminScopeResolver::new(user_pool));
    let service = Arc::new(FlagService::new(Registry::default(), store, bus, resolver));
    if let Err(e) = service.refresh().await {
        // Non-fatal: evaluation falls back to code defaults until the next refresh.
        tracing::warn!("initial feature-flag load failed (serving defaults): {e}");
    }
    Ok(service)
}

/// Spawn the background refreshers: a TTL backstop tick and (when a store is
/// configured) the Redis invalidation listener that refreshes L1 on each ping.
pub fn spawn_flag_refreshers(cfg: &Config, service: Arc<FlagService>) {
    let ttl = service.ttl();
    {
        let svc = service.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(ttl);
            loop {
                tick.tick().await;
                if let Err(e) = svc.refresh_if_stale().await {
                    tracing::warn!("feature-flag TTL refresh failed: {e}");
                }
            }
        });
    }
    if cfg.flags_database_url.is_some() {
        let svc = service;
        let redis_url = cfg.redis_url.clone();
        tokio::spawn(async move {
            let refresh = move || {
                let s = svc.clone();
                async move {
                    if let Err(e) = s.refresh().await {
                        tracing::warn!("feature-flag refresh on invalidation failed: {e}");
                    }
                }
            };
            if let Err(e) = cymbra_feature_flags::run_invalidation_listener(
                &redis_url,
                DEFAULT_CHANNEL,
                refresh,
            )
            .await
            {
                tracing::warn!("feature-flag invalidation listener ended: {e}");
            }
        });
    }
}
