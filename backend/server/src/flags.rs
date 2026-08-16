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

/// Feature-flag-backed practice-streak configuration (change: add-practice-streak,
/// task 2.1). Resolved on **every** call, so an operator retuning the freeze cost
/// or the grace window in the back office changes the next offer without a
/// redeploy — the L1 snapshot behind it is kept fresh by the refreshers below.
///
/// Reads are evaluation-only and never fail: an unreachable store serves the
/// caller's defaults, which are the design's starting values.
pub struct FlagStreakConfig {
    flags: Arc<FlagService>,
}

impl FlagStreakConfig {
    pub fn new(flags: Arc<FlagService>) -> Self {
        Self { flags }
    }
}

impl cymbra_music::StreakConfigSource for FlagStreakConfig {
    fn streak_config(&self) -> cymbra_music::StreakConfig {
        // The keys are `music`-scoped config, but the streak is evaluated
        // server-side with no app context — an anonymous context resolves the
        // global override, which is the only rollout these tunables use.
        let ctx =
            cymbra_feature_flags::EvalContext::anonymous(cymbra_feature_flags::registry::APP_MUSIC);
        cymbra_music::StreakConfig {
            freeze_cost: self.flags.int(
                cymbra_feature_flags::registry::STREAK_FREEZE_COST,
                cymbra_music::DEFAULT_FREEZE_COST,
                &ctx,
            ),
            grace_days: self.flags.int(
                cymbra_feature_flags::registry::STREAK_GRACE_DAYS,
                cymbra_music::DEFAULT_GRACE_DAYS,
                &ctx,
            ),
        }
    }
}

/// Feature-flag-backed catalog daily-access configuration (change:
/// add-score-daily-access-rewards, design D1). Resolved on **every** open, so the
/// quota, the day-slot cost and the kill-switch retuned in the back office apply
/// to the next open without a redeploy. `staff` builds a staff context so a
/// `staff_only` rollout of the gate reaches admins/moderators first.
///
/// Reads are evaluation-only and never fail: an unreachable store serves the
/// code defaults (gate OFF).
pub struct FlagDailyAccessConfig {
    flags: Arc<FlagService>,
}

impl FlagDailyAccessConfig {
    pub fn new(flags: Arc<FlagService>) -> Self {
        Self { flags }
    }
}

impl cymbra_music::DailyAccessConfigSource for FlagDailyAccessConfig {
    fn daily_access_config(&self, staff: bool) -> cymbra_music::DailyAccessConfig {
        use cymbra_feature_flags::registry;
        let ctx = if staff {
            cymbra_feature_flags::EvalContext::authenticated(registry::APP_MUSIC, &["admin".into()])
        } else {
            cymbra_feature_flags::EvalContext::anonymous(registry::APP_MUSIC)
        };
        let defaults = cymbra_music::DailyAccessConfig::default();
        cymbra_music::DailyAccessConfig {
            enabled: self.flags.bool(
                registry::CATALOG_DAILY_ACCESS_ENABLED,
                defaults.enabled,
                &ctx,
            ),
            free_quota: self
                .flags
                .int(
                    registry::CATALOG_DAILY_ACCESS_FREE_QUOTA,
                    i64::from(defaults.free_quota),
                    &ctx,
                )
                .clamp(0, i64::from(u32::MAX)) as u32,
            day_slot_cost: self
                .flags
                .int(
                    registry::CATALOG_DAILY_ACCESS_DAY_SLOT_COST,
                    defaults.day_slot_cost,
                    &ctx,
                )
                .max(0),
        }
    }
}

/// Feature-flag-backed score audio-teaser configuration (change:
/// add-score-daily-access-rewards, design D7): the clip length and the catalog
/// SoundFont it is rendered with. Read per render (worker job and the back-office
/// regenerate) so retuning needs no redeploy.
pub struct FlagScorePreviewConfig {
    flags: Arc<FlagService>,
}

impl FlagScorePreviewConfig {
    pub fn new(flags: Arc<FlagService>) -> Self {
        Self { flags }
    }
}

impl cymbra_music::ScorePreviewConfigSource for FlagScorePreviewConfig {
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
        }
    }
}

/// Feature-flag-backed catalog access limits (change: add-catalog-access-limits,
/// task 1.3) — the per-user scrape guardrail's thresholds. Resolved on **every**
/// checked request, so an operator retuning a limit in the back office, or
/// tripping the kill-switch when the guardrail misfires, applies to the next
/// request without a redeploy.
///
/// The `base` config parsed from the environment is each key's default, so an
/// unset override keeps the deployment's own baseline. Reads are evaluation-only
/// and never fail: an unreachable store serves that baseline.
pub struct FlagCatalogLimitsConfig {
    flags: Arc<FlagService>,
    base: cymbra_platform::config::CatalogLimitsConfig,
}

impl FlagCatalogLimitsConfig {
    pub fn new(
        flags: Arc<FlagService>,
        base: cymbra_platform::config::CatalogLimitsConfig,
    ) -> Self {
        Self { flags, base }
    }
}

impl cymbra_music::CatalogLimitsConfigSource for FlagCatalogLimitsConfig {
    fn catalog_limits(&self) -> cymbra_platform::config::CatalogLimitsConfig {
        use cymbra_feature_flags::registry;
        // Server-side enforcement with no app context: an anonymous music context
        // resolves the global override, the only rollout these tunables use.
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_MUSIC);
        // Counts are `u32` and windows are seconds; a negative or absurd override
        // is clamped rather than wrapped — a bad edit must not mint an unlimited
        // (or zero-second) window.
        let count = |key: &str, default: u32| -> u32 {
            self.flags
                .int(key, i64::from(default), &ctx)
                .clamp(0, i64::from(u32::MAX)) as u32
        };
        let window = |key: &str, default: std::time::Duration| -> std::time::Duration {
            let secs = self
                .flags
                .int(key, default.as_secs() as i64, &ctx)
                .clamp(1, 365 * 24 * 3600);
            std::time::Duration::from_secs(secs as u64)
        };
        cymbra_platform::config::CatalogLimitsConfig {
            enabled: self
                .flags
                .bool(registry::CATALOG_LIMITS_ENABLED, self.base.enabled, &ctx),
            download_burst_max: count(
                registry::CATALOG_LIMITS_DL_BURST_MAX,
                self.base.download_burst_max,
            ),
            download_burst_window: window(
                registry::CATALOG_LIMITS_DL_BURST_WINDOW_S,
                self.base.download_burst_window,
            ),
            volume_window: window(
                registry::CATALOG_LIMITS_DL_VOLUME_WINDOW_S,
                self.base.volume_window,
            ),
            volume_base_floor: count(
                registry::CATALOG_LIMITS_DL_BASE_FLOOR,
                self.base.volume_base_floor,
            ),
            volume_per_engagement: count(
                registry::CATALOG_LIMITS_DL_PER_ENGAGEMENT,
                self.base.volume_per_engagement,
            ),
            volume_hard_ceiling: count(
                registry::CATALOG_LIMITS_DL_HARD_CEILING,
                self.base.volume_hard_ceiling,
            ),
            enum_max: count(registry::CATALOG_LIMITS_ENUM_MAX, self.base.enum_max),
            enum_window: window(
                registry::CATALOG_LIMITS_ENUM_WINDOW_S,
                self.base.enum_window,
            ),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_feature_flags::registry::{self, APP_MUSIC};
    use cymbra_feature_flags::resolver::MockAdminScopeResolver;
    use cymbra_feature_flags::store::{MockFlagStore, StoredOverride};
    use cymbra_feature_flags::{FlagValue, RolloutScope};
    use cymbra_music::CatalogLimitsConfigSource;
    use cymbra_platform::config::CatalogLimitsConfig;
    use std::time::Duration;

    /// The env baseline the overrides below have to visibly move away from.
    fn base() -> CatalogLimitsConfig {
        CatalogLimitsConfig {
            enabled: true,
            download_burst_max: 20,
            download_burst_window: Duration::from_secs(60),
            volume_window: Duration::from_secs(24 * 3600),
            volume_base_floor: 30,
            volume_per_engagement: 3,
            volume_hard_ceiling: 500,
            enum_max: 60,
            enum_window: Duration::from_secs(60),
        }
    }

    fn int_override(key: &str, v: i64) -> StoredOverride {
        StoredOverride {
            app: APP_MUSIC.into(),
            key: key.into(),
            value_type: cymbra_feature_flags::ValueType::Int,
            value: FlagValue::Int(v),
            rollout: RolloutScope::Global,
            sensitive: false,
            updated_by: "op".into(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Build a flag service whose store serves `overrides`.
    async fn service(overrides: Vec<StoredOverride>) -> Arc<FlagService> {
        let mut store = MockFlagStore::new();
        store
            .expect_load_all()
            .returning(move || Ok(overrides.clone()));
        let svc = Arc::new(FlagService::new(
            cymbra_feature_flags::Registry::default(),
            Some(Arc::new(store)),
            Arc::new(cymbra_feature_flags::NoopBus),
            Arc::new(MockAdminScopeResolver::new()),
        ));
        svc.refresh().await.unwrap();
        svc
    }

    #[tokio::test]
    async fn unset_overrides_serve_the_env_baseline() {
        let src = FlagCatalogLimitsConfig::new(service(vec![]).await, base());
        assert_eq!(src.catalog_limits(), base());
    }

    /// Every knob is wired to its own key: a distinct override per key must land
    /// on the matching field (a copy/paste key mix-up shows up here).
    #[tokio::test]
    async fn each_override_lands_on_its_own_threshold() {
        let src = FlagCatalogLimitsConfig::new(
            service(vec![
                int_override(registry::CATALOG_LIMITS_DL_BURST_MAX, 7),
                int_override(registry::CATALOG_LIMITS_DL_BURST_WINDOW_S, 11),
                int_override(registry::CATALOG_LIMITS_DL_VOLUME_WINDOW_S, 3_600),
                int_override(registry::CATALOG_LIMITS_DL_BASE_FLOOR, 5),
                int_override(registry::CATALOG_LIMITS_DL_PER_ENGAGEMENT, 9),
                int_override(registry::CATALOG_LIMITS_DL_HARD_CEILING, 42),
                int_override(registry::CATALOG_LIMITS_ENUM_MAX, 13),
                int_override(registry::CATALOG_LIMITS_ENUM_WINDOW_S, 17),
            ])
            .await,
            base(),
        );
        let cfg = src.catalog_limits();
        assert_eq!(cfg.download_burst_max, 7);
        assert_eq!(cfg.download_burst_window, Duration::from_secs(11));
        assert_eq!(cfg.volume_window, Duration::from_secs(3_600));
        assert_eq!(cfg.volume_base_floor, 5);
        assert_eq!(cfg.volume_per_engagement, 9);
        assert_eq!(cfg.volume_hard_ceiling, 42);
        assert_eq!(cfg.enum_max, 13);
        assert_eq!(cfg.enum_window, Duration::from_secs(17));
        assert!(cfg.enabled, "untouched by the threshold overrides");
    }

    #[tokio::test]
    async fn the_kill_switch_override_disables_the_guardrail() {
        let off = StoredOverride {
            value_type: cymbra_feature_flags::ValueType::Bool,
            value: FlagValue::Bool(false),
            ..int_override(registry::CATALOG_LIMITS_ENABLED, 0)
        };
        let src = FlagCatalogLimitsConfig::new(service(vec![off]).await, base());
        assert!(!src.catalog_limits().enabled);
    }

    /// A fat-fingered edit must not wrap into an unlimited allowance or a
    /// zero-second (i.e. never-resetting-usefully) window.
    #[tokio::test]
    async fn absurd_overrides_are_clamped() {
        let src = FlagCatalogLimitsConfig::new(
            service(vec![
                int_override(registry::CATALOG_LIMITS_DL_BASE_FLOOR, -1),
                int_override(registry::CATALOG_LIMITS_ENUM_WINDOW_S, 0),
                int_override(registry::CATALOG_LIMITS_DL_HARD_CEILING, i64::MAX),
            ])
            .await,
            base(),
        );
        let cfg = src.catalog_limits();
        assert_eq!(cfg.volume_base_floor, 0);
        assert_eq!(cfg.enum_window, Duration::from_secs(1));
        assert_eq!(cfg.volume_hard_ceiling, u32::MAX);
    }
}
