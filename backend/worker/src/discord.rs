//! Discord announcements and reports in the worker (change:
//! add-discord-notifications) — composition glue, excluded from the coverage
//! gate: the decisions live in `cymbra_discord::{Publisher, Digest}`, unit-tested
//! there. This file only binds their ports to the worker's pools and flag service.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use cymbra_discord::digest::Digest;
use cymbra_discord::pg::{PgAnnouncementLedger, PgAnnouncementSource, PgReportSource};
use cymbra_discord::ports::LinguaSource;
use cymbra_discord::reports::{Figure, LinguaFigures, Period, SeasonInfo};
use cymbra_discord::webhook::WebhookDiscordSender;
use cymbra_discord::{
    AnnouncementEvent, Category, FlagView, Locale, Outcome, Publisher, Retry, Routing,
};
use cymbra_feature_flags::{EvalContext, FlagService};
use cymbra_lingua::{LinguaAdminRepo, PgLinguaAdminRepo};
use sqlx::PgPool;

/// Everything `discord_notify` and `discord_digest` need. Built only when at
/// least one webhook is configured; otherwise both jobs are logged no-ops.
pub struct Announcer {
    routing: Routing,
    locale: Locale,
    source: PgAnnouncementSource,
    reports: PgReportSource,
    lingua: Option<LinguaUsage>,
    ledger: PgAnnouncementLedger,
    sender: WebhookDiscordSender,
}

impl Announcer {
    /// `queue_pool` is `worker_svc` (owner of the `jobs.discord_announcements`
    /// ledger); `admin_pool` is `admin_svc`, the worker's cross-schema reader of
    /// the catalog and account figures; `lingua_pool`, when configured, is
    /// `lingua_svc` for the Lingua report.
    pub fn new(
        sender: WebhookDiscordSender,
        locale: Locale,
        queue_pool: PgPool,
        admin_pool: PgPool,
        lingua_pool: Option<PgPool>,
    ) -> Self {
        Self {
            routing: Routing::default(),
            locale,
            source: PgAnnouncementSource::new(admin_pool.clone()),
            reports: PgReportSource::new(admin_pool),
            lingua: lingua_pool.map(|p| LinguaUsage(PgLinguaAdminRepo::new(p))),
            ledger: PgAnnouncementLedger::new(queue_pool),
            sender,
        }
    }

    pub async fn publish(
        &self,
        flags: &FlagService,
        event: &AnnouncementEvent,
    ) -> Result<Outcome, Retry> {
        Publisher {
            routing: &self.routing,
            locale: self.locale,
            flags: &Flags(flags),
            source: &self.source,
            ledger: &self.ledger,
            sender: &self.sender,
        }
        .publish(event)
        .await
    }

    /// Run every report due at `now`.
    pub async fn digest(
        &self,
        flags: &FlagService,
        now: DateTime<Utc>,
    ) -> Vec<(Category, Result<Outcome, Retry>)> {
        Digest {
            routing: &self.routing,
            locale: self.locale,
            flags: &Flags(flags),
            source: &self.reports,
            lingua: self.lingua.as_ref().map(|l| l as &dyn LinguaSource),
            ledger: &self.ledger,
            sender: &self.sender,
            season: Some(season(now)),
            today: now.date_naive(),
        }
        .run()
        .await
    }
}

/// The current global-leaderboard season (the music module's own config).
fn season(now: DateTime<Utc>) -> SeasonInfo {
    let now_ms = now.timestamp_millis();
    let s = cymbra_music::GlobalConfig::default().season_at(now_ms);
    let last_day = DateTime::from_timestamp_millis(s.end_ms - 1)
        .map(|d| d.date_naive())
        .unwrap_or_else(|| now.date_naive());
    SeasonInfo {
        last_day,
        days_left: ((s.end_ms - now_ms).max(0) + 86_399_999) / 86_400_000,
    }
}

/// UTC epoch day, the unit of `lingua.daily_stats.day`.
fn epoch_day(d: NaiveDate) -> i32 {
    (d - NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()).num_days() as i32
}

/// The Lingua ops aggregate seen through the report's port.
struct LinguaUsage(PgLinguaAdminRepo);

#[async_trait]
impl LinguaSource for LinguaUsage {
    async fn lingua(&self, period: &Period) -> anyhow::Result<LinguaFigures> {
        let u = self
            .0
            .usage(epoch_day(period.start), epoch_day(period.last_day()))
            .await
            .map_err(|e| anyhow::anyhow!("lingua usage: {e}"))?;
        Ok(LinguaFigures {
            words_read: Figure::new(u.words_read, u.readers),
            words_learned: Figure::new(u.words_learned, u.learners),
            reviews: Figure::new(u.reviews, u.reviewers),
            languages: u
                .by_language
                .into_iter()
                .map(|l| (l.language, l.active_accounts))
                .collect(),
        })
    }
}

/// The flag service seen through the publisher's port. Server-side keys are
/// declared under `APP_ALL`, evaluated without a user. `FlagService` applies
/// the **caller's** default when no override reaches the key, so the defaults
/// passed by the core must equal the registry's (pinned below).
struct Flags<'a>(&'a FlagService);

impl FlagView for Flags<'_> {
    fn enabled(&self, key: &str) -> bool {
        self.0.bool(key, false, &EvalContext::anonymous(""))
    }

    fn int(&self, key: &str, default: i64) -> i64 {
        self.0.int(key, default, &EvalContext::anonymous(""))
    }

    fn string(&self, key: &str, default: &str) -> String {
        self.0.string(key, default, &EvalContext::anonymous(""))
    }
}

#[cfg(test)]
mod tests {
    use cymbra_discord::Category;
    use cymbra_discord::digest::{DEFAULT_MIN_CONTRIBUTORS, default_cadence};
    use cymbra_discord::flags::{
        DISCORD_ENABLED, REPORTS_MIN_CONTRIBUTORS, cadence_flag, category_flag,
    };
    use cymbra_feature_flags::{FlagValue, Registry};

    use super::*;

    #[test]
    fn every_key_the_publisher_reads_is_registered_and_off() {
        let registry = Registry::default();
        let keys = std::iter::once(DISCORD_ENABLED.to_string())
            .chain(Category::ALL.into_iter().map(category_flag));
        for key in keys {
            let def = registry
                .get_by_key(&key)
                .unwrap_or_else(|| panic!("{key} is not registered"));
            assert_eq!(def.default, FlagValue::Bool(false), "{key}");
        }
    }

    #[test]
    fn report_defaults_in_code_match_the_registry() {
        let registry = Registry::default();
        for category in Category::REPORTS {
            if let Some(key) = cadence_flag(category) {
                let def = registry
                    .get_by_key(&key)
                    .unwrap_or_else(|| panic!("{key} is not registered"));
                assert_eq!(
                    def.default,
                    FlagValue::String(default_cadence(category).as_str().into()),
                    "{key}"
                );
            }
        }
        assert_eq!(
            registry
                .get_by_key(REPORTS_MIN_CONTRIBUTORS)
                .unwrap()
                .default,
            FlagValue::Int(DEFAULT_MIN_CONTRIBUTORS)
        );
    }

    #[test]
    fn lingua_days_are_utc_epoch_days() {
        assert_eq!(epoch_day(NaiveDate::from_ymd_opt(1970, 1, 2).unwrap()), 1);
        assert_eq!(
            epoch_day(NaiveDate::from_ymd_opt(2026, 9, 28).unwrap()),
            20_724
        );
    }

    #[test]
    fn the_season_reports_its_last_day_and_days_left() {
        // Seasons are 30 days from 2026-01-01: the one holding 2026-09-29 runs
        // 2026-09-28 .. 2026-10-27.
        let now = NaiveDate::from_ymd_opt(2026, 9, 29)
            .unwrap()
            .and_hms_opt(0, 30, 0)
            .unwrap()
            .and_utc();
        let s = season(now);
        assert_eq!(s.last_day, NaiveDate::from_ymd_opt(2026, 10, 27).unwrap());
        assert_eq!(s.days_left, 29);
    }
}
