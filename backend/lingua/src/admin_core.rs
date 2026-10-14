// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Pure shaping for the Lingua ops console (host-tested): the UTC-day window parse,
//! the epoch-day ⇄ `yyyy-mm-dd` conversion (`lingua.daily_stats.day` is an epoch-day
//! integer; the console speaks `yyyy-mm-dd` like `/usage`), the aggregate value
//! objects, and the day-series formatting. No account data appears in any type here —
//! the privacy allow-list is a property of the shapes, and it is tested.

use chrono::{Duration, NaiveDate};
use cymbra_platform::{AppError, Result};

use crate::usage_counts_core::{Browser, Reason, Step};

/// The Unix epoch as a civil date — the origin of the epoch-day key.
fn epoch() -> NaiveDate {
    NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 is a valid date")
}

/// Parse a `yyyy-mm-dd` UTC day into the epoch-day integer stored in `daily_stats`.
pub fn parse_iso_day(s: &str) -> Result<i32> {
    let d = NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").map_err(|_| {
        AppError::InvalidArgument(format!("invalid day {s:?}, expected yyyy-mm-dd"))
    })?;
    Ok((d - epoch()).num_days() as i32)
}

/// Format an epoch-day integer back to `yyyy-mm-dd`.
pub fn iso_day(day: i32) -> String {
    (epoch() + Duration::days(day as i64))
        .format("%Y-%m-%d")
        .to_string()
}

/// Parse and validate a closed window; `from` must not be after `to`.
pub fn parse_window(from: &str, to: &str) -> Result<(i32, i32)> {
    let (f, t) = (parse_iso_day(from)?, parse_iso_day(to)?);
    if f > t {
        return Err(AppError::InvalidArgument(
            "window from_day is after to_day".into(),
        ));
    }
    Ok((f, t))
}

/// Which per-day metric a series returns (chosen server-side, never a client column).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeriesMetric {
    WordsLearned,
    Reviews,
    Exposures,
    UnknownSeen,
}

/// One studied language's aggregates within a window (counts only).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageUsage {
    pub language: String,
    pub active_accounts: i64,
    pub words_learned: i64,
    pub reviews: i64,
    pub words_read: i64,
    pub new_words_seen: i64,
}

/// The console's tile + breakdown aggregate for a window (no account data).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Usage {
    pub active_accounts: i64,
    pub words_learned: i64,
    pub reviews: i64,
    pub words_read: i64,
    pub new_words_seen: i64,
    pub by_language: Vec<LanguageUsage>,
    /// Accounts that read, learned a word, or reviewed in the window — the
    /// contributors behind `words_read`, `words_learned` and `reviews`, so a
    /// published figure can be gated on the accounts that made it (change:
    /// add-discord-notifications, task 3.8). Identifier-free counts, kept off
    /// the admin RPC.
    pub readers: i64,
    pub learners: i64,
    pub reviewers: i64,
}

/// One day × provider × pair of « Traduction en ligne » totals, as stored (no account, no
/// text — change: add-lingua-online-translation).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct OnlineUsageRow {
    pub day: i32,
    pub provider: String,
    pub pair: String,
    pub answered: i64,
    pub failed: i64,
    pub limited: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_micro_usd: i64,
}

/// The window's totals of online translation rows.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct OnlineUsageTotals {
    pub answered: i64,
    pub failed: i64,
    pub limited: i64,
    pub cost_micro_usd: i64,
}

pub fn online_usage_totals(rows: &[OnlineUsageRow]) -> OnlineUsageTotals {
    rows.iter()
        .fold(OnlineUsageTotals::default(), |t, r| OnlineUsageTotals {
            answered: t.answered + r.answered,
            failed: t.failed + r.failed,
            limited: t.limited + r.limited,
            cost_micro_usd: t.cost_micro_usd + r.cost_micro_usd,
        })
}

/// The browser filter of the installation funnel and the leaving reasons (change:
/// add-lingua-usage-counts, D13): empty is every family; otherwise one of the three families,
/// or `unknown` where `allow_unknown` (leaving answers may not know theirs).
pub fn browser_filter(s: &str, allow_unknown: bool) -> Result<Option<&'static str>> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    if let Some(b) = Browser::parse(s) {
        return Ok(Some(b.as_str()));
    }
    if allow_unknown && s == crate::usage_counts_core::UNKNOWN {
        return Ok(Some(crate::usage_counts_core::UNKNOWN));
    }
    Err(AppError::InvalidArgument(format!(
        "unknown browser family {s:?}"
    )))
}

/// One week × step of the installation counters, versions summed, as the repository
/// returns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunnelRow {
    pub week: i32,
    pub step: String,
    pub count: i64,
    /// One of the counters summed reached the ceiling.
    pub capped: bool,
}

/// One installation week of the funnel (counts only, no identifier).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct FunnelWeek {
    /// The Monday that starts the installation week (epoch day).
    pub week: i32,
    pub installed: i64,
    pub set_up: i64,
    pub first_analysis: i64,
    pub returned_day_1: i64,
    pub returned_days_7_13: i64,
    pub returned_days_30_59: i64,
    pub capped: bool,
}

/// Pivot week × step rows into one funnel row per week, ascending; a step the server does
/// not know (none should exist) is ignored.
pub fn funnel_weeks(rows: Vec<FunnelRow>) -> Vec<FunnelWeek> {
    let mut weeks: std::collections::BTreeMap<i32, FunnelWeek> =
        std::collections::BTreeMap::new();
    for r in rows {
        let Some(step) = Step::ALL.into_iter().find(|s| s.as_str() == r.step) else {
            continue;
        };
        let w = weeks.entry(r.week).or_insert_with(|| FunnelWeek {
            week: r.week,
            ..Default::default()
        });
        let slot = match step {
            Step::Installed => &mut w.installed,
            Step::SetUp => &mut w.set_up,
            Step::FirstAnalysis => &mut w.first_analysis,
            Step::ReturnedDay1 => &mut w.returned_day_1,
            Step::ReturnedDays7To13 => &mut w.returned_days_7_13,
            Step::ReturnedDays30To59 => &mut w.returned_days_30_59,
        };
        *slot += r.count;
        w.capped |= r.capped;
    }
    weeks.into_values().collect()
}

/// One leaving reason's total over a window, as the repository returns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReasonRow {
    pub reason: String,
    pub count: i64,
    pub capped: bool,
}

/// The six leaving reasons' totals, in their fixed order (zero when none was given).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LeaveTotals {
    pub reasons: Vec<(&'static str, i64)>,
    pub capped: bool,
}

pub fn leave_totals(rows: &[ReasonRow]) -> LeaveTotals {
    LeaveTotals {
        reasons: Reason::ALL
            .into_iter()
            .map(|reason| {
                let total = rows
                    .iter()
                    .filter(|r| r.reason == reason.as_str())
                    .map(|r| r.count)
                    .sum();
                (reason.as_str(), total)
            })
            .collect(),
        capped: rows.iter().any(|r| r.capped),
    }
}

/// One point of a per-day series: a day and a count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeriesPoint {
    pub day: String,
    pub value: i64,
}

/// Format raw `(epoch_day, value)` rows into an ISO-day series, ascending by day.
pub fn series_points(mut rows: Vec<(i32, i64)>) -> Vec<SeriesPoint> {
    rows.sort_by_key(|(day, _)| *day);
    rows.into_iter()
        .map(|(day, value)| SeriesPoint {
            day: iso_day(day),
            value,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_day_round_trips_through_the_epoch_day() {
        for s in ["1970-01-01", "2026-09-11", "2000-02-29"] {
            assert_eq!(iso_day(parse_iso_day(s).unwrap()), s);
        }
        assert_eq!(parse_iso_day("1970-01-01").unwrap(), 0);
        assert_eq!(parse_iso_day("1970-01-02").unwrap(), 1);
    }

    #[test]
    fn a_bad_day_is_an_invalid_argument() {
        assert!(matches!(
            parse_iso_day("11-09-2026"),
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            parse_iso_day("not-a-day"),
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[test]
    fn a_window_must_not_run_backwards() {
        assert_eq!(
            parse_window("2026-09-01", "2026-09-30").unwrap(),
            (
                parse_iso_day("2026-09-01").unwrap(),
                parse_iso_day("2026-09-30").unwrap()
            )
        );
        // A single day is a valid (degenerate) window.
        let (f, t) = parse_window("2026-09-11", "2026-09-11").unwrap();
        assert_eq!(f, t);
        assert!(matches!(
            parse_window("2026-09-30", "2026-09-01"),
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[test]
    fn series_points_are_iso_dated_and_sorted() {
        let base = parse_iso_day("2026-09-10").unwrap();
        let out = series_points(vec![(base + 2, 5), (base, 1), (base + 1, 3)]);
        assert_eq!(
            out,
            vec![
                SeriesPoint {
                    day: "2026-09-10".into(),
                    value: 1
                },
                SeriesPoint {
                    day: "2026-09-11".into(),
                    value: 3
                },
                SeriesPoint {
                    day: "2026-09-12".into(),
                    value: 5
                },
            ]
        );
    }

    #[test]
    fn online_usage_totals_sum_every_row() {
        let rows = vec![
            OnlineUsageRow {
                answered: 3,
                failed: 1,
                cost_micro_usd: 700,
                ..Default::default()
            },
            OnlineUsageRow {
                answered: 2,
                limited: 4,
                cost_micro_usd: 300,
                ..Default::default()
            },
        ];
        assert_eq!(
            online_usage_totals(&rows),
            OnlineUsageTotals {
                answered: 5,
                failed: 1,
                limited: 4,
                cost_micro_usd: 1_000
            }
        );
        assert_eq!(online_usage_totals(&[]), OnlineUsageTotals::default());
    }

    #[test]
    fn the_browser_filter_is_closed() {
        assert_eq!(browser_filter("", false).unwrap(), None);
        assert_eq!(browser_filter(" firefox ", false).unwrap(), Some("firefox"));
        assert_eq!(browser_filter("unknown", true).unwrap(), Some("unknown"));
        assert!(matches!(
            browser_filter("unknown", false),
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            browser_filter("chromium' OR 1=1", true),
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[test]
    fn funnel_rows_pivot_into_weeks() {
        let row = |week, step: &str, count, capped| FunnelRow {
            week,
            step: step.into(),
            count,
            capped,
        };
        let weeks = funnel_weeks(vec![
            row(11, "installed", 3, false),
            row(4, "installed", 40, false),
            row(4, "set_up", 30, false),
            row(4, "first_analysis", 25, false),
            row(4, "returned_day_1", 12, false),
            row(4, "returned_days_7_13", 8, true),
            row(4, "returned_days_30_59", 2, false),
            row(4, "retired_step", 99, true),
            row(18, "retired_step", 1, true),
        ]);
        assert_eq!(
            weeks,
            vec![
                FunnelWeek {
                    week: 4,
                    installed: 40,
                    set_up: 30,
                    first_analysis: 25,
                    returned_day_1: 12,
                    returned_days_7_13: 8,
                    returned_days_30_59: 2,
                    capped: true,
                },
                FunnelWeek {
                    week: 11,
                    installed: 3,
                    ..Default::default()
                },
            ]
        );
        assert!(funnel_weeks(vec![]).is_empty());
    }

    #[test]
    fn leave_totals_list_the_six_reasons_in_order() {
        let reason = |r: &str, count, capped| ReasonRow {
            reason: r.into(),
            count,
            capped,
        };
        let t = leave_totals(&[
            reason("slow", 2, false),
            reason("slow", 1, false),
            reason("not_using", 5, true),
        ]);
        assert_eq!(
            t.reasons,
            vec![
                ("not_using", 5),
                ("highlighting", 0),
                ("missing_language", 0),
                ("slow", 3),
                ("unclear", 0),
                ("other", 0)
            ]
        );
        assert!(t.capped);
        assert!(!leave_totals(&[]).capped);
    }
}
