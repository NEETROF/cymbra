//! Brute-force and email-send limits (change: fix-auth-lockout-dos): the tunables, the cache
//! keys and the decisions — pure and host-tested; [`crate::module::AuthModule`] applies them
//! against the shared cache.
//!
//! Every limit keys the email in normalized form, and the ones that can lock someone out
//! pair it with the client address, so a failure or a request from one address never
//! spends the budget of the same email elsewhere. Per-address and per-email ceilings bound
//! credential stuffing and distributed attacks.

use std::time::Duration;

/// The tunables, sourced from [`cymbra_platform::config::Config`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthLimits {
    /// Failed sign-ins for one (email, address) before that pair is locked.
    pub signin_max_attempts: u32,
    /// The lockout window, also the per-address failure window.
    pub signin_lockout: Duration,
    /// Failed sign-ins from one address across emails, per lockout window.
    pub signin_addr_max_failures: u32,
    /// Failed sign-ins for one email across addresses, per `signin_account_window`.
    pub signin_account_max_failures: u32,
    pub signin_account_window: Duration,
    /// Verification/reset emails for one (email, address), per `email_window`.
    pub email_max: u32,
    pub email_window: Duration,
    /// Emails (sign-up, verification, reset) from one address, per `email_addr_window`.
    pub email_addr_max: u32,
    pub email_addr_window: Duration,
    /// Verification/reset emails for one email across addresses, per `email_account_window`.
    pub email_account_max: u32,
    pub email_account_window: Duration,
}

impl AuthLimits {
    /// The pair limits as given, with the design's default ceilings (30 failures per
    /// lockout window per address, 200/1h per email, 20 emails/1h per address, 10/1h per
    /// email) — for callers that only tune the original knobs.
    pub fn with_default_ceilings(
        signin_max_attempts: u32,
        signin_lockout: Duration,
        email_max: u32,
        email_window: Duration,
    ) -> Self {
        const HOUR: Duration = Duration::from_secs(3600);
        Self {
            signin_max_attempts,
            signin_lockout,
            signin_addr_max_failures: 30,
            signin_account_max_failures: 200,
            signin_account_window: HOUR,
            email_max,
            email_window,
            email_addr_max: 20,
            email_addr_window: HOUR,
            email_account_max: 10,
            email_account_window: HOUR,
        }
    }
}

/// The form every limit keys an email in: trimmed and lower-cased, so letter case or
/// surrounding spaces never start a fresh counter.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Lockout counter for one (email, address).
pub fn signin_pair_key(email: &str, addr: &str) -> String {
    format!("signin:{}:{addr}", normalize_email(email))
}

/// Failed sign-ins from one address, across emails.
pub fn signin_addr_key(addr: &str) -> String {
    format!("signin-addr:{addr}")
}

/// Failed sign-ins for one email, across addresses.
pub fn signin_account_key(email: &str) -> String {
    format!("signin-acct:{}", normalize_email(email))
}

/// `ratelimit::check` scope for the per-address email budget (all email endpoints).
pub const EMAIL_ADDR_SCOPE: &str = "email_addr";
/// `ratelimit::check` scope for the per-email ceiling (verification and reset).
pub const EMAIL_ACCOUNT_SCOPE: &str = "email_acct";

/// `ratelimit::check` subject for one (email, address) under an endpoint scope.
pub fn email_pair_subject(email: &str, addr: &str) -> String {
    format!("{}:{addr}", normalize_email(email))
}

/// Whether a counter already at `count` refuses the next attempt.
pub fn at_limit(count: u32, max: u32) -> bool {
    count >= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_case_and_surrounding_spaces() {
        assert_eq!(normalize_email("  A@X.Dev "), "a@x.dev");
    }

    #[test]
    fn keys_share_the_normalized_email() {
        assert_eq!(
            signin_pair_key(" A@x.dev", "203.0.113.5"),
            "signin:a@x.dev:203.0.113.5"
        );
        assert_eq!(signin_account_key("a@X.DEV"), "signin-acct:a@x.dev");
        assert_eq!(signin_addr_key("203.0.113.5"), "signin-addr:203.0.113.5");
        assert_eq!(email_pair_subject("A@x.dev ", "unknown"), "a@x.dev:unknown");
    }

    #[test]
    fn a_counter_at_its_max_refuses() {
        assert!(!at_limit(4, 5));
        assert!(at_limit(5, 5));
    }

    #[test]
    fn default_ceilings_keep_the_pair_limits() {
        let l = AuthLimits::with_default_ceilings(
            5,
            Duration::from_secs(900),
            3,
            Duration::from_secs(3600),
        );
        assert_eq!((l.signin_max_attempts, l.email_max), (5, 3));
        assert_eq!(
            (l.signin_addr_max_failures, l.signin_account_max_failures),
            (30, 200)
        );
        assert_eq!((l.email_addr_max, l.email_account_max), (20, 10));
    }
}
