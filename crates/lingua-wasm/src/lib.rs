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

//! WASM bindings for Cymbra Lingua (spec `lingua-analysis` — native/WASM
//! parity).
//!
//! A thin wasm-bindgen surface over `lingua-core`: it loads a pack, holds the
//! knowledge state, and analyses a batch of blocks — no logic lives here, so
//! the WASM output is byte-for-byte identical to the native one at an equal
//! `analyzer_version` and equal pack. Build with `wasm-pack build --target
//! web`; the where-it-runs question (content script vs event page) is the
//! extension's, behind its `AnalyzerPort`.

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::engine::analyse_page_json;
use lingua_core::knowledge::state::KnowledgeState;
use lingua_core::knowledge::status::{KnownSource, Status};
use lingua_core::packs::Pack;
use wasm_bindgen::prelude::*;

/// A loaded analysis engine: one pack plus the user's knowledge state.
#[wasm_bindgen]
pub struct LinguaEngine {
    pack: Pack,
    knowledge: KnowledgeState,
}

#[wasm_bindgen]
impl LinguaEngine {
    /// Loads the engine from pack bytes (the embedded `pack.lingua`). Errors
    /// if the pack is malformed or built for an incompatible analyser.
    #[wasm_bindgen(constructor)]
    pub fn new(pack_bytes: &[u8]) -> Result<LinguaEngine, JsError> {
        let pack = Pack::load(pack_bytes).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(LinguaEngine {
            pack,
            knowledge: KnowledgeState::new(),
        })
    }

    /// Sets the English calibration threshold ("I know the N most common
    /// words").
    #[wasm_bindgen(js_name = setCalibration)]
    pub fn set_calibration(&mut self, threshold: u32) {
        self.knowledge
            .set_calibration(StudiedLanguage::English, threshold);
    }

    /// Sets an explicit status for a lemma. `status` is one of `learning`,
    /// `known`, `ignored`; anything else clears it.
    #[wasm_bindgen(js_name = setStatus)]
    pub fn set_status(&mut self, lemma: &str, status: &str) {
        match status {
            "learning" => {
                self.knowledge
                    .set_status(StudiedLanguage::English, lemma, Status::Learning)
            }
            "known" => self.knowledge.set_status(
                StudiedLanguage::English,
                lemma,
                Status::Known(KnownSource::Manual),
            ),
            "ignored" => {
                self.knowledge
                    .set_status(StudiedLanguage::English, lemma, Status::Ignored)
            }
            _ => self.knowledge.clear_status(StudiedLanguage::English, lemma),
        }
    }

    /// Analyses a batch of blocks, returning the canonical JSON of the page
    /// analysis (classified tokens, statuses, percentage, glosses).
    pub fn analyse(&self, blocks: Vec<String>) -> String {
        let refs: Vec<&str> = blocks.iter().map(String::as_str).collect();
        analyse_page_json(&refs, StudiedLanguage::English, &self.pack, &self.knowledge)
    }

    /// The native-language gloss for a lemma, if the pack carries one.
    pub fn gloss(&self, lemma: &str) -> Option<String> {
        self.pack.gloss(lemma).map(str::to_owned)
    }
}
