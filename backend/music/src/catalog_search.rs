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
    // Attribution + facets for the generated cover (change: score-catalog-facets-
    // cover). Facets are `None` until backfilled.
    pub arranger: Option<String>,
    pub min_note_value: Option<i32>,
    pub tempo_bpm: Option<i32>,
    pub note_count: Option<i32>,
    pub lowest_midi: Option<i32>,
    pub highest_midi: Option<i32>,
    pub time_sig: String,
    pub key_fifths: i32,
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
    // --- musical facet filters (change: score-catalog-facets) --------------
    // Each `None` = no constraint. When a filter is set, a row whose facet is
    // NULL is excluded (an unknown trait can't be asserted to satisfy the filter).
    /// Keyboard/grand-staff only.
    pub is_piano: Option<bool>,
    /// Fastest allowed note value (power-of-two denominator) → `min_note_value <= v`.
    pub max_note_value: Option<i16>,
    pub has_chords: Option<bool>,
    pub has_tuplets: Option<bool>,
    pub has_dotted: Option<bool>,
    /// Maximum hand span → `highest_midi - lowest_midi <= v`.
    pub max_ambitus_semitones: Option<i16>,
    pub staff_count: Option<i16>,
    /// Tempo range (marked BPM) → `tempo_bpm BETWEEN min_bpm AND max_bpm`.
    pub min_bpm: Option<i32>,
    pub max_bpm: Option<i32>,
    pub limit: i64,
    pub offset: i64,
}

/// Raw search inputs from the caller (the gRPC request), before validation and
/// text normalisation. The module turns this into a [`CatalogSearchParams`].
#[derive(Debug, Clone, Default)]
pub struct CatalogQuery {
    pub query: String,
    pub author: Option<String>,
    pub level: Option<String>,
    pub is_piano: Option<bool>,
    pub max_note_value: Option<i16>,
    pub has_chords: Option<bool>,
    pub has_tuplets: Option<bool>,
    pub has_dotted: Option<bool>,
    pub max_ambitus_semitones: Option<i16>,
    pub staff_count: Option<i16>,
    pub min_bpm: Option<i32>,
    pub max_bpm: Option<i32>,
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
#[derive(Debug, Clone, Default)]
pub struct FakeCatalogRow {
    pub id: String,
    pub title: Option<String>,
    pub composer: Option<String>,
    pub level: Option<String>,
    pub license: String,
    pub source: String,
    pub object_key: String,
    // Facets (change: score-catalog-facets) — default None so text/author/level
    // tests are unaffected; set via `with_facets` for facet-filter tests.
    pub is_piano: Option<bool>,
    pub min_note_value: Option<i16>,
    pub has_chords: Option<bool>,
    pub has_tuplets: Option<bool>,
    pub has_dotted: Option<bool>,
    pub lowest_midi: Option<i16>,
    pub highest_midi: Option<i16>,
    pub staff_count: Option<i16>,
    pub tempo_bpm: Option<i32>,
    pub arranger: Option<String>,
    pub note_count: Option<i32>,
    pub time_sig: String,
    pub key_fifths: i32,
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
            ..Default::default()
        }
    }

    /// Set the facet fields used by the facet-filter tests (piano, fastest note
    /// value, tempo, ambitus).
    pub fn with_facets(
        mut self,
        is_piano: bool,
        min_note_value: i16,
        tempo_bpm: Option<i32>,
        ambitus: (i16, i16),
    ) -> Self {
        self.is_piano = Some(is_piano);
        self.min_note_value = Some(min_note_value);
        self.tempo_bpm = tempo_bpm;
        self.lowest_midi = Some(ambitus.0);
        self.highest_midi = Some(ambitus.1);
        self
    }

    fn to_hit(&self) -> CatalogHit {
        CatalogHit {
            id: self.id.clone(),
            title: self.title.clone(),
            composer: self.composer.clone(),
            level: self.level.clone(),
            license: self.license.clone(),
            source: self.source.clone(),
            arranger: self.arranger.clone(),
            min_note_value: self.min_note_value.map(i32::from),
            tempo_bpm: self.tempo_bpm,
            note_count: self.note_count,
            lowest_midi: self.lowest_midi.map(i32::from),
            highest_midi: self.highest_midi.map(i32::from),
            time_sig: self.time_sig.clone(),
            key_fifths: self.key_fifths,
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
                text_ok && author_ok && level_ok && facets_match(r, p)
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

/// Applies the facet filters of `p` to a fake row, mirroring the Pg adapter: a
/// set filter excludes rows whose corresponding facet is unknown (`None`).
fn facets_match(r: &FakeCatalogRow, p: &CatalogSearchParams) -> bool {
    fn bool_ok(filter: Option<bool>, value: Option<bool>) -> bool {
        filter.is_none_or(|f| value == Some(f))
    }
    if let Some(pi) = p.is_piano
        && r.is_piano != Some(pi)
    {
        return false;
    }
    if let Some(mv) = p.max_note_value
        && !matches!(r.min_note_value, Some(v) if v <= mv)
    {
        return false;
    }
    if !bool_ok(p.has_chords, r.has_chords)
        || !bool_ok(p.has_tuplets, r.has_tuplets)
        || !bool_ok(p.has_dotted, r.has_dotted)
    {
        return false;
    }
    if let Some(span) = p.max_ambitus_semitones
        && !matches!((r.lowest_midi, r.highest_midi), (Some(lo), Some(hi)) if hi - lo <= span)
    {
        return false;
    }
    if let Some(sc) = p.staff_count
        && r.staff_count != Some(sc)
    {
        return false;
    }
    if p.min_bpm.is_some() || p.max_bpm.is_some() {
        let Some(t) = r.tempo_bpm else { return false };
        if p.min_bpm.is_some_and(|m| t < m) || p.max_bpm.is_some_and(|m| t > m) {
            return false;
        }
    }
    true
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
            limit: 50,
            ..Default::default()
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
            author_norm: Some(normalize_text("Debussy")),
            level: Some("advanced".into()),
            limit: 50,
            ..Default::default()
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
            limit: 2,
            offset,
            ..Default::default()
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
