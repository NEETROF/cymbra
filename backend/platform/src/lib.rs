//! `cymbra-platform` — cross-cutting primitives shared by every Cymbra ID module.
//!
//! Per design D3/D9 this crate will own: typed config, telemetry (the three OTel
//! signals), the internal-token JWT codec + interceptor, the OIDC/JWKS
//! verification helper, an argon2id hashing helper, the email-sender port, the
//! Redis client/port, the rate-limiter, and the [`AuthIdentity`] request context.
//! It MUST NOT depend on any module crate.
//!
//! Implemented across task group 2; this scaffold establishes the crate.

/// Verified caller identity injected by the internal-token interceptor (task 2.5).
///
/// `roles` is the **effective** set for the token's audience (`global` + that
/// app's scope), read from the user module at sign-in — never from a provider
/// token. Populated in group 2.
#[derive(Debug, Clone, Default)]
pub struct AuthIdentity {
    /// Internal account id (UUID v7) the token was issued for.
    pub user_id: String,
    /// Effective role names for the token's audience.
    pub roles: Vec<String>,
}

impl AuthIdentity {
    /// True when `role` is present in the effective set (`is_admin` == has "admin").
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}
