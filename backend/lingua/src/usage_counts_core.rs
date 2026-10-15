// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The pure rules of the anonymous usage counts (change: add-lingua-usage-counts, design
//! D1, D9, D10): the closed sets (six steps, three browser families, six leaving reasons),
//! the version pattern, the accepted window of installation days and the retention cut-off.
//! Host-tested; nothing here touches a network address, a header or a clock.

use chrono::{Months, NaiveDate};

/// Days before the server's UTC day an installation day may be (design D10): a device sends
/// its last step on day 59 and forgets the installation at day 60; the rest is room for a
/// device clock that runs up to a month behind.
pub const WINDOW_DAYS_BEFORE: i32 = 90;

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

/// The version a report is filed under when its own is not one the counters name (design
/// D10): a well-formed `major.minor` outside the accepted range.
pub const OTHER_VERSION: &str = "other";

/// The latest released `major.minor` when the runtime setting is absent or malformed.
pub const DEFAULT_LATEST_VERSION: &str = "1.10";

/// How many minor releases past the latest the counters still name, so a release that
/// ships before the setting is moved is still counted under its own version (design D10).
pub const MINORS_AHEAD: u32 = 3;

/// `major.minor`, one to three digits each and no leading zero (`1.10`, never `01.9` or
/// `1.09`, which would name one release twice).
pub fn parse_version(s: &str) -> Option<(u32, u32)> {
    let number = |p: &str| {
        let well_formed = (1..=3).contains(&p.len())
            && p.bytes().all(|b| b.is_ascii_digit())
            && (p == "0" || !p.starts_with('0'));
        if well_formed {
            p.parse::<u32>().ok()
        } else {
            None
        }
    };
    let (major, minor) = s.split_once('.')?;
    Some((number(major)?, number(minor)?))
}

/// Whether `s` is a well-formed `major.minor` (see [`parse_version`]).
pub fn is_version(s: &str) -> bool {
    parse_version(s).is_some()
}

/// The version a well-formed `version` is counted under, given the latest released
/// `major.minor` (a runtime setting; [`DEFAULT_LATEST_VERSION`] when it is malformed): its
/// own name when it has the latest's major and a minor at most [`MINORS_AHEAD`] past it,
/// [`OTHER_VERSION`] otherwise. The version is part of a counter's key, so this bounds the
/// rows a day can hold: a forged `999.999` adds to `other`, never a row of its own.
pub fn filed_version(version: (u32, u32), latest: &str) -> String {
    let (major, minor) = parse_version(latest)
        .or_else(|| parse_version(DEFAULT_LATEST_VERSION))
        .expect("the default latest version is well formed");
    if version.0 == major && version.1 <= minor + MINORS_AHEAD {
        format!("{}.{}", version.0, version.1)
    } else {
        OTHER_VERSION.to_string()
    }
}

/// The Monday that starts the week holding `day` (days since 1970-01-01, a Thursday): a
/// leaving answer is filed under the UTC week it arrived in.
pub fn monday_of(day: i32) -> i32 {
    day - (day - 4).rem_euclid(7)
}

/// Whether an installation day is counted on the server's UTC `today`: from
/// [`WINDOW_DAYS_BEFORE`] days before it to the day after it (time zones ahead of UTC).
pub fn day_in_window(day: i32, today: i32) -> bool {
    day >= today - WINDOW_DAYS_BEFORE && day <= today + 1
}

fn epoch() -> NaiveDate {
    NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 is a valid date")
}

/// The first day still kept on `today`: installation counters whose day, and leaving
/// counters whose week, is before it are deleted (25 months back, calendar months).
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
            Invalid::UnknownReason => "unknown reason",
        }
    }
}

/// A report whose values are all bounded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedReport {
    /// Day 0: the installation's local calendar day, as a day number.
    pub day: i32,
    pub browser: Browser,
    /// `(major, minor)`; [`filed_version`] says what it is counted under.
    pub version: (u32, u32),
    pub steps: Vec<Step>,
}

/// Check a report's values (design D10): one to six distinct known steps, a known browser
/// family and a `major.minor` version. Any day number is well formed; the window is checked
/// separately: a day out of range is acknowledged, not refused.
pub fn check_report(
    day: i32,
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
    let version = parse_version(version).ok_or(Invalid::BadVersion)?;
    checked.sort();
    Ok(CheckedReport {
        day,
        browser,
        version,
        steps: checked,
    })
}

/// A leaving answer whose values are all bounded; `browser` and `version` may be `unknown`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedLeave {
    pub reason: Reason,
    pub browser: &'static str,
    /// `(major, minor)`, or `None` for `unknown`.
    pub version: Option<(u32, u32)>,
}

impl CheckedLeave {
    /// The version the answer is counted under: `unknown`, or what [`filed_version`] says.
    pub fn filed_version(&self, latest: &str) -> String {
        self.version
            .map_or_else(|| UNKNOWN.to_string(), |v| filed_version(v, latest))
    }
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
    let version = if version == UNKNOWN {
        None
    } else {
        Some(parse_version(version).ok_or(Invalid::BadVersion)?)
    };
    Ok(CheckedLeave {
        reason,
        browser,
        version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-19, a Monday.
    fn monday_19_october() -> i32 {
        (NaiveDate::from_ymd_opt(2026, 10, 19).unwrap() - epoch()).num_days() as i32
    }

    /// 2026-10-21, a Wednesday (the spec's example installation day).
    fn wednesday_21_october() -> i32 {
        monday_19_october() + 2
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
        for ok in ["1.9", "1.10", "0.0", "1.0", "123.456"] {
            assert!(is_version(ok), "{ok}");
        }
        assert_eq!(parse_version("1.10"), Some((1, 10)));
        for bad in [
            "", "1", "1.", ".1", "1.9.0", "1234.1", "1.1234", "v1.9", "1.x", " 1.9", "１.９",
            // A leading zero would name one release twice (`1.09` is `1.9`).
            "007.0", "01.9", "1.09", "1.00", "00.1",
        ] {
            assert!(!is_version(bad), "{bad:?}");
        }
    }

    #[test]
    fn only_versions_near_the_latest_are_counted_under_their_name() {
        // The latest release, those before it in its major, and up to three minors past it
        // keep their own name.
        for (v, latest) in [
            ((1, 10), "1.10"),
            ((1, 13), "1.10"),
            ((1, 0), "1.10"),
            ((1, 9), "1.10"),
        ] {
            assert_eq!(filed_version(v, latest), format!("{}.{}", v.0, v.1));
        }
        // Anything else is filed under `other`: a forged version adds no row of its own.
        for (v, latest) in [
            ((1, 14), "1.10"),
            ((999, 999), "1.10"),
            ((2, 0), "1.10"),
            ((0, 9), "1.10"),
        ] {
            assert_eq!(filed_version(v, latest), OTHER_VERSION, "{v:?}");
        }
        // Moving the setting moves the range; a malformed setting falls back to the default.
        assert_eq!(filed_version((2, 1), "2.0"), "2.1");
        assert_eq!(filed_version((1, 13), "not a version"), "1.13");
        assert_eq!(filed_version((1, 14), "01.20"), OTHER_VERSION);
        // However many versions are sent, a counter's key names at most this many: the
        // latest's major from .0 to three past it, and `other`.
        let named: std::collections::BTreeSet<String> = (0..1000)
            .flat_map(|major| (0..1000).map(move |minor| (major, minor)))
            .map(|v| filed_version(v, DEFAULT_LATEST_VERSION))
            .collect();
        assert_eq!(named.len(), 10 + 1 + MINORS_AHEAD as usize + 1);
    }

    #[test]
    fn a_leaving_answer_week_starts_on_its_monday() {
        let monday = monday_19_october();
        assert_eq!(monday % 7, 4);
        for d in 0..7 {
            assert_eq!(monday_of(monday + d), monday, "day {d} of the week");
        }
        assert_eq!(monday_of(monday + 7), monday + 7);
        // 1970-01-05 is the first Monday; a day before the epoch belongs to 1969-12-29's week.
        assert_eq!(monday_of(4), 4);
        assert_eq!(monday_of(0), -3);
    }

    #[test]
    fn the_window_runs_from_90_days_before_to_the_day_after() {
        let today = wednesday_21_october();
        assert!(day_in_window(today, today));
        // Day 59 of an installation is well inside it.
        assert!(day_in_window(today - 59, today));
        assert!(day_in_window(today - 90, today));
        assert!(!day_in_window(today - 91, today));
        // A device ahead of UTC is already on tomorrow; two days ahead is a wrong clock.
        assert!(day_in_window(today + 1, today));
        assert!(!day_in_window(today + 2, today));
        // The spec's scenario: an installation day 200 days ago.
        assert!(!day_in_window(today - 200, today));
    }

    #[test]
    fn counters_are_kept_25_months() {
        let today = (NaiveDate::from_ymd_opt(2028, 11, 15).unwrap() - epoch()).num_days() as i32;
        let cutoff = retention_cutoff(today);
        assert_eq!(
            cutoff,
            (NaiveDate::from_ymd_opt(2026, 10, 15).unwrap() - epoch()).num_days() as i32
        );
        // A day 26 months old is before the cut-off; one 24 months old is not.
        let old = (NaiveDate::from_ymd_opt(2026, 9, 14).unwrap() - epoch()).num_days() as i32;
        let young = (NaiveDate::from_ymd_opt(2026, 11, 16).unwrap() - epoch()).num_days() as i32;
        assert!(old < cutoff);
        assert!(young >= cutoff);
        // Near the epoch the cut-off is simply a day before it.
        assert!(retention_cutoff(10) < 0);
    }

    #[test]
    fn a_valid_report() {
        let day = wednesday_21_october();
        let r = check_report(day, 2, "1.10", &[2, 1]).unwrap();
        assert_eq!(r.day, day);
        assert_eq!(r.browser, Browser::Firefox);
        assert_eq!(r.version, (1, 10));
        assert_eq!(r.steps, vec![Step::Installed, Step::SetUp]);
        // All six at once.
        assert_eq!(
            check_report(day, 1, "1.9", &[1, 2, 3, 4, 5, 6])
                .unwrap()
                .steps
                .len(),
            6
        );
        // Every day of the week is an installation day: the grain is the day.
        for d in 0..7 {
            let monday = monday_19_october();
            assert_eq!(
                check_report(monday + d, 3, "1.10", &[1]).unwrap().day,
                monday + d
            );
        }
    }

    #[test]
    fn malformed_reports_are_invalid() {
        let day = wednesday_21_october();
        assert_eq!(check_report(day, 1, "1.9", &[]), Err(Invalid::NoStep));
        assert_eq!(
            check_report(day, 1, "1.9", &[1, 2, 3, 4, 5, 6, 1]),
            Err(Invalid::TooManySteps)
        );
        assert_eq!(
            check_report(day, 1, "1.9", &[1, 9]),
            Err(Invalid::UnknownStep)
        );
        assert_eq!(check_report(day, 1, "1.9", &[0]), Err(Invalid::UnknownStep));
        assert_eq!(
            check_report(day, 1, "1.9", &[3, 3]),
            Err(Invalid::RepeatedStep)
        );
        assert_eq!(
            check_report(day, 0, "1.9", &[1]),
            Err(Invalid::UnknownBrowser)
        );
        assert_eq!(
            check_report(day, 4, "1.9", &[1]),
            Err(Invalid::UnknownBrowser)
        );
        assert_eq!(
            check_report(day, 1, "1.9.0", &[1]),
            Err(Invalid::BadVersion)
        );
        assert_eq!(
            check_report(day, 1, "007.0", &[1]),
            Err(Invalid::BadVersion)
        );
        for invalid in [
            Invalid::NoStep,
            Invalid::TooManySteps,
            Invalid::UnknownStep,
            Invalid::RepeatedStep,
            Invalid::UnknownBrowser,
            Invalid::BadVersion,
            Invalid::UnknownReason,
        ] {
            assert!(!invalid.message().is_empty());
        }
    }

    #[test]
    fn leaving_answers_are_bounded() {
        let a = check_leave("slow", "firefox", "1.10").unwrap();
        assert_eq!(a.reason, Reason::Slow);
        assert_eq!(
            (a.browser, a.filed_version("1.10").as_str()),
            ("firefox", "1.10")
        );
        assert_eq!(
            check_leave("slow", "firefox", "999.999")
                .unwrap()
                .filed_version("1.10"),
            OTHER_VERSION
        );
        let opened_by_hand = check_leave("other", "unknown", "unknown").unwrap();
        assert_eq!(
            (
                opened_by_hand.browser,
                opened_by_hand.filed_version("1.10").as_str()
            ),
            ("unknown", "unknown")
        );
        assert_eq!(
            check_leave("slow", "firefox", "1.09"),
            Err(Invalid::BadVersion)
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
