//! The push registry behind a trait (change: add-push-notifications, task 1.2) —
//! device tokens, per-category consent and the per-user timezone.
//!
//! Behind a trait so the gRPC adapter and the dispatch loop are unit-testable with
//! a `mockall`-generated `MockPushRegistry` (rust-testing default) — no DB needed.
//! The Postgres implementation lives in [`crate::pg`]; this module holds only the
//! contract and its pure, host-testable value types.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};

/// The platforms that can hold an FCM token (design D1).
///
/// Windows and Linux are deliberately absent: they have no reliable app-closed
/// push path, so their clients never register — and a request claiming one is
/// rejected here rather than silently storing an undeliverable token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Ios,
    Android,
    Macos,
}

impl Platform {
    /// The stored/wire identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Ios => "ios",
            Platform::Android => "android",
            Platform::Macos => "macos",
        }
    }

    /// Parse a client-supplied platform, case-insensitively.
    ///
    /// `windows` / `linux` are *known* platforms that are not FCM-capable; they
    /// get the same rejection as an unknown value — the message names the
    /// supported set so a client bug is obvious in the logs.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "ios" => Ok(Platform::Ios),
            "android" => Ok(Platform::Android),
            "macos" => Ok(Platform::Macos),
            other => Err(AppError::InvalidArgument(format!(
                "unsupported push platform {other:?} (expected ios, android or macos)"
            ))),
        }
    }
}

/// One candidate row for a category send: a registered device plus the two
/// per-user inputs selection needs (the explicit category choice and the
/// timezone).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub user_id: String,
    pub token: String,
    pub platform: String,
    /// IANA timezone name, `None` when the user has never reported one.
    pub timezone: Option<String>,
    /// The user's *explicit* choice for this category. `None` means the user has
    /// never expressed one, so the category's product default applies.
    pub pref: Option<bool>,
}

/// One stored category preference, as returned to the owning client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryPref {
    pub category: String,
    pub enabled: bool,
}

/// Who a dispatch considers, before any consent/flag/hour gate runs.
///
/// This is the seam a feature uses to bring its own candidate query (design D6):
/// it resolves *its* user set (e.g. "streak > 0 and hasn't played today") and
/// hands the ids over as [`Audience::Users`]; the platform then loads their
/// tokens/timezones/prefs and applies the shared gates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Audience {
    /// Every user holding at least one registered device token.
    All,
    /// Only these user ids (a feature-supplied candidate set).
    Users(Vec<String>),
}

/// Reads and writes over the push registry (`user_account.push_tokens`,
/// `user_account.notification_prefs`, `user_account.users.timezone`).
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PushRegistry: Send + Sync {
    /// Upsert a device token for `user_id`, refreshing its last-seen. Re-pointing
    /// an existing token at a different user is intentional: the token identifies
    /// an app install, so this is how a shared device follows the signed-in user.
    async fn register_token(&self, user_id: &str, token: &str, platform: Platform) -> Result<()>;

    /// Drop `token`, but only if `user_id` owns it (logout). A no-op otherwise.
    async fn unregister_token(&self, user_id: &str, token: &str) -> Result<()>;

    /// Drop `token` regardless of owner — the send path's reaction to an
    /// invalid-token outcome (design D5). Idempotent.
    async fn prune_token(&self, token: &str) -> Result<()>;

    /// Whether `user_id` has at least one registered device.
    async fn has_device(&self, user_id: &str) -> Result<bool>;

    /// Record `user_id`'s explicit choice for `category`.
    async fn set_pref(&self, user_id: &str, category: &str, enabled: bool) -> Result<()>;

    /// Every explicit category choice `user_id` has made.
    async fn prefs(&self, user_id: &str) -> Result<Vec<CategoryPref>>;

    /// Overwrite `user_id`'s stored IANA timezone.
    async fn set_timezone(&self, user_id: &str, timezone: &str) -> Result<()>;

    /// Read `user_id`'s stored timezone (`None` when unset).
    async fn timezone(&self, user_id: &str) -> Result<Option<String>>;

    /// Load the candidate rows for a `category` send over `audience`.
    async fn candidates(&self, category: &str, audience: &Audience) -> Result<Vec<Candidate>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_platforms_round_trip_case_insensitively() {
        for (input, expected) in [
            ("ios", Platform::Ios),
            ("iOS", Platform::Ios),
            ("android", Platform::Android),
            (" ANDROID ", Platform::Android),
            ("macos", Platform::Macos),
        ] {
            assert_eq!(Platform::parse(input).unwrap(), expected);
        }
        assert_eq!(Platform::Ios.as_str(), "ios");
        assert_eq!(Platform::Android.as_str(), "android");
        assert_eq!(Platform::Macos.as_str(), "macos");
    }

    #[test]
    fn desktop_and_unknown_platforms_are_rejected() {
        // Windows/Linux are not FCM-capable: registering one would store a token
        // that can never be delivered to.
        for bad in ["windows", "linux", "web", ""] {
            let err = Platform::parse(bad).unwrap_err();
            assert!(
                matches!(err, AppError::InvalidArgument(_)),
                "{bad:?} should be rejected, got {err:?}"
            );
        }
    }
}
