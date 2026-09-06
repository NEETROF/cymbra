//! `cymbra-plans` — the plans module (change: add-premium-subscription).
//!
//! Two independent axes on an account, both server-side:
//! - the **plan** (`free` / `premium`): a multi-source, expiring entitlement
//!   ledger (`apple` / `google` / `web` / `code` / `admin`), premium while any
//!   row is active, with a **fixed** premium unlock set (design D2);
//! - **beta memberships**: campaigns of kind `premium_trial` (premium N days
//!   from each tester's own enrolment) or `feature` (early access, closed by the
//!   operator), plus single-use access codes (design D4).
//!
//! The crate owns the `plans` schema and stores **identifiers only** — never
//! names, addresses, cards or invoices. Purchase-channel adapters (Apple, Google,
//! web merchant-of-record) turn provider events into ledger writes through
//! [`service::PlanService::apply`].
//!
//! Layering: [`model`] + [`core`] + [`codes`] are pure and host-tested; [`ports`]
//! are the seams (mockable); [`service`] sequences them; [`pg`] and the channel
//! adapters are thin I/O glue (coverage-excluded like the other adapters).
//! `cymbra-music` and the flags service consume the crate through
//! [`ports::PlanSource`] only — `plans` never depends on `music`.

pub mod billing;
pub mod codes;
pub mod core;
pub mod grpc;
pub mod model;
pub mod pg;
pub mod ports;
pub mod service;
pub mod web;

pub use cymbra_platform::AppError;
pub use model::{
    AccessCode, BetaInfo, Campaign, CampaignKind, EntitlementRow, EntitlementStatus, EventProvider,
    Membership, MembershipRow, MembershipSource, PREMIUM_UNLOCKS, Plan, PlanSnapshot, Redemption,
    Source, TrialInfo, Unlock,
};
pub use ports::{
    AccessCodeIssuer, AccessCodeRepo, AuditEntry, AuditRecord, AuditRepo, BillingEventRepo,
    CacheSecretRotator, CampaignRepo, Channel, Clock, Enrolment, EntitlementRepo, EntitlementWrite,
    FixedPaywallConfig, FixedPlanConfig, HandleResolver, MembershipRepo, MintedCode, NewCampaign,
    PaywallConfigSource, PlanConfig, PlanConfigSource, PlanSource, Platform, StoreCustomerEraser,
    StoreCustomerSource, StoreSubscription, SystemClock, WebBillingProvider,
    WebSubscriptionCanceller,
};
pub use service::{AccountPlan, EnrolOutcome, PlanDeps, PlanFilter, PlanService};

/// Generated `cymbra.plans.v1` protobuf types + tonic client/server stubs.
// `tonic::Status` is large by design; newer clippy flags every generated
// client/server signature for it.
#[allow(clippy::result_large_err)]
pub mod proto {
    tonic::include_proto!("cymbra.plans.v1");
}

/// Embedded migrations for the `plans` schema, run by the server at startup on
/// the `plans_svc` connection via `MIGRATOR.run(&pool)`.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
