// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The Lingua ops-console module: aggregates over the `lingua` schema (tiles, a
//! per-day series) for the back office. Aggregates only — the repository returns counts
//! grouped by day/language and never a row attributable to an account (the privacy
//! allow-list). The window parse + series formatting live in [`admin_core`]. Host-tested
//! against a fake repo; the Postgres adapter is in `pg_admin`.

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_platform::Result;

use crate::admin_core::{
    FunnelRow, FunnelWeek, LeaveTotals, OnlineUsageRow, OnlineUsageTotals, ReasonRow, SeriesMetric,
    SeriesPoint, Usage, browser_filter, funnel_weeks, leave_totals, online_usage_totals,
    parse_window, series_points,
};
use crate::usage_counts::{DEFAULT_CEILING, UsageCountsPolicy};
use crate::usage_counts_core::monday_of;

/// Storage port for the ops aggregates (consumer-declared). Every method returns
/// counts only — the SQL groups by day and studied language, never by account.
#[async_trait]
pub trait LinguaAdminRepo: Send + Sync {
    /// Tiles + per-language breakdown over an inclusive epoch-day window.
    async fn usage(&self, from_day: i32, to_day: i32) -> Result<Usage>;
    /// Raw `(epoch_day, value)` points for one metric over the window, optionally for a
    /// single studied language.
    async fn series(
        &self,
        from_day: i32,
        to_day: i32,
        metric: SeriesMetric,
        language: Option<&str>,
    ) -> Result<Vec<(i32, i64)>>;
    /// « Traduction en ligne » daily totals over the window, one row per day × provider ×
    /// pair (change: add-lingua-online-translation).
    async fn online_translation_usage(
        &self,
        from_day: i32,
        to_day: i32,
    ) -> Result<Vec<OnlineUsageRow>>;
    /// The installation counters (change: add-lingua-usage-counts) of the weeks starting
    /// from `from_week` to `to_day`, one row per week × step with the versions summed, and
    /// whether a summed counter reached `ceiling`; `browser` narrows to one family.
    async fn usage_funnel(
        &self,
        from_week: i32,
        to_day: i32,
        browser: Option<&str>,
        ceiling: i64,
    ) -> Result<Vec<FunnelRow>>;
    /// The leaving answers of the weeks starting from `from_week` to `to_day`, one row per
    /// reason, the same way.
    async fn leave_reasons(
        &self,
        from_week: i32,
        to_day: i32,
        browser: Option<&str>,
        ceiling: i64,
    ) -> Result<Vec<ReasonRow>>;
}

/// Orchestrates the ops console over a [`LinguaAdminRepo`].
pub struct LinguaAdminModule {
    repo: Arc<dyn LinguaAdminRepo>,
    /// The counters' ceiling, to mark a capped week (the default without it).
    usage_policy: Option<Arc<dyn UsageCountsPolicy>>,
}

impl LinguaAdminModule {
    pub fn new(repo: Arc<dyn LinguaAdminRepo>) -> Self {
        Self {
            repo,
            usage_policy: None,
        }
    }

    /// Read the usage counters' ceiling from the runtime policy (change:
    /// add-lingua-usage-counts).
    pub fn with_usage_policy(mut self, policy: Arc<dyn UsageCountsPolicy>) -> Self {
        self.usage_policy = Some(policy);
        self
    }

    fn ceiling(&self) -> i64 {
        self.usage_policy
            .as_ref()
            .map_or(DEFAULT_CEILING, |p| p.ceiling())
    }

    /// The installation funnel of the weeks that meet a `yyyy-mm-dd` window, ascending,
    /// optionally for one browser family (`chromium` | `firefox` | `safari`).
    pub async fn get_usage_funnel(
        &self,
        from: &str,
        to: &str,
        browser: &str,
    ) -> Result<Vec<FunnelWeek>> {
        let (f, t) = parse_window(from, to)?;
        let browser = browser_filter(browser, false)?;
        let rows = self
            .repo
            .usage_funnel(monday_of(f), t, browser, self.ceiling())
            .await?;
        Ok(funnel_weeks(rows))
    }

    /// The leaving reasons' totals over the weeks that meet a `yyyy-mm-dd` window, optionally
    /// for one browser family or `unknown`.
    pub async fn get_leave_reasons(
        &self,
        from: &str,
        to: &str,
        browser: &str,
    ) -> Result<LeaveTotals> {
        let (f, t) = parse_window(from, to)?;
        let browser = browser_filter(browser, true)?;
        let rows = self
            .repo
            .leave_reasons(monday_of(f), t, browser, self.ceiling())
            .await?;
        Ok(leave_totals(&rows))
    }

    /// Aggregate tiles + breakdown for a `yyyy-mm-dd` window.
    pub async fn get_usage(&self, from: &str, to: &str) -> Result<Usage> {
        let (f, t) = parse_window(from, to)?;
        self.repo.usage(f, t).await
    }

    /// A per-day series for one metric over a `yyyy-mm-dd` window.
    pub async fn get_series(
        &self,
        from: &str,
        to: &str,
        metric: SeriesMetric,
        language: Option<&str>,
    ) -> Result<Vec<SeriesPoint>> {
        let (f, t) = parse_window(from, to)?;
        let rows = self.repo.series(f, t, metric, language).await?;
        Ok(series_points(rows))
    }

    /// « Traduction en ligne » rows and totals for a `yyyy-mm-dd` window.
    pub async fn get_online_translation_usage(
        &self,
        from: &str,
        to: &str,
    ) -> Result<(Vec<OnlineUsageRow>, OnlineUsageTotals)> {
        let (f, t) = parse_window(from, to)?;
        let rows = self.repo.online_translation_usage(f, t).await?;
        let totals = online_usage_totals(&rows);
        Ok((rows, totals))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_core::{LanguageUsage, parse_iso_day};
    use cymbra_platform::AppError;
    use std::sync::Mutex;

    /// `(from_week, to_day, browser, ceiling)` of one usage-count read.
    type CountsRead = (i32, i32, Option<String>, i64);

    // A recording fake, not a mockall mock: `automock` cannot generate this port's
    // `Option<&str>` arguments without naming a lifetime the trait does not have.
    #[derive(Default)]
    struct FakeRepo {
        usage: Usage,
        // (day, value) rows the series call should echo back (unsorted on purpose).
        series_rows: Vec<(i32, i64)>,
        seen: Mutex<Vec<(i32, i32, Option<String>)>>, // (from, to, language) of series calls
        // The usage counts' rows, and (from_week, to_day, browser, ceiling) of each read.
        funnel_rows: Vec<FunnelRow>,
        reason_rows: Vec<ReasonRow>,
        counts_seen: Mutex<Vec<CountsRead>>,
    }

    #[async_trait]
    impl LinguaAdminRepo for FakeRepo {
        async fn usage(&self, _from: i32, _to: i32) -> Result<Usage> {
            Ok(self.usage.clone())
        }
        async fn series(
            &self,
            from: i32,
            to: i32,
            _metric: SeriesMetric,
            language: Option<&str>,
        ) -> Result<Vec<(i32, i64)>> {
            self.seen
                .lock()
                .unwrap()
                .push((from, to, language.map(str::to_owned)));
            Ok(self.series_rows.clone())
        }
        async fn online_translation_usage(
            &self,
            _from: i32,
            _to: i32,
        ) -> Result<Vec<OnlineUsageRow>> {
            Ok(vec![OnlineUsageRow {
                day: 20_000,
                provider: "gemini".into(),
                pair: "es-fr".into(),
                answered: 7,
                cost_micro_usd: 900,
                ..Default::default()
            }])
        }
        async fn usage_funnel(
            &self,
            from_week: i32,
            to_day: i32,
            browser: Option<&str>,
            ceiling: i64,
        ) -> Result<Vec<FunnelRow>> {
            self.counts_seen.lock().unwrap().push((
                from_week,
                to_day,
                browser.map(str::to_owned),
                ceiling,
            ));
            Ok(self.funnel_rows.clone())
        }
        async fn leave_reasons(
            &self,
            from_week: i32,
            to_day: i32,
            browser: Option<&str>,
            ceiling: i64,
        ) -> Result<Vec<ReasonRow>> {
            self.counts_seen.lock().unwrap().push((
                from_week,
                to_day,
                browser.map(str::to_owned),
                ceiling,
            ));
            Ok(self.reason_rows.clone())
        }
    }

    /// 2026-10-19 (a Monday) and the Wednesday after it.
    const MONDAY: &str = "2026-10-19";
    const WEDNESDAY: &str = "2026-10-21";

    #[tokio::test]
    async fn the_funnel_reads_the_weeks_that_meet_the_window() {
        let monday = parse_iso_day(MONDAY).unwrap();
        let repo = Arc::new(FakeRepo {
            funnel_rows: vec![
                FunnelRow {
                    week: monday,
                    step: "installed".into(),
                    count: 40,
                    capped: false,
                },
                FunnelRow {
                    week: monday,
                    step: "first_analysis".into(),
                    count: 25,
                    capped: false,
                },
            ],
            ..Default::default()
        });
        let m = LinguaAdminModule::new(repo.clone() as Arc<dyn LinguaAdminRepo>);
        let weeks = m
            .get_usage_funnel(WEDNESDAY, "2026-10-28", "firefox")
            .await
            .unwrap();
        assert_eq!(weeks.len(), 1);
        assert_eq!((weeks[0].installed, weeks[0].first_analysis), (40, 25));
        // A window starting mid-week reads that whole week, with the default ceiling.
        let seen = repo.counts_seen.lock().unwrap();
        assert_eq!(
            seen.as_slice(),
            [(monday, monday + 9, Some("firefox".to_string()), 5_000)]
        );
    }

    #[tokio::test]
    async fn the_ceiling_comes_from_the_policy() {
        let mut policy = crate::usage_counts::MockUsageCountsPolicy::new();
        policy.expect_ceiling().return_const(12i64);
        let repo = Arc::new(FakeRepo {
            reason_rows: vec![ReasonRow {
                reason: "slow".into(),
                count: 12,
                capped: true,
            }],
            ..Default::default()
        });
        let m = LinguaAdminModule::new(repo.clone() as Arc<dyn LinguaAdminRepo>)
            .with_usage_policy(Arc::new(policy));
        assert!(
            m.get_usage_funnel(MONDAY, MONDAY, "")
                .await
                .unwrap()
                .is_empty()
        );
        let totals = m
            .get_leave_reasons(MONDAY, MONDAY, "unknown")
            .await
            .unwrap();
        assert_eq!(totals.reasons[3], ("slow", 12));
        assert!(totals.capped);
        let monday = parse_iso_day(MONDAY).unwrap();
        let seen = repo.counts_seen.lock().unwrap();
        assert_eq!(
            seen.as_slice(),
            [
                (monday, monday, None, 12),
                (monday, monday, Some("unknown".to_string()), 12)
            ]
        );
    }

    #[tokio::test]
    async fn a_bad_filter_or_window_never_reaches_the_repo() {
        let repo = Arc::new(FakeRepo::default());
        let m = LinguaAdminModule::new(repo.clone() as Arc<dyn LinguaAdminRepo>);
        for (from, to, browser) in [
            (MONDAY, MONDAY, "netscape"),
            (MONDAY, MONDAY, "unknown"),
            (WEDNESDAY, MONDAY, ""),
        ] {
            assert!(matches!(
                m.get_usage_funnel(from, to, browser).await,
                Err(AppError::InvalidArgument(_))
            ));
        }
        assert!(matches!(
            m.get_leave_reasons(MONDAY, MONDAY, "edge").await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(repo.counts_seen.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn online_translation_usage_passes_rows_and_totals_through() {
        let m = module(FakeRepo::default());
        let (rows, totals) = m
            .get_online_translation_usage("2026-10-01", "2026-10-11")
            .await
            .unwrap();
        assert_eq!(rows[0].provider, "gemini");
        assert_eq!((totals.answered, totals.cost_micro_usd), (7, 900));
        assert!(
            m.get_online_translation_usage("2026-10-11", "2026-10-01")
                .await
                .is_err()
        );
    }

    fn module(repo: FakeRepo) -> LinguaAdminModule {
        LinguaAdminModule::new(Arc::new(repo))
    }

    #[tokio::test]
    async fn usage_passes_through_and_a_bad_window_is_rejected() {
        let repo = FakeRepo {
            usage: Usage {
                active_accounts: 4,
                words_learned: 20,
                reviews: 30,
                words_read: 1_000,
                new_words_seen: 50,
                by_language: vec![LanguageUsage {
                    language: "en".into(),
                    active_accounts: 4,
                    words_learned: 20,
                    reviews: 30,
                    words_read: 1_000,
                    new_words_seen: 50,
                }],
                readers: 4,
                learners: 3,
                reviewers: 1,
            },
            ..Default::default()
        };
        let m = module(repo);
        let u = m.get_usage("2026-09-01", "2026-09-30").await.unwrap();
        assert_eq!(u.active_accounts, 4);
        assert_eq!(u.by_language[0].language, "en");
        assert_eq!((u.words_read, u.new_words_seen), (1_000, 50));
        assert_eq!(u.by_language[0].new_words_seen, 50);
        // A backwards window never reaches the repo.
        assert!(matches!(
            m.get_usage("2026-09-30", "2026-09-01").await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn series_is_formatted_sorted_and_scopes_language() {
        let base = parse_iso_day("2026-09-10").unwrap();
        let repo = Arc::new(FakeRepo {
            series_rows: vec![(base + 1, 3), (base, 1)],
            ..Default::default()
        });
        let m = LinguaAdminModule::new(repo.clone() as Arc<dyn LinguaAdminRepo>);
        let pts = m
            .get_series(
                "2026-09-10",
                "2026-09-12",
                SeriesMetric::Reviews,
                Some("en"),
            )
            .await
            .unwrap();
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0].day, "2026-09-10"); // sorted ascending by day
        assert_eq!(pts[1].value, 3);
        // The window and language filter reached the repo verbatim.
        let seen = repo.seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].0, base);
        assert_eq!(seen[0].2.as_deref(), Some("en"));
    }
}
