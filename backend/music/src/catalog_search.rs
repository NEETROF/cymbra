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

//! Catalog full-text search — the read surface over the public `catalog_scores`
//! corpus (change: score-hub-search).
//!
//! Distinct from [`crate::CatalogRepo`] (the crawler's `anyhow` WRITE surface):
//! this is gRPC-facing, so it returns the platform [`Result`]. The module
//! normalises the query/author (accent/case-fold) and validates the level/limit
//! *before* calling the repo, so the adapter only matches an already-normalised
//! query against the persisted `title_norm`/`composer_norm` columns.

use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_musicxml_core::normalize_text;
use cymbra_platform::Result;

/// One catalog score as surfaced by search or the saved list: attribution-
/// complete, but WITHOUT bytes (bytes are fetched separately, by id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogHit {
    pub id: String,
    pub title: Option<String>,
    pub composer: Option<String>,
    pub level: Option<String>,
    pub license: String,
    pub source: String,
}

/// Normalised, validated search parameters. The module fills these in: `text_norm`
/// and `author_norm` are already accent/case-folded, `level` already validated
/// against the fixed set, `limit` already clamped to the server maximum.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogSearchParams {
    /// Accent/case-folded query; empty = browse the corpus (no text constraint).
    pub text_norm: String,
    /// Accent/case-folded composer filter; `None` = no composer constraint.
    pub author_norm: Option<String>,
    /// Difficulty filter (validated); `None` = every level (incl. unleveled).
    pub level: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

/// The catalog read port: search, resolve saved ids to hits, and resolve bytes.
#[async_trait]
pub trait CatalogSearchRepo: Send + Sync {
    /// One page of attribution-complete hits matching `p`, deterministically
    /// ordered so paging is stable.
    async fn search(&self, p: &CatalogSearchParams) -> Result<Vec<CatalogHit>>;

    /// Hits for the given catalog ids, existing rows only (missing ids are simply
    /// absent). Order is unspecified — the caller re-orders to the saved order.
    async fn hits_by_ids(&self, ids: &[String]) -> Result<Vec<CatalogHit>>;

    /// The object-store key of a catalog score (for the byte fetch); `None` when
    /// the id does not exist. Doubles as the existence check used on save.
    async fn object_key(&self, id: &str) -> Result<Option<String>>;
}

/// A seed row for [`FakeCatalogSearchRepo`]. Norm keys are derived on the fly, so
/// tests only supply human-readable title/composer.
#[derive(Debug, Clone)]
pub struct FakeCatalogRow {
    pub id: String,
    pub title: Option<String>,
    pub composer: Option<String>,
    pub level: Option<String>,
    pub license: String,
    pub source: String,
    pub object_key: String,
}

impl FakeCatalogRow {
    /// A minimal row with a fixed licence/source and a derived object key.
    pub fn new(id: &str, title: &str, composer: &str, level: Option<&str>) -> Self {
        Self {
            id: id.into(),
            title: Some(title.into()),
            composer: Some(composer.into()),
            level: level.map(Into::into),
            license: "CC-BY-4.0".into(),
            source: "pdmx".into(),
            object_key: format!("safe/pdmx/{id}.mxl"),
        }
    }

    fn to_hit(&self) -> CatalogHit {
        CatalogHit {
            id: self.id.clone(),
            title: self.title.clone(),
            composer: self.composer.clone(),
            level: self.level.clone(),
            license: self.license.clone(),
            source: self.source.clone(),
        }
    }

    fn title_norm(&self) -> String {
        self.title
            .as_deref()
            .map(normalize_text)
            .unwrap_or_default()
    }

    fn composer_norm(&self) -> String {
        self.composer
            .as_deref()
            .map(normalize_text)
            .unwrap_or_default()
    }
}

/// In-memory [`CatalogSearchRepo`] for unit tests. Substring matching on the
/// accent/case-folded title/composer mirrors the trigram `ILIKE` the Pg adapter
/// runs; results are ordered by `(title_norm, id)` for stable paging.
#[derive(Default)]
pub struct FakeCatalogSearchRepo {
    rows: Mutex<Vec<FakeCatalogRow>>,
}

impl FakeCatalogSearchRepo {
    /// Seed the fake with a fixed set of rows.
    pub fn with(rows: Vec<FakeCatalogRow>) -> Self {
        Self {
            rows: Mutex::new(rows),
        }
    }

    /// Replace the stored rows (through a shared `Arc`) — e.g. to simulate a
    /// crawler re-ingest that dropped or changed a catalog entry.
    pub fn set_rows(&self, rows: Vec<FakeCatalogRow>) {
        *self.rows.lock().expect("catalog search fake lock") = rows;
    }
}

#[async_trait]
impl CatalogSearchRepo for FakeCatalogSearchRepo {
    async fn search(&self, p: &CatalogSearchParams) -> Result<Vec<CatalogHit>> {
        let rows = self.rows.lock().expect("catalog search fake lock");
        let mut matched: Vec<&FakeCatalogRow> = rows
            .iter()
            .filter(|r| {
                let (t, c) = (r.title_norm(), r.composer_norm());
                let text_ok =
                    p.text_norm.is_empty() || t.contains(&p.text_norm) || c.contains(&p.text_norm);
                let author_ok = p.author_norm.as_ref().is_none_or(|a| c.contains(a));
                let level_ok = p
                    .level
                    .as_ref()
                    .is_none_or(|l| r.level.as_deref() == Some(l));
                text_ok && author_ok && level_ok
            })
            .collect();
        // Deterministic order → stable paging (the Pg adapter adds similarity
        // ranking ahead of this tiebreak).
        matched.sort_by(|a, b| a.title_norm().cmp(&b.title_norm()).then(a.id.cmp(&b.id)));
        Ok(matched
            .into_iter()
            .skip(p.offset.max(0) as usize)
            .take(p.limit.max(0) as usize)
            .map(FakeCatalogRow::to_hit)
            .collect())
    }

    async fn hits_by_ids(&self, ids: &[String]) -> Result<Vec<CatalogHit>> {
        let rows = self.rows.lock().expect("catalog search fake lock");
        Ok(rows
            .iter()
            .filter(|r| ids.iter().any(|id| id == &r.id))
            .map(FakeCatalogRow::to_hit)
            .collect())
    }

    async fn object_key(&self, id: &str) -> Result<Option<String>> {
        let rows = self.rows.lock().expect("catalog search fake lock");
        Ok(rows
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.object_key.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded() -> FakeCatalogSearchRepo {
        FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new("1", "Clair de Lune", "Claude Debussy", Some("intermediate")),
            FakeCatalogRow::new("2", "Gymnopédie No. 1", "Erik Satie", Some("beginner")),
            FakeCatalogRow::new("3", "Nocturne", "Frédéric Chopin", Some("advanced")),
            FakeCatalogRow::new("4", "Prélude", "Claude Debussy", Some("advanced")),
        ])
    }

    fn params(text: &str) -> CatalogSearchParams {
        CatalogSearchParams {
            text_norm: normalize_text(text),
            author_norm: None,
            level: None,
            limit: 50,
            offset: 0,
        }
    }

    #[tokio::test]
    async fn matches_title_and_composer_ignoring_case_and_accents() {
        let repo = seeded();
        // Title fragment.
        let hits = repo.search(&params("lune")).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["1"]
        );
        // Composer fragment, accent-insensitive ("frederic" matches "Frédéric").
        let hits = repo.search(&params("frederic")).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["3"]
        );
        // Composer surname shared by two works.
        let hits = repo.search(&params("debussy")).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["1", "4"] // ordered by title_norm: "clair..." < "prelude"
        );
    }

    #[tokio::test]
    async fn empty_query_browses_everything_deterministically() {
        let repo = seeded();
        let hits = repo.search(&params("")).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["1", "2", "3", "4"] // title_norm order
        );
    }

    #[tokio::test]
    async fn author_and_level_filters_compose_conjunctively() {
        let repo = seeded();
        let p = CatalogSearchParams {
            text_norm: String::new(),
            author_norm: Some(normalize_text("Debussy")),
            level: Some("advanced".into()),
            limit: 50,
            offset: 0,
        };
        let hits = repo.search(&p).await.unwrap();
        // Only the advanced Debussy work (Prélude), not the intermediate one.
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["4"]
        );
    }

    #[tokio::test]
    async fn paging_slices_without_overlap() {
        let repo = seeded();
        let page = |offset| CatalogSearchParams {
            text_norm: String::new(),
            author_norm: None,
            level: None,
            limit: 2,
            offset,
        };
        let p1: Vec<String> = repo
            .search(&page(0))
            .await
            .unwrap()
            .into_iter()
            .map(|h| h.id)
            .collect();
        let p2: Vec<String> = repo
            .search(&page(2))
            .await
            .unwrap()
            .into_iter()
            .map(|h| h.id)
            .collect();
        assert_eq!(p1, ["1", "2"]);
        assert_eq!(p2, ["3", "4"]);
        assert!(p1.iter().all(|id| !p2.contains(id))); // no dup across pages
    }

    #[tokio::test]
    async fn object_key_resolves_existence() {
        let repo = seeded();
        assert_eq!(
            repo.object_key("2").await.unwrap().as_deref(),
            Some("safe/pdmx/2.mxl")
        );
        assert!(repo.object_key("nope").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn hits_by_ids_returns_only_existing() {
        let repo = seeded();
        let hits = repo
            .hits_by_ids(&["3".into(), "missing".into(), "1".into()])
            .await
            .unwrap();
        let mut ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
        ids.sort_unstable();
        assert_eq!(ids, ["1", "3"]); // "missing" omitted
    }
}
