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

//! The webhook [`DiscordSender`] (change: add-discord-notifications, design D2) —
//! thin `reqwest` glue, excluded from the coverage gate; the decisions it feeds
//! (status classification, webhook parsing) are unit-tested where they live.
//!
//! A webhook URL is a **secret**: anyone holding it can post as Cymbra in that
//! channel. It is read from the environment only, never logged, and stripped
//! from every error this module produces.

use std::collections::BTreeMap;
use std::time::Duration;

use async_trait::async_trait;

use crate::ports::{DiscordSender, SendError, classify_status};
use crate::render::Message;

/// Discord's `SUPPRESS_EMBEDS` message flag: a link in a title never unfurls.
const SUPPRESS_EMBEDS: u32 = 1 << 2;

/// Posts through one webhook per channel.
pub struct WebhookDiscordSender {
    client: reqwest::Client,
    hooks: BTreeMap<String, String>,
}

impl std::fmt::Debug for WebhookDiscordSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Channel names only — never the URLs.
        f.debug_struct("WebhookDiscordSender")
            .field("channels", &self.hooks.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl WebhookDiscordSender {
    /// `hooks` maps a channel name to its webhook URL (see
    /// [`crate::routing::webhooks_from_env`]). `None` when there is none.
    pub fn new(hooks: BTreeMap<String, String>) -> anyhow::Result<Option<Self>> {
        if hooks.is_empty() {
            return Ok(None);
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()?;
        Ok(Some(Self { client, hooks }))
    }
}

#[async_trait]
impl DiscordSender for WebhookDiscordSender {
    fn serves(&self, channel: &str) -> bool {
        self.hooks.contains_key(channel)
    }

    async fn publish(&self, channel: &str, message: &Message) -> Result<(), SendError> {
        let Some(url) = self.hooks.get(channel) else {
            return Err(SendError::Terminal(format!("no webhook for #{channel}")));
        };
        let body = serde_json::json!({
            "content": message.content,
            // Never ping anyone, whatever the text contains.
            "allowed_mentions": { "parse": [] },
            "flags": SUPPRESS_EMBEDS,
        });
        let response = self
            .client
            .post(url)
            .query(&[("wait", "true")])
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                let connect = e.is_connect();
                let e = e.without_url();
                if connect {
                    SendError::Retryable(format!("connect to discord: {e}"))
                } else {
                    // The request may have reached Discord: do not risk a repost.
                    SendError::Ambiguous(format!("post to discord: {e}"))
                }
            })?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(classify_status(status.as_u16()))
        }
    }
}
