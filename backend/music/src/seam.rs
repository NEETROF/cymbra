//! Optional outbound calls: the ones whose failure must not change the response.
//!
//! A handful of reads through `UserPort` decorate a response — a proposer's display
//! name, an opt-in public credit, a leaderboard handle. When the answer is absent the
//! field is simply omitted, and that is correct: a private profile has no public
//! credit, an unknown account has no name.
//!
//! They were written as `let Ok(x) = …` / `.ok()`, which discards **every** error the
//! same way. So a database that is down produced exactly the response a private
//! profile does, and nothing anywhere recorded it — a silent degradation that already
//! bites today, with no network involved (change: harden-module-boundaries, group 10).
//!
//! The visible behaviour is unchanged on purpose. Only the diagnostic is new.

use cymbra_platform::AppError;

/// Whether an error is the seam's own legitimate "there is nothing here" answer, as
/// opposed to a dependency failing.
///
/// `NotFound` is what a private or missing profile returns; `PermissionDenied` is what
/// a viewer not entitled to see it returns. Both mean the field is genuinely absent.
/// Everything else — `Internal`, `Unavailable`, a bad argument — means the question
/// was never answered, and omitting the field then hides a fault.
pub(crate) fn is_domain_outcome(e: &AppError) -> bool {
    matches!(e, AppError::NotFound(_) | AppError::PermissionDenied(_))
}

/// Resolve an optional decoration, recording a dependency failure without changing the
/// answer. `what` names the call, so the log line says which decoration was lost.
pub(crate) fn optional<T>(what: &str, r: cymbra_platform::Result<T>) -> Option<T> {
    match r {
        Ok(v) => Some(v),
        Err(e) if is_domain_outcome(&e) => None,
        Err(e) => {
            tracing::warn!(seam = what, error = %e, "optional seam failed; field omitted");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The classification is what decides whether a failure is recorded, so it is what
    /// the test pins. Both branches must yield `None` — that is the behaviour this
    /// change promises not to alter.
    #[test]
    fn a_missing_profile_is_silent_and_a_failure_is_not() {
        // Domain outcomes: absent, and nothing to report.
        for e in [
            AppError::NotFound("profile".into()),
            AppError::PermissionDenied("not visible".into()),
        ] {
            assert!(is_domain_outcome(&e), "{e} should be a domain outcome");
            assert!(optional::<u8>("x", Err(e)).is_none());
        }

        // Dependency failures: same absent field, but reportable.
        for e in [
            AppError::Unavailable("cache".into()),
            AppError::Internal(anyhow::anyhow!("pool exhausted")),
            AppError::InvalidArgument("bad uuid".into()),
        ] {
            assert!(!is_domain_outcome(&e), "{e} should be reportable");
            assert!(
                optional::<u8>("x", Err(e)).is_none(),
                "the response must not change"
            );
        }

        assert_eq!(optional("x", Ok(7u8)), Some(7));
    }
}
