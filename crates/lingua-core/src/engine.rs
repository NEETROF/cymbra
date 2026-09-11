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

//! The page-analysis entry point every surface calls.
//!
//! Ties the pieces together: [`analysis::pipeline`] tokenises and lemmatises a
//! document, the [`knowledge`](crate::knowledge) model classifies each token
//! against the user's state, the [`packs::Pack`] supplies ranks and glosses,
//! and [`analysis::percent`] folds it into a known-token percentage. The
//! output is a plain ordered, integer-only structure — no `f64` — so its
//! serialisation is byte-for-byte identical across the native and WASM
//! targets (the parity contract, `add-lingua-wasm`).

use serde::Serialize;

use crate::analysis::ANALYZER_VERSION;
use crate::analysis::language::StudiedLanguage;
use crate::analysis::percent::{
    Coverage, TokenClass, compound_is_out_of_lexicon_proper_noun, is_out_of_lexicon_proper_noun,
};
use crate::analysis::pipeline::{DocumentAnalysis, analyse_document};
use crate::knowledge::state::KnowledgeState;
use crate::packs::Pack;

/// One analysed token, ready for the surface to highlight and, on click,
/// gloss.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalyzedToken {
    /// Index of the source block in the submitted document.
    pub block: usize,
    /// Byte start of the source word in its block.
    pub start: usize,
    /// Byte end (exclusive) of the source word in its block.
    pub end: usize,
    /// Surface text (case preserved).
    pub surface: String,
    /// The dictionary form.
    pub lemma: String,
    /// How the token counts and highlights.
    pub class: TokenClass,
    /// The native-language gloss, when the token is not already known and the
    /// pack carries one (so the popup has it without a second call).
    pub gloss: Option<String>,
}

/// The analysis of one page (a batch of blocks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageAnalysis {
    /// The analyser generation that produced this (parity/comparability key).
    pub analyzer_version: String,
    /// Whether the page had enough studied-language content to analyse.
    pub analysable: bool,
    /// The analysed tokens, in document order (empty when not analysable).
    pub tokens: Vec<AnalyzedToken>,
    /// Occurrences that entered the percentage (proper nouns excluded).
    pub counted: u32,
    /// Occurrences counting as known.
    pub known: u32,
    /// The known-token percentage, or `None` when nothing was counted.
    pub percent: Option<u8>,
}

/// Analyses a page: a batch of text blocks in the studied language, against a
/// loaded pack and the user's knowledge state.
pub fn analyse_page(
    blocks: &[&str],
    studied: StudiedLanguage,
    pack: &Pack,
    knowledge: &KnowledgeState,
) -> PageAnalysis {
    let tokens = match analyse_document(blocks, studied, pack.lexicon()) {
        DocumentAnalysis::NotAnalysable => {
            return PageAnalysis {
                analyzer_version: ANALYZER_VERSION.to_owned(),
                analysable: false,
                tokens: Vec::new(),
                counted: 0,
                known: 0,
                percent: None,
            };
        }
        DocumentAnalysis::Analysed(tokens) => tokens,
    };

    let mut coverage = Coverage::default();
    let mut out = Vec::with_capacity(tokens.len());
    for token in tokens {
        let class = if token.parts.is_empty() {
            if is_out_of_lexicon_proper_noun(&token.surface, &token.lemma, pack.lexicon()) {
                TokenClass::ProperNounOutOfLexicon
            } else {
                knowledge.classify(studied, &[token.lemma.as_str()], pack)
            }
        } else if compound_is_out_of_lexicon_proper_noun(
            &token.surface,
            &token.parts,
            pack.lexicon(),
        ) {
            // A hyphenated name (`Jean-Pierre`), excluded like any proper noun.
            TokenClass::ProperNounOutOfLexicon
        } else {
            knowledge.classify_compound(studied, &token.lemma, &token.parts, pack)
        };
        coverage.add(class);
        // A gloss is only useful for words the reader does not yet know.
        let gloss = match class {
            TokenClass::Learning | TokenClass::Unknown => {
                pack.gloss(&token.lemma).map(str::to_owned)
            }
            _ => None,
        };
        out.push(AnalyzedToken {
            block: token.block,
            start: token.start,
            end: token.end,
            surface: token.surface,
            lemma: token.lemma,
            class,
            gloss,
        });
    }

    PageAnalysis {
        analyzer_version: ANALYZER_VERSION.to_owned(),
        analysable: true,
        tokens: out,
        counted: coverage.counted,
        known: coverage.known,
        percent: coverage.percent_rounded(),
    }
}

/// The canonical JSON of a page analysis — the exact string both the native
/// and WASM targets must produce for identical inputs.
pub fn analyse_page_json(
    blocks: &[&str],
    studied: StudiedLanguage,
    pack: &Pack,
    knowledge: &KnowledgeState,
) -> String {
    serde_json::to_string(&analyse_page(blocks, studied, pack, knowledge))
        .expect("PageAnalysis serialises")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lexicon::{FstLexicon, build_lexicon_blobs};
    use crate::knowledge::status::{KnownSource, Status};
    use crate::packs::format::write_container;
    use crate::packs::meta::PackMeta;
    use crate::packs::pack::section;

    const EN: StudiedLanguage = StudiedLanguage::English;

    fn build_gloss_zst(entries: &[(u32, &str)]) -> Vec<u8> {
        let mut payload = Vec::new();
        let mut index = Vec::new();
        for (id, gloss) in entries {
            let off = payload.len() as u32;
            payload.extend_from_slice(gloss.as_bytes());
            index.push((*id, off, gloss.len() as u32));
        }
        let mut raw = Vec::new();
        raw.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        for (id, off, len) in index {
            raw.extend_from_slice(&id.to_le_bytes());
            raw.extend_from_slice(&off.to_le_bytes());
            raw.extend_from_slice(&len.to_le_bytes());
        }
        raw.extend_from_slice(&payload);
        zstd::encode_all(raw.as_slice(), 19).expect("zstd")
    }

    fn sample_pack() -> Pack {
        let (forms, pool) = build_lexicon_blobs(
            &[("teams", "team"), ("ships", "ship"), ("seldom", "seldom")],
            &["do", "not", "on", "friday", "the", "code", "team", "ship"],
        )
        .expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let mut freq = vec![0u32; lex.lemma_count()];
        for (lemma, rank) in [
            ("the", 1),
            ("do", 20),
            ("not", 30),
            ("on", 25),
            ("friday", 1_800),
            ("team", 900),
            ("ship", 1_200),
            ("code", 2_500),
        ] {
            if let Some(id) = lex.id_of(lemma) {
                freq[id as usize] = rank;
            }
        }
        let mut freq_bytes = Vec::new();
        for r in &freq {
            freq_bytes.extend_from_slice(&r.to_le_bytes());
        }
        let gloss = build_gloss_zst(&[(lex.id_of("seldom").unwrap() as u32, "rarement")]);
        let meta = serde_json::to_vec(&PackMeta {
            studied: "en".into(),
            native: "fr".into(),
            pack_version: "t".into(),
            analyzer_version: ANALYZER_VERSION.into(),
            licences: vec![],
        })
        .unwrap();
        let bytes = write_container(
            &meta,
            &[
                (section::FORMS, &forms),
                (section::LEMMAS, pool.as_bytes()),
                (section::FREQ, &freq_bytes),
                (section::GLOSS_ZST, &gloss),
            ],
        );
        Pack::load(&bytes).expect("load")
    }

    #[test]
    fn analyses_a_page_with_calibration_and_a_gloss() {
        let pack = sample_pack();
        let mut knowledge = KnowledgeState::new();
        knowledge.set_calibration(EN, 3_000); // common words known, `seldom` (unranked) not
        let page = analyse_page(
            &["Teams do not ship code on Friday, and they seldom code the code."],
            EN,
            &pack,
            &knowledge,
        );
        assert!(page.analysable);
        let seldom = page
            .tokens
            .iter()
            .find(|t| t.lemma == "seldom")
            .expect("seldom");
        assert_eq!(seldom.class, TokenClass::Unknown);
        assert_eq!(seldom.gloss.as_deref(), Some("rarement"));
        // A known word carries no gloss.
        let team = page
            .tokens
            .iter()
            .find(|t| t.lemma == "ship")
            .expect("ship");
        assert_eq!(team.class, TokenClass::Known);
        assert!(team.gloss.is_none());
        // Most of the page is known; only the unranked words (and/they/seldom) are not.
        assert!(page.percent.unwrap() >= 70);
    }

    #[test]
    fn a_hyphenated_compound_is_one_token_judged_by_its_weakest_part() {
        let pack = sample_pack();
        let mut knowledge = KnowledgeState::new();
        knowledge.set_calibration(EN, 3_000);
        let page = analyse_page(
            &["The team-ship do not ship code, yet the code-seldom team do not ship."],
            EN,
            &pack,
            &knowledge,
        );
        assert!(page.analysable);
        // Both parts below the threshold → the compound is a single Known token.
        let known = page
            .tokens
            .iter()
            .find(|t| t.surface == "team-ship")
            .expect("team-ship is one token");
        assert_eq!(known.class, TokenClass::Known);
        assert!(known.gloss.is_none());
        // `seldom` is above the threshold, so the compound drops to Unknown, and
        // its whole-compound lemma has no gloss in the pack.
        let unknown = page
            .tokens
            .iter()
            .find(|t| t.surface == "code-seldom")
            .expect("code-seldom is one token");
        assert_eq!(unknown.class, TokenClass::Unknown);
        assert_eq!(unknown.lemma, "code-seldom");
        assert!(unknown.gloss.is_none());
    }

    #[test]
    fn a_sentence_initial_compound_is_counted_not_excluded_as_a_proper_noun() {
        let pack = sample_pack();
        let mut knowledge = KnowledgeState::new();
        knowledge.set_calibration(EN, 3_000);
        // "Team-ship" leads the sentence; both parts are known words, so it is
        // counted (Known) exactly as the lowercase "team-ship" is — not dropped
        // as a proper noun merely for being sentence-cased.
        let page = analyse_page(
            &["Team-ship do not ship the code, and the team-ship ships the code today."],
            EN,
            &pack,
            &knowledge,
        );
        let initial = page
            .tokens
            .iter()
            .find(|t| t.surface == "Team-ship")
            .expect("Team-ship");
        let mid = page
            .tokens
            .iter()
            .find(|t| t.surface == "team-ship")
            .expect("team-ship");
        assert_eq!(initial.class, TokenClass::Known);
        assert_eq!(
            initial.class, mid.class,
            "counting must not depend on sentence position"
        );

        // A genuine hyphenated name (no part is a known word) is still excluded.
        let named = analyse_page(
            &["Foo-bar do not ship the code on Friday, and the code ships the code today."],
            EN,
            &pack,
            &knowledge,
        );
        let foo = named
            .tokens
            .iter()
            .find(|t| t.surface == "Foo-bar")
            .expect("Foo-bar");
        assert_eq!(foo.class, TokenClass::ProperNounOutOfLexicon);
    }

    #[test]
    fn learning_status_beats_calibration_and_carries_a_gloss() {
        let pack = sample_pack();
        let mut knowledge = KnowledgeState::new();
        knowledge.set_calibration(EN, 6_000);
        knowledge.set_status(EN, "seldom", Status::Learning);
        knowledge.set_status(EN, "ship", Status::Known(KnownSource::Manual));
        let page = analyse_page(
            &["They do not ship the code on Friday, and they seldom ship the code, seldom."],
            EN,
            &pack,
            &knowledge,
        );
        assert!(page.analysable);
        let seldom = page
            .tokens
            .iter()
            .find(|t| t.lemma == "seldom")
            .expect("seldom");
        assert_eq!(seldom.class, TokenClass::Learning);
        assert_eq!(seldom.gloss.as_deref(), Some("rarement"));
    }

    #[test]
    fn json_is_stable_across_runs() {
        let pack = sample_pack();
        let knowledge = KnowledgeState::new();
        let blocks = ["Teams do not ship code on Friday."];
        let a = analyse_page_json(&blocks, EN, &pack, &knowledge);
        let b = analyse_page_json(&blocks, EN, &pack, &knowledge);
        assert_eq!(a, b);
        assert!(a.contains(&format!("\"analyzer_version\":\"{ANALYZER_VERSION}\"")));
    }
}
