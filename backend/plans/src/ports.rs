//! Ports (trait seams) of the plans crate. Postgres implementations live in
//! [`crate::pg`]; the server supplies the config/clock/rotation glue. Every
//! trait is `#[automock]`-able under the `mock` feature (rust-testing default).

use crate::model::{
    AccessCode, Campaign, CampaignKind, EntitlementRow, EntitlementStatus, EventProvider,
    Membership, MembershipRow, MembershipSource, PlanSnapshot, Redemption, Source,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cymbra_platform::Result;
#[cfg(any(test, feature = "mock"))]
use mockall::automock;
use std::collections::HashMap;
use uuid::Uuid;

/// One write to the ledger, upserted by `(source, provider_ref)` (design D3):
/// `ends_at` only moves forward unless `status` is terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementWrite {
    pub user_id: String,
    pub source: Source,
    pub provider_ref: String,
    pub campaign_id: Option<Uuid>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub status: EntitlementStatus,
}

/// Runtime knobs read per call (flags): the kill-switch and the grace period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanConfig {
    pub enabled: bool,
    pub grace_days: u32,
}

impl Default for PlanConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            grace_days: 3,
        }
    }
}

/// Where the plan service reads its runtime configuration.
#[cfg_attr(any(test, feature = "mock"), automock)]
pub trait PlanConfigSource: Send + Sync {
    fn plan_config(&self) -> PlanConfig;
}

/// A fixed configuration (tests, or a deployment with no flag store).
pub struct FixedPlanConfig(pub PlanConfig);

impl PlanConfigSource for FixedPlanConfig {
    fn plan_config(&self) -> PlanConfig {
        self.0
    }
}

/// Injectable clock so the invariants are testable at fixed instants.
#[cfg_attr(any(test, feature = "mock"), automock)]
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// The wall clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// The ledger.
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait EntitlementRepo: Send + Sync {
    async fn list_for_user(&self, user_id: &str) -> Result<Vec<EntitlementRow>>;
    async fn get(&self, id: Uuid) -> Result<Option<EntitlementRow>>;
    /// The row for `(source, provider_ref)` — how a provider notification that
    /// carries no account token finds its user.
    async fn find_by_provider_ref(
        &self,
        source: Source,
        provider_ref: &str,
    ) -> Result<Option<EntitlementRow>>;
    /// Upsert by `(source, provider_ref)`; forward-only `ends_at` unless terminal.
    async fn upsert(&self, write: EntitlementWrite) -> Result<EntitlementRow>;
    async fn revoke(&self, id: Uuid, at: DateTime<Utc>) -> Result<()>;
    /// Stamp `withdrawn_at` on rows that are not yet stamped; returns how many
    /// rows this call stamped (0 ⇒ someone else already did).
    async fn mark_withdrawn(&self, ids: &[Uuid], at: DateTime<Utc>) -> Result<u64>;
    /// Users owning at least one un-withdrawn row whose end is at or before
    /// `before` — the daily sweep's candidates (re-checked in the core).
    async fn users_with_unwithdrawn_ended_rows(&self, before: DateTime<Utc>)
    -> Result<Vec<String>>;
    /// Rows from `sources` ending in `[from, to)` — the reconciliation window.
    async fn list_ending_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        sources: &[Source],
    ) -> Result<Vec<EntitlementRow>>;
    /// User ids with a row active at `now` (grace applied), optionally only
    /// rows produced by a trial campaign — the directory filter.
    async fn active_user_ids(
        &self,
        now: DateTime<Utc>,
        grace_days: u32,
        trial_only: bool,
    ) -> Result<Vec<String>>;
    async fn purge_user(&self, user_id: &str) -> Result<()>;
}

/// New campaign input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCampaign {
    pub key: String,
    pub name: String,
    pub kind: CampaignKind,
    pub created_by: String,
}

#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait CampaignRepo: Send + Sync {
    async fn create(&self, new: NewCampaign) -> Result<Campaign>;
    async fn get(&self, id: Uuid) -> Result<Option<Campaign>>;
    async fn get_by_key(&self, key: &str) -> Result<Option<Campaign>>;
    async fn list(&self, include_closed: bool) -> Result<Vec<Campaign>>;
    async fn close_enrollment(&self, id: Uuid, at: DateTime<Utc>) -> Result<()>;
    async fn close(&self, id: Uuid, at: DateTime<Utc>) -> Result<()>;

    /// Clear the campaign's closed state. Memberships are untouched by closing,
    /// so this alone brings every unrevoked member back (change:
    /// reopen-beta-campaign).
    async fn reopen(&self, id: Uuid) -> Result<()>;

    /// Clear the enrolment deadline. Separate from [`Self::reopen`] because
    /// closing a campaign closes its enrolment as a side effect: without this,
    /// a reopened beta could be restored but never grown.
    async fn reopen_enrollment(&self, id: Uuid) -> Result<()>;
}

/// One enrolment, applied in a single transaction by the repo: membership
/// insert, code consumption (if any), and the trial row (if any).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enrolment {
    pub campaign_id: Uuid,
    pub user_id: String,
    pub enrolled_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub source: MembershipSource,
    pub code_id: Option<Uuid>,
    pub trial_row: Option<EntitlementWrite>,
}

#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait MembershipRepo: Send + Sync {
    async fn list_for_user(&self, user_id: &str) -> Result<Vec<Membership>>;
    async fn list_members(&self, campaign_id: Uuid) -> Result<Vec<MembershipRow>>;
    /// Member ids of `campaign_id` active at `now` (not revoked, not ended).
    async fn active_member_ids(&self, campaign_id: Uuid, now: DateTime<Utc>)
    -> Result<Vec<String>>;
    /// Transactional enrolment (see [`Enrolment`]). `AlreadyExists` when the
    /// membership exists; `FailedPrecondition` when the code is spent.
    async fn enrol(&self, enrolment: Enrolment) -> Result<()>;
    async fn revoke(&self, campaign_id: Uuid, user_id: &str, at: DateTime<Utc>) -> Result<()>;
    async fn purge_user(&self, user_id: &str) -> Result<()>;
}

#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait AccessCodeRepo: Send + Sync {
    async fn insert(
        &self,
        campaign_id: Uuid,
        code_hash: &str,
        issued_by: &str,
        issued_to_hint: Option<String>,
        max_uses: u32,
    ) -> Result<AccessCode>;
    async fn find_by_hash(&self, code_hash: &str) -> Result<Option<AccessCode>>;
    async fn revoke(&self, id: Uuid, at: DateTime<Utc>) -> Result<()>;
    /// Revoke every still-redeemable code of a campaign (spent codes are already
    /// inert and are left as redemption records); returns how many were revoked.
    async fn revoke_campaign(&self, campaign_id: Uuid, at: DateTime<Utc>) -> Result<u64>;
    async fn redemptions(&self, campaign_id: Uuid) -> Result<Vec<Redemption>>;
    async fn purge_user(&self, user_id: &str) -> Result<()>;
}

/// Idempotency ledger for provider notifications.
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait BillingEventRepo: Send + Sync {
    /// Record `(provider, event_id)`; `true` when it was new (apply it),
    /// `false` when already seen (no-op).
    async fn record_if_new(
        &self,
        provider: EventProvider,
        event_id: &str,
        user_id: Option<String>,
        payload_ref: &str,
    ) -> Result<bool>;
    async fn mark_applied(&self, provider: EventProvider, event_id: &str) -> Result<()>;
    async fn purge_user(&self, user_id: &str) -> Result<()>;
}

/// One audited admin mutation (mirrors `role_grants`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    pub actor: String,
    pub action: String,
    pub target_user: Option<String>,
    pub target_ref: Option<String>,
    pub reason: String,
}

/// One audited mutation as it is READ BACK: the write model plus when it happened.
/// The console asks for a reason on every destructive plan action, so it must be able
/// to show those reasons — an audit nobody can consult is a field the operator fills in
/// for nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditRecord {
    pub actor: String,
    pub action: String,
    pub target_ref: Option<String>,
    pub reason: String,
    pub at: DateTime<Utc>,
}

#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait AuditRepo: Send + Sync {
    async fn record(&self, entry: AuditEntry) -> Result<()>;
    /// One account's audited plan changes, most recent first.
    async fn list_for_user(&self, user_id: &str, limit: u32) -> Result<Vec<AuditRecord>>;
}

/// The seam through which a lapse rotates the user's offline cache secret
/// (design D13). Implemented by the server over the music offline-secret repo;
/// `plans` never depends on `music`.
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait CacheSecretRotator: Send + Sync {
    async fn rotate(&self, user_id: &str) -> Result<()>;
}

/// Cancels a web-MoR subscription on the provider (account erasure: a deleted
/// account must not be billed). Implemented by the web billing adapter.
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait WebSubscriptionCanceller: Send + Sync {
    async fn cancel(&self, subscription_ref: &str) -> Result<()>;
}

/// The one read every consumer uses (music gates, flags, the app RPC).
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait PlanSource: Send + Sync {
    async fn snapshot(&self, user_id: &str) -> Result<PlanSnapshot>;
}

/// A minted code: the clear text is returned **once**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintedCode {
    pub code: String,
    pub campaign_key: String,
    pub campaign_name: String,
    pub kind: CampaignKind,
}

/// The issuing port every issuer uses (back office, Discord `/beta`, …).
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait AccessCodeIssuer: Send + Sync {
    /// Mint one single-use code for `campaign_key`; refused when the campaign
    /// does not accept enrolment.
    async fn mint(
        &self,
        campaign_key: &str,
        issued_by: &str,
        issued_to_hint: Option<String>,
    ) -> Result<MintedCode>;
}

/// Purchase channels (mirrors the proto enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Channel {
    Apple,
    Google,
    Web,
}

impl Channel {
    pub fn source(self) -> Source {
        match self {
            Channel::Apple => Source::Apple,
            Channel::Google => Source::Google,
            Channel::Web => Source::Web,
        }
    }

    pub fn from_source(s: Source) -> Option<Channel> {
        match s {
            Source::Apple => Some(Channel::Apple),
            Source::Google => Some(Channel::Google),
            Source::Web => Some(Channel::Web),
            Source::Code | Source::Admin => None,
        }
    }
}

/// The client's declared platform (mirrors the proto enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Ios,
    Macos,
    Android,
    Linux,
    Windows,
    Web,
}

impl Platform {
    /// The one channel a platform may buy through: store builds their store,
    /// desktop / web the merchant of record.
    pub fn channel(self) -> Channel {
        match self {
            Platform::Ios | Platform::Macos => Channel::Apple,
            Platform::Android => Channel::Google,
            Platform::Linux | Platform::Windows | Platform::Web => Channel::Web,
        }
    }
}

/// Paywall knobs read per call: which channels are open and which products to
/// offer.
#[cfg_attr(any(test, feature = "mock"), automock)]
pub trait PaywallConfigSource: Send + Sync {
    fn channel_enabled(&self, channel: Channel) -> bool;
    fn products(&self) -> Vec<String>;
}

/// Fixed paywall config (tests).
#[derive(Debug, Clone)]
pub struct FixedPaywallConfig {
    pub apple: bool,
    pub google: bool,
    pub web: bool,
    pub products: Vec<String>,
}

impl PaywallConfigSource for FixedPaywallConfig {
    fn channel_enabled(&self, channel: Channel) -> bool {
        match channel {
            Channel::Apple => self.apple,
            Channel::Google => self.google,
            Channel::Web => self.web,
        }
    }
    fn products(&self) -> Vec<String> {
        self.products.clone()
    }
}

/// Resolves a handle to an account id (the console addresses accounts by handle;
/// implemented by the server over the identity port).
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait HandleResolver: Send + Sync {
    async fn user_id_for_handle(&self, handle: &str) -> Result<Option<String>>;
    /// Handles for the given account ids (ids with no handle are simply absent), so a
    /// listing can name the admin who acted instead of showing a raw uuid. `plans` owns
    /// no account table and cannot join one — the composition root wires this.
    async fn handles_for_ids(&self, ids: &[String]) -> Result<HashMap<String, String>>;
}

/// One store subscription as the aggregator reports it for a customer (change:
/// swap-store-billing-to-revenuecat, D3) — provider-neutral shape the pure
/// mapper consumes. Dates are the store's; `provider_ref` is the store's original
/// transaction id as the aggregator reports it (the ledger key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreSubscription {
    /// Aggregator store name (`APP_STORE`, `MAC_APP_STORE`, `PLAY_STORE`, `PADDLE`, …).
    pub store: String,
    pub product_id: String,
    pub provider_ref: String,
    pub purchase_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub unsubscribe_detected_at: Option<DateTime<Utc>>,
    pub billing_issues_detected_at: Option<DateTime<Utc>>,
    pub grace_period_expires_at: Option<DateTime<Utc>>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub is_sandbox: bool,
}

/// Reads one account's subscriptions from the store aggregator (the pull side of
/// D3: plan sync after purchase/restore, and reconciliation). Reads only the
/// given account — never another one's.
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait StoreCustomerSource: Send + Sync {
    async fn subscriptions(&self, user_id: &str) -> Result<Vec<StoreSubscription>>;
}

/// Deletes an account's aggregator customer (account erasure, D6).
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait StoreCustomerEraser: Send + Sync {
    async fn delete_customer(&self, user_id: &str) -> Result<()>;
}

/// The web merchant-of-record: hosted checkout + portal + cancellation.
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait WebBillingProvider: Send + Sync {
    /// A hosted checkout URL for `product_id` carrying `user_id` as custom data.
    async fn create_checkout(&self, user_id: &str, product_id: &str) -> Result<String>;
    /// The hosted management portal URL for a subscription.
    async fn portal_url(&self, subscription_ref: &str) -> Result<String>;
    /// Cancel a subscription (account erasure).
    async fn cancel(&self, subscription_ref: &str) -> Result<()>;
}
