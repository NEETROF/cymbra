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

//! Document-level orchestration: language gating → tokenisation →
//! lemmatisation, block by block.
//!
//! Outputs are plain ordered vectors — serialisable, order-stable, and
//! therefore byte-for-byte comparable across runs and compilation targets
//! (the determinism contract, design D5).

use serde::Serialize;

use super::language::{MIN_ANALYSABLE_TOKENS, StudiedLanguage, block_is_studied};
use super::lemmatize::lemmatize;
use super::lexicon::Lexicon;
use super::tokenize::tokenize;

/// One analysed token occurrence, in document order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysedToken {
    /// Index of the source block in the submitted document.
    pub block: usize,
    /// Surface text after the tokeniser's pre-pass (case preserved).
    pub surface: String,
    /// The lemma the cascade produced (lowercase).
    pub lemma: String,
    /// Byte span of the source word inside its block.
    pub start: usize,
    /// Byte end (exclusive) of the source word inside its block.
    pub end: usize,
}

/// Result of analysing a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DocumentAnalysis {
    /// Too little studied-language content to score honestly.
    NotAnalysable,
    /// The studied-language tokens, in document order.
    Analysed(Vec<AnalysedToken>),
}

/// Analyses a document supplied as blocks of text (paragraphs, DOM blocks…).
///
/// Blocks outside the studied language are excluded entirely; when the
/// remaining studied content yields fewer than
/// [`MIN_ANALYSABLE_TOKENS`](super::language::MIN_ANALYSABLE_TOKENS) tokens,
/// the document is [`DocumentAnalysis::NotAnalysable`].
pub fn analyse_document(
    blocks: &[&str],
    studied: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> DocumentAnalysis {
    let mut out = Vec::new();
    for (block_idx, block) in blocks.iter().enumerate() {
        if !block_is_studied(block, studied) {
            continue;
        }
        for token in tokenize(block, studied, lexicon) {
            let lemma = lemmatize(&token.text, lexicon);
            out.push(AnalysedToken {
                block: block_idx,
                surface: token.text,
                lemma,
                start: token.start,
                end: token.end,
            });
        }
    }
    if out.len() < MIN_ANALYSABLE_TOKENS {
        return DocumentAnalysis::NotAnalysable;
    }
    DocumentAnalysis::Analysed(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lexicon::{FstLexicon, build_lexicon_blobs};

    fn lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) = build_lexicon_blobs(
            &[("teams", "team"), ("ships", "ship"), ("jumps", "jump")],
            &[
                "a", "i", "the", "quick", "brown", "fox", "over", "lazy", "dog", "do", "not",
                "code", "every", "single", "morning",
            ],
        )
        .expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    #[test]
    fn spec_scenario_fully_french_page_is_not_analysable() {
        let blocks = [
            "Les équipes ne livrent jamais le vendredi soir, c'est une règle ancienne.",
            "La revue de code reste obligatoire pour chaque changement proposé.",
        ];
        assert_eq!(
            analyse_document(&blocks, StudiedLanguage::English, &lexicon()),
            DocumentAnalysis::NotAnalysable
        );
    }

    #[test]
    fn french_blocks_are_excluded_english_ones_analysed() {
        let blocks = [
            "The quick brown fox jumps over the lazy dog every single morning.",
            "Les équipes ne livrent jamais le vendredi soir, c'est une règle ancienne.",
        ];
        match analyse_document(&blocks, StudiedLanguage::English, &lexicon()) {
            DocumentAnalysis::Analysed(tokens) => {
                assert!(
                    tokens.iter().all(|t| t.block == 0),
                    "only the English block counts"
                );
                assert!(tokens.len() >= 10);
                let jumps = tokens.iter().find(|t| t.surface == "jumps").expect("jumps");
                assert_eq!(jumps.lemma, "jump");
            }
            other => panic!("expected analysed document, got {other:?}"),
        }
    }

    #[test]
    fn spans_index_into_their_own_block() {
        let block = "The quick brown fox jumps over the lazy dog every single morning.";
        match analyse_document(&[block], StudiedLanguage::English, &lexicon()) {
            DocumentAnalysis::Analysed(tokens) => {
                for t in &tokens {
                    assert_eq!(&block[t.start..t.end], t.surface, "span must match surface");
                }
            }
            other => panic!("expected analysed document, got {other:?}"),
        }
    }
}
