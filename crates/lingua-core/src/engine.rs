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
//! that exist to score pages (`add-lingua-phrase-gloss`), and answers a word
//! card's grammar ([`word_grammar`], `add-lingua-word-grammar`).

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::analysis::function_words::is_function_word;
use crate::analysis::language::StudiedLanguage;
use crate::analysis::lexicon::Lexicon;
use crate::analysis::percent::{
    Coverage, TokenClass, compound_is_out_of_lexicon_proper_noun, is_out_of_lexicon_proper_noun,
};
use crate::analysis::pipeline::{
    AnalysedToken, DocumentAnalysis, analyse_document, headword_reading, resolve_lemmas,
};
use crate::analysis::tokenize::tokenize;
use crate::knowledge::state::KnowledgeState;
use crate::packs::Pack;
use crate::packs::grammar::Tag;

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
                analyzer_version: studied.analyzer_version().to_owned(),
                analysable: false,
                tokens: Vec::new(),
                counted: 0,
                known: 0,
                percent: None,
            };
        }
        DocumentAnalysis::Analysed(tokens) => tokens,
    };

    // A Spanish or French document's names are set aside like a proper noun outside the
    // lexicon (add-lingua-spanish-names, add-lingua-french-analysis D4); English's analysis
    // does not change.
    let names = match studied {
        StudiedLanguage::Spanish | StudiedLanguage::French => {
            document_names(&tokens, blocks, studied, pack)
        }
        StudiedLanguage::English => HashSet::new(),
    };
    let mut coverage = Coverage::default();
    let mut out = Vec::with_capacity(tokens.len());
    for token in tokens {
        let class = match classify_token(
            &token.surface,
            &token.lemma,
            &token.parts,
            studied,
            pack,
            knowledge,
        ) {
            // Only a word the reader has said nothing of: a name they mark stays theirs. French
            // also reads a hyphenated run the pack does not list as one form.
            TokenClass::Unknown
                if (token.parts.is_empty() || studied == StudiedLanguage::French)
                    && names.contains(&token.surface.to_lowercase()) =>
            {
                TokenClass::ProperNounOutOfLexicon
            }
            class => class,
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
        analyzer_version: studied.analyzer_version().to_owned(),
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

/// The forms a Spanish document writes as names (add-lingua-spanish-names D1):
/// never in lowercase in the document, capitalised at least once in
/// mid-sentence, and with a dictionary form that is not one of the pack's
/// dictionary words ([`Pack::is_dictionary_word`], add-lingua-pack-lexical-layer
/// D2), so what is set aside does not depend on the pack's glosses. `Nela`,
/// `Augusto` and `Eugenia` are lemmas of the lexicon, which the out-of-lexicon
/// rule leaves alone; a name that is a dictionary word (`Dios`) stays a word to
/// learn.
///
/// A French document's names follow the same rule with two French readings
/// (add-lingua-french-analysis D4): a capital right after an elided piece is in
/// mid-sentence ([`after_elided_piece`]: `l'Europe`, `qu'Augusto`, even at the
/// head of a sentence), and a hyphenated run the pack does not list is one form,
/// keyed as a word is by its surface lowercased (`Saint-Étienne`, `Jean-Pierre`,
/// whose parts `saint` and `pierre` resolve, so the compound rule keeps them).
/// Spanish reads neither: an apostrophe gives it no evidence, and its runs are
/// judged by their parts.
fn document_names(
    tokens: &[AnalysedToken],
    blocks: &[&str],
    studied: StudiedLanguage,
    pack: &Pack,
) -> HashSet<String> {
    let french = studied == StudiedLanguage::French;
    let mut lowercase = HashSet::new();
    let mut capitalised: HashMap<String, &str> = HashMap::new();
    for token in tokens
        .iter()
        .filter(|token| french || token.parts.is_empty())
    {
        let Some(first) = token.surface.chars().next() else {
            continue;
        };
        let form = token.surface.to_lowercase();
        let text = blocks.get(token.block).copied().unwrap_or("");
        if !first.is_uppercase() {
            lowercase.insert(form);
        } else if mid_sentence(text, token.start)
            || (french && after_elided_piece(text, token.start))
        {
            capitalised.entry(form).or_insert(&token.lemma);
        }
    }
    capitalised
        .into_iter()
        .filter(|(form, lemma)| !lowercase.contains(form) && !pack.is_dictionary_word(lemma))
        .map(|(form, _)| form)
        .collect()
}

/// Whether the word at byte `start` of `text` follows an elided piece: an
/// apostrophe, straight or typographic, right after a letter (`l'Europe`,
/// `d’Espagne`). The capital is then the word's own, whatever stands before the
/// piece (add-lingua-french-analysis D4).
fn after_elided_piece(text: &str, start: usize) -> bool {
    let Some(before) = text.get(..start) else {
        return false;
    };
    let mut back = before.chars().rev();
    matches!(back.next(), Some('\'' | '\u{2019}')) && back.next().is_some_and(char::is_alphabetic)
}

/// Whether the word at byte `start` of `text` stands in mid-sentence: right
/// after a letter, a digit, a comma or a semicolon. At the head of a block, a
/// sentence, a quotation or a line of dialogue (`—Augusto`), any word takes a
/// capital, so it says nothing of a name.
fn mid_sentence(text: &str, start: usize) -> bool {
    text.get(..start)
        .and_then(|before| before.trim_end().chars().next_back())
        .is_some_and(|c| c.is_alphanumeric() || matches!(c, ',' | ';'))
}

/// The one classification of a token, shared by the page analysis and the
/// phrase gloss so the two can never disagree — except for a Spanish or French
/// document's names (a French one's hyphenated runs among them), which only the
/// whole page can tell: a plain token is a proper noun
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
    /// The expression's dictionary form, which is the card's. For English and
    /// Spanish, the covered tokens' lemmas joined by single spaces, which is the
    /// pack's key. For French, the expression's name — its headword as the
    /// dictionary writes it (`au revoir`, `il y a`) — which the pack carries
    /// wherever it differs from the key (`à le revoir`, `il y avoir`), and the
    /// key itself where it carries none (add-lingua-french-expression-keys D3).
    pub key: String,
    /// The expression's own status class, read on that dictionary form: the
    /// knowledge model treats an expression as a lemma of its own, so the reader
    /// can settle `give up` as they settle a word.
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

/// How many tokens an English or Spanish expression may span. 98.9 % of the
/// English table is five words or fewer (`add-lingua-expression-table`, design
/// D3), and every token of a selection pays for the ones beyond it.
const EXPRESSION_WINDOW: usize = 5;

/// How many tokens a French expression may span: French's pieces lengthen a key
/// (`au fur et à mesure` is five words and six tokens), and seven tokens hold
/// 98.8 % of its keys as five hold 98.9 % of English's
/// (add-lingua-french-expression-keys D4). The builder leaves a longer key out.
pub const FRENCH_EXPRESSION_WINDOW: usize = 7;

/// The words a French expression key writes as the pre-pass gives them, in
/// lowercase, rather than as their dictionary form: the articles, the possessive
/// and demonstrative determiners and the pronouns that share their forms, sorted
/// (add-lingua-french-expression-keys D2).
///
/// M8 files `la`, `les` and `l'` under `le` and `une` under `un` (and the tables
/// `cette`, `ces` under `ce`, `sa`, `ses` under `son`…): right for a word, wrong
/// for an expression. Measured on the English Wiktionary's 17,523 French
/// candidates and change 39's corpus, every token lemmatised keys `à la` (« in the
/// style of ») as `à le`, 7 of whose 10 corpus matches fell on a contracted « au »
/// or « aux », and leaves 73 keys reached by several headwords against 60 with
/// these words written: the 13 told apart include distinct expressions
/// (`haut la main` and `haut les mains`, `faire la course` and
/// `faire les courses`).
pub const FRENCH_KEY_WRITTEN: &[&str] = &[
    "ce", "ces", "cet", "cette", "la", "le", "les", "leur", "leurs", "ma", "mes", "mon", "nos",
    "notre", "sa", "ses", "son", "ta", "tes", "ton", "un", "une", "vos", "votre",
];

/// A token's piece of an expression key (add-lingua-french-expression-keys D2):
/// its dictionary form, except in French a word of [`FRENCH_KEY_WRITTEN`], written
/// as the pre-pass gives it — lowercased, the typographic apostrophe read as the
/// straight one — so `à la` is never `au` (`à le`). English and Spanish keys are
/// their lemmas, as before.
pub fn expression_piece(surface: &str, lemma: &str, studied: StudiedLanguage) -> String {
    if studied == StudiedLanguage::French {
        let written = surface.replace('\u{2019}', "'").to_lowercase();
        if FRENCH_KEY_WRITTEN.binary_search(&written.as_str()).is_ok() {
            return written;
        }
    }
    lemma.to_owned()
}

/// The key a French expression's headword — or its name, which is the same
/// thing — is filed under (add-lingua-french-expression-keys D1, D4, D6): what
/// French's analysis reads in it ([`headword_reading`]), each token written by
/// [`expression_piece`], joined by single spaces: `au revoir` → `à le revoir`,
/// `coup d'œil` → `coup de œil`, `d'abord` → `de abord`, `il y a` →
/// `il y avoir`, `à la` → `à la`.
///
/// `None` unless the headword reads as two to [`FRENCH_EXPRESSION_WINDOW`] tokens,
/// every word of it giving one, and every token's dictionary form is a lemma of
/// the lexicon: a key no reading of a page can produce would sit in the pack
/// unreachable. The one implementation the pack builder keys a French pack with
/// and review finds a French expression card's gloss through, so the two cannot
/// drift; it asks the lexicon whether a dictionary form is a lemma, never which
/// lemma a form files under.
pub fn french_expression_key(headword: &str, lexicon: &(impl Lexicon + ?Sized)) -> Option<String> {
    let reading = headword_reading(headword, StudiedLanguage::French, lexicon)?;
    if !(2..=FRENCH_EXPRESSION_WINDOW).contains(&reading.len())
        || !reading
            .iter()
            .all(|(_, lemma)| lexicon.contains_lemma(lemma))
    {
        return None;
    }
    Some(
        reading
            .iter()
            .map(|(token, lemma)| expression_piece(&token.text, lemma, StudiedLanguage::French))
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// How many tokens an expression may span in the studied language.
fn expression_window(studied: StudiedLanguage) -> usize {
    match studied {
        StudiedLanguage::French => FRENCH_EXPRESSION_WINDOW,
        StudiedLanguage::English | StudiedLanguage::Spanish => EXPRESSION_WINDOW,
    }
}

/// The table's gloss for a run of key pieces, with the key it was found at.
///
/// A French run no key matches whose last piece is `du` or `des` is tried once
/// more with it read as `de`: M21 keeps `du` and `des` whole, and after a word
/// that governs `de` they are its contraction with the article that follows, so
/// « à cause des » answers `à cause de` (add-lingua-french-expression-keys D5).
fn expression_for_run(
    pieces: &[String],
    studied: StudiedLanguage,
    pack: &Pack,
) -> Option<(String, String)> {
    let key = pieces.join(" ");
    if let Some(gloss) = pack.expression(&key) {
        return Some((key, gloss.to_owned()));
    }
    let (last, before) = pieces.split_last()?;
    if studied != StudiedLanguage::French || !matches!(last.as_str(), "du" | "des") {
        return None;
    }
    let key = before
        .iter()
        .map(String::as_str)
        .chain(std::iter::once("de"))
        .collect::<Vec<_>>()
        .join(" ");
    let gloss = pack.expression(&key)?.to_owned();
    Some((key, gloss))
}

/// Finds the pack's expressions in an already-glossed selection: from each
/// token, the longest run whose key pieces ([`expression_piece`]: the dictionary
/// forms, and in French the determiners as written) are a key of the table
/// wins, and the next run starts past it, so a token belongs to at most one
/// match. Runs start at two tokens because every key holds a space — a single
/// lemma is a word, not an expression, and could never be one — and stop at the
/// language's window. A match reports the expression's name where the pack
/// carries one (French), else its key, and the reader's status is read on what
/// it reports (add-lingua-french-expression-keys D3).
fn match_expressions(
    tokens: &[PhraseToken],
    studied: StudiedLanguage,
    pack: &Pack,
    knowledge: &KnowledgeState,
) -> Vec<PhraseMatch> {
    if !pack.has_expressions() {
        return Vec::new();
    }
    let pieces: Vec<String> = tokens
        .iter()
        .map(|token| expression_piece(&token.surface, &token.lemma, studied))
        .collect();
    let window = expression_window(studied);
    let mut matches = Vec::new();
    let mut start = 0;
    while start < tokens.len() {
        let longest = window.min(tokens.len() - start);
        let hit = (2..=longest).rev().find_map(|len| {
            expression_for_run(&pieces[start..start + len], studied, pack)
                .map(|(key, gloss)| (len, key, gloss))
        });
        match hit {
            Some((len, key, gloss)) => {
                let key = match pack.expression_name(&key) {
                    Some(name) => name.to_owned(),
                    None => key,
                };
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
            let (lemma, parts) = resolve_lemmas(&token, studied, lexicon);
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

/// The separator between the senses of a pack gloss. The reducer guarantees
/// no sense holds it (`add-lingua-word-grammar`, design D4).
const SENSE_SEPARATOR: &str = "; ";

/// The senses of a gloss that share a part of speech.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SenseGroup {
    /// The part of speech of the group, with the features the word carries in
    /// it (a noun's gender); `None` — and omitted from the JSON — when the
    /// pack does not say, or says it in a tag this core cannot read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<Tag>,
    /// The group's senses, as the gloss writes them.
    pub text: String,
}

/// Another dictionary form a written form is also a reading of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OtherReading {
    /// That dictionary form.
    pub lemma: String,
    /// The form's readings as that dictionary form.
    pub readings: Vec<Tag>,
}

/// A word card's grammar: what the form is, what else it may be, the pieces
/// the pre-pass split it into, and the gloss laid out by part of speech.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WordGrammar {
    /// The dictionary form's pack gloss, one text, exactly as a card stores it.
    pub gloss: Option<String>,
    /// The same gloss, grouped by part of speech. One untagged group when the
    /// pack carries no runs for the word; empty when there is no gloss.
    pub senses: Vec<SenseGroup>,
    /// The readings of the form as the card's dictionary form.
    pub readings: Vec<Tag>,
    /// The other dictionary forms the form is also a reading of — none for a word
    /// the pre-pass split, whose pieces the split has already settled.
    pub others: Vec<OtherReading>,
    /// The pieces the studied language's pre-pass split the written word into
    /// (`don't` → `do`, `not`), as written; empty when it was not split.
    pub pieces: Vec<String>,
}

/// Answers a word card's grammar from the pack, for the word as written on the
/// page and the dictionary form the card is keyed by.
///
/// The written word goes through the page's tokeniser and pre-pass; the piece
/// whose dictionary form is `lemma` — or the first, when none is — is the one
/// whose readings are answered. The page analysis never calls this, so its
/// output, and `ANALYZER_VERSION`, stay where they are.
pub fn word_grammar(
    written: &str,
    lemma: &str,
    studied: StudiedLanguage,
    pack: &Pack,
) -> WordGrammar {
    let lexicon = pack.lexicon();
    let lemma = lemma.to_lowercase();
    let tokens = tokenize(written, studied, lexicon);
    let piece = tokens
        .iter()
        .find(|token| resolve_lemmas(token, studied, lexicon).0 == lemma)
        .or_else(|| tokens.first())
        .map(|token| token.text.replace('\u{2019}', "'").to_lowercase())
        .unwrap_or_else(|| written.trim().to_lowercase());
    let pieces = if tokens.len() > 1 {
        tokens.into_iter().map(|token| token.text).collect()
    } else {
        Vec::new()
    };
    let gloss = pack.gloss(&lemma).map(str::to_owned);
    let senses = gloss
        .as_deref()
        .map(|gloss| group_senses(gloss, &pack.sense_runs(&lemma)))
        .unwrap_or_default();
    // A word the pre-pass split is already settled: `does` in `doesn't` is the verb, and naming
    // the plural of `doe` beside it would be noise.
    let others = if pieces.is_empty() {
        pack.other_readings(&lemma, &piece)
            .into_iter()
            .map(|(lemma, readings)| OtherReading { lemma, readings })
            .collect()
    } else {
        Vec::new()
    };
    WordGrammar {
        readings: pack.readings(&lemma, &piece),
        others,
        pieces,
        senses,
        gloss,
    }
}

/// Lays a gloss out by its runs. Runs that do not account for exactly the
/// gloss's senses — which the builder refuses to write — give one untagged
/// group, as a pack without runs does: the reader still gets the whole gloss.
fn group_senses(gloss: &str, runs: &[(Option<&Tag>, usize)]) -> Vec<SenseGroup> {
    let senses: Vec<&str> = gloss.split(SENSE_SEPARATOR).collect();
    let covered: usize = runs.iter().map(|(_, count)| count).sum();
    if runs.is_empty() || covered != senses.len() || runs.iter().any(|(_, count)| *count == 0) {
        return vec![SenseGroup {
            tag: None,
            text: gloss.to_owned(),
        }];
    }
    let mut at = 0;
    runs.iter()
        .map(|(tag, count)| {
            let text = senses[at..at + count].join(SENSE_SEPARATOR);
            at += count;
            SenseGroup {
                tag: tag.cloned(),
                text,
            }
        })
        .collect()
}

/// The canonical JSON of a word's grammar — like [`analyse_page_json`], the
/// exact string both the native and WASM targets must produce.
pub fn word_grammar_json(
    written: &str,
    lemma: &str,
    studied: StudiedLanguage,
    pack: &Pack,
) -> String {
    serde_json::to_string(&word_grammar(written, lemma, studied, pack))
        .expect("WordGrammar serialises")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lexicon::{FstLexicon, build_lexicon_blobs};
    use crate::analysis::{ANALYZER_VERSION, FRENCH_ANALYZER_VERSION, SPANISH_ANALYZER_VERSION};
    use crate::knowledge::status::{KnownSource, Status};
    use crate::packs::format::write_container;
    use crate::packs::meta::PackMeta;
    use crate::packs::pack::section;

    #[test]
    fn a_capital_counts_as_a_name_only_in_mid_sentence() {
        // add-lingua-spanish-names D1: after a word, a comma or a semicolon.
        assert!(mid_sentence("dijo Augusto", 5));
        assert!(mid_sentence("y, Augusto", 3));
        assert!(mid_sentence("calló;\u{a0}Augusto", 9));
        // At the head of a block, a sentence, a quotation or a line of dialogue.
        assert!(!mid_sentence("Augusto", 0));
        assert!(!mid_sentence("Llegó. Augusto", 7));
        assert!(!mid_sentence("¿Augusto", 2));
        assert!(!mid_sentence("—Augusto", 3));
        assert!(!mid_sentence("«Augusto", 2));
        // An offset inside a character is no position at all.
        assert!(!mid_sentence("—Augusto", 1));
    }

    const EN: StudiedLanguage = StudiedLanguage::English;
    const ES: StudiedLanguage = StudiedLanguage::Spanish;
    const FR: StudiedLanguage = StudiedLanguage::French;

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
        build_pack_for(EN, forms, lemmas, ranks, glosses, expressions)
    }

    /// [`build_pack_with_expressions`] for any studied language, stamped with
    /// that language's analyser version.
    fn build_pack_for(
        studied: StudiedLanguage,
        forms: &[(&str, &str)],
        lemmas: &[&str],
        ranks: &[(&str, u32)],
        glosses: &[(&str, &str)],
        expressions: &[(&str, &str)],
    ) -> Pack {
        build_pack_with_lexical(studied, forms, lemmas, ranks, glosses, expressions, None)
    }

    /// [`build_pack_for`] plus, when `lexical` is given, a lexical table naming those
    /// dictionary words (add-lingua-pack-lexical-layer D1).
    fn build_pack_with_lexical(
        studied: StudiedLanguage,
        forms: &[(&str, &str)],
        lemmas: &[&str],
        ranks: &[(&str, u32)],
        glosses: &[(&str, &str)],
        expressions: &[(&str, &str)],
        lexical: Option<&[&str]>,
    ) -> Pack {
        build_pack_with_names(
            studied,
            forms,
            lemmas,
            ranks,
            glosses,
            expressions,
            &[],
            lexical,
        )
    }

    /// [`build_pack_with_lexical`] plus, when `names` holds any, the expression names
    /// section: each (key, name) pair filed under the key's id, as the builder files a
    /// French pack's (add-lingua-french-expression-keys D3).
    #[allow(clippy::too_many_arguments)]
    fn build_pack_with_names(
        studied: StudiedLanguage,
        forms: &[(&str, &str)],
        lemmas: &[&str],
        ranks: &[(&str, u32)],
        glosses: &[(&str, &str)],
        expressions: &[(&str, &str)],
        names: &[(&str, &str)],
        lexical: Option<&[&str]>,
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
        // A French pack is glossed in English: a pack is never glossed in the language it studies.
        let native = match studied {
            StudiedLanguage::French => "en",
            _ => "fr",
        };
        let meta = serde_json::to_vec(&PackMeta {
            studied: studied.tag().into(),
            native: native.into(),
            pack_version: "t".into(),
            analyzer_version: studied.analyzer_version().into(),
            levels_estimated: false,
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
        let mut keys: Vec<&str> = expressions.iter().map(|(key, _)| *key).collect();
        keys.sort_unstable();
        let mut named: Vec<(u32, &str)> = names
            .iter()
            .map(|(key, name)| {
                let id = keys.binary_search(key).expect("a key of the table");
                (id as u32, *name)
            })
            .collect();
        named.sort_unstable();
        let expr_names = build_gloss_zst(&named);
        if !names.is_empty() {
            sections.push((section::EXPR_NAMES_ZST, &expr_names));
        }
        let mut bits = vec![0u8; lex.lemma_count().div_ceil(8)];
        for lemma in lexical.unwrap_or_default() {
            let id = lex.id_of(lemma).expect("dictionary word listed") as usize;
            bits[id / 8] |= 1 << (id % 8);
        }
        if lexical.is_some() {
            sections.push((section::LEXICAL, &bits));
        }
        Pack::load(&write_container(&meta, &sections)).expect("load")
    }

    /// The classes of every token written `surface` on a Spanish page.
    fn spanish_classes(pack: &Pack, blocks: &[&str], surface: &str) -> Vec<TokenClass> {
        analyse_page(blocks, ES, pack, &KnowledgeState::new())
            .tokens
            .into_iter()
            .filter(|token| token.surface == surface)
            .map(|token| token.class)
            .collect()
    }

    #[test]
    fn spec_scenario_a_spanish_documents_names_are_the_packs_non_dictionary_words() {
        // add-lingua-pack-lexical-layer D2: the names rule reads dictionary words, not glosses.
        let blocks = [
            "Augusto miró a Eugenia y rezó a Dios en la casa.",
            "Todos los nobles de la corte miraban la escena con mucha atención.",
        ];
        let forms = [("miró", "mirar"), ("rezó", "rezar"), ("miraban", "mirar")];
        let lemmas = [
            "augusto",
            "eugenia",
            "dios",
            "mirar",
            "rezar",
            "a",
            "y",
            "en",
            "la",
            "casa",
            "todo",
            "el",
            "noble",
            "de",
            "corte",
            "escena",
            "con",
            "mucho",
            "atención",
        ];
        let glosses = [("dios", "Dieu"), ("casa", "Maison")];
        let pack = |lexical: Option<&[&str]>| {
            build_pack_with_lexical(ES, &forms, &lemmas, &[], &glosses, &[], lexical)
        };
        const NAME: TokenClass = TokenClass::ProperNounOutOfLexicon;
        const WORD: TokenClass = TokenClass::Unknown;

        // Without a table, the glossed lemmas are the dictionary words: today's rule.
        let today = pack(None);
        assert_eq!(spanish_classes(&today, &blocks, "Augusto"), [WORD]);
        assert_eq!(spanish_classes(&today, &blocks, "Eugenia"), [NAME]);
        assert_eq!(spanish_classes(&today, &blocks, "Dios"), [WORD]);

        // With one, the table decides: `eugenia`, a dictionary word without a gloss, stays a
        // word, and `dios`, glossed but not named, is set aside.
        let table = pack(Some(&["casa", "eugenia"]));
        assert_eq!(spanish_classes(&table, &blocks, "Eugenia"), [WORD]);
        assert_eq!(spanish_classes(&table, &blocks, "Dios"), [NAME]);
        // The block-initial `Augusto` is never capitalised in mid-sentence: no evidence.
        assert_eq!(spanish_classes(&table, &blocks, "Augusto"), [WORD]);
        // Glosses are untouched: `Dios` keeps none on the page only because names show none.
        assert_eq!(table.gloss("dios"), Some("Dieu"));
        // English's analysis has no names rule, table or not.
        let english = build_pack_with_lexical(
            EN,
            &[],
            &["augusto", "met", "the", "man"],
            &[],
            &[],
            &[],
            Some(&[]),
        );
        let page = analyse_page(
            &["The man met Augusto, and the man met Augusto."],
            EN,
            &english,
            &KnowledgeState::new(),
        );
        assert!(
            page.tokens
                .iter()
                .filter(|t| t.surface == "Augusto")
                .all(|t| t.class == WORD)
        );
    }

    #[test]
    fn spec_scenario_each_analysis_names_its_own_languages_version() {
        let blocks = [
            "Los equipos nunca entregan el viernes, y tú has trabajado toda la semana por la noche.",
        ];
        let pack = build_pack_for(
            ES,
            &[("has", "haber"), ("equipos", "equipo")],
            &[
                "haber", "equipo", "nunca", "el", "viernes", "y", "tú", "toda", "la", "semana",
                "por", "noche",
            ],
            &[],
            &[],
            &[],
        );
        let knowledge = KnowledgeState::new();
        let page = analyse_page(&blocks, ES, &pack, &knowledge);
        assert!(page.analysable);
        assert_eq!(page.analyzer_version, SPANISH_ANALYZER_VERSION);
        assert!(
            page.tokens
                .iter()
                .any(|t| t.surface == "has" && t.lemma == "haber")
        );
        assert!(page.tokens.iter().all(|t| t.lemma != "have"));
        let english = analyse_page(
            &["Teams don't ship code on Friday."],
            EN,
            &sample_pack(),
            &knowledge,
        );
        assert_eq!(english.analyzer_version, ANALYZER_VERSION);
    }

    #[test]
    fn spec_scenario_a_french_page_is_read_by_its_analysis() {
        // add-lingua-french-analysis: French sets its document's names aside (D4), flags its
        // closed classes (D3) and names its own version, no longer a `0.x` one (D5). Its pre-pass
        // splits an elision into pieces with their own spans and `au` into `à` + `le` sharing one
        // (add-lingua-french-tokenisation); the pieces are lemmatised as any French word is.
        let blocks = [
            "Hier soir, Augusto a longtemps regardé Eugenia, puis il a prié Dieu dans la maison.",
            "Tous les nobles de la cour regardaient la scène avec beaucoup d'attention ce soir.",
            "L'homme est allé au marché ce matin, puis il a longtemps regardé la maison du port.",
        ];
        let forms = [("regardé", "regarder"), ("regardaient", "regarder")];
        let lemmas = [
            "eugenia", "dieu", "regarder", "maison", "la", "de", "il", "le", "homme",
        ];
        // `eugenia` and `dieu` are lemmas but not dictionary words: capitalised in mid-sentence
        // and never written in lowercase, they are the document's names.
        let pack = build_pack_with_lexical(
            FR,
            &forms,
            &lemmas,
            &[],
            &[],
            &[],
            Some(&["maison", "regarder"]),
        );
        let page = analyse_page(&blocks, FR, &pack, &KnowledgeState::new());
        assert!(page.analysable);
        assert_eq!(page.analyzer_version, FRENCH_ANALYZER_VERSION);
        assert!(!page.analyzer_version.starts_with("0."));
        let class_of = |surface: &str| {
            page.tokens
                .iter()
                .find(|t| t.surface == surface)
                .map(|t| (t.lemma.as_str(), t.class))
        };
        assert_eq!(
            class_of("Eugenia"),
            Some(("eugenia", TokenClass::ProperNounOutOfLexicon))
        );
        assert_eq!(
            class_of("Dieu"),
            Some(("dieu", TokenClass::ProperNounOutOfLexicon))
        );
        // The rule that belongs to no language still holds: a capital outside the lexicon.
        assert_eq!(
            class_of("Augusto"),
            Some(("augusto", TokenClass::ProperNounOutOfLexicon))
        );
        assert_eq!(class_of("regardé"), Some(("regarder", TokenClass::Unknown)));
        // An unlisted lowercase plural reads as its singular (D2).
        assert_eq!(class_of("nobles"), Some(("noble", TokenClass::Unknown)));
        // The tokens a block has at a byte offset: (surface, lemma, end).
        let at = |block: usize, start: usize| -> Vec<(&str, &str, usize)> {
            page.tokens
                .iter()
                .filter(|t| t.block == block && t.start == start)
                .map(|t| (t.surface.as_str(), t.lemma.as_str(), t.end))
                .collect()
        };
        // `d'attention` is `de` + `attention`, each piece with its own span.
        let elided = blocks[1].find("d'attention").expect("d'attention");
        assert_eq!(at(1, elided), [("de", "de", elided + 2)]);
        assert_eq!(at(1, elided + 2), [("attention", "attention", elided + 11)]);
        assert!(page.tokens.iter().all(|t| t.surface != "d'attention"));
        // `L'homme` is `Le` [0, 2) + `homme` [2, 7); `Le`, at the head of its block, is no name.
        assert_eq!(at(2, 0), [("Le", "le", 2)]);
        assert_eq!(at(2, 2), [("homme", "homme", 7)]);
        assert_eq!(class_of("Le"), Some(("le", TokenClass::Unknown)));
        // `au` is `à` + `le`, sharing its span; `du` is whole.
        let au = blocks[2].find(" au ").expect("au") + 1;
        assert_eq!(at(2, au), [("à", "à", au + 2), ("le", "le", au + 2)]);
        let du = blocks[2].find(" du ").expect("du") + 1;
        assert_eq!(at(2, du), [("du", "du", du + 2)]);

        // The closed classes are flagged, « pas » among them (M21); `maison` is not.
        let phrase = gloss_phrase("ne pas le de la maison", FR, &pack, &KnowledgeState::new());
        let flags: Vec<(&str, bool)> = phrase
            .tokens
            .iter()
            .map(|t| (t.lemma.as_str(), t.function_word))
            .collect();
        assert_eq!(
            flags,
            [
                ("ne", true),
                ("pas", true),
                ("le", true),
                ("de", true),
                ("la", true),
                ("maison", false)
            ]
        );
        assert_eq!(pack.pair().key(), "fr-en");
    }

    /// The classes of every token written `surface` on a French page.
    fn french_classes(pack: &Pack, blocks: &[&str], surface: &str) -> Vec<TokenClass> {
        french_classes_for(pack, blocks, surface, &KnowledgeState::new())
    }

    /// [`french_classes`] for a reader with a history.
    fn french_classes_for(
        pack: &Pack,
        blocks: &[&str],
        surface: &str,
        knowledge: &KnowledgeState,
    ) -> Vec<TokenClass> {
        let page = analyse_page(blocks, FR, pack, knowledge);
        assert!(page.analysable, "{blocks:?}");
        page.tokens
            .into_iter()
            .filter(|token| token.surface == surface)
            .map(|token| token.class)
            .collect()
    }

    /// The names a document's French tokens give under `studied`'s rule: the same tokens, so
    /// that the two rules are compared on what each reads.
    fn names_of(blocks: &[&str], studied: StudiedLanguage, pack: &Pack) -> Vec<String> {
        let DocumentAnalysis::Analysed(tokens) = analyse_document(blocks, FR, pack.lexicon())
        else {
            panic!("{blocks:?} is not analysable as French");
        };
        let mut names: Vec<String> = document_names(&tokens, blocks, studied, pack)
            .into_iter()
            .collect();
        names.sort_unstable();
        names
    }

    const NAME: TokenClass = TokenClass::ProperNounOutOfLexicon;
    const WORD: TokenClass = TokenClass::Unknown;

    #[test]
    fn a_capital_after_an_elided_piece_is_the_word_s_own() {
        // add-lingua-french-analysis D4: an apostrophe, straight or typographic, after a letter.
        assert!(after_elided_piece("l'Europe", 2));
        assert!(after_elided_piece("d\u{2019}Espagne", 4));
        assert!(after_elided_piece("Il dit qu'Augusto", 10));
        // An apostrophe after no letter is a quotation mark, and a space ends the piece.
        assert!(!after_elided_piece("'Europe", 1));
        assert!(!after_elided_piece("dit ' Europe", 6));
        assert!(!after_elided_piece("l' Europe", 3));
        assert!(!after_elided_piece("Europe", 0));
        // An offset inside a character is no position at all.
        assert!(!after_elided_piece("d\u{2019}Espagne", 2));
    }

    #[test]
    fn spec_scenario_a_french_city() {
        let blocks = [
            "Nous partons de Paris demain matin avec toute la famille et le chien.",
            "Paris est loin de notre petit village de montagne, mais le voyage est beau.",
        ];
        let pack = build_pack_for(FR, &[], &["paris", "de", "la", "le"], &[], &[], &[]);
        // `paris` is a lemma without a gloss: no dictionary word of a pack without a lexical
        // section. Both occurrences go, the one at a sentence's head with the other.
        assert_eq!(french_classes(&pack, &blocks, "Paris"), [NAME, NAME]);
    }

    #[test]
    fn spec_scenario_after_an_elided_piece() {
        // The only capitalised `Aube` follows `l'` at the head of its block: the elision alone is
        // the evidence.
        let blocks = [
            "L'Aube rejoint la Seine dans la plaine, après un long voyage vers le nord.",
            "Les bateaux descendent la rivière chaque matin avec leurs marchandises.",
        ];
        let pack = build_pack_for(FR, &[], &["aube", "le", "la"], &[], &[], &[]);
        assert_eq!(french_classes(&pack, &blocks, "Aube"), [NAME]);
        // Spanish's rule, on the same tokens, finds no evidence there.
        assert!(names_of(&blocks, FR, &pack).contains(&"aube".to_owned()));
        assert!(!names_of(&blocks, ES, &pack).contains(&"aube".to_owned()));
        // The typographic apostrophe is evidence too.
        let typographic = [
            "Il rentre d\u{2019}Aube avec ses amis, après un long voyage vers le nord.",
            "Les bateaux descendent la rivière chaque matin avec leurs marchandises.",
        ];
        assert_eq!(french_classes(&pack, &typographic, "Aube"), [NAME]);
    }

    #[test]
    fn spec_scenario_a_hyphenated_name() {
        let blocks = [
            "Les gendarmes ont retrouvé Jean-Pierre à Saint-Étienne hier soir, près de la gare.",
            "Il était parti depuis trois jours avec son vélo et un sac de pommes.",
        ];
        let pack = build_pack_for(
            FR,
            &[],
            &["pierre", "saint", "de", "la", "le"],
            &[],
            &[("pierre", "stone"), ("saint", "saint")],
            &[],
        );
        // `pierre` and `saint` resolve, so the compound rule keeps both runs; the names rule reads
        // each run as one form, which the pack does not list.
        assert_eq!(french_classes(&pack, &blocks, "Jean-Pierre"), [NAME]);
        assert_eq!(french_classes(&pack, &blocks, "Saint-Étienne"), [NAME]);
        assert_eq!(
            names_of(&blocks, FR, &pack),
            ["jean-pierre", "saint-étienne"]
        );
        // Spanish's rule judges a run by its parts: on the same tokens, no run is a name.
        assert!(names_of(&blocks, ES, &pack).is_empty());
        // A Spanish page keeps them words, judged by their parts.
        let spanish = build_pack_for(
            ES,
            &[],
            &["pierre", "saint", "de", "la", "el"],
            &[],
            &[("pierre", "pierre"), ("saint", "saint")],
            &[],
        );
        let page = analyse_page(
            &[
                "Los gendarmes encontraron a Jean-Pierre en Saint-Étienne ayer por la noche, cerca de la estación.",
            ],
            ES,
            &spanish,
            &KnowledgeState::new(),
        );
        assert!(page.analysable);
        for run in ["Jean-Pierre", "Saint-Étienne"] {
            let classes: Vec<TokenClass> = page
                .tokens
                .iter()
                .filter(|t| t.surface == run)
                .map(|t| t.class)
                .collect();
            assert_eq!(classes, [WORD], "{run}");
        }
    }

    #[test]
    fn spec_scenario_a_dictionary_word_stays_a_word() {
        // `Orange` is glossed in a pack without a lexical section, so it is a dictionary word; the
        // tables read `vienne` as the dictionary word `venir` (M8).
        let blocks = [
            "Le train roule entre Orange et Vienne pendant toute la matinée.",
            "Les voyageurs regardent les champs de lavande par la fenêtre du wagon.",
        ];
        let pack = build_pack_for(
            FR,
            &[("vienne", "venir")],
            &["orange", "venir", "le", "la"],
            &[],
            &[("orange", "orange"), ("venir", "to come")],
            &[],
        );
        assert_eq!(french_classes(&pack, &blocks, "Orange"), [WORD]);
        assert_eq!(french_classes(&pack, &blocks, "Vienne"), [WORD]);
        let page = analyse_page(&blocks, FR, &pack, &KnowledgeState::new());
        let vienne = page
            .tokens
            .iter()
            .find(|t| t.surface == "Vienne")
            .expect("Vienne");
        assert_eq!(
            (vienne.lemma.as_str(), vienne.gloss.as_deref()),
            ("venir", Some("to come"))
        );
    }

    #[test]
    fn spec_scenario_capitals_at_the_head_of_sentences_only() {
        // `Mme` is written only at the head of its blocks, and after `M.`'s full stop: no evidence.
        let blocks = [
            "Mme Durand ouvre la boutique tous les matins à huit heures.",
            "Mme Durand vend du pain, des gâteaux et des croissants au beurre.",
        ];
        let pack = build_pack_for(FR, &[], &["mme", "le", "la"], &[], &[], &[]);
        assert_eq!(french_classes(&pack, &blocks, "Mme"), [WORD, WORD]);
    }

    #[test]
    fn spec_scenario_the_same_form_as_a_word() {
        // « le Lot » in mid-sentence, « un lot » elsewhere: the form is written in lowercase.
        let blocks = [
            "La rivière du Lot traverse le village avant de rejoindre la Garonne.",
            "Il a acheté un lot de livres anciens au marché de la place.",
        ];
        let pack = build_pack_for(FR, &[], &["lot", "le", "la", "garonne"], &[], &[], &[]);
        assert_eq!(french_classes(&pack, &blocks, "Lot"), [WORD]);
        // `Garonne`, never in lowercase, goes.
        assert_eq!(french_classes(&pack, &blocks, "Garonne"), [NAME]);
    }

    #[test]
    fn a_name_the_reader_marked_keeps_its_status() {
        let blocks = [
            "Nous partons de Paris demain matin avec toute la famille et le chien.",
            "Les gendarmes ont retrouvé Jean-Pierre près de la gare hier soir.",
        ];
        let pack = build_pack_for(
            FR,
            &[],
            &["paris", "pierre", "de", "la", "le"],
            &[],
            &[],
            &[],
        );
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(FR, "paris", Status::Learning);
        knowledge.set_status(FR, "jean-pierre", Status::Known(KnownSource::Manual));
        assert_eq!(
            french_classes_for(&pack, &blocks, "Paris", &knowledge),
            [TokenClass::Learning]
        );
        assert_eq!(
            french_classes_for(&pack, &blocks, "Jean-Pierre", &knowledge),
            [TokenClass::Known]
        );
        // Without the marks, both are names.
        assert_eq!(french_classes(&pack, &blocks, "Paris"), [NAME]);
        assert_eq!(french_classes(&pack, &blocks, "Jean-Pierre"), [NAME]);
    }

    #[test]
    fn spec_scenario_each_languages_closed_classes_are_its_own() {
        // add-lingua-spanish-analysis: Spanish's tables flag `de` and `la`, never `the`.
        let pack = build_pack_for(ES, &[], &["de", "la", "casa", "the"], &[], &[], &[]);
        let spanish = gloss_phrase("de la casa the", ES, &pack, &KnowledgeState::new());
        let flagged: Vec<(&str, bool)> = spanish
            .tokens
            .iter()
            .map(|t| (t.surface.as_str(), t.function_word))
            .collect();
        assert_eq!(
            flagged,
            [("de", true), ("la", true), ("casa", false), ("the", false)]
        );
        let english = gloss_phrase("on the code", EN, &sample_pack(), &KnowledgeState::new());
        assert!(english.tokens.iter().any(|t| t.function_word));
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

    // --- French expression keys (add-lingua-french-expression-keys) ---

    #[test]
    fn the_words_a_french_key_writes_are_sorted_and_twenty_four() {
        assert_eq!(FRENCH_KEY_WRITTEN.len(), 24);
        assert!(FRENCH_KEY_WRITTEN.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn a_french_key_writes_its_determiners_and_lemmatises_the_rest() {
        // D2: the determiners as the pre-pass gives them, lowercased; every other token its
        // dictionary form.
        assert_eq!(expression_piece("la", "le", FR), "la");
        assert_eq!(expression_piece("Les", "le", FR), "les");
        assert_eq!(expression_piece("cet", "ce", FR), "cet");
        // `l'` is read `le` by the pre-pass, which is what the key writes.
        let elided = crate::analysis::tokenize::tokenize("l'homme", FR, &french_lexicon());
        assert_eq!(elided[0].text, "le");
        assert_eq!(expression_piece(&elided[0].text, "le", FR), "le");
        assert_eq!(expression_piece("bonne", "bon", FR), "bon");
        assert_eq!(expression_piece("a", "avoir", FR), "avoir");
        // English and Spanish keys are their lemmas, a determiner too.
        assert_eq!(expression_piece("The", "the", EN), "the");
        assert_eq!(expression_piece("la", "el", ES), "el");
    }

    /// A French lexicon for the keys and matches below.
    fn french_lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) = build_lexicon_blobs(FRENCH_FORMS, FRENCH_LEMMAS).expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    const FRENCH_FORMS: &[(&str, &str)] = &[
        ("a", "avoir"),
        ("avait", "avoir"),
        ("bonne", "bon"),
        ("la", "le"),
        ("les", "le"),
        ("mains", "main"),
    ];

    const FRENCH_LEMMAS: &[&str] = &[
        "à", "le", "revoir", "coup", "de", "œil", "abord", "il", "y", "avoir", "haut", "main",
        "bon", "heure", "compte", "en", "cause", "des", "du", "fur", "et", "mesure", "marché",
        "un", "maison", "enfant", "pain",
    ];

    #[test]
    fn a_french_expression_is_keyed_as_french_is_read() {
        let lexicon = french_lexicon();
        let key = |headword: &str| french_expression_key(headword, &lexicon);
        assert_eq!(key("au revoir").as_deref(), Some("à le revoir"));
        assert_eq!(key("coup d'œil").as_deref(), Some("coup de œil"));
        assert_eq!(key("coup d\u{2019}œil").as_deref(), Some("coup de œil"));
        assert_eq!(key("d'abord").as_deref(), Some("de abord"));
        assert_eq!(key("il y a").as_deref(), Some("il y avoir"));
        assert_eq!(key("de bonne heure").as_deref(), Some("de bon heure"));
        // The determiners are written: `à la` is not `au`, nor `haut les mains` `haut la main`.
        assert_eq!(key("à la").as_deref(), Some("à la"));
        assert_eq!(key("haut la main").as_deref(), Some("haut la main"));
        assert_eq!(key("haut les mains").as_deref(), Some("haut les main"));
        // A name is read as its headword is, so review finds the key from it.
        assert_eq!(key("Au revoir").as_deref(), Some("à le revoir"));
    }

    #[test]
    fn a_french_headword_outside_the_rules_has_no_key() {
        let lexicon = french_lexicon();
        let key = |headword: &str| french_expression_key(headword, &lexicon);
        // Seven tokens are French's window; eight are beyond it (D4).
        assert_eq!(
            key(&["coup"; 7].join(" ")).as_deref(),
            Some(["coup"; 7].join(" ").as_str())
        );
        assert_eq!(key(&["coup"; 8].join(" ")), None);
        // `au fur et à mesure`: five words, six tokens, within it.
        assert_eq!(
            key("au fur et à mesure").as_deref(),
            Some("à le fur et à mesure")
        );
        // A word the analysis drops (D6): the lexicon lists no `t`.
        assert_eq!(key("compte en t"), None);
        // One token is a word, not an expression; a dictionary form the lexicon does not hold
        // could never be met.
        assert_eq!(key("revoir"), None);
        assert_eq!(key("coup de foudre"), None);
        assert_eq!(key(""), None);
    }

    /// The fr-en pack the matching scenarios read: the keys French's reading makes, each named
    /// by its headword where it differs.
    fn french_expression_pack() -> Pack {
        build_pack_with_names(
            FR,
            FRENCH_FORMS,
            FRENCH_LEMMAS,
            &[],
            &[],
            &[
                ("à le revoir", "goodbye"),
                ("coup de œil", "glance"),
                ("de abord", "first, at first"),
                ("il y avoir", "there is, there are; ago"),
                ("à la", "in the style of"),
                ("à cause de", "because of"),
                ("à le fur et à mesure", "as one goes along"),
                ("de enfant", "of a child"),
            ],
            &[
                ("à le revoir", "au revoir"),
                ("coup de œil", "coup d'œil"),
                ("de abord", "d'abord"),
                ("il y avoir", "il y a"),
                ("à le fur et à mesure", "au fur et à mesure"),
            ],
            None,
        )
    }

    fn french_spans(text: &str) -> Vec<(usize, usize, String)> {
        gloss_phrase(text, FR, &french_expression_pack(), &KnowledgeState::new())
            .expressions
            .into_iter()
            .map(|m| (m.start, m.end, m.key))
            .collect()
    }

    fn span(start: usize, end: usize, key: &str) -> (usize, usize, String) {
        (start, end, key.to_owned())
    }

    #[test]
    fn spec_scenario_a_contracted_article() {
        let pack = french_expression_pack();
        let phrase = gloss_phrase("Au revoir", FR, &pack, &KnowledgeState::new());
        let surfaces: Vec<&str> = phrase.tokens.iter().map(|t| t.surface.as_str()).collect();
        assert_eq!(surfaces, ["À", "le", "revoir"]);
        assert_eq!(
            phrase.expressions,
            [PhraseMatch {
                start: 0,
                end: 3,
                key: "au revoir".into(),
                class: TokenClass::Unknown,
                gloss: "goodbye".into(),
            }]
        );
    }

    #[test]
    fn spec_scenario_an_elided_word() {
        assert_eq!(
            french_spans("un coup d\u{2019}œil"),
            [span(1, 4, "coup d'œil")]
        );
    }

    #[test]
    fn spec_scenario_a_word_the_pre_pass_splits() {
        assert_eq!(french_spans("D\u{2019}abord"), [span(0, 2, "d'abord")]);
    }

    #[test]
    fn spec_scenario_another_tense_of_the_expression() {
        assert_eq!(french_spans("il y avait"), [span(0, 3, "il y a")]);
        assert_eq!(french_spans("il y a"), [span(0, 3, "il y a")]);
    }

    #[test]
    fn spec_scenario_a_contracted_article_is_not_the_feminine_one() {
        assert_eq!(french_spans("au marché"), []);
        // The feminine article as written is.
        assert_eq!(french_spans("à la maison"), [span(0, 2, "à la")]);
    }

    #[test]
    fn spec_scenario_an_expression_ending_on_de_before_a_contracted_article() {
        let pack = french_expression_pack();
        let phrase = gloss_phrase("à cause des", FR, &pack, &KnowledgeState::new());
        assert_eq!(phrase.expressions.len(), 1);
        assert_eq!(
            (phrase.expressions[0].start, phrase.expressions[0].end),
            (0, 3),
            "`des` included"
        );
        assert_eq!(phrase.expressions[0].key, "à cause de");
        assert_eq!(phrase.expressions[0].gloss, "because of");
        // The token keeps its own lemma: the retry is the phrase gloss's.
        assert_eq!(phrase.tokens[2].lemma, "des");
        // Only the run's last token is read as `de`.
        assert_eq!(french_spans("des enfant"), []);
        assert_eq!(french_spans("de enfant"), [span(0, 2, "de enfant")]);
    }

    #[test]
    fn spec_scenario_an_expression_of_six_tokens() {
        let pack = french_expression_pack();
        let phrase = gloss_phrase("au fur et à mesure", FR, &pack, &KnowledgeState::new());
        let lemmas: Vec<&str> = phrase.tokens.iter().map(|t| t.lemma.as_str()).collect();
        assert_eq!(lemmas, ["à", "le", "fur", "et", "à", "mesure"]);
        assert_eq!(
            french_spans("au fur et à mesure"),
            [span(0, 6, "au fur et à mesure")]
        );
    }

    #[test]
    fn spec_scenario_the_status_follows_the_name() {
        let pack = french_expression_pack();
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(FR, "il y a", Status::Known(KnownSource::Manual));
        let phrase = gloss_phrase("il y avait", FR, &pack, &knowledge);
        assert_eq!(phrase.expressions[0].key, "il y a");
        assert_eq!(phrase.expressions[0].class, TokenClass::Known);
        // A status set on the key is not the name's.
        let mut on_key = KnowledgeState::new();
        on_key.set_status(FR, "il y avoir", Status::Known(KnownSource::Manual));
        let phrase = gloss_phrase("il y avait", FR, &pack, &on_key);
        assert_eq!(phrase.expressions[0].class, TokenClass::Unknown);
    }

    #[test]
    fn a_french_selection_s_tokens_do_not_move_with_the_table() {
        // The tokens are what they are without the table; the matches sit beside them.
        let bare = build_pack_for(FR, FRENCH_FORMS, FRENCH_LEMMAS, &[], &[], &[]);
        let knowledge = KnowledgeState::new();
        for text in [
            "Au revoir",
            "à cause des",
            "au fur et à mesure",
            "il y avait",
        ] {
            let with_table = gloss_phrase(text, FR, &french_expression_pack(), &knowledge);
            let without = gloss_phrase(text, FR, &bare, &knowledge);
            assert_eq!(with_table.tokens, without.tokens, "{text}");
            assert!(without.expressions.is_empty());
            assert!(!with_table.expressions.is_empty(), "{text}");
        }
    }

    #[test]
    fn an_english_or_spanish_run_ending_on_du_or_des_is_not_retried() {
        // D5 is French's: an English or Spanish key ending on `de` is never met through `des`.
        for studied in [EN, ES] {
            let pack = build_pack_for(
                studied,
                &[],
                &["por", "de", "des"],
                &[],
                &[],
                &[("por de", "x")],
            );
            let phrase = gloss_phrase("por des", studied, &pack, &KnowledgeState::new());
            assert!(phrase.expressions.is_empty(), "{studied:?}");
        }
    }

    // — word grammar (`add-lingua-word-grammar`) —

    /// A reading as a test states it: the form, its dictionary form, its tag.
    type Reading<'a> = (&'a str, &'a str, &'a str);
    /// Another dictionary form filed under the one the analysis reads the
    /// form as: (form, filed under, other dictionary form).
    type Also<'a> = (&'a str, &'a str, &'a str);
    /// A gloss's runs: (dictionary form, [(tag, senses)]).
    type Runs<'a> = (&'a str, &'a [(&'a str, u8)]);

    /// [`build_pack`] plus grammar tables, laid out as the builder lays them:
    /// a sorted tag pool, and paradigms and runs keyed by lemma id, each
    /// zstd-compressed.
    fn build_pack_with_grammar(
        forms: &[(&str, &str)],
        lemmas: &[&str],
        glosses: &[(&str, &str)],
        readings: &[Reading],
        also: &[Also],
        runs: &[Runs],
    ) -> Pack {
        build_pack_with_grammar_for(EN, forms, lemmas, &[], glosses, readings, also, runs)
    }

    /// [`build_pack_with_grammar`] for any studied language, with ranks.
    #[allow(clippy::too_many_arguments)]
    fn build_pack_with_grammar_for(
        studied: StudiedLanguage,
        forms: &[(&str, &str)],
        lemmas: &[&str],
        ranks: &[(&str, u32)],
        glosses: &[(&str, &str)],
        readings: &[Reading],
        also: &[Also],
        runs: &[Runs],
    ) -> Pack {
        use crate::packs::grammar::{
            FormEdit, ParadigmEntry, SenseRun, encode_indexed, encode_paradigm, encode_runs,
            encode_tag_pool,
        };
        use std::collections::BTreeMap;
        let (form_bytes, pool) = build_lexicon_blobs(forms, lemmas).expect("lexicon");
        let lex = FstLexicon::from_slices(form_bytes.clone(), &pool).unwrap();
        let id = |lemma: &str| lex.id_of(lemma).expect("listed") as u32;
        let mut tags: Vec<String> = readings
            .iter()
            .map(|(_, _, tag)| (*tag).to_owned())
            .chain(
                runs.iter()
                    .flat_map(|(_, r)| r.iter().map(|(t, _)| (*t).to_owned())),
            )
            .collect();
        tags.sort_unstable();
        tags.dedup();
        let tag_id = |tag: &str| tags.binary_search(&tag.to_owned()).unwrap() as u16;
        let mut paradigms: BTreeMap<u32, Vec<ParadigmEntry>> = BTreeMap::new();
        for (form, lemma, tag) in readings {
            paradigms
                .entry(id(lemma))
                .or_default()
                .push(ParadigmEntry::Reading {
                    form: FormEdit::between(lemma, form).unwrap(),
                    tag: tag_id(tag),
                });
        }
        for (form, under, other) in also {
            paradigms
                .entry(id(under))
                .or_default()
                .push(ParadigmEntry::Also {
                    form: FormEdit::between(under, form).unwrap(),
                    other: id(other),
                });
        }
        let paradigms: Vec<(u32, Vec<u8>)> = paradigms
            .into_iter()
            .map(|(id, entries)| (id, encode_paradigm(&entries)))
            .collect();
        let mut run_entries: Vec<(u32, Vec<u8>)> = runs
            .iter()
            .map(|(lemma, r)| {
                let r: Vec<SenseRun> = r
                    .iter()
                    .map(|(tag, count)| SenseRun {
                        tag: tag_id(tag),
                        count: *count,
                    })
                    .collect();
                (id(lemma), encode_runs(&r))
            })
            .collect();
        run_entries.sort_unstable_by_key(|(id, _)| *id);
        let gloss_entries: Vec<(u32, &str)> = glosses.iter().map(|(l, g)| (id(l), *g)).collect();
        let mut freq = vec![0u32; lex.lemma_count()];
        for (lemma, rank) in ranks {
            freq[id(lemma) as usize] = *rank;
        }
        let freq_bytes: Vec<u8> = freq.iter().flat_map(|r| r.to_le_bytes()).collect();
        let meta = serde_json::to_vec(&PackMeta {
            studied: studied.tag().into(),
            native: "fr".into(),
            pack_version: "t".into(),
            analyzer_version: studied.analyzer_version().into(),
            levels_estimated: false,
            licences: vec![],
        })
        .unwrap();
        let gloss = build_gloss_zst(&gloss_entries);
        let tag_pool = encode_tag_pool(&tags);
        let paradigm_zst = zstd::encode_all(encode_indexed(&paradigms).as_slice(), 19).unwrap();
        let runs_zst = zstd::encode_all(encode_indexed(&run_entries).as_slice(), 19).unwrap();
        let sections: Vec<(&str, &[u8])> = vec![
            (section::FORMS, &form_bytes),
            (section::LEMMAS, pool.as_bytes()),
            (section::FREQ, &freq_bytes),
            (section::GLOSS_ZST, &gloss),
            (section::TAGS, &tag_pool),
            (section::PARADIGMS_ZST, &paradigm_zst),
            (section::SENSES_ZST, &runs_zst),
        ];
        Pack::load(&write_container(&meta, &sections)).expect("load")
    }

    const PAST: &str = "VERB|Mood=Ind|Tense=Past|VerbForm=Fin";
    const PARTICIPLE: &str = "VERB|Tense=Past|VerbForm=Part";
    const THIRD_SINGULAR: &str = "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin";
    const PLURAL: &str = "NOUN|Number=Plur";

    fn ud(tags: &[Tag]) -> Vec<String> {
        tags.iter().map(Tag::to_ud).collect()
    }

    /// One pack for the `lingua-analysis` scenarios.
    fn grammar_pack() -> Pack {
        build_pack_with_grammar(
            &[
                ("went", "go"),
                ("walked", "walk"),
                ("leaves", "leave"),
                ("does", "do"),
            ],
            &[
                "go", "walk", "leave", "leaf", "do", "doe", "not", "can", "put",
            ],
            &[
                ("go", "Aller"),
                ("can", "Boîte de conserve; Pouvoir, savoir; Mettre en boîte"),
                ("put", "Mettre"),
            ],
            &[
                ("went", "go", PAST),
                ("walked", "walk", PAST),
                ("walked", "walk", PARTICIPLE),
                ("leaves", "leave", THIRD_SINGULAR),
                ("leaves", "leaf", PLURAL),
                ("does", "do", THIRD_SINGULAR),
                ("does", "doe", PLURAL),
                ("put", "put", PAST),
                ("put", "put", PARTICIPLE),
            ],
            &[("leaves", "leave", "leaf"), ("does", "do", "doe")],
            &[
                ("can", &[("NOUN", 1), ("VERB", 2)]),
                ("put", &[("VERB", 1)]),
            ],
        )
    }

    #[test]
    fn spec_scenario_an_irregular_form() {
        let grammar = word_grammar("went", "go", EN, &grammar_pack());
        assert_eq!(ud(&grammar.readings), [PAST]);
        assert!(grammar.pieces.is_empty());
        assert!(grammar.others.is_empty());
    }

    #[test]
    fn spec_scenario_a_form_with_two_readings() {
        let grammar = word_grammar("walked", "walk", EN, &grammar_pack());
        assert_eq!(ud(&grammar.readings), [PAST, PARTICIPLE]);
    }

    #[test]
    fn spec_scenario_a_contraction() {
        let grammar = word_grammar("doesn't", "do", EN, &grammar_pack());
        assert_eq!(grammar.pieces, ["does", "not"]);
        assert_eq!(ud(&grammar.readings), [THIRD_SINGULAR]);
        assert!(
            grammar.others.is_empty(),
            "the split settled `does`: no plural of `doe` beside it"
        );
        // Written alone, `does` may well be that plural.
        let alone = word_grammar("does", "do", EN, &grammar_pack());
        assert_eq!(alone.others.len(), 1);
        assert_eq!(alone.others[0].lemma, "doe");
    }

    #[test]
    fn a_contraction_keeps_the_case_it_was_written_in() {
        let grammar = word_grammar("Doesn\u{2019}t", "do", EN, &grammar_pack());
        assert_eq!(grammar.pieces, ["Does", "not"]);
        assert_eq!(ud(&grammar.readings), [THIRD_SINGULAR]);
    }

    #[test]
    fn spec_scenario_a_form_of_another_dictionary_form_too() {
        let grammar = word_grammar("leaves", "leave", EN, &grammar_pack());
        assert_eq!(ud(&grammar.readings), [THIRD_SINGULAR]);
        assert_eq!(grammar.others.len(), 1);
        assert_eq!(grammar.others[0].lemma, "leaf");
        assert_eq!(ud(&grammar.others[0].readings), [PLURAL]);
    }

    #[test]
    fn spec_scenario_senses_grouped_by_part_of_speech() {
        let grammar = word_grammar("can", "can", EN, &grammar_pack());
        let groups: Vec<(Option<String>, &str)> = grammar
            .senses
            .iter()
            .map(|g| (g.tag.as_ref().map(Tag::to_ud), g.text.as_str()))
            .collect();
        assert_eq!(
            groups,
            [
                (Some("NOUN".to_owned()), "Boîte de conserve"),
                (Some("VERB".to_owned()), "Pouvoir, savoir; Mettre en boîte"),
            ]
        );
        assert_eq!(
            grammar.gloss.as_deref(),
            Some("Boîte de conserve; Pouvoir, savoir; Mettre en boîte"),
            "the flat gloss a card stores is untouched"
        );
    }

    #[test]
    fn a_form_spelled_like_its_dictionary_form_carries_its_other_readings() {
        let grammar = word_grammar("Put", "put", EN, &grammar_pack());
        assert_eq!(ud(&grammar.readings), [PAST, PARTICIPLE]);
    }

    #[test]
    fn spec_scenario_a_pack_without_grammar_tables() {
        let pack = build_pack(&[("went", "go")], &["go"], &[], &[("go", "Aller")]);
        assert!(!pack.has_grammar());
        let grammar = word_grammar("went", "go", EN, &pack);
        assert!(grammar.readings.is_empty());
        assert!(grammar.others.is_empty());
        assert_eq!(
            grammar.senses,
            [SenseGroup {
                tag: None,
                text: "Aller".into()
            }]
        );
        // The pieces come from the pre-pass, not from the pack.
        assert_eq!(word_grammar("don't", "do", EN, &pack).pieces, ["do", "not"]);
    }

    #[test]
    fn a_word_with_no_gloss_has_no_senses() {
        let grammar = word_grammar("walked", "walk", EN, &grammar_pack());
        assert_eq!(grammar.gloss, None);
        assert!(grammar.senses.is_empty());
    }

    #[test]
    fn runs_that_disagree_with_the_gloss_give_one_untagged_group() {
        let pack = build_pack_with_grammar(
            &[],
            &["run"],
            &[("run", "Course; Courir")],
            &[],
            &[],
            &[("run", &[("NOUN", 1)])],
        );
        let grammar = word_grammar("run", "run", EN, &pack);
        assert_eq!(
            grammar.senses,
            [SenseGroup {
                tag: None,
                text: "Course; Courir".into()
            }]
        );
    }

    #[test]
    fn a_word_that_is_not_a_word_answers_nothing_but_its_gloss() {
        let grammar = word_grammar("42", "go", EN, &grammar_pack());
        assert!(grammar.readings.is_empty());
        assert!(grammar.pieces.is_empty());
        assert_eq!(grammar.gloss.as_deref(), Some("Aller"));
    }

    #[test]
    fn spec_scenario_the_page_analysis_does_not_move() {
        let forms = [("went", "go"), ("teams", "team")];
        let lemmas = ["go", "team", "the", "home", "early", "and", "code"];
        let glosses = [("go", "Aller"), ("team", "Équipe")];
        let with = build_pack_with_grammar(
            &forms,
            &lemmas,
            &glosses,
            &[("went", "go", PAST), ("teams", "team", PLURAL)],
            &[],
            &[("go", &[("VERB", 1)])],
        );
        let without = build_pack_with_grammar(&forms, &lemmas, &glosses, &[], &[], &[]);
        let blocks = ["The teams went home early and the teams code the code at home."];
        let knowledge = KnowledgeState::new();
        let a = analyse_page_json(&blocks, EN, &with, &knowledge);
        assert_eq!(a, analyse_page_json(&blocks, EN, &without, &knowledge));
        assert!(a.contains("\"analyzer_version\":\"1.1.0\""));
    }

    #[test]
    fn the_json_names_parts_of_speech_and_features_never_codes_of_its_own() {
        let json = word_grammar_json("leaves", "leave", EN, &grammar_pack());
        assert_eq!(
            json,
            r#"{"gloss":null,"senses":[],"readings":[{"pos":"VERB","features":{"Mood":"Ind","Number":"Sing","Person":"3","Tense":"Pres","VerbForm":"Fin"}}],"others":[{"lemma":"leaf","readings":[{"pos":"NOUN","features":{"Number":"Plur"}}]}],"pieces":[]}"#
        );
    }

    #[test]
    fn an_unknown_feature_is_skipped_and_an_unreadable_tag_drops_its_reading() {
        let pack = build_pack_with_grammar(
            &[("went", "go")],
            &["go", "gone"],
            &[("go", "Aller; Tour")],
            &[
                ("went", "go", "VERB|Polite=Form|Tense=Past"),
                ("went", "gone", "WORD"),
            ],
            &[("went", "go", "gone")],
            &[("go", &[("SPEECH", 1), ("NOUN", 1)])],
        );
        let grammar = word_grammar("went", "go", EN, &pack);
        assert_eq!(ud(&grammar.readings), ["VERB|Tense=Past"]);
        assert!(
            grammar.others.is_empty(),
            "a dictionary form with no reading this core can name is left out"
        );
        assert_eq!(grammar.senses[0].tag, None);
        assert_eq!(
            grammar.senses[1].tag.as_ref().map(Tag::to_ud),
            Some("NOUN".into())
        );
    }

    // — a dictionary form is read as itself (fix-lingua-lemma-lookup) —

    /// What a card answers for a string that is no dictionary form of the pack.
    fn nothing() -> WordGrammar {
        WordGrammar {
            gloss: None,
            senses: Vec::new(),
            readings: Vec::new(),
            others: Vec::new(),
            pieces: Vec::new(),
        }
    }

    #[test]
    fn spec_scenario_a_form_asked_about_as_its_own_dictionary_form() {
        // `saw` is held only as a form of see, which files its gloss, its runs and `saw` as its
        // past tense.
        let pack = build_pack_with_grammar(
            &[("saw", "see"), ("seen", "see")],
            &["see"],
            &[("see", "Voir; Siège")],
            &[("saw", "see", PAST), ("seen", "see", PARTICIPLE)],
            &[],
            &[("see", &[("VERB", 1), ("NOUN", 1)])],
        );
        assert_eq!(word_grammar("saw", "saw", EN, &pack), nothing());
        assert_eq!(
            word_grammar_json("saw", "saw", EN, &pack),
            r#"{"gloss":null,"senses":[],"readings":[],"others":[],"pieces":[]}"#
        );
        // Asked for see, the card reads see's entry, as before.
        let see = word_grammar("saw", "see", EN, &pack);
        assert_eq!(see.gloss.as_deref(), Some("Voir; Siège"));
        let groups: Vec<(Option<String>, &str)> = see
            .senses
            .iter()
            .map(|g| (g.tag.as_ref().map(Tag::to_ud), g.text.as_str()))
            .collect();
        assert_eq!(
            groups,
            [
                (Some("VERB".to_owned()), "Voir"),
                (Some("NOUN".to_owned()), "Siège")
            ]
        );
        assert_eq!(ud(&see.readings), [PAST]);
    }

    #[test]
    fn spec_scenario_a_dictionary_form_keeps_its_own() {
        let pack = grammar_pack();
        let went = word_grammar("went", "go", EN, &pack);
        assert_eq!(went.gloss.as_deref(), Some("Aller"));
        assert_eq!(
            went.senses,
            [SenseGroup {
                tag: None,
                text: "Aller".into()
            }]
        );
        assert_eq!(ud(&went.readings), [PAST]);
        let leaves = word_grammar("leaves", "leave", EN, &pack);
        assert_eq!(ud(&leaves.readings), [THIRD_SINGULAR]);
        assert_eq!(leaves.others.len(), 1);
        assert_eq!(leaves.others[0].lemma, "leaf");
        assert_eq!(ud(&leaves.others[0].readings), [PLURAL]);
    }

    /// An en→fr pack holding `building` only as a form of *build*, whose paradigm files
    /// `builds` — the edit `+s` — as its present third person singular.
    fn building_pack() -> Pack {
        build_pack_with_grammar_for(
            EN,
            &[
                ("building", "build"),
                ("builds", "build"),
                ("built", "build"),
            ],
            &[
                "again", "and", "build", "every", "in", "old", "stand", "still", "the", "town",
                "year",
            ],
            &[
                ("the", 1),
                ("and", 2),
                ("in", 4),
                ("again", 70),
                ("every", 150),
                ("year", 180),
                ("still", 200),
                ("old", 300),
                ("build", 500),
                ("town", 600),
                ("stand", 800),
            ],
            &[("build", "Construire, édifier")],
            &[
                ("builds", "build", THIRD_SINGULAR),
                ("built", "build", PAST),
                ("building", "build", "VERB|VerbForm=Ger"),
            ],
            &[],
            &[("build", &[("VERB", 1)])],
        )
    }

    #[test]
    fn spec_scenario_an_english_plural_the_pack_does_not_list() {
        let pack = building_pack();
        // The plural fallback reads `buildings` as `building`, a form of build: no gloss, no
        // reading — not build's present third person singular put on another word.
        assert_eq!(
            crate::analysis::lemmatize::lemmatize("buildings", EN, pack.lexicon()),
            "building"
        );
        assert_eq!(word_grammar("buildings", "building", EN, &pack), nothing());
        let blocks =
            ["The old buildings still stand in the town, and the town builds again every year."];
        let mut knowledge = KnowledgeState::new();
        // Under build's rank: the token is unknown, glossed with nothing, beside `builds`
        // glossed with build's gloss.
        knowledge.set_calibration(EN, 400);
        let page = analyse_page(&blocks, EN, &pack, &knowledge);
        let token = |surface: &str| {
            page.tokens
                .iter()
                .find(|t| t.surface == surface)
                .unwrap_or_else(|| panic!("{surface}"))
                .clone()
        };
        let buildings = token("buildings");
        assert_eq!(buildings.lemma, "building");
        assert_eq!(buildings.class, TokenClass::Unknown);
        assert_eq!(buildings.gloss, None);
        let builds = token("builds");
        assert_eq!(builds.class, TokenClass::Unknown);
        assert_eq!(builds.gloss.as_deref(), Some("Construire, édifier"));
        // Over it: build's rank still makes the token known, so the page counts it as before —
        // every word of the page is ranked under 3,000.
        knowledge.set_calibration(EN, 3_000);
        let page = analyse_page(&blocks, EN, &pack, &knowledge);
        let buildings = page
            .tokens
            .iter()
            .find(|t| t.surface == "buildings")
            .unwrap();
        assert_eq!(buildings.class, TokenClass::Known);

        assert_eq!(buildings.gloss, None);
        assert_eq!(
            (page.counted, page.known, page.percent),
            (15, 15, Some(100))
        );
    }

    #[test]
    fn a_phrase_gloss_glosses_a_form_with_nothing() {
        let pack = building_pack();
        let phrase = gloss_phrase("old buildings", EN, &pack, &KnowledgeState::new());
        let buildings = &phrase.tokens[1];
        assert_eq!(buildings.lemma, "building");
        assert_eq!(buildings.gloss, None);
        // A token read as build itself keeps build's gloss.
        let phrase = gloss_phrase("old builds", EN, &pack, &KnowledgeState::new());
        assert_eq!(
            phrase.tokens[1].gloss.as_deref(),
            Some("Construire, édifier")
        );
    }

    #[test]
    fn spec_scenario_a_spanish_plural_the_pack_does_not_list() {
        const MASC_PLURAL: &str = "ADJ|Gender=Masc|Number=Plur";
        // `ablativa` is held only as a form of ablativo, whose paradigm files `ablativos` — the
        // edit `+s` — as its masculine plural.
        let pack = build_pack_with_grammar_for(
            ES,
            &[("ablativa", "ablativo"), ("ablativos", "ablativo")],
            &["ablativo", "las", "son", "raras"],
            &[
                ("las", 1),
                ("son", 20),
                ("raras", 4_000),
                ("ablativo", 30_000),
            ],
            &[("ablativo", "Ablatif")],
            &[
                ("ablativos", "ablativo", MASC_PLURAL),
                ("ablativa", "ablativo", "ADJ|Gender=Fem|Number=Sing"),
            ],
            &[],
            &[("ablativo", &[("ADJ", 1)])],
        );
        assert_eq!(
            crate::analysis::lemmatize::lemmatize("ablativas", ES, pack.lexicon()),
            "ablativa"
        );
        // The card names no reading — not ablativo's masculine plural — and no gloss.
        assert_eq!(word_grammar("ablativas", "ablativa", ES, &pack), nothing());
        assert_eq!(
            ud(&word_grammar("ablativos", "ablativo", ES, &pack).readings),
            [MASC_PLURAL]
        );
        // The page analysis glosses the token with nothing; its class is ablativo's.
        let page = analyse_page(
            &["Las ablativas son raras en los textos que leemos cada día en la escuela."],
            ES,
            &pack,
            &KnowledgeState::new(),
        );
        let ablativas = page
            .tokens
            .iter()
            .find(|t| t.surface == "ablativas")
            .expect("ablativas");
        assert_eq!(ablativas.lemma, "ablativa");
        assert_eq!(ablativas.class, TokenClass::Unknown);
        assert_eq!(ablativas.gloss, None);
        let phrase = gloss_phrase("ablativas", ES, &pack, &KnowledgeState::new());
        assert_eq!(phrase.tokens[0].gloss, None);
    }
}
