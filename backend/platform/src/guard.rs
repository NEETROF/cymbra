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
}
