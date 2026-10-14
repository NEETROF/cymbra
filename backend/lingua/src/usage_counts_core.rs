// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The pure rules of the anonymous usage counts (change: add-lingua-usage-counts, design
//! D1, D9, D10): the closed sets (six steps, three browser families, six leaving reasons),
//! the version pattern, the week check, the accepted window and the retention cut-off.
//! Host-tested; nothing here touches a network address, a header or a clock.

use chrono::{Months, NaiveDate};

/// Weeks before the server's UTC day an installation week may start: day 59 of an
/// installation plus time offline (design D10).
pub const WINDOW_WEEKS_BEFORE: i32 = 18;

/// Months a counter is kept (design D9, the CNIL's figure for audience measurement).
pub const RETENTION_MONTHS: u32 = 25;

/// The value a leaving page sends when its parameters were absent or malformed.
pub const UNKNOWN: &str = "unknown";

/// The six steps of an installation (design D1), as stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Step {
    Installed,
    SetUp,
    FirstAnalysis,
    ReturnedDay1,
    ReturnedDays7To13,
    ReturnedDays30To59,
}

impl Step {
    pub const ALL: [Step; 6] = [
        Step::Installed,
        Step::SetUp,
        Step::FirstAnalysis,
        Step::ReturnedDay1,
        Step::ReturnedDays7To13,
        Step::ReturnedDays30To59,
    ];

    /// The step's stored name.
    pub fn as_str(self) -> &'static str {
        match self {
            Step::Installed => "installed",
            Step::SetUp => "set_up",
            Step::FirstAnalysis => "first_analysis",
            Step::ReturnedDay1 => "returned_day_1",
            Step::ReturnedDays7To13 => "returned_days_7_13",
            Step::ReturnedDays30To59 => "returned_days_30_59",
        }
    }

    /// The step a `UsageStep` wire value names (`USAGE_STEP_UNSPECIFIED` and unknown
    /// values name none).
    pub fn from_wire(v: i32) -> Option<Step> {
        match v {
            1 => Some(Step::Installed),
            2 => Some(Step::SetUp),
            3 => Some(Step::FirstAnalysis),
            4 => Some(Step::ReturnedDay1),
            5 => Some(Step::ReturnedDays7To13),
            6 => Some(Step::ReturnedDays30To59),
            _ => None,
        }
    }
}

/// A browser family, from the extension's build (never the User-Agent).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Browser {
    Chromium,
    Firefox,
    Safari,
}

impl Browser {
    pub const ALL: [Browser; 3] = [Browser::Chromium, Browser::Firefox, Browser::Safari];

    pub fn as_str(self) -> &'static str {
        match self {
            Browser::Chromium => "chromium",
            Browser::Firefox => "firefox",
            Browser::Safari => "safari",
        }
    }

    /// The family a `BrowserFamily` wire value names (unspecified names none).
    pub fn from_wire(v: i32) -> Option<Browser> {
        match v {
            1 => Some(Browser::Chromium),
            2 => Some(Browser::Firefox),
            3 => Some(Browser::Safari),
            _ => None,
        }
    }

    pub fn parse(s: &str) -> Option<Browser> {
        Browser::ALL.into_iter().find(|b| b.as_str() == s)
    }
}

/// The six fixed answers of the leaving page (design D12), as stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    NotUsing,
    Highlighting,
    MissingLanguage,
    Slow,
    Unclear,
    Other,
}

impl Reason {
    pub const ALL: [Reason; 6] = [
        Reason::NotUsing,
        Reason::Highlighting,
        Reason::MissingLanguage,
        Reason::Slow,
        Reason::Unclear,
        Reason::Other,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Reason::NotUsing => "not_using",
            Reason::Highlighting => "highlighting",
            Reason::MissingLanguage => "missing_language",
            Reason::Slow => "slow",
            Reason::Unclear => "unclear",
            Reason::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Reason> {
        Reason::ALL.into_iter().find(|r| r.as_str() == s)
    }
}

/// `major.minor`, one to three digits each (`1.10`).
pub fn is_version(s: &str) -> bool {
    let digits = |p: &str| (1..=3).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_digit());
    match s.split_once('.') {
        Some((major, minor)) => digits(major) && digits(minor),
        None => false,
    }
}

/// Whether a day number (days since 1970-01-01, a Thursday) is a Monday.
pub fn is_monday(day: i32) -> bool {
    day.rem_euclid(7) == 4
}

/// The Monday that starts the week holding `day`.
pub fn monday_of(day: i32) -> i32 {
    day - (day - 4).rem_euclid(7)
}

/// Whether an installation week is counted on the server's UTC `today`: from 18 weeks
/// before it to the day after it (time zones ahead of UTC).
pub fn week_in_window(week: i32, today: i32) -> bool {
    week >= today - WINDOW_WEEKS_BEFORE * 7 && week <= today + 1
}

fn epoch() -> NaiveDate {
    NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 is a valid date")
}

/// The first day still kept on `today`: counters whose week starts before it are deleted
/// (25 months back, calendar months).
pub fn retention_cutoff(today: i32) -> i32 {
    let date = epoch() + chrono::Duration::days(i64::from(today));
    let cutoff = date
        .checked_sub_months(Months::new(RETENTION_MONTHS))
        .unwrap_or(epoch());
    (cutoff - epoch()).num_days() as i32
}

/// Why a report or an answer is refused as invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    NoStep,
    TooManySteps,
    UnknownStep,
    RepeatedStep,
    UnknownBrowser,
    BadVersion,
    NotAMonday,
    UnknownReason,
}

impl Invalid {
    /// The neutral message a client receives (it names the rule, never the value).
    pub fn message(self) -> &'static str {
        match self {
            Invalid::NoStep => "a report holds at least one step",
            Invalid::TooManySteps => "a report holds at most six steps",
            Invalid::UnknownStep => "unknown step",
            Invalid::RepeatedStep => "a step appears twice",
            Invalid::UnknownBrowser => "unknown browser family",
            Invalid::BadVersion => "the version is not major.minor",
            Invalid::NotAMonday => "the installation week does not start on a Monday",
            Invalid::UnknownReason => "unknown reason",
        }
    }
}

/// A report whose values are all bounded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedReport {
    pub week: i32,
    pub browser: Browser,
    pub version: String,
    pub steps: Vec<Step>,
}

/// Check a report's values (design D10): one to six distinct known steps, a known browser
/// family, a `major.minor` version and a Monday. The window is checked separately: a week
/// out of range is acknowledged, not refused.
pub fn check_report(
    week: i32,
    browser: i32,
    version: &str,
    steps: &[i32],
) -> Result<CheckedReport, Invalid> {
    if steps.is_empty() {
        return Err(Invalid::NoStep);
    }
    if steps.len() > Step::ALL.len() {
        return Err(Invalid::TooManySteps);
    }
    let mut checked = Vec::with_capacity(steps.len());
    for &s in steps {
        let step = Step::from_wire(s).ok_or(Invalid::UnknownStep)?;
        if checked.contains(&step) {
            return Err(Invalid::RepeatedStep);
        }
        checked.push(step);
    }
    let browser = Browser::from_wire(browser).ok_or(Invalid::UnknownBrowser)?;
    if !is_version(version) {
        return Err(Invalid::BadVersion);
    }
    if !is_monday(week) {
        return Err(Invalid::NotAMonday);
    }
    checked.sort();
    Ok(CheckedReport {
        week,
        browser,
        version: version.to_string(),
        steps: checked,
    })
}

/// A leaving answer whose values are all bounded; `browser` and `version` may be `unknown`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedLeave {
    pub reason: Reason,
    pub browser: &'static str,
    pub version: String,
}

/// Check a leaving answer (design D10, D12): one of the six reasons, a browser family or
/// `unknown`, a `major.minor` version or `unknown`. Nothing else is accepted.
pub fn check_leave(reason: &str, browser: &str, version: &str) -> Result<CheckedLeave, Invalid> {
    let reason = Reason::parse(reason).ok_or(Invalid::UnknownReason)?;
    let browser = match Browser::parse(browser) {
        Some(b) => b.as_str(),
        None if browser == UNKNOWN => UNKNOWN,
        None => return Err(Invalid::UnknownBrowser),
    };
    if version != UNKNOWN && !is_version(version) {
        return Err(Invalid::BadVersion);
    }
    Ok(CheckedLeave {
        reason,
        browser,
        version: version.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-19, a Monday (the spec's example week).
    fn monday_19_october() -> i32 {
        (NaiveDate::from_ymd_opt(2026, 10, 19).unwrap() - epoch()).num_days() as i32
    }

    #[test]
    fn the_wire_and_stored_names_of_the_closed_sets() {
        let names: Vec<_> = Step::ALL.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            names,
            [
                "installed",
                "set_up",
                "first_analysis",
                "returned_day_1",
                "returned_days_7_13",
                "returned_days_30_59"
            ]
        );
        for (i, step) in Step::ALL.iter().enumerate() {
            assert_eq!(Step::from_wire(i as i32 + 1), Some(*step));
        }
        assert_eq!(Step::from_wire(0), None);
        assert_eq!(Step::from_wire(7), None);
        assert_eq!(Browser::from_wire(1), Some(Browser::Chromium));
        assert_eq!(Browser::from_wire(2), Some(Browser::Firefox));
        assert_eq!(Browser::from_wire(3), Some(Browser::Safari));
        assert_eq!(Browser::from_wire(0), None);
        assert_eq!(Browser::parse("safari"), Some(Browser::Safari));
        assert_eq!(Browser::parse("edge"), None);
        let reasons: Vec<_> = Reason::ALL.iter().map(|r| r.as_str()).collect();
        assert_eq!(
            reasons,
            [
                "not_using",
                "highlighting",
                "missing_language",
                "slow",
                "unclear",
                "other"
            ]
        );
        for r in Reason::ALL {
            assert_eq!(Reason::parse(r.as_str()), Some(r));
        }
    }

    #[test]
    fn a_version_is_major_dot_minor() {
        for ok in ["1.9", "1.10", "0.0", "123.456"] {
            assert!(is_version(ok), "{ok}");
        }
        for bad in [
            "", "1", "1.", ".1", "1.9.0", "1234.1", "1.1234", "v1.9", "1.x", " 1.9", "１.９",
        ] {
            assert!(!is_version(bad), "{bad:?}");
        }
    }

    #[test]
    fn mondays_and_their_weeks() {
        let monday = monday_19_october();
        assert!(is_monday(monday));
        assert_eq!(monday % 7, 4);
        for d in 1..7 {
            assert!(!is_monday(monday + d));
            assert_eq!(monday_of(monday + d), monday, "day {d} of the week");
        }
        assert_eq!(monday_of(monday), monday);
        // 1970-01-05, the first Monday, and a day before the epoch.
        assert!(is_monday(4));
        assert_eq!(monday_of(0), -3);
        assert!(is_monday(-3));
    }

    #[test]
    fn the_window_runs_from_18_weeks_before_to_the_day_after() {
        let today = monday_19_october() + 2; // a Wednesday
        assert!(week_in_window(today - 2, today));
        assert!(week_in_window(today - 126, today));
        assert!(!week_in_window(today - 127, today));
        assert!(week_in_window(today + 1, today));
        assert!(!week_in_window(today + 2, today));
        // The spec's scenario: a week that started 200 days ago.
        assert!(!week_in_window(today - 200, today));
    }

    #[test]
    fn counters_are_kept_25_months() {
        let today = (NaiveDate::from_ymd_opt(2028, 11, 15).unwrap() - epoch()).num_days() as i32;
        let cutoff = retention_cutoff(today);
        assert_eq!(
            cutoff,
            (NaiveDate::from_ymd_opt(2026, 10, 15).unwrap() - epoch()).num_days() as i32
        );
        // A week 26 months old is before the cut-off; one 24 months old is not.
        let old = (NaiveDate::from_ymd_opt(2026, 9, 14).unwrap() - epoch()).num_days() as i32;
        let young = (NaiveDate::from_ymd_opt(2026, 11, 16).unwrap() - epoch()).num_days() as i32;
        assert!(old < cutoff);
        assert!(young >= cutoff);
        // Near the epoch the cut-off is simply a day before it.
        assert!(retention_cutoff(10) < 0);
    }

    #[test]
    fn a_valid_report() {
        let week = monday_19_october();
        let r = check_report(week, 2, "1.10", &[2, 1]).unwrap();
        assert_eq!(r.week, week);
        assert_eq!(r.browser, Browser::Firefox);
        assert_eq!(r.version, "1.10");
        assert_eq!(r.steps, vec![Step::Installed, Step::SetUp]);
        // All six at once.
        assert_eq!(
            check_report(week, 1, "1.9", &[1, 2, 3, 4, 5, 6])
                .unwrap()
                .steps
                .len(),
            6
        );
    }

    #[test]
    fn malformed_reports_are_invalid() {
        let week = monday_19_october();
        assert_eq!(check_report(week, 1, "1.9", &[]), Err(Invalid::NoStep));
        assert_eq!(
            check_report(week, 1, "1.9", &[1, 2, 3, 4, 5, 6, 1]),
            Err(Invalid::TooManySteps)
        );
        assert_eq!(
            check_report(week, 1, "1.9", &[1, 9]),
            Err(Invalid::UnknownStep)
        );
        assert_eq!(
            check_report(week, 1, "1.9", &[0]),
            Err(Invalid::UnknownStep)
        );
        assert_eq!(
            check_report(week, 1, "1.9", &[3, 3]),
            Err(Invalid::RepeatedStep)
        );
        assert_eq!(
            check_report(week, 0, "1.9", &[1]),
            Err(Invalid::UnknownBrowser)
        );
        assert_eq!(
            check_report(week, 4, "1.9", &[1]),
            Err(Invalid::UnknownBrowser)
        );
        assert_eq!(
            check_report(week, 1, "1.9.0", &[1]),
            Err(Invalid::BadVersion)
        );
        assert_eq!(
            check_report(week + 1, 1, "1.9", &[1]),
            Err(Invalid::NotAMonday)
        );
        for invalid in [
            Invalid::NoStep,
            Invalid::TooManySteps,
            Invalid::UnknownStep,
            Invalid::RepeatedStep,
            Invalid::UnknownBrowser,
            Invalid::BadVersion,
            Invalid::NotAMonday,
            Invalid::UnknownReason,
        ] {
            assert!(!invalid.message().is_empty());
        }
    }

    #[test]
    fn leaving_answers_are_bounded() {
        let a = check_leave("slow", "firefox", "1.10").unwrap();
        assert_eq!(a.reason, Reason::Slow);
        assert_eq!((a.browser, a.version.as_str()), ("firefox", "1.10"));
        let opened_by_hand = check_leave("other", "unknown", "unknown").unwrap();
        assert_eq!(
            (opened_by_hand.browser, opened_by_hand.version.as_str()),
            ("unknown", "unknown")
        );
        assert_eq!(
            check_leave("I hate it, mail me at a@b.c", "firefox", "1.10"),
            Err(Invalid::UnknownReason)
        );
        assert_eq!(
            check_leave("slow", "netscape", "1.10"),
            Err(Invalid::UnknownBrowser)
        );
        assert_eq!(
            check_leave("slow", "firefox", "latest"),
            Err(Invalid::BadVersion)
        );
    }
}
