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
            drum_soundfont_id: self.flags.string(
                registry::CATALOG_PREVIEW_DRUM_SOUNDFONT_ID,
                &defaults.drum_soundfont_id,
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

// --- plans (change: add-premium-subscription) ------------------------------------

/// Flag-backed [`cymbra_plans::PlanConfigSource`]: the kill-switch + grace period,
/// read per call so an operator flips them with no redeploy.
pub struct FlagPlanConfig {
    flags: Arc<FlagService>,
}

impl FlagPlanConfig {
    pub fn new(flags: Arc<FlagService>) -> Self {
        Self { flags }
    }
}

impl cymbra_plans::PlanConfigSource for FlagPlanConfig {
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

/// Flag-backed private `.sf2` library quota per plan.
pub struct FlagLibraryQuota {
    flags: Arc<FlagService>,
}

impl FlagLibraryQuota {
    pub fn new(flags: Arc<FlagService>) -> Self {
        Self { flags }
    }
}

impl cymbra_music::LibraryQuotaSource for FlagLibraryQuota {
    fn max_fonts(&self, extended: bool) -> i64 {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_MUSIC);
        let defaults = cymbra_music::FixedLibraryQuota::default();
        let (key, default) = if extended {
            (registry::PLANS_SF_LIBRARY_MAX_PREMIUM, defaults.premium)
        } else {
            (registry::PLANS_SF_LIBRARY_MAX_FREE, defaults.free)
        };
        self.flags.int(key, default, &ctx).max(0)
    }
}

/// Flag-backed score quotas per plan (rolling upload quota + private library cap).
/// `base` = the environment defaults for the free plan (`CYMBRA_SCORE_UPLOAD_QUOTA_*`).
pub struct FlagScoreQuotas {
    flags: Arc<FlagService>,
    base: cymbra_music::ScoreQuotas,
}

impl FlagScoreQuotas {
    pub fn new(flags: Arc<FlagService>, base: cymbra_music::ScoreQuotas) -> Self {
        Self { flags, base }
    }
}

impl cymbra_music::ScoreQuotaSource for FlagScoreQuotas {
    fn score_quotas(&self, extended: bool) -> cymbra_music::ScoreQuotas {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_MUSIC);
        let (quota_key, lib_key, default_max, default_lib) = if extended {
            (
                registry::PLANS_SCORES_UPLOAD_QUOTA_PREMIUM,
                registry::PLANS_SCORES_LIBRARY_MAX_PREMIUM,
                self.base.upload_max.max(50),
                500,
            )
        } else {
            (
                registry::PLANS_SCORES_UPLOAD_QUOTA_FREE,
                registry::PLANS_SCORES_LIBRARY_MAX_FREE,
                self.base.upload_max,
                20,
            )
        };
        let quota = self.flags.json(
            quota_key,
            serde_json::json!({
                "max": default_max,
                "window_days": self.base.upload_window_days,
            }),
            &ctx,
        );
        let upload_max = quota
            .get("max")
            .and_then(|v| v.as_u64())
            .map(|v| v.min(u64::from(u32::MAX)) as u32)
            .unwrap_or(default_max);
        let upload_window_days = quota
            .get("window_days")
            .and_then(|v| v.as_u64())
            .map(|v| v.clamp(1, 3650) as u32)
            .unwrap_or(self.base.upload_window_days);
        let library_max = self.flags.int(lib_key, default_lib, &ctx);
        cymbra_music::ScoreQuotas {
            upload_max,
            upload_window_days,
            library_max: (library_max > 0).then_some(library_max),
        }
    }
}

/// The plans → flags bridge: the caller's effective plan + active beta keys, so a
/// `premium_only` / `beta:<key>` rollout can be evaluated. Read errors ⇒ free.
pub struct PlanContext {
    plans: Arc<dyn cymbra_plans::PlanSource>,
}

impl PlanContext {
    pub fn new(plans: Arc<dyn cymbra_plans::PlanSource>) -> Self {
        Self { plans }
    }
}

#[async_trait]
impl cymbra_feature_flags::PlanContextSource for PlanContext {
    async fn plan_context(
        &self,
        user_id: &str,
    ) -> cymbra_platform::error::Result<(bool, Vec<String>)> {
        // Propagated, never defaulted (change: add-drum-input-mapping — beta fix):
        // answering "free, no betas" on a failed read is a silent entitlement
        // downgrade the caller cannot detect and will cache. The plans kill-switch
        // being off is a different thing entirely — that resolves successfully to
        // an empty beta set, and still does.
        let s = self.plans.snapshot(user_id).await.inspect_err(|e| {
            tracing::warn!(error = %e, "plan snapshot failed; flag read refused rather than downgraded");
        })?;
        Ok((s.plan == cymbra_plans::Plan::Premium, s.beta_keys()))
    }
}

/// The plans → flags campaign-existence bridge (change:
/// add-flag-campaign-integrity), beside [`PlanContext`] — same seam, same
/// direction, opposite error posture: unlike `plan_context`, this one
/// PROPAGATES errors, because swallowing one would tell the operator a
/// campaign does not exist when the truth is that nobody could ask.
pub struct CampaignExistence {
    campaigns: Arc<dyn cymbra_plans::CampaignRepo>,
}

impl CampaignExistence {
    pub fn new(campaigns: Arc<dyn cymbra_plans::CampaignRepo>) -> Self {
        Self { campaigns }
    }
}

#[async_trait]
impl cymbra_feature_flags::CampaignDirectory for CampaignExistence {
    async fn campaign_exists(&self, key: &str) -> Result<bool> {
        // `get_by_key` does not filter on `closed_at`: existence includes
        // closed campaigns, which is exactly the contract.
        Ok(self.campaigns.get_by_key(key).await?.is_some())
    }
}

/// The plans → music bridge for withdrawal on lapse (design D13): rotating the
/// user's offline cache secret makes every cached catalog score unreadable.
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
    async fn rotate(&self, user_id: &str) -> Result<()> {
        let fresh = cymbra_music::generate_offline_secret();
        self.secrets
            .rotate(user_id, &fresh)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("rotate offline secret: {e}")))
    }
}

/// Flag-backed paywall knobs: per-channel switches + the product ids offered.
pub struct FlagPaywallConfig {
    flags: Arc<FlagService>,
}

impl FlagPaywallConfig {
    pub fn new(flags: Arc<FlagService>) -> Self {
        Self { flags }
    }
}

impl cymbra_plans::PaywallConfigSource for FlagPaywallConfig {
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

/// Handle → account id for the plan console, over the identity port's directory
/// (prefix search, then the exact normalized match).
pub struct UserPortHandles {
    users: Arc<dyn cymbra_user_port::UserPort>,
}

impl UserPortHandles {
    pub fn new(users: Arc<dyn cymbra_user_port::UserPort>) -> Self {
        Self { users }
    }
}

#[async_trait]
impl cymbra_plans::HandleResolver for UserPortHandles {
    async fn user_id_for_handle(&self, handle: &str) -> Result<Option<String>> {
        let wanted = handle.trim().trim_start_matches('@').to_lowercase();
        let page = self.users.list_accounts(&wanted, 10, 0, &[]).await?;
        Ok(page
            .entries
            .into_iter()
            .find(|a| {
                a.handle
                    .as_deref()
                    .is_some_and(|h| h.to_lowercase() == wanted)
            })
            .map(|a| a.user_id))
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

    fn string_override(key: &str, v: &str) -> StoredOverride {
        StoredOverride {
            value_type: cymbra_feature_flags::ValueType::String,
            value: FlagValue::String(v.into()),
            ..int_override(key, 0)
        }
    }

    /// The two preview font keys are independent (change: add-drum-audio-channel):
    /// each override lands on its own field, and both default empty (dormant).
    #[tokio::test]
    async fn score_preview_config_reads_both_font_keys() {
        use cymbra_music::ScorePreviewConfigSource as _;
        let src = FlagScorePreviewConfig::new(service(vec![]).await);
        let cfg = src.score_preview_config();
        assert!(cfg.soundfont_id.is_empty());
        assert!(cfg.drum_soundfont_id.is_empty());

        let src = FlagScorePreviewConfig::new(
            service(vec![
                string_override(registry::CATALOG_PREVIEW_SOUNDFONT_ID, "grand"),
                string_override(registry::CATALOG_PREVIEW_DRUM_SOUNDFONT_ID, "kit"),
            ])
            .await,
        );
        let cfg = src.score_preview_config();
        assert_eq!(cfg.soundfont_id, "grand");
        assert_eq!(cfg.drum_soundfont_id, "kit");
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
