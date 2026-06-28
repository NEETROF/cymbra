//! Pure, host-testable selection logic for the orphan reaper
//! (change: fix-handle-onboarding-escape). The Postgres query and the in-memory
//! fake both filter with [`reapable`]; the SQL mirrors it.

/// The cutoff instant (unix seconds): a handle-less account created strictly
/// before this predates the grace period and may be reaped.
pub fn cutoff(now_unix: i64, grace_secs: i64) -> i64 {
    now_unix - grace_secs
}

/// An account is reapable iff it has **no handle** (onboarding never completed)
/// and was created **before** the cutoff. An account exactly at the cutoff is
/// kept (strict `<`), as is any account that has a handle.
pub fn reapable(handle: Option<&str>, created_at_unix: i64, cutoff_unix: i64) -> bool {
    handle.is_none() && created_at_unix < cutoff_unix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoff_subtracts_grace() {
        assert_eq!(cutoff(1_000, 600), 400);
    }

    #[test]
    fn handle_less_and_old_is_reapable() {
        // created at 100, cutoff 400 → older than grace
        assert!(reapable(None, 100, 400));
    }

    #[test]
    fn account_with_handle_is_never_reapable() {
        assert!(!reapable(Some("alice"), 100, 400));
    }

    #[test]
    fn recent_account_is_kept() {
        assert!(!reapable(None, 500, 400)); // created after the cutoff
    }

    #[test]
    fn exactly_at_cutoff_is_kept() {
        assert!(!reapable(None, 400, 400)); // strict <, so boundary survives
    }
}
