//! Postgres adapter for [`JobsAdminRepo`] (change: add-admin-jobs-console). Thin sqlx
//! glue, coverage-excluded; the rules are host-tested in `admin_core`/`admin` and the
//! SQL in the `#[ignore]` `admin_queue_test`. It runs as `jobs_admin_svc`, which holds
//! EXECUTE on the four `jobs.admin_*` SECURITY DEFINER functions and no table privilege,
//! so every call below goes through one of them — and none returns a payload.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::admin::{JobsAdminRepo, PeriodRow};
use crate::admin_core::{JobState, Page, PeriodStats, QueueRow, Window};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("jobs admin db: {e}"))
}

pub struct PgJobsAdminRepo {
    pool: PgPool,
}

impl PgJobsAdminRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobsAdminRepo for PgJobsAdminRepo {
    async fn list(
        &self,
        state: Option<JobState>,
        kind: Option<String>,
        page: Page,
    ) -> Result<Vec<QueueRow>> {
        let rows = sqlx::query(
            "SELECT id, job_name, channel_name, state, attempts_made, attempts_left, \
                    created_at, attempt_at, running_started_at \
             FROM jobs.admin_list_jobs($1, $2, $3, $4)",
        )
        .bind(state.map(JobState::as_db))
        .bind(kind)
        .bind(page.limit)
        .bind(page.offset)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| QueueRow {
                id: r.get("id"),
                kind: r.get("job_name"),
                channel: r.get("channel_name"),
                state: r.get("state"),
                attempts_made: r.get("attempts_made"),
                attempts_left: r.get("attempts_left"),
                created_at: r.get("created_at"),
                attempt_at: r.get("attempt_at"),
                running_started_at: r.get("running_started_at"),
            })
            .collect())
    }

    async fn queue_counts(&self, kind: Option<String>) -> Result<Vec<(String, i64)>> {
        let rows = sqlx::query("SELECT state, n FROM jobs.admin_queue_counts($1)")
            .bind(kind)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| (r.get("state"), r.get("n")))
            .collect())
    }

    async fn period_stats(&self, window: Window, kind: Option<String>) -> Result<PeriodRow> {
        let r = sqlx::query(
            "SELECT completed, failed_attempts, dead_lettered, cancelled, avg_run_ms, \
                    history_since \
             FROM jobs.admin_period_stats($1, $2, $3)",
        )
        .bind(window.from)
        .bind(window.to)
        .bind(kind)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(PeriodRow {
            stats: PeriodStats {
                completed: r.get("completed"),
                failed_attempts: r.get("failed_attempts"),
                dead_lettered: r.get("dead_lettered"),
                cancelled: r.get("cancelled"),
                avg_run_ms: r.get("avg_run_ms"),
            },
            history_since: r.get("history_since"),
        })
    }

    async fn cancel(&self, job_id: Uuid, actor: String, protected: Vec<String>) -> Result<String> {
        sqlx::query_scalar("SELECT jobs.admin_cancel($1, $2, $3)")
            .bind(job_id)
            .bind(actor)
            .bind(protected)
            .fetch_one(&self.pool)
            .await
            .map_err(internal)
    }
}
