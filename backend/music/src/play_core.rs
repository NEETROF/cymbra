//! Pure, host-testable aggregation for the play heatmap (change: add-play-
//! activity-profile, design D3/D4). The repo fetches a user's raw session points;
//! this module buckets them into per-day activity. No I/O — so the day-bucketing
//! (which is timezone-sensitive) is fully unit-tested.
//!
//! Day bucketing uses the **player's local day** (design D4): each session
//! carries the client's UTC offset at the moment it ended, so a late-night
//! session lands on the player's calendar day, not UTC's.

use chrono::{DateTime, Duration, NaiveDate};
use std::collections::BTreeMap;

use crate::play::{DayActivity, PlayActivity, PracticePoint, SessionPoint};

/// The player's local calendar day for a session: the UTC instant shifted by the
/// client's offset, then truncated to a date.
pub fn local_day(played_at_ms: i64, tz_offset_minutes: i32) -> NaiveDate {
    let utc = DateTime::from_timestamp_millis(played_at_ms).unwrap_or_default();
    (utc + Duration::minutes(tz_offset_minutes as i64)).date_naive()
}

/// Aggregate raw session points into per-day activity (count + average overall
/// synchronization %), ordered by day, plus the songs-played total. Empty input
/// yields no days and a zero total (the heatmap renders those as blank cells).
pub fn aggregate(points: &[SessionPoint]) -> PlayActivity {
    aggregate_with_practice(points, &[])
}

/// Aggregate scored sessions **and** scoreless practice sessions into one per-day
/// grid (change: add-measure-range-practice). A day gets a cell as soon as it has
/// either kind; `avg_sync_pct` is computed from the **scored** sessions only, so
/// practice never drags a day's success colour down (a practice-only day has
/// `count == 0`, which the client renders as a neutral active cell rather than a
/// 0 % failure). Ordered by day; both totals are returned.
pub fn aggregate_with_practice(
    points: &[SessionPoint],
    practices: &[PracticePoint],
) -> PlayActivity {
    // (scored count, running sum of sync %, practice count) per local day;
    // BTreeMap keeps days ordered.
    let mut by_day: BTreeMap<NaiveDate, (u32, f64, u32)> = BTreeMap::new();
    for p in points {
        let day = local_day(p.played_at_ms, p.tz_offset_minutes);
        let entry = by_day.entry(day).or_insert((0, 0.0, 0));
        entry.0 += 1;
        entry.1 += p.overall_sync_pct as f64;
    }
    for p in practices {
        let day = local_day(p.practiced_at_ms, p.tz_offset_minutes);
        by_day.entry(day).or_insert((0, 0.0, 0)).2 += 1;
    }
    let days = by_day
        .into_iter()
        .map(|(day, (count, sum, practice_count))| DayActivity {
            day,
            count,
            // No scored session that day ⇒ no success score to report.
            avg_sync_pct: if count == 0 {
                0.0
            } else {
                (sum / count as f64) as f32
            },
            practice_count,
        })
        .collect();
    PlayActivity {
        days,
        total_sessions: points.len() as u32,
        total_practices: practices.len() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    // 2024-06-15T23:30:00Z as epoch ms.
    const JUN15_2330Z: i64 = 1_718_494_200_000;

    #[test]
    fn utc_offset_shifts_the_day() {
        // At UTC the session is on the 15th...
        assert_eq!(local_day(JUN15_2330Z, 0), ymd(2024, 6, 15));
        // ...but at +60min (e.g. CET) 23:30Z is 00:30 local → the 16th.
        assert_eq!(local_day(JUN15_2330Z, 60), ymd(2024, 6, 16));
        // ...and at -300min (US East) it is 18:30 local → still the 15th.
        assert_eq!(local_day(JUN15_2330Z, -300), ymd(2024, 6, 15));
    }

    #[test]
    fn aggregate_counts_and_averages_per_day() {
        let day = 24 * 3600 * 1000i64;
        let points = vec![
            SessionPoint {
                played_at_ms: JUN15_2330Z,
                tz_offset_minutes: 0,
                overall_sync_pct: 80.0,
            },
            // Same UTC day, different session → averages with the first.
            SessionPoint {
                played_at_ms: JUN15_2330Z - 3600 * 1000,
                tz_offset_minutes: 0,
                overall_sync_pct: 60.0,
            },
            // The next day.
            SessionPoint {
                played_at_ms: JUN15_2330Z + day,
                tz_offset_minutes: 0,
                overall_sync_pct: 100.0,
            },
        ];
        let a = aggregate(&points);
        assert_eq!(a.total_sessions, 3);
        assert_eq!(a.days.len(), 2);
        // Day 1: two sessions, avg (80+60)/2 = 70.
        assert_eq!(a.days[0].day, ymd(2024, 6, 15));
        assert_eq!(a.days[0].count, 2);
        assert!((a.days[0].avg_sync_pct - 70.0).abs() < 1e-4);
        // Day 2: one session at 100.
        assert_eq!(a.days[1].day, ymd(2024, 6, 16));
        assert_eq!(a.days[1].count, 1);
        assert!((a.days[1].avg_sync_pct - 100.0).abs() < 1e-4);
    }

    #[test]
    fn empty_input_is_blank() {
        let a = aggregate(&[]);
        assert_eq!(a.total_sessions, 0);
        assert_eq!(a.total_practices, 0);
        assert!(a.days.is_empty());
    }

    fn practice(at: i64) -> PracticePoint {
        PracticePoint {
            practiced_at_ms: at,
            tz_offset_minutes: 0,
        }
    }

    #[test]
    fn practice_only_day_counts_without_a_success_score() {
        let a = aggregate_with_practice(&[], &[practice(JUN15_2330Z), practice(JUN15_2330Z)]);
        assert_eq!(a.total_sessions, 0);
        assert_eq!(a.total_practices, 2);
        assert_eq!(a.days.len(), 1);
        assert_eq!(a.days[0].day, ymd(2024, 6, 15));
        // The day is active but has NO scored play: no count, no success score —
        // the client renders it neutral, never as a 0 % failure.
        assert_eq!(a.days[0].count, 0);
        assert_eq!(a.days[0].practice_count, 2);
        assert_eq!(a.days[0].avg_sync_pct, 0.0);
    }

    #[test]
    fn practice_does_not_move_a_scored_day_average() {
        let scored = vec![SessionPoint {
            played_at_ms: JUN15_2330Z,
            tz_offset_minutes: 0,
            overall_sync_pct: 80.0,
        }];
        let a = aggregate_with_practice(&scored, &[practice(JUN15_2330Z)]);
        assert_eq!(a.days.len(), 1);
        assert_eq!(a.days[0].count, 1);
        assert_eq!(a.days[0].practice_count, 1);
        // Only the scored session feeds the colour.
        assert!((a.days[0].avg_sync_pct - 80.0).abs() < 1e-4);
    }

    #[test]
    fn practice_and_play_days_merge_in_order() {
        let day = 24 * 3600 * 1000i64;
        let scored = vec![SessionPoint {
            played_at_ms: JUN15_2330Z + day,
            tz_offset_minutes: 0,
            overall_sync_pct: 90.0,
        }];
        let a = aggregate_with_practice(&scored, &[practice(JUN15_2330Z)]);
        assert_eq!(a.days.len(), 2);
        assert_eq!(a.days[0].day, ymd(2024, 6, 15));
        assert_eq!(a.days[0].practice_count, 1);
        assert_eq!(a.days[1].day, ymd(2024, 6, 16));
        assert_eq!(a.days[1].count, 1);
        assert_eq!(a.days[1].practice_count, 0);
    }
}
