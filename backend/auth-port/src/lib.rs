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

/// A summary of one active refresh-token session (a family), for listing. Carries no
/// secret — the refresh token itself is never surfaced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSummary {
    /// Opaque session-family id (used to revoke a specific session).
    pub id: String,
    /// The app/audience this session was minted for (one login per app).
    pub audience: String,
}

/// The auth module's port: sign-up, verification, sign-in (local + OIDC), token
/// lifecycle (refresh/logout), password reset, and identity link/unlink.
///
/// `user_id` parameters on the authenticated operations (`link`/`unlink`) come
/// from the validated internal access token, supplied by the server adapter.
#[async_trait]
pub trait AuthPort: Send + Sync {
    async fn sign_up_local(&self, email: &str, password: &str) -> Result<()>;
    async fn verify_email(&self, token: &str) -> Result<()>;
    async fn resend_verification(&self, email: &str) -> Result<()>;
    async fn sign_in_local(&self, email: &str, password: &str, audience: &str)
    -> Result<TokenPair>;
    async fn sign_in_oidc(&self, id_token: &str, audience: &str) -> Result<TokenPair>;
    async fn refresh(&self, refresh_token: &str) -> Result<TokenPair>;
    async fn logout(&self, refresh_token: &str) -> Result<()>;
    /// List `user_id`'s active sessions (id + audience); no refresh token is returned.
    async fn list_sessions(&self, user_id: &str) -> Result<Vec<SessionSummary>>;
    /// Revoke one of `user_id`'s sessions by id (owner-scoped; foreign/absent → no-op).
    async fn revoke_session(&self, user_id: &str, session_id: &str) -> Result<()>;
    /// Revoke every session for `user_id` (self sign-out-everywhere).
    async fn revoke_all_sessions(&self, user_id: &str) -> Result<()>;
    /// Revoke every session for `target_user_id` **as an admin action**, recording a
    /// durable audit entry (acting admin + target + count). Authorization (the admin
    /// role) is enforced by the caller — the gRPC adapter.
    async fn revoke_account_sessions(&self, acting_admin: &str, target_user_id: &str)
    -> Result<()>;
    async fn request_password_reset(&self, email: &str) -> Result<()>;
    async fn reset_password(&self, token: &str, new_password: &str) -> Result<()>;
    async fn link_identity(&self, user_id: &str, id_token: &str) -> Result<()>;
    async fn unlink_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()>;
}
