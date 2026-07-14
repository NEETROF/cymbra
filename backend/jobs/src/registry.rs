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
/// Stable name of the session-reaper job (change: durable-sessions-postgres).
pub const SESSION_REAP: &str = "session_reap";
/// Stable name of the account-purge job (change: complete-account-deletion).
/// Payload `{ user_id }`. Run by `cymbra-worker` as `admin_svc` to erase a
/// deleted user's data across the `user_account` and `auth` schemas atomically.
pub const PURGE_USER: &str = "purge_user";
/// Stable name of the stored-object cleanup job (change: add-user-score-upload).
/// Payload `{ object_key }`. Deletes one object from the store (idempotent). One
/// is enqueued per user upload during account erasure (and, later, on a failed
/// single-score object delete).
pub const PURGE_SCORE_OBJECT: &str = "purge_score_object";

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
        JobSpec::new(
            SESSION_REAP,
            // Delete expired `auth.sessions` rows; dedup'd per occurrence, so
            // ordering is moot — parallel to avoid head-of-line blocking.
            Channel::parallel("auth", "reap"),
            RetryPolicy::new(3, Duration::from_secs(30), Duration::from_secs(600)),
        ),
        JobSpec::new(
            PURGE_USER,
            // Each purge targets a distinct user and is independent → parallel.
            // Erasure is idempotent, so retries after a partial failure are safe.
            Channel::parallel("user", "purge"),
            RetryPolicy::new(5, Duration::from_secs(30), Duration::from_secs(3600)),
        ),
        JobSpec::new(
            PURGE_SCORE_OBJECT,
            // Each object delete is independent and idempotent → parallel; retry
            // generously since a transient S3 outage should not drop the object.
            Channel::parallel("music", "purge"),
            RetryPolicy::new(8, Duration::from_secs(30), Duration::from_secs(3600)),
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
        assert!(names.contains(&SESSION_REAP.to_string()));
        assert!(names.contains(&PURGE_USER.to_string()));
        assert!(names.contains(&PURGE_SCORE_OBJECT.to_string()));
    }

    #[test]
    fn purge_score_object_is_parallel_on_music_with_generous_retries() {
        let s = spec(PURGE_SCORE_OBJECT).unwrap();
        assert_eq!(s.channel().name(), "music.purge");
        assert!(!s.channel().is_ordered());
        assert_eq!(s.default_retry().max_attempts(), 8);
    }

    #[test]
    fn purge_user_spec_is_parallel_on_the_user_module() {
        let s = spec(PURGE_USER).unwrap();
        assert_eq!(s.channel().name(), "user.purge");
        assert!(!s.channel().is_ordered());
        assert_eq!(s.default_retry().max_attempts(), 5);
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
