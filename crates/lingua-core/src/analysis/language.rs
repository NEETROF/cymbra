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

//! Per-block language gating (design D4).
//!
//! Real pages mix languages (native-language UI, quotations, code), so
//! detection runs per block of text, never per document. A block outside the
//! studied language — the user's native language included — is excluded from
//! the analysis entirely; a document with too little studied-language content
//! is *not analysable* rather than misleadingly scored.

use serde::{Deserialize, Serialize};

/// Languages the pipeline can study. Extended change by change; each variant
/// carries its own tokenisation pre-pass, lemmatisation cascade and analyser
/// version (generalise-lingua-analysis-by-language).
///
/// New variants go AFTER the existing ones: the derived order keys serialised
/// maps, so appending keeps every stored state byte-identical.
///
/// `Ord`/`Hash` so it can key the knowledge model's per-`(language, lemma)`
/// maps (`add-lingua-knowledge-model`); ordering keeps serialised state
/// deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StudiedLanguage {
    /// English.
    English,
    /// Spanish. Its analyser is a baseline until its own pre-pass and cascade
    /// land (add-lingua-spanish-analysis): no English rule ever runs on it.
    Spanish,
}

impl StudiedLanguage {
    /// Every language the core can analyse, in declaration order.
    pub const ALL: [StudiedLanguage; 2] = [StudiedLanguage::English, StudiedLanguage::Spanish];

    /// ISO 639-1 tag: `en`, `es`. The form packs (`meta.studied`), the wire
    /// and the extension use.
    pub fn tag(self) -> &'static str {
        match self {
            StudiedLanguage::English => "en",
            StudiedLanguage::Spanish => "es",
        }
    }

    /// The language a tag names, or `None` for a language the core has no
    /// analyser for. Exact: tags are normalised where they enter the system.
    pub fn from_tag(tag: &str) -> Option<StudiedLanguage> {
        StudiedLanguage::ALL.into_iter().find(|l| l.tag() == tag)
    }

    /// Version of this language's analysis pipeline. Each language has its
    /// own, so a change to one language's rules never invalidates another's
    /// pack or counts. Bump on ANY change that can alter this language's
    /// output (see [`crate::analysis::ANALYZER_VERSION`] for English's).
    pub fn analyzer_version(self) -> &'static str {
        match self {
            StudiedLanguage::English => crate::analysis::ANALYZER_VERSION,
            StudiedLanguage::Spanish => crate::analysis::SPANISH_ANALYZER_VERSION,
        }
    }

    fn whichlang_target(self) -> whichlang::Lang {
        match self {
            StudiedLanguage::English => whichlang::Lang::Eng,
            StudiedLanguage::Spanish => whichlang::Lang::Spa,
        }
    }
}

/// Blocks shorter than this (in bytes, once trimmed) are too small for
/// reliable detection; they are excluded from the analysis rather than
/// guessed at.
pub const MIN_BLOCK_BYTES: usize = 12;

/// A document whose studied-language blocks yield fewer counted tokens than
/// this is reported not analysable (spec: "without enough content in the
/// studied language").
pub const MIN_ANALYSABLE_TOKENS: usize = 10;

/// Whether one block of text is in the studied language.
pub fn block_is_studied(text: &str, studied: StudiedLanguage) -> bool {
    let trimmed = text.trim();
    if trimmed.len() < MIN_BLOCK_BYTES {
        return false;
    }
    whichlang::detect_language(trimmed) == studied.whichlang_target()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_block_is_studied() {
        assert!(block_is_studied(
            "The quick brown fox jumps over the lazy dog every single morning.",
            StudiedLanguage::English,
        ));
    }

    #[test]
    fn french_block_is_excluded() {
        assert!(!block_is_studied(
            "Les équipes ne livrent jamais le vendredi soir, c'est une règle ancienne.",
            StudiedLanguage::English,
        ));
    }

    #[test]
    fn a_spanish_block_is_studied_by_a_spanish_learner_only() {
        let block = "Los equipos nunca entregan el viernes por la noche, es una regla antigua.";
        assert!(block_is_studied(block, StudiedLanguage::Spanish));
        assert!(!block_is_studied(block, StudiedLanguage::English));
    }

    #[test]
    fn an_english_block_is_excluded_for_a_spanish_learner() {
        assert!(!block_is_studied(
            "The quick brown fox jumps over the lazy dog every single morning.",
            StudiedLanguage::Spanish,
        ));
    }

    #[test]
    fn tags_round_trip_and_unknown_tags_have_no_analyser() {
        for lang in StudiedLanguage::ALL {
            assert_eq!(StudiedLanguage::from_tag(lang.tag()), Some(lang));
        }
        assert_eq!(StudiedLanguage::from_tag("pt"), None);
        assert_eq!(
            StudiedLanguage::from_tag("EN"),
            None,
            "tags are normalised upstream"
        );
    }

    #[test]
    fn english_keeps_its_analyser_version_and_spanish_has_its_own() {
        assert_eq!(StudiedLanguage::English.analyzer_version(), "1.1.0");
        assert_ne!(
            StudiedLanguage::Spanish.analyzer_version(),
            StudiedLanguage::English.analyzer_version()
        );
    }

    #[test]
    fn appending_spanish_keeps_english_first_and_its_serialised_name() {
        assert!(StudiedLanguage::English < StudiedLanguage::Spanish);
        assert_eq!(
            serde_json::to_string(&StudiedLanguage::English).unwrap(),
            "\"English\""
        );
    }

    #[test]
    fn tiny_blocks_are_excluded_not_guessed() {
        assert!(!block_is_studied("OK", StudiedLanguage::English));
        assert!(!block_is_studied("  the  ", StudiedLanguage::English));
    }
}
