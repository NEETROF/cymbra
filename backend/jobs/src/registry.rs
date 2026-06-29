//! Job-type registry (task 2.4). The pure, shared description of each job type:
//! its stable `name` (used as the sqlxmq message name and the
//! `jobs.retry_policy` / `jobs.schedules` `kind`), its [`Channel`] (module +
//! ordering), and a built-in default [`RetryPolicy`]. Producers look a spec up to
//! enqueue; the worker looks the same specs up to register handlers and build the
//! channel allow-list. Runtime overrides come from `jobs.retry_policy`.

use std::time::Duration;

use crate::channel::Channel;
use crate::retry::RetryPolicy;

/// Stable name of the verification-email job (first slice, design D10).
pub const VERIFICATION_EMAIL: &str = "verification_email";
/// Stable name of the orphan-reaper job (first slice, design D10).
pub const ORPHAN_REAP: &str = "orphan_reap";

/// Static description of one job type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSpec {
    name: String,
    channel: Channel,
    default_retry: RetryPolicy,
}

impl JobSpec {
    pub fn new(name: impl Into<String>, channel: Channel, default_retry: RetryPolicy) -> Self {
        Self {
            name: name.into(),
            channel,
            default_retry,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn default_retry(&self) -> &RetryPolicy {
        &self.default_retry
    }
}

/// The built-in job-type registry. Returned as a `Vec` so producers and the
/// worker share one source of truth; extend this as new job types land.
pub fn builtin() -> Vec<JobSpec> {
    vec![
        JobSpec::new(
            VERIFICATION_EMAIL,
            // Email to distinct recipients is independent → parallel.
            Channel::parallel("auth", "email"),
            RetryPolicy::new(5, Duration::from_secs(10), Duration::from_secs(3600)),
        ),
        JobSpec::new(
            ORPHAN_REAP,
            // Maintenance sweep; only one is enqueued per occurrence (dedup), so
            // ordering is moot — keep it parallel so it never head-of-line blocks.
            Channel::parallel("user", "reap"),
            RetryPolicy::new(3, Duration::from_secs(30), Duration::from_secs(600)),
        ),
    ]
}

/// Look a spec up by job name.
pub fn spec(name: &str) -> Option<JobSpec> {
    builtin().into_iter().find(|s| s.name() == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_has_first_slice_jobs() {
        let names: Vec<_> = builtin().iter().map(|s| s.name().to_string()).collect();
        assert!(names.contains(&VERIFICATION_EMAIL.to_string()));
        assert!(names.contains(&ORPHAN_REAP.to_string()));
    }

    #[test]
    fn spec_lookup_resolves_channel_and_retry() {
        let s = spec(VERIFICATION_EMAIL).unwrap();
        assert_eq!(s.channel().name(), "auth.email");
        assert!(!s.channel().is_ordered());
        assert_eq!(s.default_retry().max_attempts(), 5);
    }

    #[test]
    fn unknown_name_has_no_spec() {
        assert!(spec("nope").is_none());
    }
}
