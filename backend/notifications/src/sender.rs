//! The send seam (change: add-push-notifications, task 2.1, design D5).
//!
//! [`PushSender`] is the port every dispatch goes through; the FCM HTTP v1
//! implementation lives in [`crate::fcm`] and is the only thing that talks to a
//! third party. The port is `#[automock]`ed so tests drive a `MockPushSender` and
//! **never** reach FCM.

use async_trait::async_trait;

/// One notification to deliver. Deliberately minimal and provider-agnostic: the
/// concrete notification *type* owns the copy (already localized by the feature
/// that declares it), the platform only transports it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PushMessage {
    pub title: String,
    pub body: String,
    /// Opaque key/value payload delivered alongside the notification — how a
    /// feature routes a tap to the right screen (e.g. `{"route": "/practice"}`).
    pub data: Vec<(String, String)>,
}

impl PushMessage {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            data: Vec::new(),
        }
    }

    /// Attach one routing/data entry.
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.push((key.into(), value.into()));
        self
    }
}

/// What happened to one send — the three outcomes the caller must distinguish
/// (design D5): success, a transient failure worth retrying, and a token that
/// will never work again and must be pruned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendOutcome {
    /// Accepted by the provider.
    Delivered,
    /// Transient failure (network, 5xx, quota) — the job may retry.
    Retryable(String),
    /// The token is unregistered/invalid — prune it from the registry.
    Invalid(String),
}

impl SendOutcome {
    pub fn is_delivered(&self) -> bool {
        matches!(self, SendOutcome::Delivered)
    }

    /// True when the token must be removed from the registry.
    pub fn is_invalid(&self) -> bool {
        matches!(self, SendOutcome::Invalid(_))
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, SendOutcome::Retryable(_))
    }
}

/// Delivers one message to one device token.
///
/// Implementations MUST NOT return `Err` for a per-token failure — that is what
/// [`SendOutcome`] is for. `Err` is reserved for a failure of the sender itself
/// (e.g. credentials that cannot be minted), which aborts the whole dispatch.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PushSender: Send + Sync {
    async fn send(&self, token: &str, msg: &PushMessage) -> anyhow::Result<SendOutcome>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_builder_carries_title_body_and_data() {
        let m = PushMessage::new("Streak", "Play today")
            .with_data("route", "/practice")
            .with_data("category", "practice_streak");
        assert_eq!(m.title, "Streak");
        assert_eq!(m.body, "Play today");
        assert_eq!(
            m.data,
            vec![
                ("route".to_string(), "/practice".to_string()),
                ("category".to_string(), "practice_streak".to_string()),
            ]
        );
    }

    #[test]
    fn outcomes_are_mutually_exclusive() {
        let delivered = SendOutcome::Delivered;
        assert!(delivered.is_delivered() && !delivered.is_invalid() && !delivered.is_retryable());

        let retry = SendOutcome::Retryable("503".into());
        assert!(retry.is_retryable() && !retry.is_invalid() && !retry.is_delivered());

        let invalid = SendOutcome::Invalid("UNREGISTERED".into());
        assert!(invalid.is_invalid() && !invalid.is_retryable() && !invalid.is_delivered());
    }
}
