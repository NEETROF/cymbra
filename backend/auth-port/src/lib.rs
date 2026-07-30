//! `cymbra-auth-port` — the auth module's **contract** crate.
//!
//! Carries the [`AuthPort`] trait + DTOs, the generated protobuf types, and the
//! gRPC **client** adapter. Consumers depend on this crate only, never on
//! `cymbra-auth` (design D0).

use async_trait::async_trait;
use cymbra_platform::Result;

/// Generated protobuf messages + tonic client/server stubs for `cymbra.auth.v1`.
pub mod proto {
    tonic::include_proto!("cymbra.auth.v1");
}

/// Backend-issued session tokens (short access + sliding refresh).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
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
    /// action, recording a durable audit entry (acting admin + target + audience +
    /// count). Scoped to `audience` so a caller can't cut sessions in an app they don't
    /// administer; authorization (the admin role) is enforced by the caller (gRPC adapter).
    async fn revoke_account_sessions(
        &self,
        acting_admin: &str,
        target_user_id: &str,
        audience: &str,
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
