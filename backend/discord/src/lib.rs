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

//! `cymbra-discord` — Discord announcements (change: add-discord-notifications).
//!
//! A producer enqueues an [`AnnouncementEvent`] with [`pg::enqueue_notify`] in
//! the same transaction as its domain write; the worker's `discord_notify` job
//! hands it to a [`Publisher`], which decides whether and what to post and does
//! it at most once. Split like `cymbra-notifications`:
//!
//! - the **pure core** — [`event`] (the event model, whose variants are the
//!   deny-list), [`routing`], [`render`], [`flags`] and [`publish`] — is
//!   unit-tested against `mockall` doubles of the [`ports`];
//! - the **I/O edge** — [`webhook`] and [`pg`] — is thin and excluded from the
//!   coverage gate.
//!
//! Release announcements are not here: CI posts them (`scripts/discord/`).

pub mod digest;
pub mod event;
pub mod flags;
pub mod pg;
pub mod ports;
pub mod publish;
pub mod render;
pub mod reports;
pub mod routing;
pub mod webhook;

pub use event::{AnnouncementEvent, Category, Product, RecordMode, beats_record};
pub use ports::{
    AnnouncementLedger, AnnouncementSource, Claim, DiscordSender, FlagView, SendError,
};
pub use publish::{CATALOG_GROUPING_DELAY, Outcome, Publisher, Retry, Skip};
pub use render::{CatalogItem, Locale, Message};
pub use routing::{Routing, webhooks_from_env};

use cymbra_jobs::EnqueueRequest;
use cymbra_jobs::registry::{self, DISCORD_NOTIFY};

/// The `discord_notify` job request carrying `event`. A catalog acceptance runs
/// after [`CATALOG_GROUPING_DELAY`], so the acceptances of one moderation session
/// are announced together; a season record runs at once.
pub fn notify_request(event: &AnnouncementEvent) -> anyhow::Result<EnqueueRequest> {
    let spec = registry::spec(DISCORD_NOTIFY)
        .ok_or_else(|| anyhow::anyhow!("{DISCORD_NOTIFY} is not registered"))?;
    let request = EnqueueRequest::for_job(&spec, event, None)?;
    Ok(match event.category() {
        Category::MusicCatalog => request.with_delay(CATALOG_GROUPING_DELAY),
        _ => request,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_request_carries_the_event_on_the_discord_channel() {
        let event = AnnouncementEvent::ScoreAccepted {
            catalog_score_id: "c1".into(),
        };
        let req = notify_request(&event).unwrap();
        assert_eq!(req.name, DISCORD_NOTIFY);
        assert_eq!(req.channel_name, "discord.notify");
        let back: AnnouncementEvent = serde_json::from_str(&req.payload_json).unwrap();
        assert_eq!(back, event);
        // Catalog acceptances wait for the grouping window; records do not.
        assert_eq!(req.delay, CATALOG_GROUPING_DELAY);
        let record = AnnouncementEvent::SeasonRecord {
            season_id: "s".into(),
            catalog_score_id: "c1".into(),
            mode: RecordMode::Tempo,
            subscore: 90.0,
            achieved_on: chrono::NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        };
        assert_eq!(
            notify_request(&record).unwrap().delay,
            std::time::Duration::ZERO
        );
    }
}
