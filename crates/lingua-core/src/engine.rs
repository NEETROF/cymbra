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
//!
//! The same file glosses a reader's selection ([`gloss_phrase`]): the same
//! tokeniser, lemma cascade and classification as the page, without the gates
//! that exist to score pages (`add-lingua-phrase-gloss`).

use serde::Serialize;

use crate::analysis::ANALYZER_VERSION;
use crate::analysis::function_words::is_function_word;
use crate::analysis::language::StudiedLanguage;
use crate::analysis::percent::{
    Coverage, TokenClass, compound_is_out_of_lexicon_proper_noun, is_out_of_lexicon_proper_noun,
};
use crate::analysis::pipeline::{DocumentAnalysis, analyse_document, resolve_lemmas};
use crate::analysis::tokenize::tokenize;
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
        let class = classify_token(
            &token.surface,
            &token.lemma,
            &token.parts,
            studied,
            pack,
            knowledge,
        );
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

/// The one classification of a token, shared by the page analysis and the
/// phrase gloss so the two can never disagree: a plain token is a proper noun
/// outside the lexicon or whatever the knowledge model says of its lemma; a
/// listed compound has no parts and goes the same way; an unlisted compound is
/// a hyphenated name (`Jean-Pierre`, excluded like any proper noun) or is
/// judged by its parts.
fn classify_token(
    surface: &str,
    lemma: &str,
    parts: &[String],
    studied: StudiedLanguage,
    pack: &Pack,
    knowledge: &KnowledgeState,
) -> TokenClass {
    if parts.is_empty() {
        if is_out_of_lexicon_proper_noun(surface, lemma, pack.lexicon()) {
            TokenClass::ProperNounOutOfLexicon
        } else {
            knowledge.classify(studied, &[lemma], pack)
        }
    } else if compound_is_out_of_lexicon_proper_noun(surface, parts, pack.lexicon()) {
        TokenClass::ProperNounOutOfLexicon
    } else {
        knowledge.classify_compound(studied, lemma, parts, pack)
    }
}

/// One part of a hyphenated compound the lexicon does not list, described like
/// a token of its own so the surface can filter parts exactly as it filters
/// words (`opt-in` must not show `in`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PhrasePart {
    /// The part's dictionary form.
    pub lemma: String,
    /// The part's own status class.
    pub class: TokenClass,
    /// The pack gloss of the part, whatever its class.
    pub gloss: Option<String>,
    /// Whether the part is a closed-class word of the studied language.
    pub function_word: bool,
}

/// One token of a glossed selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PhraseToken {
    /// Surface text (case preserved).
    pub surface: String,
    /// The dictionary form.
    pub lemma: String,
    /// How the token would highlight on a page.
    pub class: TokenClass,
    /// The pack gloss, whatever the class — a card opened on a known word
    /// shows it too — or `None` when the pack carries none.
    pub gloss: Option<String>,
    /// Whether the dictionary form is a closed-class word of the studied
    /// language, which a word-by-word gloss leaves out.
    pub function_word: bool,
    /// The parts of a hyphenated compound the lexicon does not list, empty
    /// (and omitted from the JSON) otherwise.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<PhrasePart>,
}

/// One expression of the pack's table found in a selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PhraseMatch {
    /// Index of the first token covered, into [`PhraseGloss::tokens`].
    pub start: usize,
    /// Index just past the last token covered.
    pub end: usize,
    /// The expression's dictionary form — the covered tokens' lemmas joined by
    /// single spaces — which is the pack's key and the card's.
    pub key: String,
    /// The expression's own status class, read on that key: the knowledge
    /// model treats an expression as a lemma of its own, so the reader can
    /// settle `give up` as they settle a word.
    pub class: TokenClass,
    /// The pack gloss of the expression.
    pub gloss: String,
}

/// The gloss of a reader's selection: its tokens, in order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PhraseGloss {
    /// Every token of the text, whatever its class.
    pub tokens: Vec<PhraseToken>,
    /// The expressions the pack recognised, in token order. Empty — and
    /// omitted from the JSON — on a pack with no expression table, so such a
    /// pack produces the bytes it produced before the table existed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub expressions: Vec<PhraseMatch>,
}

/// How many tokens an expression may span. 98.9 % of the table is five words
/// or fewer (`add-lingua-expression-table`, design D3), and every token of a
/// selection pays for the ones beyond it.
const EXPRESSION_WINDOW: usize = 5;

/// Finds the pack's expressions in an already-glossed selection: from each
/// token, the longest run whose dictionary forms are a key of the table wins,
/// and the next run starts past it, so a token belongs to at most one match.
/// Runs start at two tokens because every key holds a space — a single lemma
/// is a word, not an expression, and could never be one.
fn match_expressions(
    tokens: &[PhraseToken],
    studied: StudiedLanguage,
    pack: &Pack,
    knowledge: &KnowledgeState,
) -> Vec<PhraseMatch> {
    if !pack.has_expressions() {
        return Vec::new();
    }
    let mut matches = Vec::new();
    let mut start = 0;
    while start < tokens.len() {
        let longest = EXPRESSION_WINDOW.min(tokens.len() - start);
        let hit = (2..=longest).rev().find_map(|len| {
            let key = tokens[start..start + len]
                .iter()
                .map(|token| token.lemma.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let gloss = pack.expression(&key)?.to_owned();
            Some((len, key, gloss))
        });
        match hit {
            Some((len, key, gloss)) => {
                let class = knowledge.classify(studied, &[key.as_str()], pack);
                matches.push(PhraseMatch {
                    start,
                    end: start + len,
                    key,
                    class,
                    gloss,
                });
                start += len;
            }
            None => start += 1,
        }
    }
    matches
}

/// Glosses a short text — a reader's selection — the way a page is read:
/// the same tokeniser, the same lemma cascade and the same classification,
/// but as one block with neither the language detection nor the minimum
/// token count, which exist to score pages. A text in another language simply
/// comes back unglossed.
///
/// The expressions the pack recognises are reported beside the tokens, never
/// instead of them: what the table holds cannot change what the selection is
/// made of.
pub fn gloss_phrase(
    text: &str,
    studied: StudiedLanguage,
    pack: &Pack,
    knowledge: &KnowledgeState,
) -> PhraseGloss {
    let lexicon = pack.lexicon();
    let tokens: Vec<PhraseToken> = tokenize(text, studied, lexicon)
        .into_iter()
        .map(|token| {
            let (lemma, parts) = resolve_lemmas(&token, lexicon);
            let class = classify_token(&token.text, &lemma, &parts, studied, pack, knowledge);
            let parts = parts
                .into_iter()
                .map(|part| PhrasePart {
                    class: knowledge.classify(studied, &[part.as_str()], pack),
                    gloss: pack.gloss(&part).map(str::to_owned),
                    function_word: is_function_word(&part, studied),
                    lemma: part,
                })
                .collect();
            PhraseToken {
                surface: token.text,
                gloss: pack.gloss(&lemma).map(str::to_owned),
                function_word: is_function_word(&lemma, studied),
                lemma,
                class,
                parts,
            }
        })
        .collect();
    let expressions = match_expressions(&tokens, studied, pack, knowledge);
    PhraseGloss {
        tokens,
        expressions,
    }
}

/// The canonical JSON of a phrase gloss — like [`analyse_page_json`], the
/// exact string both the native and WASM targets must produce.
pub fn gloss_phrase_json(
    text: &str,
    studied: StudiedLanguage,
    pack: &Pack,
    knowledge: &KnowledgeState,
) -> String {
    serde_json::to_string(&gloss_phrase(text, studied, pack, knowledge))
        .expect("PhraseGloss serialises")
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

    /// The two sections of an expression table, from (key, gloss) pairs: the
    /// key FST, byte-wise sorted as `fst::MapBuilder` demands, and the gloss
    /// blob its dense ids index into.
    fn build_expr_sections(entries: &[(&str, &str)]) -> (Vec<u8>, Vec<u8>) {
        let mut sorted = entries.to_vec();
        sorted.sort_unstable();
        let mut keys = fst::MapBuilder::memory();
        for (id, (key, _)) in sorted.iter().enumerate() {
            keys.insert(key, id as u64).expect("sorted, unique keys");
        }
        let glosses: Vec<(u32, &str)> = sorted
            .iter()
            .enumerate()
            .map(|(id, (_, gloss))| (id as u32, *gloss))
            .collect();
        (keys.into_inner().expect("fst"), build_gloss_zst(&glosses))
    }

    /// Assembles an en→fr pack holding exactly the forms, lemmas, ranks and
    /// glosses a test names — nothing else, so what a scenario finds in the
    /// pack is what it put there. No expression table, as most scenarios want.
    fn build_pack(
        forms: &[(&str, &str)],
        lemmas: &[&str],
        ranks: &[(&str, u32)],
        glosses: &[(&str, &str)],
    ) -> Pack {
        build_pack_with_expressions(forms, lemmas, ranks, glosses, &[])
    }

    /// [`build_pack`] plus an expression table, keyed as the builder keys it:
    /// the words' dictionary forms joined by single spaces.
    fn build_pack_with_expressions(
        forms: &[(&str, &str)],
        lemmas: &[&str],
        ranks: &[(&str, u32)],
        glosses: &[(&str, &str)],
        expressions: &[(&str, &str)],
    ) -> Pack {
        let (forms, pool) = build_lexicon_blobs(forms, lemmas).expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let mut freq = vec![0u32; lex.lemma_count()];
        for (lemma, rank) in ranks {
            if let Some(id) = lex.id_of(lemma) {
                freq[id as usize] = *rank;
            }
        }
        let mut freq_bytes = Vec::new();
        for r in &freq {
            freq_bytes.extend_from_slice(&r.to_le_bytes());
        }
        let entries: Vec<(u32, &str)> = glosses
            .iter()
            .map(|(lemma, gloss)| {
                (
                    lex.id_of(lemma).expect("glossed lemma listed") as u32,
                    *gloss,
                )
            })
            .collect();
        let gloss = build_gloss_zst(&entries);
        let meta = serde_json::to_vec(&PackMeta {
            studied: "en".into(),
            native: "fr".into(),
            pack_version: "t".into(),
            analyzer_version: ANALYZER_VERSION.into(),
            licences: vec![],
        })
        .unwrap();
        let mut sections: Vec<(&str, &[u8])> = vec![
            (section::FORMS, &forms),
            (section::LEMMAS, pool.as_bytes()),
            (section::FREQ, &freq_bytes),
            (section::GLOSS_ZST, &gloss),
        ];
        // Built out here so the sections outlive the borrow, but written only
        // when there is a table — as the pack builder writes it, so a test
        // that names no expression gets the pack it got before the table
        // existed.
        let (expr_keys, expr_glosses) = build_expr_sections(expressions);
        if !expressions.is_empty() {
            sections.push((section::EXPR, &expr_keys));
            sections.push((section::EXPR_ZST, &expr_glosses));
        }
        Pack::load(&write_container(&meta, &sections)).expect("load")
    }

    fn sample_pack() -> Pack {
        build_pack(
            &[("teams", "team"), ("ships", "ship"), ("seldom", "seldom")],
            &["do", "not", "on", "friday", "the", "code", "team", "ship"],
            &[
                ("the", 1),
                ("do", 20),
                ("not", 30),
                ("on", 25),
                ("friday", 1_800),
                ("team", 900),
                ("ship", 1_200),
                ("code", 2_500),
            ],
            &[("seldom", "rarement")],
        )
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

    // --- The phrase gloss (spec `lingua-analysis`, "Phrase gloss") ---

    #[test]
    fn spec_scenario_a_selection_too_short_for_the_page_analysis() {
        let pack = build_pack(
            &[("gave", "give")],
            &["give", "up"],
            &[],
            &[("give", "donner"), ("up", "en haut")],
        );
        let knowledge = KnowledgeState::new();
        let phrase = gloss_phrase("gave up", EN, &pack, &knowledge);
        let tokens: Vec<(&str, &str, Option<&str>)> = phrase
            .tokens
            .iter()
            .map(|t| (t.surface.as_str(), t.lemma.as_str(), t.gloss.as_deref()))
            .collect();
        assert_eq!(
            tokens,
            [
                ("gave", "give", Some("donner")),
                ("up", "up", Some("en haut"))
            ]
        );
        // The particle is a function word, the verb is not.
        assert!(!phrase.tokens[0].function_word);
        assert!(phrase.tokens[1].function_word);
        // The same text is far under the page analysis' minimum token count.
        assert!(!analyse_page(&["gave up"], EN, &pack, &knowledge).analysable);
    }

    #[test]
    fn spec_scenario_a_known_word_keeps_its_gloss() {
        let pack = build_pack(&[], &["the", "city"], &[], &[("city", "ville")]);
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(EN, "city", Status::Known(KnownSource::Manual));
        let phrase = gloss_phrase("the city", EN, &pack, &knowledge);
        let city = phrase
            .tokens
            .iter()
            .find(|t| t.lemma == "city")
            .expect("city");
        assert_eq!(city.class, TokenClass::Known);
        assert_eq!(city.gloss.as_deref(), Some("ville"));
    }

    #[test]
    fn spec_scenario_function_words_are_told_apart() {
        let pack = build_pack(&[], &["in", "spite", "of"], &[], &[]);
        let phrase = gloss_phrase("in spite of", EN, &pack, &KnowledgeState::new());
        let flags: Vec<(&str, bool)> = phrase
            .tokens
            .iter()
            .map(|t| (t.lemma.as_str(), t.function_word))
            .collect();
        assert_eq!(flags, [("in", true), ("spite", false), ("of", true)]);
    }

    #[test]
    fn spec_scenario_a_compound_the_lexicon_does_not_list() {
        let pack = build_pack(
            &[],
            &["error", "prone"],
            &[],
            &[("error", "erreur"), ("prone", "enclin")],
        );
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(EN, "error", Status::Known(KnownSource::Manual));
        let phrase = gloss_phrase("error-prone", EN, &pack, &knowledge);
        assert_eq!(phrase.tokens.len(), 1);
        let token = &phrase.tokens[0];
        assert_eq!(token.lemma, "error-prone");
        assert!(token.gloss.is_none(), "no gloss of its own");
        // Judged by its weakest part, as on a page.
        assert_eq!(token.class, TokenClass::Unknown);
        assert_eq!(
            token.parts,
            [
                PhrasePart {
                    lemma: "error".into(),
                    class: TokenClass::Known,
                    gloss: Some("erreur".into()),
                    function_word: false,
                },
                PhrasePart {
                    lemma: "prone".into(),
                    class: TokenClass::Unknown,
                    gloss: Some("enclin".into()),
                    function_word: false,
                },
            ]
        );
    }

    #[test]
    fn a_part_that_is_a_function_word_is_flagged_as_one() {
        // `opt-in` must not show `in` in a word-by-word gloss (design D1).
        let pack = build_pack(&[], &["opt", "in"], &[], &[("opt", "choisir")]);
        let phrase = gloss_phrase("opt-in", EN, &pack, &KnowledgeState::new());
        let flags: Vec<(&str, bool)> = phrase.tokens[0]
            .parts
            .iter()
            .map(|p| (p.lemma.as_str(), p.function_word))
            .collect();
        assert_eq!(flags, [("opt", false), ("in", true)]);
        assert!(
            !phrase.tokens[0].function_word,
            "the compound itself is not"
        );
    }

    #[test]
    fn spec_scenario_a_compound_the_lexicon_lists() {
        let pack = build_pack(
            &[("x-ray", "x-ray")],
            &[],
            &[],
            &[("x-ray", "radiographie")],
        );
        let phrase = gloss_phrase("x-ray", EN, &pack, &KnowledgeState::new());
        assert_eq!(phrase.tokens.len(), 1);
        let token = &phrase.tokens[0];
        assert_eq!(token.lemma, "x-ray");
        assert_eq!(token.gloss.as_deref(), Some("radiographie"));
        assert!(
            token.parts.is_empty(),
            "a listed compound is a lexical unit"
        );
    }

    #[test]
    fn spec_scenario_a_name_outside_the_lexicon() {
        let pack = build_pack(&[], &["the", "city"], &[], &[]);
        let phrase = gloss_phrase("Jenkins", EN, &pack, &KnowledgeState::new());
        assert_eq!(phrase.tokens.len(), 1);
        assert_eq!(
            phrase.tokens[0].class,
            TokenClass::ProperNounOutOfLexicon,
            "as it would on a page"
        );
        assert!(phrase.tokens[0].gloss.is_none());
    }

    #[test]
    fn a_hyphenated_name_is_a_proper_noun_outside_the_lexicon() {
        // The compound branch of the shared classifier: no part is a lexicon
        // word and the surface is sentence-cased, exactly as on a page.
        let pack = build_pack(&[], &["the", "city"], &[], &[]);
        let phrase = gloss_phrase("Jean-Pierre", EN, &pack, &KnowledgeState::new());
        assert_eq!(phrase.tokens.len(), 1);
        assert_eq!(phrase.tokens[0].class, TokenClass::ProperNounOutOfLexicon);
        assert_eq!(phrase.tokens[0].parts.len(), 2, "its parts still travel");
    }

    #[test]
    fn spec_scenario_a_text_the_language_detector_would_reject() {
        // The pack happens to hold `code` — a French word too — and nothing else
        // of the sentence.
        let pack = build_pack(&[], &["the", "code"], &[], &[("code", "code")]);
        let phrase = gloss_phrase(
            "Les équipes ne livrent jamais le code le vendredi soir.",
            EN,
            &pack,
            &KnowledgeState::new(),
        );
        assert!(!phrase.tokens.is_empty(), "returned, not rejected");
        let glossed: Vec<&str> = phrase
            .tokens
            .iter()
            .filter(|t| t.gloss.is_some())
            .map(|t| t.lemma.as_str())
            .collect();
        assert_eq!(
            glossed,
            ["code"],
            "glossed only where the pack holds the form"
        );
    }

    #[test]
    fn phrase_gloss_json_is_stable_across_runs_and_omits_empty_parts() {
        let pack = build_pack(
            &[("gave", "give")],
            &["give", "up", "error", "prone"],
            &[],
            &[("give", "donner"), ("prone", "enclin")],
        );
        let knowledge = KnowledgeState::new();
        let text = "gave up, error-prone";
        let a = gloss_phrase_json(text, EN, &pack, &knowledge);
        let b = gloss_phrase_json(text, EN, &pack, &knowledge);
        assert_eq!(a, b);
        // A plain token carries no `parts` key; an absent gloss is `null`.
        assert!(a.contains(
            r#"{"surface":"gave","lemma":"give","class":"Unknown","gloss":"donner","function_word":false}"#
        ));
        assert!(a.contains(
            r#""surface":"up","lemma":"up","class":"Unknown","gloss":null,"function_word":true}"#
        ));
        // An unlisted compound carries its parts, each described like a token.
        assert!(a.contains(
            r#""lemma":"error-prone","class":"Unknown","gloss":null,"function_word":false,"parts":[{"lemma":"error","class":"Unknown","gloss":null,"function_word":false},{"lemma":"prone","class":"Unknown","gloss":"enclin","function_word":false}]}"#
        ));
    }

    // --- Expressions in a selection (spec `lingua-analysis`, "Expression
    // lookup in a phrase gloss") ---

    /// The spans a phrase gloss reports, as (first token, past the last, key).
    fn spans(phrase: &PhraseGloss) -> Vec<(usize, usize, &str)> {
        phrase
            .expressions
            .iter()
            .map(|m| (m.start, m.end, m.key.as_str()))
            .collect()
    }

    #[test]
    fn spec_scenario_an_inflected_expression() {
        let pack = build_pack_with_expressions(
            &[("gave", "give")],
            &["give", "up"],
            &[],
            &[("give", "donner")],
            &[("give up", "Abandonner")],
        );
        let phrase = gloss_phrase("gave up", EN, &pack, &KnowledgeState::new());
        assert_eq!(
            phrase.expressions,
            [PhraseMatch {
                start: 0,
                end: 2,
                key: "give up".into(),
                class: TokenClass::Unknown,
                gloss: "Abandonner".into(),
            }],
            "matched on the dictionary forms, not the words as written"
        );
        // The tokens themselves are untouched by the match.
        let surfaces: Vec<&str> = phrase.tokens.iter().map(|t| t.surface.as_str()).collect();
        assert_eq!(surfaces, ["gave", "up"]);
    }

    #[test]
    fn spec_scenario_the_longest_run_wins() {
        let pack = build_pack_with_expressions(
            &[],
            &["look", "forward", "to", "it"],
            &[],
            &[],
            &[
                ("look forward", "Regarder devant"),
                ("look forward to", "Attendre avec impatience"),
            ],
        );
        let phrase = gloss_phrase("look forward to it", EN, &pack, &KnowledgeState::new());
        assert_eq!(spans(&phrase), [(0, 3, "look forward to")]);
        assert_eq!(phrase.expressions[0].gloss, "Attendre avec impatience");
    }

    #[test]
    fn spec_scenario_an_expression_inside_a_longer_selection() {
        let pack = build_pack_with_expressions(
            &[("starting", "start")],
            &["a", "compelling", "start", "point"],
            &[],
            &[("compelling", "convaincant")],
            &[("start point", "Point de départ")],
        );
        let phrase = gloss_phrase(
            "a compelling starting point",
            EN,
            &pack,
            &KnowledgeState::new(),
        );
        assert_eq!(spans(&phrase), [(2, 4, "start point")]);
        // Every word of the selection is still an ordinary token, `compelling`
        // with its own gloss: the card decides what a match replaces.
        let lemmas: Vec<&str> = phrase.tokens.iter().map(|t| t.lemma.as_str()).collect();
        assert_eq!(lemmas, ["a", "compelling", "start", "point"]);
        assert_eq!(phrase.tokens[1].gloss.as_deref(), Some("convaincant"));
    }

    #[test]
    fn spec_scenario_nothing_matches() {
        let pack = build_pack_with_expressions(
            &[],
            &["the", "city"],
            &[],
            &[("city", "ville")],
            &[("give up", "Abandonner")],
        );
        let phrase = gloss_phrase("the city", EN, &pack, &KnowledgeState::new());
        assert!(phrase.expressions.is_empty());
        let lemmas: Vec<&str> = phrase.tokens.iter().map(|t| t.lemma.as_str()).collect();
        assert_eq!(lemmas, ["the", "city"]);
    }

    #[test]
    fn spec_scenario_a_pack_without_the_table() {
        // The same selection under the same reader, with the table and without:
        // no match, and tokens byte-for-byte what they were.
        let lemmas: &[&str] = &["give", "up"];
        let bare = build_pack(&[("gave", "give")], lemmas, &[], &[("give", "donner")]);
        let with_table = build_pack_with_expressions(
            &[("gave", "give")],
            lemmas,
            &[],
            &[("give", "donner")],
            &[("give up", "Abandonner")],
        );
        let knowledge = KnowledgeState::new();
        assert!(!bare.has_expressions());
        let phrase = gloss_phrase("gave up", EN, &bare, &knowledge);
        assert!(phrase.expressions.is_empty());
        assert_eq!(
            phrase.tokens,
            gloss_phrase("gave up", EN, &with_table, &knowledge).tokens
        );
        // And in the JSON the field is not there at all, so such a pack
        // produces the bytes it produced before the table existed.
        let json = gloss_phrase_json("gave up", EN, &bare, &knowledge);
        assert!(!json.contains("expressions"));
        assert!(json.ends_with(r#""function_word":true}]}"#));
    }

    #[test]
    fn a_match_carries_the_status_the_reader_put_on_its_key() {
        // The card keys an expression by its dictionary form and offers a
        // word's actions on it, so the match has to say where the reader
        // stands on that key.
        let pack = build_pack_with_expressions(
            &[("gave", "give")],
            &["give", "up", "in", "spite", "of"],
            &[],
            &[],
            &[("give up", "Abandonner"), ("in spite of", "En dépit de")],
        );
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(EN, "give up", Status::Known(KnownSource::Manual));
        let settled = gloss_phrase("gave up", EN, &pack, &knowledge);
        assert_eq!(settled.expressions[0].class, TokenClass::Known);
        let fresh = gloss_phrase("in spite of", EN, &pack, &knowledge);
        assert_eq!(fresh.expressions[0].class, TokenClass::Unknown);
    }

    #[test]
    fn a_token_belongs_to_at_most_one_match() {
        let pack = build_pack_with_expressions(
            &[],
            &["give", "up", "with", "it"],
            &[],
            &[],
            &[("give up", "Abandonner"), ("up with", "En haut avec")],
        );
        let phrase = gloss_phrase("give up with it", EN, &pack, &KnowledgeState::new());
        assert_eq!(
            spans(&phrase),
            [(0, 2, "give up")],
            "`up` is taken, so the run starting on it is never tried"
        );
    }

    #[test]
    fn a_key_longer_than_the_window_is_never_reached() {
        // Runs stop at five tokens (design D3): a longer key would cost every
        // selection probes that 98.9 % of the table cannot answer.
        // Six words, each its own dictionary form, so the key the lookup would
        // build is exactly the one the table holds.
        let six = "give up on the whole thing";
        let pack = build_pack_with_expressions(
            &[],
            &["give", "up", "on", "the", "whole", "thing"],
            &[],
            &[],
            &[(six, "Tout laisser tomber")],
        );
        let phrase = gloss_phrase(six, EN, &pack, &KnowledgeState::new());
        assert!(phrase.expressions.is_empty());
    }
}
