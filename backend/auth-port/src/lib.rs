//! `cymbra-auth-port` — the auth module's **contract** crate.
//!
//! Carries the [`AuthPort`] trait + DTOs and the generated protobuf types. Consumers
//! depend on this crate only, never on `cymbra-auth` (design D0).
//!
//! It carries no gRPC **client** adapter, and the header claimed one until group 7 of
//! harden-module-boundaries. The `.proto` here is the EXTERNAL contract, for the apps;
//! the boundary between modules is the trait, and an internal transport is written
//! when a module is split out, not before.

use async_trait::async_trait;
use cymbra_platform::Result;

/// Generated protobuf messages + tonic client/server stubs for `cymbra.auth.v1`.
// `tonic::Status` is large by design; newer clippy flags every generated
// client/server signature for it.
#[allow(clippy::result_large_err)]
pub mod proto {
    tonic::include_proto!("cymbra.auth.v1");
}

/// Backend-issued session tokens (short access + sliding refresh).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

/// Which of a target account's sessions an admin revocation may cut.
///
/// Deliberately NOT a plain list with "empty means everything": that shape is how a
/// filter silently becomes a no-op in the permissive direction. Here an empty
/// [`Self::Only`] revokes nothing, which is the safe failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevocationScope {
    /// Every audience — the `global/admin` break-glass.
    All,
    /// Only these audiences. An admin scoped to one product cuts that product's
    /// sessions and no other.
    Only(Vec<String>),
}

/// The auth module's port: sign-up, verification, sign-in (local + OIDC), token
/// lifecycle (refresh/logout), password reset, and identity link/unlink.
///
/// `user_id` parameters on the authenticated operations (`link`/`unlink`) come
/// from the validated internal access token, supplied by the server adapter.
// `#[automock]` sits ABOVE `#[async_trait]` so mockall sees the async trait
// before it desugars; gated on the `mock` feature (test-only, never shipped).
#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait AuthPort: Send + Sync {
    /// `locale` (optional; empty string = unset) selects the transactional-email
    /// language, falling back to English (change: template-backend-emails).
    async fn sign_up_local(&self, email: &str, password: &str, locale: &str) -> Result<()>;
    async fn verify_email(&self, token: &str) -> Result<()>;
    async fn resend_verification(&self, email: &str, locale: &str) -> Result<()>;
    async fn sign_in_local(&self, email: &str, password: &str, audience: &str)
    -> Result<TokenPair>;
    async fn sign_in_oidc(&self, id_token: &str, audience: &str) -> Result<TokenPair>;
    async fn refresh(&self, refresh_token: &str) -> Result<TokenPair>;
    async fn logout(&self, refresh_token: &str) -> Result<()>;
    /// Revoke every session for `user_id` (self sign-out-everywhere).
    async fn revoke_all_sessions(&self, user_id: &str) -> Result<()>;
    /// Revoke every session for `target_user_id` **within `audience`** as an admin
    /// action, recording a durable audit entry (acting admin + target + scope + count).
    ///
    /// `scope` is what the ACTING ADMIN is entitled to cut, derived from their roles —
    /// **not** the audience of their own token. Using the token's audience is how this
    /// used to work, and it meant the console (always `back-office`) cut the target's
    /// console sessions and left their app sessions alive, while reporting success.
    ///
    /// Authorization (the admin role) is enforced by the caller (gRPC adapter).
    async fn revoke_account_sessions(
        &self,
        acting_admin: &str,
        target_user_id: &str,
        scope: &RevocationScope,
    ) -> Result<()>;
    async fn request_password_reset(&self, email: &str, locale: &str) -> Result<()>;
    async fn reset_password(&self, token: &str, new_password: &str) -> Result<()>;
    async fn link_identity(&self, user_id: &str, id_token: &str) -> Result<()>;
    async fn unlink_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()>;
    /// Add a local (email+password) credential to `user_id` so the account can
    /// also sign in with email. The credential is created **unverified** and a
    /// verification email is sent — the password is usable only after the email
    /// is confirmed (mirrors `sign_up_local`). `AlreadyExists` if the account
    /// already has a local credential or the email is bound to another account.
    async fn set_local_credential(
        &self,
        user_id: &str,
        email: &str,
        password: &str,
        locale: &str,
    ) -> Result<()>;
}
