// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Politeness primitives: exponential back-off with retry, and a concurrency
//! cap.
//!
//! These are transport-agnostic so both the git and (later) web adapters reuse
//! them. The back-off schedule is a pure function (deterministically tested);
//! [`retry_async`] retries only *transient* failures up to a bound; and
//! [`ConcurrencyLimiter`] wraps a Tokio semaphore to keep in-flight work within
//! a low cap.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Semaphore, SemaphorePermit};

/// An exponential back-off schedule, capped.
#[derive(Debug, Clone, Copy)]
pub struct Backoff {
    /// Delay for the first retry; doubles each subsequent attempt.
    pub base: Duration,
    /// Upper bound on any single delay.
    pub max: Duration,
    /// Maximum number of retries before giving up.
    pub max_retries: u32,
}

impl Default for Backoff {
    fn default() -> Self {
        Self {
            base: Duration::from_millis(500),
            max: Duration::from_secs(30),
            max_retries: 5,
        }
    }
}

impl Backoff {
    /// Delay before retry `attempt` (0-based): `base * 2^attempt`, capped at
    /// `max`. Pure — no sleeping.
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let mult = 1u32.checked_shl(attempt).unwrap_or(u32::MAX);
        self.base
            .checked_mul(mult)
            .unwrap_or(self.max)
            .min(self.max)
    }
}

/// Runs `op`, retrying while it returns a *transient* error, following `policy`.
/// Permanent errors (per `is_transient`) return immediately. Never panics.
pub async fn retry_async<T, E, F, Fut>(
    policy: &Backoff,
    is_transient: impl Fn(&E) -> bool,
    mut op: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempt = 0u32;
    loop {
        match op().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if attempt >= policy.max_retries || !is_transient(&e) {
                    return Err(e);
                }
                tokio::time::sleep(policy.delay_for(attempt)).await;
                attempt += 1;
            }
        }
    }
}

/// A low concurrency cap shared across tasks (default 2), so we never hammer a
/// host with many simultaneous requests.
#[derive(Clone)]
pub struct ConcurrencyLimiter {
    sem: Arc<Semaphore>,
}

impl ConcurrencyLimiter {
    /// Creates a limiter allowing `permits` concurrent holders (min 1).
    pub fn new(permits: usize) -> Self {
        Self {
            sem: Arc::new(Semaphore::new(permits.max(1))),
        }
    }

    /// Acquires a permit, awaiting if the cap is reached. Held until dropped.
    pub async fn acquire(&self) -> SemaphorePermit<'_> {
        // The semaphore is never closed, so acquire cannot fail.
        self.sem
            .acquire()
            .await
            .expect("crawler semaphore is never closed")
    }

    /// Permits currently available (for observability/tests).
    pub fn available(&self) -> usize {
        self.sem.available_permits()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn backoff_doubles_and_caps() {
        let b = Backoff {
            base: Duration::from_millis(100),
            max: Duration::from_millis(700),
            max_retries: 10,
        };
        assert_eq!(b.delay_for(0), Duration::from_millis(100));
        assert_eq!(b.delay_for(1), Duration::from_millis(200));
        assert_eq!(b.delay_for(2), Duration::from_millis(400));
        // 800ms would exceed the 700ms cap.
        assert_eq!(b.delay_for(3), Duration::from_millis(700));
        // Very large attempts saturate to the cap, never overflow/panic.
        assert_eq!(b.delay_for(64), Duration::from_millis(700));
    }

    fn fast() -> Backoff {
        Backoff {
            base: Duration::ZERO,
            max: Duration::ZERO,
            max_retries: 5,
        }
    }

    #[tokio::test]
    async fn retries_transient_then_succeeds() {
        let calls = AtomicUsize::new(0);
        let out: Result<u32, &str> = retry_async(
            &fast(),
            |_| true,
            || async {
                let n = calls.fetch_add(1, Ordering::SeqCst);
                if n < 2 { Err("temporary") } else { Ok(42) }
            },
        )
        .await;
        assert_eq!(out, Ok(42));
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn permanent_error_is_not_retried() {
        let calls = AtomicUsize::new(0);
        let out: Result<u32, &str> = retry_async(
            &fast(),
            |_| false,
            || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Err("fatal")
            },
        )
        .await;
        assert_eq!(out, Err("fatal"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn gives_up_after_max_retries() {
        let calls = AtomicUsize::new(0);
        let policy = Backoff {
            max_retries: 3,
            ..fast()
        };
        let out: Result<u32, &str> = retry_async(
            &policy,
            |_| true,
            || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Err("always")
            },
        )
        .await;
        assert_eq!(out, Err("always"));
        // Initial attempt + 3 retries.
        assert_eq!(calls.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn concurrency_limiter_caps_permits() {
        let limiter = ConcurrencyLimiter::new(2);
        assert_eq!(limiter.available(), 2);
        let p1 = limiter.acquire().await;
        let p2 = limiter.acquire().await;
        assert_eq!(limiter.available(), 0);
        drop(p1);
        assert_eq!(limiter.available(), 1);
        drop(p2);
        assert_eq!(limiter.available(), 2);
    }

    #[test]
    fn limiter_floors_at_one() {
        assert_eq!(ConcurrencyLimiter::new(0).available(), 1);
    }
}
