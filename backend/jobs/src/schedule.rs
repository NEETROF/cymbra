//! Recurring-schedule core (design D5; tasks 5.1–5.4). Pure and host-testable:
//! cron + timezone due-time computation, the idempotent time-bucket dedup key,
//! and the per-schedule missed-run policy. The worker's scheduler calls these to
//! decide *which* occurrences to enqueue; the actual enqueue (with
//! `ON CONFLICT DO NOTHING` on `jobs.schedule_occurrences`) is engine glue, so a
//! singleton job is created per occurrence even across replicas.

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::error::JobError;

/// What to do when the worker was down across one or more scheduled times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissedRun {
    /// Collapse all missed occurrences into a single run (the latest one).
    Skip,
    /// Enqueue every missed occurrence.
    CatchUp,
}

impl MissedRun {
    /// Parse the `jobs.schedules.missed_run_policy` column.
    pub fn parse(s: &str) -> Result<Self, JobError> {
        match s {
            "skip" => Ok(MissedRun::Skip),
            "catch_up" => Ok(MissedRun::CatchUp),
            other => Err(JobError::InvalidSchedule {
                name: String::new(),
                reason: format!("unknown missed_run_policy {other:?}"),
            }),
        }
    }
}

/// A parsed schedule: a cron expression bound to a timezone, plus the enabled
/// flag and missed-run policy from `jobs.schedules`.
#[derive(Debug, Clone)]
pub struct Schedule {
    name: String,
    schedule: cron::Schedule,
    tz: Tz,
    enabled: bool,
    missed_run: MissedRun,
}

/// Safety bound on how many occurrences a single evaluation will materialize,
/// so a wide window against a frequent cron can't run away.
const MAX_OCCURRENCES: usize = 10_000;

impl Schedule {
    /// Parse a schedule. Accepts standard 5-field cron (`m h dom mon dow`) as
    /// well as the 6/7-field form the `cron` crate expects, by defaulting the
    /// seconds field to `0` when omitted.
    pub fn parse(
        name: impl Into<String>,
        cron_expr: &str,
        timezone: &str,
        enabled: bool,
        missed_run: MissedRun,
    ) -> Result<Self, JobError> {
        let name = name.into();
        let tz: Tz = timezone.parse().map_err(|_| JobError::InvalidSchedule {
            name: name.clone(),
            reason: format!("unknown timezone {timezone:?}"),
        })?;
        let normalized = normalize_cron(cron_expr);
        let schedule =
            normalized
                .parse::<cron::Schedule>()
                .map_err(|e| JobError::InvalidSchedule {
                    name: name.clone(),
                    reason: format!("invalid cron {cron_expr:?}: {e}"),
                })?;
        Ok(Self {
            name,
            schedule,
            tz,
            enabled,
            missed_run,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// All scheduled times `t` with `after < t <= now`, in chronological order
    /// (UTC). Empty when the schedule is disabled (task 5.4).
    pub fn due_occurrences(&self, after: DateTime<Utc>, now: DateTime<Utc>) -> Vec<DateTime<Utc>> {
        if !self.enabled || after >= now {
            return Vec::new();
        }
        let after_tz = after.with_timezone(&self.tz);
        let now_tz = now.with_timezone(&self.tz);
        let mut out = Vec::new();
        for t in self.schedule.after(&after_tz) {
            if t > now_tz {
                break;
            }
            out.push(t.with_timezone(&Utc));
            if out.len() >= MAX_OCCURRENCES {
                break;
            }
        }
        out
    }

    /// The occurrences to actually enqueue this evaluation, after applying the
    /// missed-run policy (task 5.3): `CatchUp` enqueues every due occurrence;
    /// `Skip` collapses them to the most recent.
    pub fn occurrences_to_enqueue(
        &self,
        after: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Vec<DateTime<Utc>> {
        let due = self.due_occurrences(after, now);
        match self.missed_run {
            MissedRun::CatchUp => due,
            MissedRun::Skip => due.into_iter().last().into_iter().collect(),
        }
    }
}

/// Normalize a cron expression to the 6-field form the `cron` crate parses:
/// a bare 5-field crontab gets a leading `0` seconds field.
fn normalize_cron(expr: &str) -> String {
    let fields = expr.split_whitespace().count();
    if fields == 5 {
        format!("0 {}", expr.trim())
    } else {
        expr.trim().to_string()
    }
}

/// The idempotency bucket for a scheduled occurrence: its exact instant as a
/// unix timestamp. Two replicas computing the same occurrence land on the same
/// bucket, so the `dedup_key` collides and only one job is enqueued.
pub fn bucket(occurrence: DateTime<Utc>) -> i64 {
    occurrence.timestamp()
}

/// The dedup key written to `jobs.schedule_occurrences`: `"<name>:<bucket>"`.
pub fn dedup_key(schedule_name: &str, bucket: i64) -> String {
    format!("{schedule_name}:{bucket}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn five_field_cron_is_normalized_and_parses() {
        // every day at 03:00
        let s = Schedule::parse("nightly", "0 3 * * *", "UTC", true, MissedRun::Skip).unwrap();
        let due = s.due_occurrences(utc("2026-06-01T00:00:00Z"), utc("2026-06-03T12:00:00Z"));
        assert_eq!(
            due,
            vec![
                utc("2026-06-01T03:00:00Z"),
                utc("2026-06-02T03:00:00Z"),
                utc("2026-06-03T03:00:00Z"),
            ]
        );
    }

    #[test]
    fn timezone_is_honored() {
        // 00:30 in New York is 04:30 or 05:30 UTC depending on DST; pick a winter
        // date (EST = UTC-5) → 05:30 UTC.
        let s = Schedule::parse(
            "tz",
            "30 0 * * *",
            "America/New_York",
            true,
            MissedRun::Skip,
        )
        .unwrap();
        let due = s.due_occurrences(utc("2026-01-10T00:00:00Z"), utc("2026-01-11T00:00:00Z"));
        assert_eq!(due, vec![utc("2026-01-10T05:30:00Z")]);
    }

    #[test]
    fn disabled_schedule_yields_nothing() {
        let s = Schedule::parse("off", "0 3 * * *", "UTC", false, MissedRun::CatchUp).unwrap();
        assert!(
            s.due_occurrences(utc("2026-06-01T00:00:00Z"), utc("2026-06-30T00:00:00Z"))
                .is_empty()
        );
        assert!(!s.enabled());
    }

    #[test]
    fn skip_collapses_to_latest_catchup_keeps_all() {
        let after = utc("2026-06-01T00:00:00Z");
        let now = utc("2026-06-04T12:00:00Z"); // three 03:00 occurrences missed

        let skip = Schedule::parse("s", "0 3 * * *", "UTC", true, MissedRun::Skip).unwrap();
        assert_eq!(
            skip.occurrences_to_enqueue(after, now),
            vec![utc("2026-06-04T03:00:00Z")]
        );

        let catch = Schedule::parse("c", "0 3 * * *", "UTC", true, MissedRun::CatchUp).unwrap();
        assert_eq!(catch.occurrences_to_enqueue(after, now).len(), 4);
    }

    #[test]
    fn dedup_key_is_singleton_per_occurrence() {
        let occ = utc("2026-06-04T03:00:00Z");
        let b = bucket(occ);
        assert_eq!(b, occ.timestamp());
        // Two independent evaluations of the same occurrence produce the same key.
        assert_eq!(dedup_key("reap", b), dedup_key("reap", bucket(occ)));
        assert_eq!(dedup_key("reap", b), format!("reap:{}", occ.timestamp()));
        // Different occurrences differ.
        assert_ne!(
            dedup_key("reap", b),
            dedup_key("reap", bucket(utc("2026-06-05T03:00:00Z")))
        );
    }

    #[test]
    fn empty_window_and_bad_inputs() {
        let s = Schedule::parse("x", "0 3 * * *", "UTC", true, MissedRun::Skip).unwrap();
        // after >= now → nothing.
        assert!(
            s.due_occurrences(utc("2026-06-02T00:00:00Z"), utc("2026-06-01T00:00:00Z"))
                .is_empty()
        );
        // Bad timezone / cron / policy.
        assert!(Schedule::parse("x", "0 3 * * *", "Mars/Olympus", true, MissedRun::Skip).is_err());
        assert!(Schedule::parse("x", "not a cron", "UTC", true, MissedRun::Skip).is_err());
        assert!(MissedRun::parse("nonsense").is_err());
        assert_eq!(MissedRun::parse("skip").unwrap(), MissedRun::Skip);
        assert_eq!(MissedRun::parse("catch_up").unwrap(), MissedRun::CatchUp);
    }
}
