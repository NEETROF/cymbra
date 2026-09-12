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
//! product [`LinguaState`] (knowledge + exposure + deck + FSRS) plus the loaded
//! pack and an optional in-flight review session — so a surface (the extension,
//! later the Apple app) can read, build a deck, run an FSRS review and take a
//! lossless backup without reaching into the core directly. No logic lives here:
//! the analysis output stays byte-for-byte identical to the native build at an
//! equal `analyzer_version` and pack (the parity contract). Build with
//! `wasm-pack build --target web`.

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::decks::backup::LinguaState;
use lingua_core::decks::card::{Card, EncounterSource, Provenance};
use lingua_core::decks::fsrs::{Rating, ReviewState};
use lingua_core::decks::review::ReviewSession;
use lingua_core::engine::analyse_page_json;
use lingua_core::knowledge::level::CefrLevel;
use lingua_core::knowledge::state::{FrequencyRanks, KnowledgeState};
use lingua_core::knowledge::status::{KnownSource, Status};
use lingua_core::packs::Pack;
use wasm_bindgen::prelude::*;

/// The MVP studies English only; every method hard-codes it (the pair-keyed
/// multi-language model arrives with a later change).
const EN: StudiedLanguage = StudiedLanguage::English;

/// A loaded analysis + review engine: one pack, the whole local state, and an
/// optional in-flight review session.
#[wasm_bindgen]
pub struct LinguaEngine {
    pack: Pack,
    state: LinguaState,
    session: Option<ReviewSession>,
}

#[wasm_bindgen]
impl LinguaEngine {
    /// Loads the engine from pack bytes (the embedded `pack.lingua`). Errors if
    /// the pack is malformed or built for an incompatible analyser.
    #[wasm_bindgen(constructor)]
    pub fn new(pack_bytes: &[u8]) -> Result<LinguaEngine, JsError> {
        let pack = Pack::load(pack_bytes).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(LinguaEngine {
            pack,
            state: LinguaState::default(),
            session: None,
        })
    }

    // --- Knowledge + analysis (the reading surface; signatures unchanged) ---

    /// Sets the English calibration threshold ("I know the N most common words").
    #[wasm_bindgen(js_name = setCalibration)]
    pub fn set_calibration(&mut self, threshold: u32) {
        self.state.knowledge.set_calibration(EN, threshold);
    }

    /// The current calibration threshold.
    pub fn calibration(&self) -> u32 {
        self.state.knowledge.calibration(EN)
    }

    /// Declares the reader's CEFR level (`"A1"`..`"C2"`); any other value —
    /// including `""` — clears it, returning to frequency calibration.
    #[wasm_bindgen(js_name = setDeclaredLevel)]
    pub fn set_declared_level(&mut self, level: &str) {
        match CefrLevel::from_label(level) {
            Some(l) => self.state.knowledge.set_declared_level(EN, l),
            None => self.state.knowledge.clear_declared_level(EN),
        }
    }

    /// The declared CEFR level label (`"A1"`..`"C2"`), or `None` if none is set.
    #[wasm_bindgen(js_name = declaredLevel)]
    pub fn declared_level(&self) -> Option<String> {
        self.state
            .knowledge
            .declared_level(EN)
            .map(|l| l.label().to_owned())
    }

    /// Like `setDeclaredLevel`, but stamps the decision with a sync timestamp
    /// (epoch millis, the caller's clock) so the outbox and cross-device LWW can
    /// order it. A `""` / invalid label is the explicit "débutant" decision (no
    /// level), which syncs just like a level.
    #[wasm_bindgen(js_name = setDeclaredLevelAt)]
    pub fn set_declared_level_at(&mut self, level: &str, at_ms: f64) {
        self.state
            .knowledge
            .set_declared_level_at(EN, CefrLevel::from_label(level), at_ms as i64);
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
                    "language": "en",
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
            if c.get("language").and_then(|v| v.as_str()).unwrap_or("en") != "en" {
                continue; // the MVP studies English only
            }
            let level =
                CefrLevel::from_label(c.get("level").and_then(|v| v.as_str()).unwrap_or(""));
            let updated_at = c
                .get("updated_at")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            if self
                .state
                .knowledge
                .apply_declared_level_lww(EN, level, updated_at)
            {
                changed += 1;
            }
        }
        Ok(changed)
    }

    /// Whether the loaded pack carries a CEFR level table (else the ladder and
    /// level-targeted feeding fall back to frequency bands).
    #[wasm_bindgen(js_name = hasLevels)]
    pub fn has_levels(&self) -> bool {
        self.pack.has_levels()
    }

    /// The CEFR progression ladder as JSON — an array of
    /// `{level, confirmed, presumed, toLearn, total}`, one row per level A1..C2,
    /// folded over the pack's lemmas at each level. `[]` when the pack carries no
    /// CEFR data.
    #[wasm_bindgen(js_name = levelLadder)]
    pub fn level_ladder(&self) -> String {
        if !self.pack.has_levels() {
            return "[]".to_owned();
        }
        let rows: Vec<serde_json::Value> = CefrLevel::ALL
            .iter()
            .map(|&level| {
                let lemmas = self.pack.lemmas_at_level(level);
                let stats =
                    self.state
                        .knowledge
                        .band_stats(EN, lemmas.iter().map(|(l, _)| *l), &self.pack);
                serde_json::json!({
                    "level": level.label(),
                    "confirmed": stats.confirmed,
                    "presumed": stats.presumed,
                    "toLearn": stats.to_learn,
                    "total": stats.total(),
                })
            })
            .collect();
        serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_owned())
    }

    /// Records one reading exposure per lemma (`source` tag, `at_ms` in millis),
    /// feeding the distinct-day counters that back exposure-confirmed known.
    /// Recording never changes a status (design D5) — promotion is the separate,
    /// explicit `promoteByExposure`.
    #[wasm_bindgen(js_name = recordExposures)]
    pub fn record_exposures(&mut self, lemmas: Vec<String>, source: &str, at_ms: f64) {
        let secs = (at_ms as i64).div_euclid(1000); // exposure timestamps are epoch seconds
        for lemma in &lemmas {
            self.state.exposure.record(EN, lemma, 1, source, secs);
        }
    }

    /// Confirms presumed-known lemmas that reading has vouched for: below the
    /// declared level, no explicit status, seen on at least `threshold_days`
    /// distinct days. Returns the number promoted to `Known(Exposure)`; a no-op
    /// (0) without a declared level. `at_ms` (millis) stamps the sync timestamp.
    #[wasm_bindgen(js_name = promoteByExposure)]
    pub fn promote_by_exposure(&mut self, threshold_days: u32, at_ms: f64) -> usize {
        self.state
            .knowledge
            .promote_by_exposure(
                EN,
                &self.state.exposure,
                &self.pack,
                threshold_days,
                at_ms as i64,
            )
            .len()
    }

    /// Sets an explicit status for a form. `status` is one of `learning`,
    /// `known`, `ignored`; anything else clears it.
    #[wasm_bindgen(js_name = setStatus)]
    pub fn set_status(&mut self, lemma: &str, status: &str) {
        match status {
            "learning" => self.state.knowledge.set_status(EN, lemma, Status::Learning),
            "known" => {
                self.state
                    .knowledge
                    .set_status(EN, lemma, Status::Known(KnownSource::Manual))
            }
            "ignored" => self.state.knowledge.set_status(EN, lemma, Status::Ignored),
            _ => self.state.knowledge.clear_status(EN, lemma),
        }
    }

    // --- Status sync (add-lingua-connected-clients §2): the extension's outbox
    // and cursor-pull talk to KnownWordsService in these shapes. English only. ---

    /// Like `setStatus`, but stamps the change with a sync timestamp (epoch
    /// millis, the caller's clock) so the outbox and cross-device LWW can order
    /// it. `status` is `learning` | `known` | `ignored`; anything else clears.
    #[wasm_bindgen(js_name = setStatusAt)]
    pub fn set_status_at(&mut self, lemma: &str, status: &str, at_ms: f64) {
        match Status::from_wire(status, "manual") {
            Some(s) => self
                .state
                .knowledge
                .set_status_at(EN, lemma, s, at_ms as i64),
            None => self.state.knowledge.clear_status(EN, lemma),
        }
    }

    /// The full set of explicit statuses as `StatusOp`-shaped JSON
    /// (`{language, lemma, status, provenance, updated_at}`), for a push (the
    /// first-sign-in full upload, or an incremental drain the caller filters).
    #[wasm_bindgen(js_name = exportStatusOps)]
    pub fn export_status_ops(&self) -> String {
        let ops: Vec<serde_json::Value> = self
            .state
            .knowledge
            .export_statuses()
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "language": "en",
                    "lemma": r.lemma,
                    "status": r.status.wire_kind(),
                    "provenance": r.status.wire_provenance(),
                    "updated_at": r.updated_at,
                })
            })
            .collect();
        serde_json::to_string(&ops).unwrap_or_else(|_| "[]".to_owned())
    }

    /// Apply a batch of pulled `StatusChange`s (JSON array of
    /// `{language, lemma, status, updated_at}`) under last-write-wins. Returns
    /// how many changed local state. A pulled change carries no provenance, so a
    /// synced `known` lands as manual (the provenance nuance stays device-local).
    #[wasm_bindgen(js_name = applyStatusChanges)]
    pub fn apply_status_changes(&mut self, json: &str) -> Result<usize, JsError> {
        let changes: Vec<serde_json::Value> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let mut changed = 0usize;
        for c in &changes {
            if c.get("language").and_then(|v| v.as_str()).unwrap_or("en") != "en" {
                continue; // the MVP studies English only
            }
            let Some(lemma) = c.get("lemma").and_then(|v| v.as_str()) else {
                continue;
            };
            let kind = c.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let updated_at = c
                .get("updated_at")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            if self.state.knowledge.apply_status_lww(
                EN,
                lemma,
                Status::from_wire(kind, "manual"),
                updated_at,
            ) {
                changed += 1;
            }
        }
        Ok(changed)
    }

    /// The whole deck as `CardOp`-shaped JSON for a push to `DeckService`
    /// (one card per lemma; `client_id` = lemma). The FSRS state travels as an
    /// opaque JSON string. `device_id` is left empty for the caller to attach;
    /// `client_ts` is the card's `updated_at` in millis.
    #[wasm_bindgen(js_name = exportCardOps)]
    pub fn export_card_ops(&self) -> String {
        let ops: Vec<serde_json::Value> = self
            .state
            .deck
            .export_cards()
            .into_iter()
            .map(|(_lang, card)| {
                let source = match &card.provenance.source {
                    EncounterSource::Web { url } => url.clone(),
                    _ => String::new(), // agent-captured cards are local-only; no source on the wire
                };
                serde_json::json!({
                    "client_id": card.lemma,
                    "language": "en",
                    "lemma": card.lemma,
                    "surface_form": card.encountered_form,
                    "source_sentence": card.provenance.sentence,
                    "source": source,
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
    /// card takes the op timestamp as a proxy; the deck preserves it thereafter.
    #[wasm_bindgen(js_name = applyCardOps)]
    pub fn apply_card_ops(&mut self, json: &str) -> Result<usize, JsError> {
        let ops: Vec<serde_json::Value> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let mut changed = 0usize;
        for op in &ops {
            if op.get("language").and_then(|v| v.as_str()).unwrap_or("en") != "en" {
                continue;
            }
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
            if self.state.deck.apply_card_lww(EN, card) {
                changed += 1;
            }
        }
        Ok(changed)
    }

    /// Analyses a batch of blocks, returning the canonical JSON of the page
    /// analysis (classified tokens, statuses, percentage, glosses).
    pub fn analyse(&self, blocks: Vec<String>) -> String {
        let refs: Vec<&str> = blocks.iter().map(String::as_str).collect();
        analyse_page_json(&refs, EN, &self.pack, &self.state.knowledge)
    }

    /// The native-language gloss for a form, if the pack carries one.
    pub fn gloss(&self, lemma: &str) -> Option<String> {
        self.pack.gloss(lemma).map(str::to_owned)
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
    pub fn add_card(
        &mut self,
        lemma: &str,
        surface: &str,
        sentence: &str,
        url: &str,
        gloss: Option<String>,
        captured_at: f64,
    ) {
        self.state.knowledge.set_status(EN, lemma, Status::Learning);
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
        self.state.deck.upsert(EN, card);
    }

    /// Seeds up to `count` deck cards from a CEFR level's lemmas. `order` is
    /// `"common"` (commonest-first, the default) or `"rare"`; unranked lemmas
    /// always sort last. Skips lemmas already carded or with an explicit status.
    /// `at` is Unix-epoch seconds. Returns the number actually added; a no-op (0)
    /// for an unknown level or a pack with no CEFR data.
    #[wasm_bindgen(js_name = seedLevel)]
    pub fn seed_level(&mut self, level: &str, count: usize, order: &str, at: f64) -> usize {
        let Some(lvl) = CefrLevel::from_label(level) else {
            return 0;
        };
        let mut items = self.pack.lemmas_at_level(lvl);
        if order == "rare" {
            items.sort_by_key(|(l, _)| {
                (
                    self.pack.rank(l).is_none(),
                    std::cmp::Reverse(self.pack.rank(l).unwrap_or(0)),
                )
            });
        } else {
            items.sort_by_key(|(l, _)| {
                (self.pack.rank(l).is_none(), self.pack.rank(l).unwrap_or(0))
            });
        }
        self.state.deck.seed_lemmas(
            EN,
            items.iter().map(|(l, g)| (*l, *g)),
            &self.state.knowledge,
            count,
            at as i64,
        )
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
    /// gloss, revealed, remaining }` — or `null` when the session is finished or
    /// not started.
    #[wasm_bindgen(js_name = reviewCurrent)]
    pub fn review_current(&self) -> Option<String> {
        let session = self.session.as_ref()?;
        let card = session.current(&self.state.deck)?;
        let view = serde_json::json!({
            "headword": card.lemma,
            "surface": card.encountered_form,
            "sentence": card.provenance.sentence,
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
    pub fn notice(&self) -> String {
        self.pack.notice().to_owned()
    }

    /// The pack's source licences, as a JSON array of strings.
    pub fn licences(&self) -> String {
        serde_json::to_string(&self.pack.meta().licences).unwrap_or_else(|_| "[]".to_owned())
    }
}
