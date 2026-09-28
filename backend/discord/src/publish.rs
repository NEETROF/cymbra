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

//! The publication sequence of one announcement (change: add-discord-notifications,
//! design D4, D5, D8).
//!
//! Everything that decides **whether** and **what** to post runs here, against
//! the ports, in this order: kill-switch → category flag → channel configured →
//! subject still public → something to say → claim the dedup key → post →
//! settle. Every check before the claim is re-run on each attempt, so a flag
//! turned off after the enqueue silences the announcement.
//!
//! `Err` is returned **only** when a later attempt can succeed — that is what
//! makes the job engine retry. A terminal failure is recorded and returns `Ok`.

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
    /// The subject has nothing to show (no title).
    NothingToSay,
}

/// What an attempt did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Posted {
        channel: &'static str,
    },
    /// An earlier attempt already settled this announcement.
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
        let message = match self.render(event).await? {
            Ok(message) => message,
            Err(skip) => return Ok(Outcome::Skipped(skip)),
        };

        let key = event.dedup_key();
        let claim = self
            .ledger
            .claim(&key, CLAIM_STALE_AFTER_SECS)
            .await
            .map_err(|e| Retry(format!("claim {key}: {e}")))?;
        match claim {
            Claim::Settled => return Ok(Outcome::AlreadySettled),
            Claim::InFlight => {
                return Err(Retry(format!("{key} is being posted by another attempt")));
            }
            Claim::Claimed => {}
        }

        match self.sender.publish(channel, &message).await {
            Ok(()) => {
                // Posted: a failure to record it only risks a repost after the
                // stale window if this job ran again — it will not, it succeeds.
                if let Err(e) = self.ledger.mark_posted(&key).await {
                    tracing::warn!(key, error = %e, "discord announcement posted but not recorded");
                }
                Ok(Outcome::Posted { channel })
            }
            Err(SendError::Retryable(why)) => {
                if let Err(e) = self.ledger.release(&key).await {
                    // The claim goes stale and is re-claimed after the window.
                    tracing::warn!(key, error = %e, "discord claim not released");
                }
                Err(Retry(why))
            }
            Err(SendError::Ambiguous(why) | SendError::Terminal(why)) => {
                tracing::error!(key, channel, reason = %why, "discord announcement failed for good");
                if let Err(e) = self.ledger.mark_failed(&key, &why).await {
                    tracing::warn!(key, error = %e, "discord failure not recorded");
                }
                Ok(Outcome::Failed(why))
            }
        }
    }

    /// Load the subject and render it; the inner `Err` is a skip.
    async fn render(&self, event: &AnnouncementEvent) -> Result<Result<Message, Skip>, Retry> {
        let locale = self.locale;
        let load = |e: anyhow::Error| Retry(format!("load announcement subject: {e}"));
        let rendered = match event {
            AnnouncementEvent::SoundFontAccepted { soundfont_id } => {
                let Some(card) = self
                    .source
                    .accepted_soundfont(soundfont_id)
                    .await
                    .map_err(load)?
                else {
                    return Ok(Err(Skip::NotPublic));
                };
                render::soundfont_accepted(&card, locale)
            }
            AnnouncementEvent::ScoreAccepted { catalog_score_id } => {
                let Some(card) = self
                    .source
                    .accepted_score(catalog_score_id)
                    .await
                    .map_err(load)?
                else {
                    return Ok(Err(Skip::NotPublic));
                };
                render::score_accepted(&card, locale)
            }
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
                    return Ok(Err(Skip::NotPublic));
                };
                render::season_record(&card, *mode, *subscore, locale)
            }
        };
        Ok(rendered.ok_or(Skip::NothingToSay))
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
    use crate::render::{ScoreCard, SoundFontCard};

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

    fn source_with_score() -> MockAnnouncementSource {
        let mut s = MockAnnouncementSource::new();
        s.expect_accepted_score().returning(|_| {
            Ok(Some(ScoreCard {
                title: Some("Clair de lune".into()),
                composer: Some("Debussy".into()),
            }))
        });
        s.expect_accepted_soundfont().returning(|_| {
            Ok(Some(SoundFontCard {
                label: "Grand".into(),
                instrument: "keyboard".into(),
                license: "CC0".into(),
                attribution: None,
            }))
        });
        s
    }

    fn claiming(claim: Claim) -> MockAnnouncementLedger {
        let mut l = MockAnnouncementLedger::new();
        l.expect_claim()
            .with(
                eq("discord:music.score_accepted:c1"),
                eq(CLAIM_STALE_AFTER_SECS),
            )
            .returning(move |_, _| Ok(claim));
        l
    }

    fn score_event() -> AnnouncementEvent {
        AnnouncementEvent::ScoreAccepted {
            catalog_score_id: "c1".into(),
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
    async fn posts_once_and_records_it() {
        let mut ledger = claiming(Claim::Claimed);
        ledger.expect_mark_posted().times(1).returning(|_| Ok(()));
        let mut sender = sender_serving_all();
        sender
            .expect_publish()
            .withf(|channel, msg| {
                channel == "scores-and-soundfonts" && msg.content.contains("Clair de lune")
            })
            .times(1)
            .returning(|_, _| Ok(()));
        let out = run(
            &flags_on(),
            &source_with_score(),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert_eq!(
            out.unwrap(),
            Outcome::Posted {
                channel: "scores-and-soundfonts"
            }
        );
    }

    #[tokio::test]
    async fn a_settled_key_is_not_posted_again() {
        let ledger = claiming(Claim::Settled);
        let mut sender = sender_serving_all();
        sender.expect_publish().never();
        let out = run(
            &flags_on(),
            &source_with_score(),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert_eq!(out.unwrap(), Outcome::AlreadySettled);
    }

    #[tokio::test]
    async fn an_in_flight_claim_is_retried_later_without_posting() {
        let ledger = claiming(Claim::InFlight);
        let mut sender = sender_serving_all();
        sender.expect_publish().never();
        let out = run(
            &flags_on(),
            &source_with_score(),
            &ledger,
            &sender,
            &score_event(),
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
        let mut ledger = MockAnnouncementLedger::new();
        ledger.expect_claim().never();
        let mut sender = MockDiscordSender::new();
        sender.expect_publish().never();
        let out = run(
            &flags,
            &MockAnnouncementSource::new(),
            &ledger,
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
        let record = AnnouncementEvent::SeasonRecord {
            season_id: "s".into(),
            catalog_score_id: "c1".into(),
            mode: RecordMode::Tempo,
            subscore: 90.0,
            achieved_on: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        };
        let mut sender = sender_serving_all();
        sender.expect_publish().never();
        let out = run(
            &flags,
            &MockAnnouncementSource::new(),
            &MockAnnouncementLedger::new(),
            &sender,
            &record,
        )
        .await;
        assert_eq!(out.unwrap(), Outcome::Skipped(Skip::CategoryDisabled));

        // The catalog category still posts under the same flags.
        let mut ledger = claiming(Claim::Claimed);
        ledger.expect_mark_posted().returning(|_| Ok(()));
        let mut sender = sender_serving_all();
        sender.expect_publish().times(1).returning(|_, _| Ok(()));
        let out = run(
            &flags,
            &source_with_score(),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert!(matches!(out.unwrap(), Outcome::Posted { .. }));
    }

    #[tokio::test]
    async fn an_unconfigured_channel_is_a_no_op() {
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
    }

    #[tokio::test]
    async fn an_unrouted_category_is_a_no_op() {
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
    async fn a_subject_no_longer_public_is_skipped() {
        let mut source = MockAnnouncementSource::new();
        source.expect_accepted_score().returning(|_| Ok(None));
        source.expect_accepted_soundfont().returning(|_| Ok(None));
        let mut ledger = MockAnnouncementLedger::new();
        ledger.expect_claim().never();
        let sender = sender_serving_all();
        for event in [
            score_event(),
            AnnouncementEvent::SoundFontAccepted {
                soundfont_id: "f".into(),
            },
            AnnouncementEvent::SeasonRecord {
                season_id: "s".into(),
                catalog_score_id: "c1".into(),
                mode: RecordMode::Reaction,
                subscore: 90.0,
                achieved_on: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
            },
        ] {
            let out = run(&flags_on(), &source, &ledger, &sender, &event).await;
            assert_eq!(out.unwrap(), Outcome::Skipped(Skip::NotPublic));
        }
    }

    #[tokio::test]
    async fn an_untitled_piece_has_nothing_to_say() {
        let mut source = MockAnnouncementSource::new();
        source.expect_accepted_score().returning(|_| {
            Ok(Some(ScoreCard {
                title: None,
                composer: None,
            }))
        });
        let mut ledger = MockAnnouncementLedger::new();
        ledger.expect_claim().never();
        let out = run(
            &flags_on(),
            &source,
            &ledger,
            &sender_serving_all(),
            &score_event(),
        )
        .await;
        assert_eq!(out.unwrap(), Outcome::Skipped(Skip::NothingToSay));
    }

    #[tokio::test]
    async fn a_source_error_is_retried() {
        let mut source = MockAnnouncementSource::new();
        source
            .expect_accepted_score()
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
    }

    #[tokio::test]
    async fn a_ledger_error_is_retried_without_posting() {
        let mut ledger = MockAnnouncementLedger::new();
        ledger
            .expect_claim()
            .returning(|_, _| Err(anyhow::anyhow!("pool closed")));
        let mut sender = sender_serving_all();
        sender.expect_publish().never();
        let out = run(
            &flags_on(),
            &source_with_score(),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert!(out.is_err());
    }

    #[tokio::test]
    async fn a_retryable_failure_releases_the_claim_and_retries() {
        let mut ledger = claiming(Claim::Claimed);
        ledger.expect_release().times(1).returning(|_| Ok(()));
        ledger.expect_mark_failed().never();
        let mut sender = sender_serving_all();
        sender
            .expect_publish()
            .returning(|_, _| Err(SendError::Retryable("discord answered 503".into())));
        let out = run(
            &flags_on(),
            &source_with_score(),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert_eq!(out.unwrap_err().0, "discord answered 503");
    }

    #[tokio::test]
    async fn a_release_failure_still_retries() {
        let mut ledger = claiming(Claim::Claimed);
        ledger
            .expect_release()
            .returning(|_| Err(anyhow::anyhow!("gone")));
        let mut sender = sender_serving_all();
        sender
            .expect_publish()
            .returning(|_, _| Err(SendError::Retryable("timeout before send".into())));
        let out = run(
            &flags_on(),
            &source_with_score(),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert!(out.is_err());
    }

    #[tokio::test]
    async fn terminal_and_ambiguous_failures_are_recorded_not_retried() {
        for err in [
            SendError::Terminal("discord answered 404".into()),
            SendError::Ambiguous("timed out after sending".into()),
        ] {
            let mut ledger = claiming(Claim::Claimed);
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
                &source_with_score(),
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
        let mut ledger = claiming(Claim::Claimed);
        ledger
            .expect_mark_posted()
            .returning(|_| Err(anyhow::anyhow!("lost")));
        let mut sender = sender_serving_all();
        sender.expect_publish().times(1).returning(|_, _| Ok(()));
        let out = run(
            &flags_on(),
            &source_with_score(),
            &ledger,
            &sender,
            &score_event(),
        )
        .await;
        assert!(matches!(out.unwrap(), Outcome::Posted { .. }));
    }

    #[tokio::test]
    async fn soundfonts_and_records_render_from_their_subject() {
        let mut ledger = MockAnnouncementLedger::new();
        ledger.expect_claim().returning(|_, _| Ok(Claim::Claimed));
        ledger.expect_mark_posted().returning(|_| Ok(()));
        let mut sender = sender_serving_all();
        sender
            .expect_publish()
            .withf(|c, m| c == "scores-and-soundfonts" && m.content.contains("Grand"))
            .times(1)
            .returning(|_, _| Ok(()));
        sender
            .expect_publish()
            .withf(|c, m| c == "music-leaderboards" && m.content.contains("Clair de lune"))
            .times(1)
            .returning(|_, _| Ok(()));
        let source = source_with_score();
        let sf = AnnouncementEvent::SoundFontAccepted {
            soundfont_id: "f".into(),
        };
        let record = AnnouncementEvent::SeasonRecord {
            season_id: "s".into(),
            catalog_score_id: "c1".into(),
            mode: RecordMode::Tempo,
            subscore: 99.0,
            achieved_on: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        };
        for e in [sf, record] {
            let out = run(&flags_on(), &source, &ledger, &sender, &e).await;
            assert!(matches!(out.unwrap(), Outcome::Posted { .. }));
        }
    }
}
