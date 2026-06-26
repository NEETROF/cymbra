//! Rate-limiting over the shared cache (task 2.8).
//!
//! `ratelimit_core` is the pure decision logic; [`check`] applies it against a
//! windowed Redis counter so limits hold across instances.

use crate::cache::Cache;
use crate::error::{AppError, Result};
use std::time::Duration;

/// Pure, host-testable rate-limit logic.
pub mod ratelimit_core {
    /// A request is allowed while the post-increment count stays within `max`.
    pub fn is_allowed(count_after_incr: u64, max: u32) -> bool {
        count_after_incr <= max as u64
    }

    /// Cache key for a (scope, subject) window.
    pub fn window_key(scope: &str, subject: &str) -> String {
        format!("rl:{scope}:{subject}")
    }
}

/// Count one attempt for `(scope, subject)`; error with `ResourceExhausted` once
/// the window exceeds `max`.
pub async fn check(
    cache: &dyn Cache,
    scope: &str,
    subject: &str,
    max: u32,
    window: Duration,
) -> Result<()> {
    let key = ratelimit_core::window_key(scope, subject);
    let n = cache.incr_with_ttl(&key, window).await?;
    if ratelimit_core::is_allowed(n, max) {
        Ok(())
    } else {
        Err(AppError::ResourceExhausted(format!(
            "too many {scope} attempts, try again later"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::FakeCache;

    #[test]
    fn core_decision() {
        assert!(ratelimit_core::is_allowed(5, 5));
        assert!(!ratelimit_core::is_allowed(6, 5));
    }

    #[tokio::test]
    async fn blocks_after_max() {
        let c = FakeCache::default();
        let w = Duration::from_secs(60);
        for _ in 0..3 {
            check(&c, "signin", "u1", 3, w).await.unwrap();
        }
        assert!(matches!(
            check(&c, "signin", "u1", 3, w).await,
            Err(AppError::ResourceExhausted(_))
        ));
    }
}
