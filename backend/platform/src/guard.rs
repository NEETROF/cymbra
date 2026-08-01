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

/// `is_admin` == requires the `admin` role (in any scope — the coarse gate).
pub fn require_admin(id: &AuthIdentity) -> Result<()> {
    require_role(id, "admin")
}

/// Require the caller to hold `admin` **in `scope`** — i.e. admin in that scope
/// or the `global/admin` break-glass. This is the scope-matched authorization for
/// role administration: a `music/admin` may act on `music` but is refused on
/// `live`, and only a `global/admin` may act on the `global` scope (change:
/// scope-aware-role-admin).
pub fn require_admin_in_scope(id: &AuthIdentity, scope: &str) -> Result<()> {
    if id.has_role_in_scope(scope, "admin") {
        Ok(())
    } else {
        Err(AppError::PermissionDenied(format!(
            "requires `admin` in scope `{scope}`"
        )))
    }
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
            roles_by_scope: std::collections::BTreeMap::new(),
        }
    }

    /// An identity carrying per-scope roles, for scope-matched guard tests.
    fn scoped_id(pairs: &[(&str, &[&str])]) -> AuthIdentity {
        let roles_by_scope: std::collections::BTreeMap<String, Vec<String>> = pairs
            .iter()
            .map(|(s, rs)| (s.to_string(), rs.iter().map(|r| r.to_string()).collect()))
            .collect();
        let mut roles: Vec<String> = Vec::new();
        for rs in roles_by_scope.values() {
            for r in rs {
                if !roles.contains(r) {
                    roles.push(r.clone());
                }
            }
        }
        AuthIdentity {
            user_id: "u".into(),
            audience: "back-office".into(),
            roles,
            roles_by_scope,
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
    fn admin_in_scope_is_scope_matched() {
        // A music-only admin may act on music but is refused on live and global.
        let music_admin = scoped_id(&[("global", &["user"]), ("music", &["admin"])]);
        assert!(require_admin_in_scope(&music_admin, "music").is_ok());
        assert!(matches!(
            require_admin_in_scope(&music_admin, "live"),
            Err(AppError::PermissionDenied(_))
        ));
        assert!(matches!(
            require_admin_in_scope(&music_admin, "global"),
            Err(AppError::PermissionDenied(_))
        ));

        // A global admin (break-glass) may act on every scope, including global.
        let global_admin = scoped_id(&[("global", &["admin"])]);
        assert!(require_admin_in_scope(&global_admin, "music").is_ok());
        assert!(require_admin_in_scope(&global_admin, "live").is_ok());
        assert!(require_admin_in_scope(&global_admin, "global").is_ok());

        // A plain user is refused everywhere.
        let plain = scoped_id(&[("global", &["user"])]);
        assert!(matches!(
            require_admin_in_scope(&plain, "music"),
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
