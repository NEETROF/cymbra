//! Postgres-backed [`CatalogRepo`] — thin I/O glue (excluded from the coverage
//! gate; exercised by integration tests against a real database).
//!
//! Uses the runtime `sqlx::query(...).bind(...)` API (not the compile-time
//! `query!` macros), matching `backend/user/src/pg.rs`. Table names are fully
//! qualified (`score.catalog_scores`) so it works regardless of the connecting
//! role's `search_path`.

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::repo::{CatalogEntry, CatalogRepo};

/// Postgres implementation over a pool (typically an admin/ingestion role).
pub struct PgCatalogRepo {
    pool: PgPool,
}

impl PgCatalogRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatalogRepo for PgCatalogRepo {
    async fn sha_exists(&self, sha256: &str) -> Result<bool> {
        let row =
            sqlx::query("SELECT EXISTS (SELECT 1 FROM score.catalog_scores WHERE sha256 = $1)")
                .bind(sha256)
                .fetch_one(&self.pool)
                .await
                .context("catalog sha_exists")?;
        Ok(row.get::<bool, _>(0))
    }

    async fn fingerprint_exists(&self, fingerprint: &str) -> Result<bool> {
        let row = sqlx::query(
            "SELECT EXISTS (SELECT 1 FROM score.catalog_scores WHERE content_fingerprint = $1)",
        )
        .bind(fingerprint)
        .fetch_one(&self.pool)
        .await
        .context("catalog fingerprint_exists")?;
        Ok(row.get::<bool, _>(0))
    }

    async fn insert(&self, e: &CatalogEntry) -> Result<bool> {
        let id = uuid::Uuid::parse_str(&e.id).unwrap_or_else(|_| uuid::Uuid::now_v7());
        // ON CONFLICT (sha256) DO NOTHING makes re-ingestion idempotent; the
        // affected-row count tells us whether this was a new insert.
        let result = sqlx::query(
            "INSERT INTO score.catalog_scores (\
                id, title, composer, arranger, source, source_url, source_item_id, \
                license, license_url, confidence, sha256, origin_format, conversion_status, \
                object_key, size_bytes, work_key, title_norm, is_piano, key_fifths, time_sig, \
                measure_count, language, voicing, level, level_source, content_fingerprint) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,\
                $21,$22,$23,$24,$25,$26) \
             ON CONFLICT (sha256) DO NOTHING",
        )
        .bind(id)
        .bind(&e.title)
        .bind(&e.composer)
        .bind(&e.arranger)
        .bind(&e.source)
        .bind(&e.source_url)
        .bind(&e.source_item_id)
        .bind(&e.license)
        .bind(&e.license_url)
        .bind(&e.confidence)
        .bind(&e.sha256)
        .bind(&e.origin_format)
        .bind(&e.conversion_status)
        .bind(&e.object_key)
        .bind(e.size_bytes)
        .bind(&e.work_key)
        .bind(&e.title_norm)
        .bind(e.is_piano)
        .bind(e.key_fifths)
        .bind(&e.time_sig)
        .bind(e.measure_count)
        .bind(&e.language)
        .bind(&e.voicing)
        .bind(&e.level)
        .bind(&e.level_source)
        .bind(&e.content_fingerprint)
        .execute(&self.pool)
        .await
        .context("catalog insert")?;
        Ok(result.rows_affected() > 0)
    }
}
