//! Evaluation context and rollout scope.
//!
//! Two orthogonal scoping dimensions (design D0/D5):
//! - **app scope** — a key applies to `all` apps or one specific app; resolved
//!   from the caller's token audience.
//! - **rollout scope** — an override applies to everyone (`global`), only staff
//!   (`staff_only`), premium-plan holders (`premium_only`), or the members of one
//!   beta campaign (`beta:<campaign>`); resolved from the caller's roles, effective
//!   plan and beta memberships (change: add-premium-subscription).

use std::collections::BTreeSet;

/// The shared app-scope sentinel: a key scoped to `all` applies to every app.
pub const APP_ALL: &str = "all";

/// Who an override applies to within an app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RolloutScope {
    /// Applies to everyone.
    Global,
    /// Applies only to admin/moderator identities (pre-rollout dogfooding).
    StaffOnly,
    /// Applies to identities whose effective plan is `premium`, and to staff.
    PremiumOnly,
    /// Applies to active members of one beta campaign (by key), and to staff —
    /// **not** to premium payers outside the campaign (they pay for stability).
    Beta(String),
}

impl RolloutScope {
    /// The stored / wire form: `global`, `staff_only`, `premium_only`, `beta:<key>`.
    pub fn to_key(&self) -> String {
        match self {
            RolloutScope::Global => "global".into(),
            RolloutScope::StaffOnly => "staff_only".into(),
            RolloutScope::PremiumOnly => "premium_only".into(),
            RolloutScope::Beta(k) => format!("beta:{k}"),
        }
    }

    /// Parse the stored / wire form; a `beta:` key must be `[a-z0-9-]+`.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "global" => RolloutScope::Global,
            "staff_only" => RolloutScope::StaffOnly,
            "premium_only" => RolloutScope::PremiumOnly,
            other => {
                let key = other.strip_prefix("beta:")?;
                if key.is_empty()
                    || !key
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                {
                    return None;
                }
                RolloutScope::Beta(key.to_string())
            }
        })
    }

    /// True for the plan-/beta-scoped rollouts (the console marks them distinctly).
    pub fn is_plan_scoped(&self) -> bool {
        matches!(self, RolloutScope::PremiumOnly | RolloutScope::Beta(_))
    }
}

/// Where the flags service learns a caller's plan dimensions (change:
/// add-premium-subscription): the effective plan and the active beta campaign
/// keys. Implemented by the server over the plans module; [`NoPlanContext`]
/// answers "free, no betas" (plans not deployed).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait::async_trait]
pub trait PlanContextSource: Send + Sync {
    /// `(premium, beta campaign keys)` for `user_id`; errors are treated as free.
    async fn plan_context(&self, user_id: &str) -> (bool, Vec<String>);
}

/// The inert plan-context source.
pub struct NoPlanContext;

#[async_trait::async_trait]
impl PlanContextSource for NoPlanContext {
    async fn plan_context(&self, _user_id: &str) -> (bool, Vec<String>) {
        (false, Vec::new())
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
    /// True when the caller's effective plan is `premium` (change:
    /// add-premium-subscription); `false` for anonymous callers and while the
    /// plans kill-switch is off.
    pub premium: bool,
    /// Keys of the beta campaigns the caller is an active member of.
    pub betas: BTreeSet<String>,
}

impl EvalContext {
    /// Build from the authenticated identity's audience + effective roles.
    pub fn authenticated(app: &str, roles: &[String]) -> Self {
        let staff = roles.iter().any(|r| r == "admin" || r == "moderator");
        Self {
            app: app.to_string(),
            staff,
            premium: false,
            betas: BTreeSet::new(),
        }
    }

    /// Build an anonymous (non-staff) context for the given app.
    pub fn anonymous(app: &str) -> Self {
        Self {
            app: app.to_string(),
            staff: false,
            premium: false,
            betas: BTreeSet::new(),
        }
    }

    /// Attach the caller's plan dimensions (effective plan + active beta keys).
    pub fn with_plan(mut self, premium: bool, betas: impl IntoIterator<Item = String>) -> Self {
        self.premium = premium;
        self.betas = betas.into_iter().collect();
        self
    }

    /// Whether a key declared for `key_app` applies to this caller's app: shared
    /// `all` keys always do; a specific-app key only for that exact app.
    pub fn app_matches(&self, key_app: &str) -> bool {
        key_app == APP_ALL || key_app == self.app
    }

    /// Whether an override at `rollout` reaches this caller. Staff reach every
    /// scope (dogfooding); premium reaches `premium_only`; a beta campaign's
    /// members reach `beta:<key>` — whatever their plan — and nobody else does.
    pub fn rollout_reaches(&self, rollout: &RolloutScope) -> bool {
        match rollout {
            RolloutScope::Global => true,
            RolloutScope::StaffOnly => self.staff,
            RolloutScope::PremiumOnly => self.staff || self.premium,
            RolloutScope::Beta(key) => self.staff || self.betas.contains(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollout_scope_round_trips() {
        for s in [
            RolloutScope::Global,
            RolloutScope::StaffOnly,
            RolloutScope::PremiumOnly,
            RolloutScope::Beta("midi-drums".into()),
        ] {
            assert_eq!(RolloutScope::parse(&s.to_key()), Some(s));
        }
        assert_eq!(RolloutScope::parse("cohort"), None);
        assert_eq!(RolloutScope::parse("beta:"), None);
        assert_eq!(RolloutScope::parse("beta:Midi Drums"), None);
        assert!(RolloutScope::PremiumOnly.is_plan_scoped());
        assert!(!RolloutScope::StaffOnly.is_plan_scoped());
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
        assert!(staff.rollout_reaches(&RolloutScope::Global));
        assert!(staff.rollout_reaches(&RolloutScope::StaffOnly));
        assert!(user.rollout_reaches(&RolloutScope::Global));
        assert!(!user.rollout_reaches(&RolloutScope::StaffOnly));
    }

    /// The reach matrix of the plan-scoped rollouts (change: add-premium-
    /// subscription): premium_only ⊇ {premium, staff}; beta:<k> ⊇ {members of k,
    /// staff} — a premium payer outside the beta does NOT match.
    #[test]
    fn plan_scoped_rollouts_reach_matrix() {
        let free = EvalContext::authenticated("music", &["user".into()]);
        let premium = EvalContext::authenticated("music", &["user".into()]).with_plan(true, []);
        let member_free = EvalContext::authenticated("music", &["user".into()])
            .with_plan(false, ["midi-drums".to_string()]);
        let member_premium = EvalContext::authenticated("music", &["user".into()])
            .with_plan(true, ["midi-drums".to_string()]);
        let staff = EvalContext::authenticated("music", &["moderator".into()]);
        let drums = RolloutScope::Beta("midi-drums".into());
        let other = RolloutScope::Beta("other".into());

        assert!(!free.rollout_reaches(&RolloutScope::PremiumOnly));
        assert!(premium.rollout_reaches(&RolloutScope::PremiumOnly));
        assert!(staff.rollout_reaches(&RolloutScope::PremiumOnly));

        assert!(!free.rollout_reaches(&drums));
        assert!(
            !premium.rollout_reaches(&drums),
            "premium outside the beta never sees it"
        );
        assert!(member_free.rollout_reaches(&drums));
        assert!(member_premium.rollout_reaches(&drums));
        assert!(!member_free.rollout_reaches(&other));
        assert!(staff.rollout_reaches(&drums));
    }
}
