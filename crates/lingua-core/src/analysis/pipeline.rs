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
            let (lemma, parts) = resolve_lemmas(&token, studied, lexicon);
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
///
/// Crate-visible so the phrase gloss (`engine::gloss_phrase`) resolves a
/// selection's tokens exactly as a page's, without the gates above.
pub(crate) fn resolve_lemmas(
    token: &Token,
    studied: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> (String, Vec<String>) {
    if token.parts.is_empty() {
        return (lemmatize(&token.text, studied, lexicon), Vec::new());
    }
    let whole = token.text.replace('\u{2019}', "'").to_lowercase();
    if let Some(lemma) = lexicon.lemma_of(&whole) {
        return (lemma.to_owned(), Vec::new());
    }
    let parts = token
        .parts
        .iter()
        .map(|piece| lemmatize(piece, studied, lexicon))
        .collect();
    (whole, parts)
}

/// What the analysis reads in an expression's headword: its tokens, through the
/// studied language's own pre-pass, each paired with its dictionary form resolved
/// as a page's token is ([`resolve_lemmas`]) — the reading [`analyse_document`]
/// and the phrase gloss make of the same words on a page.
///
/// `None` when a word of the headword, split at its spaces, starts no token: a
/// single letter the lexicon does not list (`compte en t`), a word holding a
/// digit. The reading would then be shorter than the headword and match words
/// it does not hold (add-lingua-french-expression-keys D6).
///
/// The pack builder keys a French expression on it, through
/// [`french_expression_key`](crate::engine::french_expression_key), and so does
/// review, finding a French expression card by its name (D1, D3).
pub fn headword_reading(
    headword: &str,
    studied: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> Option<Vec<(Token, String)>> {
    let tokens = tokenize(headword, studied, lexicon);
    let mut start = 0;
    for word in headword.split(' ') {
        let end = start + word.len();
        if !word.is_empty() && !tokens.iter().any(|t| (start..end).contains(&t.start)) {
            return None;
        }
        start = end + 1;
    }
    Some(
        tokens
            .into_iter()
            .map(|token| {
                let (lemma, _) = resolve_lemmas(&token, studied, lexicon);
                (token, lemma)
            })
            .collect(),
    )
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

    // — a headword's reading (add-lingua-french-expression-keys D1, D6) —

    /// A French lexicon holding the words of the headwords below, `t` and `c` left out.
    fn french_lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) = build_lexicon_blobs(
            &[("a", "avoir"), ("est", "être")],
            &[
                "à", "le", "revoir", "coup", "de", "œil", "abord", "que", "être", "ce", "il", "y",
                "avoir", "compte", "en", "vitamine",
            ],
        )
        .expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    /// A headword's reading as (surface, dictionary form) pairs.
    fn reading(
        headword: &str,
        studied: StudiedLanguage,
        lexicon: &FstLexicon<Vec<u8>>,
    ) -> Option<Vec<(String, String)>> {
        headword_reading(headword, studied, lexicon).map(|tokens| {
            tokens
                .into_iter()
                .map(|(token, lemma)| (token.text, lemma))
                .collect()
        })
    }

    fn pairs(expected: &[(&str, &str)]) -> Option<Vec<(String, String)>> {
        Some(
            expected
                .iter()
                .map(|(surface, lemma)| ((*surface).to_owned(), (*lemma).to_owned()))
                .collect(),
        )
    }

    #[test]
    fn a_french_headword_is_read_as_a_page_is() {
        let lexicon = french_lexicon();
        let fr = StudiedLanguage::French;
        // A contracted article is `à` + `le`, in the case it was written in.
        assert_eq!(
            reading("au revoir", fr, &lexicon),
            pairs(&[("à", "à"), ("le", "le"), ("revoir", "revoir")])
        );
        assert_eq!(
            reading("Au revoir", fr, &lexicon),
            pairs(&[("À", "à"), ("le", "le"), ("revoir", "revoir")])
        );
        // An elided word is the word it stands for, its own token.
        assert_eq!(
            reading("coup d'œil", fr, &lexicon),
            pairs(&[("coup", "coup"), ("de", "de"), ("œil", "œil")])
        );
        assert_eq!(
            reading("d'abord", fr, &lexicon),
            pairs(&[("de", "de"), ("abord", "abord")])
        );
        // An inversion is read as words, the typographic apostrophe as the straight one.
        assert_eq!(
            reading("Qu\u{2019}est-ce que", fr, &lexicon),
            pairs(&[
                ("Que", "que"),
                ("est", "être"),
                ("ce", "ce"),
                ("que", "que")
            ])
        );
        // Each token is resolved to its dictionary form.
        assert_eq!(
            reading("il y a", fr, &lexicon),
            pairs(&[("il", "il"), ("y", "y"), ("a", "avoir")])
        );
    }

    #[test]
    fn a_headword_part_of_which_gives_no_token_has_no_reading() {
        // `t` and `c` are single letters the lexicon does not list: the tokeniser drops them,
        // and what remains (`compte en`) is not the headword.
        let lexicon = french_lexicon();
        assert_eq!(
            reading("compte en t", StudiedLanguage::French, &lexicon),
            None
        );
        assert_eq!(
            reading("vitamine c", StudiedLanguage::French, &lexicon),
            None
        );
        // A word holding a digit gives none either.
        assert_eq!(reading("coup 2", StudiedLanguage::French, &lexicon), None);
        // A headword's extra spaces are no words.
        assert_eq!(
            reading(" coup  de ", StudiedLanguage::French, &lexicon),
            pairs(&[("coup", "coup"), ("de", "de")])
        );
    }

    #[test]
    fn an_english_and_a_spanish_headword_read_as_their_tokens() {
        // English's pre-pass expands `n't`; Spanish's splits `al` into `a` + `el`. Only French's
        // pack keys its expressions on a reading; these show the function is every language's.
        let english = lexicon();
        assert_eq!(
            reading("do not jumps", StudiedLanguage::English, &english),
            pairs(&[("do", "do"), ("not", "not"), ("jumps", "jump")])
        );
        assert_eq!(
            reading("don't code", StudiedLanguage::English, &english),
            pairs(&[("do", "do"), ("not", "not"), ("code", "code")])
        );
        let spanish = {
            let (bytes, pool) =
                build_lexicon_blobs(&[], &["a", "el", "pie", "de", "letra"]).expect("build");
            FstLexicon::from_slices(bytes, &pool).expect("load")
        };
        assert_eq!(
            reading("al pie de la letra", StudiedLanguage::Spanish, &spanish),
            pairs(&[
                ("a", "a"),
                ("el", "el"),
                ("pie", "pie"),
                ("de", "de"),
                ("la", "la"),
                ("letra", "letra")
            ])
        );
    }
}
