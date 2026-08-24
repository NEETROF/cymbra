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

//! One-off backfill: recompute catalog titles from each score's stored bytes.
//!
//! Older crawls let a git corpus's *filename* shadow the real title: OpenScore
//! Lieder files are named by an opaque id (`lc28971056`), so every such row was
//! ingested with `title = "lc28971056"` instead of the embedded `<work-title>`
//! (the crawler bug fixed in `score-crawler`'s `crawl.rs`). Re-running the crawler
//! does NOT repair them — ingest dedups on `sha256`, and the title fix leaves the
//! bytes unchanged, so every existing row is skipped. This backfill instead reads
//! each row's `.mxl` back from the object store, re-derives the title, and updates
//! it in place.
//!
//! Search correctness is the load-bearing constraint. The catalog search matches
//! the normalised `title_norm` column (never the display `title`), and the search
//! query is normalised through the very same [`normalize_text`] path. So the
//! backfill rewrites `title`, `title_norm` and `work_key` **together**, all from
//! one [`ScoreSummary::from_document`] — identical to what a corrected crawl would
//! have written — and a renamed score stays findable by its real title.
//!
//! The pure decision ([`plan_title_update`]) and the orchestration
//! ([`run_title_backfill`]) are host-testable with fakes; the Postgres data-access
//! ([`crate::pg::PgTitleBackfillRepo`]) is the only untested seam (thin SQL glue).

use anyhow::{Context, Result};
use async_trait::async_trait;
use cymbra_musicxml_core::{ScoreSummary, mxl};
use cymbra_storage::ObjectStorage;

/// A catalog row's minimal identity for the title backfill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackfillRow {
    pub id: String,
    pub object_key: String,
    /// The currently-stored display title (possibly the wrong filename id).
    pub title: Option<String>,
    /// Source-relative path of the original file (e.g. the Mutopia `.ly`). The
    /// stored-bytes title backfill ignores it; the Mutopia backfill locates the
    /// source `\header` with it (the converted MusicXML carries no title).
    pub source_item_id: String,
    /// The currently-stored composer. Read-only here — the Mutopia backfill reuses
    /// it to keep `work_key`/`composer_norm` consistent when only the title moves.
    pub composer: Option<String>,
}

/// The title triple to persist. Kept together so the display title (`title`) and
/// the search key (`title_norm`), plus the cross-source `work_key`, never diverge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleUpdate {
    pub title: String,
    pub title_norm: Option<String>,
    pub work_key: String,
}

/// Decide whether a row's title must be rewritten from its parsed score.
///
/// Returns `Some` only when the score embeds a real title that differs from
/// what's stored — so a score with no `<work-title>` (the adapter fallback is
/// correct) and an already-correct row are both left untouched. The
/// `title_norm`/`work_key` come from the SAME [`ScoreSummary`] derivation the
/// crawler and the search-query normalisation use, so the rewrite stays findable.
pub fn plan_title_update(current: Option<&str>, summary: &ScoreSummary) -> Option<TitleUpdate> {
    let parsed = summary.title.as_deref()?; // no embedded title → keep the fallback
    if Some(parsed) == current {
        return None; // already correct
    }
    Some(TitleUpdate {
        title: parsed.to_string(),
        title_norm: summary.title_norm.clone(),
        work_key: summary.work_key.clone(),
    })
}

/// Data-access seam for the backfill. The Postgres impl lives in
/// [`crate::pg::PgTitleBackfillRepo`]; a fake backs the orchestration tests.
#[async_trait]
pub trait TitleBackfillRepo: Send + Sync {
    /// One page of rows ordered by `id`, with `id > after` (keyset paging; `after`
    /// empty → from the start), up to `limit`. When `source` is `Some`, only rows
    /// from that provenance are returned (scope the scan to e.g. `openscore`).
    async fn page(&self, after: &str, source: Option<&str>, limit: i64)
    -> Result<Vec<BackfillRow>>;

    /// Persist the recomputed title triple for `id`.
    async fn update_title(&self, id: &str, update: &TitleUpdate) -> Result<()>;
}

/// Tallies for one backfill run.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BackfillReport {
    /// Rows examined.
    pub scanned: usize,
    /// Rows whose title was (or would be) rewritten.
    pub updated: usize,
    /// Rows with an embedded title already matching the stored one.
    pub unchanged: usize,
    /// Rows whose score carries no `<work-title>` — the adapter fallback is kept.
    pub no_title: usize,
    /// Rows skipped after a fetch/decode/parse/update failure (logged, never fatal).
    pub errors: usize,
}

/// Recompute titles for every catalog row (optionally scoped to one `source`) from
/// its stored `.mxl`. `apply == false` is a dry run: it plans and counts but writes
/// nothing. A failure on a single row is logged and skipped — one unreadable object
/// never aborts the run. Idempotent: a second run over corrected rows is all
/// `unchanged`/`no_title`.
pub async fn run_title_backfill(
    repo: &dyn TitleBackfillRepo,
    storage: &dyn ObjectStorage,
    source: Option<&str>,
    apply: bool,
    page_size: i64,
) -> Result<BackfillReport> {
    let mut report = BackfillReport::default();
    let mut after = String::new();
    loop {
        let rows = repo
            .page(&after, source, page_size)
            .await
            .context("paging catalog for backfill")?;
        let Some(last) = rows.last() else { break };
        after = last.id.clone();

        for row in rows {
            report.scanned += 1;
            let bytes = match storage.get(&row.object_key).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!(id = %row.id, key = %row.object_key, error = %e,
                        "backfill: object fetch failed, skipping");
                    report.errors += 1;
                    continue;
                }
            };
            let summary = match decode_summary(&bytes) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(id = %row.id, error = %e, "backfill: parse failed, skipping");
                    report.errors += 1;
                    continue;
                }
            };
            match plan_title_update(row.title.as_deref(), &summary) {
                // No embedded title: the stored adapter fallback stays.
                None if summary.title.is_none() => report.no_title += 1,
                // Embedded title already stored: nothing to do.
                None => report.unchanged += 1,
                Some(update) => {
                    if apply && let Err(e) = repo.update_title(&row.id, &update).await {
                        tracing::warn!(id = %row.id, error = %e,
                            "backfill: update failed, skipping");
                        report.errors += 1;
                        continue;
                    }
                    tracing::info!(id = %row.id, from = ?row.title, to = %update.title,
                        applied = apply, "backfill: title rewritten");
                    report.updated += 1;
                }
            }
        }
    }
    Ok(report)
}

/// Decode stored bytes → MusicXML → [`ScoreSummary`] (title/title_norm/work_key).
/// The catalog stores compressed `.mxl`, but plain XML is accepted too so the
/// backfill is robust to any legacy uncompressed row.
fn decode_summary(bytes: &[u8]) -> Result<ScoreSummary> {
    let inner = if mxl::is_mxl(bytes) {
        mxl::decode(bytes).context("decode mxl container")?
    } else {
        bytes.to_vec()
    };
    let doc = cymbra_musicxml_core::parse(&inner).context("parse musicxml")?;
    Ok(ScoreSummary::from_document(&doc))
}

// ---------------------------------------------------------------------------
// Instrument re-derivation (change: add-drums-access)
//
// The `instrument` column cannot be backfilled in SQL: the tables hold an
// `object_key`, not the bytes, and translating the retired `is_piano` flag
// would be WRONG — the corpus already contains percussion ingested despite the
// old playable-notes gate, and a flag translation would record every one of
// them `unknown`, which the drum gate serves to everyone. So each row's object
// is streamed back, parsed with the shared classifier, and the derived family
// persisted. Idempotent and resumable: a second run is all `unchanged`. The
// drum gate is NOT a boundary until this pass has completed.

/// A row of either score table for the instrument re-derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstrumentRow {
    pub id: String,
    pub object_key: String,
    /// The currently-stored family (post-migration default: `unknown`).
    pub instrument: crate::repo::Instrument,
}

/// Which score table a page/update targets: both carry the column and both are
/// re-derived, so the gate holds on catalog and user scores alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreTable {
    Catalog,
    User,
}

/// Data-access seam for the instrument pass. The Postgres impl lives in
/// [`crate::pg::PgInstrumentBackfillRepo`]; a fake backs the orchestration tests.
#[async_trait]
pub trait InstrumentBackfillRepo: Send + Sync {
    /// One page of `table` rows ordered by `id`, with `id > after` (keyset
    /// paging; `after` empty → from the start), up to `limit`.
    async fn page(&self, table: ScoreTable, after: &str, limit: i64) -> Result<Vec<InstrumentRow>>;

    /// Persist the derived family for `id` in `table`.
    async fn update_instrument(
        &self,
        table: ScoreTable,
        id: &str,
        instrument: crate::repo::Instrument,
    ) -> Result<()>;
}

/// Tallies for one instrument re-derivation run (per table, summed by the bin).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InstrumentBackfillReport {
    pub scanned: usize,
    /// Rows whose stored family was (or would be) rewritten.
    pub updated: usize,
    /// Rows already carrying the derived family.
    pub unchanged: usize,
    /// Rows whose bytes could not be fetched or parsed — left untouched (they
    /// stay `unknown`, never guessed), counted so the operator sees the tail.
    pub unreadable: usize,
    /// Update failures (logged, never fatal).
    pub errors: usize,
}

/// Re-derive the instrument for every row of `table` from its stored bytes.
/// `apply == false` is a dry run. One bad row never aborts the run.
pub async fn run_instrument_backfill(
    repo: &dyn InstrumentBackfillRepo,
    storage: &dyn ObjectStorage,
    table: ScoreTable,
    apply: bool,
    page_size: i64,
) -> Result<InstrumentBackfillReport> {
    let mut report = InstrumentBackfillReport::default();
    let mut after = String::new();
    loop {
        let rows = repo
            .page(table, &after, page_size)
            .await
            .context("paging scores for the instrument backfill")?;
        let Some(last) = rows.last() else { break };
        after = last.id.clone();

        for row in rows {
            report.scanned += 1;
            let bytes = match storage.get(&row.object_key).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!(id = %row.id, key = %row.object_key, error = %e,
                        "instrument backfill: object fetch failed, row left as is");
                    report.unreadable += 1;
                    continue;
                }
            };
            let derived = match derive_instrument(&bytes) {
                Ok(i) => i,
                Err(e) => {
                    tracing::warn!(id = %row.id, error = %e,
                        "instrument backfill: parse failed, row left as is");
                    report.unreadable += 1;
                    continue;
                }
            };
            if derived == row.instrument {
                report.unchanged += 1;
                continue;
            }
            if apply && let Err(e) = repo.update_instrument(table, &row.id, derived).await {
                tracing::warn!(id = %row.id, error = %e,
                    "instrument backfill: update failed, skipping");
                report.errors += 1;
                continue;
            }
            tracing::info!(id = %row.id, from = row.instrument.as_str(),
                to = derived.as_str(), applied = apply,
                "instrument backfill: family re-derived");
            report.updated += 1;
        }
    }
    Ok(report)
}

/// Decode stored bytes → parse → derived instrument family.
fn derive_instrument(bytes: &[u8]) -> Result<crate::repo::Instrument> {
    let inner = if mxl::is_mxl(bytes) {
        mxl::decode(bytes).context("decode mxl container")?
    } else {
        bytes.to_vec()
    };
    let doc = cymbra_musicxml_core::parse(&inner).context("parse musicxml")?;
    Ok(crate::repo::Instrument::from_core(
        cymbra_musicxml_core::instrument_of(&doc),
    ))
}

// ---------------------------------------------------------------------------
// Percussion difficulty re-grade (change: add-drum-scoring)
//
// The crawler's difficulty heuristic used to read pitched features only —
// density, ambitus, melodic leap, key accidentals, grand staff. On a drum part
// every one of them counts zero notes, so every percussion row it graded came
// out Beginner *by degeneracy*, not by judgment. Now that the heuristic is
// instrument-aware, those rows have to be re-graded: the catalog `level` feeds
// one shared difficulty weight (play rewards and the global season score), so
// leaving them would underpay honest drum play and — worse — make the drum
// corpus the cheapest farming target the moment percussion becomes scorable.
//
// Only `level_source = 'heuristic'` rows are touched. Overwriting a heuristic
// value while leaving `source` and `manual` grades alone is exactly the
// overwrite the provenance rule exists to permit, and the re-graded row stays
// `heuristic` — a better guess is still a guess.
//
// The *estimate* is deliberately not here: difficulty policy belongs to the
// crawler (`score_crawler::difficulty`), which already depends on this crate,
// so only the data-access seam and the report live in `music`. The
// orchestration is `score_crawler::backfill::run_percussion_regrade`.

/// A catalog row eligible for the percussion re-grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifficultyRow {
    pub id: String,
    pub object_key: String,
    /// The currently-stored level (`beginner` | `intermediate` | `advanced`),
    /// or `None` for a row that carries none.
    pub level: Option<String>,
}

/// Data-access seam for the re-grade. The Postgres impl lives in
/// [`crate::pg::PgDifficultyBackfillRepo`]; the orchestration is host-tested
/// against a mock.
#[async_trait]
pub trait DifficultyBackfillRepo: Send + Sync {
    /// One page of **catalog** rows classified `percussion` and graded
    /// `level_source = 'heuristic'`, ordered by `id`, with `id > after` (keyset
    /// paging; `after` empty → from the start), up to `limit`.
    ///
    /// User scores are out of scope: only the crawler writes heuristic grades,
    /// and it only writes them to the catalog.
    async fn page_percussion_heuristic(
        &self,
        after: &str,
        limit: i64,
    ) -> Result<Vec<DifficultyRow>>;

    /// Persist a re-graded `level` for `id`. `level_source` stays `heuristic`.
    async fn update_level(&self, id: &str, level: &str) -> Result<()>;
}

/// Tallies for one re-grade run.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DifficultyBackfillReport {
    pub scanned: usize,
    /// Rows whose level was (or would be) re-graded.
    pub updated: usize,
    /// Rows the instrument-aware heuristic puts back where they already are.
    pub unchanged: usize,
    /// Rows whose stored family says percussion but whose bytes do not — left
    /// untouched rather than graded on a drum scale (the stored family is only
    /// as fresh as the last `backfill-instruments` run).
    pub not_percussion: usize,
    /// Rows whose bytes could not be fetched or parsed — left untouched.
    pub unreadable: usize,
    /// Update failures (logged, never fatal).
    pub errors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_storage::FakeStore;
    use std::sync::Mutex;

    // A minimal, valid MusicXML score carrying `title` as its `<work-title>`.
    fn score_xml(title: &str) -> Vec<u8> {
        format!(
            r#"<?xml version="1.0"?>
<score-partwise version="4.0"><work><work-title>{title}</work-title></work>
<part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<key><fifths>0</fifths></key><time><beats>4</beats><beat-type>4</beat-type></time>
<clef><sign>G</sign><line>2</line></clef></attributes>
<note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note></measure></part></score-partwise>"#
        )
        .into_bytes()
    }

    // A valid score with no `<work-title>` at all.
    fn untitled_xml() -> Vec<u8> {
        br#"<?xml version="1.0"?>
<score-partwise version="4.0">
<part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<key><fifths>0</fifths></key><time><beats>4</beats><beat-type>4</beat-type></time>
<clef><sign>G</sign><line>2</line></clef></attributes>
<note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note></measure></part></score-partwise>"#
            .to_vec()
    }

    fn summary_of(xml: &[u8]) -> ScoreSummary {
        ScoreSummary::from_document(&cymbra_musicxml_core::parse(xml).unwrap())
    }

    #[test]
    fn plans_update_when_embedded_title_differs() {
        let s = summary_of(&score_xml("Der Lindenbaum"));
        let plan = plan_title_update(Some("lc28971056"), &s).expect("should plan");
        assert_eq!(plan.title, "Der Lindenbaum");
        // The search key is recomputed with the shared normalisation, so the
        // renamed score is findable — this is the whole point of the backfill.
        assert_eq!(plan.title_norm.as_deref(), Some("der lindenbaum"));
        assert!(plan.work_key.ends_with("::der lindenbaum"));
    }

    #[test]
    fn no_plan_when_already_correct() {
        let s = summary_of(&score_xml("Der Lindenbaum"));
        assert_eq!(plan_title_update(Some("Der Lindenbaum"), &s), None);
    }

    #[test]
    fn no_plan_when_score_has_no_title() {
        let s = summary_of(&untitled_xml());
        assert!(s.title.is_none());
        assert_eq!(plan_title_update(Some("lc28971056"), &s), None);
    }

    /// In-memory [`TitleBackfillRepo`] returning a fixed page once, then empty, and
    /// recording every applied update.
    struct FakeRepo {
        rows: Vec<BackfillRow>,
        applied: Mutex<Vec<(String, TitleUpdate)>>,
    }

    #[async_trait]
    impl TitleBackfillRepo for FakeRepo {
        async fn page(
            &self,
            after: &str,
            _source: Option<&str>,
            _limit: i64,
        ) -> Result<Vec<BackfillRow>> {
            // Keyset emulation: return everything after `after` (ids are ordered).
            Ok(self
                .rows
                .iter()
                .filter(|r| r.id.as_str() > after)
                .cloned()
                .collect())
        }

        async fn update_title(&self, id: &str, update: &TitleUpdate) -> Result<()> {
            self.applied
                .lock()
                .unwrap()
                .push((id.to_string(), update.clone()));
            Ok(())
        }
    }

    fn row(id: &str, key: &str, title: Option<&str>) -> BackfillRow {
        BackfillRow {
            id: id.into(),
            object_key: key.into(),
            title: title.map(Into::into),
            source_item_id: format!("{id}.ly"),
            composer: None,
        }
    }

    #[tokio::test]
    async fn run_applies_only_wrong_titles_and_tallies() {
        let store = FakeStore::default();
        store.put("k1", score_xml("Der Lindenbaum")).await.unwrap();
        store.put("k2", score_xml("Ave Verum")).await.unwrap();
        store.put("k3", untitled_xml()).await.unwrap();
        // k4 is referenced by a row but absent from the store → an error row.

        let repo = FakeRepo {
            rows: vec![
                row("id1", "k1", Some("lc28971056")), // wrong id-title → rewrite
                row("id2", "k2", Some("Ave Verum")),  // already correct → unchanged
                row("id3", "k3", Some("lc00000000")), // no <work-title> → kept
                row("id4", "k4", Some("whatever")),   // missing object → error
            ],
            applied: Mutex::new(Vec::new()),
        };

        let report = run_title_backfill(&repo, &store, None, true, 100)
            .await
            .unwrap();

        assert_eq!(report.scanned, 4);
        assert_eq!(report.updated, 1);
        assert_eq!(report.unchanged, 1);
        assert_eq!(report.no_title, 1);
        assert_eq!(report.errors, 1);

        // Exactly the one wrong row was written, with a consistent title triple.
        let applied = repo.applied.lock().unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].0, "id1");
        assert_eq!(applied[0].1.title, "Der Lindenbaum");
        assert_eq!(applied[0].1.title_norm.as_deref(), Some("der lindenbaum"));
    }

    #[tokio::test]
    async fn dry_run_writes_nothing_but_counts() {
        let store = FakeStore::default();
        store.put("k1", score_xml("Der Lindenbaum")).await.unwrap();
        let repo = FakeRepo {
            rows: vec![row("id1", "k1", Some("lc28971056"))],
            applied: Mutex::new(Vec::new()),
        };

        let report = run_title_backfill(&repo, &store, None, false, 100)
            .await
            .unwrap();

        assert_eq!(report.updated, 1); // would rewrite
        assert!(repo.applied.lock().unwrap().is_empty()); // but wrote nothing
    }

    // --- instrument re-derivation (change: add-drums-access) ---------------

    /// A minimal drum score: one unpitched note referencing a declared snare
    /// (1-based `midi-unpitched` 39 → GM 38), classifying `percussion`.
    fn drum_xml() -> Vec<u8> {
        br#"<?xml version="1.0"?>
<score-partwise version="4.0">
<part-list><score-part id="P1">
<score-instrument id="P1-I38"><instrument-name>Snare Drum</instrument-name></score-instrument>
<midi-instrument id="P1-I38"><midi-unpitched>39</midi-unpitched></midi-instrument>
</score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<clef><sign>percussion</sign><line>2</line></clef></attributes>
<note><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched><duration>4</duration><instrument id="P1-I38"/><voice>1</voice></note>
</measure></part></score-partwise>"#
            .to_vec()
    }

    struct FakeInstrumentRepo {
        rows: Vec<InstrumentRow>,
        applied: Mutex<Vec<(String, crate::repo::Instrument)>>,
    }

    #[async_trait]
    impl InstrumentBackfillRepo for FakeInstrumentRepo {
        async fn page(
            &self,
            _table: ScoreTable,
            after: &str,
            limit: i64,
        ) -> Result<Vec<InstrumentRow>> {
            Ok(self
                .rows
                .iter()
                .filter(|r| r.id.as_str() > after)
                .take(limit as usize)
                .cloned()
                .collect())
        }

        async fn update_instrument(
            &self,
            _table: ScoreTable,
            id: &str,
            instrument: crate::repo::Instrument,
        ) -> Result<()> {
            self.applied
                .lock()
                .unwrap()
                .push((id.to_string(), instrument));
            Ok(())
        }
    }

    #[tokio::test]
    async fn instrument_backfill_rederives_from_bytes_and_leaves_unreadable_rows() {
        let store = FakeStore::default();
        store.put("k/drums.xml", drum_xml()).await.unwrap();
        store.put("k/piano.xml", score_xml("Sonata")).await.unwrap();
        // "k/missing.xml" is never stored: that row must be left untouched.
        let row = |id: &str, key: &str| InstrumentRow {
            id: id.into(),
            object_key: key.into(),
            instrument: crate::repo::Instrument::Unknown,
        };
        let repo = FakeInstrumentRepo {
            rows: vec![
                row("a", "k/drums.xml"),
                row("b", "k/piano.xml"),
                row("c", "k/missing.xml"),
            ],
            applied: Mutex::new(Vec::new()),
        };

        let report = run_instrument_backfill(&repo, &store, ScoreTable::Catalog, true, 2)
            .await
            .unwrap();
        assert_eq!(report.scanned, 3);
        assert_eq!(report.updated, 2);
        assert_eq!(report.unreadable, 1); // the missing object, left as is
        assert_eq!(report.errors, 0);
        let applied = repo.applied.lock().unwrap().clone();
        assert_eq!(
            applied,
            vec![
                ("a".to_string(), crate::repo::Instrument::Percussion),
                ("b".to_string(), crate::repo::Instrument::Keyboard),
            ]
        );
    }

    #[tokio::test]
    async fn instrument_backfill_is_idempotent_and_dry_run_writes_nothing() {
        let store = FakeStore::default();
        store.put("k/drums.xml", drum_xml()).await.unwrap();
        let repo = FakeInstrumentRepo {
            rows: vec![InstrumentRow {
                id: "a".into(),
                object_key: "k/drums.xml".into(),
                // Already re-derived: a second run must change nothing.
                instrument: crate::repo::Instrument::Percussion,
            }],
            applied: Mutex::new(Vec::new()),
        };
        let report = run_instrument_backfill(&repo, &store, ScoreTable::Catalog, true, 100)
            .await
            .unwrap();
        assert_eq!(report.unchanged, 1);
        assert!(repo.applied.lock().unwrap().is_empty());

        // Dry run over a row that WOULD change: counted, not written.
        let repo = FakeInstrumentRepo {
            rows: vec![InstrumentRow {
                id: "a".into(),
                object_key: "k/drums.xml".into(),
                instrument: crate::repo::Instrument::Unknown,
            }],
            applied: Mutex::new(Vec::new()),
        };
        let report = run_instrument_backfill(&repo, &store, ScoreTable::Catalog, false, 100)
            .await
            .unwrap();
        assert_eq!(report.updated, 1);
        assert!(repo.applied.lock().unwrap().is_empty());
    }
}
