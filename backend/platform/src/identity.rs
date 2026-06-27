//! The verified caller identity injected by the internal-token interceptor.

/// Verified caller identity attached to a request after the internal access
/// token validates (task 2.5).
///
/// `roles` is the **effective** set for the token's audience (`global` + that
/// app's scope), stamped at sign-in from the user module — never read from a
/// provider token.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthIdentity {
    /// Internal account id (UUID v7) the token was issued for.
    pub user_id: String,
    /// App audience the token is scoped to (`music` / `live`).
    pub audience: String,
    /// Effective role names for the token's audience.
    pub roles: Vec<String>,
}

impl AuthIdentity {
    /// True when `role` is present in the effective set.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Convenience: `is_admin` == has the `admin` role.
    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_checks() {
        let id = AuthIdentity {
            user_id: "u1".into(),
            audience: "live".into(),
            roles: vec!["user".into(), "admin".into()],
        };
        assert!(id.has_role("admin"));
        assert!(id.is_admin());
        assert!(!id.has_role("broadcaster"));
    }
}
