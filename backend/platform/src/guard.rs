//! Role-based authorization guard (task 2.11), reading roles from [`AuthIdentity`].

use crate::error::{AppError, Result};
use crate::identity::AuthIdentity;

/// Require `role` to be present in the caller's effective set, else
/// `PermissionDenied`.
pub fn require_role(id: &AuthIdentity, role: &str) -> Result<()> {
    if id.has_role(role) {
        Ok(())
    } else {
        Err(AppError::PermissionDenied(format!(
            "requires role `{role}`"
        )))
    }
}

/// `is_admin` == requires the `admin` role.
pub fn require_admin(id: &AuthIdentity) -> Result<()> {
    require_role(id, "admin")
}

/// Require the caller to be a moderator **or** an admin (change: add-moderation-
/// back-office). Because `AuthIdentity.roles` is the effective set for the token's
/// audience — the audience scope unioned with `global` — holding `moderator` here
/// means the caller is a moderator in that audience's scope (e.g. `music/moderator`),
/// and `admin` covers both a scope admin and a `global/admin` break-glass. This is
/// the authorization for every moderation operation (evaluate, the privileged
/// status filter, non-`accepted` fetch-bytes, moderation-oriented sort keys).
pub fn require_moderator_or_admin(id: &AuthIdentity) -> Result<()> {
    if id.has_role("admin") || id.has_role("moderator") {
        Ok(())
    } else {
        Err(AppError::PermissionDenied(
            "requires role `moderator` or `admin`".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(roles: &[&str]) -> AuthIdentity {
        AuthIdentity {
            user_id: "u".into(),
            audience: "live".into(),
            roles: roles.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn allows_holder_denies_others() {
        assert!(require_role(&id(&["user", "admin"]), "admin").is_ok());
        assert!(require_admin(&id(&["user"])).is_err());
        assert!(matches!(
            require_role(&id(&["user"]), "broadcaster"),
            Err(AppError::PermissionDenied(_))
        ));
    }

    #[test]
    fn moderator_or_admin_allows_either_and_denies_normal() {
        assert!(require_moderator_or_admin(&id(&["user", "moderator"])).is_ok());
        assert!(require_moderator_or_admin(&id(&["user", "admin"])).is_ok());
        assert!(matches!(
            require_moderator_or_admin(&id(&["user"])),
            Err(AppError::PermissionDenied(_))
        ));
    }
}
