//! Postgres-backed [`CatalogRepo`] — thin I/O glue (excluded from the coverage
//! gate; exercised by integration tests against a real database).
//!
//! Uses the runtime `sqlx::query(...).bind(...)` API (not the compile-time
//! `query!` macros), matching `backend/user/src/pg.rs`. Table names are fully
//! qualified (`music.catalog_scores`) so it works regardless of the connecting
//! role's `search_path`.

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Row, postgres::PgRow};

use crate::catalog_search::{CatalogHit, CatalogSearchParams, CatalogSearchRepo};
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
            sqlx::query("SELECT EXISTS (SELECT 1 FROM music.catalog_scores WHERE sha256 = $1)")
                .bind(sha256)
                .fetch_one(&self.pool)
                .await
                .context("catalog sha_exists")?;
        Ok(row.get::<bool, _>(0))
    }

    async fn fingerprint_exists(&self, fingerprint: &str) -> Result<bool> {
        let row = sqlx::query(
            "SELECT EXISTS (SELECT 1 FROM music.catalog_scores WHERE content_fingerprint = $1)",
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
            "INSERT INTO music.catalog_scores (\
                id, title, composer, arranger, source, source_url, source_item_id, \
                license, license_url, confidence, sha256, origin_format, conversion_status, \
                object_key, size_bytes, work_key, title_norm, is_piano, key_fifths, time_sig, \
                measure_count, language, voicing, level, level_source, content_fingerprint, \
                composer_norm) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,\
                $21,$22,$23,$24,$25,$26,$27) \
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
        .bind(&e.composer_norm)
        .execute(&self.pool)
        .await
        .context("catalog insert")?;
        Ok(result.rows_affected() > 0)
    }
}

// ---------------------------------------------------------------------------
// Catalog full-text search (change: score-hub-search) — gRPC-facing, so it
// returns the platform `Result`. Same thin-I/O glue, coverage-excluded and
// exercised by integration tests; the pure logic lives in the module + fakes.
// ---------------------------------------------------------------------------

use cymbra_platform::{AppError, Result as PlatformResult};

/// Maps a sqlx error to an internal `AppError` (no detail leaked to clients).
fn search_internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("catalog_search db: {e}"))
}

fn row_to_hit(r: &PgRow) -> CatalogHit {
    CatalogHit {
        id: r.get::<uuid::Uuid, _>("id").to_string(),
        title: r.get("title"),
        composer: r.get("composer"),
        level: r.get("level"),
        license: r.get("license"),
        source: r.get("source"),
    }
}

const HIT_COLS: &str = "id, title, composer, level, license, source";

/// Postgres-backed [`CatalogSearchRepo`] over the `music_svc` pool.
pub struct PgCatalogSearchRepo {
    pool: PgPool,
}

impl PgCatalogSearchRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatalogSearchRepo for PgCatalogSearchRepo {
    async fn search(&self, p: &CatalogSearchParams) -> PlatformResult<Vec<CatalogHit>> {
        // Substring match on the normalised columns (trigram-GIN accelerated),
        // ranked by trigram similarity with a deterministic `(title_norm, id)`
        // tiebreak so paging is stable. The query text/author are pre-normalised
        // by the module, so no re-parse/unaccent is needed at query time.
        let rows = sqlx::query(&format!(
            "SELECT {HIT_COLS} FROM music.catalog_scores \
             WHERE ($1 = '' OR title_norm ILIKE '%'||$1||'%' OR composer_norm ILIKE '%'||$1||'%') \
               AND ($2::text IS NULL OR composer_norm ILIKE '%'||$2||'%') \
               AND ($3::text IS NULL OR level = $3) \
             ORDER BY \
               CASE WHEN $1 = '' THEN 0 \
                    ELSE GREATEST(COALESCE(similarity(title_norm, $1), 0), \
                                  COALESCE(similarity(composer_norm, $1), 0)) END DESC, \
               title_norm ASC NULLS LAST, id ASC \
             LIMIT $4 OFFSET $5"
        ))
        .bind(&p.text_norm)
        .bind(&p.author_norm)
        .bind(&p.level)
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&self.pool)
        .await
        .map_err(search_internal)?;
        Ok(rows.iter().map(row_to_hit).collect())
    }

    async fn hits_by_ids(&self, ids: &[String]) -> PlatformResult<Vec<CatalogHit>> {
        // Skip unparseable ids (never a 500); an empty set short-circuits.
        let uuids: Vec<uuid::Uuid> = ids
            .iter()
            .filter_map(|s| uuid::Uuid::parse_str(s).ok())
            .collect();
        if uuids.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query(&format!(
            "SELECT {HIT_COLS} FROM music.catalog_scores WHERE id = ANY($1)"
        ))
        .bind(&uuids)
        .fetch_all(&self.pool)
        .await
        .map_err(search_internal)?;
        Ok(rows.iter().map(row_to_hit).collect())
    }

    async fn object_key(&self, id: &str) -> PlatformResult<Option<String>> {
        let Ok(uuid) = uuid::Uuid::parse_str(id) else {
            return Ok(None); // malformed id → not found, never a 500
        };
        let row = sqlx::query("SELECT object_key FROM music.catalog_scores WHERE id = $1")
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(search_internal)?;
        Ok(row.map(|r| r.get::<String, _>("object_key")))
    }
}
