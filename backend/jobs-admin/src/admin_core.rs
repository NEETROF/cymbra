//! Pure rules for the back-office Jobs console (change: add-admin-jobs-console),
//! host-tested: request validation (page, window, filters, job id), the state and
//! outcome vocabularies shared with the `jobs.admin_*` SQL functions, per-row
//! cancellability, and the shaping of per-state counts.
//!
//! The queue state itself is derived once, in the `jobs.admin_queue` view (design D2):
//! filtering, paging and counting by state all have to happen server-side, so the rule
//! cannot live here. This module only names the states the view emits.

use chrono::{DateTime, Duration, Utc};
use cymbra_platform::{AppError, Result};
use uuid::Uuid;

use cymbra_jobs::attempt::HISTORY_RETENTION_DAYS;
use cymbra_jobs::registry;

/// Largest page the console may request.
pub const MAX_PAGE_SIZE: i32 = 100;

/// How far the caller's clock may run ahead of the server's. A preset window ends at
/// the browser's "now", which can be a little later than the server's; refusing it
/// would fail the page for an operator whose laptop clock drifts by a few seconds.
const CLOCK_SKEW: Duration = Duration::minutes(5);

/// Where a queued job stands — the `state` column of `jobs.admin_queue`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JobState {
    Running,
    Ready,
    Scheduled,
    RetryWait,
    Blocked,
    Exhausted,
}

impl JobState {
    pub const ALL: [JobState; 6] = [
        JobState::Running,
        JobState::Ready,
        JobState::Scheduled,
        JobState::RetryWait,
        JobState::Blocked,
        JobState::Exhausted,
    ];

    /// The word the view emits for this state.
    pub fn as_db(self) -> &'static str {
        match self {
            JobState::Running => "running",
            JobState::Ready => "ready",
            JobState::Scheduled => "scheduled",
            JobState::RetryWait => "retry_wait",
            JobState::Blocked => "blocked",
            JobState::Exhausted => "exhausted",
        }
    }

    pub fn from_db(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|state| state.as_db() == s)
    }
}

/// A validated page request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Page {
    pub limit: i32,
    pub offset: i32,
}

pub fn page(limit: i32, offset: i32) -> Result<Page> {
    if !(1..=MAX_PAGE_SIZE).contains(&limit) {
        return Err(AppError::InvalidArgument(format!(
            "limit must be between 1 and {MAX_PAGE_SIZE}"
        )));
    }
    if offset < 0 {
        return Err(AppError::InvalidArgument(
            "offset must not be negative".into(),
        ));
    }
    Ok(Page { limit, offset })
}

/// A validated half-open window `[from, to)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

/// Validate a window given in epoch milliseconds against the server's `now`: it must
/// start before it ends, not end in the future, and start within the history
/// retention — before that the figures are gone, and answering zero would be a lie.
/// An end at most [`CLOCK_SKEW`] ahead is clamped to `now` rather than refused.
pub fn window(from_ms: i64, to_ms: i64, now: DateTime<Utc>) -> Result<Window> {
    let at = |ms: i64| {
        DateTime::<Utc>::from_timestamp_millis(ms)
            .ok_or_else(|| AppError::InvalidArgument(format!("timestamp {ms} is out of range")))
    };
    let (from, to) = (at(from_ms)?, at(to_ms)?);
    if from >= to {
        return Err(AppError::InvalidArgument(
            "the window must start before it ends".into(),
        ));
    }
    if to > now + CLOCK_SKEW {
        return Err(AppError::InvalidArgument(
            "the window must not end in the future".into(),
        ));
    }
    if from < now - Duration::days(HISTORY_RETENTION_DAYS) - CLOCK_SKEW {
        return Err(AppError::InvalidArgument(format!(
            "the window must start within the last {HISTORY_RETENTION_DAYS} days"
        )));
    }
    let to = to.min(now);
    if from >= to {
        return Err(AppError::InvalidArgument(
            "the window must start before it ends".into(),
        ));
    }
    Ok(Window { from, to })
}

/// A kind filter: blank means every kind.
pub fn kind_filter(kind: &str) -> Option<String> {
    let kind = kind.trim();
    (!kind.is_empty()).then(|| kind.to_owned())
}

pub fn parse_job_id(s: &str) -> Result<Uuid> {
    Uuid::parse_str(s.trim())
        .map_err(|_| AppError::InvalidArgument(format!("invalid job id {s:?}")))
}

/// Whether jobs of this kind may ever be cancelled. A kind the registry does not know
/// is cancellable: no handler would run it anyway, so removing it loses nothing.
pub fn kind_cancellable(kind: &str) -> bool {
    registry::spec(kind).is_none_or(|spec| spec.cancellable())
}

/// Whether the console offers Cancel on a row. The server re-checks both conditions
/// under the row lock; this only decides what the page shows.
pub fn cancellable(kind: &str, state: JobState) -> bool {
    state != JobState::Running && kind_cancellable(kind)
}

/// One queued job as `jobs.admin_list_jobs` returns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueRow {
    pub id: Uuid,
    pub kind: String,
    pub channel: String,
    pub state: String,
    pub attempts_made: i32,
    pub attempts_left: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub attempt_at: Option<DateTime<Utc>>,
    pub running_started_at: Option<DateTime<Utc>>,
}

/// One queued job as the console shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedJob {
    pub id: Uuid,
    pub kind: String,
    pub channel: String,
    pub state: JobState,
    pub attempts_made: i32,
    pub attempts_left: i32,
    pub enqueued_at: Option<DateTime<Utc>>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub cancellable: bool,
}

pub fn queued_job(row: QueueRow) -> Result<QueuedJob> {
    let state = JobState::from_db(&row.state)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("unknown job state {:?}", row.state)))?;
    // While an attempt runs, `attempt_at` is the lease the runner's keep-alive extends,
    // not a planned attempt; an exhausted job has none left.
    let next_attempt_at = match state {
        JobState::Running | JobState::Exhausted => None,
        _ => row.attempt_at,
    };
    let started_at = if state == JobState::Running {
        row.running_started_at
    } else {
        None
    };
    Ok(QueuedJob {
        cancellable: cancellable(&row.kind, state),
        id: row.id,
        kind: row.kind,
        channel: row.channel,
        state,
        attempts_made: row.attempts_made,
        attempts_left: row.attempts_left,
        enqueued_at: row.created_at,
        next_attempt_at,
        started_at,
    })
}

/// The queue as it is now, per state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueueCounts {
    pub total: i64,
    pub running: i64,
    pub ready: i64,
    pub scheduled: i64,
    pub retry_wait: i64,
    pub blocked: i64,
    pub exhausted: i64,
}

impl QueueCounts {
    /// Fold `(state, count)` rows. A state this build does not know still counts toward
    /// the total, so the total never under-reports what is in the queue.
    pub fn from_rows(rows: &[(String, i64)]) -> Self {
        rows.iter().fold(Self::default(), |mut counts, (state, n)| {
            counts.total += n;
            match JobState::from_db(state) {
                Some(JobState::Running) => counts.running += n,
                Some(JobState::Ready) => counts.ready += n,
                Some(JobState::Scheduled) => counts.scheduled += n,
                Some(JobState::RetryWait) => counts.retry_wait += n,
                Some(JobState::Blocked) => counts.blocked += n,
                Some(JobState::Exhausted) => counts.exhausted += n,
                None => {}
            }
            counts
        })
    }

    pub fn of(&self, state: JobState) -> i64 {
        match state {
            JobState::Running => self.running,
            JobState::Ready => self.ready,
            JobState::Scheduled => self.scheduled,
            JobState::RetryWait => self.retry_wait,
            JobState::Blocked => self.blocked,
            JobState::Exhausted => self.exhausted,
        }
    }
}

/// Figures over a window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PeriodStats {
    pub completed: i64,
    pub failed_attempts: i64,
    pub dead_lettered: i64,
    pub cancelled: i64,
    pub avg_run_ms: Option<i64>,
}

/// Everything the console's cards show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobStats {
    pub queue: QueueCounts,
    pub period: PeriodStats,
    /// The oldest attempt on record; before it the figures are unknown.
    pub history_since: Option<DateTime<Utc>>,
}

/// The result of a cancellation request — the words `jobs.admin_cancel` returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelOutcome {
    Cancelled,
    Gone,
    Running,
    Protected,
}

impl CancelOutcome {
    pub fn from_db(word: &str) -> Result<Self> {
        match word {
            "cancelled" => Ok(Self::Cancelled),
            "gone" => Ok(Self::Gone),
            "running" => Ok(Self::Running),
            "protected" => Ok(Self::Protected),
            other => Err(AppError::Internal(anyhow::anyhow!(
                "unknown cancellation outcome {other:?}"
            ))),
        }
    }
}

/// A registered job kind, for the console's filter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobKind {
    pub name: String,
    pub channel: String,
    pub cancellable: bool,
}

/// Every registered kind, sorted by name.
pub fn job_kinds() -> Vec<JobKind> {
    let mut kinds: Vec<JobKind> = registry::builtin()
        .into_iter()
        .map(|spec| JobKind {
            name: spec.name().to_owned(),
            channel: spec.channel().name(),
            cancellable: spec.cancellable(),
        })
        .collect();
    kinds.sort_by(|a, b| a.name.cmp(&b.name));
    kinds
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_jobs::registry::{PURGE_USER, VERIFICATION_EMAIL};

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-09-15T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn ms(t: DateTime<Utc>) -> i64 {
        t.timestamp_millis()
    }

    fn row(kind: &str, state: &str) -> QueueRow {
        QueueRow {
            id: Uuid::nil(),
            kind: kind.into(),
            channel: "auth.email".into(),
            state: state.into(),
            attempts_made: 1,
            attempts_left: 4,
            created_at: Some(now() - Duration::minutes(10)),
            attempt_at: Some(now() + Duration::seconds(30)),
            running_started_at: Some(now() - Duration::seconds(5)),
        }
    }

    #[test]
    fn page_accepts_the_bounds_and_refuses_beyond() {
        assert_eq!(
            page(1, 0).unwrap(),
            Page {
                limit: 1,
                offset: 0
            }
        );
        assert_eq!(page(100, 50).unwrap().limit, 100);
        assert!(matches!(page(0, 0), Err(AppError::InvalidArgument(_))));
        assert!(matches!(page(101, 0), Err(AppError::InvalidArgument(_))));
        assert!(matches!(page(25, -1), Err(AppError::InvalidArgument(_))));
    }

    #[test]
    fn window_accepts_a_preset_ending_now() {
        let w = window(ms(now() - Duration::hours(24)), ms(now()), now()).unwrap();
        assert_eq!(w.to, now());
        assert_eq!(w.from, now() - Duration::hours(24));
    }

    #[test]
    fn window_refuses_an_inverted_or_empty_range() {
        let t = ms(now());
        assert!(matches!(
            window(t, t, now()),
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            window(t, t - 1_000, now()),
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[test]
    fn window_refuses_the_future_but_clamps_a_small_skew() {
        let from = ms(now() - Duration::hours(1));
        let skewed = window(from, ms(now() + Duration::seconds(20)), now()).unwrap();
        assert_eq!(skewed.to, now());
        assert!(matches!(
            window(from, ms(now() + Duration::hours(1)), now()),
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[test]
    fn window_refuses_a_start_beyond_the_retention() {
        let to = ms(now());
        assert!(window(ms(now() - Duration::days(30)), to, now()).is_ok());
        assert!(window(ms(now() - Duration::days(90)), to, now()).is_ok());
        assert!(matches!(
            window(ms(now() - Duration::days(91)), to, now()),
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[test]
    fn window_refuses_an_unrepresentable_timestamp() {
        assert!(matches!(
            window(i64::MIN, ms(now()), now()),
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[test]
    fn kind_filter_treats_blank_as_every_kind() {
        assert_eq!(kind_filter(""), None);
        assert_eq!(kind_filter("   "), None);
        assert_eq!(
            kind_filter(" verification_email "),
            Some("verification_email".into())
        );
    }

    #[test]
    fn job_id_must_be_a_uuid() {
        assert_eq!(
            parse_job_id("00000000-0000-0000-0000-000000000000").unwrap(),
            Uuid::nil()
        );
        assert!(matches!(
            parse_job_id("42"),
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[test]
    fn states_round_trip_through_their_view_words() {
        for state in JobState::ALL {
            assert_eq!(JobState::from_db(state.as_db()), Some(state));
        }
        assert_eq!(JobState::from_db("paused"), None);
    }

    #[test]
    fn a_running_row_shows_its_start_and_no_next_attempt() {
        let job = queued_job(row(VERIFICATION_EMAIL, "running")).unwrap();
        assert_eq!(job.state, JobState::Running);
        assert_eq!(job.next_attempt_at, None);
        assert_eq!(job.started_at, Some(now() - Duration::seconds(5)));
        assert!(!job.cancellable);
    }

    #[test]
    fn a_waiting_row_shows_its_next_attempt_and_can_be_cancelled() {
        let job = queued_job(row(VERIFICATION_EMAIL, "retry_wait")).unwrap();
        assert_eq!(job.next_attempt_at, Some(now() + Duration::seconds(30)));
        assert_eq!(job.started_at, None);
        assert!(job.cancellable);
        assert_eq!(job.enqueued_at, Some(now() - Duration::minutes(10)));
    }

    #[test]
    fn an_exhausted_row_has_no_next_attempt() {
        let job = queued_job(row(VERIFICATION_EMAIL, "exhausted")).unwrap();
        assert_eq!(job.next_attempt_at, None);
        assert!(job.cancellable);
    }

    #[test]
    fn an_erasure_row_is_never_cancellable() {
        let job = queued_job(row(PURGE_USER, "ready")).unwrap();
        assert!(!job.cancellable);
    }

    #[test]
    fn an_unregistered_kind_is_cancellable() {
        assert!(kind_cancellable("retired_job"));
        assert!(cancellable("retired_job", JobState::Blocked));
        assert!(!cancellable("retired_job", JobState::Running));
    }

    #[test]
    fn an_unknown_state_is_an_internal_error() {
        assert!(matches!(
            queued_job(row(VERIFICATION_EMAIL, "paused")),
            Err(AppError::Internal(_))
        ));
    }

    #[test]
    fn counts_fold_per_state_and_keep_unknown_rows_in_the_total() {
        let counts = QueueCounts::from_rows(&[
            ("running".into(), 3),
            ("ready".into(), 12),
            ("scheduled".into(), 1),
            ("retry_wait".into(), 2),
            ("blocked".into(), 4),
            ("exhausted".into(), 1),
            ("paused".into(), 5),
        ]);
        assert_eq!(counts.total, 28);
        assert_eq!(counts.of(JobState::Running), 3);
        assert_eq!(counts.of(JobState::Ready), 12);
        assert_eq!(counts.of(JobState::Scheduled), 1);
        assert_eq!(counts.of(JobState::RetryWait), 2);
        assert_eq!(counts.of(JobState::Blocked), 4);
        assert_eq!(counts.of(JobState::Exhausted), 1);
        assert_eq!(QueueCounts::from_rows(&[]), QueueCounts::default());
    }

    #[test]
    fn cancellation_words_map_to_outcomes() {
        assert_eq!(
            CancelOutcome::from_db("cancelled").unwrap(),
            CancelOutcome::Cancelled
        );
        assert_eq!(CancelOutcome::from_db("gone").unwrap(), CancelOutcome::Gone);
        assert_eq!(
            CancelOutcome::from_db("running").unwrap(),
            CancelOutcome::Running
        );
        assert_eq!(
            CancelOutcome::from_db("protected").unwrap(),
            CancelOutcome::Protected
        );
        assert!(matches!(
            CancelOutcome::from_db("maybe"),
            Err(AppError::Internal(_))
        ));
    }

    #[test]
    fn kinds_are_sorted_and_flag_erasure() {
        let kinds = job_kinds();
        assert!(kinds.windows(2).all(|w| w[0].name <= w[1].name));
        let purge = kinds.iter().find(|k| k.name == PURGE_USER).unwrap();
        assert!(!purge.cancellable);
        assert_eq!(purge.channel, "user.purge");
        let email = kinds.iter().find(|k| k.name == VERIFICATION_EMAIL).unwrap();
        assert!(email.cancellable);
    }
}
