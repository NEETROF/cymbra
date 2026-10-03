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

//! WASM bindings for Cymbra Lingua (spec `lingua-analysis` — native/WASM parity,
//! plus the deck/review/backup surface the browser extension drives).
//!
//! A thin wasm-bindgen surface over `lingua-core`. The engine holds the whole
//! product [`LinguaState`] (knowledge + exposure + deck + FSRS), one pack per
//! studied language, and an optional in-flight review session — so a surface (the extension,
//! later the Apple app) can read, build a deck, run an FSRS review and take a
//! lossless backup without reaching into the core directly. A binding whose answer
//! depends on the studied language takes it as its last, optional parameter (ISO
//! 639-1); without one it answers in the language of the first pack loaded, and a
//! language the engine holds no pack for is refused (generalise-lingua-wasm-engine).
//! The bindings about the whole reader — backup, restore, resets, counts, the review
//! session, sync exports and applies — cover every language. No logic lives here:
//! the analysis output stays byte-for-byte identical to the native build at an
//! equal `analyzer_version` and pack (the parity contract). Build with
//! `wasm-pack build --target web`.

use lingua_core::analysis::language::{StudiedLanguage, detect_document_language};
use lingua_core::decks::backup::LinguaState;
use lingua_core::decks::card::{Card, EncounterSource, Provenance};
use lingua_core::decks::fsrs::{Rating, ReviewState};
use lingua_core::decks::review::ReviewSession;
use lingua_core::engine::{analyse_page_json, gloss_phrase_json, word_grammar_json};
use lingua_core::knowledge::level::CefrLevel;
use lingua_core::knowledge::state::{FrequencyRanks, KnowledgeState};
use lingua_core::knowledge::status::{KnownSource, Status};
use lingua_core::knowledge::vocabulary::level_vocabulary;
use lingua_core::packs::{Pack, PackSet};
use wasm_bindgen::prelude::*;

/// How long a lemma's exposure counter survives without being read again. Promotion wants
/// reading spread over days; a word not met for a season is not on its way there.
const EXPOSURE_KEEP_DAYS: i64 = 90;
/// Ceiling on the counters kept (the most recently read win). It bounds the stored state,
/// which every surface loads and the browser caps — far above a reader's working vocabulary.
const EXPOSURE_MAX_LEMMAS: usize = 5_000;

/// A loaded analysis + review engine: a pack per studied language, the whole local
/// state, and an optional in-flight review session.
#[wasm_bindgen]
pub struct LinguaEngine {
    packs: PackSet,
    state: LinguaState,
    session: Option<ReviewSession>,
    /// Per language, and per level A1..C2, the vocabulary typical of a reader at it — a
    /// property of the pack alone, so computed once, on that language's first ladder.
    level_vocabularies:
        std::collections::BTreeMap<StudiedLanguage, std::cell::OnceCell<[usize; 6]>>,
}

/// The language a call names (`None`: the default, the first pack loaded) and its
/// pack, or the error that refuses the call before any state is touched.
fn resolve<'a>(
    packs: &'a PackSet,
    language: Option<&str>,
) -> Result<(StudiedLanguage, &'a Pack), JsError> {
    packs
        .resolve(language)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Drop the counters that can no longer confirm anything in `language`, then bound what
/// is left. Runs on every recording and on restore, so a store saturated by an older
/// build heals.
fn prune_exposures(state: &mut LinguaState, language: StudiedLanguage, pack: &Pack, now_secs: i64) {
    let cutoff_day = now_secs.div_euclid(86_400) - EXPOSURE_KEEP_DAYS;
    let knowledge = &state.knowledge;
    state.exposure.retain(language, |lemma, exposure| {
        exposure.last_day >= cutoff_day && knowledge.promotable_by_exposure(language, lemma, pack)
    });
    state.exposure.cap_by_recency(language, EXPOSURE_MAX_LEMMAS);
}

#[wasm_bindgen]
impl LinguaEngine {
    /// Loads the engine from pack bytes (the embedded `pack.lingua`); that pack's
    /// language becomes the default. Errors if the pack is malformed or built for an
    /// incompatible analyser.
    #[wasm_bindgen(constructor)]
    pub fn new(pack_bytes: &[u8]) -> Result<LinguaEngine, JsError> {
        let pack = Pack::load(pack_bytes).map_err(|e| JsError::new(&e.to_string()))?;
        let default = pack.studied();
        Ok(LinguaEngine {
            packs: PackSet::new(pack),
            state: LinguaState::default(),
            session: None,
            level_vocabularies: std::iter::once((default, std::cell::OnceCell::new())).collect(),
        })
    }

    /// Loads another studied language's pack, returning its ISO 639-1 tag. Errors if
    /// the pack is malformed, built for an incompatible analyser, or for a language the
    /// engine already holds a pack for.
    #[wasm_bindgen(js_name = addPack)]
    pub fn add_pack(&mut self, pack_bytes: &[u8]) -> Result<String, JsError> {
        let pack = Pack::load(pack_bytes).map_err(|e| JsError::new(&e.to_string()))?;
        let language = self
            .packs
            .add(pack)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.level_vocabularies
            .insert(language, std::cell::OnceCell::new());
        Ok(language.tag().to_owned())
    }

    /// The languages the engine holds a pack for, as a JSON array of ISO 639-1 tags,
    /// the default first.
    pub fn languages(&self) -> String {
        let tags: Vec<&str> = self
            .packs
            .languages()
            .into_iter()
            .map(StudiedLanguage::tag)
            .collect();
        serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_owned())
    }

    /// The reader's studied languages, from their profile, as a JSON array of ISO
    /// 639-1 tags, the primary first (add-lingua-studied-language-profile). Not the
    /// packs held (`languages`): a reader may study a language whose pack is not
    /// loaded yet.
    #[wasm_bindgen(js_name = studiedLanguages)]
    pub fn studied_languages(&self) -> String {
        let tags: Vec<&str> = self
            .state
            .profile
            .studied_languages
            .iter()
            .map(|language| language.tag())
            .collect();
        serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_owned())
    }

    /// Sets the reader's studied languages from ISO 639-1 tags, the primary first.
    /// Errors on an unknown tag, an empty list or a language named twice, leaving
    /// the current ones in place.
    #[wasm_bindgen(js_name = setStudiedLanguages)]
    pub fn set_studied_languages(&mut self, tags: Vec<String>) -> Result<(), JsError> {
        let languages = tags
            .iter()
            .map(|tag| {
                StudiedLanguage::from_tag(tag)
                    .ok_or_else(|| JsError::new(&format!("unknown studied language \"{tag}\"")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.state
            .profile
            .set_studied_languages(languages)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// The language of a document among `candidates` (ISO 639-1 tags, the reader's order), with
    /// the document's declared language as `hint` (add-lingua-language-routing D2). Loads and
    /// needs no pack. Errors on an empty list or an unknown candidate; an unknown hint is
    /// ignored, since a page may declare any language.
    #[wasm_bindgen(js_name = detectLanguage)]
    pub fn detect_language(
        &self,
        blocks: Vec<String>,
        candidates: Vec<String>,
        hint: Option<String>,
    ) -> Result<String, JsError> {
        let candidates = candidates
            .iter()
            .map(|tag| {
                StudiedLanguage::from_tag(tag)
                    .ok_or_else(|| JsError::new(&format!("unknown studied language \"{tag}\"")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let hint = hint.as_deref().and_then(StudiedLanguage::from_tag);
        let blocks: Vec<&str> = blocks.iter().map(String::as_str).collect();
        detect_document_language(&blocks, &candidates, hint)
            .map(|language| language.tag().to_owned())
            .ok_or_else(|| JsError::new("no candidate language"))
    }

    // --- Knowledge + analysis (the reading surface; signatures unchanged) ---

    /// Sets the calibration threshold ("I know the N most common words").
    #[wasm_bindgen(js_name = setCalibration)]
    pub fn set_calibration(
        &mut self,
        threshold: u32,
        language: Option<String>,
    ) -> Result<(), JsError> {
        let (language, _) = resolve(&self.packs, language.as_deref())?;
        self.state.knowledge.set_calibration(language, threshold);
        Ok(())
    }

    /// The current calibration threshold.
    pub fn calibration(&self, language: Option<String>) -> Result<u32, JsError> {
        let (language, _) = resolve(&self.packs, language.as_deref())?;
        Ok(self.state.knowledge.calibration(language))
    }

    /// Declares the reader's CEFR level (`"A1"`..`"C2"`); any other value —
    /// including `""` — clears it, returning to frequency calibration.
    #[wasm_bindgen(js_name = setDeclaredLevel)]
    pub fn set_declared_level(
        &mut self,
        level: &str,
        language: Option<String>,
    ) -> Result<(), JsError> {
        let (language, _) = resolve(&self.packs, language.as_deref())?;
        match CefrLevel::from_label(level) {
            Some(l) => self.state.knowledge.set_declared_level(language, l),
            None => self.state.knowledge.clear_declared_level(language),
        }
        Ok(())
    }

    /// The declared CEFR level label (`"A1"`..`"C2"`), or `None` if none is set.
    #[wasm_bindgen(js_name = declaredLevel)]
    pub fn declared_level(&self, language: Option<String>) -> Result<Option<String>, JsError> {
        let (language, _) = resolve(&self.packs, language.as_deref())?;
        Ok(self
            .state
            .knowledge
            .declared_level(language)
            .map(|l| l.label().to_owned()))
    }

    /// Like `setDeclaredLevel`, but stamps the decision with a sync timestamp
    /// (epoch millis, the caller's clock) so the outbox and cross-device LWW can
    /// order it. A `""` / invalid label is the explicit "débutant" decision (no
    /// level), which syncs just like a level.
    #[wasm_bindgen(js_name = setDeclaredLevelAt)]
    pub fn set_declared_level_at(
        &mut self,
        level: &str,
        at_ms: f64,
        language: Option<String>,
    ) -> Result<(), JsError> {
        let (language, _) = resolve(&self.packs, language.as_deref())?;
        self.state.knowledge.set_declared_level_at(
            language,
            CefrLevel::from_label(level),
            at_ms as i64,
        );
        Ok(())
    }

    /// The declared-level decisions as sync JSON (array of
    /// `{language, level, updated_at}`, `level` a label or `""` for débutant), for
    /// a push. Mirrors `exportStatusOps`.
    #[wasm_bindgen(js_name = exportDeclaredLevels)]
    pub fn export_declared_levels(&self) -> String {
        let rows: Vec<serde_json::Value> = self
            .state
            .knowledge
            .export_declared_levels()
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "language": r.language.tag(),
                    "level": r.level.map(|l| l.label()).unwrap_or(""),
                    "updated_at": r.updated_at,
                })
            })
            .collect();
        serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_owned())
    }

    /// Apply pulled declared-level changes (JSON array of
    /// `{language, level, updated_at}`) under last-write-wins. `level` `""` (or
    /// absent) is the débutant decision. Returns how many changed local state.
    /// Mirrors `applyStatusChanges`.
    #[wasm_bindgen(js_name = applyDeclaredLevelChanges)]
    pub fn apply_declared_level_changes(&mut self, json: &str) -> Result<usize, JsError> {
        let changes: Vec<serde_json::Value> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let mut changed = 0usize;
        for c in &changes {
            let tag = c.get("language").and_then(|v| v.as_str()).unwrap_or("en");
            let Ok(language) = self.packs.resolve(Some(tag)).map(|(l, _)| l) else {
                continue; // a language this engine holds no pack for
            };
            let level =
                CefrLevel::from_label(c.get("level").and_then(|v| v.as_str()).unwrap_or(""));
            let updated_at = c
                .get("updated_at")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            if self
                .state
                .knowledge
                .apply_declared_level_lww(language, level, updated_at)
            {
                changed += 1;
            }
        }
        Ok(changed)
    }

    /// Whether the loaded pack carries a CEFR level table (else the ladder and
    /// level-targeted feeding fall back to frequency bands).
    #[wasm_bindgen(js_name = hasLevels)]
    pub fn has_levels(&self, language: Option<String>) -> Result<bool, JsError> {
        let (_, pack) = resolve(&self.packs, language.as_deref())?;
        Ok(pack.has_levels())
    }

    /// The CEFR progression ladder as JSON — an array of
    /// `{level, confirmed, presumed, toLearn, total, typicalVocabulary}`, one row per
    /// level A1..C2, folded over the pack's lemmas at each level; `typicalVocabulary` is
    /// the vocabulary size of a reader at the level (see `level_vocabulary`;
    /// 0 at A1, which presumes nothing). `[]` when the pack carries no CEFR data.
    #[wasm_bindgen(js_name = levelLadder)]
    pub fn level_ladder(&self, language: Option<String>) -> Result<String, JsError> {
        let (language, pack) = resolve(&self.packs, language.as_deref())?;
        if !pack.has_levels() {
            return Ok("[]".to_owned());
        }
        let compute = || {
            let words = pack.dictionary_words();
            CefrLevel::ALL.map(|level| level_vocabulary(level, words.iter().copied(), pack))
        };
        let typical: [usize; 6] = match self.level_vocabularies.get(&language) {
            Some(cell) => *cell.get_or_init(compute),
            None => compute(),
        };
        let rows: Vec<serde_json::Value> = CefrLevel::ALL
            .iter()
            .zip(typical.iter())
            .map(|(&level, &typical)| {
                let lemmas = pack.lemmas_at_level(level);
                let stats =
                    self.state
                        .knowledge
                        .band_stats(language, lemmas.iter().map(|(l, _)| *l), pack);
                serde_json::json!({
                    "level": level.label(),
                    "confirmed": stats.confirmed,
                    "presumed": stats.presumed,
                    "toLearn": stats.to_learn,
                    "total": stats.total(),
                    "typicalVocabulary": typical,
                })
            })
            .collect();
        Ok(serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_owned()))
    }

    /// The reader's estimated vocabulary size as JSON `{estimated, confirmed, universe}`:
    /// each frequency band's known share, extrapolated over the pack's dictionary words
    /// (see `KnowledgeState::vocabulary_estimate`). Works with or without CEFR data.
    #[wasm_bindgen(js_name = vocabularyEstimate)]
    pub fn vocabulary_estimate(&self, language: Option<String>) -> Result<String, JsError> {
        let (language, pack) = resolve(&self.packs, language.as_deref())?;
        let estimate =
            self.state
                .knowledge
                .vocabulary_estimate(language, pack.dictionary_words(), pack);
        Ok(serde_json::to_string(&estimate).unwrap_or_else(|_| "{}".to_owned()))
    }

    /// Records one reading exposure per lemma (`source` tag, `at_ms` in millis),
    /// feeding the distinct-day counters that back exposure-confirmed known.
    /// Recording never changes a status (design D5) — promotion is the separate,
    /// explicit `promoteByExposure`.
    #[wasm_bindgen(js_name = recordExposures)]
    pub fn record_exposures(
        &mut self,
        lemmas: Vec<String>,
        source: &str,
        at_ms: f64,
        language: Option<String>,
    ) -> Result<(), JsError> {
        let (language, pack) = resolve(&self.packs, language.as_deref())?;
        let secs = (at_ms as i64).div_euclid(1000); // exposure timestamps are epoch seconds
        for lemma in &lemmas {
            // Only a lemma reading could still confirm is worth a counter. Counting every
            // word ever met filled the extension's storage — ~200 bytes each, for nothing
            // when no level is declared (dogfooding, TestFlight 70/71).
            if !self
                .state
                .knowledge
                .promotable_by_exposure(language, lemma, pack)
            {
                continue;
            }
            self.state.exposure.record(language, lemma, 1, source, secs);
        }
        prune_exposures(&mut self.state, language, pack, secs);
        Ok(())
    }

    /// Confirms presumed-known lemmas that reading has vouched for: below the
    /// declared level, no explicit status, seen on at least `threshold_days`
    /// distinct days. Returns the number promoted to `Known(Exposure)`; a no-op
    /// (0) without a declared level. `at_ms` (millis) stamps the sync timestamp.
    #[wasm_bindgen(js_name = promoteByExposure)]
    pub fn promote_by_exposure(
        &mut self,
        threshold_days: u32,
        at_ms: f64,
        language: Option<String>,
    ) -> Result<usize, JsError> {
        let (language, pack) = resolve(&self.packs, language.as_deref())?;
        Ok(self
            .state
            .knowledge
            .promote_by_exposure(
                language,
                &self.state.exposure,
                pack,
                threshold_days,
                at_ms as i64,
            )
            .len())
    }

    /// Sets an explicit status for a form. `status` is one of `learning`,
    /// `known`, `ignored`; anything else clears it.
    #[wasm_bindgen(js_name = setStatus)]
    pub fn set_status(
        &mut self,
        lemma: &str,
        status: &str,
        language: Option<String>,
    ) -> Result<(), JsError> {
        let (language, _) = resolve(&self.packs, language.as_deref())?;
        match status {
            "learning" => self
                .state
                .knowledge
                .set_status(language, lemma, Status::Learning),
            "known" => {
                self.state
                    .knowledge
                    .set_status(language, lemma, Status::Known(KnownSource::Manual))
            }
            "ignored" => self
                .state
                .knowledge
                .set_status(language, lemma, Status::Ignored),
            _ => self.state.knowledge.clear_status(language, lemma),
        }
        Ok(())
    }

    // --- Status sync (add-lingua-connected-clients §2): the extension's outbox
    // and cursor-pull talk to KnownWordsService in these shapes. Each record names
    // its own language. ---

    /// Like `setStatus`, but stamps the change with a sync timestamp (epoch
    /// millis, the caller's clock) so the outbox and cross-device LWW can order
    /// it. `status` is `learning` | `known` | `ignored`; anything else withdraws
    /// the status ("Remettre à apprendre"): stamped, so the word stays highlighted
    /// even where calibration or the declared level would presume it known,
    /// exposure no longer re-confirms it, and the undo syncs as `cleared`.
    #[wasm_bindgen(js_name = setStatusAt)]
    pub fn set_status_at(
        &mut self,
        lemma: &str,
        status: &str,
        at_ms: f64,
        language: Option<String>,
    ) -> Result<(), JsError> {
        let (language, _) = resolve(&self.packs, language.as_deref())?;
        match Status::from_wire(status, "manual") {
            Some(s) => self
                .state
                .knowledge
                .set_status_at(language, lemma, s, at_ms as i64),
            None => self
                .state
                .knowledge
                .clear_status_at(language, lemma, at_ms as i64),
        }
        Ok(())
    }

    /// The full set of explicit statuses as `StatusOp`-shaped JSON
    /// (`{language, lemma, status, provenance, updated_at}`), for a push (the
    /// first-sign-in full upload, or an incremental drain the caller filters).
    /// A withdrawn status exports as `cleared`, so an undo reaches other devices.
    #[wasm_bindgen(js_name = exportStatusOps)]
    pub fn export_status_ops(&self) -> String {
        let ops: Vec<serde_json::Value> = self
            .state
            .knowledge
            .export_statuses()
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "language": r.language.tag(),
                    "lemma": r.lemma,
                    "status": r.status.map_or("cleared", Status::wire_kind),
                    "provenance": r.status.map_or("manual", Status::wire_provenance),
                    "updated_at": r.updated_at,
                })
            })
            .collect();
        serde_json::to_string(&ops).unwrap_or_else(|_| "[]".to_owned())
    }

    /// Apply a batch of pulled `StatusChange`s (JSON array of
    /// `{language, lemma, status, provenance?, updated_at}`) under last-write-wins.
    /// Returns how many changed local state. The pulled change now carries `provenance`,
    /// so an exposure-confirmed known keeps its reversible tag on a second device; a
    /// missing/empty value (a server that predates the field) degrades to manual.
    #[wasm_bindgen(js_name = applyStatusChanges)]
    pub fn apply_status_changes(&mut self, json: &str) -> Result<usize, JsError> {
        let changes: Vec<serde_json::Value> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let mut changed = 0usize;
        for c in &changes {
            let tag = c.get("language").and_then(|v| v.as_str()).unwrap_or("en");
            let Ok(language) = self.packs.resolve(Some(tag)).map(|(l, _)| l) else {
                continue; // a language this engine holds no pack for
            };
            let Some(lemma) = c.get("lemma").and_then(|v| v.as_str()) else {
                continue;
            };
            let kind = c.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let provenance = c
                .get("provenance")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or("manual");
            let updated_at = c
                .get("updated_at")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            let incoming = Status::from_wire(kind, provenance);
            if self
                .state
                .knowledge
                .apply_status_lww(language, lemma, incoming, updated_at)
            {
                changed += 1;
                // A word reclassified known/ignored elsewhere stops coming due here too —
                // the same retirement the gesture performs on the device that made it,
                // which that device could not do for a card it did not have yet.
                if matches!(incoming, Some(Status::Known(_)) | Some(Status::Ignored)) {
                    self.state.deck.retire(language, lemma, updated_at / 1000);
                }
            }
        }
        Ok(changed)
    }

    /// The whole deck as `CardOp`-shaped JSON for a push to `DeckService`
    /// (one card per lemma; `client_id` = lemma). The FSRS state travels as an
    /// opaque JSON string. `device_id` is left empty for the caller to attach;
    /// `client_ts` is the card's `updated_at` in millis. `source` is always empty: the
    /// page a card was captured from stays on the device (add-lingua-privacy-controls)
    /// and only the local backup keeps it.
    #[wasm_bindgen(js_name = exportCardOps)]
    pub fn export_card_ops(&self) -> String {
        let ops: Vec<serde_json::Value> = self
            .state
            .deck
            .export_cards()
            .into_iter()
            .map(|(language, card)| {
                serde_json::json!({
                    "client_id": card.lemma,
                    "language": language.tag(),
                    "lemma": card.lemma,
                    "surface_form": card.encountered_form,
                    "source_sentence": card.provenance.sentence,
                    "source": "",
                    "gloss": card.gloss.clone().unwrap_or_default(),
                    "fsrs_state": serde_json::to_string(&card.review).unwrap_or_default(),
                    "deleted": false,
                    "client_ts": card.updated_at * 1000,
                    "device_id": "",
                })
            })
            .collect();
        serde_json::to_string(&ops).unwrap_or_else(|_| "[]".to_owned())
    }

    /// Apply a batch of pulled `CardOp`s (JSON array) under last-write-wins.
    /// Returns how many changed local state. `deleted` ops are skipped — card
    /// deletion is not an MVP feature (mark-known retires a card, it does not
    /// remove it), and the tombstones a correct delete would need arrive with the
    /// change that adds deletion. `captured_at` has no wire field, so a first-seen
    /// card takes the op timestamp as a proxy; the deck preserves it thereafter, and
    /// keeps the local page address when the op carries none (it never does now).
    #[wasm_bindgen(js_name = applyCardOps)]
    pub fn apply_card_ops(&mut self, json: &str) -> Result<usize, JsError> {
        let ops: Vec<serde_json::Value> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let mut changed = 0usize;
        for op in &ops {
            let tag = op.get("language").and_then(|v| v.as_str()).unwrap_or("en");
            let Ok(language) = self.packs.resolve(Some(tag)).map(|(l, _)| l) else {
                continue; // a language this engine holds no pack for
            };
            if op
                .get("deleted")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                continue; // deletion + tombstones are a future change
            }
            let lemma = op
                .get("lemma")
                .or_else(|| op.get("client_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if lemma.is_empty() {
                continue;
            }
            let str_field = |k: &str| op.get(k).and_then(|v| v.as_str()).unwrap_or("").to_owned();
            let client_ts = op
                .get("client_ts")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            let updated_at = client_ts / 1000; // wire millis → the deck's second-based unit
            let gloss = op
                .get("gloss")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_owned);
            let review: ReviewState =
                serde_json::from_str(&str_field("fsrs_state")).unwrap_or_default();
            let mut card = Card::new(
                lemma,
                &str_field("surface_form"),
                Provenance {
                    sentence: str_field("source_sentence"),
                    source: EncounterSource::Web {
                        url: str_field("source"),
                    },
                    captured_at: updated_at,
                },
                gloss,
            );
            card.review = review;
            card.updated_at = updated_at;
            if self.state.deck.apply_card_lww(language, card) {
                changed += 1;
            }
        }
        Ok(changed)
    }

    /// Analyses a batch of blocks, returning the canonical JSON of the page
    /// analysis (classified tokens, statuses, percentage, glosses).
    pub fn analyse(
        &self,
        blocks: Vec<String>,
        language: Option<String>,
    ) -> Result<String, JsError> {
        let (language, pack) = resolve(&self.packs, language.as_deref())?;
        let refs: Vec<&str> = blocks.iter().map(String::as_str).collect();
        Ok(analyse_page_json(
            &refs,
            language,
            pack,
            &self.state.knowledge,
        ))
    }

    /// The native-language gloss for a form, if the pack carries one.
    pub fn gloss(&self, lemma: &str, language: Option<String>) -> Result<Option<String>, JsError> {
        let (_, pack) = resolve(&self.packs, language.as_deref())?;
        Ok(pack.gloss(lemma).map(str::to_owned))
    }

    /// Glosses a reader's selection, returning the canonical JSON of the phrase
    /// gloss: every token with its dictionary form, its class, its gloss whatever
    /// the class, its function-word flag, and the parts of an unlisted compound —
    /// beside them, the expressions the pack's table recognises, each with the
    /// tokens it covers, its key, its class and its gloss (absent from the JSON
    /// on a pack carrying no expression table).
    /// No page gate applies — a selection is read as one block.
    #[wasm_bindgen(js_name = phraseGloss)]
    pub fn phrase_gloss(&self, text: &str, language: Option<String>) -> Result<String, JsError> {
        let (language, pack) = resolve(&self.packs, language.as_deref())?;
        Ok(gloss_phrase_json(
            text,
            language,
            pack,
            &self.state.knowledge,
        ))
    }

    /// A word card's grammar, as canonical JSON: the dictionary form's gloss,
    /// the same gloss grouped by part of speech, the readings of the word as
    /// written as that dictionary form, the other dictionary forms it is also
    /// a reading of, and the pieces the pre-pass split it into
    /// (`add-lingua-word-grammar`). `written` is the word as it stands on the
    /// page — both halves of `don't` pass `don't` — and `lemma` the dictionary
    /// form the card is keyed by. Pure pack data: the reader's state plays no
    /// part.
    #[wasm_bindgen(js_name = wordGrammar)]
    pub fn word_grammar(
        &self,
        written: &str,
        lemma: &str,
        language: Option<String>,
    ) -> Result<String, JsError> {
        let (language, pack) = resolve(&self.packs, language.as_deref())?;
        Ok(word_grammar_json(written, lemma, language, pack))
    }

    /// Number of forms the reader has explicitly marked (any status).
    #[wasm_bindgen(js_name = trackedCount)]
    pub fn tracked_count(&self) -> usize {
        self.state.knowledge.explicit_count()
    }

    // --- Deck ---

    /// Adds (or replaces) a deck card and marks its form `learning`. `gloss` and
    /// `url` may be empty; `captured_at` is Unix-epoch seconds (caller's clock).
    #[wasm_bindgen(js_name = addCard)]
    // The JS binding's own parameters, plus the language every language-bound binding
    // takes last: grouping them would change the extension's call.
    #[allow(clippy::too_many_arguments)]
    pub fn add_card(
        &mut self,
        lemma: &str,
        surface: &str,
        sentence: &str,
        url: &str,
        gloss: Option<String>,
        captured_at: f64,
        language: Option<String>,
    ) -> Result<(), JsError> {
        let (language, _) = resolve(&self.packs, language.as_deref())?;
        // Stamped, like every other decision: adding a word to the deck is one, and an
        // unstamped "learning" would lose last-write-wins to any dated decision made on
        // another device — even an older one (add-lingua-connected-clients §2).
        self.state.knowledge.set_status_at(
            language,
            lemma,
            Status::Learning,
            (captured_at as i64) * 1000,
        );
        let card = Card::new(
            lemma,
            surface,
            Provenance {
                sentence: sentence.to_owned(),
                source: EncounterSource::Web {
                    url: url.to_owned(),
                },
                captured_at: captured_at as i64,
            },
            gloss,
        );
        self.state.deck.upsert(language, card);
        Ok(())
    }

    /// Retire the deck card for `lemma` if present (keep it, stop it coming due) — used
    /// when a word is reclassified `known`/`ignored` outside a review, so its card stops
    /// surfacing. `now` is Unix-epoch seconds (the card's time unit). No-op when there is
    /// no card for the lemma.
    #[wasm_bindgen(js_name = retireCard)]
    pub fn retire_card(
        &mut self,
        lemma: &str,
        now: f64,
        language: Option<String>,
    ) -> Result<(), JsError> {
        let (language, _) = resolve(&self.packs, language.as_deref())?;
        self.state.deck.retire(language, lemma, now as i64);
        Ok(())
    }

    /// Seeds up to `count` deck cards from a CEFR level's lemmas. `order` is
    /// `"common"` (commonest-first, the default) or `"rare"`; unranked lemmas
    /// always sort last. Skips lemmas already carded or with an explicit status.
    /// `at` is Unix-epoch seconds. Returns the number actually added; a no-op (0)
    /// for an unknown level or a pack with no CEFR data.
    #[wasm_bindgen(js_name = seedLevel)]
    pub fn seed_level(
        &mut self,
        level: &str,
        count: usize,
        order: &str,
        at: f64,
        language: Option<String>,
    ) -> Result<usize, JsError> {
        let (language, pack) = resolve(&self.packs, language.as_deref())?;
        let Some(lvl) = CefrLevel::from_label(level) else {
            return Ok(0);
        };
        let mut items = pack.lemmas_at_level(lvl);
        if order == "rare" {
            items.sort_by_key(|(l, _)| {
                (
                    pack.rank(l).is_none(),
                    std::cmp::Reverse(pack.rank(l).unwrap_or(0)),
                )
            });
        } else {
            items.sort_by_key(|(l, _)| (pack.rank(l).is_none(), pack.rank(l).unwrap_or(0)));
        }
        Ok(self.state.deck.seed_lemmas(
            language,
            items.iter().map(|(l, g)| (*l, *g)),
            &self.state.knowledge,
            count,
            at as i64,
        ))
    }

    /// Total number of cards in the deck.
    #[wasm_bindgen(js_name = deckCount)]
    pub fn deck_count(&self) -> usize {
        self.state.deck.len()
    }

    /// Number of cards due at `now` (Unix-epoch seconds).
    #[wasm_bindgen(js_name = dueCount)]
    pub fn due_count(&self, now: f64) -> usize {
        self.state.deck.due_count(now as i64)
    }

    // --- Review session ---

    /// Starts a review session over everything due at `now`; returns how many
    /// cards it will walk.
    #[wasm_bindgen(js_name = startReview)]
    pub fn start_review(&mut self, now: f64) -> usize {
        let session = ReviewSession::start(&self.state.deck, now as i64);
        let remaining = session.remaining();
        self.session = Some(session);
        remaining
    }

    /// The current card as a JSON view model — `{ headword, surface, sentence,
    /// source, gloss, revealed, remaining }` — or `null` when the session is finished
    /// or not started. `source` is where the word was met, as the device kept it: a
    /// page address, or a book and its chapter (`add-lingua-reader`); empty when none.
    #[wasm_bindgen(js_name = reviewCurrent)]
    pub fn review_current(&self) -> Option<String> {
        let session = self.session.as_ref()?;
        let card = session.current(&self.state.deck)?;
        let source = match &card.provenance.source {
            EncounterSource::Web { url } => url.as_str(),
            EncounterSource::AgentSession { .. } | EncounterSource::Import => "",
        };
        let view = serde_json::json!({
            "headword": card.lemma,
            "surface": card.encountered_form,
            "sentence": card.provenance.sentence,
            "source": source,
            "gloss": card.gloss,
            "revealed": session.is_revealed(),
            "remaining": session.remaining(),
        });
        Some(view.to_string())
    }

    /// Reveals the current card's answer.
    #[wasm_bindgen(js_name = reviewReveal)]
    pub fn review_reveal(&mut self) {
        if let Some(session) = self.session.as_mut() {
            session.reveal();
        }
    }

    /// Cards left in the current session (including the current one).
    #[wasm_bindgen(js_name = reviewRemaining)]
    pub fn review_remaining(&self) -> usize {
        self.session.as_ref().map_or(0, ReviewSession::remaining)
    }

    /// Grades the current card (`again`/`hard`/`good`/`easy`) and advances.
    #[wasm_bindgen(js_name = reviewGrade)]
    pub fn review_grade(&mut self, rating: &str, now: f64) {
        let rating = match rating {
            "again" => Rating::Again,
            "hard" => Rating::Hard,
            "easy" => Rating::Easy,
            _ => Rating::Good,
        };
        let params = self.state.fsrs.clone();
        if let Some(session) = self.session.as_mut() {
            session.grade(&mut self.state.deck, &params, rating, now as i64);
        }
    }

    /// Marks the current card known (provenance `srs`) and retires it. Advances.
    #[wasm_bindgen(js_name = reviewMarkKnown)]
    pub fn review_mark_known(&mut self, now: f64) {
        if let Some(session) = self.session.as_mut() {
            session.mark_known(&mut self.state.deck, &mut self.state.knowledge, now as i64);
        }
    }

    // --- Backup / restore ---

    /// The whole state as a lossless, versioned backup string.
    pub fn backup(&self) -> String {
        self.state.to_backup()
    }

    /// Replaces the whole state from a backup string, refusing an unknown schema
    /// version. Clears any in-flight review session.
    pub fn restore(&mut self, json: &str) -> Result<(), JsError> {
        self.state = LinguaState::from_backup(json).map_err(|e| JsError::new(&e.to_string()))?;
        self.session = None;
        // A backup written by a build that counted every word can be megabytes: bound it
        // here, so the first load after an update shrinks the store instead of failing to
        // write it.
        // No clock here: each language's own latest observation dates its window. A
        // language the engine holds no pack for keeps its counters: they cannot be
        // judged, and the backup is lossless.
        for (language, pack) in self.packs.iter() {
            let latest = self
                .state
                .exposure
                .lemmas(language)
                .map(|(_, e)| e.last_seen)
                .max()
                .unwrap_or(0);
            prune_exposures(&mut self.state, language, pack, latest);
        }
        Ok(())
    }

    /// Resets the whole state to empty defaults (a full reset) — statuses,
    /// exposure counters, and the whole deck (review cards + FSRS). The caller
    /// re-applies its default calibration afterwards.
    pub fn reset(&mut self) {
        self.state = LinguaState::default();
        self.session = None;
    }

    /// A partial reset: clears explicit statuses, calibration and the declared
    /// level, but KEEPS the deck (review cards + FSRS) and the exposure
    /// counters. The caller re-applies its default calibration afterwards.
    #[wasm_bindgen(js_name = resetStatuses)]
    pub fn reset_statuses(&mut self) {
        self.state.knowledge = KnowledgeState::default();
        self.session = None;
    }

    // --- Attributions ---

    /// The pack's bundled attribution NOTICE.
    pub fn notice(&self, language: Option<String>) -> Result<String, JsError> {
        let (_, pack) = resolve(&self.packs, language.as_deref())?;
        Ok(pack.notice().to_owned())
    }

    /// The pack's source licences, as a JSON array of strings.
    pub fn licences(&self, language: Option<String>) -> Result<String, JsError> {
        let (_, pack) = resolve(&self.packs, language.as_deref())?;
        Ok(serde_json::to_string(&pack.meta().licences).unwrap_or_else(|_| "[]".to_owned()))
    }
}
