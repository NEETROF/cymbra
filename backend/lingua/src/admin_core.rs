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
}

/// One studied language's aggregates within a window (counts only).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageUsage {
    pub language: String,
    pub active_accounts: i64,
    pub words_learned: i64,
    pub reviews: i64,
}

/// The console's tile + breakdown aggregate for a window (no account data).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Usage {
    pub active_accounts: i64,
    pub words_learned: i64,
    pub reviews: i64,
    pub by_language: Vec<LanguageUsage>,
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
}
