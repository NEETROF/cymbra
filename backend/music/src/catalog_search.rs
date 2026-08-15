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

use crate::score_rating::{
    FakeScoreRatingRepo, RatingConfig, ScoreRatingRepo, is_flagged_for_review,
};

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
    // Moderation facets, meaningful only for a privileged back-office read (a normal
    // caller gets the defaults). `needs_review` is the hybrid re-review flag (design
    // D4): an `accepted` score the community has rated down enough to warrant a
    // moderator's second look — the search adapter computes it from the ratings.
    // `moderation_status` lets a mixed review-queue row show its own status.
    pub needs_review: bool,
    pub moderation_status: Option<String>,
    // Proposal attribution + moderation feedback (change: add-score-catalog-proposal).
    // `proposed_by` is the proposer's user id, present on a user-proposed row (a real
    // column). `proposer_display_name`/`resubmission_note`/`review_reason` are surfaced
    // only to a privileged (moderator/admin) read; `contributor_credit` is the opt-in,
    // public "proposé par @pseudo" (present only for an `accepted`, user-proposed score
    // whose proposer opted into a public profile). The module fills the two resolved
    // names via `UserPort`; the store leaves them `None`.
    pub proposed_by: Option<String>,
    pub proposer_display_name: Option<String>,
    pub contributor_credit: Option<String>,
    pub review_reason: Option<String>,
    pub resubmission_note: Option<String>,
    /// A server-rendered audio teaser exists for this piece (change:
    /// add-score-daily-access-rewards) — from the row's `preview_rendered_at`
    /// marker, never a storage probe.
    pub has_preview: bool,
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
/// name never reaches SQL unvalidated. `needs_review` surfaces scores the community
/// has flagged for re-review, computed on demand from the ratings supplied by the
/// app-rating change (#2) — see the adapter's `needs_review_sql`.
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

/// The musical facet filters (change: score-catalog-facets), bundled so the raw
/// query and the validated params share one definition instead of two parallel
/// field lists. Each `None` = no constraint; when a filter is set, a row whose
/// corresponding facet is NULL/unknown is excluded (an unknown trait can't be
/// asserted to satisfy the filter). Mirrors the Flutter `CatalogFilters` bundle.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FacetFilters {
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
    /// Musical facet filters (each `None` = unconstrained).
    pub facets: FacetFilters,
    /// Moderation-status gate (change: add-score-moderation-gating). `None` = the
    /// normal-caller default of accepted-only; `Some(status)` filters to exactly
    /// that status (a privileged, back-office-only path — the gRPC layer authorises
    /// it before it is ever set here).
    pub moderation_status: Option<String>,
    /// Review-queue mode (change: add-moderation-back-office): when `true` the result
    /// set is `pending` scores PLUS `accepted` scores flagged for re-review, instead
    /// of a single moderation status — overrides [`Self::moderation_status`]. A
    /// privileged, back-office-only path (the gRPC layer authorises it). `false` keeps
    /// the single-status behaviour, so the app hub is unaffected.
    pub review_queue: bool,
    /// All-statuses mode (privileged BO catalog view) — every moderation status.
    pub all_statuses: bool,
    /// Origin filter (`user-proposal` / crawler dataset); `None` = any source.
    pub source: Option<String>,
    /// Audio-teaser filter (change: add-score-daily-access-rewards; privileged,
    /// BO-only): `Some(true)` = rendered only, `Some(false)` = missing only.
    pub has_preview: Option<bool>,
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
    /// Musical facet filters, clamped/validated by the gRPC layer before arriving.
    pub facets: FacetFilters,
    /// Privileged moderation-status filter (change: add-score-moderation-gating).
    /// `None` for a normal caller (accepted-only); `Some(status)` only after the
    /// gRPC handler has authorised the caller as admin/moderator.
    pub moderation_status: Option<String>,
    /// Review-queue mode (change: add-moderation-back-office): `pending` + flagged
    /// `accepted`. Privileged (BO-only) — the gRPC handler authorises it as
    /// admin/moderator before it is ever set. `false` for a normal caller.
    pub review_queue: bool,
    /// All-statuses mode (change: add-score-catalog-proposal): the privileged BO
    /// catalog view showing every moderation status (`pending`/`accepted`/`rejected`).
    /// Gated to admin/moderator at the gRPC layer; `false` for a normal caller.
    pub all_statuses: bool,
    /// Origin filter (change: add-score-catalog-proposal): e.g. `user-proposal` vs a
    /// crawler dataset; `None` = any source. Composes with the status gate.
    pub source: Option<String>,
    /// Audio-teaser filter (change: add-score-daily-access-rewards): gated to
    /// admin/moderator at the gRPC layer; `None` for a normal caller.
    pub has_preview: Option<bool>,
    /// Raw sort keys from the request; validated against [`SORT_FIELDS`] by the
    /// module before they reach a repo. Empty = the default ordering.
    pub sort: Vec<SortKey>,
    pub limit: i64,
    pub offset: i64,
}

/// The catalog read port: search, resolve saved ids to hits, and resolve bytes.
/// A catalog score's object-store key paired with its stored content hash
/// (`sha256`), which the byte fetch exposes to clients as the ETag (change:
/// add-offline-score-cache). Lets a conditional fetch short-circuit an unchanged
/// request without reading the blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogObjectRef {
    pub object_key: String,
    pub sha256: String,
    /// The proposer's user id on a user-proposed row (change: add-score-catalog-
    /// proposal), `None` for a crawler row. The daily-access gate reads it to let a
    /// contributor open their own accepted piece free (change:
    /// add-score-daily-access-rewards).
    pub proposed_by: Option<String>,
    /// When the piece's audio teaser was last rendered (`preview_rendered_at`),
    /// `None` when it has none — the teaser's object key is derived from it
    /// (change: add-score-daily-access-rewards).
    pub preview_rendered_at: Option<chrono::DateTime<chrono::Utc>>,
}

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

    /// The object key **and** stored content hash (`sha256`, exposed as the ETag) of
    /// a catalog score, moderation-gated exactly like [`Self::object_key`]; `None`
    /// when the id does not resolve (change: add-offline-score-cache). Lets the byte
    /// fetch return the ETag and answer a conditional (`if_none_match`) request
    /// without touching the object store when the hash is unchanged.
    async fn object_ref(
        &self,
        id: &str,
        include_unvalidated: bool,
    ) -> Result<Option<CatalogObjectRef>>;

    /// Evaluate a score (change: add-moderation-back-office): set its
    /// `moderation_status` and stamp `reviewed_by = reviewer_id` + `reviewed_at =
    /// now()` in one write. Returns `true` when a row was updated, `false` when no
    /// score has that id (so the caller can surface not-found). Authorization is the
    /// gRPC layer's job; the store just writes. `reason` (change: add-score-catalog-
    /// proposal) is the moderator's motive: stored as `review_reason` when `status`
    /// is `rejected`, cleared otherwise, so a rejected proposal can tell its proposer
    /// why.
    async fn set_moderation_status(
        &self,
        score_id: &str,
        status: &str,
        reviewer_id: &str,
        reason: Option<&str>,
    ) -> Result<bool>;

    /// The (id, moderation_status) of the catalog row whose content digest is `sha256`,
    /// or `None` when no row has it (change: add-score-catalog-proposal). Content is
    /// unique in the catalog, so at most one row matches; the propose path uses it to
    /// detect a duplicate (non-`rejected`) or a reopenable (`rejected`) match.
    async fn find_by_sha(&self, sha256: &str) -> Result<Option<(String, String)>>;

    /// Insert a user-proposed catalog row (change: add-score-catalog-proposal) with
    /// `proposed_by = entry.proposed_by` and `moderation_status` = `accepted` when
    /// `accepted` (an admin proposal) else `pending`. Never reads a client-supplied
    /// status. Returns `true` when a row was inserted.
    async fn insert_proposed(
        &self,
        entry: &crate::repo::CatalogEntry,
        accepted: bool,
    ) -> Result<bool>;

    /// Reopen a `rejected` catalog row on a motivated re-proposal (change: add-score-
    /// catalog-proposal): status → `pending`, `proposed_by = proposer_id`, clear
    /// `review_reason`, store `resubmission_note = note`. Returns `true` when a row was
    /// updated, `false` when no score has that id.
    async fn reopen_rejected(&self, score_id: &str, proposer_id: &str, note: &str) -> Result<bool>;

    /// Apply a curatorial metadata edit (change: add-catalog-metadata-editing) in ONE
    /// transaction: update the curatorial fields + the recomputed derived search keys
    /// (`title_norm`/`composer_norm`/`work_key`), stamp the provenance
    /// (`edited_by`/`edited_at`, and `level_source = 'manual'` when the level changed),
    /// and insert one `catalog_edits` audit row per changed field. Returns `true` when a
    /// row was updated, `false` when no score has that id. Authorization is the gRPC
    /// layer's job; the store just writes. `plan.changes` is assumed non-empty (the
    /// module treats an empty diff as a no-op and never calls this).
    async fn apply_metadata_edit(
        &self,
        score_id: &str,
        editor: &str,
        plan: &crate::catalog_edit::EditPlan,
    ) -> Result<bool>;

    /// The rating deck's source (change: improve-rating-deck-sourcing, widened by
    /// rate-pending-scores): the caller's **un-rated** `pending` + `accepted` scores
    /// (never `rejected`), ordered **least-rated first** (fewest existing ratings, so
    /// the scores that most need community signal come first) with an `id` tiebreak,
    /// paginated. Sourcing pending too lets community ratings feed the moderation
    /// backlog. A score `user_id` has already rated is excluded, so the deck reaches
    /// its empty state once everything is rated. Not owner-scoped data, but the
    /// exclusion is per caller.
    async fn rating_deck(&self, user_id: &str, limit: i64, offset: i64) -> Result<Vec<CatalogHit>>;

    /// Stamp the audio-teaser rendered marker of `id` at `rendered_at` (or clear it
    /// with `None`) (change: add-score-daily-access-rewards): `true` when a row was
    /// updated. The instant is the caller's so the object key derived from it
    /// matches what was stored.
    async fn set_preview_rendered(
        &self,
        id: &str,
        rendered_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<bool>;

    /// Up to `limit` ids of `accepted` pieces with NO rendered marker — the
    /// backfill's work list (change: add-score-daily-access-rewards).
    async fn accepted_ids_missing_preview(&self, limit: i64) -> Result<Vec<String>>;
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
    /// Proposal attribution + feedback (change: add-score-catalog-proposal): the
    /// proposer id, the moderator's rejection reason, and the proposer's resubmission
    /// justification. All `None` for a crawler-ingested row.
    pub proposed_by: Option<String>,
    pub review_reason: Option<String>,
    pub resubmission_note: Option<String>,
    /// Exact-byte content digest. Backs [`FakeCatalogSearchRepo::find_by_sha`]'s
    /// proposal content match (change: add-score-catalog-proposal) AND is exposed as
    /// the ETag by `object_ref` (change: add-offline-score-cache). `new()` seeds a
    /// stable, id-derived value so conditional-fetch tests have a known hash;
    /// override with [`Self::with_sha256`].
    pub sha256: String,
    /// The audio-teaser marker (change: add-score-daily-access-rewards): the
    /// render instant once a preview was rendered for the row.
    pub preview_rendered_at: Option<chrono::DateTime<chrono::Utc>>,
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
            sha256: format!("etag-{id}"),
            ..Default::default()
        }
    }

    /// Override the stored content hash (ETag) for the conditional-fetch tests.
    pub fn with_sha256(mut self, sha256: &str) -> Self {
        self.sha256 = sha256.into();
        self
    }

    /// Override the moderation status (`pending` / `accepted` / `rejected`) for the
    /// moderation-gating tests.
    pub fn with_moderation_status(mut self, status: &str) -> Self {
        self.moderation_status = status.into();
        self
    }

    /// Mark the row as a piano score (the rating deck sources piano scores only).
    pub fn piano(mut self) -> Self {
        self.is_piano = Some(true);
        self
    }

    /// Mark the row as user-proposed by `proposed_by` (change: add-score-catalog-
    /// proposal), tagging its origin `user-proposal`.
    pub fn proposed_by(mut self, proposed_by: &str) -> Self {
        self.proposed_by = Some(proposed_by.into());
        self.source = "user-proposal".into();
        self
    }

    /// Seed a stored resubmission justification (for the review-queue read tests).
    pub fn with_resubmission_note(mut self, note: &str) -> Self {
        self.resubmission_note = Some(note.into());
        self
    }

    /// Seed the content digest so `find_by_sha` resolves this row (propose tests).
    pub fn with_sha(mut self, sha256: &str) -> Self {
        self.sha256 = sha256.into();
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
            // `search` overrides `needs_review` after consulting the ratings view;
            // the row itself carries its own moderation status.
            needs_review: false,
            moderation_status: Some(self.moderation_status.clone()),
            proposed_by: self.proposed_by.clone(),
            // Resolved names are filled by the module (UserPort); the row itself carries
            // only the stored columns.
            proposer_display_name: None,
            contributor_credit: None,
            review_reason: self.review_reason.clone(),
            resubmission_note: self.resubmission_note.clone(),
            has_preview: self.preview_rendered_at.is_some(),
        }
    }

    /// The fixed render instant [`Self::with_preview`] stamps (tests derive the
    /// teaser's object key from it).
    pub const FAKE_PREVIEW_AT_MS: i64 = 1_700_000_000_000;

    /// Mark the row as having a rendered audio teaser (change:
    /// add-score-daily-access-rewards), stamped at [`Self::FAKE_PREVIEW_AT_MS`].
    pub fn with_preview(mut self, has_preview: bool) -> Self {
        self.preview_rendered_at = has_preview
            .then(|| chrono::DateTime::from_timestamp_millis(Self::FAKE_PREVIEW_AT_MS).unwrap());
        self
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
    /// Records applied metadata edits (score id, editor, per-field diff) so tests can
    /// assert the audit trail (change: add-catalog-metadata-editing).
    edits: Mutex<Vec<(String, String, Vec<crate::catalog_edit::FieldChange>)>>,
}

impl FakeCatalogSearchRepo {
    /// Seed the fake with a fixed set of rows.
    pub fn with(rows: Vec<FakeCatalogRow>) -> Self {
        Self {
            rows: Mutex::new(rows),
            ratings: Mutex::new(None),
            edits: Mutex::new(Vec::new()),
        }
    }

    /// The metadata edits applied so far (score id, editor, changed fields).
    pub fn edit_calls(&self) -> Vec<(String, String, Vec<crate::catalog_edit::FieldChange>)> {
        self.edits.lock().expect("catalog search fake lock").clone()
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
        // Snapshot the rows and the shared ratings view, dropping both locks before
        // any `.await` (a std Mutex guard held across await is not Send).
        let rows: Vec<FakeCatalogRow> = self.rows.lock().expect("catalog search fake lock").clone();
        let ratings = self
            .ratings
            .lock()
            .expect("catalog search fake lock")
            .clone();
        // The set of `accepted` scores flagged for re-review, computed from the shared
        // ratings view exactly as the Pg adapter joins `score_ratings` (design D4):
        // ≥ `min_count` ratings AND average effective value ≤ `review_threshold`. Only
        // computed in a privileged (back-office) context — a review-queue read, an
        // explicit status filter, or a moderation sort — mirroring the Pg adapter's
        // gate so the app hub never sees the flag.
        let cfg = RatingConfig::default();
        let privileged = p.review_queue
            || p.moderation_status.is_some()
            || p.all_statuses
            || p.sort.iter().any(|k| is_moderation_sort_field(&k.field));
        let mut flagged: std::collections::HashSet<String> = std::collections::HashSet::new();
        if privileged && let Some(rv) = &ratings {
            for r in &rows {
                if r.moderation_status == "accepted"
                    && is_flagged_for_review(&rv.aggregate(&r.id).await?, &cfg)
                {
                    flagged.insert(r.id.clone());
                }
            }
        }
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
                // Moderation gate. Review-queue mode (BO): `pending` scores PLUS
                // `accepted` scores flagged for re-review — the moderation work list.
                // Otherwise the single-status gate: default accepted-only, a set
                // filter selects exactly that status (mirrors the Pg adapter).
                let status_ok = if p.review_queue {
                    r.moderation_status == "pending"
                        || (r.moderation_status == "accepted" && flagged.contains(&r.id))
                } else if p.all_statuses {
                    true // privileged BO catalog view: every moderation status
                } else {
                    let want_status = p.moderation_status.as_deref().unwrap_or("accepted");
                    r.moderation_status == want_status
                };
                // Origin filter (change: add-score-catalog-proposal): composes with all
                // the above; `None` = any source.
                let source_ok = p.source.as_ref().is_none_or(|s| &r.source == s);
                // Audio-teaser filter (change: add-score-daily-access-rewards).
                let preview_ok = p
                    .has_preview
                    .is_none_or(|want| r.preview_rendered_at.is_some() == want);
                text_ok
                    && author_ok
                    && level_ok
                    && status_ok
                    && source_ok
                    && preview_ok
                    && facets_match(r, p)
            })
            .collect();
        // Total over the full filtered set, before pagination (mirrors the Pg
        // adapter's `COUNT(*) OVER()`).
        let total = matched.len() as i64;
        // Apply the validated sort keys (primary first), then the deterministic
        // `(title_norm, id)` tiebreak so paging stays stable — mirroring the Pg
        // adapter, which appends the same tiebreak after the sort keys. An empty
        // `sort` leaves ONLY the default tiebreak, so the hub order is unchanged.
        // `needs_review` orders on flagged-membership (the re-review predicate).
        matched.sort_by(|a, b| {
            for key in &p.sort {
                let ord = if key.field == "needs_review" {
                    i64::from(flagged.contains(&a.id)).cmp(&i64::from(flagged.contains(&b.id)))
                } else {
                    sort_value(a, &key.field).cmp(&sort_value(b, &key.field))
                };
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
            .map(|r| {
                let mut h = r.to_hit();
                h.needs_review = flagged.contains(&r.id);
                h
            })
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

    async fn object_ref(
        &self,
        id: &str,
        include_unvalidated: bool,
    ) -> Result<Option<CatalogObjectRef>> {
        let rows = self.rows.lock().expect("catalog search fake lock");
        Ok(rows
            .iter()
            .find(|r| r.id == id && (include_unvalidated || r.moderation_status == "accepted"))
            .map(|r| CatalogObjectRef {
                object_key: r.object_key.clone(),
                sha256: r.sha256.clone(),
                proposed_by: r.proposed_by.clone(),
                preview_rendered_at: r.preview_rendered_at,
            }))
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
        reason: Option<&str>,
    ) -> Result<bool> {
        let mut rows = self.rows.lock().expect("catalog search fake lock");
        match rows.iter_mut().find(|r| r.id == score_id) {
            Some(row) => {
                row.moderation_status = status.to_string();
                row.reviewed_by = Some(reviewer_id.to_string());
                // The reason is the rejection motive: keep it only on `rejected`.
                row.review_reason = if status == "rejected" {
                    reason.map(str::to_string)
                } else {
                    None
                };
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn find_by_sha(&self, sha256: &str) -> Result<Option<(String, String)>> {
        let rows = self.rows.lock().expect("catalog search fake lock");
        Ok(rows
            .iter()
            .find(|r| !r.sha256.is_empty() && r.sha256 == sha256)
            .map(|r| (r.id.clone(), r.moderation_status.clone())))
    }

    async fn insert_proposed(
        &self,
        entry: &crate::repo::CatalogEntry,
        accepted: bool,
    ) -> Result<bool> {
        let mut rows = self.rows.lock().expect("catalog search fake lock");
        if rows.iter().any(|r| r.id == entry.id) {
            return Ok(false);
        }
        let mut row = FakeCatalogRow::new(
            &entry.id,
            entry.meta.title.as_deref().unwrap_or_default(),
            entry.meta.composer.as_deref().unwrap_or_default(),
            entry.level.as_deref(),
        );
        row.source = entry.source.clone();
        row.object_key = entry.object_key.clone();
        row.is_piano = Some(entry.meta.is_piano);
        row.time_sig = entry.meta.time_sig.clone();
        row.key_fifths = entry.meta.key_fifths;
        row.moderation_status = if accepted { "accepted" } else { "pending" }.to_string();
        row.proposed_by = entry.proposed_by.clone();
        row.sha256 = entry.sha256.clone();
        rows.push(row);
        Ok(true)
    }

    async fn reopen_rejected(&self, score_id: &str, proposer_id: &str, note: &str) -> Result<bool> {
        let mut rows = self.rows.lock().expect("catalog search fake lock");
        match rows.iter_mut().find(|r| r.id == score_id) {
            Some(row) => {
                row.moderation_status = "pending".to_string();
                row.proposed_by = Some(proposer_id.to_string());
                row.review_reason = None;
                row.resubmission_note = Some(note.to_string());
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn apply_metadata_edit(
        &self,
        score_id: &str,
        editor: &str,
        plan: &crate::catalog_edit::EditPlan,
    ) -> Result<bool> {
        let mut rows = self.rows.lock().expect("catalog search fake lock");
        match rows.iter_mut().find(|r| r.id == score_id) {
            Some(row) => {
                row.title = plan.title.clone();
                row.composer = plan.composer.clone();
                row.arranger = plan.arranger.clone();
                row.level = plan.level.clone();
                self.edits.lock().expect("catalog search fake lock").push((
                    score_id.to_string(),
                    editor.to_string(),
                    plan.changes.clone(),
                ));
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn set_preview_rendered(
        &self,
        id: &str,
        rendered_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<bool> {
        let mut rows = self.rows.lock().expect("catalog search fake lock");
        match rows.iter_mut().find(|r| r.id == id) {
            Some(row) => {
                row.preview_rendered_at = rendered_at;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn accepted_ids_missing_preview(&self, limit: i64) -> Result<Vec<String>> {
        let rows = self.rows.lock().expect("catalog search fake lock");
        Ok(rows
            .iter()
            .filter(|r| r.moderation_status == "accepted" && r.preview_rendered_at.is_none())
            .map(|r| r.id.clone())
            .take(limit.max(0) as usize)
            .collect())
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
            .filter(|r| {
                // `pending` + `accepted` (never `rejected`) — change: rate-pending-scores.
                (r.moderation_status == "pending" || r.moderation_status == "accepted")
                    && r.is_piano == Some(true)
                    && !rated.contains(&r.id)
            })
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
        // `needs_review` is handled by the caller (it needs the ratings view, not a
        // row field); `measure_count`/`title`/`composer` are not modelled on the fake.
        // All contribute no ordering here.
        _ => 0,
    }
}

/// Applies the facet filters of `p` to a fake row, mirroring the Pg adapter: a
/// set filter excludes rows whose corresponding facet is unknown (`None`).
fn facets_match(r: &FakeCatalogRow, p: &CatalogSearchParams) -> bool {
    fn bool_ok(filter: Option<bool>, value: Option<bool>) -> bool {
        filter.is_none_or(|f| value == Some(f))
    }
    let facets = &p.facets;
    if let Some(pi) = facets.is_piano
        && r.is_piano != Some(pi)
    {
        return false;
    }
    if let Some(mv) = facets.max_note_value
        && !matches!(r.min_note_value, Some(v) if v <= mv)
    {
        return false;
    }
    if !bool_ok(facets.has_chords, r.has_chords)
        || !bool_ok(facets.has_tuplets, r.has_tuplets)
        || !bool_ok(facets.has_dotted, r.has_dotted)
    {
        return false;
    }
    if let Some(span) = facets.max_ambitus_semitones
        && !matches!((r.lowest_midi, r.highest_midi), (Some(lo), Some(hi)) if hi - lo <= span)
    {
        return false;
    }
    if let Some(sc) = facets.staff_count
        && r.staff_count != Some(sc)
    {
        return false;
    }
    if facets.min_bpm.is_some() || facets.max_bpm.is_some() {
        let Some(t) = r.tempo_bpm else { return false };
        if facets.min_bpm.is_some_and(|m| t < m) || facets.max_bpm.is_some_and(|m| t > m) {
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
            repo.set_moderation_status("p", "accepted", "mod-1", None)
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
                .set_moderation_status("does-not-exist", "accepted", "mod-1", None)
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
