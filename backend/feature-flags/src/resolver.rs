//! Admin-scope resolution seam.
//!
//! The internal access token flattens scoped roles: a `music/admin` and a
//! `global/admin` both surface as bare `admin` for the `music` audience, so a
//! handler cannot tell a per-app admin from a platform admin by the token alone.
//! Changing an `all`-scoped key (or another app's key) requires the **platform**
//! (global) admin, so the admin surface asks this resolver, backed by the user
//! account's scoped roles (queried on the user-owned pool at the composition root).
//! Unit tests mock it ([`MockAdminScopeResolver`]).

use async_trait::async_trait;
use cymbra_platform::error::Result;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait AdminScopeResolver: Send + Sync {
    /// True when the account holds the platform-wide (`global`) admin role.
    async fn is_platform_admin(&self, user_id: &str) -> Result<bool>;
}
