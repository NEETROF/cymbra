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
use lingua_core::decks::fsrs::Rating;
use lingua_core::decks::review::ReviewSession;
use lingua_core::engine::analyse_page_json;
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

    /// Resets the whole state to empty defaults (a full reset). The caller
    /// re-applies its default calibration afterwards.
    pub fn reset(&mut self) {
        self.state = LinguaState::default();
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
