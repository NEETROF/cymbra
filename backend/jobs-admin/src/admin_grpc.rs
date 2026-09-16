//! Tonic adapter for `JobsAdminService` (change: add-admin-jobs-console) — thin
//! transport, coverage-excluded; the rules live in `admin_core`/`admin`. Every RPC gates
//! on `admin` in the **`global`** scope: the queue carries every product's work
//! (identity emails, account erasure, Music renders, plan reconciliation), so an admin
//! of one product is refused. The actor recorded on a cancellation is the interceptor
//! identity, never a request field.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use chrono::{DateTime, Utc};
use cymbra_platform::{AuthIdentity, GLOBAL_SCOPE, guard::require_admin_in_scope};
use tonic::{Request, Response, Status};

use crate::admin::{HistoryQuery, JobsAdminModule};
use crate::admin_core::{
    CancelOutcome, FinishedAttempt, HistoryOutcome, JobKind, JobState, PeriodStats, QueuedJob,
};
use crate::proto as pb;
use crate::proto::jobs_admin_service_server::{JobsAdminService, JobsAdminServiceServer};

pub struct JobsAdminGrpc {
    module: Arc<JobsAdminModule>,
}

impl JobsAdminGrpc {
    pub fn new(module: Arc<JobsAdminModule>) -> Self {
        Self { module }
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> JobsAdminServiceServer<Self> {
        JobsAdminServiceServer::new(self)
    }
}

/// The caller, gated on `global/admin`. Identity comes only from the interceptor.
fn global_admin<T>(req: &Request<T>) -> Result<AuthIdentity, Status> {
    let id = req
        .extensions()
        .get::<AuthIdentity>()
        .cloned()
        .ok_or_else(|| Status::unauthenticated("missing identity"))?;
    require_admin_in_scope(&id, GLOBAL_SCOPE)?;
    Ok(id)
}

fn state_from_proto(v: i32) -> Result<Option<JobState>, Status> {
    match pb::JobState::try_from(v) {
        Ok(pb::JobState::Unspecified) => Ok(None),
        Ok(pb::JobState::Running) => Ok(Some(JobState::Running)),
        Ok(pb::JobState::Ready) => Ok(Some(JobState::Ready)),
        Ok(pb::JobState::Scheduled) => Ok(Some(JobState::Scheduled)),
        Ok(pb::JobState::RetryWait) => Ok(Some(JobState::RetryWait)),
        Ok(pb::JobState::Blocked) => Ok(Some(JobState::Blocked)),
        Ok(pb::JobState::Exhausted) => Ok(Some(JobState::Exhausted)),
        Err(_) => Err(Status::invalid_argument(format!("unknown job state {v}"))),
    }
}

fn state_to_proto(state: JobState) -> pb::JobState {
    match state {
        JobState::Running => pb::JobState::Running,
        JobState::Ready => pb::JobState::Ready,
        JobState::Scheduled => pb::JobState::Scheduled,
        JobState::RetryWait => pb::JobState::RetryWait,
        JobState::Blocked => pb::JobState::Blocked,
        JobState::Exhausted => pb::JobState::Exhausted,
    }
}

fn outcome_to_proto(outcome: CancelOutcome) -> pb::CancelOutcome {
    match outcome {
        CancelOutcome::Cancelled => pb::CancelOutcome::Cancelled,
        CancelOutcome::Gone => pb::CancelOutcome::Gone,
        CancelOutcome::Running => pb::CancelOutcome::Running,
        CancelOutcome::Protected => pb::CancelOutcome::Protected,
    }
}

fn history_outcome_from_proto(v: i32) -> Result<Option<HistoryOutcome>, Status> {
    match pb::AttemptOutcome::try_from(v) {
        Ok(pb::AttemptOutcome::Unspecified) => Ok(None),
        Ok(pb::AttemptOutcome::Succeeded) => Ok(Some(HistoryOutcome::Succeeded)),
        Ok(pb::AttemptOutcome::Failed) => Ok(Some(HistoryOutcome::Failed)),
        Ok(pb::AttemptOutcome::Abandoned) => Ok(Some(HistoryOutcome::Abandoned)),
        Err(_) => Err(Status::invalid_argument(format!(
            "unknown attempt outcome {v}"
        ))),
    }
}

fn history_outcome_to_proto(outcome: HistoryOutcome) -> pb::AttemptOutcome {
    match outcome {
        HistoryOutcome::Succeeded => pb::AttemptOutcome::Succeeded,
        HistoryOutcome::Failed => pb::AttemptOutcome::Failed,
        HistoryOutcome::Abandoned => pb::AttemptOutcome::Abandoned,
    }
}

fn ms(t: DateTime<Utc>) -> i64 {
    t.timestamp_millis()
}

fn period_to_proto(p: PeriodStats) -> pb::PeriodStats {
    pb::PeriodStats {
        completed: p.completed,
        failed_attempts: p.failed_attempts,
        dead_lettered: p.dead_lettered,
        cancelled: p.cancelled,
        avg_run_ms: p.avg_run_ms,
    }
}

fn kind_to_proto(k: JobKind) -> pb::JobKind {
    pb::JobKind {
        name: k.name,
        channel: k.channel,
        cancellable: k.cancellable,
        schedules: k
            .schedules
            .into_iter()
            .map(|s| pb::JobSchedule {
                name: s.name,
                cron: s.cron,
                timezone: s.timezone,
                enabled: s.enabled,
            })
            .collect(),
    }
}

fn attempt_to_proto(a: FinishedAttempt) -> pb::FinishedAttempt {
    pb::FinishedAttempt {
        job_id: a.job_id.to_string(),
        kind: a.kind,
        channel: a.channel,
        outcome: history_outcome_to_proto(a.outcome) as i32,
        attempt: a.attempt,
        started_at_ms: ms(a.started_at),
        finished_at_ms: ms(a.finished_at),
        duration_ms: a.duration_ms,
    }
}

fn job_to_proto(job: QueuedJob) -> pb::QueuedJob {
    pb::QueuedJob {
        id: job.id.to_string(),
        kind: job.kind,
        channel: job.channel,
        state: state_to_proto(job.state) as i32,
        attempts_made: job.attempts_made,
        attempts_left: job.attempts_left,
        enqueued_at_ms: job.enqueued_at.map_or(0, ms),
        next_attempt_at_ms: job.next_attempt_at.map(ms),
        started_at_ms: job.started_at.map(ms),
        cancellable: job.cancellable,
    }
}

#[tonic::async_trait]
impl JobsAdminService for JobsAdminGrpc {
    async fn admin_list_jobs(
        &self,
        req: Request<pb::AdminListJobsRequest>,
    ) -> Result<Response<pb::AdminListJobsResponse>, Status> {
        global_admin(&req)?;
        let r = req.into_inner();
        let page = self
            .module
            .list_jobs(state_from_proto(r.state)?, &r.kind, r.limit, r.offset)
            .await?;
        Ok(Response::new(pb::AdminListJobsResponse {
            jobs: page.jobs.into_iter().map(job_to_proto).collect(),
            total: page.total,
        }))
    }

    async fn admin_get_job_stats(
        &self,
        req: Request<pb::AdminGetJobStatsRequest>,
    ) -> Result<Response<pb::AdminGetJobStatsResponse>, Status> {
        global_admin(&req)?;
        let r = req.into_inner();
        let window = r
            .window
            .ok_or_else(|| Status::invalid_argument("window is required"))?;
        let stats = self
            .module
            .stats(window.from_ms, window.to_ms, &r.kind, Utc::now())
            .await?;
        let q = stats.queue;
        let p = stats.period;
        Ok(Response::new(pb::AdminGetJobStatsResponse {
            queue: Some(pb::QueueCounts {
                total: q.total,
                running: q.running,
                ready: q.ready,
                scheduled: q.scheduled,
                retry_wait: q.retry_wait,
                blocked: q.blocked,
                exhausted: q.exhausted,
            }),
            period: Some(period_to_proto(p)),
            history_since_ms: stats.history_since.map(ms),
            by_kind: stats
                .by_kind
                .into_iter()
                .map(|k| pb::KindPeriodStats {
                    kind: k.kind,
                    period: Some(period_to_proto(k.stats)),
                    last_finished_at_ms: k.last_finished_at.map(ms),
                })
                .collect(),
        }))
    }

    async fn admin_list_job_kinds(
        &self,
        req: Request<pb::AdminListJobKindsRequest>,
    ) -> Result<Response<pb::AdminListJobKindsResponse>, Status> {
        global_admin(&req)?;
        Ok(Response::new(pb::AdminListJobKindsResponse {
            kinds: self
                .module
                .kinds()
                .await?
                .into_iter()
                .map(kind_to_proto)
                .collect(),
        }))
    }

    async fn admin_list_job_history(
        &self,
        req: Request<pb::AdminListJobHistoryRequest>,
    ) -> Result<Response<pb::AdminListJobHistoryResponse>, Status> {
        global_admin(&req)?;
        let r = req.into_inner();
        let window = r
            .window
            .ok_or_else(|| Status::invalid_argument("window is required"))?;
        let page = self
            .module
            .history(
                HistoryQuery {
                    from_ms: window.from_ms,
                    to_ms: window.to_ms,
                    kind: &r.kind,
                    outcome: history_outcome_from_proto(r.outcome)?,
                    limit: r.limit,
                    offset: r.offset,
                },
                Utc::now(),
            )
            .await?;
        Ok(Response::new(pb::AdminListJobHistoryResponse {
            attempts: page.attempts.into_iter().map(attempt_to_proto).collect(),
            total: page.total,
        }))
    }

    async fn admin_cancel_job(
        &self,
        req: Request<pb::AdminCancelJobRequest>,
    ) -> Result<Response<pb::AdminCancelJobResponse>, Status> {
        let caller = global_admin(&req)?;
        let r = req.into_inner();
        let outcome = self.module.cancel(&r.job_id, &caller.user_id).await?;
        tracing::info!(
            job_id = %r.job_id,
            actor = %caller.user_id,
            outcome = ?outcome,
            "job cancellation requested from the console"
        );
        Ok(Response::new(pb::AdminCancelJobResponse {
            outcome: outcome_to_proto(outcome) as i32,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::{MockJobsAdminRepo, PeriodRow};
    use crate::admin_core::{AttemptRow, KindPeriodStats, QueueRow, ScheduleRow};
    use std::collections::BTreeMap;
    use tonic::Code;

    fn grpc(repo: MockJobsAdminRepo) -> JobsAdminGrpc {
        JobsAdminGrpc::new(Arc::new(JobsAdminModule::new(Arc::new(repo))))
    }

    /// A request carrying an identity with the given `scope -> roles`.
    fn as_caller<T>(msg: T, scopes: &[(&str, &[&str])]) -> Request<T> {
        let mut by_scope = BTreeMap::new();
        for (scope, roles) in scopes {
            by_scope.insert(
                scope.to_string(),
                roles.iter().map(|r| r.to_string()).collect(),
            );
        }
        let mut req = Request::new(msg);
        req.extensions_mut().insert(AuthIdentity {
            user_id: "admin-1".into(),
            audience: "back-office".into(),
            roles: scopes
                .iter()
                .flat_map(|(_, rs)| rs.iter().map(|r| r.to_string()))
                .collect(),
            roles_by_scope: by_scope,
        });
        req
    }

    const GLOBAL_ADMIN: &[(&str, &[&str])] = &[("global", &["admin"])];
    const MUSIC_ADMIN: &[(&str, &[&str])] = &[("music", &["admin"])];

    fn list_request() -> pb::AdminListJobsRequest {
        pb::AdminListJobsRequest {
            state: 0,
            kind: String::new(),
            limit: 25,
            offset: 0,
        }
    }

    #[tokio::test]
    async fn a_global_admin_reads_the_kinds_with_their_schedules() {
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_schedules().returning(|| {
            Ok(vec![ScheduleRow {
                name: "streak_reminder_hourly".into(),
                kind: "streak_reminder".into(),
                cron: "5 * * * *".into(),
                timezone: "UTC".into(),
                enabled: true,
            }])
        });
        let res = grpc(repo)
            .admin_list_job_kinds(as_caller(pb::AdminListJobKindsRequest {}, GLOBAL_ADMIN))
            .await
            .unwrap()
            .into_inner();
        assert!(
            res.kinds
                .iter()
                .any(|k| k.name == "purge_user" && !k.cancellable && k.schedules.is_empty())
        );
        let streak = res
            .kinds
            .iter()
            .find(|k| k.name == "streak_reminder")
            .unwrap();
        assert_eq!(
            streak.schedules,
            vec![pb::JobSchedule {
                name: "streak_reminder_hourly".into(),
                cron: "5 * * * *".into(),
                timezone: "UTC".into(),
                enabled: true,
            }]
        );
    }

    fn history_request() -> pb::AdminListJobHistoryRequest {
        let now = Utc::now().timestamp_millis();
        pb::AdminListJobHistoryRequest {
            window: Some(pb::StatsWindow {
                from_ms: now - 3_600_000,
                to_ms: now,
            }),
            kind: String::new(),
            outcome: pb::AttemptOutcome::Unspecified as i32,
            limit: 25,
            offset: 0,
        }
    }

    #[tokio::test]
    async fn a_global_admin_reads_the_history() {
        let finished = Utc::now();
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_history()
            .withf(|_, kind, outcome, _| {
                kind.as_deref() == Some("verification_email")
                    && *outcome == Some(HistoryOutcome::Abandoned)
            })
            .returning(move |_, _, _, _| {
                Ok(vec![AttemptRow {
                    job_id: uuid::Uuid::from_u128(5),
                    kind: "verification_email".into(),
                    channel: "auth.email".into(),
                    attempt: 3,
                    outcome: "abandoned".into(),
                    started_at: finished - chrono::Duration::seconds(90),
                    finished_at: finished,
                }])
            });
        repo.expect_history_count().returning(|_, _, _| Ok(1));
        let res = grpc(repo)
            .admin_list_job_history(as_caller(
                pb::AdminListJobHistoryRequest {
                    kind: "verification_email".into(),
                    outcome: pb::AttemptOutcome::Abandoned as i32,
                    ..history_request()
                },
                GLOBAL_ADMIN,
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(res.total, 1);
        let a = &res.attempts[0];
        assert_eq!(a.job_id, uuid::Uuid::from_u128(5).to_string());
        assert_eq!(a.outcome, pb::AttemptOutcome::Abandoned as i32);
        assert_eq!(a.attempt, 3);
        assert_eq!(a.finished_at_ms, finished.timestamp_millis());
        assert_eq!(a.duration_ms, None);
    }

    #[tokio::test]
    async fn history_arguments_are_checked_before_storage() {
        // No repo expectations: an invalid request must never reach storage.
        let g = grpc(MockJobsAdminRepo::new());
        for req in [
            pb::AdminListJobHistoryRequest {
                window: None,
                ..history_request()
            },
            pb::AdminListJobHistoryRequest {
                outcome: 99,
                ..history_request()
            },
            pb::AdminListJobHistoryRequest {
                limit: 0,
                ..history_request()
            },
        ] {
            let err = g
                .admin_list_job_history(as_caller(req, GLOBAL_ADMIN))
                .await
                .unwrap_err();
            assert_eq!(err.code(), Code::InvalidArgument);
        }
    }

    #[tokio::test]
    async fn stats_carry_the_per_kind_breakdown() {
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_queue_counts().returning(|_| Ok(vec![]));
        repo.expect_period_stats().returning(|_, _| {
            Ok(PeriodRow {
                stats: PeriodStats {
                    completed: 3,
                    ..PeriodStats::default()
                },
                history_since: None,
                by_kind: vec![KindPeriodStats {
                    kind: "orphan_reap".into(),
                    stats: PeriodStats {
                        completed: 3,
                        avg_run_ms: Some(40),
                        ..PeriodStats::default()
                    },
                    last_finished_at: None,
                }],
            })
        });
        let now = Utc::now().timestamp_millis();
        let res = grpc(repo)
            .admin_get_job_stats(as_caller(
                pb::AdminGetJobStatsRequest {
                    window: Some(pb::StatsWindow {
                        from_ms: now - 3_600_000,
                        to_ms: now,
                    }),
                    kind: String::new(),
                },
                GLOBAL_ADMIN,
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(
            res.by_kind,
            vec![pb::KindPeriodStats {
                kind: "orphan_reap".into(),
                period: Some(pb::PeriodStats {
                    completed: 3,
                    failed_attempts: 0,
                    dead_lettered: 0,
                    cancelled: 0,
                    avg_run_ms: Some(40),
                }),
                last_finished_at_ms: None,
            }]
        );
    }

    #[tokio::test]
    async fn a_product_admin_is_refused_on_every_rpc() {
        // No repo expectations: a refused call must never reach storage.
        let g = grpc(MockJobsAdminRepo::new());
        let codes = [
            g.admin_list_jobs(as_caller(list_request(), MUSIC_ADMIN))
                .await
                .unwrap_err()
                .code(),
            g.admin_get_job_stats(as_caller(
                pb::AdminGetJobStatsRequest {
                    window: None,
                    kind: String::new(),
                },
                MUSIC_ADMIN,
            ))
            .await
            .unwrap_err()
            .code(),
            g.admin_list_job_kinds(as_caller(pb::AdminListJobKindsRequest {}, MUSIC_ADMIN))
                .await
                .unwrap_err()
                .code(),
            g.admin_cancel_job(as_caller(
                pb::AdminCancelJobRequest {
                    job_id: uuid::Uuid::nil().to_string(),
                },
                MUSIC_ADMIN,
            ))
            .await
            .unwrap_err()
            .code(),
            g.admin_list_job_history(as_caller(history_request(), MUSIC_ADMIN))
                .await
                .unwrap_err()
                .code(),
        ];
        assert!(codes.iter().all(|c| *c == Code::PermissionDenied));
    }

    #[tokio::test]
    async fn a_missing_identity_is_unauthenticated() {
        let err = grpc(MockJobsAdminRepo::new())
            .admin_list_jobs(Request::new(list_request()))
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn the_cancelling_actor_is_the_interceptor_identity() {
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_cancel()
            .withf(|_, actor, _| actor == "admin-1")
            .returning(|_, _, _| Ok("cancelled".into()));
        let res = grpc(repo)
            .admin_cancel_job(as_caller(
                pb::AdminCancelJobRequest {
                    job_id: uuid::Uuid::nil().to_string(),
                },
                GLOBAL_ADMIN,
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(res.outcome, pb::CancelOutcome::Cancelled as i32);
    }

    #[tokio::test]
    async fn an_unknown_state_filter_is_invalid() {
        let err = grpc(MockJobsAdminRepo::new())
            .admin_list_jobs(as_caller(
                pb::AdminListJobsRequest {
                    state: 99,
                    ..list_request()
                },
                GLOBAL_ADMIN,
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn stats_require_a_window() {
        let err = grpc(MockJobsAdminRepo::new())
            .admin_get_job_stats(as_caller(
                pb::AdminGetJobStatsRequest {
                    window: None,
                    kind: String::new(),
                },
                GLOBAL_ADMIN,
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn a_running_row_is_mapped_without_cancel_or_next_attempt() {
        let started = Utc::now();
        let mut repo = MockJobsAdminRepo::new();
        repo.expect_list().returning(move |_, _, _| {
            Ok(vec![QueueRow {
                id: uuid::Uuid::from_u128(3),
                kind: "verification_email".into(),
                channel: "auth.email".into(),
                state: "running".into(),
                attempts_made: 2,
                attempts_left: 3,
                created_at: Some(started),
                attempt_at: Some(started),
                running_started_at: Some(started),
            }])
        });
        repo.expect_queue_counts()
            .returning(|_| Ok(vec![("running".into(), 1)]));
        let res = grpc(repo)
            .admin_list_jobs(as_caller(list_request(), GLOBAL_ADMIN))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(res.total, 1);
        let job = &res.jobs[0];
        assert_eq!(job.state, pb::JobState::Running as i32);
        assert_eq!(job.started_at_ms, Some(started.timestamp_millis()));
        assert_eq!(job.next_attempt_at_ms, None);
        assert!(!job.cancellable);
    }

    #[test]
    fn every_state_round_trips_through_the_wire() {
        for state in JobState::ALL {
            assert_eq!(
                state_from_proto(state_to_proto(state) as i32).unwrap(),
                Some(state)
            );
        }
        assert_eq!(state_from_proto(0).unwrap(), None);
    }

    #[test]
    fn every_history_outcome_round_trips_through_the_wire() {
        for outcome in HistoryOutcome::ALL {
            assert_eq!(
                history_outcome_from_proto(history_outcome_to_proto(outcome) as i32).unwrap(),
                Some(outcome)
            );
        }
        assert_eq!(history_outcome_from_proto(0).unwrap(), None);
    }

    #[test]
    fn every_outcome_has_a_wire_value() {
        for (outcome, wire) in [
            (CancelOutcome::Cancelled, pb::CancelOutcome::Cancelled),
            (CancelOutcome::Gone, pb::CancelOutcome::Gone),
            (CancelOutcome::Running, pb::CancelOutcome::Running),
            (CancelOutcome::Protected, pb::CancelOutcome::Protected),
        ] {
            assert_eq!(outcome_to_proto(outcome), wire);
        }
    }
}
