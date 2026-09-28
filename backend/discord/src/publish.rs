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

//! The publication sequence of one announcement job (change:
//! add-discord-notifications, design D4, D5, D8).
//!
//! Everything that decides **whether** and **what** to post runs here, against
//! the ports: kill-switch → category flag → channel configured → load → claim the
//! dedup keys → post → settle. Every check before the claim is re-run on each
//! attempt, so a flag turned off after the enqueue silences the announcement.
//!
//! **Catalog acceptances are grouped.** Their job is enqueued with a delay
//! ([`CATALOG_GROUPING_DELAY`]); when it runs it announces, in **one** message,
//! every item accepted recently and not announced yet. A moderator accepting 500
//! scores in a row yields one or two messages, and every later job of that burst
//! finds nothing left to say. A season record is announced on its own.
//!
//! `Err` is returned **only** when a later attempt can succeed — that is what
//! makes the job engine retry. A terminal failure is recorded and returns `Ok`.

use std::time::Duration;

use crate::event::AnnouncementEvent;
use crate::flags::{DISCORD_ENABLED, category_flag};
use crate::ports::{
    AnnouncementLedger, AnnouncementSource, Claim, DiscordSender, FlagView, SendError,
};
use crate::render::{self, Locale, Message};
use crate::routing::Routing;

/// How long an unsettled claim blocks other attempts before it is presumed
/// abandoned (a crash between claim and post). Longer than the webhook timeout,
/// shorter than the job's retry horizon.
pub const CLAIM_STALE_AFTER_SECS: i64 = 600;

/// How long a catalog acceptance waits before it is announced, so the
/// acceptances of one moderation session share a message.
pub const CATALOG_GROUPING_DELAY: Duration = Duration::from_secs(600);

/// How far back a grouped catalog message looks for items not yet announced.
/// Well beyond the grouping delay plus the job's whole retry horizon, so an item
/// whose job was retried for a while is still picked up.
pub const CATALOG_LOOKBACK_HOURS: i64 = 24;

/// Why nothing was posted, for the logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    /// `discord.enabled` is off.
    Disabled,
    /// The category's flag is off.
    CategoryDisabled,
    /// No route, or no webhook for the routed channel.
    NoChannel,
    /// The subject is no longer public (rejected again, deleted).
    NotPublic,
    /// Nothing to show: no title, or no catalog item left to announce.
    NothingToSay,
}

/// What an attempt did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// One message posted, carrying `announcements` announcements.
    Posted {
        channel: &'static str,
        announcements: usize,
    },
    /// Earlier attempts already settled everything this one would have said.
    AlreadySettled,
    Skipped(Skip),
    /// Failed for good; recorded in the ledger, not retried.
    Failed(String),
}

/// A failure a later attempt can fix; the job is retried.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Retry(pub String);

/// The ports and settings one publication needs.
pub struct Publisher<'a> {
    pub routing: &'a Routing,
    pub locale: Locale,
    pub flags: &'a dyn FlagView,
    pub source: &'a dyn AnnouncementSource,
    pub ledger: &'a dyn AnnouncementLedger,
    pub sender: &'a dyn DiscordSender,
}

fn load(e: anyhow::Error) -> Retry {
    Retry(format!("load announcement subject: {e}"))
}

impl Publisher<'_> {
    pub async fn publish(&self, event: &AnnouncementEvent) -> Result<Outcome, Retry> {
        let category = event.category();
        if !self.flags.enabled(DISCORD_ENABLED) {
            return Ok(Outcome::Skipped(Skip::Disabled));
        }
        if !self.flags.enabled(&category_flag(category)) {
            return Ok(Outcome::Skipped(Skip::CategoryDisabled));
        }
        let Some(channel) = self
            .routing
            .resolve(category)
            .filter(|c| self.sender.serves(c))
        else {
            return Ok(Outcome::Skipped(Skip::NoChannel));
        };
        match event {
            AnnouncementEvent::SoundFontAccepted { .. }
            | AnnouncementEvent::ScoreAccepted { .. } => self.publish_catalog(channel).await,
            AnnouncementEvent::SeasonRecord {
                catalog_score_id,
                mode,
                subscore,
                ..
            } => {
                let Some(card) = self
                    .source
                    .accepted_score(catalog_score_id)
                    .await
                    .map_err(load)?
                else {
                    return Ok(Outcome::Skipped(Skip::NotPublic));
                };
                let Some(message) = render::season_record(&card, *mode, *subscore, self.locale)
                else {
                    return Ok(Outcome::Skipped(Skip::NothingToSay));
                };
                let key = event.dedup_key();
                let claim = self
                    .ledger
                    .claim(&key, CLAIM_STALE_AFTER_SECS)
                    .await
                    .map_err(|e| Retry(format!("claim {key}: {e}")))?;
                match claim {
                    Claim::Settled => Ok(Outcome::AlreadySettled),
                    Claim::InFlight => {
                        Err(Retry(format!("{key} is being posted by another attempt")))
                    }
                    Claim::Claimed => self.post(channel, &message, &[key]).await,
                }
            }
        }
    }

    /// One message for every recently accepted item nobody announced yet. The
    /// event that triggered the job only says "something was accepted": the item
    /// itself is among the candidates, unless it was rejected again since.
    async fn publish_catalog(&self, channel: &'static str) -> Result<Outcome, Retry> {
        let candidates: Vec<_> = self
            .source
            .recently_accepted(CATALOG_LOOKBACK_HOURS)
            .await
            .map_err(load)?
            .into_iter()
            .filter(|item| item.is_displayable())
            .collect();
        if candidates.is_empty() {
            return Ok(Outcome::Skipped(Skip::NothingToSay));
        }
        let keys: Vec<String> = candidates.iter().map(|i| i.dedup_key()).collect();
        let won = self
            .ledger
            .claim_batch(&keys, CLAIM_STALE_AFTER_SECS)
            .await
            .map_err(|e| Retry(format!("claim catalog announcements: {e}")))?;
        if won.is_empty() {
            return Ok(Outcome::AlreadySettled);
        }
        let mine: Vec<_> = candidates
            .into_iter()
            .filter(|item| won.contains(&item.dedup_key()))
            .collect();
        let Some(message) = render::catalog_batch(&mine, self.locale) else {
            // Unreachable (every candidate is displayable), but never hold claims.
            let _ = self.ledger.release(&won).await;
            return Ok(Outcome::Skipped(Skip::NothingToSay));
        };
        self.post(channel, &message, &won).await
    }

    /// Post `message`, which carries the claimed `keys`, and settle them.
    async fn post(
        &self,
        channel: &'static str,
        message: &Message,
        keys: &[String],
    ) -> Result<Outcome, Retry> {
        match self.sender.publish(channel, message).await {
            Ok(()) => {
                // Posted: a failure to record it only risks a repost after the
                // stale window if this job ran again — it will not, it succeeds.
                if let Err(e) = self.ledger.mark_posted(keys).await {
                    tracing::warn!(?keys, error = %e, "discord announcement posted but not recorded");
                }
                Ok(Outcome::Posted {
                    channel,
                    announcements: keys.len(),
                })
            }
            Err(SendError::Retryable(why)) => {
                if let Err(e) = self.ledger.release(keys).await {
                    // The claims go stale and are re-claimed after the window.
                    tracing::warn!(?keys, error = %e, "discord claims not released");
                }
                Err(Retry(why))
            }
            Err(SendError::Ambiguous(why) | SendError::Terminal(why)) => {
                tracing::error!(?keys, channel, reason = %why, "discord announcement failed for good");
                if let Err(e) = self.ledger.mark_failed(keys, &why).await {
                    tracing::warn!(?keys, error = %e, "discord failure not recorded");
                }
                Ok(Outcome::Failed(why))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use mockall::predicate::eq;

    use super::*;
    use crate::event::{Category, RecordMode};
    use crate::ports::{
        MockAnnouncementLedger, MockAnnouncementSource, MockDiscordSender, MockFlagView,
    };
    use crate::render::{CatalogItem, ScoreCard, SoundFontCard};

    fn flags_on() -> MockFlagView {
        let mut f = MockFlagView::new();
        f.expect_enabled().return_const(true);
        f
    }

    fn sender_serving_all() -> MockDiscordSender {
        let mut s = MockDiscordSender::new();
        s.expect_serves().return_const(true);
        s
    }

    fn score(id: &str, title: &str) -> CatalogItem {
        CatalogItem::Score {
            id: id.into(),
            card: ScoreCard {
                title: Some(title.into()),
                composer: Some("Debussy".into()),
            },
        }
    }

    fn font(id: &str) -> CatalogItem {
        CatalogItem::SoundFont {
            id: id.into(),
            card: SoundFontCard {
                label: "Grand".into(),
                instrument: "keyboard".into(),
                license: "CC0".into(),
                attribution: None,
            },
        }
    }

    fn source_with(items: Vec<CatalogItem>) -> MockAnnouncementSource {
        let mut s = MockAnnouncementSource::new();
        s.expect_recently_accepted()
            .with(eq(CATALOG_LOOKBACK_HOURS))
            .returning(move |_| Ok(items.clone()));
        s.expect_accepted_score().returning(|_| {
            Ok(Some(ScoreCard {
                title: Some("Clair de lune".into()),
                composer: Some("Debussy".into()),
            }))
        });
        s
    }

    /// A ledger where every key is free.
    fn ledger_all_free() -> MockAnnouncementLedger {
        let mut l = MockAnnouncementLedger::new();
        l.expect_claim().returning(|_, _| Ok(Claim::Claimed));
        l.expect_claim_batch()
            .returning(|keys, _| Ok(keys.to_vec()));
        l
    }

    fn score_event() -> AnnouncementEvent {
        AnnouncementEvent::ScoreAccepted {
            catalog_score_id: "c1".into(),
        }
    }

    fn record_event() -> AnnouncementEvent {
        AnnouncementEvent::SeasonRecord {
            season_id: "s".into(),
            catalog_score_id: "c1".into(),
            mode: RecordMode::Tempo,
            subscore: 99.0,
            achieved_on: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        }
    }

    async fn run(
        flags: &MockFlagView,
        source: &MockAnnouncementSource,
        ledger: &MockAnnouncementLedger,
        sender: &MockDiscordSender,
        event: &AnnouncementEvent,
    ) -> Result<Outcome, Retry> {
        let routing = Routing::default();
        Publisher {
            routing: &routing,
            locale: Locale::En,
            flags,
            source,
            ledger,
            sender,
        }
        .publish(event)
        .await
    }

    #[tokio::test]
    async fn a_burst_of_acceptances_becomes_one_message() {
        let items: Vec<CatalogItem> = (0..500)
            .map(|i| score(&format!("c{i}"), &format!("Piece {i}")))
            .chain([font("f1")])
            .collect();
        let mut ledger = ledger_all_free();
        ledger
            .expect_mark_posted()
            .withf(|keys| keys.len() == 501)
            .times(1)
            .returning(|_| Ok(()));
        let mut sender = sender_serving_all();
        sender
            .expect_publish()
            .withf(|channel, msg| {
                channel == "scores-and-soundfonts"
                    && msg
                        .content
                        .starts_with("🎼 **500 new scores and 1 new sound")
                    && msg.content.contains("…and 491 more.")
            })
            .times(1)
            .returning(|_, _| Ok(()));
        let out = run(
            &flags_on(),
            &source_with(items),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert_eq!(
            out.unwrap(),
            Outcome::Posted {
                channel: "scores-and-soundfonts",
                announcements: 501
            }
        );
    }

    #[tokio::test]
    async fn only_the_items_not_yet_announced_are_posted() {
        let items = vec![
            score("c1", "Air"),
            score("c2", "Gavotte"),
            score("c3", "Menuet"),
        ];
        let mut ledger = MockAnnouncementLedger::new();
        // c1 was announced by an earlier job of the burst.
        ledger.expect_claim_batch().returning(|keys, _| {
            Ok(keys
                .iter()
                .filter(|k| !k.ends_with(":c1"))
                .cloned()
                .collect())
        });
        ledger
            .expect_mark_posted()
            .withf(|keys| keys.len() == 2)
            .times(1)
            .returning(|_| Ok(()));
        let mut sender = sender_serving_all();
        sender
            .expect_publish()
            .withf(|_, msg| {
                msg.content.starts_with("🎼 **2 new scores")
                    && !msg.content.contains("Air")
                    && msg.content.contains("Menuet")
            })
            .times(1)
            .returning(|_, _| Ok(()));
        let out = run(
            &flags_on(),
            &source_with(items),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert!(matches!(
            out.unwrap(),
            Outcome::Posted {
                announcements: 2,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn a_single_acceptance_gets_its_detailed_message() {
        let mut ledger = ledger_all_free();
        ledger.expect_mark_posted().times(1).returning(|_| Ok(()));
        let mut sender = sender_serving_all();
        sender
            .expect_publish()
            .withf(|_, msg| {
                msg.content
                    .starts_with("🎹 **New sound in the catalog: Grand**")
            })
            .times(1)
            .returning(|_, _| Ok(()));
        let event = AnnouncementEvent::SoundFontAccepted {
            soundfont_id: "f1".into(),
        };
        let out = run(
            &flags_on(),
            &source_with(vec![font("f1")]),
            &ledger,
            &sender,
            &event,
        )
        .await;
        assert!(matches!(
            out.unwrap(),
            Outcome::Posted {
                announcements: 1,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn the_later_jobs_of_a_burst_post_nothing() {
        let mut ledger = MockAnnouncementLedger::new();
        ledger.expect_claim_batch().returning(|_, _| Ok(vec![]));
        let mut sender = sender_serving_all();
        sender.expect_publish().never();
        let out = run(
            &flags_on(),
            &source_with(vec![score("c1", "Air")]),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert_eq!(out.unwrap(), Outcome::AlreadySettled);
    }

    #[tokio::test]
    async fn nothing_accepted_or_nothing_nameable_posts_nothing() {
        let untitled = CatalogItem::Score {
            id: "u".into(),
            card: ScoreCard {
                title: None,
                composer: None,
            },
        };
        for items in [vec![], vec![untitled]] {
            let mut ledger = MockAnnouncementLedger::new();
            ledger.expect_claim_batch().never();
            let mut sender = sender_serving_all();
            sender.expect_publish().never();
            let out = run(
                &flags_on(),
                &source_with(items),
                &ledger,
                &sender,
                &score_event(),
            )
            .await;
            assert_eq!(out.unwrap(), Outcome::Skipped(Skip::NothingToSay));
        }
    }

    #[tokio::test]
    async fn a_catalog_source_or_ledger_error_is_retried() {
        let mut source = MockAnnouncementSource::new();
        source
            .expect_recently_accepted()
            .returning(|_| Err(anyhow::anyhow!("db down")));
        let out = run(
            &flags_on(),
            &source,
            &MockAnnouncementLedger::new(),
            &sender_serving_all(),
            &score_event(),
        )
        .await;
        assert!(out.unwrap_err().0.contains("db down"));

        let mut ledger = MockAnnouncementLedger::new();
        ledger
            .expect_claim_batch()
            .returning(|_, _| Err(anyhow::anyhow!("pool closed")));
        let mut sender = sender_serving_all();
        sender.expect_publish().never();
        let out = run(
            &flags_on(),
            &source_with(vec![score("c1", "Air")]),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert!(out.is_err());
    }

    #[tokio::test]
    async fn a_record_is_posted_once_and_recorded() {
        let mut ledger = MockAnnouncementLedger::new();
        ledger
            .expect_claim()
            .with(
                eq("discord:music.season_record:s:c1:tempo:2026-09-01"),
                eq(CLAIM_STALE_AFTER_SECS),
            )
            .returning(|_, _| Ok(Claim::Claimed));
        ledger
            .expect_mark_posted()
            .withf(|keys| keys == ["discord:music.season_record:s:c1:tempo:2026-09-01"])
            .times(1)
            .returning(|_| Ok(()));
        let mut sender = sender_serving_all();
        sender
            .expect_publish()
            .withf(|c, m| c == "music-leaderboards" && m.content.contains("Clair de lune"))
            .times(1)
            .returning(|_, _| Ok(()));
        let out = run(
            &flags_on(),
            &source_with(vec![]),
            &ledger,
            &sender,
            &record_event(),
        )
        .await;
        assert_eq!(
            out.unwrap(),
            Outcome::Posted {
                channel: "music-leaderboards",
                announcements: 1
            }
        );
    }

    #[tokio::test]
    async fn a_settled_record_is_not_posted_again_and_an_in_flight_one_retries() {
        for (claim, retried) in [(Claim::Settled, false), (Claim::InFlight, true)] {
            let mut ledger = MockAnnouncementLedger::new();
            ledger.expect_claim().returning(move |_, _| Ok(claim));
            let mut sender = sender_serving_all();
            sender.expect_publish().never();
            let out = run(
                &flags_on(),
                &source_with(vec![]),
                &ledger,
                &sender,
                &record_event(),
            )
            .await;
            if retried {
                assert!(out.is_err());
            } else {
                assert_eq!(out.unwrap(), Outcome::AlreadySettled);
            }
        }
    }

    #[tokio::test]
    async fn a_record_on_a_piece_no_longer_public_or_untitled_is_skipped() {
        let mut gone = MockAnnouncementSource::new();
        gone.expect_accepted_score().returning(|_| Ok(None));
        let mut untitled = MockAnnouncementSource::new();
        untitled.expect_accepted_score().returning(|_| {
            Ok(Some(ScoreCard {
                title: None,
                composer: None,
            }))
        });
        let mut errored = MockAnnouncementSource::new();
        errored
            .expect_accepted_score()
            .returning(|_| Err(anyhow::anyhow!("db down")));
        let mut ledger = MockAnnouncementLedger::new();
        ledger.expect_claim().never();
        let sender = sender_serving_all();
        let out = run(&flags_on(), &gone, &ledger, &sender, &record_event()).await;
        assert_eq!(out.unwrap(), Outcome::Skipped(Skip::NotPublic));
        let out = run(&flags_on(), &untitled, &ledger, &sender, &record_event()).await;
        assert_eq!(out.unwrap(), Outcome::Skipped(Skip::NothingToSay));
        let out = run(&flags_on(), &errored, &ledger, &sender, &record_event()).await;
        assert!(out.is_err());
        let mut failing = MockAnnouncementLedger::new();
        failing
            .expect_claim()
            .returning(|_, _| Err(anyhow::anyhow!("pool closed")));
        let out = run(
            &flags_on(),
            &source_with(vec![]),
            &failing,
            &sender,
            &record_event(),
        )
        .await;
        assert!(out.is_err());
    }

    #[tokio::test]
    async fn kill_switch_off_after_enqueue_posts_nothing() {
        let mut flags = MockFlagView::new();
        flags
            .expect_enabled()
            .with(eq(DISCORD_ENABLED))
            .return_const(false);
        let mut sender = MockDiscordSender::new();
        sender.expect_publish().never();
        let out = run(
            &flags,
            &MockAnnouncementSource::new(),
            &MockAnnouncementLedger::new(),
            &sender,
            &score_event(),
        )
        .await;
        assert_eq!(out.unwrap(), Outcome::Skipped(Skip::Disabled));
    }

    #[tokio::test]
    async fn a_category_flag_off_silences_only_that_category() {
        let mut flags = MockFlagView::new();
        flags
            .expect_enabled()
            .returning(|k| k != "discord.music.records");
        let mut sender = sender_serving_all();
        sender.expect_publish().never();
        let out = run(
            &flags,
            &MockAnnouncementSource::new(),
            &MockAnnouncementLedger::new(),
            &sender,
            &record_event(),
        )
        .await;
        assert_eq!(out.unwrap(), Outcome::Skipped(Skip::CategoryDisabled));

        let mut ledger = ledger_all_free();
        ledger.expect_mark_posted().returning(|_| Ok(()));
        let mut sender = sender_serving_all();
        sender.expect_publish().times(1).returning(|_, _| Ok(()));
        let out = run(
            &flags,
            &source_with(vec![score("c1", "Air")]),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert!(matches!(out.unwrap(), Outcome::Posted { .. }));
    }

    #[tokio::test]
    async fn an_unconfigured_or_unrouted_channel_is_a_no_op() {
        let mut sender = MockDiscordSender::new();
        sender
            .expect_serves()
            .with(eq("scores-and-soundfonts"))
            .return_const(false);
        sender.expect_publish().never();
        let out = run(
            &flags_on(),
            &MockAnnouncementSource::new(),
            &MockAnnouncementLedger::new(),
            &sender,
            &score_event(),
        )
        .await;
        assert_eq!(out.unwrap(), Outcome::Skipped(Skip::NoChannel));

        let routing = Routing::from_routes([]);
        let mut sender = sender_serving_all();
        sender.expect_publish().never();
        let out = Publisher {
            routing: &routing,
            locale: Locale::En,
            flags: &flags_on(),
            source: &MockAnnouncementSource::new(),
            ledger: &MockAnnouncementLedger::new(),
            sender: &sender,
        }
        .publish(&score_event())
        .await;
        assert_eq!(out.unwrap(), Outcome::Skipped(Skip::NoChannel));
        assert_eq!(Category::ALL.len(), 2);
    }

    #[tokio::test]
    async fn a_retryable_failure_releases_every_claim_and_retries() {
        let mut ledger = ledger_all_free();
        ledger
            .expect_release()
            .withf(|keys| keys.len() == 2)
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("still retried")));
        ledger.expect_mark_failed().never();
        let mut sender = sender_serving_all();
        sender
            .expect_publish()
            .returning(|_, _| Err(SendError::Retryable("discord answered 503".into())));
        let items = vec![score("c1", "Air"), score("c2", "Aria")];
        let out = run(
            &flags_on(),
            &source_with(items),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert_eq!(out.unwrap_err().0, "discord answered 503");
    }

    #[tokio::test]
    async fn terminal_and_ambiguous_failures_are_recorded_not_retried() {
        for err in [
            SendError::Terminal("discord answered 404".into()),
            SendError::Ambiguous("timed out after sending".into()),
        ] {
            let mut ledger = ledger_all_free();
            ledger.expect_release().never();
            ledger
                .expect_mark_failed()
                .times(1)
                .returning(|_, _| Err(anyhow::anyhow!("not recorded, still Ok")));
            let mut sender = sender_serving_all();
            let e = err.clone();
            sender
                .expect_publish()
                .returning(move |_, _| Err(e.clone()));
            let out = run(
                &flags_on(),
                &source_with(vec![score("c1", "Air")]),
                &ledger,
                &sender,
                &score_event(),
            )
            .await;
            assert!(matches!(out.unwrap(), Outcome::Failed(_)));
        }
    }

    #[tokio::test]
    async fn a_post_whose_record_fails_still_succeeds() {
        let mut ledger = ledger_all_free();
        ledger
            .expect_mark_posted()
            .returning(|_| Err(anyhow::anyhow!("lost")));
        let mut sender = sender_serving_all();
        sender.expect_publish().times(1).returning(|_, _| Ok(()));
        let out = run(
            &flags_on(),
            &source_with(vec![]),
            &ledger,
            &sender,
            &record_event(),
        )
        .await;
        assert!(matches!(out.unwrap(), Outcome::Posted { .. }));
    }
}
