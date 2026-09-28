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

//! The ports the publication core depends on (change: add-discord-notifications,
//! design D1, D5). Declared here, by the consumer; implemented by the webhook
//! sender and the Postgres adapters in this crate, and doubled with `mockall`
//! in tests.

use async_trait::async_trait;

use crate::render::{Message, ScoreCard, SoundFontCard};

#[cfg(any(test, feature = "mock"))]
use mockall::automock;

/// Why a post did not go through, classified for the retry decision (design D5).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SendError {
    /// Nothing reached Discord and a later attempt can succeed: the connection
    /// failed, or Discord answered `429` / `5xx`. The job is retried.
    #[error("retryable: {0}")]
    Retryable(String),
    /// The request may or may not have been posted (a timeout after sending).
    /// Retrying could double-post, so it is recorded as failed, not retried.
    #[error("ambiguous: {0}")]
    Ambiguous(String),
    /// No attempt can fix it: `400` (malformed), `401`/`403` (refused), `404`
    /// (webhook deleted in Discord). Recorded, logged at `error`, not retried.
    #[error("terminal: {0}")]
    Terminal(String),
}

/// Classify a non-success HTTP status from Discord.
pub fn classify_status(status: u16) -> SendError {
    match status {
        429 | 500..=599 => SendError::Retryable(format!("discord answered {status}")),
        _ => SendError::Terminal(format!("discord answered {status}")),
    }
}

/// Posts a message to a named channel.
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait DiscordSender: Send + Sync {
    /// Whether a channel is configured (has a webhook). Unconfigured = no-op.
    fn serves(&self, channel: &str) -> bool;

    async fn publish(&self, channel: &str, message: &Message) -> Result<(), SendError>;
}

/// The outcome of claiming an announcement's dedup key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Claim {
    /// This attempt owns the key and may post.
    Claimed,
    /// Already posted, or failed terminally: nothing more to do.
    Settled,
    /// Another attempt claimed it recently and has not settled it; retry later.
    InFlight,
}

/// The idempotency ledger (design D5): claim → post → settle.
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait AnnouncementLedger: Send + Sync {
    /// Claim `key`. A claim left unsettled longer than `stale_after_secs` (a crash
    /// between claim and post) can be claimed again.
    async fn claim(&self, key: &str, stale_after_secs: i64) -> anyhow::Result<Claim>;
    /// The post went through.
    async fn mark_posted(&self, key: &str) -> anyhow::Result<()>;
    /// The post failed for good; `reason` is kept for the operator.
    async fn mark_failed(&self, key: &str, reason: &str) -> anyhow::Result<()>;
    /// Give the claim back: nothing was posted and the job will retry.
    async fn release(&self, key: &str) -> anyhow::Result<()>;
}

/// Reads the public fields of an announcement's subject at publication time.
/// Returns `None` when the subject is no longer public (rejected again, deleted).
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait AnnouncementSource: Send + Sync {
    async fn accepted_soundfont(&self, id: &str) -> anyhow::Result<Option<SoundFontCard>>;
    async fn accepted_score(&self, id: &str) -> anyhow::Result<Option<ScoreCard>>;
}

/// The back-office flags, read at publication time (design D8).
#[cfg_attr(any(test, feature = "mock"), automock)]
pub trait FlagView: Send + Sync {
    /// The boolean value of `key`; an unknown or unreadable key is **off**.
    fn enabled(&self, key: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limits_and_server_errors_are_retryable() {
        for s in [429, 500, 502, 503, 599] {
            assert!(matches!(classify_status(s), SendError::Retryable(_)), "{s}");
        }
    }

    #[test]
    fn client_errors_are_terminal() {
        for s in [400, 401, 403, 404, 413] {
            assert!(matches!(classify_status(s), SendError::Terminal(_)), "{s}");
        }
        assert_eq!(
            classify_status(404).to_string(),
            "terminal: discord answered 404"
        );
    }
}
