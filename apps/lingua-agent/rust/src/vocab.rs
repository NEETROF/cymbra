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
//! consent) — never persisted at listing time. Each word belongs to the language of the
//! reply it was met in, and goes to that language's deck (add-lingua-agent-languages D5).

use std::collections::BTreeMap;

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::percent::TokenClass;
use lingua_core::decks::card::{Card, EncounterSource, Provenance};
use lingua_core::knowledge::profile::NativeLanguage;
use lingua_core::knowledge::state::KnowledgeState;
use lingua_core::knowledge::status::Status;

use crate::engine::{Library, Skipped, analyse, blocks};
use crate::store::Store;

/// One unknown word offered by `/vocab`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabWord {
    /// The language of the reply it was met in.
    pub language: StudiedLanguage,
    /// The dictionary form (never labelled "lemma" in output).
    pub lemma: String,
    /// The native-language gloss, if the pack has one.
    pub gloss: Option<String>,
    /// Plain-language rarity note.
    pub rarity: String,
    /// The example sentence (the source line), captured onto a card only on add.
    pub sentence: String,
}

/// The distinct unknown words across `texts`, first occurrence wins, by language in the
/// order the plugin follows them, then by dictionary form. `calibration` gives each
/// language's, for the rarity note.
pub fn vocab_words(
    library: &mut Library,
    knowledge: &KnowledgeState,
    texts: &[String],
    calibration: impl Fn(StudiedLanguage) -> u32,
) -> Vec<VocabWord> {
    let order = library.languages();
    let mut found: BTreeMap<(usize, String), VocabWord> = BTreeMap::new();
    for text in texts {
        let Some(language) = library.language_of(text) else {
            continue;
        };
        let Some(pack) = library.pack(language) else {
            continue;
        };
        let rank = order
            .iter()
            .position(|l| *l == language)
            .unwrap_or(usize::MAX);
        let lines = blocks(text);
        for token in &analyse(pack, knowledge, text).tokens {
            let key = (rank, token.lemma.clone());
            if token.class != TokenClass::Unknown || found.contains_key(&key) {
                continue;
            }
            let sentence = lines
                .get(token.block)
                .map(|b| b.trim().to_owned())
                .unwrap_or_default();
            let threshold = calibration(language);
            found.insert(
                key,
                VocabWord {
                    language,
                    lemma: token.lemma.clone(),
                    gloss: pack.gloss(&token.lemma).map(str::to_owned),
                    rarity: format!(
                        "peu fréquent — au-delà de tes {threshold} mots les plus courants"
                    ),
                    sentence,
                },
            );
        }
    }
    found.into_values().collect()
}

/// The language's name in the plugin's French copy.
pub fn language_name(language: StudiedLanguage) -> &'static str {
    match language {
        StudiedLanguage::English => "Anglais",
        StudiedLanguage::Spanish => "Espagnol",
    }
}

/// A native language's name in the plugin's French copy, as a gloss language.
pub fn native_name(native: NativeLanguage) -> &'static str {
    match native {
        NativeLanguage::French => "français",
        NativeLanguage::English => "anglais",
        NativeLanguage::Spanish => "espagnol",
    }
}

/// What `/vocab` says after its listing of the packs installed but not followed, because they are
/// glossed in another native language than the plugin's (generalise-lingua-native-language D9):
/// one line per file, or nothing — every install today. The hooks and the statusline stay silent.
pub fn skipped_notice(skipped: &[Skipped], native: Option<NativeLanguage>) -> String {
    let Some(native) = native else {
        return String::new();
    };
    skipped
        .iter()
        .map(|skip| {
            let name = skip
                .file
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| skip.file.display().to_string());
            format!(
                "Pack ignoré : {name} — traduit en {}, alors que le plugin suit les packs traduits en {}.\n",
                native_name(skip.native),
                native_name(native)
            )
        })
        .collect()
}

/// The `/vocab` listing: one line per word, under a heading per language when the plugin
/// follows several (French UI copy).
pub fn listing(words: &[VocabWord], several: bool) -> String {
    if words.is_empty() {
        return "Aucun mot inconnu dans cette session.\n".to_owned();
    }
    let mut out = String::from("Mots inconnus de la session :\n");
    let mut heading: Option<StudiedLanguage> = None;
    for w in words {
        if several && heading != Some(w.language) {
            heading = Some(w.language);
            out.push_str(&format!("{} :\n", language_name(w.language)));
        }
        let gloss = w.gloss.as_deref().unwrap_or("—");
        out.push_str(&format!("  • {} — {} ({})\n", w.lemma, gloss, w.rarity));
    }
    out.push_str("Ajoute-les avec : lingua vocab --transcript <path> --add mot1,mot2");
    if several {
        out.push_str(" [--language en|es]");
    }
    out.push('\n');
    out
}

/// What adding a selection did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Added {
    /// How many cards were created.
    pub added: usize,
    /// The words listed under several languages, left out for want of a language.
    pub ambiguous: Vec<String>,
}

/// Add the chosen words to the deck (status learning + a card carrying the sentence), each
/// in the language it was listed under. A word listed under several needs `language`.
/// Only the words present in `available` are added.
pub fn add_to_deck(
    store: &Store,
    available: &[VocabWord],
    chosen: &[String],
    language: Option<StudiedLanguage>,
    timestamp: i64,
) -> rusqlite::Result<Added> {
    let mut added = 0;
    let mut ambiguous = Vec::new();
    for word in chosen {
        let key = word.to_lowercase();
        let matching: Vec<&VocabWord> = available
            .iter()
            .filter(|w| w.lemma == key && language.is_none_or(|l| w.language == l))
            .collect();
        let vocab = match matching.as_slice() {
            [] => continue,
            [vocab] => *vocab,
            _ => {
                ambiguous.push(key);
                continue;
            }
        };
        store.set_status(vocab.language, &vocab.lemma, Status::Learning)?;
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
        store.upsert_card(vocab.language, &card)?;
        added += 1;
    }
    Ok(Added { added, ambiguous })
}
