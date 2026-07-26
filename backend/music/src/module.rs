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

//! The user-upload business logic (design 2b / 4 / 9).
//!
//! Every upload is: **check client inputs → quota → re-validate & re-derive the
//! bytes (never trusting the client) → store canonical bytes → persist an
//! owner-attributed record**. Descriptive metadata comes from the server's own
//! parse; a client-supplied fallback title/composer is used ONLY when the file
//! itself carries none (a parsed value always wins — no spoofing). The client
//! otherwise supplies bytes + level + the rights attestation. Reads, deletes and
//! the quota count are all owner-scoped.

use std::sync::Arc;

use cymbra_musicxml_core::{ScoreSummary, mxl, normalize_text, validate};
use cymbra_platform::{AppError, Result};
use cymbra_storage::{ObjectStorage, StorageError};
use sha2::{Digest, Sha256};

use crate::catalog_search::{CatalogHit, CatalogQuery, CatalogSearchParams, CatalogSearchRepo};
use crate::repo::{ScoreFacets, ScoreMeta};
use crate::user_library::UserLibraryRepo;
use crate::user_scores::{UserScore, UserScoreRepo};

const LEVELS: [&str; 3] = ["beginner", "intermediate", "advanced"];
const RIGHTS_BASES: [&str; 2] = ["own_work", "public_domain"];

/// The allowed moderation statuses for the privileged search filter (change:
/// add-score-moderation-gating). A caller-supplied status outside this set is
/// rejected before the query runs.
const MODERATION_STATUSES: [&str; 3] = ["pending", "accepted", "rejected"];

/// Server maximum page size for catalog search — a caller-supplied limit is
/// clamped to `[1, SEARCH_MAX_LIMIT]` so one request can never scan the corpus.
const SEARCH_MAX_LIMIT: i64 = 50;

/// Valid note-value denominators for the rhythmic-granularity filter
/// (`4`=quarter … `128`=128th).
const NOTE_VALUE_DENOMINATORS: [i16; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

/// The caller-supplied part of an upload (identity comes separately, from auth).
pub struct UploadInput {
    pub data: Vec<u8>,
    pub filename: String,
    pub level: String,
    pub rights_basis: String,
    pub rights_ack: bool,
    /// Fallback title/composer — used ONLY when the parsed file has none
    /// (design 2b): a parsed value always wins, so these cannot override or spoof
    /// real metadata.
    pub fallback_title: Option<String>,
    pub fallback_composer: Option<String>,
}

/// Trim, drop-if-empty, and cap a client-supplied fallback string.
fn clean_fallback(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().chars().take(200).collect::<String>())
        .filter(|v| !v.is_empty())
}

/// User-upload logic + catalog search / saved library, over owner-scoped repos,
/// the public-catalog read port, and the object store.
pub struct ScoreModule {
    repo: Arc<dyn UserScoreRepo>,
    catalog: Arc<dyn CatalogSearchRepo>,
    library: Arc<dyn UserLibraryRepo>,
    storage: Arc<dyn ObjectStorage>,
    quota_max: u32,
    quota_window_days: u32,
    max_bytes: usize,
}

impl ScoreModule {
    pub fn new(
        repo: Arc<dyn UserScoreRepo>,
        catalog: Arc<dyn CatalogSearchRepo>,
        library: Arc<dyn UserLibraryRepo>,
        storage: Arc<dyn ObjectStorage>,
        quota_max: u32,
        quota_window_days: u32,
        max_bytes: usize,
    ) -> Self {
        Self {
            repo,
            catalog,
            library,
            storage,
            quota_max,
            quota_window_days,
            max_bytes,
        }
    }

    /// Validate, re-derive, enforce the quota, store, and persist. Returns the
    /// stored record. `owner_id` is the authenticated caller.
    pub async fn upload(&self, owner_id: &str, input: UploadInput) -> Result<UserScore> {
        // 1. Client inputs the server owns the truth of (design 2b).
        if !input.rights_ack {
            return Err(AppError::InvalidArgument(
                "rights acknowledgement is required".into(),
            ));
        }
        if !LEVELS.contains(&input.level.as_str()) {
            return Err(AppError::InvalidArgument(format!(
                "unknown level {:?}",
                input.level
            )));
        }
        if !RIGHTS_BASES.contains(&input.rights_basis.as_str()) {
            return Err(AppError::InvalidArgument(format!(
                "unknown rights basis {:?}",
                input.rights_basis
            )));
        }
        if input.data.len() > self.max_bytes {
            return Err(AppError::InvalidArgument("file is too large".into()));
        }

        // 2. Quota — before any validation/storage work (design 9).
        let recent = self
            .repo
            .count_recent(owner_id, self.quota_window_days)
            .await?;
        if recent >= self.quota_max as i64 {
            return Err(AppError::ResourceExhausted(format!(
                "upload quota reached ({} per {} days)",
                self.quota_max, self.quota_window_days
            )));
        }

        // 3. Re-validate the received bytes and re-derive metadata (never trust
        //    the client's parse). The summary IS the stored metadata.
        let summary: ScoreSummary = validate(&input.data).map_err(|r| {
            AppError::InvalidArgument(format!("invalid score ({}): {}", r.code(), r))
        })?;

        // 3b. Fallback title/composer: fill only what the file itself lacks; a
        //     parsed value always wins (design 2b). Re-derive the search keys from
        //     the effective title/composer so they stay consistent.
        let title = summary
            .title
            .clone()
            .or_else(|| clean_fallback(input.fallback_title));
        let composer = summary
            .composer
            .clone()
            .or_else(|| clean_fallback(input.fallback_composer));

        // A title is mandatory: reject a file with no `<work-title>` and no
        // fallback rather than storing an untitled score (the client gates this
        // too, but the server is the real guard — design 2b).
        if title.is_none() {
            return Err(AppError::InvalidArgument("a title is required".into()));
        }

        let title_norm = title.as_deref().map(cymbra_musicxml_core::normalize_text);
        let composer_norm = composer
            .as_deref()
            .map(cymbra_musicxml_core::normalize_text)
            .unwrap_or_default();
        let work_key = format!(
            "{}::{}",
            composer_norm,
            title_norm.clone().unwrap_or_default()
        );

        // 4. Canonical bytes = the decoded MusicXML (so a re-zip of the same piece
        //    dedups); the read path decodes `.mxl` transparently, so we store the
        //    plain XML. sha256 over the canonical form is the per-owner dedup key.
        let canonical = decode_canonical(&input.data)?;
        let sha = sha256_hex(&canonical);

        // 4b. Derive the musical facets from the canonical parse (same as the
        //     crawler does for the catalog), so uploads carry them too. A parse
        //     failure here is impossible after `validate`, but default rather than
        //     abort if it somehow occurs.
        let core_facets = cymbra_musicxml_core::parse(&canonical)
            .map(|doc| cymbra_musicxml_core::ScoreFacets::from_document(&doc))
            .unwrap_or_default();
        let facets = ScoreFacets::from_core(&core_facets);

        // 5. Store the object, then persist the row (design 4: object before row;
        //    an orphaned object is invisible/reclaimable, a dangling row is not).
        let id = uuid::Uuid::now_v7().to_string();
        let object_key = format!("user-scores/{owner_id}/{id}.musicxml");
        self.storage
            .put(&object_key, canonical)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("store score: {e}")))?;

        let record = UserScore {
            id,
            owner_id: owner_id.to_string(),
            level: input.level,
            rights_basis: input.rights_basis,
            rights_ack: true,
            sha256: sha,
            size_bytes: input.data.len() as i64,
            object_key: object_key.clone(),
            created_at: now_unix(),
            favorite: true, // a new upload lands in the caller's favorites
            meta: ScoreMeta {
                title,
                composer,
                title_norm,
                work_key,
                key_fifths: summary.key_fifths,
                time_sig: summary.time_sig,
                measure_count: summary.measure_count as i32,
                is_piano: summary.is_piano,
                facets,
            },
        };

        if let Err(e) = self.repo.insert(&record).await {
            // Row write lost the race (e.g. a duplicate): reclaim the object we
            // just wrote so it does not linger as an orphan.
            let _ = self.storage.delete(&object_key).await;
            return Err(e);
        }
        Ok(record)
    }

    /// The caller's own contributed scores, newest first.
    pub async fn list(&self, owner_id: &str) -> Result<Vec<UserScore>> {
        self.repo.list_by_owner(owner_id).await
    }

    /// Favorite / un-favorite one of the caller's own uploads (change:
    /// favorites-home). Un-favoriting hides it from the home but keeps the
    /// upload (still in the hub's "mes partitions").
    pub async fn set_favorite(&self, owner_id: &str, id: &str, favorite: bool) -> Result<()> {
        self.repo.set_favorite(id, owner_id, favorite).await
    }

    /// Delete a score the caller owns: remove the row, then best-effort delete the
    /// object. The row is the source of truth; a failed object delete leaves a
    /// reclaimable orphan (an idempotent cleanup job is wired in task 4.1).
    pub async fn delete(&self, owner_id: &str, id: &str) -> Result<()> {
        let removed = self.repo.delete_owned(id, owner_id).await?;
        let Some(row) = removed else {
            return Err(AppError::NotFound("score not found".into()));
        };
        let _ = self.storage.delete(&row.object_key).await;
        Ok(())
    }

    /// Fetch the bytes of a score the caller owns (for the player).
    pub async fn get_bytes(&self, owner_id: &str, id: &str) -> Result<Vec<u8>> {
        let row = self
            .repo
            .get_owned(id, owner_id)
            .await?
            .ok_or_else(|| AppError::NotFound("score not found".into()))?;
        self.storage
            .get(&row.object_key)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("read score: {e}")))
    }

    // --- catalog search + saved library (change: score-hub-search) ----------
    // The public catalog is not owner-scoped, but every op requires an
    // authenticated caller (enforced at the gRPC interceptor); save/remove/list
    // are owner-scoped by `owner_id`.

    /// Search the public catalog by free-text (title/composer), an optional
    /// author (composer) filter, and an optional difficulty — all composed
    /// conjunctively. The query/author are accent/case-folded here so they match
    /// the persisted normalised columns; `level` is validated against the fixed
    /// set; `limit` is clamped to the server maximum.
    pub async fn search_catalog(&self, q: CatalogQuery) -> Result<(Vec<CatalogHit>, i64)> {
        if let Some(l) = q.level.as_deref()
            && !LEVELS.contains(&l)
        {
            return Err(AppError::InvalidArgument(format!("unknown level {l:?}")));
        }
        if let Some(v) = q.max_note_value
            && !NOTE_VALUE_DENOMINATORS.contains(&v)
        {
            return Err(AppError::InvalidArgument(format!(
                "unknown note-value denominator {v}"
            )));
        }
        if let Some(s) = q.staff_count
            && !(1..=2).contains(&s)
        {
            return Err(AppError::InvalidArgument(format!(
                "unknown staff count {s}"
            )));
        }
        // Privileged moderation-status filter: validate the value (the gRPC layer
        // has already authorised the caller). `None` keeps the accepted-only default.
        if let Some(s) = q.moderation_status.as_deref()
            && !MODERATION_STATUSES.contains(&s)
        {
            return Err(AppError::InvalidArgument(format!(
                "unknown moderation status {s:?}"
            )));
        }
        // Validate every sort field against the allow-list; an unknown field is
        // rejected and the query does not run (change: add-moderation-back-office).
        // Privilege of moderation-oriented keys is enforced at the gRPC layer.
        for key in &q.sort {
            if crate::catalog_search::sort_field_privilege(&key.field).is_none() {
                return Err(AppError::InvalidArgument(format!(
                    "unknown sort field {:?}",
                    key.field
                )));
            }
        }
        // Normalise text/author; an empty author (after fold) imposes no filter.
        let author_norm = q
            .author
            .as_deref()
            .map(normalize_text)
            .filter(|a: &String| !a.is_empty());
        let params = CatalogSearchParams {
            text_norm: normalize_text(&q.query),
            author_norm,
            level: q.level,
            is_piano: q.is_piano,
            max_note_value: q.max_note_value,
            has_chords: q.has_chords,
            has_tuplets: q.has_tuplets,
            has_dotted: q.has_dotted,
            max_ambitus_semitones: q.max_ambitus_semitones,
            staff_count: q.staff_count,
            min_bpm: q.min_bpm,
            max_bpm: q.max_bpm,
            moderation_status: q.moderation_status,
            sort: q.sort,
            limit: q.limit.clamp(1, SEARCH_MAX_LIMIT),
            offset: q.offset.max(0),
        };
        self.catalog.search(&params).await
    }

    /// Evaluate a catalog score (change: add-moderation-back-office): set its
    /// moderation status and stamp the reviewer + time. Restricted to
    /// moderator/admin at the gRPC layer; here we validate the target status and
    /// treat an unknown score id as not-found. `reviewer_id` is the authenticated
    /// caller. Returns once the single conditional UPDATE has run.
    pub async fn set_moderation_status(
        &self,
        reviewer_id: &str,
        score_id: &str,
        status: &str,
    ) -> Result<()> {
        if !MODERATION_STATUSES.contains(&status) {
            return Err(AppError::InvalidArgument(format!(
                "unknown moderation status {status:?}"
            )));
        }
        let updated = self
            .catalog
            .set_moderation_status(score_id, status, reviewer_id)
            .await?;
        if !updated {
            return Err(AppError::NotFound("catalog score not found".into()));
        }
        Ok(())
    }

    /// Save a public catalog score to the caller's library. Validates the catalog
    /// id exists first, then records an idempotent owner-scoped save.
    pub async fn save_catalog_score(&self, owner_id: &str, catalog_id: &str) -> Result<()> {
        // A normal caller can only save a validated (`accepted`) score — an
        // unvalidated id resolves as absent, matching what search exposes.
        if self.catalog.object_key(catalog_id, false).await?.is_none() {
            return Err(AppError::NotFound("catalog score not found".into()));
        }
        self.library.save(owner_id, catalog_id).await
    }

    /// Remove a saved catalog score from the caller's library (idempotent no-op if
    /// it was not saved). Never touches the public catalog entry.
    pub async fn remove_saved_catalog_score(&self, owner_id: &str, catalog_id: &str) -> Result<()> {
        self.library.remove(owner_id, catalog_id).await
    }

    /// The caller's saved catalog scores, newest-saved first. Joins the saved ids
    /// to the catalog and omits any whose entry is gone (e.g. after a re-ingest),
    /// so a stale save is never surfaced as a broken row.
    pub async fn list_saved_catalog_scores(&self, owner_id: &str) -> Result<Vec<CatalogHit>> {
        let ids = self.library.list_ids(owner_id).await?;
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let hits = self.catalog.hits_by_ids(&ids).await?;
        // Preserve the saved (newest-first) order and drop missing entries.
        let mut by_id: std::collections::HashMap<String, CatalogHit> =
            hits.into_iter().map(|h| (h.id.clone(), h)).collect();
        Ok(ids.iter().filter_map(|id| by_id.remove(id)).collect())
    }

    /// Fetch the canonical bytes of a public catalog score by id, from the object
    /// store, so the app can open it in the player. Not owner-scoped (public
    /// corpus); a non-existent id is a typed not-found.
    ///
    /// Catalog objects are stored as compressed `.mxl` (the crawler's output),
    /// but the app's parser consumes **uncompressed** MusicXML — so decode the
    /// `.mxl` container transparently here, exactly as the upload path does before
    /// storing. A plain-XML object passes straight through.
    /// `allow_unvalidated` reflects the caller's authorisation (change:
    /// add-score-moderation-gating): a normal caller (`false`) is served only
    /// `accepted` scores — a `pending`/`rejected` id is a typed not-found, so it
    /// can't be opened by id; an authorised reviewer (`true`) is served a score in
    /// any status so they can evaluate it.
    pub async fn get_catalog_bytes(
        &self,
        catalog_id: &str,
        allow_unvalidated: bool,
    ) -> Result<Vec<u8>> {
        let object_key = self
            .catalog
            .object_key(catalog_id, allow_unvalidated)
            .await?
            .ok_or_else(|| AppError::NotFound("catalog score not found".into()))?;
        let raw = self.storage.get(&object_key).await.map_err(|e| match e {
            // The catalog row exists but its bytes are not in the store yet (e.g.
            // a corpus not synced to the serving store yet). Report a distinct,
            // typed precondition failure so the app can say "not available yet"
            // rather than a generic internal error.
            StorageError::NotFound(_) => {
                AppError::FailedPrecondition("catalog score bytes not available yet".into())
            }
            other => AppError::Internal(anyhow::anyhow!("read catalog score: {other}")),
        })?;
        decode_canonical(&raw)
    }
}

/// The canonical MusicXML bytes: the decoded payload for a `.mxl`, else the input.
fn decode_canonical(data: &[u8]) -> Result<Vec<u8>> {
    if mxl::is_mxl(data) {
        mxl::decode(data)
            .map_err(|e| AppError::InvalidArgument(format!("could not decode .mxl: {e}")))
    } else {
        Ok(data.to_vec())
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_storage::FakeStore;

    use crate::catalog_search::{FakeCatalogRow, FakeCatalogSearchRepo};
    use crate::user_library::FakeUserLibraryRepo;
    use crate::user_scores::FakeUserScoreRepo;

    const VALID: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <work><work-title>Test Piece</work-title></work>
  <identification><creator type="composer">A. Composer</creator></identification>
  <part-list><score-part id="P1"/></part-list>
  <part id="P1"><measure number="1">
    <attributes><divisions>1</divisions>
      <key><fifths>2</fifths></key>
      <time><beats>3</beats><beat-type>4</beat-type></time>
      <staves>2</staves>
    </attributes>
    <note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><staff>1</staff></note>
  </measure></part>
</score-partwise>"#;

    fn module(max: u32, window: u32) -> (ScoreModule, Arc<FakeUserScoreRepo>, Arc<FakeStore>) {
        let repo = Arc::new(FakeUserScoreRepo::default());
        let store = Arc::new(FakeStore::default());
        let catalog = Arc::new(FakeCatalogSearchRepo::default());
        let library = Arc::new(FakeUserLibraryRepo::default());
        let m = ScoreModule::new(
            repo.clone(),
            catalog,
            library,
            store.clone(),
            max,
            window,
            8 * 1024 * 1024,
        );
        (m, repo, store)
    }

    /// Module wired with a seeded catalog + an empty library, for the catalog
    /// search / saved-library tests.
    fn catalog_module() -> (
        ScoreModule,
        Arc<FakeCatalogSearchRepo>,
        Arc<FakeUserLibraryRepo>,
    ) {
        let repo = Arc::new(FakeUserScoreRepo::default());
        let store = Arc::new(FakeStore::default());
        let catalog = Arc::new(FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new(
                "11111111-1111-7111-8111-111111111111",
                "Clair de Lune",
                "Claude Debussy",
                Some("intermediate"),
            ),
            FakeCatalogRow::new(
                "22222222-2222-7222-8222-222222222222",
                "Gymnopédie",
                "Erik Satie",
                Some("beginner"),
            ),
            FakeCatalogRow::new(
                "33333333-3333-7333-8333-333333333333",
                "Prélude",
                "Claude Debussy",
                Some("advanced"),
            ),
        ]));
        let library = Arc::new(FakeUserLibraryRepo::default());
        let m = ScoreModule::new(
            repo,
            catalog.clone(),
            library.clone(),
            store,
            5,
            7,
            8 * 1024 * 1024,
        );
        (m, catalog, library)
    }

    fn input(data: &str, level: &str, basis: &str, ack: bool) -> UploadInput {
        UploadInput {
            data: data.as_bytes().to_vec(),
            filename: "x.musicxml".into(),
            level: level.into(),
            rights_basis: basis.into(),
            rights_ack: ack,
            fallback_title: None,
            fallback_composer: None,
        }
    }

    /// A valid piano score carrying NO title/composer (for the fallback tests).
    const NO_META: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"/></part-list>
  <part id="P1"><measure number="1">
    <attributes><divisions>1</divisions><staves>2</staves></attributes>
    <note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><staff>1</staff></note>
  </measure></part>
</score-partwise>"#;

    #[tokio::test]
    async fn fallback_fills_missing_title_but_never_overrides_a_parsed_one() {
        let (m, repo, _store) = module(5, 7);
        // File has no title/composer → the fallbacks are used, and the search
        // keys are re-derived from them.
        let mut i = input(NO_META, "beginner", "own_work", true);
        i.fallback_title = Some("  My Untitled Piece  ".into());
        i.fallback_composer = Some("Me".into());
        let rec = m.upload("u1", i).await.unwrap();
        assert_eq!(rec.meta.title.as_deref(), Some("My Untitled Piece")); // trimmed
        assert_eq!(rec.meta.composer.as_deref(), Some("Me"));
        assert_eq!(rec.meta.work_key, "me::my untitled piece"); // normalized keys
        assert_eq!(rec.meta.title_norm.as_deref(), Some("my untitled piece"));

        // A file WITH a title: the fallback is ignored (parsed wins — design 2b).
        let mut i2 = input(VALID, "beginner", "own_work", true);
        i2.fallback_title = Some("Spoofed Title".into());
        let rec2 = m.upload("u1", i2).await.unwrap();
        assert_eq!(rec2.meta.title.as_deref(), Some("Test Piece"));
        assert_eq!(repo.rows().len(), 2);
    }

    #[tokio::test]
    async fn upload_rejects_a_file_with_no_title_and_no_fallback() {
        let (m, repo, store) = module(5, 7);
        // File carries no <work-title> and the user typed no fallback title.
        assert!(matches!(
            m.upload("u1", input(NO_META, "beginner", "own_work", true))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        // Nothing stored, nothing persisted.
        assert!(store.is_empty());
        assert!(repo.rows().is_empty());
    }

    #[tokio::test]
    async fn upload_validates_derives_stores_and_persists() {
        let (m, repo, store) = module(5, 7);
        let rec = m
            .upload("u1", input(VALID, "intermediate", "own_work", true))
            .await
            .unwrap();
        // Server-derived metadata (client sent none).
        assert_eq!(rec.meta.title.as_deref(), Some("Test Piece"));
        assert_eq!(rec.meta.composer.as_deref(), Some("A. Composer"));
        assert_eq!(rec.meta.key_fifths, 2);
        assert_eq!(rec.meta.time_sig, "3/4");
        assert!(rec.meta.is_piano);
        assert_eq!(rec.level, "intermediate");
        assert_eq!(
            rec.object_key,
            format!("user-scores/u1/{}.musicxml", rec.id)
        );
        // Object stored + row persisted.
        assert!(store.contains(&rec.object_key));
        assert_eq!(repo.rows().len(), 1);
    }

    #[tokio::test]
    async fn upload_rejects_bad_inputs_without_storing() {
        let (m, repo, store) = module(5, 7);
        // Missing ack, bad level, bad basis, unparseable bytes.
        assert!(matches!(
            m.upload("u1", input(VALID, "intermediate", "own_work", false))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            m.upload("u1", input(VALID, "expert", "own_work", true))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            m.upload("u1", input(VALID, "beginner", "stolen", true))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            m.upload("u1", input("<not-a-score/>", "beginner", "own_work", true))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(store.is_empty());
        assert!(repo.rows().is_empty());
    }

    #[tokio::test]
    async fn upload_enforces_the_quota_before_storing() {
        let (m, _repo, store) = module(2, 7);
        for i in 0..2 {
            // Distinct content so per-owner sha dedup doesn't interfere.
            let xml = VALID.replace("Test Piece", &format!("Piece {i}"));
            m.upload("u1", input(&xml, "beginner", "own_work", true))
                .await
                .unwrap();
        }
        let third = VALID.replace("Test Piece", "Piece 3");
        assert!(matches!(
            m.upload("u1", input(&third, "beginner", "own_work", true))
                .await,
            Err(AppError::ResourceExhausted(_))
        ));
        assert_eq!(store.len(), 2); // the rejected one stored nothing
    }

    #[tokio::test]
    async fn oversized_upload_rejected_before_validation() {
        let (m, _repo, store) = module(5, 7);
        let mut m2 = m;
        m2.max_bytes = 10;
        assert!(matches!(
            m2.upload("u1", input(VALID, "beginner", "own_work", true))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(store.is_empty());
    }

    #[tokio::test]
    async fn list_delete_and_get_bytes_are_owner_scoped() {
        let (m, _repo, store) = module(5, 7);
        let rec = m
            .upload("u1", input(VALID, "beginner", "own_work", true))
            .await
            .unwrap();
        assert_eq!(m.list("u1").await.unwrap().len(), 1);
        assert!(m.list("u2").await.unwrap().is_empty());
        // Non-owner cannot read or delete.
        assert!(matches!(
            m.get_bytes("u2", &rec.id).await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            m.delete("u2", &rec.id).await,
            Err(AppError::NotFound(_))
        ));
        // Owner reads the canonical bytes back, then deletes (row + object).
        let bytes = m.get_bytes("u1", &rec.id).await.unwrap();
        assert!(!bytes.is_empty());
        m.delete("u1", &rec.id).await.unwrap();
        assert!(m.list("u1").await.unwrap().is_empty());
        assert!(!store.contains(&rec.object_key));
    }

    /// A store that stores/reads fine but always fails to delete — models a
    /// transient object-store fault during a delete.
    #[derive(Default)]
    struct DeleteFailsStore {
        inner: FakeStore,
    }

    #[async_trait::async_trait]
    impl ObjectStorage for DeleteFailsStore {
        async fn put(&self, key: &str, bytes: Vec<u8>) -> cymbra_storage::Result<()> {
            self.inner.put(key, bytes).await
        }
        async fn get(&self, key: &str) -> cymbra_storage::Result<Vec<u8>> {
            self.inner.get(key).await
        }
        async fn delete(&self, _key: &str) -> cymbra_storage::Result<()> {
            Err(cymbra_storage::StorageError::Backend(anyhow::anyhow!(
                "object store unavailable"
            )))
        }
    }

    #[tokio::test]
    async fn delete_removes_the_row_even_when_the_object_delete_fails() {
        // The row is the source of truth: a failed object delete must NOT fail the
        // request — it leaves a reclaimable orphan, but the record is gone.
        let repo = Arc::new(FakeUserScoreRepo::default());
        let store = Arc::new(DeleteFailsStore::default());
        let m = ScoreModule::new(
            repo.clone(),
            Arc::new(FakeCatalogSearchRepo::default()),
            Arc::new(FakeUserLibraryRepo::default()),
            store.clone(),
            5,
            7,
            8 * 1024 * 1024,
        );

        let rec = m
            .upload("u1", input(VALID, "beginner", "own_work", true))
            .await
            .unwrap();
        assert_eq!(repo.rows().len(), 1);

        // Object delete errors internally, but the call still succeeds…
        m.delete("u1", &rec.id).await.unwrap();
        // …and the row is gone (source of truth), while the orphan object remains.
        assert!(repo.rows().is_empty());
        assert!(store.inner.contains(&rec.object_key));
    }

    #[tokio::test]
    async fn duplicate_upload_is_rejected_and_leaves_no_orphan() {
        let (m, _repo, store) = module(5, 7);
        m.upload("u1", input(VALID, "beginner", "own_work", true))
            .await
            .unwrap();
        // Same bytes again → per-owner sha conflict; the just-written object is
        // reclaimed, so exactly one object remains.
        assert!(matches!(
            m.upload("u1", input(VALID, "beginner", "own_work", true))
                .await,
            Err(AppError::AlreadyExists(_))
        ));
        assert_eq!(store.len(), 1);
    }

    // --- catalog search + saved library ------------------------------------

    const DEBUSSY_1: &str = "11111111-1111-7111-8111-111111111111";
    const SATIE: &str = "22222222-2222-7222-8222-222222222222";
    const DEBUSSY_2: &str = "33333333-3333-7333-8333-333333333333";

    /// A [`CatalogQuery`] with just text/author/level set (facets unconstrained).
    fn q(query: &str, author: Option<&str>, level: Option<&str>, limit: i64) -> CatalogQuery {
        CatalogQuery {
            query: query.into(),
            author: author.map(Into::into),
            level: level.map(Into::into),
            limit,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn search_catalog_normalises_and_composes_filters() {
        let (m, _cat, _lib) = catalog_module();
        // Accent-insensitive composer match across two works, title_norm ordered.
        let (hits, _) = m
            .search_catalog(q("debussy", None, None, 50))
            .await
            .unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            [DEBUSSY_1, DEBUSSY_2]
        );
        // Author + difficulty compose conjunctively.
        let (hits, _) = m
            .search_catalog(q("", Some("Debussy"), Some("advanced"), 50))
            .await
            .unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            [DEBUSSY_2]
        );
    }

    #[tokio::test]
    async fn search_catalog_rejects_bad_inputs_and_clamps_limit() {
        let (m, _cat, _lib) = catalog_module();
        assert!(matches!(
            m.search_catalog(q("", None, Some("expert"), 50)).await,
            Err(AppError::InvalidArgument(_))
        ));
        // A note-value denominator outside the allowed set is rejected.
        let bad_nv = CatalogQuery {
            max_note_value: Some(7),
            limit: 50,
            ..Default::default()
        };
        assert!(matches!(
            m.search_catalog(bad_nv).await,
            Err(AppError::InvalidArgument(_))
        ));
        // A staff count outside 1..=2 is rejected.
        let bad_sc = CatalogQuery {
            staff_count: Some(3),
            limit: 50,
            ..Default::default()
        };
        assert!(matches!(
            m.search_catalog(bad_sc).await,
            Err(AppError::InvalidArgument(_))
        ));
        // A limit above the server max is clamped (3 rows exist, limit 999 → all).
        let (hits, _) = m.search_catalog(q("", None, None, 999)).await.unwrap();
        assert_eq!(hits.len(), 3);
        // A non-positive limit clamps up to 1.
        let (hits, _) = m.search_catalog(q("", None, None, 0)).await.unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[tokio::test]
    async fn search_catalog_facet_filters_compose_and_exclude_unknowns() {
        // Two rows with facets, one without (unknown) — filters must exclude the
        // unknown and compose conjunctively.
        let repo = Arc::new(FakeUserScoreRepo::default());
        let store = Arc::new(FakeStore::default());
        let catalog = Arc::new(FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new(DEBUSSY_1, "Fast", "X", Some("advanced")).with_facets(
                true,
                16,
                Some(140),
                (48, 84),
            ), // sixteenths, 140bpm, 3 octaves
            FakeCatalogRow::new(SATIE, "Slow", "Y", Some("beginner")).with_facets(
                true,
                8,
                Some(72),
                (60, 72),
            ), // eighths, 72bpm, 1 octave
            FakeCatalogRow::new(DEBUSSY_2, "Unknown", "Z", Some("beginner")), // no facets
        ]));
        let m = ScoreModule::new(
            repo,
            catalog,
            Arc::new(FakeUserLibraryRepo::default()),
            store,
            5,
            7,
            8 * 1024 * 1024,
        );

        // "Nothing faster than an eighth" keeps only the eighth-note row.
        let only_eighths = CatalogQuery {
            max_note_value: Some(8),
            limit: 50,
            ..Default::default()
        };
        let (hits, _) = m.search_catalog(only_eighths).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            [SATIE]
        );

        // Tempo range + ambitus compose; the unknown-facet row is excluded even
        // though it would pass on text.
        let slow_narrow = CatalogQuery {
            max_bpm: Some(100),
            max_ambitus_semitones: Some(12),
            limit: 50,
            ..Default::default()
        };
        let (hits, _) = m.search_catalog(slow_narrow).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            [SATIE]
        );
    }

    #[tokio::test]
    async fn save_validates_existence_then_lists_newest_first() {
        let (m, _cat, lib) = catalog_module();
        // Unknown catalog id is rejected and nothing is saved.
        assert!(matches!(
            m.save_catalog_score("u1", "99999999-9999-7999-8999-999999999999")
                .await,
            Err(AppError::NotFound(_))
        ));
        assert_eq!(lib.count("u1"), 0);

        m.save_catalog_score("u1", SATIE).await.unwrap();
        m.save_catalog_score("u1", DEBUSSY_1).await.unwrap();
        m.save_catalog_score("u1", DEBUSSY_1).await.unwrap(); // idempotent
        assert_eq!(lib.count("u1"), 2);

        // Saved list is newest-first and joined to the catalog.
        let saved = m.list_saved_catalog_scores("u1").await.unwrap();
        assert_eq!(
            saved.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            [DEBUSSY_1, SATIE]
        );
        assert_eq!(saved[0].composer.as_deref(), Some("Claude Debussy"));
    }

    #[tokio::test]
    async fn remove_is_a_no_op_when_not_saved_and_owner_scoped() {
        let (m, _cat, _lib) = catalog_module();
        m.save_catalog_score("u1", SATIE).await.unwrap();
        // Removing a not-saved score succeeds and changes nothing.
        m.remove_saved_catalog_score("u1", DEBUSSY_1).await.unwrap();
        assert_eq!(m.list_saved_catalog_scores("u1").await.unwrap().len(), 1);
        // Removing the saved one drops it; a re-list reflects that (sync source).
        m.remove_saved_catalog_score("u1", SATIE).await.unwrap();
        assert!(m.list_saved_catalog_scores("u1").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_saved_omits_entries_whose_catalog_row_is_gone() {
        let (m, cat, _lib) = catalog_module();
        m.save_catalog_score("u1", SATIE).await.unwrap();
        m.save_catalog_score("u1", DEBUSSY_1).await.unwrap();
        // Simulate a re-ingest that dropped the Satie row: the library still has
        // the save, but the join omits it rather than surfacing a broken entry.
        cat.set_rows(vec![FakeCatalogRow::new(
            DEBUSSY_1,
            "Clair de Lune",
            "Claude Debussy",
            Some("intermediate"),
        )]);
        let saved = m.list_saved_catalog_scores("u1").await.unwrap();
        assert_eq!(
            saved.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            [DEBUSSY_1]
        );
    }

    #[tokio::test]
    async fn get_catalog_bytes_rejects_unknown_id() {
        let (m, _cat, _lib) = catalog_module();
        assert!(matches!(
            m.get_catalog_bytes("99999999-9999-7999-8999-999999999999", false)
                .await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_catalog_bytes_when_object_missing_is_failed_precondition() {
        // The catalog row exists but its bytes are not in the store yet (the
        // fake store is empty) — a distinct, typed precondition failure so the
        // app can say "not available yet", not a generic internal error.
        let (m, _cat, _lib) = catalog_module();
        assert!(matches!(
            m.get_catalog_bytes("11111111-1111-7111-8111-111111111111", false)
                .await,
            Err(AppError::FailedPrecondition(_))
        ));
    }

    // --- moderation gating (change: add-score-moderation-gating) -------------

    const PENDING_ID: &str = "44444444-4444-7444-8444-444444444444";
    const REJECTED_ID: &str = "55555555-5555-7555-8555-555555555555";

    /// A module over a corpus of one accepted + one pending + one rejected score,
    /// with all three scores' bytes in the store, so both the search gate and the
    /// fetch-bytes gate can be exercised.
    async fn moderated_module() -> ScoreModule {
        let repo = Arc::new(FakeUserScoreRepo::default());
        let store = Arc::new(FakeStore::default());
        let rows = vec![
            FakeCatalogRow::new(
                DEBUSSY_1,
                "Clair de Lune",
                "Claude Debussy",
                Some("advanced"),
            ),
            FakeCatalogRow::new(PENDING_ID, "Pending Piece", "Anon", Some("beginner"))
                .with_moderation_status("pending"),
            FakeCatalogRow::new(REJECTED_ID, "Rejected Piece", "Anon", Some("beginner"))
                .with_moderation_status("rejected"),
        ];
        let catalog = Arc::new(FakeCatalogSearchRepo::with(rows));
        // Seed the object store with each row's bytes so a resolved key fetches.
        for id in [DEBUSSY_1, PENDING_ID, REJECTED_ID] {
            store
                .put(&format!("safe/pdmx/{id}.mxl"), b"<score/>".to_vec())
                .await
                .unwrap();
        }
        ScoreModule::new(
            repo,
            catalog,
            Arc::new(FakeUserLibraryRepo::default()),
            store,
            5,
            7,
            8 * 1024 * 1024,
        )
    }

    #[tokio::test]
    async fn search_catalog_hides_unvalidated_by_default_and_honours_privileged_filter() {
        let m = moderated_module().await;
        // Default (normal caller): only the accepted score.
        let (hits, _) = m.search_catalog(q("", None, None, 50)).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            [DEBUSSY_1]
        );
        // Privileged `pending` filter (already authorised at the gRPC layer).
        let pending = CatalogQuery {
            moderation_status: Some("pending".into()),
            limit: 50,
            ..Default::default()
        };
        let (hits, _) = m.search_catalog(pending).await.unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            [PENDING_ID]
        );
    }

    #[tokio::test]
    async fn search_catalog_rejects_an_unknown_moderation_status() {
        let m = moderated_module().await;
        let bad = CatalogQuery {
            moderation_status: Some("bogus".into()),
            limit: 50,
            ..Default::default()
        };
        assert!(matches!(
            m.search_catalog(bad).await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn get_catalog_bytes_gated_by_moderation_status() {
        let m = moderated_module().await;
        // Accepted bytes are served to a normal caller.
        assert_eq!(
            m.get_catalog_bytes(DEBUSSY_1, false).await.unwrap(),
            b"<score/>"
        );
        // Pending / rejected bytes are not-found for a normal caller…
        for id in [PENDING_ID, REJECTED_ID] {
            assert!(matches!(
                m.get_catalog_bytes(id, false).await,
                Err(AppError::NotFound(_))
            ));
            // …but an authorised reviewer (allow_unvalidated) is served them.
            assert_eq!(m.get_catalog_bytes(id, true).await.unwrap(), b"<score/>");
        }
    }

    #[tokio::test]
    async fn save_catalog_score_refuses_an_unvalidated_score() {
        let m = moderated_module().await;
        // A normal user cannot save a pending/rejected score (it's invisible to
        // them, exactly as search hides it).
        for id in [PENDING_ID, REJECTED_ID] {
            assert!(matches!(
                m.save_catalog_score("u1", id).await,
                Err(AppError::NotFound(_))
            ));
        }
        // The accepted one saves fine.
        m.save_catalog_score("u1", DEBUSSY_1).await.unwrap();
    }

    // --- evaluate + sort (change: add-moderation-back-office) ----------------

    #[tokio::test]
    async fn set_moderation_status_accepts_rejects_and_requeues() {
        let m = moderated_module().await;
        // Accept the pending score → it becomes visible in the default search.
        m.set_moderation_status("mod-1", PENDING_ID, "accepted")
            .await
            .unwrap();
        let (hits, _) = m.search_catalog(q("", None, None, 50)).await.unwrap();
        assert!(hits.iter().any(|h| h.id == PENDING_ID));
        // Re-queue it back to pending → it leaves the accepted-only hub again.
        m.set_moderation_status("mod-1", PENDING_ID, "pending")
            .await
            .unwrap();
        let (hits, _) = m.search_catalog(q("", None, None, 50)).await.unwrap();
        assert!(!hits.iter().any(|h| h.id == PENDING_ID));
    }

    #[tokio::test]
    async fn set_moderation_status_rejects_unknown_id_and_bad_status() {
        let m = moderated_module().await;
        assert!(matches!(
            m.set_moderation_status("mod-1", "99999999-9999-7999-8999-999999999999", "accepted")
                .await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            m.set_moderation_status("mod-1", PENDING_ID, "bogus").await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn search_catalog_rejects_an_unknown_sort_field() {
        let m = moderated_module().await;
        let bad = CatalogQuery {
            sort: vec![crate::catalog_search::SortKey {
                field: "not_a_column".into(),
                descending: true,
            }],
            limit: 50,
            ..Default::default()
        };
        assert!(matches!(
            m.search_catalog(bad).await,
            Err(AppError::InvalidArgument(_))
        ));
    }
}
