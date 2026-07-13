//! The catalog data-access port.
//!
//! [`CatalogRepo`] is the storage primitive the score-crawler writes through
//! (idempotently, deduplicating by content hash). [`FakeCatalogRepo`] backs unit
//! tests without Postgres; [`crate::pg::PgCatalogRepo`] is the real adapter.

use std::sync::Mutex;

use anyhow::Result;
use async_trait::async_trait;

/// One public-corpus catalog row: the provenance that must travel with a
/// redistributed score, plus search/musical metadata. Enum-like fields are
/// snake_case strings matching the crawler's serde output and the table CHECKs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    /// UUID v7 (text form).
    pub id: String,
    pub title: Option<String>,
    pub composer: Option<String>,
    pub arranger: Option<String>,
    pub source: String,
    pub source_url: String,
    pub source_item_id: String,
    pub license: String,
    pub license_url: Option<String>,
    pub confidence: String,
    pub sha256: String,
    pub origin_format: String,
    pub conversion_status: String,
    pub object_key: String,
    pub size_bytes: i64,
    pub work_key: String,
    pub title_norm: Option<String>,
    pub is_piano: bool,
    pub key_fifths: i32,
    pub time_sig: String,
    pub measure_count: i32,
    pub language: Option<String>,
    pub voicing: Option<String>,
    pub level: Option<String>,
    pub level_source: Option<String>,
}

/// Storage surface for the public catalog.
#[async_trait]
pub trait CatalogRepo: Send + Sync {
    /// Whether a row with this content hash already exists.
    async fn sha_exists(&self, sha256: &str) -> Result<bool>;

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
            title: Some("T".into()),
            composer: Some("C".into()),
            arranger: None,
            source: "pdmx".into(),
            source_url: "u".into(),
            source_item_id: "1".into(),
            license: "CC-BY-4.0".into(),
            license_url: None,
            confidence: "verified".into(),
            sha256: sha.into(),
            origin_format: "music_xml".into(),
            conversion_status: "converted".into(),
            object_key: "safe/pdmx/c/t.mxl".into(),
            size_bytes: 10,
            work_key: "c::t".into(),
            title_norm: Some("t".into()),
            is_piano: true,
            key_fifths: 0,
            time_sig: "4/4".into(),
            measure_count: 1,
            language: None,
            voicing: None,
            level: Some("beginner".into()),
            level_source: Some("heuristic".into()),
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
