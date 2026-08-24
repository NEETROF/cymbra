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

/// Feature-flag-backed score audio-teaser configuration for the worker's
/// `score_preview_render` job (change: add-score-daily-access-rewards, design
/// D7): the clip length and the catalog SoundFont it is rendered with. Read per
/// render so retuning in the back office needs no redeploy; the L1 snapshot is
/// refreshed on demand by the handler.
pub struct WorkerScorePreviewConfig {
    flags: Arc<FlagService>,
}

impl WorkerScorePreviewConfig {
    pub fn new(flags: Arc<FlagService>) -> Self {
        Self { flags }
    }
}

impl cymbra_music::ScorePreviewConfigSource for WorkerScorePreviewConfig {
    fn score_preview_config(&self) -> cymbra_music::ScorePreviewConfig {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_MUSIC);
        let defaults = cymbra_music::ScorePreviewConfig::default();
        cymbra_music::ScorePreviewConfig {
            max_ms: self
                .flags
                .int(
                    registry::CATALOG_PREVIEW_MAX_MS,
                    i64::from(defaults.max_ms),
                    &ctx,
                )
                .clamp(0, i64::from(u32::MAX)) as u32,
            soundfont_id: self.flags.string(
                registry::CATALOG_PREVIEW_SOUNDFONT_ID,
                &defaults.soundfont_id,
                &ctx,
            ),
            drum_soundfont_id: self.flags.string(
                registry::CATALOG_PREVIEW_DRUM_SOUNDFONT_ID,
                &defaults.drum_soundfont_id,
                &ctx,
            ),
        }
    }
}

/// Flag-backed [`cymbra_plans::PlanConfigSource`] for the worker sweeps: the
/// kill-switch + grace period (mirrors the server's `FlagPlanConfig`).
pub struct WorkerPlanConfig {
    flags: Arc<FlagService>,
}

impl WorkerPlanConfig {
    pub fn new(flags: Arc<FlagService>) -> Self {
        Self { flags }
    }
}

impl cymbra_plans::PlanConfigSource for WorkerPlanConfig {
    fn plan_config(&self) -> cymbra_plans::PlanConfig {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_MUSIC);
        let defaults = cymbra_plans::PlanConfig::default();
        cymbra_plans::PlanConfig {
            enabled: self
                .flags
                .bool(registry::PLANS_ENABLED, defaults.enabled, &ctx),
            grace_days: self
                .flags
                .int(
                    registry::PLANS_GRACE_DAYS,
                    i64::from(defaults.grace_days),
                    &ctx,
                )
                .clamp(0, 365) as u32,
        }
    }
}

/// Flag-backed [`cymbra_plans::PaywallConfigSource`] for the worker: per-channel
/// switches + the premium product ids (mirrors the server's `FlagPaywallConfig`);
/// the reconciliation sweep grants only for those products.
pub struct WorkerPaywallConfig {
    flags: Arc<FlagService>,
}

impl WorkerPaywallConfig {
    pub fn new(flags: Arc<FlagService>) -> Self {
        Self { flags }
    }
}

impl cymbra_plans::PaywallConfigSource for WorkerPaywallConfig {
    fn channel_enabled(&self, channel: cymbra_plans::Channel) -> bool {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_MUSIC);
        let key = match channel {
            cymbra_plans::Channel::Apple => registry::BILLING_APPLE_ENABLED,
            cymbra_plans::Channel::Google => registry::BILLING_GOOGLE_ENABLED,
            cymbra_plans::Channel::Web => registry::BILLING_WEB_ENABLED,
        };
        self.flags.bool(key, false, &ctx)
    }

    fn products(&self) -> Vec<String> {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_MUSIC);
        self.flags
            .json(
                registry::PLANS_PREMIUM_PRODUCTS,
                serde_json::json!(["premium_monthly", "premium_yearly"]),
                &ctx,
            )
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Withdrawal-on-lapse rotator (design D13) over the music offline-secret store,
/// on the worker's `admin_svc` connection.
pub struct OfflineSecretRotator {
    secrets: Arc<dyn cymbra_music::OfflineSecretRepo>,
}

impl OfflineSecretRotator {
    pub fn new(secrets: Arc<dyn cymbra_music::OfflineSecretRepo>) -> Self {
        Self { secrets }
    }
}

#[async_trait]
impl cymbra_plans::CacheSecretRotator for OfflineSecretRotator {
    async fn rotate(&self, user_id: &str) -> cymbra_platform::Result<()> {
        let fresh = cymbra_music::generate_offline_secret();
        self.secrets.rotate(user_id, &fresh).await.map_err(|e| {
            cymbra_platform::AppError::Internal(anyhow::anyhow!("rotate offline secret: {e}"))
        })
    }
}
