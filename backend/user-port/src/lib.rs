//! `cymbra-user-port` — the user module's **contract** crate.
//!
//! Carries the [`UserPort`] trait + DTOs, the generated protobuf types, and the
//! gRPC **client** adapter. The `cymbra-auth` module reaches accounts through this
//! crate — never `cymbra-user` directly (design D0).

use async_trait::async_trait;
use cymbra_platform::Result;

/// Generated protobuf messages + tonic client/server stubs for `cymbra.user.v1`.
pub mod proto {
    tonic::include_proto!("cymbra.user.v1");
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
}

/// One row of the admin account directory (change: add-admin-account-directory):
/// an account with the roles it holds in the `music` scope. Handle/display_name are
/// optional (handle-less accounts are onboarding-incomplete).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountSummary {
    pub user_id: String,
    pub handle: Option<String>,
    pub display_name: Option<String>,
    /// Roles held in the `music` scope (e.g. `moderator`, `admin`).
    pub roles: Vec<String>,
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
    /// accounts with their `music`-scope roles, plus the total matching count.
    /// `query` filters by handle (prefix, case-insensitive) or a `local` identity's
    /// email; an empty `query` lists all accounts. Authorization (admin-only) is
    /// enforced by the caller.
    async fn list_accounts(&self, query: &str, limit: i64, offset: i64) -> Result<AccountPage>;
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
