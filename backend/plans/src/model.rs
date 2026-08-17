//! Pure plan model (design D2): the two plans, the fixed premium unlock set, the
//! entitlement row shape, beta campaigns/memberships and the app-facing snapshot.
//!
//! Nothing here touches I/O; every consumer asks `grants(unlock)` — never a plan
//! name — so the unlock vocabulary is the contract with the rest of the backend.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The two plans. `Premium` is what the store listing sells; there is no ranked
/// "beta" plan — betas are memberships (a separate axis, see [`Membership`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    Free,
    Premium,
}

impl Plan {
    /// Wire/registry name.
    pub fn as_str(self) -> &'static str {
        match self {
            Plan::Free => "free",
            Plan::Premium => "premium",
        }
    }

    /// Whether this plan grants `unlock`. **Premium's set is fixed in code** — it
    /// is what is sold; `free` grants nothing.
    pub fn grants(self, unlock: Unlock) -> bool {
        match self {
            Plan::Free => false,
            Plan::Premium => PREMIUM_UNLOCKS.contains(&unlock),
        }
    }
}

/// Named unlocks a plan can grant. Consumers depend on these keys, not on plans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unlock {
    /// Unlimited catalog player-opens (the daily quota does not apply).
    CatalogUnlimited,
    /// Every accepted catalog SoundFont is downloadable / owned in the shop.
    SoundfontsLibrary,
    /// The larger private `.sf2` import quota.
    SoundfontLibraryExtended,
    /// The larger score upload quota + private score library cap.
    ScoresExtendedQuotas,
    /// Offline caching of **catalog** scores.
    OfflineCache,
}

impl Unlock {
    /// Stable string key (used in the plan snapshot wire format and docs).
    pub fn key(self) -> &'static str {
        match self {
            Unlock::CatalogUnlimited => "catalog.unlimited",
            Unlock::SoundfontsLibrary => "soundfonts.library",
            Unlock::SoundfontLibraryExtended => "soundfont_library.extended",
            Unlock::ScoresExtendedQuotas => "scores.extended_quotas",
            Unlock::OfflineCache => "offline.cache",
        }
    }
}

/// The premium unlock set — **fixed in code** (spec: not editable at runtime).
pub const PREMIUM_UNLOCKS: &[Unlock] = &[
    Unlock::CatalogUnlimited,
    Unlock::SoundfontsLibrary,
    Unlock::SoundfontLibraryExtended,
    Unlock::ScoresExtendedQuotas,
    Unlock::OfflineCache,
];

/// Where an entitlement row came from. Store/web rows are the provider's; `code`
/// and `admin` rows are free (trials, comps) and are the only revocable ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Apple,
    Google,
    Web,
    Code,
    Admin,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Apple => "apple",
            Source::Google => "google",
            Source::Web => "web",
            Source::Code => "code",
            Source::Admin => "admin",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "apple" => Source::Apple,
            "google" => Source::Google,
            "web" => Source::Web,
            "code" => Source::Code,
            "admin" => Source::Admin,
            _ => return None,
        })
    }

    /// Store/web rows end on the provider's side; only free rows are revocable
    /// from the console.
    pub fn is_paid_channel(self) -> bool {
        matches!(self, Source::Apple | Source::Google | Source::Web)
    }
}

/// Provider-reported lifecycle state of a row. `BillingRetry` is the only state
/// that extends activity past `ends_at` (by the grace period).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitlementStatus {
    /// Running (auto-renewing or fixed-term).
    Active,
    /// Cancelled by the user; still active until `ends_at`.
    Cancelled,
    /// Provider is retrying payment; active until `ends_at + grace`.
    BillingRetry,
    /// Ended normally (expired, not renewed).
    Ended,
    /// Refunded / revoked by the provider — inactive immediately.
    Refunded,
    /// Revoked by an admin — inactive immediately.
    Revoked,
}

impl EntitlementStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            EntitlementStatus::Active => "active",
            EntitlementStatus::Cancelled => "cancelled",
            EntitlementStatus::BillingRetry => "billing_retry",
            EntitlementStatus::Ended => "ended",
            EntitlementStatus::Refunded => "refunded",
            EntitlementStatus::Revoked => "revoked",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "active" => EntitlementStatus::Active,
            "cancelled" => EntitlementStatus::Cancelled,
            "billing_retry" => EntitlementStatus::BillingRetry,
            "ended" => EntitlementStatus::Ended,
            "refunded" => EntitlementStatus::Refunded,
            "revoked" => EntitlementStatus::Revoked,
            _ => return None,
        })
    }

    /// States under which the row can never count as active again.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            EntitlementStatus::Refunded | EntitlementStatus::Revoked
        )
    }
}

/// One ledger row: `who`, from which `source`, `until when`, plus the provider's
/// opaque reference. **Identifiers only** — never names, addresses, cards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementRow {
    pub id: Uuid,
    pub user_id: String,
    pub source: Source,
    /// Provider/issuer reference: Apple original transaction id, Google
    /// subscription id, MoR subscription id, code id, admin grant id.
    pub provider_ref: String,
    /// Set for premium-trial rows (the campaign that produced them).
    pub campaign_id: Option<Uuid>,
    pub starts_at: DateTime<Utc>,
    /// `None` only for an open-ended admin grant.
    pub ends_at: Option<DateTime<Utc>>,
    pub status: EntitlementStatus,
    pub revoked_at: Option<DateTime<Utc>>,
    /// Stamped once when the row's lapse has been withdrawn (design D13).
    pub withdrawn_at: Option<DateTime<Utc>>,
}

/// What a campaign does when someone enrols (design D2/D4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum CampaignKind {
    /// "Beta d'usage premium": premium for `duration_days` **from each tester's
    /// own enrolment**; enrolment can be closed without shortening anyone.
    PremiumTrial { duration_days: u32 },
    /// "Beta par fonctionnalité": membership only, no end date; the operator
    /// closes it when the feature is stable, ending early access for everyone.
    Feature,
}

impl CampaignKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CampaignKind::PremiumTrial { .. } => "premium_trial",
            CampaignKind::Feature => "feature",
        }
    }

    /// Default trial length when a campaign is created without one.
    pub const DEFAULT_TRIAL_DAYS: u32 = 90;

    pub fn is_trial(self) -> bool {
        matches!(self, CampaignKind::PremiumTrial { .. })
    }
}

/// A beta campaign. `closed_at` ends every membership at once (feature betas);
/// `enrollment_closes_at` only refuses new enrolments (trials).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Campaign {
    pub id: Uuid,
    /// Stable key used in flag rollouts (`beta:<key>`) and Discord config.
    pub key: String,
    pub name: String,
    pub kind: CampaignKind,
    pub enrollment_closes_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

impl Campaign {
    /// Enrolment is possible: not closed, enrolment window not closed.
    pub fn accepts_enrolment(&self, now: DateTime<Utc>) -> bool {
        self.closed_at.is_none() && self.enrollment_closes_at.is_none_or(|t| now < t)
    }
}

/// How the membership was created (Discord goes through a code).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipSource {
    Code,
    Admin,
}

impl MembershipSource {
    pub fn as_str(self) -> &'static str {
        match self {
            MembershipSource::Code => "code",
            MembershipSource::Admin => "admin",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "code" => Some(MembershipSource::Code),
            "admin" => Some(MembershipSource::Admin),
            _ => None,
        }
    }
}

/// One membership row: the account belongs to a campaign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRow {
    pub campaign_id: Uuid,
    pub user_id: String,
    pub enrolled_at: DateTime<Utc>,
    /// Trials: `enrolled_at + duration`; feature betas: `None` (ends when the
    /// campaign closes).
    pub ends_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub source: MembershipSource,
}

/// A membership joined with its campaign — the unit the core reasons about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    pub row: MembershipRow,
    pub campaign: Campaign,
}

/// One access code (the clear text is never stored — only its hash).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessCode {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub issued_by: String,
    pub issued_to_hint: Option<String>,
    pub max_uses: u32,
    pub uses: u32,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl AccessCode {
    /// Redeemable: not revoked and uses remain.
    pub fn is_redeemable(&self) -> bool {
        self.revoked_at.is_none() && self.uses < self.max_uses
    }
}

/// One redemption of a code by an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redemption {
    pub code_id: Uuid,
    pub user_id: String,
    pub redeemed_at: DateTime<Utc>,
}

/// The active premium trial, as shown to the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrialInfo {
    pub campaign_key: String,
    pub campaign_name: String,
    pub ends_at: DateTime<Utc>,
}

/// One active beta membership, as shown to the app / flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BetaInfo {
    pub campaign_key: String,
    pub campaign_name: String,
    pub kind: CampaignKind,
    pub joined_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
}

/// The one answer every consumer reads: effective plan (+ source, end), the
/// active trial if any, and the active beta memberships (design D10).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanSnapshot {
    pub plan: Plan,
    /// Source of the governing row (latest end) when premium.
    pub source: Option<Source>,
    /// End of the governing row; `None` = open-ended (or free).
    pub ends_at: Option<DateTime<Utc>>,
    /// True when the governing row will end without renewal (trial, cancelled,
    /// comp) — the app shows "rights end on <date>".
    pub ends_without_renewal: bool,
    /// Source of the active **paid-channel** row (`apple` / `google` / `web`) when
    /// there is one, whichever row governs the end date. A trial or admin grant may
    /// outlast a subscription; the paywall must still say "managed on <store>" and
    /// refuse a second purchase / a web checkout — never a double subscription.
    #[serde(default)]
    pub paid_source: Option<Source>,
    pub trial: Option<TrialInfo>,
    pub betas: Vec<BetaInfo>,
}

impl PlanSnapshot {
    /// The free plan with no memberships (also the kill-switch answer).
    pub fn free() -> Self {
        PlanSnapshot {
            plan: Plan::Free,
            source: None,
            ends_at: None,
            ends_without_renewal: false,
            paid_source: None,
            trial: None,
            betas: Vec::new(),
        }
    }

    pub fn grants(&self, unlock: Unlock) -> bool {
        self.plan.grants(unlock)
    }

    /// Campaign keys of the active memberships (flag rollout `beta:<key>`).
    pub fn beta_keys(&self) -> Vec<String> {
        self.betas.iter().map(|b| b.campaign_key.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premium_grants_every_fixed_unlock_and_free_none() {
        for u in PREMIUM_UNLOCKS {
            assert!(Plan::Premium.grants(*u));
            assert!(!Plan::Free.grants(*u));
        }
        assert_eq!(PREMIUM_UNLOCKS.len(), 5);
    }

    #[test]
    fn source_and_status_round_trip() {
        for s in [
            Source::Apple,
            Source::Google,
            Source::Web,
            Source::Code,
            Source::Admin,
        ] {
            assert_eq!(Source::parse(s.as_str()), Some(s));
        }
        assert_eq!(Source::parse("stripe"), None);
        for st in [
            EntitlementStatus::Active,
            EntitlementStatus::Cancelled,
            EntitlementStatus::BillingRetry,
            EntitlementStatus::Ended,
            EntitlementStatus::Refunded,
            EntitlementStatus::Revoked,
        ] {
            assert_eq!(EntitlementStatus::parse(st.as_str()), Some(st));
        }
        assert!(EntitlementStatus::Refunded.is_terminal());
        assert!(!EntitlementStatus::Cancelled.is_terminal());
        assert!(Source::Apple.is_paid_channel());
        assert!(!Source::Code.is_paid_channel());
    }

    #[test]
    fn campaign_enrolment_window() {
        let now = Utc::now();
        let mut c = Campaign {
            id: Uuid::nil(),
            key: "k".into(),
            name: "n".into(),
            kind: CampaignKind::Feature,
            enrollment_closes_at: None,
            closed_at: None,
            created_by: "a".into(),
            created_at: now,
        };
        assert!(c.accepts_enrolment(now));
        c.enrollment_closes_at = Some(now - chrono::Duration::seconds(1));
        assert!(!c.accepts_enrolment(now));
        c.enrollment_closes_at = None;
        c.closed_at = Some(now);
        assert!(!c.accepts_enrolment(now));
        assert_eq!(CampaignKind::Feature.as_str(), "feature");
        assert!(CampaignKind::PremiumTrial { duration_days: 1 }.is_trial());
    }

    #[test]
    fn free_snapshot_grants_nothing() {
        let s = PlanSnapshot::free();
        assert!(!s.grants(Unlock::OfflineCache));
        assert!(s.beta_keys().is_empty());
        assert_eq!(Unlock::OfflineCache.key(), "offline.cache");
        assert_eq!(
            MembershipSource::parse("admin"),
            Some(MembershipSource::Admin)
        );
    }
}
