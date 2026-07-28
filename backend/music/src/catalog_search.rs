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

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cymbra_musicxml_core::normalize_text;
use cymbra_platform::Result;

use crate::score_rating::FakeScoreRatingRepo;

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

/// One validated sort key (change: add-moderation-back-office): an allow-listed
/// `field`, descending when `descending`. In a `sort` list the first key is primary
/// and later keys break ties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortKey {
    pub field: String,
    pub descending: bool,
}

/// The allow-list of sortable search fields → whether the key is
/// **moderation-oriented** (privileged: only a moderator/admin may sort by it).
/// Substance/facet keys are open to any caller. The names are the wire contract;
/// each adapter maps a name to its own storage (SQL column / fake accessor), so a
/// name never reaches SQL unvalidated. `needs_review` is accepted but inert until
/// the rating change (#2) adds its backing column — it degrades to a no-op order.
pub const SORT_FIELDS: &[(&str, bool)] = &[
    ("measure_count", false),
    ("staff_count", false),
    ("note_count", false),
    ("min_note_value", false),
    ("tempo_bpm", false),
    ("title", false),
    ("composer", false),
    ("status_rank", true),
    ("needs_review", true),
];

/// `Some(privileged)` when `field` is allow-listed, else `None` (unknown field).
pub fn sort_field_privilege(field: &str) -> Option<bool> {
    SORT_FIELDS
        .iter()
        .find(|(f, _)| *f == field)
        .map(|(_, privileged)| *privileged)
}

/// Whether `field` is a moderation-oriented (privileged) sort key. Unknown fields
/// are not privileged (they are rejected later as invalid, not as unauthorized).
pub fn is_moderation_sort_field(field: &str) -> bool {
    sort_field_privilege(field) == Some(true)
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
    /// Moderation-status gate (change: add-score-moderation-gating). `None` = the
    /// normal-caller default of accepted-only; `Some(status)` filters to exactly
    /// that status (a privileged, back-office-only path — the gRPC layer authorises
    /// it before it is ever set here).
    pub moderation_status: Option<String>,
    /// Validated multi-key sort (change: add-moderation-back-office). Empty = the
    /// existing default ordering (so the app hub is unaffected).
    pub sort: Vec<SortKey>,
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
    /// Privileged moderation-status filter (change: add-score-moderation-gating).
    /// `None` for a normal caller (accepted-only); `Some(status)` only after the
    /// gRPC handler has authorised the caller as admin/moderator.
    pub moderation_status: Option<String>,
    /// Raw sort keys from the request; validated against [`SORT_FIELDS`] by the
    /// module before they reach a repo. Empty = the default ordering.
    pub sort: Vec<SortKey>,
    pub limit: i64,
    pub offset: i64,
}

/// The catalog read port: search, resolve saved ids to hits, and resolve bytes.
#[async_trait]
pub trait CatalogSearchRepo: Send + Sync {
    /// One page of attribution-complete hits matching `p`, deterministically
    /// ordered so paging is stable, paired with the **total** number of rows
    /// matching the query+filters (independent of `limit`/`offset`) so callers can
    /// show the full match count without loading every page.
    async fn search(&self, p: &CatalogSearchParams) -> Result<(Vec<CatalogHit>, i64)>;

    /// Hits for the given catalog ids, existing rows only (missing ids are simply
    /// absent). Order is unspecified — the caller re-orders to the saved order.
    async fn hits_by_ids(&self, ids: &[String]) -> Result<Vec<CatalogHit>>;

    /// One catalog hit by id, or `None` if it doesn't exist. `include_unvalidated`
    /// gates on moderation exactly like [`Self::object_key`]: `false` resolves only
    /// an `accepted` score; `true` (an authorised reviewer) resolves any status.
    /// Backs a self-sufficient detail/deep-link view (change: add-moderation-back-office).
    async fn hit_by_id(&self, id: &str, include_unvalidated: bool) -> Result<Option<CatalogHit>>;

    /// The object-store key of a catalog score (for the byte fetch); `None` when
    /// the id does not exist. Doubles as the existence check used on save.
    ///
    /// `include_unvalidated` gates on moderation (change: add-score-moderation-
    /// gating): when `false` (a normal caller), only an `accepted` score resolves —
    /// a `pending`/`rejected` id is reported as absent (`None`), so its bytes can't
    /// be opened and it can't be saved. When `true` (an authorised reviewer), a
    /// score in any status resolves so a moderator can open it.
    async fn object_key(&self, id: &str, include_unvalidated: bool) -> Result<Option<String>>;

    /// Evaluate a score (change: add-moderation-back-office): set its
    /// `moderation_status` and stamp `reviewed_by = reviewer_id` + `reviewed_at =
    /// now()` in one write. Returns `true` when a row was updated, `false` when no
    /// score has that id (so the caller can surface not-found). Authorization is the
    /// gRPC layer's job; the store just writes.
    async fn set_moderation_status(
        &self,
        score_id: &str,
        status: &str,
        reviewer_id: &str,
    ) -> Result<bool>;

    /// The rating deck's source (change: improve-rating-deck-sourcing): the
    /// caller's **un-rated** `accepted` scores, ordered **least-rated first**
    /// (fewest existing ratings, so the scores that most need community signal
    /// come first) with an `id` tiebreak, paginated. A score `user_id` has already
    /// rated is excluded, so the deck reaches its empty state once everything is
    /// rated. Not owner-scoped data, but the exclusion is per caller.
    async fn rating_deck(&self, user_id: &str, limit: i64, offset: i64) -> Result<Vec<CatalogHit>>;
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
    /// Moderation status (change: add-score-moderation-gating). `new()` seeds
    /// `accepted` (a hub-visible row, matching how these tests use the fake); the
    /// moderation tests set `pending`/`rejected` via [`Self::with_moderation_status`].
    pub moderation_status: String,
    /// The last reviewer stamped by an evaluate (change: add-moderation-back-office);
    /// `None` until a `set_moderation_status` writes it. Lets tests assert audit.
    pub reviewed_by: Option<String>,
}

impl FakeCatalogRow {
    /// A minimal, hub-visible (`accepted`) row with a fixed licence/source and a
    /// derived object key.
    pub fn new(id: &str, title: &str, composer: &str, level: Option<&str>) -> Self {
        Self {
            id: id.into(),
            title: Some(title.into()),
            composer: Some(composer.into()),
            level: level.map(Into::into),
            license: "CC-BY-4.0".into(),
            source: "pdmx".into(),
            object_key: format!("safe/pdmx/{id}.mxl"),
            moderation_status: "accepted".into(),
            ..Default::default()
        }
    }

    /// Override the moderation status (`pending` / `accepted` / `rejected`) for the
    /// moderation-gating tests.
    pub fn with_moderation_status(mut self, status: &str) -> Self {
        self.moderation_status = status.into();
        self
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
    /// Optional shared ratings view for the deck-sourcing query (mirrors the Pg
    /// adapter reading the same tables): both fakes point at one
    /// [`FakeScoreRatingRepo`] so "un-rated by me" / rating-count ordering match
    /// what the module actually wrote.
    ratings: Mutex<Option<Arc<FakeScoreRatingRepo>>>,
}

impl FakeCatalogSearchRepo {
    /// Seed the fake with a fixed set of rows.
    pub fn with(rows: Vec<FakeCatalogRow>) -> Self {
        Self {
            rows: Mutex::new(rows),
            ratings: Mutex::new(None),
        }
    }

    /// Replace the stored rows (through a shared `Arc`) — e.g. to simulate a
    /// crawler re-ingest that dropped or changed a catalog entry.
    pub fn set_rows(&self, rows: Vec<FakeCatalogRow>) {
        *self.rows.lock().expect("catalog search fake lock") = rows;
    }

    /// Point the deck-sourcing query at the shared ratings fake, so
    /// `rating_deck` excludes what the module rated and orders by count.
    pub fn set_rating_view(&self, ratings: Arc<FakeScoreRatingRepo>) {
        *self.ratings.lock().expect("catalog search fake lock") = Some(ratings);
    }
}

#[async_trait]
impl CatalogSearchRepo for FakeCatalogSearchRepo {
    async fn search(&self, p: &CatalogSearchParams) -> Result<(Vec<CatalogHit>, i64)> {
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
                // Moderation gate: default to accepted-only; a set filter selects
                // exactly that status (mirrors the Pg adapter's
                // `moderation_status = COALESCE($n, 'accepted')`).
                let want_status = p.moderation_status.as_deref().unwrap_or("accepted");
                let status_ok = r.moderation_status == want_status;
                text_ok && author_ok && level_ok && status_ok && facets_match(r, p)
            })
            .collect();
        // Total over the full filtered set, before pagination (mirrors the Pg
        // adapter's `COUNT(*) OVER()`).
        let total = matched.len() as i64;
        // Apply the validated sort keys (primary first), then the deterministic
        // `(title_norm, id)` tiebreak so paging stays stable — mirroring the Pg
        // adapter, which appends the same tiebreak after the sort keys. An empty
        // `sort` leaves ONLY the default tiebreak, so the hub order is unchanged.
        matched.sort_by(|a, b| {
            for key in &p.sort {
                let ord = sort_value(a, &key.field).cmp(&sort_value(b, &key.field));
                let ord = if key.descending { ord.reverse() } else { ord };
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            a.title_norm().cmp(&b.title_norm()).then(a.id.cmp(&b.id))
        });
        let hits = matched
            .into_iter()
            .skip(p.offset.max(0) as usize)
            .take(p.limit.max(0) as usize)
            .map(FakeCatalogRow::to_hit)
            .collect();
        Ok((hits, total))
    }

    async fn hits_by_ids(&self, ids: &[String]) -> Result<Vec<CatalogHit>> {
        let rows = self.rows.lock().expect("catalog search fake lock");
        Ok(rows
            .iter()
            .filter(|r| ids.iter().any(|id| id == &r.id))
            .map(FakeCatalogRow::to_hit)
            .collect())
    }

    async fn object_key(&self, id: &str, include_unvalidated: bool) -> Result<Option<String>> {
        let rows = self.rows.lock().expect("catalog search fake lock");
        Ok(rows
            .iter()
            .find(|r| r.id == id && (include_unvalidated || r.moderation_status == "accepted"))
            .map(|r| r.object_key.clone()))
    }

    async fn hit_by_id(&self, id: &str, include_unvalidated: bool) -> Result<Option<CatalogHit>> {
        let rows = self.rows.lock().expect("catalog search fake lock");
        Ok(rows
            .iter()
            .find(|r| r.id == id && (include_unvalidated || r.moderation_status == "accepted"))
            .map(FakeCatalogRow::to_hit))
    }

    async fn set_moderation_status(
        &self,
        score_id: &str,
        status: &str,
        reviewer_id: &str,
    ) -> Result<bool> {
        let mut rows = self.rows.lock().expect("catalog search fake lock");
        match rows.iter_mut().find(|r| r.id == score_id) {
            Some(row) => {
                row.moderation_status = status.to_string();
                row.reviewed_by = Some(reviewer_id.to_string());
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn rating_deck(&self, user_id: &str, limit: i64, offset: i64) -> Result<Vec<CatalogHit>> {
        let rows = self.rows.lock().expect("catalog search fake lock");
        let ratings = self.ratings.lock().expect("catalog search fake lock");
        // Exclude the caller's already-rated scores; order least-rated first
        // (fewest ratings), then `id` (mirrors the Pg `ORDER BY count ASC, id`).
        let rated = ratings
            .as_ref()
            .map(|r| r.rated_ids(user_id))
            .unwrap_or_default();
        let count = |id: &str| ratings.as_ref().map(|r| r.rating_count(id)).unwrap_or(0);
        let mut candidates: Vec<&FakeCatalogRow> = rows
            .iter()
            .filter(|r| r.moderation_status == "accepted" && !rated.contains(&r.id))
            .collect();
        candidates.sort_by(|a, b| count(&a.id).cmp(&count(&b.id)).then(a.id.cmp(&b.id)));
        Ok(candidates
            .into_iter()
            .skip(offset.max(0) as usize)
            .take(limit.max(0) as usize)
            .map(FakeCatalogRow::to_hit)
            .collect())
    }
}

/// A monotone sort value for a fake row on an allow-listed field, mirroring the Pg
/// adapter's ORDER BY expression. `status_rank` ranks `pending` > `accepted` >
/// `rejected` (queue priority); `needs_review` is inert (0) until #2 lands;
/// `measure_count` is not modelled on the fake, so it sorts as equal.
fn sort_value(r: &FakeCatalogRow, field: &str) -> i64 {
    match field {
        "staff_count" => r.staff_count.unwrap_or(0) as i64,
        "note_count" => r.note_count.unwrap_or(0) as i64,
        "min_note_value" => r.min_note_value.unwrap_or(0) as i64,
        "tempo_bpm" => r.tempo_bpm.unwrap_or(0) as i64,
        "status_rank" => match r.moderation_status.as_str() {
            "pending" => 2,
            "accepted" => 1,
            _ => 0,
        },
        // `needs_review` (no backing data yet) and `measure_count`/`title`/`composer`
        // (not modelled on the fake) contribute no ordering here.
        _ => 0,
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
        let (hits, _) = repo.search(&params("lune")).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["1"]
        );
        // Composer fragment, accent-insensitive ("frederic" matches "Frédéric").
        let (hits, _) = repo.search(&params("frederic")).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["3"]
        );
        // Composer surname shared by two works.
        let (hits, _) = repo.search(&params("debussy")).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["1", "4"] // ordered by title_norm: "clair..." < "prelude"
        );
    }

    #[tokio::test]
    async fn empty_query_browses_everything_deterministically() {
        let repo = seeded();
        let (hits, _) = repo.search(&params("")).await.unwrap();
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
        let (hits, _) = repo.search(&p).await.unwrap();
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
        let (page1, total1) = repo.search(&page(0)).await.unwrap();
        let (page2, total2) = repo.search(&page(2)).await.unwrap();
        let p1: Vec<String> = page1.into_iter().map(|h| h.id).collect();
        let p2: Vec<String> = page2.into_iter().map(|h| h.id).collect();
        assert_eq!(p1, ["1", "2"]);
        assert_eq!(p2, ["3", "4"]);
        assert!(p1.iter().all(|id| !p2.contains(id))); // no dup across pages
        // The reported total is the full match count, independent of the page.
        assert_eq!(total1, 4);
        assert_eq!(total2, 4);
    }

    #[tokio::test]
    async fn search_reports_full_match_total_independent_of_limit() {
        let repo = seeded();
        // One small page over the two Debussy works: total counts both, not the page.
        let (hits, total) = repo
            .search(&CatalogSearchParams {
                text_norm: normalize_text("debussy"),
                limit: 1,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(total, 2);
    }

    #[tokio::test]
    async fn object_key_resolves_existence() {
        let repo = seeded();
        assert_eq!(
            repo.object_key("2", false).await.unwrap().as_deref(),
            Some("safe/pdmx/2.mxl")
        );
        assert!(repo.object_key("nope", false).await.unwrap().is_none());
    }

    /// A corpus of mixed moderation status for the gating tests.
    fn moderated() -> FakeCatalogSearchRepo {
        FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new("a", "Accepted One", "Bach", Some("beginner"))
                .with_moderation_status("accepted"),
            FakeCatalogRow::new("b", "Accepted Two", "Bach", Some("advanced"))
                .with_moderation_status("accepted"),
            FakeCatalogRow::new("p", "Pending One", "Bach", Some("beginner"))
                .with_moderation_status("pending"),
            FakeCatalogRow::new("r", "Rejected One", "Bach", Some("beginner"))
                .with_moderation_status("rejected"),
        ])
    }

    #[tokio::test]
    async fn search_defaults_to_accepted_only() {
        let repo = moderated();
        // No status filter → only the two `accepted` rows; pending/rejected excluded.
        let (hits, total) = repo.search(&params("")).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(total, 2);
    }

    #[tokio::test]
    async fn privileged_status_filter_selects_exactly_that_status() {
        let repo = moderated();
        let by_status = |s: &str| CatalogSearchParams {
            moderation_status: Some(s.into()),
            limit: 50,
            ..Default::default()
        };
        let ids = |hits: Vec<CatalogHit>| hits.into_iter().map(|h| h.id).collect::<Vec<_>>();
        let (pending, _) = repo.search(&by_status("pending")).await.unwrap();
        assert_eq!(ids(pending), ["p"]);
        let (rejected, _) = repo.search(&by_status("rejected")).await.unwrap();
        assert_eq!(ids(rejected), ["r"]);
        // Even a privileged caller can ask for `accepted` explicitly.
        let (accepted, _) = repo.search(&by_status("accepted")).await.unwrap();
        assert_eq!(ids(accepted), ["a", "b"]);
    }

    #[tokio::test]
    async fn status_filter_composes_with_text_and_level_filters() {
        let repo = moderated();
        // Privileged `pending` filter AND a level filter AND a text query all
        // compose conjunctively: the pending beginner "Pending One" qualifies, but
        // the accepted rows are excluded by the status filter.
        let p = CatalogSearchParams {
            text_norm: normalize_text("one"),
            level: Some("beginner".into()),
            moderation_status: Some("pending".into()),
            limit: 50,
            ..Default::default()
        };
        let (hits, total) = repo.search(&p).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["p"]
        );
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn empty_sort_preserves_default_title_norm_order() {
        // Regression (change: add-moderation-back-office): with no sort keys, the
        // order is the pre-existing `(title_norm, id)` default — the app hub, which
        // sends no sort, is unaffected.
        let repo = seeded();
        let (hits, _) = repo.search(&params("")).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["1", "2", "3", "4"] // title_norm order, identical to before
        );
    }

    #[tokio::test]
    async fn multi_key_sort_orders_primary_then_tiebreak() {
        // Rows with distinct staff/note counts; a moderator sorts by staff_count
        // desc, then note_count desc as a tiebreaker.
        let repo = FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow {
                id: "a".into(),
                staff_count: Some(1),
                note_count: Some(500),
                moderation_status: "accepted".into(),
                ..FakeCatalogRow::new("a", "A", "X", None)
            },
            FakeCatalogRow {
                id: "b".into(),
                staff_count: Some(2),
                note_count: Some(100),
                moderation_status: "accepted".into(),
                ..FakeCatalogRow::new("b", "B", "X", None)
            },
            FakeCatalogRow {
                id: "c".into(),
                staff_count: Some(2),
                note_count: Some(900),
                moderation_status: "accepted".into(),
                ..FakeCatalogRow::new("c", "C", "X", None)
            },
        ]);
        let p = CatalogSearchParams {
            sort: vec![
                SortKey {
                    field: "staff_count".into(),
                    descending: true,
                },
                SortKey {
                    field: "note_count".into(),
                    descending: true,
                },
            ],
            limit: 50,
            ..Default::default()
        };
        let (hits, _) = repo.search(&p).await.unwrap();
        // staff 2 first (c before b by note_count desc), then staff 1 (a).
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["c", "b", "a"]
        );
    }

    #[tokio::test]
    async fn status_rank_sort_surfaces_pending_first() {
        // The queue's default primary key: status_rank desc puts pending ahead of
        // accepted ahead of rejected.
        let repo = moderated(); // ids a,b=accepted; p=pending; r=rejected
        let p = CatalogSearchParams {
            moderation_status: None,
            sort: vec![SortKey {
                field: "status_rank".into(),
                descending: true,
            }],
            // A privileged caller browsing all statuses would pass a status filter;
            // here we assert ordering over the accepted-only default set is stable.
            limit: 50,
            ..Default::default()
        };
        let (hits, _) = repo.search(&p).await.unwrap();
        // Default visibility is accepted-only, so only a,b appear, ordered by the
        // stable tiebreak (both rank equally on status_rank).
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            ["a", "b"]
        );
    }

    #[tokio::test]
    async fn set_moderation_status_writes_status_and_reviewer() {
        let repo = moderated();
        // Evaluate the pending row → accepted, stamping the reviewer.
        assert!(
            repo.set_moderation_status("p", "accepted", "mod-1")
                .await
                .unwrap()
        );
        // It now appears in the accepted-only default search…
        let (hits, _) = repo.search(&params("")).await.unwrap();
        assert!(hits.iter().any(|h| h.id == "p"));
        // …and the reviewer was recorded on the row.
        let rows = repo.rows.lock().unwrap();
        let row = rows.iter().find(|r| r.id == "p").unwrap();
        assert_eq!(row.moderation_status, "accepted");
        assert_eq!(row.reviewed_by.as_deref(), Some("mod-1"));
    }

    #[tokio::test]
    async fn set_moderation_status_unknown_id_is_no_op() {
        let repo = moderated();
        assert!(
            !repo
                .set_moderation_status("does-not-exist", "accepted", "mod-1")
                .await
                .unwrap()
        );
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
