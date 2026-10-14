//! Feature-flag wiring for the composition root (change: add-runtime-feature-flags).
//!
//! Builds the shared [`FlagService`] from config: a Postgres override store (role
//! `flags_svc`) + a Redis invalidation bus when `CYMBRA_FLAGS_DATABASE_URL` is set,
//! else defaults-only mode. The platform-admin check reads the user account's
//! scoped roles on the user-owned pool (the isolated `flags_svc` role cannot),
//! keeping the flags crate app-agnostic.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use cymbra_feature_flags::{
    AdminScopeResolver, DEFAULT_CHANNEL, FlagService, FlagStore, InvalidationBus, NoopBus,
    PgFlagStore, RedisInvalidationBus, Registry,
};
use cymbra_platform::config::Config;
use cymbra_platform::error::AppError;
use cymbra_platform::{Result, db};
use cymbra_user_port::UserPort;

/// Resolves whether an account is a platform (`global`) admin, **through the user
/// port** (change: harden-module-boundaries, group 8).
///
/// It used to run its own `SELECT … FROM user_roles` on the user pool. That was not a
/// privilege defect — the pool is `user_svc`, whose `search_path` is `user_account`, so
/// the unqualified name resolved to a table that role owns. It was a *second
/// implementation* of a rule the user module already owns, and the two would have
/// diverged in silence the day "platform admin" gained a nuance.
pub struct PortAdminScopeResolver {
    users: Arc<dyn UserPort>,
}

impl PortAdminScopeResolver {
    pub fn new(users: Arc<dyn UserPort>) -> Self {
        Self { users }
    }
}

#[async_trait]
impl AdminScopeResolver for PortAdminScopeResolver {
    async fn is_platform_admin(&self, user_id: &str) -> Result<bool> {
        let scope = cymbra_platform::GLOBAL_SCOPE.to_string();
        let held = self
            .users
            .scoped_effective_roles(user_id, std::slice::from_ref(&scope))
            .await?;
        Ok(held
            .get(&scope)
            .is_some_and(|roles| roles.iter().any(|r| r == "admin")))
    }
}

/// Build the shared flag service. Runs the flags migrations and does a best-effort
/// initial L1 load so flags are hot before serving. `users` backs the platform-admin
/// resolver — a port call, not a pool, so this crate no longer needs a second
/// connection to answer a question the user module owns.
pub async fn build_flag_service(
    cfg: &Config,
    users: Arc<dyn UserPort>,
) -> Result<Arc<FlagService>> {
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

    let resolver = Arc::new(PortAdminScopeResolver::new(users));
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
            cymbra_feature_flags::EvalContext::staff(registry::APP_MUSIC)
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

/// The plans → flags bridge: the caller's effective plan + active beta keys for the
/// product of the app being evaluated, so a `premium_only` / `beta:<key>` rollout can
/// be evaluated. Read errors propagate (see `plan_context`).
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
        app: &str,
    ) -> cymbra_platform::error::Result<(bool, Vec<String>)> {
        // The app being evaluated picks which product's snapshot is relevant; the
        // product itself stays carried by each campaign (change:
        // add-lingua-feature-betas). An app with no product has no plan and no beta.
        let Some(product) = product_of_app(app) else {
            return Ok((false, Vec::new()));
        };
        // Propagated, never defaulted (change: add-drum-input-mapping — beta fix):
        // answering "free, no betas" on a failed read is a silent entitlement
        // downgrade the caller cannot detect and will cache. The plans kill-switch
        // being off is a different thing entirely — that resolves successfully to
        // an empty beta set, and still does.
        let s = self.plans.snapshot(user_id, product).await.inspect_err(|e| {
            tracing::warn!(error = %e, "plan snapshot failed; flag read refused rather than downgraded");
        })?;
        Ok((s.plan == cymbra_plans::Plan::Premium, s.beta_keys()))
    }
}

/// « Traduction en ligne »'s runtime knobs (change: add-lingua-online-translation, D6): access
/// is `lingua.online_translation.enabled` evaluated for the caller WITH their Lingua betas, so
/// scoping it to `beta:online-translation` grants it per enrolled account and setting it off
/// withdraws it from everyone; the limit and the prices are plain Lingua config. Read on every
/// request, so a change needs no redeploy. Without plans deployed, nobody is in a beta.
pub struct FlagOnlineTranslationPolicy {
    flags: Arc<FlagService>,
    plans: Option<Arc<dyn cymbra_feature_flags::PlanContextSource>>,
}

impl FlagOnlineTranslationPolicy {
    pub fn new(
        flags: Arc<FlagService>,
        plans: Option<Arc<dyn cymbra_feature_flags::PlanContextSource>>,
    ) -> Self {
        Self { flags, plans }
    }
}

#[async_trait]
impl cymbra_lingua::online_translation::OnlineTranslationPolicy for FlagOnlineTranslationPolicy {
    async fn has_access(&self, id: &cymbra_platform::AuthIdentity) -> Result<bool> {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::for_identity(registry::APP_LINGUA, id);
        let ctx = match &self.plans {
            // A plan read that fails is an error, never "no beta" (see `PlanContext`).
            Some(plans) => {
                let (premium, betas) = plans
                    .plan_context(&id.user_id, registry::APP_LINGUA)
                    .await?;
                ctx.with_plan(premium, betas)
            }
            None => ctx,
        };
        Ok(self
            .flags
            .bool(registry::LINGUA_ONLINE_TRANSLATION_ENABLED, false, &ctx))
    }

    async fn daily_limit(&self) -> u32 {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_LINGUA);
        self.flags
            .int(registry::LINGUA_ONLINE_TRANSLATION_DAILY_LIMIT, 100, &ctx)
            .clamp(0, i64::from(u32::MAX)) as u32
    }

    async fn prices(&self, provider: &str) -> (f64, f64) {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_LINGUA);
        let (input, output, defaults) = match provider {
            "anthropic" => (
                registry::LINGUA_ONLINE_TRANSLATION_PRICE_ANTHROPIC_INPUT,
                registry::LINGUA_ONLINE_TRANSLATION_PRICE_ANTHROPIC_OUTPUT,
                (0.10, 0.50),
            ),
            _ => (
                registry::LINGUA_ONLINE_TRANSLATION_PRICE_GEMINI_INPUT,
                registry::LINGUA_ONLINE_TRANSLATION_PRICE_GEMINI_OUTPUT,
                (0.25, 1.50),
            ),
        };
        (
            self.flags.number(input, defaults.0, &ctx),
            self.flags.number(output, defaults.1, &ctx),
        )
    }
}

/// The anonymous usage counts' runtime knobs (change: add-lingua-usage-counts, D9, D10): the
/// kill switch `lingua.usage_counts.enabled` and the counters' ceiling, both plain Lingua
/// values evaluated with no caller (the service is unauthenticated). Read on every request, so
/// a change needs no redeploy.
pub struct FlagUsageCountsPolicy {
    flags: Arc<FlagService>,
}

impl FlagUsageCountsPolicy {
    pub fn new(flags: Arc<FlagService>) -> Self {
        Self { flags }
    }
}

impl cymbra_lingua::usage_counts::UsageCountsPolicy for FlagUsageCountsPolicy {
    fn enabled(&self) -> bool {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_LINGUA);
        self.flags
            .bool(registry::LINGUA_USAGE_COUNTS_ENABLED, true, &ctx)
    }

    fn ceiling(&self) -> i64 {
        use cymbra_feature_flags::registry;
        let ctx = cymbra_feature_flags::EvalContext::anonymous(registry::APP_LINGUA);
        self.flags
            .int(
                registry::LINGUA_USAGE_COUNTS_CEILING,
                cymbra_lingua::usage_counts::DEFAULT_CEILING,
                &ctx,
            )
            .max(0)
    }
}

/// The product whose plan and betas a flag evaluation for `app` reads: each app scope
/// is its own product; anything else (the site's `web`, the back office) has none.
pub(crate) fn product_of_app(app: &str) -> Option<&'static str> {
    cymbra_platform::APP_SCOPES.into_iter().find(|p| *p == app)
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
    async fn campaign_product(&self, key: &str) -> Result<Option<String>> {
        // `get_by_key` does not filter on `closed_at`: existence includes
        // closed campaigns, which is exactly the contract.
        Ok(self.campaigns.get_by_key(key).await?.map(|c| c.product))
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

    async fn handles_for_ids(&self, ids: &[String]) -> Result<HashMap<String, String>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        // The directory's `ids` filter answers this in one call; `plans` owns no account
        // table, which is why the lookup is wired here and not in the crate.
        let page = self
            .users
            .list_accounts_filtered(
                &cymbra_user_port::AccountFilter {
                    query: String::new(),
                    ids: ids.to_vec(),
                    ..Default::default()
                },
                ids.len() as i64,
                0,
                &[],
            )
            .await?;
        Ok(page
            .entries
            .into_iter()
            .filter_map(|a| a.handle.map(|h| (a.user_id, h)))
            .collect())
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

    /// A user port answering `scoped_effective_roles` with `roles` for the scope it is
    /// asked about, and nothing for any other.
    fn port_holding(scope: &'static str, roles: &'static [&'static str]) -> Arc<dyn UserPort> {
        let mut m = cymbra_user_port::MockUserPort::new();
        m.expect_scoped_effective_roles()
            .returning(move |_uid, asked| {
                Ok(asked
                    .iter()
                    .filter(|s| s.as_str() == scope)
                    .map(|s| {
                        (
                            s.clone(),
                            roles.iter().map(|r| (*r).to_string()).collect::<Vec<_>>(),
                        )
                    })
                    .collect())
            });
        Arc::new(m)
    }

    /// A flag evaluation for an app reads that app's product snapshot, and an app
    /// with no product reads none (change: add-lingua-feature-betas).
    #[tokio::test]
    async fn the_plan_context_reads_the_snapshot_of_the_evaluated_apps_product() {
        use cymbra_feature_flags::PlanContextSource;
        let mut plans = cymbra_plans::ports::MockPlanSource::new();
        plans.expect_snapshot().returning(|_, product| {
            let mut s = cymbra_plans::PlanSnapshot::free();
            s.betas.push(cymbra_plans::BetaInfo {
                campaign_key: format!("{product}-beta"),
                campaign_name: "b".into(),
                kind: cymbra_plans::CampaignKind::Feature,
                joined_at: chrono::Utc::now(),
                ends_at: None,
            });
            Ok(s)
        });
        let ctx = PlanContext::new(Arc::new(plans));
        let (_, lingua) = ctx.plan_context("u1", "lingua").await.unwrap();
        assert_eq!(lingua, vec!["lingua-beta".to_string()]);
        let (_, music) = ctx.plan_context("u1", "music").await.unwrap();
        assert_eq!(music, vec!["music-beta".to_string()]);
        // The site and the back office have no product: no plan, no beta, no read.
        assert_eq!(
            ctx.plan_context("u1", "web").await.unwrap(),
            (false, vec![])
        );
        assert_eq!(product_of_app("back-office"), None);
    }

    /// The directory answers a campaign's product, so a scope write can be checked
    /// against the flag's app; a missing campaign is `None`.
    #[tokio::test]
    async fn the_campaign_directory_answers_the_campaigns_product() {
        use cymbra_feature_flags::CampaignDirectory;
        let mut repo = cymbra_plans::ports::MockCampaignRepo::new();
        repo.expect_get_by_key().returning(|key| {
            Ok(
                (key == "online-translation").then(|| cymbra_plans::Campaign {
                    id: uuid::Uuid::nil(),
                    key: key.to_string(),
                    name: "Online".into(),
                    kind: cymbra_plans::CampaignKind::Feature,
                    enrollment_closes_at: None,
                    closed_at: None,
                    created_by: "adm".into(),
                    created_at: chrono::Utc::now(),
                    product: "lingua".into(),
                }),
            )
        });
        let dir = CampaignExistence::new(Arc::new(repo));
        assert_eq!(
            dir.campaign_product("online-translation").await.unwrap(),
            Some("lingua".to_string())
        );
        assert_eq!(dir.campaign_product("nope").await.unwrap(), None);
    }

    /// "Platform admin" is `admin` in the **global** scope, and the answer now comes
    /// from the user port — one implementation, owned by the module that owns roles
    /// (change: harden-module-boundaries, group 8). No database: that is the point of
    /// the change, so the test is written the way the code now allows.
    #[tokio::test]
    async fn platform_admin_is_global_admin_resolved_through_the_port() {
        let r = PortAdminScopeResolver::new(port_holding("global", &["user", "admin"]));
        assert!(r.is_platform_admin("u1").await.unwrap());

        // A product admin is not a platform admin, however senior it looks.
        let r = PortAdminScopeResolver::new(port_holding("music", &["admin"]));
        assert!(!r.is_platform_admin("u1").await.unwrap());

        // Holding `global` without `admin` is not enough either.
        let r = PortAdminScopeResolver::new(port_holding("global", &["user"]));
        assert!(!r.is_platform_admin("u1").await.unwrap());
    }

    /// The resolver asks for exactly one scope. Asserted because asking for more would
    /// make a `music/admin` look global the day the filter above is relaxed.
    #[tokio::test]
    async fn the_resolver_asks_only_about_the_global_scope() {
        let mut m = cymbra_user_port::MockUserPort::new();
        m.expect_scoped_effective_roles()
            .withf(|_uid, asked| asked == ["global".to_string()])
            .times(1)
            .returning(|_, _| Ok(Default::default()));
        let r = PortAdminScopeResolver::new(Arc::new(m));
        assert!(!r.is_platform_admin("u1").await.unwrap());
    }

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

    /// Online translation access is the flag evaluated with the caller's LINGUA betas: a
    /// member of `online-translation` gets it, a non-member does not, nobody without plans,
    /// and the flag off withdraws it from the member too (change: add-lingua-online-translation).
    #[tokio::test]
    async fn online_translation_access_follows_the_lingua_beta() {
        use cymbra_lingua::online_translation::OnlineTranslationPolicy;
        let scoped = |on: bool| StoredOverride {
            app: "lingua".into(),
            key: registry::LINGUA_ONLINE_TRANSLATION_ENABLED.into(),
            value_type: cymbra_feature_flags::ValueType::Bool,
            value: FlagValue::Bool(on),
            rollout: RolloutScope::Beta("online-translation".into()),
            sensitive: false,
            updated_by: "op".into(),
            updated_at: chrono::Utc::now(),
        };
        let reader = |uid: &str| cymbra_platform::AuthIdentity {
            user_id: uid.into(),
            audience: "lingua".into(),
            roles: vec!["user".into()],
            ..Default::default()
        };
        let mut plans = cymbra_feature_flags::context::MockPlanContextSource::new();
        plans
            .expect_plan_context()
            .withf(|_, app| app == "lingua")
            .returning(|uid, _| {
                Ok((
                    false,
                    if uid == "member" {
                        vec!["online-translation".to_string()]
                    } else {
                        vec![]
                    },
                ))
            });
        let plans: Arc<dyn cymbra_feature_flags::PlanContextSource> = Arc::new(plans);

        let on = FlagOnlineTranslationPolicy::new(
            service(vec![scoped(true)]).await,
            Some(plans.clone()),
        );
        assert!(on.has_access(&reader("member")).await.unwrap());
        assert!(!on.has_access(&reader("other")).await.unwrap());
        let no_plans = FlagOnlineTranslationPolicy::new(service(vec![scoped(true)]).await, None);
        assert!(!no_plans.has_access(&reader("member")).await.unwrap());
        let off = FlagOnlineTranslationPolicy::new(service(vec![scoped(false)]).await, Some(plans));
        assert!(!off.has_access(&reader("member")).await.unwrap());

        // The limit and the prices are plain Lingua config, defaults until overridden.
        assert_eq!(on.daily_limit().await, 100);
        assert_eq!(on.prices("gemini").await, (0.25, 1.50));
        assert_eq!(on.prices("anthropic").await, (0.10, 0.50));
    }

    /// The usage counts' kill switch and ceiling are plain Lingua values: on and 5 000 until
    /// an operator overrides them (change: add-lingua-usage-counts).
    #[tokio::test]
    async fn usage_counts_policy_reads_the_switch_and_the_ceiling() {
        use cymbra_lingua::usage_counts::UsageCountsPolicy;
        let lingua = |key: &str, value: FlagValue| StoredOverride {
            app: "lingua".into(),
            key: key.into(),
            value_type: value.value_type(),
            value,
            rollout: RolloutScope::Global,
            sensitive: false,
            updated_by: "op".into(),
            updated_at: chrono::Utc::now(),
        };
        let defaults = FlagUsageCountsPolicy::new(service(vec![]).await);
        assert!(defaults.enabled());
        assert_eq!(defaults.ceiling(), 5_000);
        let overridden = FlagUsageCountsPolicy::new(
            service(vec![
                lingua(
                    registry::LINGUA_USAGE_COUNTS_ENABLED,
                    FlagValue::Bool(false),
                ),
                lingua(registry::LINGUA_USAGE_COUNTS_CEILING, FlagValue::Int(300)),
            ])
            .await,
        );
        assert!(!overridden.enabled());
        assert_eq!(overridden.ceiling(), 300);
        // A negative ceiling counts nothing rather than misbehaving.
        let negative = FlagUsageCountsPolicy::new(
            service(vec![lingua(
                registry::LINGUA_USAGE_COUNTS_CEILING,
                FlagValue::Int(-1),
            )])
            .await,
        );
        assert_eq!(negative.ceiling(), 0);
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
