// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Postgres-backed [`ContentReportRepo`] (change: add-content-reporting) — thin
//! I/O glue (excluded from the coverage gate; exercised by the integration
//! tests). The de-duplication tested against the fake is mirrored in SQL by the
//! partial unique index from migration 0032: the insert is an
//! `ON CONFLICT DO NOTHING` and, when it hits, the existing open report's id is
//! read back — so a double tap returns the first id instead of raising.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::content_report::{ContentReportRepo, NewReport, ReportReason, ReportRow, ReportTarget};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("content report db: {e}"))
}

/// Postgres implementation over the `music_svc` pool (search_path = `music`).
pub struct PgContentReportRepo {
    pool: PgPool,
}

impl PgContentReportRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Build a row from a query result. The stored vocabulary is written by this
/// crate and constrained by the migration's CHECKs, so an unparseable value is a
/// schema bug, not user input — it surfaces as `Internal` rather than being
/// silently mapped to a default.
fn row_of(r: &sqlx::postgres::PgRow) -> Result<ReportRow> {
    let kind: String = r.try_get("target_kind").map_err(internal)?;
    let reason: String = r.try_get("reason").map_err(internal)?;
    let created: chrono::DateTime<chrono::Utc> = r.try_get("created_at").map_err(internal)?;
    let reporter: Option<uuid::Uuid> = r.try_get("reporter_id").map_err(internal)?;
    Ok(ReportRow {
        id: r
            .try_get::<uuid::Uuid, _>("id")
            .map_err(internal)?
            .to_string(),
        target: ReportTarget::parse(&kind)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("stored target_kind: {e}")))?,
        target_id: r.try_get("target_id").map_err(internal)?,
        reporter_id: reporter.map(|u| u.to_string()),
        reason: ReportReason::parse(&reason)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("stored reason: {e}")))?,
        note: r.try_get("note").map_err(internal)?,
        created_at: created.timestamp_millis(),
    })
}

#[async_trait]
impl ContentReportRepo for PgContentReportRepo {
    async fn insert(&self, reporter_id: &str, report: &NewReport) -> Result<String> {
        let reporter = uuid::Uuid::parse_str(reporter_id)
            .map_err(|_| AppError::InvalidArgument("reporter_id is not a uuid".into()))?;
        let id = uuid::Uuid::now_v7();
        let inserted: Option<uuid::Uuid> = sqlx::query_scalar(
            "INSERT INTO music.content_reports \
               (id, target_kind, target_id, reporter_id, reason, note) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT DO NOTHING \
             RETURNING id",
        )
        .bind(id)
        .bind(report.target.as_str())
        .bind(&report.target_id)
        .bind(reporter)
        .bind(report.reason.as_str())
        .bind(report.note.as_deref())
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;

        if let Some(new_id) = inserted {
            return Ok(new_id.to_string());
        }
        // The partial unique index refused it: an open report by this reporter
        // against this target already stands. Return ITS id — the reporter has
        // done their part, and an error here would read as "your report failed".
        let existing: uuid::Uuid = sqlx::query_scalar(
            "SELECT id FROM music.content_reports \
             WHERE target_kind = $1 AND target_id = $2 AND reporter_id = $3 AND status = 'open'",
        )
        .bind(report.target.as_str())
        .bind(&report.target_id)
        .bind(reporter)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(existing.to_string())
    }

    async fn list_open(&self, limit: i64) -> Result<Vec<ReportRow>> {
        let rows = sqlx::query(
            "SELECT id, target_kind, target_id, reporter_id, reason, note, created_at \
             FROM music.content_reports \
             WHERE status = 'open' \
             ORDER BY created_at \
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        rows.iter().map(row_of).collect()
    }

    async fn count_open_for(&self, target: ReportTarget, target_id: &str) -> Result<i64> {
        sqlx::query_scalar(
            "SELECT count(*) FROM music.content_reports \
             WHERE target_kind = $1 AND target_id = $2 AND status = 'open'",
        )
        .bind(target.as_str())
        .bind(target_id)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)
    }
}
