//! The verified caller identity injected by the internal-token interceptor.

use std::collections::BTreeMap;

/// Verified caller identity attached to a request after the internal access
/// token validates (task 2.5).
///
/// `roles` is the **effective** set for the token's audience (`global` + that
/// app's scope), stamped at sign-in from the user module — never read from a
/// provider token. `roles_by_scope` carries the same information broken down by
/// the scope each role is held in, so authorization can be **scope-matched**
/// (change: scope-aware-role-admin); it is empty on legacy/flat tokens.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthIdentity {
    /// Internal account id (UUID v7) the token was issued for.
    pub user_id: String,
    /// App audience the token is scoped to (`music` / `live` / `back-office`).
    pub audience: String,
    /// Effective role names — the union across every scope in `roles_by_scope`.
    pub roles: Vec<String>,
    /// Roles grouped by the scope they are held in (`global` / `music` / `live`).
    pub roles_by_scope: BTreeMap<String, Vec<String>>,
}

impl AuthIdentity {
    /// True when `role` is present in the effective (flat) set.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Convenience: `is_admin` == has the `admin` role in any scope.
    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }

    /// True when the caller holds `role` **in `scope`** — i.e. it is present in the
    /// `global` break-glass scope or in `scope` itself. This is the scope-matched
    /// primitive: a `music/admin` is admin in `music` but not in `live`, while a
    /// `global/admin` is admin in every scope (change: scope-aware-role-admin).
    pub fn has_role_in_scope(&self, scope: &str, role: &str) -> bool {
        let held_in = |s: &str| {
            self.roles_by_scope
                .get(s)
                .is_some_and(|rs| rs.iter().any(|r| r == role))
        };
        held_in("global") || held_in(scope)
    }

    /// The subset of `candidates` in which the caller holds `admin` — the scopes
    /// they are authorized to administer. A `global/admin` yields every candidate;
    /// a `music/admin` yields just `music`.
    pub fn admin_scopes(&self, candidates: &[&str]) -> Vec<String> {
        candidates
            .iter()
            .filter(|s| self.has_role_in_scope(s, "admin"))
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scoped(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(s, rs)| (s.to_string(), rs.iter().map(|r| r.to_string()).collect()))
            .collect()
    }

    #[test]
    fn role_checks() {
        let id = AuthIdentity {
            user_id: "u1".into(),
            audience: "live".into(),
            roles: vec!["user".into(), "admin".into()],
            roles_by_scope: BTreeMap::new(),
        };
        assert!(id.has_role("admin"));
        assert!(id.is_admin());
        assert!(!id.has_role("broadcaster"));
    }

    #[test]
    fn scope_matched_admin() {
        // A music-only admin: admin in music, not in live; global is empty.
        let music_admin = AuthIdentity {
            user_id: "u".into(),
            audience: "back-office".into(),
            roles: vec!["user".into(), "admin".into()],
            roles_by_scope: scoped(&[("global", &["user"]), ("music", &["admin"])]),
        };
        assert!(music_admin.has_role_in_scope("music", "admin"));
        assert!(!music_admin.has_role_in_scope("live", "admin"));
        assert!(!music_admin.has_role_in_scope("global", "admin"));
        assert_eq!(
            music_admin.admin_scopes(&["global", "music", "live"]),
            vec!["music".to_string()]
        );

        // A global admin is admin everywhere (break-glass).
        let global_admin = AuthIdentity {
            user_id: "u".into(),
            audience: "back-office".into(),
            roles: vec!["admin".into()],
            roles_by_scope: scoped(&[("global", &["admin"])]),
        };
        assert!(global_admin.has_role_in_scope("music", "admin"));
        assert!(global_admin.has_role_in_scope("live", "admin"));
        assert_eq!(
            global_admin.admin_scopes(&["global", "music", "live"]),
            vec![
                "global".to_string(),
                "music".to_string(),
                "live".to_string()
            ]
        );
    }
}
