//! Bounded retry policy with exponential backoff (design D6). Pure and
//! host-testable. Configurable per `(module, kind)` and read from
//! `jobs.retry_policy` at enqueue time; the resolved policy is mapped onto the
//! sqlxmq job (`set_retries` / `set_retry_backoff`) by the engine glue.
//!
//! sqlxmq counts *retries* (attempts after the first) and doubles the backoff on
//! every retry without an upper bound. We model the policy in terms of total
//! *attempts* and expose a capped [`RetryPolicy::backoff_for`] so our own
//! reasoning (and any non-sqlxmq engine) honors `max_backoff`.

use std::time::Duration;

/// Retry/backoff policy for a job type. `max_attempts` is the **total** number
/// of tries (initial attempt + retries), so it is always ≥ 1 — the system never
/// retries indefinitely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    max_attempts: u32,
    base_backoff: Duration,
    max_backoff: Duration,
}

impl RetryPolicy {
    /// Build a policy. `max_attempts` is clamped to a minimum of 1; `max_backoff`
    /// is clamped to be at least `base_backoff`.
    pub fn new(max_attempts: u32, base_backoff: Duration, max_backoff: Duration) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            base_backoff,
            max_backoff: max_backoff.max(base_backoff),
        }
    }

    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    pub fn base_backoff(&self) -> Duration {
        self.base_backoff
    }

    pub fn max_backoff(&self) -> Duration {
        self.max_backoff
    }

    /// The number of *retries* after the first attempt — what sqlxmq's
    /// `set_retries` expects (`max_attempts - 1`).
    pub fn sqlxmq_retries(&self) -> u32 {
        self.max_attempts - 1
    }

    /// Backoff before `attempt` (1-based: the wait *before* attempt N). Attempt 1
    /// has no preceding wait, so it is zero. Subsequent attempts back off
    /// exponentially from `base_backoff`, capped at `max_backoff`.
    pub fn backoff_for(&self, attempt: u32) -> Duration {
        if attempt <= 1 {
            return Duration::ZERO;
        }
        // wait before attempt N = base * 2^(N-2), capped.
        let shift = attempt - 2;
        let factor = 1u64.checked_shl(shift).unwrap_or(u64::MAX);
        let scaled = self
            .base_backoff
            .checked_mul(factor.min(u32::MAX as u64) as u32)
            .unwrap_or(self.max_backoff);
        scaled.min(self.max_backoff)
    }

    /// Whether a job that has already been attempted `attempts_made` times has
    /// exhausted its budget (no more retries → dead-letter).
    pub fn is_exhausted(&self, attempts_made: u32) -> bool {
        attempts_made >= self.max_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> RetryPolicy {
        RetryPolicy::new(5, Duration::from_secs(1), Duration::from_secs(60))
    }

    #[test]
    fn max_attempts_is_clamped_to_one() {
        assert_eq!(
            RetryPolicy::new(0, Duration::ZERO, Duration::ZERO).max_attempts(),
            1
        );
    }

    #[test]
    fn sqlxmq_retries_is_attempts_minus_one() {
        assert_eq!(p().sqlxmq_retries(), 4);
    }

    #[test]
    fn backoff_is_exponential_and_capped() {
        let p = p();
        assert_eq!(p.backoff_for(1), Duration::ZERO); // first attempt: no wait
        assert_eq!(p.backoff_for(2), Duration::from_secs(1)); // base * 2^0
        assert_eq!(p.backoff_for(3), Duration::from_secs(2)); // base * 2^1
        assert_eq!(p.backoff_for(4), Duration::from_secs(4)); // base * 2^2
        // base * 2^6 = 64s would exceed the 60s cap.
        assert_eq!(p.backoff_for(8), Duration::from_secs(60));
    }

    #[test]
    fn huge_attempt_saturates_to_max_backoff() {
        assert_eq!(p().backoff_for(1000), Duration::from_secs(60));
    }

    #[test]
    fn max_backoff_never_below_base() {
        let p = RetryPolicy::new(3, Duration::from_secs(10), Duration::from_secs(1));
        assert_eq!(p.max_backoff(), Duration::from_secs(10));
    }

    #[test]
    fn exhaustion_at_or_past_limit() {
        let p = p();
        assert!(!p.is_exhausted(4));
        assert!(p.is_exhausted(5));
        assert!(p.is_exhausted(6));
    }
}
