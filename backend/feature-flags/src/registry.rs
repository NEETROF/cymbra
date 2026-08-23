//! The code registry of declared keys (design D7).
//!
//! Every flag/config key is declared here with a typed default, its app scope,
//! default rollout intent, safe fail-direction, sensitivity, and a short doc.
//! An absent stored override resolves to the code default; only declared keys are
//! editable. The back office lists exactly this registry (no free-form typos).
//!
//! Feature on/off flags default to their **safe (disabled)** state so a store
//! outage — or simply the feature not yet being turned on — never silently enables
//! an unshipped capability (design D2 fail-direction). Enforcement gates for each
//! feature are wired as those features land (task 2.1); this change only declares
//! the keys + serves the platform.

use crate::context::{APP_ALL, RolloutScope};
use crate::value::{FlagValue, ValueType};
use serde_json::json;

/// The `music` app scope.
pub const APP_MUSIC: &str = "music";

// --- key names (stable identifiers; features read these) --------------------

// Feature on/off flags (safe state = disabled).
pub const RATING_ENABLED: &str = "rating.enabled";
pub const REWARDS_ENABLED: &str = "rewards.enabled";
pub const REWARDS_SHOP_ENABLED: &str = "rewards.shop.enabled";
pub const PROFILES_PUBLIC_ENABLED: &str = "profiles.public.enabled";
pub const LEADERBOARD_PER_PIECE_ENABLED: &str = "leaderboard.per_piece.enabled";
pub const LEADERBOARD_GLOBAL_ENABLED: &str = "leaderboard.global.enabled";
pub const ONBOARDING_ENABLED: &str = "onboarding.enabled";
/// The drum feature (change: add-drums-access). Intended rollout:
/// `beta:midi-drums` during the beta, then `global` at general availability —
/// widen the scope FIRST, close the campaign SECOND, and keep the flag as a
/// kill-switch afterwards. The backend `music` module enforces it on every
/// path that can disclose or accept a percussion score.
pub const DRUMS_ENABLED: &str = "drums.enabled";

/// Shared cross-app kill-switch: when on, apps show an "under maintenance" state.
pub const PLATFORM_MAINTENANCE: &str = "platform.maintenance";

/// Feature-usage analytics collection kill-switch (change: add-feature-usage-
/// analytics, design D5). Unlike ordinary feature flags this defaults **on**
/// (collection is an opt-out first-party audience-measurement regime); flipping it
/// off stops clients emitting usage events without a client release. Distinct from
/// the per-user consent toggle — either one off suppresses emission.
pub const ANALYTICS_COLLECTION_ENABLED: &str = "analytics.collection.enabled";

// Config tunables (the scattered straw-man values live here).
pub const RATING_REVIEW_MIN_VOTES: &str = "rating.review.min_votes";
pub const RATING_REVIEW_THRESHOLD: &str = "rating.review.threshold";
pub const REWARDS_POINTS_DAILY_CAP: &str = "rewards.points.daily_cap";
pub const REWARDS_POINTS_BANDS: &str = "rewards.points.bands";
pub const REWARDS_LEVELS: &str = "rewards.levels";
pub const REWARDS_SHOP_COSTS: &str = "rewards.shop.costs";
pub const LEADERBOARD_GLOBAL_BEST_N: &str = "leaderboard.global.best_n";
/// Points a confirmed practice-streak freeze costs (change: add-practice-streak,
/// task 2.1). Read per call by the streak module, so retuning the price in the
/// back office changes the next offer with no redeploy.
pub const STREAK_FREEZE_COST: &str = "streak.freeze_cost";
/// How many MISSED days a broken practice streak may still be recovered from
/// (change: add-practice-streak). `1` = "you can buy back yesterday's break
/// today"; `0` disables recovery entirely.
pub const STREAK_GRACE_DAYS: &str = "streak.grace_days";
pub const LEADERBOARD_DIFFICULTY_WEIGHTS: &str = "leaderboard.difficulty_weights";
pub const LEADERBOARD_SEASON_LENGTH_DAYS: &str = "leaderboard.season.length_days";

// --- catalog daily access — the freemium gate on catalog player-opens (change:
// add-score-daily-access-rewards, design D1). Read per call by the music module
// through a trait seam, so every value hot-reloads. The gate is OFF by default
// (safe direction: every open keeps being served); roll out staff-only first.

/// Kill-switch of the daily free-open quota. Off = no gate, data kept.
pub const CATALOG_DAILY_ACCESS_ENABLED: &str = "catalog.daily_access.enabled";
/// Distinct catalog pieces a user may open for free per server day (`0` = every
/// catalog open costs points).
pub const CATALOG_DAILY_ACCESS_FREE_QUOTA: &str = "catalog.daily_access.free_quota";
/// Points one extra piece for the day costs (`0` = free unlock, no effective gate).
pub const CATALOG_DAILY_ACCESS_DAY_SLOT_COST: &str = "catalog.daily_access.day_slot_cost";
/// Maximum length in ms of the audio teaser rendered for a catalog piece.
pub const CATALOG_PREVIEW_MAX_MS: &str = "catalog.preview.max_ms";
/// Id of the ACCEPTED catalog SoundFont the score teasers are rendered with.
/// Empty = previews are dormant (nothing rendered, nothing broken).
pub const CATALOG_PREVIEW_SOUNDFONT_ID: &str = "catalog.preview.soundfont_id";

// --- catalog access limits — the per-user scrape guardrail on catalog egress
// (change: add-catalog-access-limits, task 1.3). Read per request by the score
// service through a trait seam, so an operator retunes a threshold — or trips the
// kill-switch when the guardrail misfires — with no redeploy. Unlike most gates
// this one defaults **on**: leaving egress unlimited is the unsafe direction. The
// declared defaults mirror the deployment baseline parsed from the environment
// (`cymbra_platform::config::CatalogLimitsConfig`), which is what a call site
// passes as its own default, so an unset override changes nothing.

/// Kill-switch of the whole catalog access guardrail. Off = no burst cap, no
/// volume allowance, no enumeration cap.
pub const CATALOG_LIMITS_ENABLED: &str = "catalog.access_limits.enabled";
/// Max raw-bytes downloads per burst window (pure rate, everyone).
pub const CATALOG_LIMITS_DL_BURST_MAX: &str = "catalog.access_limits.download.burst_max";
/// Length in SECONDS of the download burst window.
pub const CATALOG_LIMITS_DL_BURST_WINDOW_S: &str = "catalog.access_limits.download.burst_window_s";
/// Length in SECONDS of the rolling window the volume allowance is counted over.
pub const CATALOG_LIMITS_DL_VOLUME_WINDOW_S: &str =
    "catalog.access_limits.download.volume_window_s";
/// Downloads always allowed in the volume window regardless of engagement.
pub const CATALOG_LIMITS_DL_BASE_FLOOR: &str = "catalog.access_limits.download.base_floor";
/// Extra download headroom earned per in-window engagement event (a play session
/// or a score rating).
pub const CATALOG_LIMITS_DL_PER_ENGAGEMENT: &str = "catalog.access_limits.download.per_engagement";
/// Absolute ceiling on the volume allowance, whatever the engagement.
pub const CATALOG_LIMITS_DL_HARD_CEILING: &str = "catalog.access_limits.download.hard_ceiling";
/// Max enumeration requests (search / browse / rating deck) per window.
pub const CATALOG_LIMITS_ENUM_MAX: &str = "catalog.access_limits.enum_max";
/// Length in SECONDS of the enumeration window.
pub const CATALOG_LIMITS_ENUM_WINDOW_S: &str = "catalog.access_limits.enum_window_s";

// --- plans & billing (change: add-premium-subscription). Read per call by the
// plans module + the music quotas through trait seams. `plans.enabled` is the
// kill-switch of the whole plan system (off = everyone is `free`, no memberships,
// exactly the pre-plan behaviour); each purchase channel has its own switch that
// hides the paywall's purchase button and makes the provider notification route
// acknowledge-and-ignore. Both kinds are sensitive (they change what is sold).

/// Kill-switch of the plan system: off ⇒ every plan-aware seam answers `free`.
pub const PLANS_ENABLED: &str = "plans.enabled";
/// Days a row in provider billing-retry stays active past `ends_at`.
pub const PLANS_GRACE_DAYS: &str = "plans.grace_days";
/// Store/MoR product ids offered on the paywall (JSON array of strings; prices come
/// from the store).
pub const PLANS_PREMIUM_PRODUCTS: &str = "plans.premium.products";
/// Private `.sf2` library cap for the free plan (today's constant).
pub const PLANS_SF_LIBRARY_MAX_FREE: &str = "plans.soundfont_library.max_fonts.free";
/// Private `.sf2` library cap for a plan granting `soundfont_library.extended`.
pub const PLANS_SF_LIBRARY_MAX_PREMIUM: &str = "plans.soundfont_library.max_fonts.premium";
/// Rolling score-upload quota for the free plan (JSON `{max, window_days}`).
pub const PLANS_SCORES_UPLOAD_QUOTA_FREE: &str = "plans.scores.upload_quota.free";
/// Rolling score-upload quota for a plan granting `scores.extended_quotas`.
pub const PLANS_SCORES_UPLOAD_QUOTA_PREMIUM: &str = "plans.scores.upload_quota.premium";
/// Private score-library cap for the free plan (uploads not yet accepted into the
/// public catalog).
pub const PLANS_SCORES_LIBRARY_MAX_FREE: &str = "plans.scores.library_max.free";
/// Private score-library cap for a plan granting `scores.extended_quotas`.
pub const PLANS_SCORES_LIBRARY_MAX_PREMIUM: &str = "plans.scores.library_max.premium";
/// Apple purchase channel (App Store): paywall button + notification route.
pub const BILLING_APPLE_ENABLED: &str = "billing.apple.enabled";
/// Google purchase channel (Play): paywall button + notification route.
pub const BILLING_GOOGLE_ENABLED: &str = "billing.google.enabled";
/// Web merchant-of-record channel: checkout creation + webhook route.
pub const BILLING_WEB_ENABLED: &str = "billing.web.enabled";

// Sensitive legal/infra values (not casually editable).
pub const ACCOUNT_MIN_PUBLIC_SHARING_AGE: &str = "account.min_public_sharing_age";
pub const DATA_RETENTION_PLAY_DETAIL_DAYS: &str = "data.retention.play_detail_days";
/// Raw feature-usage-event retention window in DAYS (change: add-feature-usage-
/// analytics, design D4). The worker's purge job deletes `analytics.usage_events`
/// older than this; the permanent daily aggregates are unaffected. Default 180 (≈6
/// months). BO-editable so retention is retuned without a redeploy.
pub const DATA_RETENTION_USAGE_EVENTS_DAYS: &str = "data.retention.usage_events_days";

// --- push notifications (change: add-push-notifications, design D4) ----------
//
// The platform declares ONE key — the global kill-switch. Every other key is
// per-**category**, and categories are owned by the features that declare them,
// so a feature adds its own three keys (built by [`category_enabled_key`],
// [`category_hour_key`] and [`category_foreground_key`]) to this registry when
// its notification type lands. The back office renders whatever it finds under
// the [`NOTIFICATIONS_KEY_PREFIX`].

/// Common prefix of every push-notification key — how the back-office panel
/// discovers the categories currently declared.
pub const NOTIFICATIONS_KEY_PREFIX: &str = "notifications.";

/// Global push kill-switch. Off = **no** notification is sent for any category,
/// whatever the per-category flags or user preferences say. Defaults off (the
/// safe direction): a store outage, or simply an unconfigured FCM project, must
/// never result in messaging users.
pub const NOTIFICATIONS_ENABLED: &str = "notifications.enabled";

/// The per-category enable key for `category`, e.g.
/// `notifications.category.practice_streak.enabled`.
pub fn category_enabled_key(category: &str) -> String {
    format!("{NOTIFICATIONS_KEY_PREFIX}category.{category}.enabled")
}

/// The per-category schedule-hour key for `category`, e.g.
/// `notifications.category.practice_streak.hour`. The value is the **local** hour
/// (0–23) at which the category's scheduled send fires for each user.
pub fn category_hour_key(category: &str) -> String {
    format!("{NOTIFICATIONS_KEY_PREFIX}category.{category}.hour")
}

/// The per-category foreground-presentation key for `category`, e.g.
/// `notifications.category.practice_streak.foreground` — the third per-category
/// key (change: add-foreground-notifications). On = a notification for this
/// category arriving while the app is open is surfaced in-app; off = it stays
/// silent. Call sites default it to `false`: not interrupting is the safe
/// direction, matching every other gate on this platform.
pub fn category_foreground_key(category: &str) -> String {
    format!("{NOTIFICATIONS_KEY_PREFIX}category.{category}.foreground")
}

/// The category id of the evening practice-streak reminder (change:
/// add-practice-streak, task 3.1) — the first concrete notification type on this
/// platform. Its three keys are declared as literals in [`builtin`] (the
/// declaration table needs `'static` names); a test pins them to the builders
/// above so the two can never drift.
pub const CATEGORY_PRACTICE_STREAK: &str = "practice_streak";

/// A single declared key.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyDef {
    pub key: &'static str,
    /// `all` (shared) or a specific app (`music`, `live`, …).
    pub app: &'static str,
    pub value_type: ValueType,
    /// The safe code default used when there is no applicable override.
    pub default: FlagValue,
    /// The default rollout intent shown in the BO; the actual override's rollout
    /// is what an evaluation checks.
    pub rollout: RolloutScope,
    /// Legal/infra value — visibly distinguished, requires confirmation to change.
    pub sensitive: bool,
    pub doc: &'static str,
}

fn flag(
    key: &'static str,
    app: &'static str,
    default: bool,
    sensitive: bool,
    doc: &'static str,
) -> KeyDef {
    KeyDef {
        key,
        app,
        value_type: ValueType::Bool,
        default: FlagValue::Bool(default),
        rollout: RolloutScope::Global,
        sensitive,
        doc,
    }
}

fn cfg(
    key: &'static str,
    app: &'static str,
    default: FlagValue,
    sensitive: bool,
    doc: &'static str,
) -> KeyDef {
    KeyDef {
        key,
        app,
        value_type: default.value_type(),
        default,
        rollout: RolloutScope::Global,
        sensitive,
        doc,
    }
}

/// Every declared key. This is the canonical, editable set.
pub fn builtin() -> Vec<KeyDef> {
    vec![
        // -- feature flags (safe state disabled) --
        flag(
            RATING_ENABLED,
            APP_MUSIC,
            false,
            false,
            "Score re-rating / review flow.",
        ),
        flag(
            REWARDS_ENABLED,
            APP_MUSIC,
            false,
            false,
            "Curation reward points.",
        ),
        flag(
            REWARDS_SHOP_ENABLED,
            APP_MUSIC,
            false,
            false,
            "Reward shop redemptions.",
        ),
        flag(
            PROFILES_PUBLIC_ENABLED,
            APP_MUSIC,
            false,
            false,
            "Public player profiles.",
        ),
        flag(
            LEADERBOARD_PER_PIECE_ENABLED,
            APP_MUSIC,
            false,
            false,
            "Per-piece performance leaderboards.",
        ),
        flag(
            LEADERBOARD_GLOBAL_ENABLED,
            APP_MUSIC,
            false,
            false,
            "The global performance leaderboard.",
        ),
        flag(
            DRUMS_ENABLED,
            APP_MUSIC,
            false,
            false,
            "MIDI drums: percussion scores are visible/acceptable for the caller.",
        ),
        flag(
            ONBOARDING_ENABLED,
            APP_MUSIC,
            false,
            false,
            "Welcome onboarding flow.",
        ),
        flag(
            PLATFORM_MAINTENANCE,
            APP_ALL,
            false,
            false,
            "Shared kill-switch: put every app into an under-maintenance state.",
        ),
        // Analytics collection kill-switch: defaults ON (opt-out audience
        // measurement, design D5) — deliberately NOT in the default-off set below.
        flag(
            ANALYTICS_COLLECTION_ENABLED,
            APP_ALL,
            true,
            false,
            "Feature-usage analytics collection master switch (defaults on; flip off to stop all clients emitting usage events without a release).",
        ),
        // Push kill-switch: safe state is OFF — an unconfigured or misbehaving
        // send path must never message users (change: add-push-notifications).
        flag(
            NOTIFICATIONS_ENABLED,
            APP_ALL,
            false,
            false,
            "Global push-notification kill-switch: off suppresses every category's sends.",
        ),
        // The practice-streak reminder category (change: add-practice-streak,
        // task 3.1). Declared under APP_ALL: these are server-side SEND
        // configuration, evaluated by the worker, which has no app context.
        // Enabling it is deliberately a back-office act — merging this code
        // messages nobody.
        flag(
            "notifications.category.practice_streak.enabled",
            APP_ALL,
            false,
            false,
            "Evening reminder to players whose practice streak is about to break.",
        ),
        cfg(
            "notifications.category.practice_streak.hour",
            APP_ALL,
            FlagValue::Int(20),
            false,
            "Local hour (0-23) the practice-streak reminder fires for each player.",
        ),
        flag(
            "notifications.category.practice_streak.foreground",
            APP_ALL,
            false,
            false,
            "Surface the streak reminder in-app when it arrives with the app open.",
        ),
        // -- config tunables --
        cfg(
            RATING_REVIEW_MIN_VOTES,
            APP_MUSIC,
            FlagValue::Int(5),
            false,
            "Votes required before a score's rating is re-reviewed.",
        ),
        cfg(
            RATING_REVIEW_THRESHOLD,
            APP_MUSIC,
            FlagValue::Number(2.0),
            false,
            "Average-rating threshold below which a score is flagged for review.",
        ),
        cfg(
            REWARDS_POINTS_DAILY_CAP,
            APP_MUSIC,
            FlagValue::Int(100),
            false,
            "Max reward points a user can earn per day.",
        ),
        cfg(
            REWARDS_POINTS_BANDS,
            APP_MUSIC,
            FlagValue::Json(json!({"rate": 5, "upload": 20, "first_of_day": 10})),
            false,
            "Point award bands per curation action.",
        ),
        cfg(
            REWARDS_LEVELS,
            APP_MUSIC,
            FlagValue::Json(json!([0, 100, 500, 2000])),
            false,
            "Cumulative point thresholds for each reward level.",
        ),
        cfg(
            REWARDS_SHOP_COSTS,
            APP_MUSIC,
            FlagValue::Json(json!({"theme": 200, "badge": 50})),
            false,
            "Reward-shop item costs in points.",
        ),
        cfg(
            LEADERBOARD_GLOBAL_BEST_N,
            APP_MUSIC,
            FlagValue::Int(20),
            false,
            "How many best scores feed a user's global standing.",
        ),
        cfg(
            LEADERBOARD_DIFFICULTY_WEIGHTS,
            APP_MUSIC,
            FlagValue::Json(json!({"easy": 1.0, "medium": 1.5, "hard": 2.0})),
            false,
            "Per-difficulty score multipliers for the leaderboard.",
        ),
        cfg(
            LEADERBOARD_SEASON_LENGTH_DAYS,
            APP_MUSIC,
            FlagValue::Int(90),
            false,
            "Length of a leaderboard season in days.",
        ),
        cfg(
            STREAK_FREEZE_COST,
            APP_MUSIC,
            FlagValue::Int(30),
            false,
            "Points a confirmed practice-streak freeze costs.",
        ),
        cfg(
            STREAK_GRACE_DAYS,
            APP_MUSIC,
            FlagValue::Int(1),
            false,
            "Missed days a broken practice streak may still be recovered from (0 disables recovery).",
        ),
        // -- catalog daily access (freemium gate on catalog opens) --
        flag(
            CATALOG_DAILY_ACCESS_ENABLED,
            APP_MUSIC,
            false,
            false,
            "Daily free-open quota on catalog pieces (off = every open served).",
        ),
        cfg(
            CATALOG_DAILY_ACCESS_FREE_QUOTA,
            APP_MUSIC,
            FlagValue::Int(3),
            false,
            "Distinct catalog pieces a user may open for free per server day.",
        ),
        cfg(
            CATALOG_DAILY_ACCESS_DAY_SLOT_COST,
            APP_MUSIC,
            FlagValue::Int(20),
            false,
            "Points one extra catalog piece for the day costs.",
        ),
        cfg(
            CATALOG_PREVIEW_MAX_MS,
            APP_MUSIC,
            FlagValue::Int(30_000),
            false,
            "Maximum length (ms) of the audio teaser rendered for a catalog piece.",
        ),
        cfg(
            CATALOG_PREVIEW_SOUNDFONT_ID,
            APP_MUSIC,
            FlagValue::String(String::new()),
            false,
            "Id of the accepted catalog SoundFont score teasers are rendered with (empty = dormant).",
        ),
        // -- catalog access limits (per-user scrape guardrail on catalog egress).
        // Defaults ON and mirror the env baseline: unlimited egress is the unsafe
        // direction, so this switch exists to turn the guardrail OFF if it misfires.
        flag(
            CATALOG_LIMITS_ENABLED,
            APP_MUSIC,
            true,
            false,
            "Per-user catalog access limits (defaults on; flip off to disable the scrape guardrail without a redeploy).",
        ),
        cfg(
            CATALOG_LIMITS_DL_BURST_MAX,
            APP_MUSIC,
            FlagValue::Int(20),
            false,
            "Max score-bytes downloads a user may make per burst window.",
        ),
        cfg(
            CATALOG_LIMITS_DL_BURST_WINDOW_S,
            APP_MUSIC,
            FlagValue::Int(60),
            false,
            "Length in seconds of the download burst window.",
        ),
        cfg(
            CATALOG_LIMITS_DL_VOLUME_WINDOW_S,
            APP_MUSIC,
            FlagValue::Int(24 * 3600),
            false,
            "Length in seconds of the rolling window the download volume allowance is counted over.",
        ),
        cfg(
            CATALOG_LIMITS_DL_BASE_FLOOR,
            APP_MUSIC,
            FlagValue::Int(30),
            false,
            "Downloads allowed per volume window regardless of engagement (the floor).",
        ),
        cfg(
            CATALOG_LIMITS_DL_PER_ENGAGEMENT,
            APP_MUSIC,
            FlagValue::Int(3),
            false,
            "Extra downloads earned per in-window engagement event (a play session or a score rating).",
        ),
        cfg(
            CATALOG_LIMITS_DL_HARD_CEILING,
            APP_MUSIC,
            FlagValue::Int(500),
            false,
            "Absolute ceiling on the download volume allowance, whatever the engagement.",
        ),
        cfg(
            CATALOG_LIMITS_ENUM_MAX,
            APP_MUSIC,
            FlagValue::Int(60),
            false,
            "Max catalog enumeration requests (search / browse / rating deck) per window.",
        ),
        cfg(
            CATALOG_LIMITS_ENUM_WINDOW_S,
            APP_MUSIC,
            FlagValue::Int(60),
            false,
            "Length in seconds of the catalog enumeration window.",
        ),
        // -- plans & billing (change: add-premium-subscription) --
        flag(
            PLANS_ENABLED,
            APP_MUSIC,
            false,
            true,
            "Plan system kill-switch: off = everyone is free, no betas, no paywall.",
        ),
        cfg(
            PLANS_GRACE_DAYS,
            APP_MUSIC,
            FlagValue::Int(3),
            false,
            "Days a subscription in provider billing-retry stays active past its end.",
        ),
        cfg(
            PLANS_PREMIUM_PRODUCTS,
            APP_MUSIC,
            FlagValue::Json(json!(["premium_monthly", "premium_yearly"])),
            false,
            "Store/MoR product ids the paywall offers (prices come from the store).",
        ),
        cfg(
            PLANS_SF_LIBRARY_MAX_FREE,
            APP_MUSIC,
            FlagValue::Int(5),
            false,
            "Private .sf2 library cap on the free plan.",
        ),
        cfg(
            PLANS_SF_LIBRARY_MAX_PREMIUM,
            APP_MUSIC,
            FlagValue::Int(50),
            false,
            "Private .sf2 library cap on a plan with the extended library unlock.",
        ),
        cfg(
            PLANS_SCORES_UPLOAD_QUOTA_FREE,
            APP_MUSIC,
            FlagValue::Json(json!({"max": 5, "window_days": 7})),
            false,
            "Rolling score-upload quota on the free plan ({max, window_days}).",
        ),
        cfg(
            PLANS_SCORES_UPLOAD_QUOTA_PREMIUM,
            APP_MUSIC,
            FlagValue::Json(json!({"max": 50, "window_days": 7})),
            false,
            "Rolling score-upload quota on a plan with the extended score quotas.",
        ),
        cfg(
            PLANS_SCORES_LIBRARY_MAX_FREE,
            APP_MUSIC,
            FlagValue::Int(20),
            false,
            "Private score-library cap on the free plan (accepted catalog scores excluded).",
        ),
        cfg(
            PLANS_SCORES_LIBRARY_MAX_PREMIUM,
            APP_MUSIC,
            FlagValue::Int(500),
            false,
            "Private score-library cap on a plan with the extended score quotas.",
        ),
        flag(
            BILLING_APPLE_ENABLED,
            APP_MUSIC,
            false,
            true,
            "Apple purchase channel: paywall button + App Store notification route.",
        ),
        flag(
            BILLING_GOOGLE_ENABLED,
            APP_MUSIC,
            false,
            true,
            "Google purchase channel: paywall button + Play RTDN route.",
        ),
        flag(
            BILLING_WEB_ENABLED,
            APP_MUSIC,
            false,
            true,
            "Web merchant-of-record channel: checkout + webhook route.",
        ),
        // -- sensitive legal/infra values --
        cfg(
            ACCOUNT_MIN_PUBLIC_SHARING_AGE,
            APP_ALL,
            FlagValue::Int(16),
            true,
            "Minimum age to make a profile public (legal — EU digital-consent age).",
        ),
        cfg(
            DATA_RETENTION_PLAY_DETAIL_DAYS,
            APP_ALL,
            FlagValue::Int(90),
            true,
            "Days the heavy per-session play detail is retained before pruning.",
        ),
        cfg(
            DATA_RETENTION_USAGE_EVENTS_DAYS,
            APP_ALL,
            FlagValue::Int(180),
            true,
            "Days raw feature-usage events are retained before the purge job deletes them (permanent aggregates are unaffected).",
        ),
    ]
}

/// A code registry: fast lookup of declared keys by `(app, key)`.
#[derive(Debug, Clone)]
pub struct Registry {
    defs: Vec<KeyDef>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new(builtin())
    }
}

impl Registry {
    pub fn new(defs: Vec<KeyDef>) -> Self {
        Self { defs }
    }

    /// All declared keys.
    pub fn all(&self) -> &[KeyDef] {
        &self.defs
    }

    /// Look up a declaration by its `(app, key)` pair. The app must match exactly
    /// (a `music` key is not the same identifier as an `all` key of the same name).
    pub fn get(&self, app: &str, key: &str) -> Option<&KeyDef> {
        self.defs.iter().find(|d| d.app == app && d.key == key)
    }

    /// Look up by key name alone (used for direct server-side evaluation where the
    /// caller knows the key's app is `all` or its own).
    pub fn get_by_key(&self, key: &str) -> Option<&KeyDef> {
        self.defs.iter().find(|d| d.key == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn keys_are_unique_per_app() {
        let mut seen = HashSet::new();
        for d in builtin() {
            assert!(
                seen.insert((d.app, d.key)),
                "duplicate key {}/{}",
                d.app,
                d.key
            );
        }
    }

    #[test]
    fn declared_defaults_match_declared_type() {
        for d in builtin() {
            assert_eq!(
                d.default.value_type(),
                d.value_type,
                "type mismatch for {}",
                d.key
            );
        }
    }

    #[test]
    fn feature_flags_default_off_and_are_safe() {
        let r = Registry::default();
        for key in [
            RATING_ENABLED,
            REWARDS_ENABLED,
            REWARDS_SHOP_ENABLED,
            PROFILES_PUBLIC_ENABLED,
            LEADERBOARD_PER_PIECE_ENABLED,
            LEADERBOARD_GLOBAL_ENABLED,
            ONBOARDING_ENABLED,
            PLATFORM_MAINTENANCE,
            NOTIFICATIONS_ENABLED,
        ] {
            assert_eq!(r.get_by_key(key).unwrap().default, FlagValue::Bool(false));
        }
    }

    #[test]
    fn notification_category_keys_are_namespaced_under_the_panel_prefix() {
        // The BO panel discovers categories by prefix, so all three per-category
        // keys must sit under it alongside the kill-switch.
        assert!(NOTIFICATIONS_ENABLED.starts_with(NOTIFICATIONS_KEY_PREFIX));
        assert_eq!(
            category_enabled_key("practice_streak"),
            "notifications.category.practice_streak.enabled"
        );
        assert_eq!(
            category_hour_key("practice_streak"),
            "notifications.category.practice_streak.hour"
        );
        assert_eq!(
            category_foreground_key("practice_streak"),
            "notifications.category.practice_streak.foreground"
        );
        for k in [
            category_enabled_key("x"),
            category_hour_key("x"),
            category_foreground_key("x"),
        ] {
            assert!(k.starts_with(NOTIFICATIONS_KEY_PREFIX), "{k}");
        }
    }

    #[test]
    fn every_declared_category_is_owned_by_a_feature() {
        // The platform itself declares only the kill-switch; each category's three
        // keys come from the feature that owns the type (design D6). Today the
        // only such feature is the practice-streak reminder — this pins the set so
        // a stray `notifications.category.*` key cannot appear unowned.
        let declared: Vec<&str> = builtin()
            .into_iter()
            .filter(|d| d.key.starts_with("notifications.category."))
            .map(|d| d.key)
            .collect();
        let expected = [
            category_enabled_key(CATEGORY_PRACTICE_STREAK),
            category_hour_key(CATEGORY_PRACTICE_STREAK),
            category_foreground_key(CATEGORY_PRACTICE_STREAK),
        ];
        assert_eq!(declared.len(), expected.len(), "declared: {declared:?}");
        for key in expected {
            assert!(declared.contains(&key.as_str()), "missing {key}");
        }
    }

    #[test]
    fn the_streak_reminder_category_ships_silent_but_scheduled() {
        let r = Registry::default();
        // Enabling a category that messages real users is a back-office act.
        for key in [
            category_enabled_key(CATEGORY_PRACTICE_STREAK),
            category_foreground_key(CATEGORY_PRACTICE_STREAK),
        ] {
            assert_eq!(
                r.get_by_key(&key).unwrap().default,
                FlagValue::Bool(false),
                "{key} must default off"
            );
        }
        // ...but the hour is pre-set to a sensible evening slot, so turning the
        // category on is the only decision an operator has to make.
        assert_eq!(
            r.get_by_key(&category_hour_key(CATEGORY_PRACTICE_STREAK))
                .unwrap()
                .default,
            FlagValue::Int(20)
        );
    }

    #[test]
    fn streak_freeze_tunables_have_the_designed_defaults() {
        let r = Registry::default();
        assert_eq!(
            r.get(APP_MUSIC, STREAK_FREEZE_COST).unwrap().default,
            FlagValue::Int(30)
        );
        // One missed day of grace: forgiving enough to rescue a slip, short
        // enough that a streak still means something.
        assert_eq!(
            r.get(APP_MUSIC, STREAK_GRACE_DAYS).unwrap().default,
            FlagValue::Int(1)
        );
    }

    /// The scrape guardrail is the one gate whose safe state is ON, and its
    /// declared defaults must mirror the env baseline in
    /// `cymbra_platform::config::CatalogLimitsConfig` — otherwise the value an
    /// operator sees in the back office is not the value in force.
    #[test]
    fn catalog_access_limits_default_on_with_the_env_baseline() {
        let r = Registry::default();
        assert_eq!(
            r.get(APP_MUSIC, CATALOG_LIMITS_ENABLED).unwrap().default,
            FlagValue::Bool(true),
            "unlimited egress is the unsafe direction"
        );
        for (key, expected) in [
            (CATALOG_LIMITS_DL_BURST_MAX, 20),
            (CATALOG_LIMITS_DL_BURST_WINDOW_S, 60),
            (CATALOG_LIMITS_DL_VOLUME_WINDOW_S, 86_400),
            (CATALOG_LIMITS_DL_BASE_FLOOR, 30),
            (CATALOG_LIMITS_DL_PER_ENGAGEMENT, 3),
            (CATALOG_LIMITS_DL_HARD_CEILING, 500),
            (CATALOG_LIMITS_ENUM_MAX, 60),
            (CATALOG_LIMITS_ENUM_WINDOW_S, 60),
        ] {
            assert_eq!(
                r.get(APP_MUSIC, key).unwrap().default,
                FlagValue::Int(expected),
                "{key}"
            );
        }
    }

    #[test]
    fn straw_man_defaults_present() {
        let r = Registry::default();
        assert_eq!(
            r.get(APP_MUSIC, RATING_REVIEW_MIN_VOTES).unwrap().default,
            FlagValue::Int(5)
        );
        assert_eq!(
            r.get(APP_MUSIC, RATING_REVIEW_THRESHOLD).unwrap().default,
            FlagValue::Number(2.0)
        );
        assert_eq!(
            r.get(APP_MUSIC, LEADERBOARD_GLOBAL_BEST_N).unwrap().default,
            FlagValue::Int(20)
        );
    }

    #[test]
    fn legal_infra_values_are_sensitive() {
        let r = Registry::default();
        assert!(
            r.get(APP_ALL, ACCOUNT_MIN_PUBLIC_SHARING_AGE)
                .unwrap()
                .sensitive
        );
        assert!(
            r.get(APP_ALL, DATA_RETENTION_PLAY_DETAIL_DAYS)
                .unwrap()
                .sensitive
        );
        // ordinary flags are not sensitive
        assert!(!r.get_by_key(RATING_ENABLED).unwrap().sensitive);
        // usage-event retention is a legal/infra value
        assert!(
            r.get(APP_ALL, DATA_RETENTION_USAGE_EVENTS_DAYS)
                .unwrap()
                .sensitive
        );
    }

    #[test]
    fn analytics_kill_switch_defaults_on() {
        // The analytics collection kill-switch is the one bool that defaults ON
        // (opt-out audience measurement, design D5) — so it is intentionally NOT
        // in `feature_flags_default_off_and_are_safe`.
        let r = Registry::default();
        assert_eq!(
            r.get(APP_ALL, ANALYTICS_COLLECTION_ENABLED)
                .unwrap()
                .default,
            FlagValue::Bool(true)
        );
    }

    #[test]
    fn usage_retention_default_is_six_months() {
        let r = Registry::default();
        assert_eq!(
            r.get(APP_ALL, DATA_RETENTION_USAGE_EVENTS_DAYS)
                .unwrap()
                .default,
            FlagValue::Int(180)
        );
    }

    #[test]
    fn shared_keys_scoped_all() {
        let r = Registry::default();
        assert_eq!(r.get_by_key(PLATFORM_MAINTENANCE).unwrap().app, APP_ALL);
        assert!(r.get("music", PLATFORM_MAINTENANCE).is_none());
        assert!(r.get(APP_ALL, PLATFORM_MAINTENANCE).is_some());
    }
}
