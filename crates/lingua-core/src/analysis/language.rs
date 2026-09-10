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

/// Languages the pipeline can study. Extended change by change (Romance
/// languages next); each variant carries its own tokenisation pre-pass.
///
/// `Ord`/`Hash` so it can key the knowledge model's per-`(language, lemma)`
/// maps (`add-lingua-knowledge-model`); ordering keeps serialised state
/// deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StudiedLanguage {
    /// English.
    English,
}

impl StudiedLanguage {
    fn whichlang_target(self) -> whichlang::Lang {
        match self {
            StudiedLanguage::English => whichlang::Lang::Eng,
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
    fn tiny_blocks_are_excluded_not_guessed() {
        assert!(!block_is_studied("OK", StudiedLanguage::English));
        assert!(!block_is_studied("  the  ", StudiedLanguage::English));
    }
}
