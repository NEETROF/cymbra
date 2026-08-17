//! Transport-agnostic **user-facing plan operations** shared by the gRPC surface
//! (`grpc.rs`, app + console) and the browser JSON routes of the public site
//! (`cymbra-server::web_plans`, change: add-site-account-pages).
//!
//! One implementation of the rules — the redeem audience check and throttle, the
//! checkout gates, the portal lookup, the paywall view — so the two transports
//! cannot drift. Everything here returns [`AppError`]; each transport maps it to
//! its own status.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use cymbra_platform::cache::Cache;
use cymbra_platform::{AppError, Result, ratelimit};
use serde::Serialize;

use crate::model::{PREMIUM_UNLOCKS, PlanSnapshot, Source};
use crate::ports::{Channel, PaywallConfigSource, Platform, WebBillingProvider};
use crate::service::PlanService;

/// Redemption throttle: attempts per account and per address per window.
pub const REDEEM_MAX_PER_USER: u32 = 10;
pub const REDEEM_MAX_PER_ADDR: u32 = 30;
pub const REDEEM_WINDOW: Duration = Duration::from_secs(600);

/// The store-build audience: never offered the web channel or the redeem field
/// (Apple 3.1.1). Every other audience (`web`, `back-office`, `live`) may redeem.
pub const APP_AUDIENCE: &str = "music";

/// One beta membership as shown to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BetaView {
    pub campaign_key: String,
    pub campaign_name: String,
    pub kind: String,
    pub joined_at: String,
    pub ends_at: Option<String>,
}

/// The plan answer for one caller on one platform — the same fields (and names)
/// as `GetMyPlanResponse`; the browser routes serialize it as-is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanView {
    pub plan: String,
    pub source: Option<String>,
    pub ends_at: Option<String>,
    pub ends_without_renewal: bool,
    pub trial_campaign_key: Option<String>,
    pub trial_campaign_name: Option<String>,
    pub trial_ends_at: Option<String>,
    pub betas: Vec<BetaView>,
    /// Channel the governing paid row is managed on (`apple` / `google` / `web`),
    /// `None` when there is none.
    pub managed_on: Option<String>,
    /// Whether the caller's platform may start a purchase right now.
    pub can_purchase_here: bool,
    /// The purchase channel of the caller's platform when `can_purchase_here`.
    pub purchase_channel: Option<String>,
    /// Product ids to offer when `can_purchase_here`, empty otherwise.
    pub products: Vec<String>,
    /// Premium unlock keys the snapshot grants.
    pub unlocks: Vec<String>,
}

/// Outcome of a redemption as shown to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RedeemView {
    pub campaign_key: String,
    pub campaign_name: String,
    pub kind: String,
    pub ends_at: Option<String>,
}

pub fn rfc3339(t: DateTime<Utc>) -> String {
    t.to_rfc3339()
}

/// Wire name of a channel (also the `Source` name of its paid rows).
pub fn channel_key(c: Channel) -> &'static str {
    match c {
        Channel::Apple => "apple",
        Channel::Google => "google",
        Channel::Web => "web",
    }
}

/// The paywall part of the answer: managed-on + can-purchase-here + channel.
fn purchase_view(
    svc: &PlanService,
    paywall: &dyn PaywallConfigSource,
    snapshot: &PlanSnapshot,
    platform: Option<Platform>,
) -> (Option<Channel>, bool, Option<Channel>) {
    // The paid row, not the governing one: a trial may outlast a subscription.
    let managed_on = snapshot.paid_source.and_then(Channel::from_source);
    let channel = platform.map(Platform::channel);
    let can_purchase = match (managed_on, channel) {
        (Some(_), _) => false,
        (None, Some(c)) => paywall.channel_enabled(c) && svc.enabled(),
        (None, None) => false,
    };
    (managed_on, can_purchase, channel.filter(|_| can_purchase))
}

/// Build the user-facing plan answer from a snapshot for `platform`.
pub fn plan_view(
    svc: &PlanService,
    paywall: &dyn PaywallConfigSource,
    snapshot: &PlanSnapshot,
    platform: Option<Platform>,
) -> PlanView {
    let (managed_on, can_purchase_here, purchase_channel) =
        purchase_view(svc, paywall, snapshot, platform);
    PlanView {
        plan: snapshot.plan.as_str().to_string(),
        source: snapshot.source.map(|s| s.as_str().to_string()),
        ends_at: snapshot.ends_at.map(rfc3339),
        ends_without_renewal: snapshot.ends_without_renewal,
        trial_campaign_key: snapshot.trial.as_ref().map(|t| t.campaign_key.clone()),
        trial_campaign_name: snapshot.trial.as_ref().map(|t| t.campaign_name.clone()),
        trial_ends_at: snapshot.trial.as_ref().map(|t| rfc3339(t.ends_at)),
        betas: snapshot
            .betas
            .iter()
            .map(|b| BetaView {
                campaign_key: b.campaign_key.clone(),
                campaign_name: b.campaign_name.clone(),
                kind: b.kind.as_str().to_string(),
                joined_at: rfc3339(b.joined_at),
                ends_at: b.ends_at.map(rfc3339),
            })
            .collect(),
        managed_on: managed_on.map(|c| channel_key(c).to_string()),
        can_purchase_here,
        purchase_channel: purchase_channel.map(|c| channel_key(c).to_string()),
        products: if can_purchase_here {
            paywall.products()
        } else {
            Vec::new()
        },
        unlocks: PREMIUM_UNLOCKS
            .iter()
            .filter(|u| snapshot.grants(**u))
            .map(|u| u.key().to_string())
            .collect(),
    }
}

/// `GetMyPlan` for `user_id` on `platform`.
pub async fn my_plan(
    svc: &PlanService,
    paywall: &dyn PaywallConfigSource,
    user_id: &str,
    platform: Option<Platform>,
) -> Result<PlanView> {
    let snapshot = svc.snapshot(user_id).await?;
    Ok(plan_view(svc, paywall, &snapshot, platform))
}

/// Redeem an access code for `user_id`: refused for the store-build audience,
/// throttled per account and per address **before any lookup**, then delegated to
/// the service (neutral refusals, one per campaign, one active trial).
pub async fn redeem(
    svc: &PlanService,
    cache: &dyn Cache,
    user_id: &str,
    audience: &str,
    addr: &str,
    code: &str,
) -> Result<RedeemView> {
    if audience == APP_AUDIENCE {
        return Err(AppError::FailedPrecondition(
            "codes are redeemed on the web".into(),
        ));
    }
    ratelimit::check(
        cache,
        "redeem_code_user",
        user_id,
        REDEEM_MAX_PER_USER,
        REDEEM_WINDOW,
    )
    .await?;
    ratelimit::check(
        cache,
        "redeem_code_addr",
        addr,
        REDEEM_MAX_PER_ADDR,
        REDEEM_WINDOW,
    )
    .await?;
    let out = svc.redeem(user_id, code).await?;
    Ok(RedeemView {
        campaign_key: out.campaign_key,
        campaign_name: out.campaign_name,
        kind: out.kind.as_str().to_string(),
        ends_at: out.ends_at.map(rfc3339),
    })
}

/// A hosted web checkout URL for `product_id`, bound to `user_id` — refused when
/// plans or the web channel are off, when the product is not offered, or when the
/// account is already subscribed on any paid channel.
pub async fn create_checkout(
    svc: &PlanService,
    paywall: &dyn PaywallConfigSource,
    web: Option<&Arc<dyn WebBillingProvider>>,
    user_id: &str,
    product_id: &str,
) -> Result<String> {
    if !svc.enabled() || !paywall.channel_enabled(Channel::Web) {
        return Err(AppError::FailedPrecondition("web channel disabled".into()));
    }
    let web = web.ok_or_else(|| AppError::Config("web billing not configured".into()))?;
    let snapshot = svc.snapshot(user_id).await?;
    if snapshot.paid_source.is_some() {
        return Err(AppError::FailedPrecondition(
            "already subscribed on another channel".into(),
        ));
    }
    if !paywall.products().iter().any(|p| p == product_id) {
        return Err(AppError::InvalidArgument("unknown product".into()));
    }
    web.create_checkout(user_id, product_id).await
}

/// The provider-hosted portal URL for the caller's **active web** subscription,
/// fetched at request time; refused when the plan is not governed by a web row
/// (store rows are managed on the store).
pub async fn portal_url(
    svc: &PlanService,
    web: Option<&Arc<dyn WebBillingProvider>>,
    user_id: &str,
) -> Result<String> {
    let web = web.ok_or_else(|| AppError::Config("web billing not configured".into()))?;
    let ap = svc.account_plan(user_id).await?;
    if ap.snapshot.paid_source != Some(Source::Web) {
        return Err(AppError::FailedPrecondition(
            "subscription is not managed on the web".into(),
        ));
    }
    let row = crate::core::active_paid_row(&ap.rows, svc.now(), svc.grace())
        .map(|g| g.row)
        .filter(|r| r.source == Source::Web)
        .ok_or_else(|| AppError::FailedPrecondition("no active web subscription".into()))?;
    web.portal_url(&row.provider_ref).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Campaign, CampaignKind, EntitlementRow, EntitlementStatus, Plan};
    use crate::ports::{
        FixedPaywallConfig, MockAccessCodeRepo, MockAuditRepo, MockBillingEventRepo,
        MockCampaignRepo, MockClock, MockEntitlementRepo, MockMembershipRepo, MockPlanConfigSource,
        MockWebBillingProvider, PlanConfig,
    };
    use crate::service::PlanDeps;
    use chrono::TimeZone;
    use cymbra_platform::cache::FakeCache;
    use uuid::Uuid;

    fn t(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, day, 12, 0, 0).unwrap()
    }

    fn row(source: Source, ends: Option<DateTime<Utc>>) -> EntitlementRow {
        EntitlementRow {
            id: Uuid::new_v4(),
            user_id: "u1".into(),
            source,
            provider_ref: format!("{}-ref", source.as_str()),
            campaign_id: None,
            starts_at: t(1),
            ends_at: ends,
            status: EntitlementStatus::Active,
            revoked_at: None,
            withdrawn_at: None,
        }
    }

    /// A service over mocks: `rows` for `u1`, no memberships, no codes.
    fn service(enabled: bool, rows: Vec<EntitlementRow>) -> PlanService {
        let mut config = MockPlanConfigSource::new();
        config.expect_plan_config().returning(move || PlanConfig {
            enabled,
            grace_days: 3,
        });
        let mut clock = MockClock::new();
        clock.expect_now().returning(|| t(5));
        let mut audit = MockAuditRepo::new();
        audit.expect_record().returning(|_| Ok(()));
        let mut entitlements = MockEntitlementRepo::new();
        entitlements
            .expect_list_for_user()
            .returning(move |_| Ok(rows.clone()));
        let mut memberships = MockMembershipRepo::new();
        memberships
            .expect_list_for_user()
            .returning(|_| Ok(Vec::new()));
        let mut codes = MockAccessCodeRepo::new();
        codes.expect_find_by_hash().returning(|_| Ok(None));
        let mut campaigns = MockCampaignRepo::new();
        campaigns.expect_get().returning(|_| Ok(None));
        PlanService::new(PlanDeps {
            entitlements: Arc::new(entitlements),
            campaigns: Arc::new(campaigns),
            memberships: Arc::new(memberships),
            codes: Arc::new(codes),
            billing_events: Arc::new(MockBillingEventRepo::new()),
            audit: Arc::new(audit),
            config: Arc::new(config),
            clock: Arc::new(clock),
            rotator: None,
        })
    }

    fn paywall(web: bool) -> FixedPaywallConfig {
        FixedPaywallConfig {
            apple: true,
            google: true,
            web,
            products: vec!["premium_monthly".into(), "premium_yearly".into()],
        }
    }

    #[test]
    fn plan_view_free_on_web_offers_the_web_channel() {
        let svc = service(true, vec![]);
        let v = plan_view(
            &svc,
            &paywall(true),
            &PlanSnapshot::free(),
            Some(Platform::Web),
        );
        assert_eq!(v.plan, "free");
        assert!(v.can_purchase_here);
        assert_eq!(v.purchase_channel.as_deref(), Some("web"));
        assert_eq!(v.products.len(), 2);
        assert!(v.unlocks.is_empty());
        assert_eq!(v.managed_on, None);
        // JSON field names are the proto names.
        let json = serde_json::to_value(&v).unwrap();
        assert!(json.get("can_purchase_here").is_some());
        assert!(json.get("ends_without_renewal").is_some());
    }

    #[test]
    fn plan_view_managed_elsewhere_never_offers_a_purchase() {
        let svc = service(true, vec![]);
        let snapshot = PlanSnapshot {
            plan: Plan::Premium,
            source: Some(Source::Apple),
            paid_source: Some(Source::Apple),
            ends_at: Some(t(20)),
            ..PlanSnapshot::free()
        };
        let v = plan_view(&svc, &paywall(true), &snapshot, Some(Platform::Web));
        assert_eq!(v.managed_on.as_deref(), Some("apple"));
        assert!(!v.can_purchase_here);
        assert!(v.products.is_empty());
        assert!(v.unlocks.contains(&"offline.cache".to_string()));
        // No platform → nothing to purchase on either.
        let none = plan_view(&svc, &paywall(true), &PlanSnapshot::free(), None);
        assert!(!none.can_purchase_here);
    }

    #[tokio::test]
    async fn my_plan_reads_the_snapshot() {
        let svc = service(true, vec![row(Source::Web, Some(t(30)))]);
        let v = my_plan(&svc, &paywall(true), "u1", Some(Platform::Web))
            .await
            .unwrap();
        assert_eq!(v.plan, "premium");
        assert_eq!(v.managed_on.as_deref(), Some("web"));
    }

    #[tokio::test]
    async fn redeem_refuses_the_store_audience_before_any_work() {
        let svc = service(true, vec![]);
        let cache = FakeCache::default();
        let err = redeem(&svc, &cache, "u1", APP_AUDIENCE, "1.2.3.4", "ABCD")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::FailedPrecondition(_)));
    }

    #[tokio::test]
    async fn redeem_unknown_code_is_a_neutral_refusal_and_throttles_past_the_limit() {
        let svc = service(true, vec![]);
        let cache = FakeCache::default();
        let err = redeem(&svc, &cache, "u1", "web", "1.2.3.4", "ABCD")
            .await
            .unwrap_err();
        assert!(!matches!(err, AppError::ResourceExhausted(_)));
        for _ in 1..REDEEM_MAX_PER_USER {
            let _ = redeem(&svc, &cache, "u1", "web", "1.2.3.4", "ABCD").await;
        }
        let err = redeem(&svc, &cache, "u1", "web", "1.2.3.4", "ABCD")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::ResourceExhausted(_)));
    }

    #[tokio::test]
    async fn checkout_gates_then_delegates() {
        let svc = service(true, vec![]);
        let mut web = MockWebBillingProvider::new();
        web.expect_create_checkout()
            .withf(|u, p| u == "u1" && p == "premium_monthly")
            .returning(|_, _| Ok("https://cymbra.app/checkout?_ptxn=txn_1".into()));
        let web: Arc<dyn WebBillingProvider> = Arc::new(web);

        // Channel off.
        let err = create_checkout(&svc, &paywall(false), Some(&web), "u1", "premium_monthly")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::FailedPrecondition(_)));
        // Not configured.
        let err = create_checkout(&svc, &paywall(true), None, "u1", "premium_monthly")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Config(_)));
        // Unknown product.
        let err = create_checkout(&svc, &paywall(true), Some(&web), "u1", "gold")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidArgument(_)));
        // Happy path.
        let url = create_checkout(&svc, &paywall(true), Some(&web), "u1", "premium_monthly")
            .await
            .unwrap();
        assert!(url.contains("_ptxn=txn_1"));
    }

    #[tokio::test]
    async fn a_trial_outlasting_the_subscription_still_says_managed_on_the_store() {
        // Task 11.9: trial (code) row ends after the Apple row → the trial governs
        // the end date, but the paywall says "managed on Apple", offers no purchase
        // and the web checkout refuses.
        let svc = service(
            true,
            vec![
                row(Source::Apple, Some(t(20))),
                row(Source::Code, Some(t(28))),
            ],
        );
        let v = my_plan(&svc, &paywall(true), "u1", Some(Platform::Web))
            .await
            .unwrap();
        assert_eq!(v.plan, "premium");
        assert_eq!(v.source.as_deref(), Some("code"));
        assert_eq!(v.managed_on.as_deref(), Some("apple"));
        assert!(!v.can_purchase_here);
        assert!(v.products.is_empty());
        let mut web = MockWebBillingProvider::new();
        web.expect_create_checkout().never();
        let web: Arc<dyn WebBillingProvider> = Arc::new(web);
        let err = create_checkout(&svc, &paywall(true), Some(&web), "u1", "premium_monthly")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::FailedPrecondition(_)));
    }

    #[tokio::test]
    async fn portal_works_when_a_trial_outlasts_the_web_subscription() {
        let mut web = MockWebBillingProvider::new();
        web.expect_portal_url()
            .withf(|r| r == "web-ref")
            .returning(|_| Ok("https://portal.example/session".into()));
        let web: Arc<dyn WebBillingProvider> = Arc::new(web);
        let svc = service(
            true,
            vec![
                row(Source::Web, Some(t(20))),
                row(Source::Code, Some(t(28))),
            ],
        );
        assert!(portal_url(&svc, Some(&web), "u1").await.is_ok());
    }

    #[tokio::test]
    async fn checkout_refused_when_subscribed_on_another_channel() {
        let svc = service(true, vec![row(Source::Apple, Some(t(30)))]);
        let mut web = MockWebBillingProvider::new();
        web.expect_create_checkout().never();
        let web: Arc<dyn WebBillingProvider> = Arc::new(web);
        let err = create_checkout(&svc, &paywall(true), Some(&web), "u1", "premium_monthly")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::FailedPrecondition(_)));
    }

    #[tokio::test]
    async fn portal_only_for_the_active_web_row() {
        let mut web = MockWebBillingProvider::new();
        web.expect_portal_url()
            .withf(|r| r == "web-ref")
            .returning(|_| Ok("https://portal.example/session".into()));
        let web: Arc<dyn WebBillingProvider> = Arc::new(web);

        let svc = service(true, vec![row(Source::Web, Some(t(30)))]);
        let url = portal_url(&svc, Some(&web), "u1").await.unwrap();
        assert!(url.starts_with("https://portal.example"));

        // Store row → refused, provider not called.
        let svc = service(true, vec![row(Source::Apple, Some(t(30)))]);
        let err = portal_url(&svc, Some(&web), "u1").await.unwrap_err();
        assert!(matches!(err, AppError::FailedPrecondition(_)));

        // Free → refused.
        let svc = service(true, vec![]);
        let err = portal_url(&svc, Some(&web), "u1").await.unwrap_err();
        assert!(matches!(err, AppError::FailedPrecondition(_)));

        // Not configured.
        let svc = service(true, vec![row(Source::Web, Some(t(30)))]);
        let err = portal_url(&svc, None, "u1").await.unwrap_err();
        assert!(matches!(err, AppError::Config(_)));
    }

    #[test]
    fn channel_keys_match_sources() {
        assert_eq!(channel_key(Channel::Apple), Source::Apple.as_str());
        assert_eq!(channel_key(Channel::Google), Source::Google.as_str());
        assert_eq!(channel_key(Channel::Web), Source::Web.as_str());
        let _ = Campaign {
            id: Uuid::nil(),
            key: "k".into(),
            name: "n".into(),
            kind: CampaignKind::Feature,
            enrollment_closes_at: None,
            closed_at: None,
            created_by: "a".into(),
            created_at: t(1),
        };
    }
}
