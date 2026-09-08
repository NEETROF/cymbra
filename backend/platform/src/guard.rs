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

/// Require the caller to hold `moderator` or `admin` **in `scope`** — the
/// authorization for every moderation operation (evaluate, the privileged status
/// filter, non-`accepted` fetch-bytes, moderation-oriented sort keys).
///
/// This replaces a flat check whose doc-comment justified itself by claiming
/// `AuthIdentity.roles` was "the effective set for the token's audience". That is
/// true for an app audience, and **false for `back-office`**, whose token unions the
/// roles the administrator holds across *every* scope so one console session can
/// administer several products. On a console token the flat set therefore answered
/// "moderator somewhere", and a `live/moderator` passed a music moderation gate
/// (change: harden-module-boundaries, group 3).
pub fn require_moderator_or_admin_in_scope(id: &AuthIdentity, scope: &str) -> Result<()> {
    if is_staff_in_scope(id, scope) {
        Ok(())
    } else {
        Err(AppError::PermissionDenied(format!(
            "requires `moderator` or `admin` in scope `{scope}`"
        )))
    }
}

/// Whether the caller is staff **in `scope`** — `moderator` or `admin` there, or the
/// `global` break-glass. The same test [`require_moderator_or_admin_in_scope`] gates
/// on, exposed as a predicate for the places that *widen* what a caller sees rather
/// than refusing them: drum eligibility, leaderboard audience, flag rollout staffness.
///
/// One definition on purpose (change: harden-module-boundaries, task 3.13). It was
/// written out by hand in three places — twice in `cymbra-music`, once in
/// `EvalContext::authenticated` — all reading the FLAT role set, so none of them was
/// found by searching for the guard helpers.
pub fn is_staff_in_scope(id: &AuthIdentity, scope: &str) -> bool {
    id.has_role_in_scope(scope, "admin") || id.has_role_in_scope(scope, "moderator")
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
    fn moderator_or_admin_in_scope_allows_either_and_denies_normal() {
        let music_mod = scoped_id(&[("music", &["moderator"])]);
        let music_admin = scoped_id(&[("music", &["admin"])]);
        assert!(require_moderator_or_admin_in_scope(&music_mod, "music").is_ok());
        assert!(require_moderator_or_admin_in_scope(&music_admin, "music").is_ok());
        assert!(matches!(
            require_moderator_or_admin_in_scope(&scoped_id(&[("music", &["user"])]), "music"),
            Err(AppError::PermissionDenied(_))
        ));
    }

    /// The hole this guard replaced. `scoped_id` builds a **`back-office`** token, whose
    /// flat `roles` is the union across every scope — so a `live` moderator's flat set
    /// contains "moderator" and the old check said yes at a music gate. Asserted both
    /// ways: the flat set really does still contain the role, and the guard still
    /// refuses. Without the first assertion this test could pass for the wrong reason.
    #[test]
    fn a_moderator_of_another_product_is_refused_at_a_music_gate() {
        let live_mod = scoped_id(&[("live", &["moderator"])]);
        assert!(
            live_mod.has_role("moderator"),
            "precondition: the flat set is what the old guard read"
        );
        assert!(matches!(
            require_moderator_or_admin_in_scope(&live_mod, "music"),
            Err(AppError::PermissionDenied(_))
        ));
        // ...and remains a moderator where they actually hold it.
        assert!(require_moderator_or_admin_in_scope(&live_mod, "live").is_ok());
    }

    /// The break-glass must survive the tightening: a `global` admin passes everywhere.
    #[test]
    fn the_global_break_glass_still_passes_every_scope() {
        let global_admin = scoped_id(&[("global", &["admin"])]);
        for scope in crate::SCOPES {
            assert!(
                require_moderator_or_admin_in_scope(&global_admin, scope).is_ok(),
                "global/admin refused in {scope}"
            );
        }
    }

    /// A legacy token carries no `roles_by_scope`, so every scope-matched guard refuses
    /// it. Production only mints scoped claims (`token::new_claims_scoped`) and access
    /// tokens live 15 minutes, so this is the fail-closed direction, not a lockout —
    /// but it is asserted so the property is deliberate rather than incidental.
    #[test]
    fn a_flat_legacy_token_is_refused_everywhere() {
        let legacy = id(&["admin", "moderator"]);
        assert!(legacy.roles_by_scope.is_empty());
        assert!(require_moderator_or_admin_in_scope(&legacy, "music").is_err());
        assert!(require_admin_in_scope(&legacy, "music").is_err());
    }
}
