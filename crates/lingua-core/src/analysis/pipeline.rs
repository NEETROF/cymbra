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
use super::tokenize::{Token, tokenize};

/// One analysed token occurrence, in document order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysedToken {
    /// Index of the source block in the submitted document.
    pub block: usize,
    /// Surface text after the tokeniser's pre-pass (case preserved).
    pub surface: String,
    /// The lemma the cascade produced (lowercase).
    pub lemma: String,
    /// Part lemmas of a hyphenated compound the lexicon does not know as a
    /// unit, empty otherwise. When present, the classifier judges the token by
    /// its weakest part rather than by `lemma` alone.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<String>,
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
            let (lemma, parts) = resolve_lemmas(&token, lexicon);
            out.push(AnalysedToken {
                block: block_idx,
                surface: token.text,
                lemma,
                parts,
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

/// Resolves a token's lemma, and — for a hyphenated compound the lexicon does
/// not know as a unit — its part lemmas.
///
/// A compound the lexicon recognises (`e-mail`, `x-ray` — every lemma carries
/// an identity form, so `lemma_of` catches ranked/glossed compounds too) is a
/// single lexical unit: its own lemma, no parts. An unknown compound
/// (`repo-wide`, `type-safe`) keeps the lowercased surface as its lemma — the
/// single-word suffix cascade has no business stemming it — and carries its
/// parts so the classifier can judge it by its weakest one.
fn resolve_lemmas(token: &Token, lexicon: &(impl Lexicon + ?Sized)) -> (String, Vec<String>) {
    if token.parts.is_empty() {
        return (lemmatize(&token.text, lexicon), Vec::new());
    }
    let whole = token.text.replace('\u{2019}', "'").to_lowercase();
    if let Some(lemma) = lexicon.lemma_of(&whole) {
        return (lemma.to_owned(), Vec::new());
    }
    let parts = token
        .parts
        .iter()
        .map(|piece| lemmatize(piece, lexicon))
        .collect();
    (whole, parts)
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

    #[test]
    fn an_unknown_compound_keeps_its_parts_a_known_one_is_a_unit() {
        // `x-ray` is a listed lexicon compound (a unit — its own lemma, no
        // parts); `code-quick` is not (split into part lemmas).
        let lexicon = {
            let (bytes, pool) =
                build_lexicon_blobs(&[("x-ray", "x-ray")], &["the", "fox", "code", "quick"])
                    .expect("build");
            FstLexicon::from_slices(bytes, &pool).expect("load")
        };
        let block = "The x-ray beats the code-quick fox that ran over the lazy dog today now.";
        match analyse_document(&[block], StudiedLanguage::English, &lexicon) {
            DocumentAnalysis::Analysed(tokens) => {
                let xray = tokens.iter().find(|t| t.surface == "x-ray").expect("x-ray");
                assert_eq!(xray.lemma, "x-ray");
                assert!(xray.parts.is_empty(), "a listed compound is a lexical unit");

                let cq = tokens
                    .iter()
                    .find(|t| t.surface == "code-quick")
                    .expect("code-quick");
                assert_eq!(cq.lemma, "code-quick", "unknown compound is not stemmed");
                assert_eq!(cq.parts, ["code", "quick"]);
                // The span still slices back to the surface.
                assert_eq!(&block[cq.start..cq.end], "code-quick");
            }
            other => panic!("expected analysed document, got {other:?}"),
        }
    }
}
