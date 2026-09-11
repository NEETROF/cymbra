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

//! `/vocab`: list a session's unknown words (dictionary form, gloss, rarity) with the
//! example sentence they were seen in, and add a selection to the deck. The source
//! sentence is captured onto a card **only** when the user adds the word (explicit
//! consent) — never persisted at listing time.

use std::collections::BTreeMap;

use lingua_core::analysis::percent::TokenClass;
use lingua_core::decks::card::{Card, EncounterSource, Provenance};
use lingua_core::engine::analyse_page;
use lingua_core::knowledge::state::KnowledgeState;
use lingua_core::knowledge::status::Status;
use lingua_core::packs::Pack;

use crate::engine::EN;
use crate::store::Store;

/// One unknown word offered by `/vocab`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabWord {
    /// The dictionary form (never labelled "lemma" in output).
    pub lemma: String,
    /// The native-language gloss, if the pack has one.
    pub gloss: Option<String>,
    /// Plain-language rarity note.
    pub rarity: String,
    /// The example sentence (the source line), captured onto a card only on add.
    pub sentence: String,
}

/// The distinct unknown words across `texts`, first occurrence wins, in lemma order.
pub fn vocab_words(
    pack: &Pack,
    knowledge: &KnowledgeState,
    texts: &[String],
    calibration: u32,
) -> Vec<VocabWord> {
    let mut found: BTreeMap<String, VocabWord> = BTreeMap::new();
    for text in texts {
        let blocks: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
        let analysis = analyse_page(&blocks, EN, pack, knowledge);
        for token in &analysis.tokens {
            if token.class != TokenClass::Unknown || found.contains_key(&token.lemma) {
                continue;
            }
            let sentence = blocks
                .get(token.block)
                .map(|b| b.trim().to_owned())
                .unwrap_or_default();
            found.insert(
                token.lemma.clone(),
                VocabWord {
                    lemma: token.lemma.clone(),
                    gloss: pack.gloss(&token.lemma).map(str::to_owned),
                    rarity: format!(
                        "peu fréquent — au-delà de tes {calibration} mots les plus courants"
                    ),
                    sentence,
                },
            );
        }
    }
    found.into_values().collect()
}

/// Add the chosen words to the deck (status learning + a card carrying the sentence).
/// Only the words present in `available` are added; returns how many were created.
pub fn add_to_deck(
    store: &Store,
    available: &[VocabWord],
    chosen: &[String],
    timestamp: i64,
) -> rusqlite::Result<usize> {
    let mut added = 0;
    for word in chosen {
        let key = word.to_lowercase();
        let Some(vocab) = available.iter().find(|w| w.lemma == key) else {
            continue;
        };
        store.set_status(&vocab.lemma, Status::Learning)?;
        let card = Card::new(
            &vocab.lemma,
            &vocab.lemma,
            Provenance {
                sentence: vocab.sentence.clone(),
                source: EncounterSource::AgentSession {
                    session: "claude-code".to_owned(),
                },
                captured_at: timestamp,
            },
            vocab.gloss.clone(),
        );
        store.upsert_card(&card)?;
        added += 1;
    }
    Ok(added)
}
