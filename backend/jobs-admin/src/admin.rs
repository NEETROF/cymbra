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
    self, CancelOutcome, JobKind, JobState, JobStats, Page, PeriodStats, QueueCounts, QueueRow,
    QueuedJob, Window,
};
use cymbra_jobs::registry;

/// Period figures plus the oldest attempt on record, as the storage returns them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PeriodRow {
    pub stats: PeriodStats,
    pub history_since: Option<DateTime<Utc>>,
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
    /// Figures over the window.
    async fn period_stats(&self, window: Window, kind: Option<String>) -> Result<PeriodRow>;
    /// Cancel one job; returns the outcome word of `jobs.admin_cancel`.
    async fn cancel(&self, job_id: Uuid, actor: String, protected: Vec<String>) -> Result<String>;
}

/// A page of the queue and the number of jobs matching the filters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobsPage {
    pub jobs: Vec<QueuedJob>,
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
        })
    }

    pub fn kinds(&self) -> Vec<JobKind> {
        admin_core::job_kinds()
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

    #[test]
    fn kinds_come_from_the_registry() {
        let kinds = module(MockJobsAdminRepo::new()).kinds();
        assert!(kinds.iter().any(|k| k.name == PURGE_USER && !k.cancellable));
    }
}
