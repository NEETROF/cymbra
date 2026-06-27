//! Optimistic-concurrency helper (task 3.4) — pure and host-tested.

use cymbra_platform::{AppError, Result};

/// Succeed only when the client-supplied `expected` version matches `stored`;
/// otherwise an `Aborted` conflict carrying the current server version.
pub fn check(stored: i64, expected: i64) -> Result<()> {
    if stored == expected {
        Ok(())
    } else {
        Err(AppError::Aborted(format!(
            "version conflict: expected {expected}, server has {stored}"
        )))
    }
}

/// Next version after a successful write.
pub fn next(current: i64) -> i64 {
    current + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_ok_mismatch_aborts() {
        assert!(check(5, 5).is_ok());
        assert_eq!(next(5), 6);
        assert!(matches!(check(6, 5), Err(AppError::Aborted(_))));
    }
}
