//! The Jobs console module (change: add-admin-jobs-console): validates a request with
//! [`admin_core`], asks the [`JobsAdminRepo`] port, and shapes the answer. Host-tested
//! against the mockall-generated `MockJobsAdminRepo`; the Postgres adapter is
//! `pg_admin`.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cymbra_platform::Result;
use uuid::Uuid;

use crate::admin_core::{
    self, AttemptRow, CancelOutcome, FinishedAttempt, HistoryOutcome, JobKind, JobState, JobStats,
    KindPeriodStats, Page, PeriodStats, QueueCounts, QueueRow, QueuedJob, ScheduleRow, Window,
};
use cymbra_jobs::registry;

/// Period figures, their per-kind breakdown and the oldest attempt on record, as the
/// storage returns them. Read together, from one snapshot, so the breakdown adds up to
/// the totals (change: add-jobs-console-history).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PeriodRow {
    pub stats: PeriodStats,
    pub history_since: Option<DateTime<Utc>>,
    pub by_kind: Vec<KindPeriodStats>,
}

/// Storage port for the console (consumer-declared). Every method reads or acts on
/// queue metadata; no signature here can carry a job's payload.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait JobsAdminRepo: Send + Sync {
    /// A page of queued jobs, oldest first.
    async fn list(
        &self,
        state: Option<JobState>,
        kind: Option<String>,
        page: Page,
    ) -> Result<Vec<QueueRow>>;
    /// `(state, count)` for the queue as it is now.
    async fn queue_counts(&self, kind: Option<String>) -> Result<Vec<(String, i64)>>;
    /// Figures over the window, in total and per kind.
    async fn period_stats(&self, window: Window, kind: Option<String>) -> Result<PeriodRow>;
    /// Cancel one job; returns the outcome word of `jobs.admin_cancel`.
    async fn cancel(&self, job_id: Uuid, actor: String, protected: Vec<String>) -> Result<String>;
    /// A page of the attempts that finished in the window, newest first.
    async fn history(
        &self,
        window: Window,
        kind: Option<String>,
        outcome: Option<HistoryOutcome>,
        page: Page,
    ) -> Result<Vec<AttemptRow>>;
    /// How many finished attempts match, across all pages.
    async fn history_count(
        &self,
        window: Window,
        kind: Option<String>,
        outcome: Option<HistoryOutcome>,
    ) -> Result<i64>;
    /// The recurring schedules, without their payload.
    async fn schedules(&self) -> Result<Vec<ScheduleRow>>;
}

/// A page of the queue and the number of jobs matching the filters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobsPage {
    pub jobs: Vec<QueuedJob>,
    pub total: i64,
}

/// What a history request asks for, before validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryQuery<'a> {
    pub from_ms: i64,
    pub to_ms: i64,
    pub kind: &'a str,
    pub outcome: Option<HistoryOutcome>,
    pub limit: i32,
    pub offset: i32,
}

/// A page of finished attempts and the number matching the filters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryPage {
    pub attempts: Vec<FinishedAttempt>,
    pub total: i64,
}

pub struct JobsAdminModule {
    repo: Arc<dyn JobsAdminRepo>,
}

impl JobsAdminModule {
    pub fn new(repo: Arc<dyn JobsAdminRepo>) -> Self {
        Self { repo }
    }

    /// A page of the queue. The total comes from the per-state counts, so an offset
    /// past the end still answers how many jobs match.
    pub async fn list_jobs(
        &self,
        state: Option<JobState>,
        kind: &str,
        limit: i32,
        offset: i32,
    ) -> Result<JobsPage> {
        let page = admin_core::page(limit, offset)?;
        let kind = admin_core::kind_filter(kind);
        let rows = self.repo.list(state, kind.clone(), page).await?;
        let counts = QueueCounts::from_rows(&self.repo.queue_counts(kind).await?);
        let total = state.map_or(counts.total, |state| counts.of(state));
        let jobs = rows
            .into_iter()
            .map(admin_core::queued_job)
            .collect::<Result<_>>()?;
        Ok(JobsPage { jobs, total })
    }

    /// The queue counts now and the figures over `[from_ms, to_ms)`, checked against
    /// the server's `now`.
    pub async fn stats(
        &self,
        from_ms: i64,
        to_ms: i64,
        kind: &str,
        now: DateTime<Utc>,
    ) -> Result<JobStats> {
        let window = admin_core::window(from_ms, to_ms, now)?;
        let kind = admin_core::kind_filter(kind);
        let queue = QueueCounts::from_rows(&self.repo.queue_counts(kind.clone()).await?);
        let period = self.repo.period_stats(window, kind).await?;
        Ok(JobStats {
            queue,
            period: period.stats,
            history_since: period.history_since,
            by_kind: period.by_kind,
        })
    }

    /// A page of the attempts that finished in the query's window, checked against the
    /// server's `now` before storage is touched. The total is a separate count, so an
    /// offset past the end still answers how many attempts match.
    pub async fn history(
        &self,
        query: HistoryQuery<'_>,
        now: DateTime<Utc>,
    ) -> Result<HistoryPage> {
        let window = admin_core::window(query.from_ms, query.to_ms, now)?;
        let page = admin_core::page(query.limit, query.offset)?;
        let kind = admin_core::kind_filter(query.kind);
        let rows = self
            .repo
            .history(window, kind.clone(), query.outcome, page)
            .await?;
        let total = self.repo.history_count(window, kind, query.outcome).await?;
        let attempts = rows
            .into_iter()
            .map(admin_core::finished_attempt)
            .collect::<Result<_>>()?;
        Ok(HistoryPage { attempts, total })
    }

    /// The registered kinds with the schedules that enqueue them, read now: an operator
    /// may change a cadence without a redeploy.
    pub async fn kinds(&self) -> Result<Vec<JobKind>> {
        Ok(admin_core::job_kinds(self.repo.schedules().await?))
    }

    /// Cancel one job on behalf of `actor`. The registry's protected kinds travel with
    /// the call, so the refusal is decided under the same row lock as the delete.
    pub async fn cancel(&self, job_id: &str, actor: &str) -> Result<CancelOutcome> {
        let id = admin_core::parse_job_id(job_id)?;
        let word = self
            .repo
            .cancel(id, actor.to_owned(), registry::protected_kinds())
            .await?;
        CancelOutcome::from_db(&word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use cymbra_jobs::registry::{PURGE_SCORE_OBJECT, PURGE_SOUNDFONT_OBJECT, PURGE_USER};
    use cymbra_platform::AppError;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-09-15T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn ready_row(n: u128) -> QueueRow {
        QueueRow {
            id: Uuid::from_u128(n),
            kind: "verification_email".into(),
            channel: "auth.email".into(),
            state: "ready".into(),
            attempts_made: 0,
            attempts_left: 5,
            created_at: Some(now()),
            attempt_at: Some(now()),
            running_started_at: None,
        }
    }

    fn counts() -> Vec<(String, i64)> {
        vec![("ready".into(), 12), ("running".into(), 3)]
    }

    fn module(repo: MockJobsAdminRepo) -> JobsAdminModule {
        JobsAdminModule::new(Arc::new(repo))
    }

    #[tokio::test]
    async fn a_state_filter_totals_that_state_only() {
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_list()
            .withf(|state, kind, page| {
                *state == Some(JobState::Ready)
                    && kind.as_deref() == Some("verification_email")
                    && *page
                        == Page {
                            limit: 25,
                            offset: 25,
                        }
            })
            .returning(|_, _, _| Ok(vec![ready_row(1)]));
        repo.expect_queue_counts()
            .withf(|kind| kind.as_deref() == Some("verification_email"))
            .returning(|_| Ok(counts()));
        let page = module(repo)
            .list_jobs(Some(JobState::Ready), " verification_email ", 25, 25)
            .await
            .unwrap();
        assert_eq!(page.total, 12);
        assert_eq!(page.jobs.len(), 1);
        assert_eq!(page.jobs[0].id, Uuid::from_u128(1));
    }

    #[tokio::test]
    async fn without_a_state_filter_the_total_is_the_whole_queue() {
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_list()
            .withf(|state, kind, _| state.is_none() && kind.is_none())
            .returning(|_, _, _| Ok(vec![]));
        repo.expect_queue_counts().returning(|_| Ok(counts()));
        let page = module(repo).list_jobs(None, "", 25, 0).await.unwrap();
        assert_eq!(page.total, 15);
        assert!(page.jobs.is_empty());
    }

    #[tokio::test]
    async fn an_invalid_page_never_reaches_the_repo() {
        // No expectations: any repo call would panic.
        let m = module(MockJobsAdminRepo::new());
        assert!(matches!(
            m.list_jobs(None, "", 0, 0).await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            m.list_jobs(None, "", 25, -5).await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn an_unknown_row_state_fails_the_page() {
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_list().returning(|_, _, _| {
            Ok(vec![QueueRow {
                state: "paused".into(),
                ..ready_row(1)
            }])
        });
        repo.expect_queue_counts().returning(|_| Ok(counts()));
        assert!(matches!(
            module(repo).list_jobs(None, "", 25, 0).await,
            Err(AppError::Internal(_))
        ));
    }

    #[tokio::test]
    async fn stats_combine_the_queue_and_the_period() {
        let from = now() - Duration::hours(24);
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_queue_counts().returning(|_| Ok(counts()));
        repo.expect_period_stats()
            .withf(move |window, kind| window.from == from && window.to == now() && kind.is_none())
            .returning(|_, _| {
                Ok(PeriodRow {
                    stats: PeriodStats {
                        completed: 40,
                        failed_attempts: 2,
                        dead_lettered: 1,
                        cancelled: 3,
                        avg_run_ms: Some(850),
                    },
                    history_since: Some(now() - Duration::days(3)),
                    by_kind: vec![KindPeriodStats {
                        kind: "session_reap".into(),
                        stats: PeriodStats {
                            completed: 40,
                            ..PeriodStats::default()
                        },
                        last_finished_at: Some(now()),
                    }],
                })
            });
        let stats = module(repo)
            .stats(from.timestamp_millis(), now().timestamp_millis(), "", now())
            .await
            .unwrap();
        assert_eq!(stats.queue.total, 15);
        assert_eq!(stats.queue.running, 3);
        assert_eq!(stats.period.completed, 40);
        assert_eq!(stats.period.avg_run_ms, Some(850));
        assert_eq!(stats.history_since, Some(now() - Duration::days(3)));
        assert_eq!(stats.by_kind.len(), 1);
        assert_eq!(stats.by_kind[0].kind, "session_reap");
        assert_eq!(stats.by_kind[0].stats.completed, 40);
    }

    #[tokio::test]
    async fn a_bad_window_never_reaches_the_repo() {
        let m = module(MockJobsAdminRepo::new());
        let t = now().timestamp_millis();
        assert!(matches!(
            m.stats(t, t - 1, "", now()).await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn cancel_sends_the_actor_and_every_protected_kind() {
        let id = Uuid::from_u128(7);
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_cancel()
            .withf(move |job_id, actor, protected| {
                *job_id == id
                    && actor == "admin-1"
                    && [PURGE_USER, PURGE_SCORE_OBJECT, PURGE_SOUNDFONT_OBJECT]
                        .iter()
                        .all(|k| protected.iter().any(|p| p == k))
            })
            .returning(|_, _, _| Ok("cancelled".into()));
        let outcome = module(repo)
            .cancel(&id.to_string(), "admin-1")
            .await
            .unwrap();
        assert_eq!(outcome, CancelOutcome::Cancelled);
    }

    #[tokio::test]
    async fn cancel_maps_each_refusal() {
        for (word, expected) in [
            ("gone", CancelOutcome::Gone),
            ("running", CancelOutcome::Running),
            ("protected", CancelOutcome::Protected),
        ] {
            let mut repo = MockJobsAdminRepo::new();
            repo.expect_cancel()
                .returning(move |_, _, _| Ok(word.to_owned()));
            let outcome = module(repo)
                .cancel(&Uuid::nil().to_string(), "admin-1")
                .await
                .unwrap();
            assert_eq!(outcome, expected);
        }
    }

    #[tokio::test]
    async fn a_malformed_job_id_never_reaches_the_repo() {
        assert!(matches!(
            module(MockJobsAdminRepo::new())
                .cancel("not-a-uuid", "admin-1")
                .await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn kinds_come_from_the_registry_with_their_schedules() {
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_schedules().returning(|| {
            Ok(vec![ScheduleRow {
                name: "orphan_reap_hourly".into(),
                kind: "orphan_reap".into(),
                cron: "0 * * * *".into(),
                timezone: "UTC".into(),
                enabled: true,
            }])
        });
        let kinds = module(repo).kinds().await.unwrap();
        assert!(kinds.iter().any(|k| k.name == PURGE_USER && !k.cancellable));
        let reap = kinds.iter().find(|k| k.name == "orphan_reap").unwrap();
        assert_eq!(reap.schedules.len(), 1);
        assert_eq!(reap.schedules[0].cron, "0 * * * *");
    }

    #[tokio::test]
    async fn a_schedule_read_failure_fails_the_kind_list() {
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_schedules()
            .returning(|| Err(AppError::Internal(anyhow::anyhow!("db down"))));
        assert!(matches!(
            module(repo).kinds().await,
            Err(AppError::Internal(_))
        ));
    }

    fn query(limit: i32, offset: i32) -> HistoryQuery<'static> {
        HistoryQuery {
            from_ms: (now() - Duration::hours(24)).timestamp_millis(),
            to_ms: now().timestamp_millis(),
            kind: " session_reap ",
            outcome: Some(HistoryOutcome::Failed),
            limit,
            offset,
        }
    }

    fn attempt(outcome: &str) -> AttemptRow {
        AttemptRow {
            job_id: Uuid::from_u128(4),
            kind: "session_reap".into(),
            channel: "auth.session_reap".into(),
            attempt: 1,
            outcome: outcome.into(),
            started_at: now() - Duration::seconds(3),
            finished_at: now(),
        }
    }

    #[tokio::test]
    async fn history_passes_the_trimmed_filters_and_keeps_a_total_past_the_end() {
        let from = now() - Duration::hours(24);
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_history()
            .withf(move |window, kind, outcome, page| {
                window.from == from
                    && window.to == now()
                    && kind.as_deref() == Some("session_reap")
                    && *outcome == Some(HistoryOutcome::Failed)
                    && *page
                        == Page {
                            limit: 25,
                            offset: 50,
                        }
            })
            .returning(|_, _, _, _| Ok(vec![]));
        repo.expect_history_count()
            .withf(|_, kind, outcome| {
                kind.as_deref() == Some("session_reap") && *outcome == Some(HistoryOutcome::Failed)
            })
            .returning(|_, _, _| Ok(7));
        let page = module(repo).history(query(25, 50), now()).await.unwrap();
        assert!(page.attempts.is_empty());
        assert_eq!(page.total, 7);
    }

    #[tokio::test]
    async fn history_rows_are_shaped() {
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_history()
            .returning(|_, _, _, _| Ok(vec![attempt("failed"), attempt("abandoned")]));
        repo.expect_history_count().returning(|_, _, _| Ok(2));
        let page = module(repo).history(query(25, 0), now()).await.unwrap();
        assert_eq!(page.attempts[0].duration_ms, Some(3_000));
        assert_eq!(page.attempts[1].outcome, HistoryOutcome::Abandoned);
        assert_eq!(page.attempts[1].duration_ms, None);
    }

    #[tokio::test]
    async fn a_running_row_in_the_history_fails_the_page() {
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_history()
            .returning(|_, _, _, _| Ok(vec![attempt("running")]));
        repo.expect_history_count().returning(|_, _, _| Ok(1));
        assert!(matches!(
            module(repo).history(query(25, 0), now()).await,
            Err(AppError::Internal(_))
        ));
    }

    #[tokio::test]
    async fn an_invalid_history_window_or_page_never_reaches_the_repo() {
        // No expectations: any repo call would panic.
        let m = module(MockJobsAdminRepo::new());
        let t = now().timestamp_millis();
        let inverted = HistoryQuery {
            from_ms: t,
            to_ms: t - 1,
            ..query(25, 0)
        };
        let too_old = HistoryQuery {
            from_ms: (now() - Duration::days(120)).timestamp_millis(),
            ..query(25, 0)
        };
        for q in [inverted, too_old, query(0, 0), query(101, 0), query(25, -1)] {
            assert!(matches!(
                m.history(q, now()).await,
                Err(AppError::InvalidArgument(_))
            ));
        }
    }
}
