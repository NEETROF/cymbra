//! OIDC verification helper (task 2.9).
//!
//! `oidc_core` holds the pure issuer/audience/expiry checks (host-tested). The
//! full provider-token verification (JWKS fetch + RS256 decode) is built on top
//! of this by the auth module's `OidcJwtVerifier` (task 4.3); the network fetch
//! is thin I/O glue excluded from the coverage gate.

use crate::error::{AppError, Result};

/// A normalized external identity produced by a verifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalIdentity {
    /// Provider key: `google`, `apple`, or `local`.
    pub provider: String,
    /// Stable subject (OIDC `sub`, or the email for `local`).
    pub subject: String,
    /// Email if the provider supplied one (may be a private relay; absent for some).
    pub email: Option<String>,
}

/// Pure OIDC claim checks shared by every OIDC verifier.
pub mod oidc_core {
    use super::{AppError, Result};

    /// Validate a decoded provider token's `iss` / `aud` / `exp`.
    ///
    /// `aud_matches` is computed by the caller (the `aud` claim may be a string or
    /// an array). `now`/`exp` are unix seconds.
    pub fn validate(
        iss: &str,
        expected_iss: &str,
        aud_matches: bool,
        exp: u64,
        now: u64,
    ) -> Result<()> {
        if iss != expected_iss {
            return Err(AppError::Unauthenticated("issuer mismatch".into()));
        }
        if !aud_matches {
            return Err(AppError::Unauthenticated("audience mismatch".into()));
        }
        if exp <= now {
            return Err(AppError::Unauthenticated("provider token expired".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::oidc_core::*;
    use crate::error::AppError;

    const ISS: &str = "https://accounts.google.com";

    #[test]
    fn accepts_valid() {
        assert!(validate(ISS, ISS, true, 2_000, 1_000).is_ok());
    }

    #[test]
    fn rejects_issuer_audience_expiry() {
        assert!(matches!(
            validate("https://evil", ISS, true, 2_000, 1_000),
            Err(AppError::Unauthenticated(_))
        ));
        assert!(matches!(
            validate(ISS, ISS, false, 2_000, 1_000),
            Err(AppError::Unauthenticated(_))
        ));
        assert!(matches!(
            validate(ISS, ISS, true, 900, 1_000),
            Err(AppError::Unauthenticated(_))
        ));
    }
}
