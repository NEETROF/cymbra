//! The catalog data-access port.
//!
//! [`CatalogRepo`] is the storage primitive the score-crawler writes through
//! (idempotently, deduplicating by content hash). [`FakeCatalogRepo`] backs unit
//! tests without Postgres; [`crate::pg::PgCatalogRepo`] is the real adapter.

use std::sync::Mutex;

use anyhow::Result;
use async_trait::async_trait;

/// Musical facets a catalog row carries (change: score-catalog-facets),
/// populated by the crawler at ingest so the search filters + generated cover
/// have them without a backfill pass. Mirrors the crawler's `ScoreFacets`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScoreFacets {
    pub min_note_value: Option<u8>,
    pub has_tuplets: bool,
    pub has_dotted: bool,
    pub has_chords: bool,
    pub lowest_midi: Option<u8>,
    pub highest_midi: Option<u8>,
    pub staff_count: u8,
    pub note_count: u32,
    pub tempo_bpm: Option<u16>,
    pub has_dynamics: bool,
}

impl ScoreFacets {
    /// Copy from the engine's [`cymbra_musicxml_core::ScoreFacets`] (the shared
    /// derivation), so the crawler and the upload path map facets the same way.
    pub fn from_core(f: &cymbra_musicxml_core::ScoreFacets) -> Self {
        Self {
            min_note_value: f.min_note_value,
            has_tuplets: f.has_tuplets,
            has_dotted: f.has_dotted,
            has_chords: f.has_chords,
            lowest_midi: f.lowest_midi,
            highest_midi: f.highest_midi,
            staff_count: f.staff_count,
            note_count: f.note_count,
            tempo_bpm: f.tempo_bpm,
            has_dynamics: f.has_dynamics,
        }
    }
}

/// The descriptive + facet metadata derived from a parsed score. The public
/// catalog ([`CatalogEntry`]) and user uploads ([`crate::user_scores::UserScore`])
/// carry the *same* block, so it lives here once instead of being repeated on both
/// structs. The two tables still store these columns flat (each keeps its own
/// indexes); the shared column list + row/bind mapping is in [`crate::pg`]
/// ([`crate::pg::META_COLS`] / `bind_meta` / `meta_from_row`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScoreMeta {
    pub title: Option<String>,
    pub composer: Option<String>,
    pub title_norm: Option<String>,
    pub work_key: String,
    pub key_fifths: i32,
    pub time_sig: String,
    pub measure_count: i32,
    pub is_piano: bool,
    pub facets: ScoreFacets,
}

/// One public-corpus catalog row: the provenance that must travel with a
/// redistributed score, plus search/musical metadata. Enum-like fields are
/// snake_case strings matching the crawler's serde output and the table CHECKs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    /// UUID v7 (text form).
    pub id: String,
    pub arranger: Option<String>,
    pub source: String,
    pub source_url: String,
    pub source_item_id: String,
    pub license: String,
    pub license_url: Option<String>,
    pub confidence: String,
    pub sha256: String,
    /// Musical content fingerprint — dedup across re-encodings/sources.
    pub content_fingerprint: String,
    pub origin_format: String,
    pub conversion_status: String,
    pub object_key: String,
    pub size_bytes: i64,
    /// Accent/case-folded composer for typo-tolerant search (parity with
    /// `title_norm`); populated by the crawler, backfilled for older rows.
    pub composer_norm: Option<String>,
    pub language: Option<String>,
    pub voicing: Option<String>,
    pub level: Option<String>,
    pub level_source: Option<String>,
    /// The proposer's user id for a user-proposed row (change: add-score-catalog-
    /// proposal); `None` for a crawler-ingested row. A plain uuid (no cross-schema FK),
    /// resolved to a pseudo at read time.
    pub proposed_by: Option<String>,
    /// Shared descriptive + facet metadata (title/composer/key/time-sig/facets…),
    /// stored flat in `catalog_scores` but held here as one block.
    pub meta: ScoreMeta,
}

/// Storage surface for the public catalog.
#[async_trait]
pub trait CatalogRepo: Send + Sync {
    /// Whether a row with this exact content hash already exists.
    async fn sha_exists(&self, sha256: &str) -> Result<bool>;

    /// Whether a row with this musical content fingerprint already exists (the
    /// same piece, possibly re-encoded or from another source).
    async fn fingerprint_exists(&self, fingerprint: &str) -> Result<bool>;

    /// Inserts a catalog row, ignoring a duplicate `sha256`. Returns `true` when
    /// a row was inserted, `false` when it already existed (idempotent).
    async fn insert(&self, entry: &CatalogEntry) -> Result<bool>;
}

/// In-memory [`CatalogRepo`] for unit tests.
#[derive(Default)]
pub struct FakeCatalogRepo {
    rows: Mutex<Vec<CatalogEntry>>,
}

impl FakeCatalogRepo {
    /// Snapshot of the inserted rows.
    pub fn rows(&self) -> Vec<CatalogEntry> {
        self.rows.lock().expect("catalog fake lock").clone()
    }
}

#[async_trait]
impl CatalogRepo for FakeCatalogRepo {
    async fn sha_exists(&self, sha256: &str) -> Result<bool> {
        let rows = self.rows.lock().expect("catalog fake lock");
        Ok(rows.iter().any(|r| r.sha256 == sha256))
    }

    async fn fingerprint_exists(&self, fingerprint: &str) -> Result<bool> {
        let rows = self.rows.lock().expect("catalog fake lock");
        Ok(rows.iter().any(|r| r.content_fingerprint == fingerprint))
    }

    async fn insert(&self, entry: &CatalogEntry) -> Result<bool> {
        let mut rows = self.rows.lock().expect("catalog fake lock");
        if rows.iter().any(|r| r.sha256 == entry.sha256) {
            return Ok(false);
        }
        rows.push(entry.clone());
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(sha: &str) -> CatalogEntry {
        CatalogEntry {
            id: "id".into(),
            arranger: None,
            source: "pdmx".into(),
            source_url: "u".into(),
            source_item_id: "1".into(),
            license: "CC-BY-4.0".into(),
            license_url: None,
            confidence: "verified".into(),
            sha256: sha.into(),
            content_fingerprint: format!("fp-{sha}"),
            origin_format: "music_xml".into(),
            conversion_status: "converted".into(),
            object_key: "safe/pdmx/c/t.mxl".into(),
            size_bytes: 10,
            composer_norm: Some("c".into()),
            language: None,
            voicing: None,
            level: Some("beginner".into()),
            level_source: Some("heuristic".into()),
            proposed_by: None,
            meta: ScoreMeta {
                title: Some("T".into()),
                composer: Some("C".into()),
                title_norm: Some("t".into()),
                work_key: "c::t".into(),
                key_fifths: 0,
                time_sig: "4/4".into(),
                measure_count: 1,
                is_piano: true,
                facets: ScoreFacets::default(),
            },
        }
    }

    #[tokio::test]
    async fn insert_is_idempotent_by_sha() {
        let repo = FakeCatalogRepo::default();
        assert!(repo.insert(&entry("aaa")).await.unwrap());
        assert!(!repo.insert(&entry("aaa")).await.unwrap()); // dup
        assert!(repo.insert(&entry("bbb")).await.unwrap());
        assert_eq!(repo.rows().len(), 2);
        assert!(repo.sha_exists("aaa").await.unwrap());
        assert!(!repo.sha_exists("zzz").await.unwrap());
    }
}
