//! `cymbra-user-port` — the user module's **contract** crate.
//!
//! Carries the [`UserPort`] trait + DTOs, the generated protobuf types, and the
//! gRPC **client** adapter. The `cymbra-auth` module reaches accounts through this
//! crate — never `cymbra-user` directly (design D0).

use async_trait::async_trait;
use chrono::NaiveDate;
use cymbra_platform::{AppError, Result};
use std::collections::BTreeMap;

/// Generated protobuf messages + tonic client/server stubs for `cymbra.user.v1`.
pub mod proto {
    tonic::include_proto!("cymbra.user.v1");
}

/// How visible a profile is to OTHER players (change: add-play-activity-profile).
/// **Private by default** (opt-in sharing): the public-profile read exposes a
/// profile to arbitrary viewers only when it is `Public` (fail-closed). A
/// followers-only tier can be added later with its own semantics + migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

impl Visibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Visibility::Private => "private",
            Visibility::Public => "public",
        }
    }

    /// Parse the wire/stored form; unknown values are rejected (fail-closed).
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "private" => Ok(Visibility::Private),
            "public" => Ok(Visibility::Public),
            other => Err(AppError::InvalidArgument(format!(
                "unknown visibility {other:?}"
            ))),
        }
    }
}

/// A player's profile as exposed to another player (change: add-play-activity-
/// profile): an explicit allow-list of public fields. It NEVER carries email,
/// curator alignment/reliability, or moderation state. The play heatmap +
/// songs-played totals are read separately (music `PlayService`) and composed
/// client-side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerProfile {
    pub user_id: String,
    pub handle: Option<String>,
    pub display_name: Option<String>,
    pub visibility: Visibility,
}

/// Account aggregate (domain DTO, independent of protobuf/SQL).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub user_id: String,
    pub display_name: Option<String>,
    /// Preferences as a JSON object string.
    pub preferences: String,
    pub version: i64,
    pub updated_at: i64,
    /// Unique display handle; `None` until the user completes onboarding.
    pub handle: Option<String>,
    /// Preferred language tag (change: sync-account-language-preference); `None`
    /// until the identity system records one.
    pub locale: Option<String>,
}

/// A provider identity linked to an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub provider: String,
    pub subject: String,
    pub linked_at: i64,
}

/// One append-only role-grant audit entry (change: add-moderation-back-office):
/// who was granted/revoked which role in which scope, by which admin, and when.
/// The current authorization state lives in the roles store; this is history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleGrant {
    pub target_user_id: String,
    pub scope: String,
    pub role: String,
    /// `grant` or `revoke`.
    pub action: String,
    pub acting_admin: String,
    pub at: i64, // unix seconds
    /// Resolved handle of the acting admin (`None` if it has none / was deleted).
    pub acting_admin_handle: Option<String>,
}

/// The roles an account holds in one scope (change: scope-aware-role-admin).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeRoles {
    pub scope: String,
    pub roles: Vec<String>,
}

/// One row of the admin account directory (change: add-admin-account-directory):
/// an account with the roles it holds, **grouped by scope** and restricted to the
/// scopes the calling admin may administer (change: scope-aware-role-admin).
/// Handle/display_name are optional (handle-less accounts are onboarding-incomplete).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountSummary {
    pub user_id: String,
    pub handle: Option<String>,
    pub display_name: Option<String>,
    /// Roles grouped by scope (only the caller's authorized scopes), ordered by the
    /// scope list the caller was authorized for.
    pub roles_by_scope: Vec<ScopeRoles>,
}

/// A page of the admin account directory plus the total matching count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountPage {
    pub entries: Vec<AccountSummary>,
    pub total: i64,
}

/// The user module's port: the contract `cymbra-auth` and the server adapter call.
///
/// Implemented in-process by the direct adapter (`cymbra-user`) and — for the
/// public account-management subset — over the wire by [`GrpcUserClient`].
// `#[automock]` sits ABOVE `#[async_trait]` (gated on the `mock` feature) so the
// music PlayService can double the play-activity visibility gate in unit tests.
#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait UserPort: Send + Sync {
    /// Resolve the account for `(provider, subject)`, provisioning it (with the
    /// default `(global, user)` role) on first sight. Returns the `user_id`.
    async fn resolve_or_provision(&self, provider: &str, subject: &str) -> Result<String>;

    /// Attach a new identity to `user_id`; rejects an identity bound elsewhere.
    async fn link_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()>;

    /// Remove an identity from `user_id`; rejects removing the last one.
    async fn unlink_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()>;

    /// List the identities linked to `user_id`.
    async fn list_identities(&self, user_id: &str) -> Result<Vec<Identity>>;

    /// Read the account for `user_id`.
    async fn get_account(&self, user_id: &str) -> Result<Account>;

    /// Persist `user_id`'s preferred locale, last-writer-wins (change:
    /// persist-user-locale). A **no-op when `locale` is empty**, so a call that
    /// carries no language never clears a stored preference. Written by the auth
    /// module after resolving the user on any locale-carrying call.
    async fn set_locale(&self, user_id: &str, locale: &str) -> Result<()>;

    /// Read `user_id`'s stored preferred locale, if any (`None` = never recorded,
    /// treated as English by the caller). Consulted as the email-localization
    /// fallback when a request carries no locale.
    async fn locale(&self, user_id: &str) -> Result<Option<String>>;

    /// Update profile/preferences with optimistic concurrency on `expected_version`.
    /// When `handle` is `Some`, validate and (re)assign it, enforcing
    /// case-insensitive uniqueness; when `None`, the stored handle is unchanged.
    async fn update_account(
        &self,
        user_id: &str,
        display_name: Option<String>,
        handle: Option<String>,
        preferences: &str,
        expected_version: i64,
    ) -> Result<Account>;

    /// Whether `handle` is currently free (advisory; the write path is the
    /// authority). Errors with `InvalidArgument` when the handle fails policy.
    async fn check_handle_availability(&self, handle: &str) -> Result<bool>;

    /// Erase the account, its identities, and its roles.
    async fn delete_account(&self, user_id: &str) -> Result<()>;

    /// Effective roles for `scope` (the account's `global` roles plus that scope).
    async fn effective_roles(&self, user_id: &str, scope: &str) -> Result<Vec<String>>;

    /// Roles grouped by scope, for each requested scope the account actually holds a
    /// role in (change: scope-aware-role-admin). Used to mint a back-office token
    /// that carries the admin's real roles across `global`/`music`/`live` so
    /// authorization can be scope-matched. Scopes with no role are omitted.
    async fn scoped_effective_roles(
        &self,
        user_id: &str,
        scopes: &[String],
    ) -> Result<BTreeMap<String, Vec<String>>>;

    /// Grant `role` in `scope` to `user_id`, idempotently (granting a held role is a
    /// no-op success), recording an audit entry attributed to `acting_admin`
    /// (change: add-moderation-back-office). The role/scope are validated against the
    /// recognized sets; authorization (admin-only) is enforced by the caller.
    async fn grant_role(
        &self,
        acting_admin: &str,
        user_id: &str,
        scope: &str,
        role: &str,
    ) -> Result<()>;

    /// Revoke `role` in `scope` from `user_id`, recording an audit entry. Revoking a
    /// role the account does not hold is a no-op success.
    async fn revoke_role(
        &self,
        acting_admin: &str,
        user_id: &str,
        scope: &str,
        role: &str,
    ) -> Result<()>;

    /// The role-grant audit history for `user_id`, most recent first — answers "who
    /// granted/revoked which role, and when", independent of current role state.
    async fn list_role_grants(&self, user_id: &str) -> Result<Vec<RoleGrant>>;

    /// A page of the admin account directory (change: add-admin-account-directory):
    /// accounts with their roles **grouped by `scopes`**, plus the total matching
    /// count. Only roles in `scopes` are returned, so the caller passes exactly the
    /// scopes it is authorized to administer and the directory can never leak roles
    /// from another scope (change: scope-aware-role-admin). `query` filters by handle
    /// (prefix, case-insensitive) or a `local` identity's email; an empty `query`
    /// lists all accounts. Authorization (admin-only) is enforced by the caller.
    async fn list_accounts(
        &self,
        query: &str,
        limit: i64,
        offset: i64,
        scopes: &[String],
    ) -> Result<AccountPage>;

    // --- Public player profile (change: add-play-activity-profile) -----------

    /// Read the profile of `target_id` as seen by `viewer_id`. The owner always
    /// sees their own profile in full; another player sees it only when it is
    /// `Public` AND the owner is age-eligible — otherwise this is `NotFound`
    /// (fail-closed: never reveal a private profile's existence). `today` is the
    /// current UTC date, injected so the eligibility check is deterministic.
    async fn get_player_profile(
        &self,
        viewer_id: &str,
        target_id: &str,
        today: NaiveDate,
    ) -> Result<PlayerProfile>;

    /// Set the caller's own visibility. Going `Public` is gated by the minimum-age
    /// safeguard (design D6): a supplied `date_of_birth` always (re-)derives
    /// `share_eligible_from = dob + min_public_sharing_age years` and is then
    /// discarded (only the derived date is stored); with no DOB the stored date is
    /// used, and if there is none the call is refused. REFUSES (fail-closed) when
    /// the user is not yet eligible on `today` (UTC, one-day margin). Returns the
    /// visibility now in effect. `Private` is always allowed and never touches the
    /// age data.
    async fn set_profile_visibility(
        &self,
        user_id: &str,
        visibility: Visibility,
        date_of_birth: Option<NaiveDate>,
        today: NaiveDate,
    ) -> Result<Visibility>;

    /// Whether `viewer_id` may see `owner_id`'s play activity (the heatmap): true
    /// when the viewer IS the owner, or the owner's profile is `Public` AND the
    /// owner is age-eligible. Fail-closed. Used in-process by the music
    /// `PlayService` to gate the activity read without crossing schemas. `today`
    /// is the current UTC date.
    async fn activity_visible_to(
        &self,
        owner_id: &str,
        viewer_id: &str,
        today: NaiveDate,
    ) -> Result<bool>;
}

/// gRPC **client** adapter for the public account-management surface — used to
/// reach an *extracted* user service (design D0/D1). The in-process internal
/// methods (resolve/provision, link/unlink, roles) stay on the direct adapter.
pub struct GrpcUserClient {
    inner: proto::user_service_client::UserServiceClient<tonic::transport::Channel>,
}

impl GrpcUserClient {
    pub fn new(channel: tonic::transport::Channel) -> Self {
        Self {
            inner: proto::user_service_client::UserServiceClient::new(channel),
        }
    }

    /// Read the caller's account (auth carried in request metadata).
    pub async fn get_account(&mut self) -> std::result::Result<proto::Account, tonic::Status> {
        Ok(self
            .inner
            .get_account(proto::GetAccountRequest {})
            .await?
            .into_inner())
    }
}
