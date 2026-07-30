//! Evaluation context and rollout scope.
//!
//! Two orthogonal scoping dimensions (design D0/D5):
//! - **app scope** — a key applies to `all` apps or one specific app; resolved
//!   from the caller's token audience.
//! - **rollout scope** — an override applies to everyone (`global`) or only staff
//!   (`staff_only`); resolved from the caller's roles.

/// The shared app-scope sentinel: a key scoped to `all` applies to every app.
pub const APP_ALL: &str = "all";

/// Who an override applies to within an app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolloutScope {
    /// Applies to everyone.
    Global,
    /// Applies only to admin/moderator identities (pre-rollout dogfooding).
    StaffOnly,
}

impl RolloutScope {
    pub fn as_str(self) -> &'static str {
        match self {
            RolloutScope::Global => "global",
            RolloutScope::StaffOnly => "staff_only",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "global" => RolloutScope::Global,
            "staff_only" => RolloutScope::StaffOnly,
            _ => return None,
        })
    }
}

/// The caller context an evaluation resolves against.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EvalContext {
    /// The caller's app (token audience), e.g. `music`. Empty for an anonymous
    /// caller that named no app — only `all` keys then resolve.
    pub app: String,
    /// True when the caller holds a staff role (admin or moderator), which is what
    /// a `staff_only` override checks.
    pub staff: bool,
}

impl EvalContext {
    /// Build from the authenticated identity's audience + effective roles.
    pub fn authenticated(app: &str, roles: &[String]) -> Self {
        let staff = roles.iter().any(|r| r == "admin" || r == "moderator");
        Self {
            app: app.to_string(),
            staff,
        }
    }

    /// Build an anonymous (non-staff) context for the given app.
    pub fn anonymous(app: &str) -> Self {
        Self {
            app: app.to_string(),
            staff: false,
        }
    }

    /// Whether a key declared for `key_app` applies to this caller's app: shared
    /// `all` keys always do; a specific-app key only for that exact app.
    pub fn app_matches(&self, key_app: &str) -> bool {
        key_app == APP_ALL || key_app == self.app
    }

    /// Whether an override at `rollout` reaches this caller.
    pub fn rollout_reaches(&self, rollout: RolloutScope) -> bool {
        match rollout {
            RolloutScope::Global => true,
            RolloutScope::StaffOnly => self.staff,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollout_scope_round_trips() {
        for s in [RolloutScope::Global, RolloutScope::StaffOnly] {
            assert_eq!(RolloutScope::parse(s.as_str()), Some(s));
        }
        assert_eq!(RolloutScope::parse("cohort"), None);
    }

    #[test]
    fn staff_derived_from_roles() {
        assert!(EvalContext::authenticated("music", &["admin".into()]).staff);
        assert!(EvalContext::authenticated("music", &["moderator".into()]).staff);
        assert!(!EvalContext::authenticated("music", &["user".into()]).staff);
        assert!(!EvalContext::anonymous("music").staff);
    }

    #[test]
    fn app_matches_all_and_own_only() {
        let ctx = EvalContext::anonymous("music");
        assert!(ctx.app_matches(APP_ALL));
        assert!(ctx.app_matches("music"));
        assert!(!ctx.app_matches("live"));
    }

    #[test]
    fn rollout_reaches_by_staffness() {
        let staff = EvalContext::authenticated("music", &["admin".into()]);
        let user = EvalContext::anonymous("music");
        assert!(staff.rollout_reaches(RolloutScope::Global));
        assert!(staff.rollout_reaches(RolloutScope::StaffOnly));
        assert!(user.rollout_reaches(RolloutScope::Global));
        assert!(!user.rollout_reaches(RolloutScope::StaffOnly));
    }
}
