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

//! Facet backfill (change: score-catalog-facets) — re-read each stored score
//! object and derive its musical facets in place, without re-crawling.
//!
//! Thin I/O glue over the object store + Postgres (coverage-excluded; the pure
//! derivation lives in `ScoreSummary`). Idempotent + resumable: only rows whose
//! facets are unset (`note_count IS NULL`) are processed, so a re-run is a cheap
//! no-op, and a missing/unreadable object is logged and skipped — never aborts
//! the whole run. Runs for `catalog_scores` (`.mxl` objects) and `user_scores`
//! (already-plain XML); `decode_canonical` handles either.

use anyhow::{Context, Result};
use cymbra_musicxml_core::{ScoreFacets, mxl, parse};
use cymbra_storage::ObjectStorage;
use sqlx::{PgPool, Row};

/// How many rows were updated vs. skipped (missing/unreadable object or unparseable).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BackfillStats {
    pub updated: usize,
    pub skipped: usize,
}

/// The canonical (uncompressed) MusicXML for a stored object: decode a `.mxl`
/// container, else pass plain XML through.
fn decode_canonical(bytes: &[u8]) -> Result<Vec<u8>> {
    if mxl::is_mxl(bytes) {
        mxl::decode(bytes).context("decode .mxl")
    } else {
        Ok(bytes.to_vec())
    }
}

/// Backfill both catalog and user-score facet columns from their stored objects.
pub async fn backfill_all(pool: &PgPool, storage: &dyn ObjectStorage) -> Result<BackfillStats> {
    let mut stats = BackfillStats::default();
    for table in ["music.catalog_scores", "music.user_scores"] {
        let s = backfill_table(pool, storage, table).await?;
        stats.updated += s.updated;
        stats.skipped += s.skipped;
    }
    Ok(stats)
}

async fn backfill_table(
    pool: &PgPool,
    storage: &dyn ObjectStorage,
    table: &str,
) -> Result<BackfillStats> {
    // `note_count IS NULL` marks a not-yet-backfilled row (an empty score gets 0,
    // not null, so it is not reprocessed).
    let rows = sqlx::query(&format!(
        "SELECT id, object_key FROM {table} WHERE note_count IS NULL"
    ))
    .fetch_all(pool)
    .await
    .with_context(|| format!("selecting {table} rows to backfill"))?;

    let mut stats = BackfillStats::default();
    for row in rows {
        let id: uuid::Uuid = row.get("id");
        let object_key: String = row.get("object_key");

        let bytes = match storage.get(&object_key).await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(%id, object_key, error = %e, "backfill: object unreadable, skipping");
                stats.skipped += 1;
                continue;
            }
        };
        let doc = match decode_canonical(&bytes).and_then(|xml| parse(&xml)) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!(%id, object_key, error = %e, "backfill: unparseable, skipping");
                stats.skipped += 1;
                continue;
            }
        };
        let s = ScoreFacets::from_document(&doc);
        update_facets(pool, table, id, &s).await?;
        stats.updated += 1;
    }
    tracing::info!(
        table,
        updated = stats.updated,
        skipped = stats.skipped,
        "backfill complete"
    );
    Ok(stats)
}

async fn update_facets(pool: &PgPool, table: &str, id: uuid::Uuid, s: &ScoreFacets) -> Result<()> {
    sqlx::query(&format!(
        "UPDATE {table} SET \
            min_note_value = $2, has_tuplets = $3, has_dotted = $4, has_chords = $5, \
            lowest_midi = $6, highest_midi = $7, staff_count = $8, note_count = $9, \
            tempo_bpm = $10, has_dynamics = $11 \
         WHERE id = $1"
    ))
    .bind(id)
    .bind(s.min_note_value.map(i16::from))
    .bind(s.has_tuplets)
    .bind(s.has_dotted)
    .bind(s.has_chords)
    .bind(s.lowest_midi.map(i16::from))
    .bind(s.highest_midi.map(i16::from))
    .bind(i16::from(s.staff_count))
    .bind(s.note_count as i32)
    .bind(s.tempo_bpm.map(i32::from))
    .bind(s.has_dynamics)
    .execute(pool)
    .await
    .with_context(|| format!("updating {table} facets for {id}"))?;
    Ok(())
}
