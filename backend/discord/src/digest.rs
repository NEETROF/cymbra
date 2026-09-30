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

//! One run of the scheduled `discord_digest` job (change:
//! add-discord-notifications, D7): every periodic report that is enabled,
//! routed, due today and has something to say is posted **once** into its
//! product's channel.
//!
//! Each report is decided independently — a failing or unconfigured source
//! skips that report only — and is claimed in the same ledger as the
//! announcements under `discord:report.<category>:<cadence>:<period start>`,
//! so a retried or re-delivered run never posts a period twice.

use chrono::NaiveDate;

use crate::event::Category;
use crate::flags::{DISCORD_ENABLED, REPORTS_MIN_CONTRIBUTORS, cadence_flag, category_flag};
use crate::ports::{
    AnnouncementLedger, Claim, DiscordSender, FlagView, LinguaSource, ReportSource,
};
use crate::publish::{CLAIM_STALE_AFTER_SECS, Outcome, Retry, Skip, deliver};
use crate::render::{Locale, Message};
use crate::reports::{
    Cadence, Period, SeasonInfo, WEEKLY_TOP, due_period, id_report, lingua_report, music_report,
    top_pieces_report,
};
use crate::routing::Routing;

/// Default of `discord.reports.min_contributors`: a figure behind fewer than three
/// accounts is dropped (maintainer decision, 2026-09-30).
pub const DEFAULT_MIN_CONTRIBUTORS: i64 = 3;

/// A report's default cadence (Music daily, ID and Lingua weekly) and whether
/// it is flag-driven. The weekly top pieces always run weekly.
pub fn default_cadence(category: Category) -> Cadence {
    match category {
        Category::MusicReport => Cadence::Daily,
        _ => Cadence::Weekly,
    }
}

/// Days a report waits after its period closes: Lingua figures are computed
/// on devices and reach the server by sync, so its period is reported a day
/// late.
pub fn lag_days(category: Category) -> i64 {
    match category {
        Category::LinguaReport => 1,
        _ => 0,
    }
}

/// The ledger key of one report period.
pub fn report_key(category: Category, period: &Period) -> String {
    format!(
        "discord:report.{}:{}:{}",
        category.key(),
        period.cadence.as_str(),
        period.start
    )
}

/// The ports and settings one digest run needs.
pub struct Digest<'a> {
    pub routing: &'a Routing,
    pub locale: Locale,
    pub flags: &'a dyn FlagView,
    pub source: &'a dyn ReportSource,
    /// `None` when the Lingua pool is not configured: its report is skipped.
    pub lingua: Option<&'a dyn LinguaSource>,
    pub ledger: &'a dyn AnnouncementLedger,
    pub sender: &'a dyn DiscordSender,
    /// The current leaderboard season, for the weekly top pieces.
    pub season: Option<SeasonInfo>,
    /// The UTC day the run happens on.
    pub today: NaiveDate,
}

impl Digest<'_> {
    /// Every report, in order, with what happened to it. An `Err` asks for the
    /// job to be retried; reports already posted stop at their claim.
    pub async fn run(&self) -> Vec<(Category, Result<Outcome, Retry>)> {
        let mut out = Vec::new();
        for category in Category::REPORTS {
            out.push((category, self.report(category).await));
        }
        out
    }

    async fn report(&self, category: Category) -> Result<Outcome, Retry> {
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
        let default = default_cadence(category);
        let cadence = match cadence_flag(category) {
            Some(key) => Cadence::parse(&self.flags.string(&key, default.as_str()), default),
            None => default,
        };
        let Some(period) = due_period(cadence, self.today, lag_days(category)) else {
            return Ok(Outcome::Skipped(Skip::NotDue));
        };
        let k = self
            .flags
            .int(REPORTS_MIN_CONTRIBUTORS, DEFAULT_MIN_CONTRIBUTORS)
            .max(1);
        let message = match self.render(category, &period, k).await? {
            Ok(message) => message,
            Err(skip) => return Ok(Outcome::Skipped(skip)),
        };
        let key = report_key(category, &period);
        let claim = self
            .ledger
            .claim(&key, CLAIM_STALE_AFTER_SECS)
            .await
            .map_err(|e| Retry(format!("claim {key}: {e}")))?;
        match claim {
            Claim::Settled => Ok(Outcome::AlreadySettled),
            Claim::InFlight => Err(Retry(format!("{key} is being posted by another run"))),
            Claim::Claimed => deliver(self.ledger, self.sender, channel, &message, &[key]).await,
        }
    }

    /// Load the figures and render them; the inner `Err` is a skip.
    async fn render(
        &self,
        category: Category,
        period: &Period,
        k: i64,
    ) -> Result<Result<Message, Skip>, Retry> {
        let load = |e: anyhow::Error| Retry(format!("load {} figures: {e}", category.key()));
        let locale = self.locale;
        let message = match category {
            Category::MusicReport => {
                let f = self.source.music(period).await.map_err(load)?;
                music_report(&f, period, k, locale)
            }
            Category::MusicTopPieces => {
                let top = self
                    .source
                    .top_pieces(period, WEEKLY_TOP)
                    .await
                    .map_err(load)?;
                top_pieces_report(&top, self.season, period, k, locale)
            }
            Category::IdReport => {
                let f = self.source.id(period).await.map_err(load)?;
                id_report(&f, period, k, locale)
            }
            Category::LinguaReport => {
                let Some(lingua) = self.lingua else {
                    return Ok(Err(Skip::NoSource));
                };
                let f = lingua.lingua(period).await.map_err(load)?;
                lingua_report(&f, period, k, locale)
            }
            // Announcements are not reports.
            Category::MusicCatalog | Category::MusicRecords => None,
        };
        Ok(message.ok_or(Skip::NothingToSay))
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::ports::{
        MockAnnouncementLedger, MockDiscordSender, MockFlagView, MockLinguaSource, MockReportSource,
    };
    use crate::render::ScoreCard;
    use crate::reports::{Figure, IdFigures, LinguaFigures, MusicFigures, PieceStat};

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    /// Every flag on; cadences and the minimum from `strings` / `k`.
    fn flags(k: i64, music_cadence: &'static str) -> MockFlagView {
        let mut f = MockFlagView::new();
        f.expect_enabled().return_const(true);
        f.expect_int().returning(move |_, _| k);
        f.expect_string().returning(move |key, default| {
            if key == "discord.music.report.cadence" {
                music_cadence.to_string()
            } else {
                default.to_string()
            }
        });
        f
    }

    fn piece() -> PieceStat {
        PieceStat {
            card: ScoreCard {
                title: Some("Air".into()),
                composer: None,
            },
            plays: 3,
            players: 1,
        }
    }

    fn source() -> MockReportSource {
        let mut s = MockReportSource::new();
        s.expect_music().returning(|_| {
            Ok(MusicFigures {
                sessions: Figure::new(5, 1),
                ..Default::default()
            })
        });
        s.expect_top_pieces().returning(|_, _| Ok(vec![piece()]));
        s.expect_id().returning(|_| {
            Ok(IdFigures {
                new_accounts: 1,
                methods: vec![("google".into(), 1)],
                locales: vec![],
            })
        });
        s
    }

    fn lingua() -> MockLinguaSource {
        let mut l = MockLinguaSource::new();
        l.expect_lingua().returning(|_| {
            Ok(LinguaFigures {
                words_read: Figure::new(900, 1),
                ..Default::default()
            })
        });
        l
    }

    fn ledger_free() -> MockAnnouncementLedger {
        let mut l = MockAnnouncementLedger::new();
        l.expect_claim().returning(|_, _| Ok(Claim::Claimed));
        l.expect_mark_posted().returning(|_| Ok(()));
        l
    }

    fn sender_ok() -> MockDiscordSender {
        let mut s = MockDiscordSender::new();
        s.expect_serves().return_const(true);
        s.expect_publish().returning(|_, _| Ok(()));
        s
    }

    fn outcome(run: &[(Category, Result<Outcome, Retry>)], c: Category) -> String {
        match &run.iter().find(|(cat, _)| *cat == c).unwrap().1 {
            Ok(o) => format!("{o:?}"),
            Err(e) => format!("Err({e})"),
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn run(
        flags: &MockFlagView,
        source: &MockReportSource,
        lingua: Option<&MockLinguaSource>,
        ledger: &MockAnnouncementLedger,
        sender: &MockDiscordSender,
        today: NaiveDate,
    ) -> Vec<(Category, Result<Outcome, Retry>)> {
        let routing = Routing::default();
        Digest {
            routing: &routing,
            locale: Locale::En,
            flags,
            source,
            lingua: lingua.map(|l| l as &dyn LinguaSource),
            ledger,
            sender,
            season: None,
            today,
        }
        .run()
        .await
    }

    #[test]
    fn cadences_lags_and_keys() {
        assert_eq!(default_cadence(Category::MusicReport), Cadence::Daily);
        assert_eq!(default_cadence(Category::IdReport), Cadence::Weekly);
        assert_eq!(default_cadence(Category::MusicTopPieces), Cadence::Weekly);
        assert_eq!(lag_days(Category::LinguaReport), 1);
        assert_eq!(lag_days(Category::MusicReport), 0);
        let p = due_period(Cadence::Weekly, d(2026, 9, 28), 0).unwrap();
        assert_eq!(
            report_key(Category::IdReport, &p),
            "discord:report.id.report:weekly:2026-09-21"
        );
    }

    #[tokio::test]
    async fn a_monday_posts_music_daily_and_the_weekly_reports_once_each() {
        let mut sender = MockDiscordSender::new();
        sender.expect_serves().return_const(true);
        for channel in ["music-stats", "music-leaderboards", "id-stats"] {
            sender
                .expect_publish()
                .withf(move |c, m| c == channel && m.embed.is_some())
                .times(1)
                .returning(|_, _| Ok(()));
        }
        let mut ledger = MockAnnouncementLedger::new();
        for key in [
            "discord:report.music.report:daily:2026-09-27",
            "discord:report.music.top_pieces:weekly:2026-09-21",
            "discord:report.id.report:weekly:2026-09-21",
        ] {
            ledger
                .expect_claim()
                .with(eq(key), eq(CLAIM_STALE_AFTER_SECS))
                .times(1)
                .returning(|_, _| Ok(Claim::Claimed));
        }
        ledger.expect_mark_posted().times(3).returning(|_| Ok(()));
        let out = run(
            &flags(1, "daily"),
            &source(),
            Some(&lingua()),
            &ledger,
            &sender,
            d(2026, 9, 28),
        )
        .await;
        // Lingua's week is due on Tuesday, not Monday.
        assert_eq!(outcome(&out, Category::LinguaReport), "Skipped(NotDue)");
        assert!(outcome(&out, Category::MusicReport).starts_with("Posted"));
    }

    #[tokio::test]
    async fn a_tuesday_posts_the_lingua_week_and_music_daily() {
        let out = run(
            &flags(1, "daily"),
            &source(),
            Some(&lingua()),
            &ledger_free(),
            &sender_ok(),
            d(2026, 9, 29),
        )
        .await;
        assert!(outcome(&out, Category::LinguaReport).starts_with("Posted"));
        assert!(outcome(&out, Category::MusicReport).starts_with("Posted"));
        assert_eq!(outcome(&out, Category::IdReport), "Skipped(NotDue)");
        assert_eq!(outcome(&out, Category::MusicTopPieces), "Skipped(NotDue)");
    }

    #[tokio::test]
    async fn no_lingua_source_skips_lingua_only() {
        let out = run(
            &flags(1, "daily"),
            &source(),
            None,
            &ledger_free(),
            &sender_ok(),
            d(2026, 9, 29),
        )
        .await;
        assert_eq!(outcome(&out, Category::LinguaReport), "Skipped(NoSource)");
        assert!(outcome(&out, Category::MusicReport).starts_with("Posted"));
    }

    #[tokio::test]
    async fn a_weekly_music_cadence_waits_for_monday() {
        let out = run(
            &flags(1, "weekly"),
            &source(),
            None,
            &ledger_free(),
            &sender_ok(),
            d(2026, 9, 29),
        )
        .await;
        assert_eq!(outcome(&out, Category::MusicReport), "Skipped(NotDue)");
    }

    #[tokio::test]
    async fn a_raised_minimum_leaves_nothing_to_say() {
        let mut sender = MockDiscordSender::new();
        sender.expect_serves().return_const(true);
        sender.expect_publish().never();
        let out = run(
            &flags(5, "daily"),
            &source(),
            Some(&lingua()),
            &MockAnnouncementLedger::new(),
            &sender,
            d(2026, 9, 28),
        )
        .await;
        for c in [
            Category::MusicReport,
            Category::MusicTopPieces,
            Category::IdReport,
        ] {
            assert_eq!(outcome(&out, c), "Skipped(NothingToSay)", "{c:?}");
        }
    }

    #[tokio::test]
    async fn flags_and_channels_gate_every_report() {
        let mut off = MockFlagView::new();
        off.expect_enabled()
            .with(eq(DISCORD_ENABLED))
            .return_const(false);
        let out = run(
            &off,
            &MockReportSource::new(),
            None,
            &MockAnnouncementLedger::new(),
            &MockDiscordSender::new(),
            d(2026, 9, 28),
        )
        .await;
        assert!(
            out.iter()
                .all(|(_, o)| matches!(o, Ok(Outcome::Skipped(Skip::Disabled))))
        );

        let mut reports_off = MockFlagView::new();
        reports_off
            .expect_enabled()
            .returning(|k| k == DISCORD_ENABLED || k == "discord.id.report");
        reports_off.expect_string().returning(|_, d| d.to_string());
        reports_off.expect_int().returning(|_, d| d);
        let mut sender = MockDiscordSender::new();
        sender
            .expect_serves()
            .with(eq("id-stats"))
            .return_const(false);
        let out = run(
            &reports_off,
            &MockReportSource::new(),
            None,
            &MockAnnouncementLedger::new(),
            &sender,
            d(2026, 9, 28),
        )
        .await;
        assert_eq!(
            outcome(&out, Category::MusicReport),
            "Skipped(CategoryDisabled)"
        );
        assert_eq!(outcome(&out, Category::IdReport), "Skipped(NoChannel)");
    }

    #[tokio::test]
    async fn a_period_already_posted_or_in_flight_is_not_posted_again() {
        let mut ledger = MockAnnouncementLedger::new();
        ledger.expect_claim().returning(|key, _| {
            Ok(if key.contains("music.report") {
                Claim::Settled
            } else {
                Claim::InFlight
            })
        });
        let mut sender = MockDiscordSender::new();
        sender.expect_serves().return_const(true);
        sender.expect_publish().never();
        let out = run(
            &flags(1, "daily"),
            &source(),
            None,
            &ledger,
            &sender,
            d(2026, 9, 28),
        )
        .await;
        assert_eq!(outcome(&out, Category::MusicReport), "AlreadySettled");
        assert!(outcome(&out, Category::IdReport).starts_with("Err("));
    }

    #[tokio::test]
    async fn a_failing_source_or_ledger_retries_that_report_only() {
        let mut source = MockReportSource::new();
        source
            .expect_music()
            .returning(|_| Err(anyhow::anyhow!("db down")));
        source
            .expect_top_pieces()
            .returning(|_, _| Ok(vec![piece()]));
        source.expect_id().returning(|_| Ok(IdFigures::default()));
        let mut lingua = MockLinguaSource::new();
        lingua
            .expect_lingua()
            .returning(|_| Err(anyhow::anyhow!("lingua down")));
        let mut ledger = MockAnnouncementLedger::new();
        ledger
            .expect_claim()
            .returning(|_, _| Err(anyhow::anyhow!("pool closed")));
        let out = run(
            &flags(1, "daily"),
            &source,
            Some(&lingua),
            &ledger,
            &sender_ok(),
            d(2026, 9, 29),
        )
        .await;
        assert!(outcome(&out, Category::MusicReport).contains("db down"));
        assert!(outcome(&out, Category::LinguaReport).contains("lingua down"));
        // The top pieces are not due on a Tuesday; the ID week has nothing.
        assert_eq!(outcome(&out, Category::MusicTopPieces), "Skipped(NotDue)");

        // On a Monday, the top pieces reach the ledger, which fails: retry.
        let out = run(
            &flags(1, "daily"),
            &source,
            None,
            &ledger,
            &sender_ok(),
            d(2026, 9, 28),
        )
        .await;
        assert!(outcome(&out, Category::MusicTopPieces).contains("pool closed"));
        assert_eq!(outcome(&out, Category::IdReport), "Skipped(NothingToSay)");
    }
}
