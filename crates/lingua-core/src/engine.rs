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

use crate::analysis::carries_enclitic;
use crate::analysis::function_words::{is_function_word, is_preposition};
use crate::analysis::language::StudiedLanguage;
use crate::analysis::lexicon::Lexicon;
use crate::analysis::percent::{
    Coverage, TokenClass, compound_is_out_of_lexicon_proper_noun, is_out_of_lexicon_proper_noun,
};
use crate::analysis::pipeline::{
    AnalysedToken, DocumentAnalysis, alone_in_capitals, analyse_document, headword_reading,
    resolve_lemmas,
};
use crate::analysis::tokenize::{Token, is_elided, is_hyphen_join, tokenize, without_soft_hyphens};
use crate::knowledge::level::CefrLevels;
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

    // A document's names are set aside like a proper noun outside the lexicon, in every studied
    // language (add-lingua-spanish-names, add-lingua-french-analysis D4,
    // leave-lingua-names-out-of-english-and-spanish-words D4).
    let names = document_names(&tokens, blocks, studied, pack);
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
///
/// An English document's names follow Spanish's rule as written, with two English
/// readings (leave-lingua-names-out-of-english-and-spanish-words D4): the pronoun
/// `I` and its contractions (`I'm`, `I'll`, `I’d`), capitalised wherever they
/// stand, give no evidence ([`is_pronoun_i`]); and a lemma a CEFR list levels
/// (`june`, `mr`, `mrs`, [`CefrLevels::level`]) is a word a reader learns, never a
/// name. Neither reading runs for Spanish or French, whose levels are estimated
/// and go only to dictionary words.
fn document_names(
    tokens: &[AnalysedToken],
    blocks: &[&str],
    studied: StudiedLanguage,
    pack: &Pack,
) -> HashSet<String> {
    let french = studied == StudiedLanguage::French;
    let english = studied == StudiedLanguage::English;
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
        if english && is_pronoun_i(&form) {
            continue;
        }
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
        .filter(|(form, lemma)| {
            !lowercase.contains(form)
                && !pack.is_dictionary_word(lemma)
                && !(english && pack.level(lemma).is_some())
        })
        .map(|(form, _)| form)
        .collect()
}

/// Whether an English form, lowercased, is the pronoun `I` or one of its
/// contractions (`i`, `i'm`, `i'll`, `i’d`, `i've`): capitalised wherever it
/// stands, so its capital says nothing of a name
/// (leave-lingua-names-out-of-english-and-spanish-words D4).
fn is_pronoun_i(form: &str) -> bool {
    form == "i" || form.starts_with("i'") || form.starts_with("i\u{2019}")
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
/// phrase gloss so the two can never disagree — except for a document's names (a
/// French one's hyphenated runs among them), which only the
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
    /// The expression's dictionary form, which is the card's. For English, the
    /// covered tokens' lemmas joined by single spaces, which is the pack's key.
    /// For French and Spanish, the expression's name — its headword as the
    /// dictionary writes it (`au revoir`, `il y a`, `tener en cuenta`) — which
    /// the pack carries wherever it differs from the key (`à le revoir`,
    /// `il y avoir`, `tener en contar`), and the key itself where it carries none
    /// (add-lingua-french-expression-keys D3, add-lingua-spanish-expression-keys
    /// D3). Except that a Spanish expression the reader settled under its lemma
    /// chain — the covered tokens' lemmas joined, before the article of a
    /// contraction is covered, the key this field held before Spanish
    /// expressions were named — and not under its name keeps that chain, so its
    /// status and deck card are still read and acted on
    /// (add-lingua-spanish-expression-keys D6).
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

/// How many tokens an English expression may span. 98.9 % of the English
/// table is five words or fewer (`add-lingua-expression-table`, design D3), and
/// every token of a selection pays for the ones beyond it.
const EXPRESSION_WINDOW: usize = 5;

/// How many tokens a French expression may span: French's pieces lengthen a key
/// (`au fur et à mesure` is five words and six tokens), and seven tokens hold
/// 98.8 % of its keys as five hold 98.9 % of English's
/// (add-lingua-french-expression-keys D4). The builder leaves a longer key out.
pub const FRENCH_EXPRESSION_WINDOW: usize = 7;

/// How many tokens a Spanish expression may span: `al` and `del` read as two
/// tokens each lengthen a key (`al fin y al cabo` is five words and seven
/// tokens), and seven tokens hold 99.3 % of es-fr's keyed headwords and 99.5 %
/// of es-en's (add-lingua-spanish-expression-keys D4). The builder leaves a
/// longer key out.
pub const SPANISH_EXPRESSION_WINDOW: usize = 7;

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

/// The words a Spanish expression key writes as the pre-pass gives them, in
/// lowercase, rather than as their dictionary form: the articles, and the
/// possessive and demonstrative determiners, sorted
/// (add-lingua-spanish-expression-keys D2) — French's [`FRENCH_KEY_WRITTEN`],
/// transposed. The `el` the pre-pass writes for « al » and « del » is written
/// `el`, so `al menos` is `a el menos` and `a la vez` stays `a la vez`.
///
/// M8 files `la`, `los` and `las` under `el`, `una`, `unos` and `unas` under
/// `uno`, `esta`, `estos`… under `este`, `mis` under `mi` and `nuestra`… under
/// `nuestro`: right for a word, wrong for an expression. The list holds those
/// and, for completeness, the forms that are their own dictionary form.
/// Measured on es-fr's and es-en's committed tables and the Spanish baseline's
/// corpus, every token lemmatised keys `a la`, `a las` and `a los` as one `a el`
/// and `de las` and `de los` as one `de el`, which then answered « al », « del »
/// and every other article: with these words written, the keys several
/// headwords reach fall from 116 to 98 (es-fr) and from 151 to 138 (es-en), and
/// the matches on the corpus from 44 to 32 and from 43 to 27, the article
/// entries' answers on another article or a contraction gone. The clitic
/// pronouns are left out, as French's list leaves out `me`, `te` and `se`: they
/// change nothing measurable.
pub const SPANISH_KEY_WRITTEN: &[&str] = &[
    "aquel", "aquella", "aquellas", "aquellos", "el", "esa", "esas", "ese", "esos", "esta",
    "estas", "este", "estos", "la", "las", "lo", "los", "mi", "mis", "nuestra", "nuestras",
    "nuestro", "nuestros", "su", "sus", "tu", "tus", "un", "una", "unas", "unos", "vuestra",
    "vuestras", "vuestro", "vuestros",
];

/// The clitic pronouns a Spanish page may write just before an expression's run
/// (pick-lingua-spanish-expression-written-as-the-page D3): where the page's word
/// before the run is one of them, lowercased — « No te hagas ilusiones », « Los tenía
/// puestos » —, the page writes a clitic pronoun for the run, and a headword written
/// with one is weighed. The same eleven words Spanish's enclitic rule strips. Sorted
/// for binary search (a test enforces it).
pub const SPANISH_CLITIC_PRONOUNS: &[&str] = &[
    "la", "las", "le", "les", "lo", "los", "me", "nos", "os", "se", "te",
];

/// A token's piece of an expression key (add-lingua-french-expression-keys D2,
/// add-lingua-spanish-expression-keys D2): its dictionary form, except a word of
/// the studied language's written list — [`FRENCH_KEY_WRITTEN`],
/// [`SPANISH_KEY_WRITTEN`] — written as the pre-pass gives it, lowercased, the
/// typographic apostrophe read as the straight one: so `à la` is never `au`
/// (`à le`), nor `a la` « al » (`a el`). English keys are their lemmas, as
/// before.
pub fn expression_piece(surface: &str, lemma: &str, studied: StudiedLanguage) -> String {
    let listed = match studied {
        StudiedLanguage::French => FRENCH_KEY_WRITTEN,
        StudiedLanguage::Spanish => SPANISH_KEY_WRITTEN,
        StudiedLanguage::English => return lemma.to_owned(),
    };
    let written = surface.replace('\u{2019}', "'").to_lowercase();
    if listed.binary_search(&written.as_str()).is_ok() {
        return written;
    }
    lemma.to_owned()
}

/// The key an expression's headword — or its name, which is the same thing — is
/// filed under in a language read through its reading, French or Spanish
/// (add-lingua-french-expression-keys D1, D4, D6; add-lingua-spanish-expression-keys
/// D1, D2, D4): what the studied language's analysis reads in it
/// ([`headword_reading`]), each token written by [`expression_piece`], joined by
/// single spaces: `au revoir` → `à le revoir`, `coup d'œil` → `coup de œil`,
/// `il y a` → `il y avoir`, `à la` → `à la`; `al menos` → `a el menos`,
/// `tener en cuenta` → `tener en contar`, `a la vez` → `a la vez`.
///
/// `None` for English, which keys its expressions at their spaces, and unless the
/// headword reads as two to the language's window of tokens
/// ([`FRENCH_EXPRESSION_WINDOW`], [`SPANISH_EXPRESSION_WINDOW`]), every word of it
/// giving one, and every token's dictionary form is a lemma of the lexicon: a key
/// no reading of a page can produce would sit in the pack unreachable. The one
/// implementation the pack builder keys a French or Spanish pack with and review
/// finds such an expression card's gloss through, so the two cannot drift; it
/// asks the lexicon whether a dictionary form is a lemma, never which lemma a form
/// files under.
pub fn reading_expression_key(
    headword: &str,
    studied: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> Option<String> {
    if studied == StudiedLanguage::English {
        return None;
    }
    let reading = headword_reading(headword, studied, lexicon)?;
    if !(2..=expression_window(studied)).contains(&reading.len())
        || !reading
            .iter()
            .all(|(_, lemma)| lexicon.contains_lemma(lemma))
    {
        return None;
    }
    Some(
        reading
            .iter()
            .map(|(token, lemma)| expression_piece(&token.text, lemma, studied))
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// The key a French expression's headword — or its name — is filed under:
/// [`reading_expression_key`] for French (add-lingua-french-expression-keys D1),
/// `au revoir` → `à le revoir`, `d'abord` → `de abord`.
pub fn french_expression_key(headword: &str, lexicon: &(impl Lexicon + ?Sized)) -> Option<String> {
    reading_expression_key(headword, StudiedLanguage::French, lexicon)
}

/// How many tokens an expression may span in the studied language.
fn expression_window(studied: StudiedLanguage) -> usize {
    match studied {
        StudiedLanguage::French => FRENCH_EXPRESSION_WINDOW,
        StudiedLanguage::Spanish => SPANISH_EXPRESSION_WINDOW,
        StudiedLanguage::English => EXPRESSION_WINDOW,
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
/// forms, and in French and Spanish the determiners as written) are a key of the
/// table wins, and the next run starts past it, so a token belongs to at most one
/// match. Runs start at two tokens because every key holds a space — a single
/// lemma is a word, not an expression, and could never be one — and stop at the
/// language's window. A match reports the expression's name where the pack
/// carries one (French, Spanish), else its key, and the reader's status is read
/// on what it reports (add-lingua-french-expression-keys D3,
/// add-lingua-spanish-expression-keys D3).
///
/// `shares_span[i]` says whether token `i` shares its predecessor's source span —
/// the second half of a written word the pre-pass split. A Spanish or French match
/// never ends inside a written word: one whose last token is followed by such a
/// token, the article of « al » or « del », of « au » or « aux », covers it too, so
/// « después del » and « jusqu'au » are answered whole
/// (add-lingua-spanish-expression-keys D5, add-lingua-french-word-card D12). English's
/// split words (`don't`) stay as they were. And a Spanish expression the reader
/// settled under the run's lemma chain, and not under its name, keeps that chain
/// ([`settled_chain`], D6).
///
/// `elided[i]` says whether token `i` is an elided piece ([`is_elided`]). A French
/// run that is a key is kept only where its pieces meet the words as the
/// expression's headword writes them ([`meets_as_written`],
/// match-lingua-french-elided-pieces D1, match-lingua-french-contracted-pieces D1):
/// a piece the headword writes elided meets only an elided token, a piece written
/// in full only a token written in full — but the last piece written in full meets
/// either, French eliding it or not by the page's next word —, so « de le faire »
/// no longer answers `de l'`, nor « d’un » `de un`; and two pieces the headword
/// writes as one word, « au » or « aux », meet only two tokens sharing a span
/// (`shares_span`), two pieces it writes apart only tokens with spans of their
/// own — the run's edges being the page's: its first token may be the article of
/// a « au » whose `à` is before the run, its last the `à` of one whose article
/// is covered as above —, so « prêt à le faire » no longer answers `au fait`. A
/// run whose pieces do not meet does not match at that length, and the shorter
/// runs are tried as before (« d’un peu plus » answers `un peu`). English and
/// Spanish runs are matched as before.
///
/// A French run that is a key several headwords reach ([`Pack::expression_headwords`]:
/// `ça va` and `ça ira` both read `ça aller`) is named by the headword written most like
/// the page ([`written_as_the_page`], pick-lingua-expression-written-as-the-page D2,
/// D3): each headword is held to the run as above, the ones that meet it weighed, and the
/// one written as the page at the most places reports its name and its gloss, the status
/// read on that name — « Ça va ? » answers `ça va`, « Ça ira. » `ça ira`. A run none of
/// them meets does not match at that length, and the shorter runs are tried. A key one
/// headword reaches is matched as before, without a reading.
///
/// A Spanish run that is a key several headwords reach (`hacer tiempo` and `hace tiempo`
/// both read `hacer tiempo`) is named the same way (pick-lingua-spanish-expression-written-
/// as-the-page D2, D3): every headword is weighed, there being no Spanish counterpart of
/// [`meets_as_written`], except that one written with a clitic pronoun (`tenerlos puestos`,
/// `hacerse ilusiones`) is weighed only where the page writes one for the run
/// ([`spanish_weighed`]); where none is weighed so, all are. So « hace tiempo » answers
/// `hace tiempo`, and « tenía puestos » `tener puesto`, not `tenerlos puestos`. No run
/// matches, or stops matching, for it. The lemma chain a reader settled before is then
/// reported only where it is no headword's name ([`settled_chain`], D4).
///
/// And in the three languages, a run that is a key and meets its headword gives
/// its last token to a longer expression when it is built on a verb
/// (choose-lingua-overlapping-expressions D1, D2): [`ExpressionScan::gives_last_word`].
/// It then does not match at that length, exactly as a run that does not meet its
/// headword: the shorter runs are tried, else the next token, where the scan finds
/// the expression as it finds any — « Elles sont au nombre de sept » answers
/// `au nombre de` and not `être à`, « Lo hizo de todos modos » `de todos modos`,
/// « went over the top » `over the top`. `hyphened[i]` says whether token `i` is
/// joined to token `i - 1` by a hyphen on the page ([`is_hyphen_join`]), read only
/// by the rule's French guard (G1).
fn match_expressions(
    tokens: &[PhraseToken],
    joins: &PageJoins,
    studied: StudiedLanguage,
    pack: &Pack,
    knowledge: &KnowledgeState,
) -> Vec<PhraseMatch> {
    if !pack.has_expressions() {
        return Vec::new();
    }
    let scan = ExpressionScan::new(tokens, joins, studied, pack);
    let mut matches = Vec::new();
    let mut start = 0;
    while start < tokens.len() {
        let longest = scan.window.min(tokens.len() - start);
        let hit = (2..=longest).rev().find_map(|len| {
            let (name, gloss, headwords) = scan.run(start, len)?;
            (!scan.gives_last_word(start, len)).then_some((len, name, gloss, headwords))
        });
        match hit {
            Some((len, name, gloss, headwords)) => {
                let key = if studied == StudiedLanguage::Spanish {
                    settled_chain(
                        &tokens[start..start + len],
                        &name,
                        &headwords,
                        studied,
                        knowledge,
                    )
                    .unwrap_or(name)
                } else {
                    name
                };
                let end = scan.covered_end(start, len);
                let class = knowledge.classify(studied, &[key.as_str()], pack);
                matches.push(PhraseMatch {
                    start,
                    end,
                    key,
                    class,
                    gloss,
                });
                start = end;
            }
            None => start += 1,
        }
    }
    matches
}

/// What the phrase gloss reads between a selection's tokens on the page, one entry
/// per token, entry `i` about token `i` and the token before it.
struct PageJoins {
    /// [`shared_spans`]: the second half of a written word the pre-pass split.
    shares_span: Vec<bool>,
    /// [`is_elided`]: an elided piece.
    elided: Vec<bool>,
    /// [`is_hyphen_join`] over the text between the two tokens' spans
    /// (choose-lingua-overlapping-expressions D3, G1); never the first token.
    hyphened: Vec<bool>,
}

impl PageJoins {
    /// The joins of `read`, the selection `text`'s tokens.
    fn of(text: &str, read: &[Token]) -> Self {
        let hyphened = (0..read.len())
            .map(|i| {
                i.checked_sub(1).is_some_and(|before| {
                    read[before].end <= read[i].start
                        && is_hyphen_join(&text[read[before].end..read[i].start])
                })
            })
            .collect();
        Self {
            shares_span: shared_spans(read),
            elided: read.iter().map(|token| is_elided(text, token)).collect(),
            hyphened,
        }
    }
}

/// One selection's expression lookup, as [`match_expressions`] scans it.
struct ExpressionScan<'a> {
    tokens: &'a [PhraseToken],
    joins: &'a PageJoins,
    /// Each token's key piece ([`expression_piece`]).
    pieces: Vec<String>,
    studied: StudiedLanguage,
    pack: &'a Pack,
    window: usize,
}

impl<'a> ExpressionScan<'a> {
    fn new(
        tokens: &'a [PhraseToken],
        joins: &'a PageJoins,
        studied: StudiedLanguage,
        pack: &'a Pack,
    ) -> Self {
        Self {
            pieces: tokens
                .iter()
                .map(|token| expression_piece(&token.surface, &token.lemma, studied))
                .collect(),
            tokens,
            joins,
            studied,
            pack,
            window: expression_window(studied),
        }
    }

    fn french(&self) -> bool {
        self.studied == StudiedLanguage::French
    }

    /// French and Spanish, whose matches never end inside a written word.
    fn whole_words(&self) -> bool {
        matches!(
            self.studied,
            StudiedLanguage::Spanish | StudiedLanguage::French
        )
    }

    /// The run of `len` tokens from `start` as an expression — its name (the pack's,
    /// else its key) and its gloss — when it is a key and, in French, meets its
    /// headword as written ([`meets_as_written`]); a French or Spanish key several
    /// headwords reach is named by the one written most like the page
    /// ([`written_as_the_page`]), and every headword's name comes with it, which the
    /// settled chain reads ([`settled_chain`]).
    fn run(&self, start: usize, len: usize) -> Option<(String, String, Vec<String>)> {
        let (key, gloss) =
            expression_for_run(&self.pieces[start..start + len], self.studied, self.pack)?;
        let (tokens, elided, shares_span) =
            (self.tokens, &self.joins.elided, &self.joins.shares_span);
        if self.whole_words() {
            let headwords = self.pack.expression_headwords(&key);
            if headwords.len() > 1 {
                let run = start..start + len;
                let lexicon = self.pack.lexicon();
                let (name, gloss) = if self.french() {
                    written_as_the_page(
                        &headwords,
                        &tokens[run.clone()],
                        self.studied,
                        lexicon,
                        |name| {
                            meets_as_written(
                                &elided[run.clone()],
                                &shares_span[run.clone()],
                                name,
                                lexicon,
                            )
                        },
                    )?
                } else {
                    let weighed = spanish_weighed(&headwords, tokens, run.clone(), lexicon);
                    written_as_the_page(&weighed, &tokens[run], self.studied, lexicon, |_| true)?
                };
                let names = headwords
                    .iter()
                    .map(|(name, _)| (*name).to_owned())
                    .collect();
                return Some((name.to_owned(), gloss.to_owned(), names));
            }
        }
        let name = match self.pack.expression_name(&key) {
            Some(name) => name.to_owned(),
            None => key,
        };
        (!self.french()
            || meets_as_written(
                &elided[start..start + len],
                &shares_span[start..start + len],
                &name,
                self.pack.lexicon(),
            ))
        .then(|| (name.clone(), gloss, vec![name]))
    }

    /// The longest run from `start` that is an expression ([`ExpressionScan::run`]),
    /// its length and name: the one the scan looks up at a start, without asking
    /// whether it would give its own last word away.
    fn longest_at(&self, start: usize) -> Option<(usize, String)> {
        let longest = self.window.min(self.tokens.len().saturating_sub(start));
        (2..=longest)
            .rev()
            .find_map(|len| self.run(start, len).map(|(name, _, _)| (len, name)))
    }

    /// Where a match of `len` tokens from `start` ends: past its last token, and in
    /// French and Spanish past the article of a written « au », « aux », « al » or
    /// « del » it would otherwise end inside.
    fn covered_end(&self, start: usize, len: usize) -> usize {
        let mut end = start + len;
        if self.whole_words() {
            while self.joins.shares_span.get(end).copied().unwrap_or(false) {
                end += 1;
            }
        }
        end
    }

    /// Whether the expression of `len` tokens from `start` gives its last token to
    /// the expression starting on it (choose-lingua-overlapping-expressions D1):
    /// that expression, the longest run that is a key and meets its headword there
    /// ([`ExpressionScan::longest_at`]), holds more tokens than the run, or (French,
    /// Spanish) starts on the `à` of « au »/« aux » or the `a`/`de` of « al »/« del »
    /// the run ends on; the run is built on a verb ([`built_on_verb`]); and no guard
    /// of D3 holds — G1 (French): the expression taking the word meets its
    /// headword's hyphens on the page ([`meets_hyphens`]); G2 (French, Spanish): the
    /// last token is a preposition the selection's next token, `que`, follows
    /// (« pese a que »); G3: the run the scan would take next after this one ends
    /// past the expression taking the word and holds a word that is not a function
    /// word (« iba al día siguiente » keeps `ir a` and `día siguiente`). The
    /// checks run cheapest first.
    fn gives_last_word(&self, start: usize, len: usize) -> bool {
        let end = start + len;
        let last = end - 1;
        let Some((taker_len, taker)) = self.longest_at(last) else {
            return false;
        };
        let on_contraction =
            self.whole_words() && self.joins.shares_span.get(end).copied().unwrap_or(false);
        if !(on_contraction || taker_len > len) {
            return false;
        }
        // G2: a preposition the page follows with « que » stays.
        if self.whole_words()
            && self.tokens.get(end).is_some_and(|next| next.lemma == "que")
            && is_preposition(&self.tokens[last].lemma, self.studied)
        {
            return false;
        }
        // G1: the expression taking the word meets its headword's hyphens.
        if self.french()
            && !meets_hyphens(
                &self.joins.hyphened[last..last + taker_len],
                &taker,
                self.pack.lexicon(),
            )
        {
            return false;
        }
        if !built_on_verb(&self.tokens[start..end], self.studied, self.pack) {
            return false;
        }
        // G3: the expression next is not cut.
        let taker_end = self.covered_end(last, taker_len);
        let next = (self.covered_end(start, len)..taker_end)
            .find_map(|at| self.longest_at(at).map(|(len, _)| (at, len)));
        !next.is_some_and(|(at, next_len)| {
            let next_end = self.covered_end(at, next_len);
            next_end > taker_end
                && self.tokens[at..next_end]
                    .iter()
                    .any(|token| !token.function_word)
        })
    }
}

/// Whether a run is built on a verb (choose-lingua-overlapping-expressions D2): one of
/// its tokens, its last included, has a reading in the pack's grammar
/// ([`Pack::readings`] on its dictionary form and lowercased surface) as a verb or an
/// auxiliary, and no token before that one is a preposition of the studied language
/// ([`is_preposition`]: `à partir de`, `a pesar de` are built on a preposition). A pack
/// without grammar reads no verb.
fn built_on_verb(run: &[PhraseToken], studied: StudiedLanguage, pack: &Pack) -> bool {
    for token in run {
        let verb = pack
            .readings(&token.lemma, &token.surface.to_lowercase())
            .iter()
            .any(|tag| matches!(tag.pos, "VERB" | "AUX"));
        if verb {
            return true;
        }
        if is_preposition(&token.lemma, studied) {
            return false;
        }
    }
    false
}

/// Whether a French run meets its expression's headword's hyphens
/// (choose-lingua-overlapping-expressions D3, G1): `hyphened` holds the run's tokens'
/// [`PageJoins::hyphened`], `headword` the expression's name. Every place the headword,
/// read by French's pre-pass ([`headword_reading`]), joins two pieces with a hyphen —
/// the euphonic `t` between counted as one ([`is_hyphen_join`]) — the page joins the
/// run's two tokens with one too: `est-ce que` meets « Est-ce que » and not « c'est ce
/// que ». A headword without a hyphen, or whose reading does not line up with the run,
/// meets. Asked only of the expression that would take a word: an expression matched on
/// its own meets the page as before (« Est ce que tu viens ? » answers `est-ce que`).
fn meets_hyphens(hyphened: &[bool], headword: &str, lexicon: &(impl Lexicon + ?Sized)) -> bool {
    if !headword.contains('-') {
        return true;
    }
    let Some(reading) = headword_reading(headword, StudiedLanguage::French, lexicon)
        .filter(|reading| reading.len() == hyphened.len())
    else {
        return true;
    };
    reading
        .windows(2)
        .zip(&hyphened[1..])
        .all(|(pair, &on_page)| {
            let (before, piece) = (&pair[0].0, &pair[1].0);
            on_page
                || before.end > piece.start
                || !is_hyphen_join(&headword[before.end..piece.start])
        })
}

/// Whether a French run's tokens meet, piece by piece, its expression's headword as
/// the headword writes them (match-lingua-french-elided-pieces D1, D3;
/// match-lingua-french-contracted-pieces D1, D3): `elided` holds the run's tokens'
/// [`is_elided`], `joined` their [`shared_spans`] as the selection's tokens give them
/// (entry 0 says whether the run's first token shares its span with the token before
/// the run), `headword` the expression's name — the pack's, or its key where it carries
/// none.
///
/// A piece the headword writes elided (`l'` of `de l'`, `qu'` of `qu'est-ce que`) meets
/// only an elided token; a piece it writes in full meets only a token written in full,
/// except its last, which meets either: French elides a word by the word that follows
/// it, and after the expression's last piece that word is the page's (« parce qu’il »
/// answers `parce que`).
///
/// Between two adjacent tokens of the run, the tokens are one written word exactly where
/// the headword writes its two pieces at that place as one: two pieces it writes « au »
/// or « aux » (`à` + `le`, `à` + `les`) meet only a written « au » or « aux », the two
/// tokens sharing a span — `au fait` meets « Au fait » and not « prêt à le faire » —, and
/// two pieces it writes apart only tokens with spans of their own (a headword written
/// `à le revoir` does not meet « au revoir »). The run's edges are the page's: `joined[0]`
/// is not compared, so the run's first token may be the article of a « au » or « aux »
/// whose `à` is the page's word before the run (`les miennes` on « aux miennes »); and the
/// run's last token may be the `à` of one whose article, the page's next word, is outside
/// the slice — `match_expressions` then covers it (add-lingua-french-word-card D12:
/// `grâce à` on « grâce au soleil », `jusqu'à` on « jusqu'au soir »).
///
/// The headword is read by French's pre-pass ([`headword_reading`], the reading its key
/// was made from, so its pieces line up with the run's one for one), each piece's
/// elision taken from its span and its contractions from the reading's
/// [`shared_spans`] — one reading for both, made only when the headword holds an
/// apostrophe or the word « au » or « aux » ([`holds_french_contraction`]): a name
/// holding neither is written in full, every piece apart, and a key the pack does not
/// name is its own headword, written so. A headword whose reading does not line up with
/// the run, or that reads as none, is matched as match-lingua-french-elided-pieces matched
/// it, its contractions unchecked.
fn meets_as_written(
    elided: &[bool],
    joined: &[bool],
    headword: &str,
    lexicon: &(impl Lexicon + ?Sized),
) -> bool {
    let last = elided.len().saturating_sub(1);
    let apostrophe = headword.contains(['\'', '\u{2019}']);
    if !apostrophe && !holds_french_contraction(headword) {
        return !elided[..last].contains(&true) && !joined.iter().skip(1).any(|&on_page| on_page);
    }
    let Some(reading) = headword_reading(headword, StudiedLanguage::French, lexicon)
        .filter(|reading| reading.len() == elided.len())
    else {
        return apostrophe || !elided[..last].contains(&true);
    };
    let pieces: Vec<Token> = reading.into_iter().map(|(token, _)| token).collect();
    let elisions_meet = pieces
        .iter()
        .zip(elided)
        .enumerate()
        .all(|(i, (piece, &on_page))| {
            let written = is_elided(headword, piece);
            on_page == written || (i == last && !written)
        });
    elisions_meet
        && joined
            .iter()
            .skip(1)
            .eq(shared_spans(&pieces).iter().skip(1))
}

/// The headword a French or Spanish run is named by where several headwords reach the key
/// it matches (pick-lingua-expression-written-as-the-page D2, D3;
/// pick-lingua-spanish-expression-written-as-the-page D2). `headwords` are the key's, as
/// the pack lists them ([`Pack::expression_headwords`]: the one that wins the key first,
/// then the others in the builder's order) — for Spanish, those [`spanish_weighed`] keeps
/// —; `run` is the run's tokens, `studied` its language.
///
/// Only the headwords `weighed` accepts are weighed — in French those whose pieces meet the
/// run as written ([`meets_as_written`]): `ce qui` does not meet « c'qui », whose `c'` is
/// elided, and `c'qui` does; in Spanish every one. Each is read by
/// [`headword_reading`], the reading its key was made from, so its pieces line up with the
/// run's tokens one for one, and a place counts where a piece's text is its token's, both
/// lowercased and the typographic apostrophe read as the straight one, as
/// [`expression_piece`] writes a key; a reading that does not line up counts none. The
/// headword written as the page at the most places wins, and the first in the pack's order
/// between equals: « Ça va ? » answers `ça va` (two places, `ça ira` one), « Ça ira. »
/// `ça ira`, « quelle que soit » `quel que soit` (`que` and `soit`, `quel que fût` `que`
/// alone), « ça allait » `ça ira`, which wins the key (`ça` in both); « hace tiempo »
/// `hace tiempo`, « dando vueltas » `dar vueltas` (`vueltas`). `None` when no headword is
/// weighed: it does not match at that length.
fn written_as_the_page<'a, L: Lexicon + ?Sized>(
    headwords: &[(&'a str, &'a str)],
    run: &[PhraseToken],
    studied: StudiedLanguage,
    lexicon: &L,
    weighed: impl Fn(&str) -> bool,
) -> Option<(&'a str, &'a str)> {
    let compared = |text: &str| text.replace('\u{2019}', "'").to_lowercase();
    let page: Vec<String> = run.iter().map(|token| compared(&token.surface)).collect();
    let mut best: Option<((&'a str, &'a str), usize)> = None;
    for &(name, gloss) in headwords {
        if !weighed(name) {
            continue;
        }
        let places = headword_reading(name, studied, lexicon)
            .filter(|reading| reading.len() == page.len())
            .map_or(0, |reading| {
                reading
                    .iter()
                    .zip(&page)
                    .filter(|((piece, _), written)| compared(&piece.text) == **written)
                    .count()
            });
        if best.is_none_or(|(_, most)| places > most) {
            best = Some(((name, gloss), places));
        }
    }
    best.map(|(headword, _)| headword)
}

/// The headwords of a Spanish key several reach that are weighed for a run
/// (pick-lingua-spanish-expression-written-as-the-page D3): a headword one of whose
/// words carries a clitic pronoun ([`carries_enclitic`]: `tenerlos puestos`, `hacerse
/// ilusiones`, `puedes ayudarme`) only where the page writes a clitic pronoun for the run
/// — attached to one of the run's tokens (« hacerme ilusiones ») or as the page's token
/// just before the run, lowercased, one of [`SPANISH_CLITIC_PRONOUNS`] (« No te hagas
/// ilusiones ») —, every other headword always; and every headword where none is weighed
/// so, so the guard chooses among the headwords and never makes a run match or stop
/// matching. `tokens` are the selection's, `run` the run's range in them. A headword
/// written as the page writes its pronoun, so the guard only acts on the closest
/// spelling: « tenía puestos unos audífonos » answers `tener puesto`, not `tenerlos
/// puestos`, whose `puestos` is written as the page.
fn spanish_weighed<'a>(
    headwords: &[(&'a str, &'a str)],
    tokens: &[PhraseToken],
    run: std::ops::Range<usize>,
    lexicon: &(impl Lexicon + ?Sized),
) -> Vec<(&'a str, &'a str)> {
    let before = run
        .start
        .checked_sub(1)
        .map(|at| tokens[at].surface.to_lowercase());
    let page_writes_one = before.is_some_and(|word| {
        SPANISH_CLITIC_PRONOUNS
            .binary_search(&word.as_str())
            .is_ok()
    }) || tokens[run]
        .iter()
        .any(|token| carries_enclitic(&token.surface, lexicon));
    if page_writes_one {
        return headwords.to_vec();
    }
    let bare: Vec<(&'a str, &'a str)> = headwords
        .iter()
        .filter(|(name, _)| {
            !headword_reading(name, StudiedLanguage::Spanish, lexicon).is_some_and(|reading| {
                reading
                    .iter()
                    .any(|(piece, _)| carries_enclitic(&piece.text, lexicon))
            })
        })
        .copied()
        .collect();
    if bare.is_empty() {
        headwords.to_vec()
    } else {
        bare
    }
}

/// Whether a French expression's headword holds the word « au » or « aux », in any case
/// and after an elision too (`jusqu'aux`), read without its soft hyphens: the words French's
/// pre-pass splits into two tokens sharing a span, and the only ones
/// (match-lingua-french-contracted-pieces D2, D3). [`meets_as_written`] reads a headword's
/// contractions only where it holds one; `committed_tables.rs` checks that every committed
/// French headword whose reading joins two pieces does.
pub fn holds_french_contraction(headword: &str) -> bool {
    without_soft_hyphens(headword)
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| word.eq_ignore_ascii_case("au") || word.eq_ignore_ascii_case("aux"))
}

/// Which of `tokens` share their predecessor's source span: entry `i` says whether token
/// `i` is the second half of a written word the pre-pass split (`del` → `de` + `el`,
/// `au` → `à` + `le`, `don't` → `do` + `not`); entry 0 never does. The phrase gloss reads
/// it on a selection's tokens — a Spanish or French match never leaves such a half outside
/// (add-lingua-spanish-expression-keys D5, add-lingua-french-word-card D12) — and on a
/// French expression's headword, whose contractions a French run meets as written
/// ([`meets_as_written`], match-lingua-french-contracted-pieces D2, D3). In French only
/// the halves of « au » and « aux » share a span.
pub fn shared_spans(tokens: &[Token]) -> Vec<bool> {
    tokens
        .iter()
        .enumerate()
        .map(|(i, token)| {
            i.checked_sub(1).is_some_and(|before| {
                (tokens[before].start, tokens[before].end) == (token.start, token.end)
            })
        })
        .collect()
}

/// The lemma chain a reader settled a Spanish expression under before it was named
/// (add-lingua-spanish-expression-keys D6): the run's dictionary forms joined by
/// single spaces — taken before the article of a contraction is covered, so it is
/// exactly the key the phrase gloss reported for those tokens before — when the
/// reader holds a record on it (a status, or a withdrawn one:
/// [`KnowledgeState::holds_record`]) and none on the name. `None` otherwise: the
/// match is then reported under its name.
///
/// `headwords` are the names of every headword of the run's key
/// ([`Pack::expression_headwords`], whether the guard weighed it or not; the name alone
/// for a key one headword reaches). A chain that is one of them is never reported
/// (pick-lingua-spanish-expression-written-as-the-page D4): a record on `hacer tiempo`,
/// `un poco` or `red social` is that headword's, so « hace tiempo » answers `hace tiempo`
/// and « un poquito » `un poquito`, not the record set on the other spelling. A chain no
/// headword is named (`tener en contar`, `bueno día`) is reported as before.
fn settled_chain(
    run: &[PhraseToken],
    name: &str,
    headwords: &[String],
    studied: StudiedLanguage,
    knowledge: &KnowledgeState,
) -> Option<String> {
    let chain = run
        .iter()
        .map(|token| token.lemma.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    (chain != name
        && !headwords.contains(&chain)
        && knowledge.holds_record(studied, &chain)
        && !knowledge.holds_record(studied, name))
    .then_some(chain)
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
    let read = tokenize(text, studied, lexicon);
    // Which tokens share their predecessor's source span: the second half of a word
    // the pre-pass split (`del` → `de` + `el`, `au` → `à` + `le`), which a Spanish or
    // French match never leaves outside (add-lingua-spanish-expression-keys D5,
    // add-lingua-french-word-card D12), and a French match meets only where its
    // headword writes the contraction (match-lingua-french-contracted-pieces D2).
    // Which tokens are elided pieces, read from their spans, which a French match holds
    // to its headword's (match-lingua-french-elided-pieces D2). And which are joined to
    // the token before by a hyphen, which a French expression taking a word from another
    // meets as its headword writes it (choose-lingua-overlapping-expressions D3, G1).
    let joins = PageJoins::of(text, &read);
    // Which tokens stand alone in capitals, read in the selection as a page reads its block
    // (keep-lingua-french-acronyms-in-capitals D1).
    let alone = alone_in_capitals(text, &read, studied);
    let tokens: Vec<PhraseToken> = read
        .into_iter()
        .zip(alone)
        .map(|(token, alone)| {
            let (lemma, parts) = resolve_lemmas(&token, alone, studied, lexicon);
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
    let expressions = match_expressions(&tokens, &joins, studied, pack, knowledge);
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
/// whose readings are answered. A written word that yields no token is read
/// without its soft hyphens, as the tokeniser reads every word
/// (ignore-lingua-soft-hyphens D4). The page analysis never calls this, so its
/// output, and `ANALYZER_VERSION`, stay where they are.
///
/// The dictionary form is answered as asked when the pack holds it as a lemma — an
/// acronym in capitals, `CA` — and lowercased otherwise, as before; the written word's
/// tokens are resolved as a page's, a word standing alone in capitals in it read as
/// the acronym its pack lists (keep-lingua-french-acronyms-in-capitals D1, D6).
pub fn word_grammar(
    written: &str,
    lemma: &str,
    studied: StudiedLanguage,
    pack: &Pack,
) -> WordGrammar {
    let lexicon = pack.lexicon();
    let lemma = if lexicon.contains_lemma(lemma) {
        lemma.to_owned()
    } else {
        lemma.to_lowercase()
    };
    let tokens = tokenize(written, studied, lexicon);
    let alone = alone_in_capitals(written, &tokens, studied);
    let piece = tokens
        .iter()
        .zip(alone)
        .find(|(token, alone)| resolve_lemmas(token, *alone, studied, lexicon).0 == lemma)
        .map(|(token, _)| token)
        .or_else(|| tokens.first())
        .map(|token| token.text.replace('\u{2019}', "'").to_lowercase())
        .unwrap_or_else(|| without_soft_hyphens(written).trim().to_lowercase());
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
    use crate::knowledge::level::CefrLevel;
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
        build_pack_with_levels(
            studied,
            forms,
            lemmas,
            ranks,
            glosses,
            expressions,
            names,
            lexical,
            &[],
        )
    }

    /// [`build_pack_with_names`] plus, when `levels` holds any, the level table: each
    /// lemma's CEFR level, as the builder writes a pack's `level.tsv`.
    #[allow(clippy::too_many_arguments)]
    fn build_pack_with_levels(
        studied: StudiedLanguage,
        forms: &[(&str, &str)],
        lemmas: &[&str],
        ranks: &[(&str, u32)],
        glosses: &[(&str, &str)],
        expressions: &[(&str, &str)],
        names: &[(&str, &str)],
        lexical: Option<&[&str]>,
        levels: &[(&str, CefrLevel)],
    ) -> Pack {
        build_pack_with_readings(
            studied,
            forms,
            lemmas,
            ranks,
            glosses,
            expressions,
            names,
            lexical,
            levels,
            &[],
        )
    }

    /// [`build_pack_with_levels`] plus, when `readings` holds any, the grammar tables
    /// ([`grammar_blobs`]), as [`build_pack_with_grammar_for`] lays them out.
    #[allow(clippy::too_many_arguments)]
    fn build_pack_with_readings(
        studied: StudiedLanguage,
        forms: &[(&str, &str)],
        lemmas: &[&str],
        ranks: &[(&str, u32)],
        glosses: &[(&str, &str)],
        expressions: &[(&str, &str)],
        names: &[(&str, &str)],
        lexical: Option<&[&str]>,
        levels: &[(&str, CefrLevel)],
        readings: &[Reading],
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
        let mut level_codes = vec![0u8; lex.lemma_count()];
        for (lemma, level) in levels {
            level_codes[lex.id_of(lemma).expect("levelled lemma listed") as usize] =
                level.to_code();
        }
        if !levels.is_empty() {
            sections.push((section::LEVELS, &level_codes));
        }
        let [tag_pool, paradigm_zst, runs_zst] = grammar_blobs(&lex, readings, &[], &[]);
        if !readings.is_empty() {
            sections.push((section::TAGS, &tag_pool));
            sections.push((section::PARADIGMS_ZST, &paradigm_zst));
            sections.push((section::SENSES_ZST, &runs_zst));
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
        // English reads the same rule since leave-lingua-names-out-of-english-and-spanish-words
        // (D4): `augusto`, a lemma of the pack and none of its dictionary words, is set aside.
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
            &english_page("The man met Augusto at the market, and the man met Augusto again."),
            EN,
            &english,
            &KnowledgeState::new(),
        );
        assert!(page.analysable);
        let augusto: Vec<TokenClass> = page
            .tokens
            .iter()
            .filter(|t| t.surface == "Augusto")
            .map(|t| t.class)
            .collect();
        assert_eq!(augusto, [NAME, NAME]);
    }

    /// The classes of every token written `surface` on an English page, for a reader
    /// with no history.
    fn english_classes(pack: &Pack, blocks: &[&str], surface: &str) -> Vec<TokenClass> {
        let page = analyse_page(blocks, EN, pack, &KnowledgeState::new());
        assert!(page.analysable, "{blocks:?}");
        page.tokens
            .into_iter()
            .filter(|token| token.surface == surface)
            .map(|token| token.class)
            .collect()
    }

    /// An en→fr pack for English's names rule: `lemmas` in the lexicon, `words` its
    /// dictionary words (a lexical table, as en-fr's since
    /// leave-lingua-names-out-of-english-and-spanish-words), `levels` its CEFR levels.
    fn english_names_pack(lemmas: &[&str], words: &[&str], levels: &[(&str, CefrLevel)]) -> Pack {
        let glosses: Vec<(&str, &str)> = lemmas.iter().map(|lemma| (*lemma, "glose")).collect();
        build_pack_with_levels(
            EN,
            &[],
            lemmas,
            &[],
            &glosses,
            &[],
            &[],
            Some(words),
            levels,
        )
    }

    /// A long-enough English page around `sentence`, so the page is analysable.
    fn english_page(sentence: &str) -> [&str; 2] {
        [
            sentence,
            "The rest of the evening passed quietly, and nobody in the house said a word about it.",
        ]
    }

    #[test]
    fn spec_scenario_an_english_character_of_a_story() {
        let blocks = english_page(
            "When the train pulled into the station, Margaret grabbed her brother's coat and ran.",
        );
        // `margaret` is glossed (« Marguerite ») but no dictionary word: a name alone.
        let pack = english_names_pack(
            &["margaret", "train", "station"],
            &["train", "station"],
            &[],
        );
        assert_eq!(pack.gloss("margaret"), Some("glose"));
        assert_eq!(english_classes(&pack, &blocks, "Margaret"), [NAME]);
        let page = analyse_page(&blocks, EN, &pack, &KnowledgeState::new());
        assert_eq!(page.analyzer_version, ANALYZER_VERSION);
        // Not underlined, and out of the percentage.
        let without = analyse_page(
            &blocks,
            EN,
            &english_names_pack(
                &["margaret", "train", "station"],
                &["margaret", "train", "station"],
                &[],
            ),
            &KnowledgeState::new(),
        );
        assert_eq!(page.counted + 1, without.counted);
    }

    #[test]
    fn spec_scenario_an_english_city() {
        let blocks =
            english_page("We flew from Paris to London. London was cold and wet all week.");
        let pack = english_names_pack(&["london", "paris", "cold"], &["cold"], &[]);
        assert_eq!(english_classes(&pack, &blocks, "London"), [NAME, NAME]);
    }

    #[test]
    fn spec_scenario_the_pronoun_i() {
        // `I'll` follows a word and `I'm` a comma: both capitalised in mid-sentence, neither a
        // dictionary word, and neither a name. `Mum` is a word of the pack.
        let blocks = english_page("Tell Mum I'll call tonight, and I'm sure she knows already.");
        let pack = english_names_pack(&["i'll", "i'm", "i", "mum"], &["i", "mum"], &[]);
        assert_eq!(english_classes(&pack, &blocks, "I'll"), [WORD]);
        assert_eq!(english_classes(&pack, &blocks, "I'm"), [WORD]);
        // A typographic apostrophe, and the pronoun alone, give no evidence either.
        assert!(is_pronoun_i("i\u{2019}d"));
        assert!(is_pronoun_i("i've"));
        assert!(is_pronoun_i("i"));
        assert!(!is_pronoun_i("ian"));
        assert!(!is_pronoun_i("it's"));
        // The tokeniser reads `I’d` as `I'd`; the pronoun alone is no name either.
        let blocks =
            english_page("Then I\u{2019}d say that I would rather walk home alone tonight.");
        let pack = english_names_pack(&["i'd", "i"], &[], &[]);
        assert_eq!(english_classes(&pack, &blocks, "I'd"), [WORD]);
        assert_eq!(english_classes(&pack, &blocks, "I"), [WORD]);
    }

    #[test]
    fn spec_scenario_a_word_a_cefr_list_levels() {
        let blocks = english_page("The lanes open in June, said Mr Brown to the eager swimmers.");
        // `june` is glossed as a name alone and `mr` not at all; both carry a level.
        let levels = [("june", CefrLevel::A1), ("mr", CefrLevel::A1)];
        let pack = english_names_pack(&["june", "mr", "brown"], &["brown"], &levels);
        assert_eq!(english_classes(&pack, &blocks, "June"), [WORD]);
        assert_eq!(english_classes(&pack, &blocks, "Mr"), [WORD]);
        // Without their levels, both would be names.
        let unlevelled = english_names_pack(&["june", "mr", "brown"], &["brown"], &[]);
        assert_eq!(english_classes(&unlevelled, &blocks, "June"), [NAME]);
        assert_eq!(english_classes(&unlevelled, &blocks, "Mr"), [NAME]);
    }

    #[test]
    fn spec_scenario_an_english_dictionary_word() {
        let blocks =
            english_page("Everyone at the table turned when Bill finally stood up to speak.");
        let pack = english_names_pack(&["bill", "table"], &["bill", "table"], &[]);
        assert_eq!(english_classes(&pack, &blocks, "Bill"), [WORD]);
        // Its card still has its gloss.
        let page = analyse_page(&blocks, EN, &pack, &KnowledgeState::new());
        let bill = page
            .tokens
            .iter()
            .find(|t| t.surface == "Bill")
            .expect("Bill");
        assert_eq!(bill.gloss.as_deref(), Some("glose"));
    }

    #[test]
    fn spec_scenario_english_capitals_at_the_head_of_sentences_only() {
        let blocks = english_page("Margaret ran. Margaret stopped at the gate and looked back.");
        let pack = english_names_pack(&["margaret"], &[], &[]);
        assert_eq!(english_classes(&pack, &blocks, "Margaret"), [WORD, WORD]);
    }

    #[test]
    fn spec_scenario_the_same_english_form_as_a_word() {
        let blocks = [
            "Nobody answered the door, said Sam to the others waiting outside in the rain.",
            "The account was registered under the user name sam on the old forum years ago.",
        ];
        let pack = english_names_pack(&["sam"], &[], &[]);
        assert_eq!(english_classes(&pack, &blocks, "Sam"), [WORD]);
    }

    #[test]
    fn a_spanish_document_still_sets_aside_a_levelled_lemma_that_is_no_dictionary_word() {
        // English's two readings are English's alone (D4): a Spanish capital after an apostrophe
        // and a Spanish level say nothing, and Spanish's `yo` is no pronoun I.
        let blocks = [
            "Cuando llegó el tren, Junio cogió el abrigo de su hermano y salió corriendo.",
            "Todos los nobles de la corte miraban la escena con mucha atención.",
        ];
        let pack = build_pack_with_levels(
            ES,
            &[],
            &["junio", "tren", "abrigo"],
            &[],
            &[],
            &[],
            &[],
            Some(&["tren", "abrigo"]),
            &[("junio", CefrLevel::A1)],
        );
        assert_eq!(spanish_classes(&pack, &blocks, "Junio"), [NAME]);
        // And a French one too.
        let blocks = [
            "Quand le train est arrivé, Juin a pris le manteau de son frère et il est parti.",
            "Tous les nobles de la cour regardaient la scène avec beaucoup d'attention ce soir.",
        ];
        let pack = build_pack_with_levels(
            FR,
            &[],
            &["juin", "train", "manteau"],
            &[],
            &[],
            &[],
            &[],
            Some(&["train", "manteau"]),
            &[("juin", CefrLevel::A1)],
        );
        assert_eq!(french_classes(&pack, &blocks, "Juin"), [NAME]);
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
        // English keys are their lemmas, a determiner too; Spanish writes its own
        // determiners (add-lingua-spanish-expression-keys D2), never French's.
        assert_eq!(expression_piece("The", "the", EN), "the");
        assert_eq!(expression_piece("la", "el", ES), "la");
        assert_eq!(expression_piece("cette", "ce", ES), "ce");
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

    // --- French elided pieces (match-lingua-french-elided-pieces) ---

    const ELIDED_FORMS: &[(&str, &str)] = &[
        ("a", "avoir"),
        ("amis", "ami"),
        ("attend", "attendre"),
        ("décidé", "décider"),
        ("est", "être"),
        ("pleut", "pleuvoir"),
        ("sont", "être"),
        ("était", "être"),
    ];

    const ELIDED_LEMMAS: &[&str] = &[
        "de", "le", "un", "peu", "plus", "parce", "que", "être", "ce", "il", "avoir", "décider",
        "faire", "eau", "hiver", "pleuvoir", "attendre", "mon", "ami", "je",
    ];

    /// The delta's expressions: the elided article, fr-en's `de un` (« first, first up »), and
    /// expressions written in full, ending on a piece French elides, or elided at their head.
    const ELIDED_EXPRESSIONS: &[(&str, &str)] = &[
        ("de l'", "some"),
        ("de un", "first, first up"),
        ("un peu", "a little"),
        ("parce que", "because"),
        ("qu'est-ce que", "what"),
        ("c'est", "it is"),
    ];

    fn elided_lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) = build_lexicon_blobs(ELIDED_FORMS, ELIDED_LEMMAS).expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    /// A French pack holding [`ELIDED_EXPRESSIONS`], keyed as the builder keys them and named by
    /// their headwords where the two differ (add-lingua-french-expression-keys D1, D3).
    fn elided_pieces_pack() -> Pack {
        let lexicon = elided_lexicon();
        let keyed: Vec<(String, &str, &str)> = ELIDED_EXPRESSIONS
            .iter()
            .map(|&(headword, gloss)| {
                let key = french_expression_key(headword, &lexicon).expect(headword);
                (key, headword, gloss)
            })
            .collect();
        let expressions: Vec<(&str, &str)> = keyed
            .iter()
            .map(|(key, _, gloss)| (key.as_str(), *gloss))
            .collect();
        let names: Vec<(&str, &str)> = keyed
            .iter()
            .filter(|(key, headword, _)| key != headword)
            .map(|(key, headword, _)| (key.as_str(), *headword))
            .collect();
        build_pack_with_names(
            FR,
            ELIDED_FORMS,
            ELIDED_LEMMAS,
            &[],
            &[],
            &expressions,
            &names,
            None,
        )
    }

    /// Each match of `text` on [`elided_pieces_pack`]: the surfaces of the tokens it covers and
    /// what it reports.
    fn elided_matches(text: &str) -> Vec<(Vec<String>, String)> {
        let phrase = gloss_phrase(text, FR, &elided_pieces_pack(), &KnowledgeState::new());
        phrase
            .expressions
            .iter()
            .map(|m| {
                let covered = phrase.tokens[m.start..m.end]
                    .iter()
                    .map(|token| token.surface.clone())
                    .collect();
                (covered, m.key.clone())
            })
            .collect()
    }

    fn covering(surfaces: &[&str], key: &str) -> (Vec<String>, String) {
        (
            surfaces.iter().map(|&surface| surface.to_owned()).collect(),
            key.to_owned(),
        )
    }

    #[test]
    fn the_delta_s_expressions_are_keyed_and_named_as_french_is_read() {
        let pack = elided_pieces_pack();
        for (key, name) in [
            ("de le", Some("de l'")),
            ("de un", None),
            ("un peu", None),
            ("parce que", None),
            ("que être ce que", Some("qu'est-ce que")),
            ("ce être", Some("c'est")),
        ] {
            assert!(pack.expression(key).is_some(), "{key}");
            assert_eq!(pack.expression_name(key), name, "{key}");
        }
    }

    #[test]
    fn spec_scenario_an_elided_article_is_not_a_pronoun() {
        // `de le` is `de l'`'s key; « le » here is a pronoun, written in full.
        assert_eq!(elided_matches("Il a décidé de le faire"), []);
    }

    #[test]
    fn spec_scenario_the_elided_article() {
        assert_eq!(
            elided_matches("de l\u{2019}eau"),
            [covering(&["de", "le"], "de l'")]
        );
        assert_eq!(
            elided_matches("de l'eau"),
            [covering(&["de", "le"], "de l'")]
        );
    }

    #[test]
    fn spec_scenario_a_piece_written_in_full_does_not_meet_its_elision() {
        assert_eq!(elided_matches("d\u{2019}un hiver"), []);
        // Written in full, it meets « de un » as before.
        assert_eq!(
            elided_matches("de un hiver"),
            [covering(&["de", "un"], "de un")]
        );
    }

    #[test]
    fn spec_scenario_a_shorter_run_takes_the_place() {
        assert_eq!(
            elided_matches("d\u{2019}un peu plus"),
            [covering(&["un", "peu"], "un peu")]
        );
    }

    #[test]
    fn spec_scenario_the_last_piece_follows_the_page() {
        assert_eq!(
            elided_matches("parce qu\u{2019}il pleut"),
            [covering(&["parce", "que"], "parce que")]
        );
        assert_eq!(
            elided_matches("parce que je"),
            [covering(&["parce", "que"], "parce que")]
        );
    }

    #[test]
    fn spec_scenario_an_elided_piece_inside_a_question() {
        assert_eq!(
            elided_matches("Qu\u{2019}est-ce qu\u{2019}il attend"),
            [covering(&["Que", "est", "ce", "que"], "qu'est-ce que")]
        );
        assert_eq!(
            elided_matches("Qu'est-ce que je"),
            [covering(&["Que", "est", "ce", "que"], "qu'est-ce que")]
        );
    }

    #[test]
    fn spec_scenario_another_form_written_in_full() {
        assert_eq!(elided_matches("Ce sont mes amis"), []);
        assert_eq!(
            elided_matches("C\u{2019}était l\u{2019}hiver"),
            [covering(&["Ce", "était"], "c'est")]
        );
    }

    #[test]
    fn a_french_headword_is_read_only_when_it_holds_an_apostrophe() {
        let lexicon = elided_lexicon();
        // `de un`, unnamed: its own headword, written in full. Its pieces but the last meet only
        // tokens written in full; its last meets either.
        assert!(!meets_as_written(
            &[true, false],
            &[false, false],
            "de un",
            &lexicon
        ));
        assert!(meets_as_written(
            &[false, false],
            &[false, false],
            "de un",
            &lexicon
        ));
        assert!(meets_as_written(
            &[false, true],
            &[false, false],
            "de un",
            &lexicon
        ));
        // A headword holding an apostrophe, straight or typographic, is read: its elided last
        // piece meets only an elided token.
        for name in ["de l'", "de l\u{2019}"] {
            assert!(
                meets_as_written(&[false, true], &[false, false], name, &lexicon),
                "{name}"
            );
            assert!(
                !meets_as_written(&[false, false], &[false, false], name, &lexicon),
                "{name}"
            );
            assert!(
                !meets_as_written(&[true, true], &[false, false], name, &lexicon),
                "{name}"
            );
        }
        // A reading that does not line up with the run, or none at all — a word the analysis
        // drops (`t`) —, leaves the run matched as before.
        assert!(meets_as_written(
            &[false, false],
            &[false, false],
            "de l'eau",
            &lexicon
        ));
        assert!(meets_as_written(
            &[true, true],
            &[false, false],
            "qu'il t",
            &lexicon
        ));
    }

    #[test]
    fn the_tokens_do_not_move_with_the_rule() {
        let bare = build_pack_for(FR, ELIDED_FORMS, ELIDED_LEMMAS, &[], &[], &[]);
        let knowledge = KnowledgeState::new();
        for text in [
            "Il a décidé de le faire",
            "de l\u{2019}eau",
            "d\u{2019}un peu plus",
            "Ce sont mes amis",
        ] {
            let with_table = gloss_phrase(text, FR, &elided_pieces_pack(), &knowledge);
            let without = gloss_phrase(text, FR, &bare, &knowledge);
            assert_eq!(with_table.tokens, without.tokens, "{text}");
        }
    }

    #[test]
    fn english_and_spanish_runs_are_not_held_to_elision() {
        // D6: the check is French's. A name holding an apostrophe, which French would read as an
        // elided piece, leaves an English or Spanish match where it was.
        for studied in [EN, ES] {
            let pack = build_pack_with_names(
                studied,
                &[],
                &["de", "un"],
                &[],
                &[],
                &[("de un", "x")],
                &[("de un", "d'un")],
                None,
            );
            let phrase = gloss_phrase("de un", studied, &pack, &KnowledgeState::new());
            assert_eq!(
                phrase
                    .expressions
                    .iter()
                    .map(|m| (m.start, m.end, m.key.as_str()))
                    .collect::<Vec<_>>(),
                [(0, 2, "d'un")],
                "{studied:?}"
            );
        }
    }

    // --- French contracted pieces (match-lingua-french-contracted-pieces) ---

    const CONTRACTED_FORMS: &[(&str, &str)] = &[
        ("armé", "armer"),
        ("armés", "armer"),
        ("dents", "dent"),
        ("est", "être"),
        ("fait", "faire"),
        ("les", "le"),
        ("miennes", "mien"),
        ("viens", "venir"),
    ];

    const CONTRACTED_LEMMAS: &[&str] = &[
        "il", "être", "prêt", "à", "le", "faire", "tu", "venir", "grâce", "soleil", "mien",
        "armer", "jusque", "dent", "revoir",
    ];

    /// The delta's expressions, as fr-en's tables write them: « au » at the head (`au fait`, keyed
    /// `à le faire` as « prêt à le faire » reads), « aux » after an elision (`armé jusqu'aux
    /// dents`), a last `à` (`grâce à`, `jusqu'à`) and a first article (`les miennes`).
    const CONTRACTED_EXPRESSIONS: &[(&str, &str)] = &[
        ("au fait", "by the way; informed"),
        ("grâce à", "thanks to"),
        ("les miennes", "mine"),
        ("armé jusqu'aux dents", "armed to the teeth"),
        ("jusqu'à", "until, up to"),
    ];

    fn contracted_lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) =
            build_lexicon_blobs(CONTRACTED_FORMS, CONTRACTED_LEMMAS).expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    /// A French pack holding `expressions`, keyed as the builder keys them and named by their
    /// headwords where the two differ (add-lingua-french-expression-keys D1, D3).
    fn contracted_pieces_pack(expressions: &[(&str, &str)]) -> Pack {
        let lexicon = contracted_lexicon();
        let keyed: Vec<(String, &str, &str)> = expressions
            .iter()
            .map(|&(headword, gloss)| {
                let key = french_expression_key(headword, &lexicon).expect(headword);
                (key, headword, gloss)
            })
            .collect();
        let table: Vec<(&str, &str)> = keyed
            .iter()
            .map(|(key, _, gloss)| (key.as_str(), *gloss))
            .collect();
        let names: Vec<(&str, &str)> = keyed
            .iter()
            .filter(|(key, headword, _)| key != headword)
            .map(|(key, headword, _)| (key.as_str(), *headword))
            .collect();
        build_pack_with_names(
            FR,
            CONTRACTED_FORMS,
            CONTRACTED_LEMMAS,
            &[],
            &[],
            &table,
            &names,
            None,
        )
    }

    /// Each match of `text` on `pack`: the surfaces of the tokens it covers and what it reports.
    fn matches_on(pack: &Pack, text: &str) -> Vec<(Vec<String>, String)> {
        let phrase = gloss_phrase(text, FR, pack, &KnowledgeState::new());
        phrase
            .expressions
            .iter()
            .map(|m| {
                let covered = phrase.tokens[m.start..m.end]
                    .iter()
                    .map(|token| token.surface.clone())
                    .collect();
                (covered, m.key.clone())
            })
            .collect()
    }

    fn contracted_matches(text: &str) -> Vec<(Vec<String>, String)> {
        matches_on(&contracted_pieces_pack(CONTRACTED_EXPRESSIONS), text)
    }

    #[test]
    fn the_contracted_expressions_are_keyed_and_named_as_french_is_read() {
        let pack = contracted_pieces_pack(CONTRACTED_EXPRESSIONS);
        for (key, name) in [
            ("à le faire", Some("au fait")),
            ("grâce à", None),
            ("les mien", Some("les miennes")),
            ("armer jusque à les dent", Some("armé jusqu'aux dents")),
            ("jusque à", Some("jusqu'à")),
        ] {
            assert!(pack.expression(key).is_some(), "{key}");
            assert_eq!(pack.expression_name(key), name, "{key}");
        }
    }

    #[test]
    fn spec_scenario_a_pronoun_after_a_is_not_a_contraction() {
        // `à le faire` is `au fait`'s key; « le » here is a pronoun, written apart from « à ».
        assert_eq!(contracted_matches("Il est prêt à le faire"), []);
    }

    #[test]
    fn spec_scenario_the_contraction() {
        assert_eq!(
            contracted_matches("Au fait, tu viens\u{202f}?"),
            [covering(&["À", "le", "fait"], "au fait")]
        );
    }

    #[test]
    fn spec_scenario_pieces_written_apart_do_not_meet_a_contraction() {
        // A headword written `à le` apart, which no committed table holds, and no `au revoir`.
        let pack = contracted_pieces_pack(&[("à le revoir", "x")]);
        assert_eq!(pack.expression_name("à le revoir"), None);
        assert_eq!(matches_on(&pack, "au revoir"), []);
        assert_eq!(
            matches_on(&pack, "à le revoir"),
            [covering(&["à", "le", "revoir"], "à le revoir")]
        );
    }

    #[test]
    fn spec_scenario_the_last_a_before_the_page_s_article() {
        // The run ends on the `à` of « au »; its article is the page's, then covered
        // (add-lingua-french-word-card D12).
        assert_eq!(
            contracted_matches("grâce au soleil"),
            [covering(&["grâce", "à", "le"], "grâce à")]
        );
    }

    #[test]
    fn spec_scenario_the_first_article_after_the_page_s_a() {
        // The run starts on the article of « aux »; its `à` is the page's.
        assert_eq!(
            contracted_matches("aux miennes"),
            [covering(&["les", "miennes"], "les miennes")]
        );
    }

    #[test]
    fn spec_scenario_an_elision_and_a_contraction_in_one_headword() {
        assert_eq!(
            contracted_matches("armés jusqu\u{2019}aux dents"),
            [covering(
                &["armés", "jusque", "à", "les", "dents"],
                "armé jusqu'aux dents"
            )]
        );
        // Written apart, the five tokens do not meet it, and the shorter run takes the place.
        assert_eq!(
            contracted_matches("armé jusqu'à les dents"),
            [covering(&["jusque", "à"], "jusqu'à")]
        );
    }

    #[test]
    fn the_spans_a_written_word_shares() {
        let lexicon = contracted_lexicon();
        let shared = |text: &str, studied| {
            shared_spans(&crate::analysis::tokenize::tokenize(
                text, studied, &lexicon,
            ))
        };
        assert_eq!(shared("jusqu'au soir", FR), [false, false, true, false]);
        assert_eq!(shared("Au fait", FR), [false, true, false]);
        assert_eq!(shared("prêt à le faire", FR), [false; 4]);
        assert_eq!(shared("aux miennes", FR), [false, true, false]);
        assert_eq!(shared("we don't", EN), [false, false, true]);
        assert_eq!(shared("después del", ES), [false, false, true]);
        assert!(shared("", FR).is_empty());
    }

    #[test]
    fn a_french_headword_holds_au_or_aux_as_a_word() {
        for headword in [
            "au fait",
            "Au revoir",
            "AUX AGUETS",
            "armé jusqu'aux dents",
            "jusqu\u{2019}au",
            "tirer au sort",
            "a\u{ad}u fait",
        ] {
            assert!(holds_french_contraction(headword), "{headword}");
        }
        for headword in [
            "à le revoir",
            "les miennes",
            "aucun",
            "tuyau",
            "de l'eau",
            "eaux",
            "",
        ] {
            assert!(!holds_french_contraction(headword), "{headword}");
        }
    }

    #[test]
    fn a_french_headword_s_contractions_are_read_only_when_it_holds_au_or_aux() {
        let lexicon = contracted_lexicon();
        // `grâce à`, unnamed: its own headword, every piece apart. It is not read: a run of three
        // that a reading would not line up with still meets only tokens written apart.
        assert!(meets_as_written(
            &[false; 2],
            &[false, false],
            "grâce à",
            &lexicon
        ));
        assert!(!meets_as_written(
            &[false; 2],
            &[false, true],
            "grâce à",
            &lexicon
        ));
        assert!(!meets_as_written(
            &[false; 3],
            &[false, false, true],
            "grâce à",
            &lexicon
        ));
        // The run's first place is the page's: its first token may share the span of the token
        // before the run.
        assert!(meets_as_written(
            &[false; 2],
            &[true, false],
            "les miennes",
            &lexicon
        ));
        // A headword holding « au », in any case, is read: its two pieces meet only a contraction.
        for name in ["au fait", "Au fait", "AU FAIT"] {
            assert!(
                meets_as_written(&[false; 3], &[false, true, false], name, &lexicon),
                "{name}"
            );
            assert!(
                !meets_as_written(&[false; 3], &[false; 3], name, &lexicon),
                "{name}"
            );
            assert!(
                !meets_as_written(&[false; 3], &[false, true, true], name, &lexicon),
                "{name}"
            );
        }
        // One reading holds both rules: an elided piece and a contraction.
        let name = "armé jusqu'aux dents";
        let elided = [false, true, false, false, false];
        assert!(meets_as_written(
            &elided,
            &[false, false, false, true, false],
            name,
            &lexicon
        ));
        assert!(!meets_as_written(&elided, &[false; 5], name, &lexicon));
        assert!(!meets_as_written(
            &[false; 5],
            &[false, false, false, true, false],
            name,
            &lexicon
        ));
        // A reading that does not line up with the run is matched as before, its contractions
        // unchecked; one holding no apostrophe still holds its pieces but the last written in
        // full.
        assert!(meets_as_written(
            &[false; 2],
            &[false, false],
            name,
            &lexicon
        ));
        assert!(meets_as_written(
            &[false; 2],
            &[false, false],
            "au fait",
            &lexicon
        ));
        assert!(!meets_as_written(
            &[true, false],
            &[false, false],
            "au fait",
            &lexicon
        ));
    }

    #[test]
    fn a_french_selection_s_tokens_do_not_move_with_the_contraction_rule() {
        let bare = build_pack_for(FR, CONTRACTED_FORMS, CONTRACTED_LEMMAS, &[], &[], &[]);
        let pack = contracted_pieces_pack(CONTRACTED_EXPRESSIONS);
        let knowledge = KnowledgeState::new();
        for text in [
            "Il est prêt à le faire",
            "Au fait, tu viens\u{202f}?",
            "grâce au soleil",
            "aux miennes",
            "armés jusqu\u{2019}aux dents",
        ] {
            let with_table = gloss_phrase(text, FR, &pack, &knowledge);
            let without = gloss_phrase(text, FR, &bare, &knowledge);
            assert_eq!(with_table.tokens, without.tokens, "{text}");
        }
    }

    #[test]
    fn english_and_spanish_runs_are_not_held_to_contractions() {
        // D6: the check is French's. English's « don't » shares its span between `do` and `not`,
        // and a key holding neither « au » nor « aux » meets it as before.
        let english = build_pack_for(EN, &[], &["i", "do", "not"], &[], &[], &[("do not", "x")]);
        let knowledge = KnowledgeState::new();
        for text in ["I don't", "I do not"] {
            let phrase = gloss_phrase(text, EN, &english, &knowledge);
            assert_eq!(spans(&phrase), [(1, 3, "do not")], "{text}");
        }
        // Spanish's « al », named, still meets « a el » written apart (« a El Tiempo »), and
        // « al » as before.
        let spanish = build_pack_with_names(
            ES,
            &[],
            &["a", "el", "tiempo"],
            &[],
            &[],
            &[("a el tiempo", "en même temps")],
            &[("a el tiempo", "al tiempo")],
            None,
        );
        for text in ["una entrevista a El Tiempo", "al tiempo"] {
            let phrase = gloss_phrase(text, ES, &spanish, &knowledge);
            let found: Vec<&str> = phrase.expressions.iter().map(|m| m.key.as_str()).collect();
            assert_eq!(found, ["al tiempo"], "{text}");
        }
    }

    // --- the headword written as the page (pick-lingua-expression-written-as-the-page) ---

    /// The words the shared keys below are made of: `ça va`, `ça ira` and `ça allait` read
    /// `ça aller`, `allez-y`, `allons-y` and `vas-y` `aller y`, `quel que fût`, `quel que soit`
    /// and « quelle que soit » `quel que être`.
    const WRITTEN_FORMS: &[(&str, &str)] = &[
        ("allait", "aller"),
        ("allez", "aller"),
        ("allons", "aller"),
        ("fût", "être"),
        ("ira", "aller"),
        ("la", "le"),
        ("quelle", "quel"),
        ("soit", "être"),
        ("va", "aller"),
        ("vas", "aller"),
    ];

    const WRITTEN_LEMMAS: &[&str] = &[
        "ça", "aller", "y", "quel", "que", "être", "le", "distance", "ce", "qui",
    ];

    /// A pack laid out as the builder lays out a key several headwords reach (D1): `entries`
    /// are (key, headword, gloss), listed in the builder's order; the first headword of a key
    /// wins it — its gloss under the key's id, its name where it is not the key — and each other
    /// is an expression of its own, named, under an id after the keys', listed in `expr.others`.
    fn pack_with_others(
        studied: StudiedLanguage,
        forms: &[(&str, &str)],
        lemmas: &[&str],
        entries: &[(&str, &str, &str)],
    ) -> Pack {
        let (forms, pool) = build_lexicon_blobs(forms, lemmas).expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let freq_bytes = vec![0u8; lex.lemma_count() * 4];
        let mut keys: Vec<&str> = entries.iter().map(|(key, _, _)| *key).collect();
        keys.sort_unstable();
        keys.dedup();
        let mut map = fst::MapBuilder::memory();
        let (mut glosses, mut names, mut others) = (Vec::new(), Vec::new(), Vec::new());
        let mut next = keys.len() as u32;
        for (id, key) in keys.iter().enumerate() {
            let id = id as u32;
            map.insert(key, u64::from(id)).expect("sorted, unique keys");
            let mut headwords = entries.iter().filter(|(k, _, _)| k == key);
            let (_, winner, gloss) = headwords.next().expect("a headword");
            glosses.push((id, *gloss));
            if winner != key {
                names.push((id, *winner));
            }
            for (_, other, gloss) in headwords {
                glosses.push((next, *gloss));
                names.push((next, *other));
                others.push((id, next));
                next += 1;
            }
        }
        let map = map.into_inner().expect("fst");
        let (glosses, names) = (build_gloss_zst(&glosses), build_gloss_zst(&names));
        let mut others_bytes = (others.len() as u32).to_le_bytes().to_vec();
        for (key, other) in &others {
            others_bytes.extend_from_slice(&key.to_le_bytes());
            others_bytes.extend_from_slice(&other.to_le_bytes());
        }
        let native = if studied == FR { "en" } else { "fr" };
        let meta = serde_json::to_vec(&PackMeta {
            studied: studied.tag().into(),
            native: native.into(),
            pack_version: "t".into(),
            analyzer_version: studied.analyzer_version().into(),
            levels_estimated: false,
            licences: vec![],
        })
        .unwrap();
        let sections: Vec<(&str, &[u8])> = vec![
            (section::FORMS, &forms),
            (section::LEMMAS, pool.as_bytes()),
            (section::FREQ, &freq_bytes),
            (section::EXPR, &map),
            (section::EXPR_ZST, &glosses),
            (section::EXPR_NAMES_ZST, &names),
            (section::EXPR_OTHERS, &others_bytes),
        ];
        Pack::load(&write_container(&meta, &sections)).expect("load")
    }

    /// A French pack holding `headwords` (headword, gloss), listed in the builder's order and
    /// keyed as the builder keys them (add-lingua-french-expression-keys D1).
    fn written_pack(headwords: &[(&str, &str)]) -> Pack {
        let (bytes, pool) = build_lexicon_blobs(WRITTEN_FORMS, WRITTEN_LEMMAS).expect("build");
        let lexicon = FstLexicon::from_slices(bytes, &pool).expect("load");
        let keys: Vec<String> = headwords
            .iter()
            .map(|(headword, _)| french_expression_key(headword, &lexicon).expect(headword))
            .collect();
        let entries: Vec<(&str, &str, &str)> = keys
            .iter()
            .zip(headwords)
            .map(|(key, (headword, gloss))| (key.as_str(), *headword, *gloss))
            .collect();
        pack_with_others(FR, WRITTEN_FORMS, WRITTEN_LEMMAS, &entries)
    }

    /// The keys several headwords reach, each won by its first as change 44's order ranks
    /// them: the one written as the key (`ce qui`), else byte order (`ça ira` before `ça va`,
    /// `allez-y` before `allons-y` and `vas-y`, `quel que fût` before `quel que soit`).
    const WRITTEN_HEADWORDS: &[(&str, &str)] = &[
        ("ça ira", "it will be all right"),
        ("ça va", "how are you?"),
        ("allez-y", "go ahead"),
        ("allons-y", "let's go"),
        ("vas-y", "go on"),
        ("quel que fût", "whatever it was"),
        ("quel que soit", "whatever"),
        ("ce qui", "what, which"),
        ("c'qui", "what, which (colloquial)"),
    ];

    /// Each match of `text` on [`WRITTEN_HEADWORDS`]'s pack: the surfaces it covers, its key
    /// and its gloss.
    fn written_matches(text: &str) -> Vec<(Vec<String>, String, String)> {
        let pack = written_pack(WRITTEN_HEADWORDS);
        let phrase = gloss_phrase(text, FR, &pack, &KnowledgeState::new());
        phrase
            .expressions
            .iter()
            .map(|m| {
                let covered = phrase.tokens[m.start..m.end]
                    .iter()
                    .map(|token| token.surface.clone())
                    .collect();
                (covered, m.key.clone(), m.gloss.clone())
            })
            .collect()
    }

    fn written(surfaces: &[&str], key: &str, gloss: &str) -> (Vec<String>, String, String) {
        (
            surfaces.iter().map(|s| (*s).to_owned()).collect(),
            key.to_owned(),
            gloss.to_owned(),
        )
    }

    #[test]
    fn the_shared_keys_are_laid_out_as_the_builder_lays_them_out() {
        let pack = written_pack(WRITTEN_HEADWORDS);
        let names = |key: &str| -> Vec<String> {
            pack.expression_headwords(key)
                .into_iter()
                .map(|(name, _)| name.to_owned())
                .collect()
        };
        assert_eq!(names("ça aller"), ["ça ira", "ça va"]);
        assert_eq!(names("aller y"), ["allez-y", "allons-y", "vas-y"]);
        assert_eq!(names("quel que être"), ["quel que fût", "quel que soit"]);
        assert_eq!(names("ce qui"), ["ce qui", "c'qui"]);
    }

    #[test]
    fn spec_scenario_the_headword_written_as_the_page() {
        assert_eq!(
            written_matches("\u{c7}a va\u{202f}?"),
            [written(&["\u{c7}a", "va"], "ça va", "how are you?")]
        );
        assert_eq!(
            written_matches("Comment ça va ?"),
            [written(&["ça", "va"], "ça va", "how are you?")]
        );
    }

    #[test]
    fn spec_scenario_the_headword_that_wins_the_key_written_as_the_page() {
        assert_eq!(
            written_matches("\u{c7}a ira."),
            [written(
                &["\u{c7}a", "ira"],
                "ça ira",
                "it will be all right"
            )]
        );
    }

    #[test]
    fn spec_scenario_one_of_three_headwords() {
        assert_eq!(
            written_matches("Allons-y\u{202f}!"),
            [written(&["Allons", "y"], "allons-y", "let's go")]
        );
        assert_eq!(
            written_matches("vas-y"),
            [written(&["vas", "y"], "vas-y", "go on")]
        );
        assert_eq!(
            written_matches("Allez-y"),
            [written(&["Allez", "y"], "allez-y", "go ahead")]
        );
    }

    #[test]
    fn spec_scenario_no_headword_written_as_the_page_one_closer() {
        // « Quelle » is written as neither headword's `quel`: `quel que soit` is written as the
        // page at two places, `quel que fût` at one.
        assert_eq!(
            written_matches("Quelle que soit la distance"),
            [written(
                &["Quelle", "que", "soit"],
                "quel que soit",
                "whatever"
            )]
        );
    }

    #[test]
    fn spec_scenario_no_headword_closer_than_another() {
        // `ça` alone is written as the page in both: the headword that wins the key answers.
        assert_eq!(
            written_matches("ça allait"),
            [written(&["ça", "allait"], "ça ira", "it will be all right")]
        );
    }

    #[test]
    fn spec_scenario_a_headword_that_does_not_meet_the_page() {
        // `ce qui` and `c'qui` read alike, and only `c'qui` meets the page's elided « c' »
        // (match-lingua-french-elided-pieces): the winner is no candidate.
        let found = written_matches("c'qui");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0.len(), 2);
        assert_eq!(found[0].1, "c'qui");
        assert_eq!(found[0].2, "what, which (colloquial)");
        // Written in full, both meet the page: the one written as it wins.
        assert_eq!(
            written_matches("ce qui"),
            [written(&["ce", "qui"], "ce qui", "what, which")]
        );
    }

    #[test]
    fn a_run_no_headword_meets_does_not_match_at_that_length() {
        // `ce qui` is written in full, so neither of the key's headwords meets « c'qui » when the
        // pack holds `ce qui` alone, and no shorter run is a key.
        let pack = written_pack(&[("ce qui", "what, which"), ("ça va", "how are you?")]);
        let phrase = gloss_phrase("c'qui", FR, &pack, &KnowledgeState::new());
        assert!(phrase.expressions.is_empty());
        let pack = pack_with_others(
            FR,
            WRITTEN_FORMS,
            WRITTEN_LEMMAS,
            &[
                ("ce qui", "ce qui", "what, which"),
                ("ce qui", "ce que", "a headword that reads elsewhere"),
            ],
        );
        let phrase = gloss_phrase("c'qui", FR, &pack, &KnowledgeState::new());
        assert!(phrase.expressions.is_empty(), "{:?}", phrase.expressions);
    }

    #[test]
    fn spec_scenario_a_key_one_headword_reaches() {
        let pack = written_pack(&[("ça va", "how are you?")]);
        let phrase = gloss_phrase("\u{c7}a ira.", FR, &pack, &KnowledgeState::new());
        assert_eq!(spans(&phrase), [(0, 2, "ça va")]);
        assert_eq!(phrase.expressions[0].gloss, "how are you?");
    }

    #[test]
    fn spec_scenario_the_status_follows_the_headword() {
        let pack = written_pack(WRITTEN_HEADWORDS);
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(FR, "ça va", Status::Known(KnownSource::Manual));
        let va = gloss_phrase("\u{c7}a va\u{202f}?", FR, &pack, &knowledge);
        assert_eq!(va.expressions[0].key, "ça va");
        assert_eq!(va.expressions[0].class, TokenClass::Known);
        let ira = gloss_phrase("\u{c7}a ira.", FR, &pack, &knowledge);
        assert_eq!(ira.expressions[0].key, "ça ira");
        assert_eq!(ira.expressions[0].class, TokenClass::Unknown);
    }

    #[test]
    fn a_french_selection_s_tokens_do_not_move_with_the_headwords() {
        let pack = written_pack(WRITTEN_HEADWORDS);
        let bare = build_pack_for(FR, WRITTEN_FORMS, WRITTEN_LEMMAS, &[], &[], &[]);
        let knowledge = KnowledgeState::new();
        for text in [
            "\u{c7}a va\u{202f}?",
            "Allons-y\u{202f}!",
            "Quelle que soit la distance",
        ] {
            assert_eq!(
                gloss_phrase(text, FR, &pack, &knowledge).tokens,
                gloss_phrase(text, FR, &bare, &knowledge).tokens,
                "{text}"
            );
        }
    }

    #[test]
    fn english_runs_are_not_named_by_another_headword_and_spanish_ones_are() {
        // The choice is French's and Spanish's (pick-lingua-spanish-expression-written-as-the-
        // page D2, D7). An English pack carrying the section — which the builder never writes for
        // it — is matched as before, every run named by the headword that wins its key.
        let knowledge = KnowledgeState::new();
        let english = pack_with_others(
            EN,
            &[("starting", "start")],
            &["start", "point"],
            &[
                ("start point", "start point", "Point de départ"),
                ("start point", "starting point", "Point de départ (2)"),
            ],
        );
        let phrase = gloss_phrase("the starting point", EN, &english, &knowledge);
        assert_eq!(spans(&phrase), [(1, 3, "start point")]);
        assert_eq!(phrase.expressions[0].gloss, "Point de départ");
        // A Spanish run is named by the headword written as the page.
        let spanish = pack_with_others(
            ES,
            SPANISH_FORMS,
            SPANISH_LEMMAS,
            &[
                ("ser inglés", "soy inglés", "Je suis anglais"),
                ("ser inglés", "soy inglesa", "Je suis anglaise"),
                ("después de", "después de", "Après"),
            ],
        );
        let phrase = gloss_phrase("soy inglesa", ES, &spanish, &knowledge);
        assert_eq!(spans(&phrase), [(0, 2, "soy inglesa")]);
        assert_eq!(phrase.expressions[0].gloss, "Je suis anglaise");
        let phrase = gloss_phrase("soy inglés", ES, &spanish, &knowledge);
        assert_eq!(spans(&phrase), [(0, 2, "soy inglés")]);
        assert_eq!(phrase.expressions[0].gloss, "Je suis anglais");
        let phrase = gloss_phrase("después del", ES, &spanish, &knowledge);
        assert_eq!(spans(&phrase), [(0, 3, "después de")]);
        assert_eq!(phrase.expressions[0].gloss, "Après");
        // And on the packs the scenarios above read, as before.
        assert_eq!(
            spanish_spans("después del barrio"),
            [span(0, 3, "después de")]
        );
    }

    // --- the Spanish headword written as the page (pick-lingua-spanish-expression-written-as-the-page) ---

    /// The words the shared Spanish keys below are made of: `hace tiempo` reads `hacer tiempo`,
    /// `dar vueltas` `dar vuelta`, `tenerlos puestos` `tener puesto` (the enclitic rule reading
    /// `tenerlos`), `hacerse ilusiones` `hacer ilusión`, `puede ayudarme` and `puedes ayudarme`
    /// `poder ayudar`, `buen día` and `buenos días` `bueno día`, « un poquito » `un poco`.
    const ES_WRITTEN_FORMS: &[(&str, &str)] = &[
        ("buen", "bueno"),
        ("buenos", "bueno"),
        ("dando", "dar"),
        ("días", "día"),
        ("hace", "hacer"),
        ("hacerse", "hacer"),
        ("hagas", "hacer"),
        ("ilusiones", "ilusión"),
        ("poquito", "poco"),
        ("puede", "poder"),
        ("puedes", "poder"),
        ("puestos", "puesto"),
        ("tenía", "tener"),
        ("vueltas", "vuelta"),
    ];

    const ES_WRITTEN_LEMMAS: &[&str] = &[
        "hacer", "tiempo", "dar", "vuelta", "tener", "puesto", "ilusión", "poder", "ayudar",
        "bueno", "día", "un", "poco",
    ];

    /// The keys several Spanish headwords reach, each won by its first as change 44's order
    /// ranks them (add-lingua-spanish-expression-keys D3): the one written as the key
    /// (`hacer tiempo`, `dar vuelta`, `tener puesto`, `hacer ilusión`), else the most written
    /// (`buen día`), else byte order (`puede ayudarme`).
    const ES_WRITTEN_HEADWORDS: &[(&str, &str)] = &[
        ("hacer tiempo", "Atermoyer"),
        ("hace tiempo", "Jadis, belle lurette"),
        ("dar vuelta", "to turn"),
        ("dar vueltas", "to go round in circles"),
        ("tener puesto", "to have on; to wear"),
        ("tenerlos puestos", "to have balls"),
        ("hacer ilusión", "to cause to get excited about something"),
        ("hacerse ilusiones", "to get one's hopes up"),
        ("puede ayudarme", "can you help me? (formal)"),
        ("puedes ayudarme", "can you help me?"),
        ("buen día", "Bonjour"),
        ("buenos días", "Bonjour (pluriel)"),
    ];

    /// A Spanish pack holding `headwords` (headword, gloss), listed in the builder's order and
    /// keyed as the builder keys them (add-lingua-spanish-expression-keys D1).
    fn es_written_pack(headwords: &[(&str, &str)]) -> Pack {
        let (bytes, pool) =
            build_lexicon_blobs(ES_WRITTEN_FORMS, ES_WRITTEN_LEMMAS).expect("build");
        let lexicon = FstLexicon::from_slices(bytes, &pool).expect("load");
        let keys: Vec<String> = headwords
            .iter()
            .map(|(headword, _)| reading_expression_key(headword, ES, &lexicon).expect(headword))
            .collect();
        let entries: Vec<(&str, &str, &str)> = keys
            .iter()
            .zip(headwords)
            .map(|(key, (headword, gloss))| (key.as_str(), *headword, *gloss))
            .collect();
        pack_with_others(ES, ES_WRITTEN_FORMS, ES_WRITTEN_LEMMAS, &entries)
    }

    /// Each match of `text` on `pack`, for `knowledge`: the surfaces it covers, its key and its
    /// gloss.
    fn es_matches_on(
        text: &str,
        pack: &Pack,
        knowledge: &KnowledgeState,
    ) -> Vec<(Vec<String>, String, String)> {
        let phrase = gloss_phrase(text, ES, pack, knowledge);
        phrase
            .expressions
            .iter()
            .map(|m| {
                let covered = phrase.tokens[m.start..m.end]
                    .iter()
                    .map(|token| token.surface.clone())
                    .collect();
                (covered, m.key.clone(), m.gloss.clone())
            })
            .collect()
    }

    fn es_matches(text: &str) -> Vec<(Vec<String>, String, String)> {
        es_matches_on(
            text,
            &es_written_pack(ES_WRITTEN_HEADWORDS),
            &KnowledgeState::new(),
        )
    }

    #[test]
    fn the_spanish_clitic_pronouns_are_sorted_and_eleven() {
        assert_eq!(SPANISH_CLITIC_PRONOUNS.len(), 11);
        assert!(
            SPANISH_CLITIC_PRONOUNS
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
    }

    #[test]
    fn the_shared_spanish_keys_are_laid_out_as_the_builder_lays_them_out() {
        let pack = es_written_pack(ES_WRITTEN_HEADWORDS);
        let names = |key: &str| -> Vec<String> {
            pack.expression_headwords(key)
                .into_iter()
                .map(|(name, _)| name.to_owned())
                .collect()
        };
        assert_eq!(names("hacer tiempo"), ["hacer tiempo", "hace tiempo"]);
        assert_eq!(names("dar vuelta"), ["dar vuelta", "dar vueltas"]);
        assert_eq!(names("tener puesto"), ["tener puesto", "tenerlos puestos"]);
        assert_eq!(
            names("hacer ilusión"),
            ["hacer ilusión", "hacerse ilusiones"]
        );
        assert_eq!(names("poder ayudar"), ["puede ayudarme", "puedes ayudarme"]);
        assert_eq!(names("bueno día"), ["buen día", "buenos días"]);
    }

    #[test]
    fn spec_scenario_the_spanish_headword_written_as_the_page() {
        assert_eq!(
            es_matches("hace tiempo"),
            [written(
                &["hace", "tiempo"],
                "hace tiempo",
                "Jadis, belle lurette"
            )]
        );
        assert_eq!(
            es_matches("Hace tiempo que no te veo."),
            [written(
                &["Hace", "tiempo"],
                "hace tiempo",
                "Jadis, belle lurette"
            )]
        );
        assert_eq!(
            es_matches("buenos días"),
            [written(
                &["buenos", "días"],
                "buenos días",
                "Bonjour (pluriel)"
            )]
        );
    }

    #[test]
    fn spec_scenario_the_spanish_headword_that_wins_the_key_written_as_the_page() {
        assert_eq!(
            es_matches("hacer tiempo"),
            [written(&["hacer", "tiempo"], "hacer tiempo", "Atermoyer")]
        );
    }

    #[test]
    fn spec_scenario_no_spanish_headword_written_as_the_page_one_closer() {
        assert_eq!(
            es_matches("dando vueltas"),
            [written(
                &["dando", "vueltas"],
                "dar vueltas",
                "to go round in circles"
            )]
        );
    }

    #[test]
    fn spec_scenario_a_clitic_pronoun_the_page_does_not_write() {
        // `tenerlos puestos` is written as the page at `puestos`, `tener puesto` nowhere: the
        // guard keeps the headword written with a pronoun away from a page that writes none.
        assert_eq!(
            es_matches("El guardaespaldas tenía puestos unos audífonos."),
            [written(
                &["tenía", "puestos"],
                "tener puesto",
                "to have on; to wear"
            )]
        );
        // Without the guard — a pronoun on the page before the run — the closest spelling wins.
        assert_eq!(
            es_matches("Los tenía puestos."),
            [written(
                &["tenía", "puestos"],
                "tenerlos puestos",
                "to have balls"
            )]
        );
    }

    #[test]
    fn spec_scenario_a_clitic_pronoun_before_the_verb() {
        assert_eq!(
            es_matches("No te hagas ilusiones."),
            [written(
                &["hagas", "ilusiones"],
                "hacerse ilusiones",
                "to get one's hopes up"
            )]
        );
        // The same run, no pronoun before it: the headword written without one.
        assert_eq!(
            es_matches("No hagas ilusiones."),
            [written(
                &["hagas", "ilusiones"],
                "hacer ilusión",
                "to cause to get excited about something"
            )]
        );
    }

    #[test]
    fn spec_scenario_a_clitic_pronoun_attached_to_the_verb() {
        assert_eq!(
            es_matches("Me encanta hacerme ilusiones."),
            [written(
                &["hacerme", "ilusiones"],
                "hacerse ilusiones",
                "to get one's hopes up"
            )]
        );
    }

    #[test]
    fn spec_scenario_every_headword_writes_a_clitic_pronoun_the_page_none() {
        let text = "Ahora no puedes ayudar a Tom.";
        assert_eq!(
            es_matches(text),
            [written(
                &["puedes", "ayudar"],
                "puedes ayudarme",
                "can you help me?"
            )]
        );
        // Over the same tokens as the pack holding the key's winner alone.
        let alone = es_written_pack(&[("puede ayudarme", "can you help me? (formal)")]);
        let before = es_matches_on(text, &alone, &KnowledgeState::new());
        assert_eq!(before.len(), 1);
        assert_eq!(before[0].0, ["puedes", "ayudar"]);
        assert_eq!(before[0].1, "puede ayudarme");
    }

    #[test]
    fn spec_scenario_a_spanish_key_one_headword_reaches() {
        let pack = es_written_pack(&[("un poco", "Un peu")]);
        assert_eq!(
            es_matches_on("un poquito", &pack, &KnowledgeState::new()),
            [written(&["un", "poquito"], "un poco", "Un peu")]
        );
    }

    #[test]
    fn spec_scenario_the_spanish_status_follows_the_headword() {
        // A record on `hacer tiempo` is that headword's: the lemma chain of « hace tiempo » is
        // the name of a headword of its key, so it is no chain settled before (D4).
        let pack = es_written_pack(ES_WRITTEN_HEADWORDS);
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status_at(
            ES,
            "hacer tiempo",
            Status::Known(KnownSource::Manual),
            1_000,
        );
        let hacer = gloss_phrase("hacer tiempo", ES, &pack, &knowledge);
        assert_eq!(hacer.expressions[0].key, "hacer tiempo");
        assert_eq!(hacer.expressions[0].class, TokenClass::Known);
        let hace = gloss_phrase("hace tiempo", ES, &pack, &knowledge);
        assert_eq!(hace.expressions[0].key, "hace tiempo");
        assert_ne!(hace.expressions[0].class, TokenClass::Known);
        assert_eq!(hace.expressions[0].gloss, "Jadis, belle lurette");
    }

    #[test]
    fn spec_scenario_a_spanish_lemma_chain_settled_before() {
        // `bueno día` is no headword's name: a record on it is a chain settled before (44b's D6),
        // reported with the gloss of the headword the page writes.
        let pack = es_written_pack(ES_WRITTEN_HEADWORDS);
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status_at(ES, "bueno día", Status::Learning, 1_000);
        assert_eq!(
            es_matches_on("buenos días", &pack, &knowledge),
            [written(
                &["buenos", "días"],
                "bueno día",
                "Bonjour (pluriel)"
            )]
        );
        let phrase = gloss_phrase("buenos días", ES, &pack, &knowledge);
        assert_eq!(phrase.expressions[0].class, TokenClass::Learning);
        // A record on the name the page writes wins over the chain, as before.
        knowledge.set_status_at(ES, "buenos días", Status::Ignored, 2_000);
        let phrase = gloss_phrase("buenos días", ES, &pack, &knowledge);
        assert_eq!(phrase.expressions[0].key, "buenos días");
        assert_eq!(phrase.expressions[0].class, TokenClass::Ignored);
    }

    #[test]
    fn a_spanish_selection_s_tokens_do_not_move_with_the_headwords() {
        let pack = es_written_pack(ES_WRITTEN_HEADWORDS);
        let bare = build_pack_for(ES, ES_WRITTEN_FORMS, ES_WRITTEN_LEMMAS, &[], &[], &[]);
        let knowledge = KnowledgeState::new();
        for text in [
            "hace tiempo",
            "No te hagas ilusiones.",
            "El guardaespaldas tenía puestos unos audífonos.",
        ] {
            assert_eq!(
                gloss_phrase(text, ES, &pack, &knowledge).tokens,
                gloss_phrase(text, ES, &bare, &knowledge).tokens,
                "{text}"
            );
        }
    }

    #[test]
    fn english_and_french_selections_gloss_as_before_beside_the_spanish_rule() {
        // A French selection on 44e's inline pack is named as 44e names it.
        assert_eq!(
            written_matches("\u{c7}a va\u{202f}?"),
            [written(&["\u{c7}a", "va"], "ça va", "how are you?")]
        );
        // An English selection with a section a builder never writes for it, as before.
        let english = pack_with_others(
            EN,
            &[("starting", "start")],
            &["start", "point"],
            &[
                ("start point", "start point", "Point de départ"),
                ("start point", "starting point", "Point de départ (2)"),
            ],
        );
        let phrase = gloss_phrase("I'm starting", EN, &english, &KnowledgeState::new());
        assert!(phrase.expressions.is_empty());
    }

    // --- Spanish expression keys (add-lingua-spanish-expression-keys) ---

    #[test]
    fn the_words_a_spanish_key_writes_are_sorted_and_thirty_five() {
        assert_eq!(SPANISH_KEY_WRITTEN.len(), 35);
        assert!(SPANISH_KEY_WRITTEN.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn a_spanish_key_writes_its_determiners_and_lemmatises_the_rest() {
        // D2: the articles and the possessive and demonstrative determiners as the pre-pass
        // gives them, lowercased; every other token its dictionary form.
        assert_eq!(expression_piece("la", "el", ES), "la");
        assert_eq!(expression_piece("Los", "el", ES), "los");
        assert_eq!(expression_piece("una", "uno", ES), "una");
        assert_eq!(expression_piece("mis", "mi", ES), "mis");
        assert_eq!(expression_piece("Estas", "este", ES), "estas");
        // The `el` the pre-pass writes for « del » is written `el`.
        let split = crate::analysis::tokenize::tokenize("Del", ES, &spanish_lexicon());
        assert_eq!(split[1].text, "el");
        assert_eq!(expression_piece(&split[1].text, "el", ES), "el");
        assert_eq!(expression_piece("cuenta", "contar", ES), "contar");
        // A clitic pronoun is no determiner: its dictionary form.
        assert_eq!(expression_piece("se", "se", ES), "se");
        // English keys are their lemmas.
        assert_eq!(expression_piece("the", "the", EN), "the");
        assert_eq!(expression_piece("These", "this", EN), "this");
    }

    /// A Spanish lexicon for the keys and matches below: M8 files `la`, `los` and `las` under
    /// `el`, `cuenta` under `contar`, and `inglesa` under `inglés`.
    fn spanish_lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) = build_lexicon_blobs(SPANISH_FORMS, SPANISH_LEMMAS).expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    const SPANISH_FORMS: &[(&str, &str)] = &[
        ("cuenta", "contar"),
        ("la", "el"),
        ("las", "el"),
        ("los", "el"),
        ("armas", "arma"),
        ("soy", "ser"),
        ("inglesa", "inglés"),
    ];

    const SPANISH_LEMMAS: &[&str] = &[
        "a", "el", "menos", "tener", "en", "contar", "vez", "fin", "y", "cabo", "de", "barrio",
        "después", "arma", "ser", "inglés", "casa", "su", "abuela", "ir",
    ];

    #[test]
    fn a_spanish_expression_is_keyed_as_spanish_is_read() {
        let lexicon = spanish_lexicon();
        let key = |headword: &str| reading_expression_key(headword, ES, &lexicon);
        // A contraction is read `a` + `el`, its article written `el`.
        assert_eq!(key("al menos").as_deref(), Some("a el menos"));
        assert_eq!(key("del barrio").as_deref(), Some("de el barrio"));
        // An inflected word is its dictionary form.
        assert_eq!(key("tener en cuenta").as_deref(), Some("tener en contar"));
        // The determiners are written: `a la vez` is not `a el vez`, nor `a las armas` `al arma`.
        assert_eq!(key("a la vez").as_deref(), Some("a la vez"));
        assert_eq!(key("a las armas").as_deref(), Some("a las arma"));
        assert_eq!(key("al arma").as_deref(), Some("a el arma"));
        // Five words, seven tokens: within Spanish's window.
        assert_eq!(
            key("al fin y al cabo").as_deref(),
            Some("a el fin y a el cabo")
        );
        // A name is read as its headword is, so review finds the key from it.
        assert_eq!(key("Al menos").as_deref(), Some("a el menos"));
        // French's function is French's alone, and English has no reading key.
        assert_eq!(french_expression_key("al menos", &lexicon), None);
        assert_eq!(reading_expression_key("al menos", EN, &lexicon), None);
    }

    #[test]
    fn a_spanish_headword_outside_the_rules_has_no_key() {
        let lexicon = spanish_lexicon();
        let key = |headword: &str| reading_expression_key(headword, ES, &lexicon);
        // Seven tokens are Spanish's window; eight are beyond it (D4).
        assert_eq!(
            key(&["vez"; 7].join(" ")).as_deref(),
            Some(["vez"; 7].join(" ").as_str())
        );
        assert_eq!(key(&["vez"; 8].join(" ")), None);
        assert_eq!(key("al fin y al cabo de"), None, "six words, eight tokens");
        // One token is a word, not an expression — « al » alone is two tokens, a key; a
        // dictionary form the lexicon does not hold could never be met.
        assert_eq!(key("menos"), None);
        assert_eq!(key("al").as_deref(), Some("a el"));
        assert_eq!(key("tener en mente"), None);
        assert_eq!(key(""), None);
    }

    /// An es-fr pack holding the keys Spanish's reading makes, each named by its headword where
    /// it differs.
    fn spanish_expression_pack() -> Pack {
        build_pack_with_names(
            ES,
            SPANISH_FORMS,
            SPANISH_LEMMAS,
            &[],
            &[],
            &[
                ("a el menos", "Au moins"),
                ("tener en contar", "Tenir compte de"),
                ("a la vez", "À la fois"),
                ("de las", "Des"),
                ("de los", "Des"),
                ("después de", "Après"),
                ("a el fin y a el cabo", "Après tout"),
                ("a la", "À la"),
            ],
            &[
                ("a el menos", "al menos"),
                ("tener en contar", "tener en cuenta"),
                ("a el fin y a el cabo", "al fin y al cabo"),
            ],
            None,
        )
    }

    fn spanish_spans_for(text: &str, knowledge: &KnowledgeState) -> Vec<(usize, usize, String)> {
        gloss_phrase(text, ES, &spanish_expression_pack(), knowledge)
            .expressions
            .into_iter()
            .map(|m| (m.start, m.end, m.key))
            .collect()
    }

    fn spanish_spans(text: &str) -> Vec<(usize, usize, String)> {
        spanish_spans_for(text, &KnowledgeState::new())
    }

    #[test]
    fn spec_scenario_a_contraction_read_as_two_words() {
        let pack = spanish_expression_pack();
        let phrase = gloss_phrase("al menos", ES, &pack, &KnowledgeState::new());
        let surfaces: Vec<&str> = phrase.tokens.iter().map(|t| t.surface.as_str()).collect();
        assert_eq!(surfaces, ["a", "el", "menos"]);
        assert_eq!(
            phrase.expressions,
            [PhraseMatch {
                start: 0,
                end: 3,
                key: "al menos".into(),
                class: TokenClass::Unknown,
                gloss: "Au moins".into(),
            }]
        );
    }

    #[test]
    fn spec_scenario_the_name_not_the_lemmas() {
        assert_eq!(
            spanish_spans("tener en cuenta"),
            [span(0, 3, "tener en cuenta")]
        );
        // A key that is its own headword carries no name and is reported as itself.
        assert_eq!(spanish_spans("a la vez"), [span(0, 3, "a la vez")]);
    }

    #[test]
    fn spec_scenario_an_article_entry_does_not_answer_another_article() {
        assert_eq!(spanish_spans("del barrio"), []);
        // Nor does `a la` answer « al »; the article as written still answers its own entry.
        assert_eq!(spanish_spans("al barrio"), []);
        assert_eq!(spanish_spans("a la casa"), [span(0, 2, "a la")]);
        assert_eq!(spanish_spans("de las armas"), [span(0, 2, "de las")]);
    }

    #[test]
    fn spec_scenario_the_article_of_a_contraction_is_covered() {
        let pack = spanish_expression_pack();
        let phrase = gloss_phrase("después del", ES, &pack, &KnowledgeState::new());
        let surfaces: Vec<&str> = phrase.tokens.iter().map(|t| t.surface.as_str()).collect();
        assert_eq!(surfaces, ["después", "de", "el"]);
        assert_eq!(
            phrase.expressions,
            [PhraseMatch {
                start: 0,
                end: 3,
                key: "después de".into(),
                class: TokenClass::Unknown,
                gloss: "Après".into(),
            }]
        );
        // Inside a longer selection too, and the next run starts past the article.
        assert_eq!(
            spanish_spans("después del barrio"),
            [span(0, 3, "después de")]
        );
        // A match ending on a word of its own is not extended over the next word.
        assert_eq!(
            spanish_spans("después de la casa"),
            [span(0, 2, "después de")]
        );
    }

    #[test]
    fn spec_scenario_an_expression_of_seven_tokens() {
        let pack = spanish_expression_pack();
        let phrase = gloss_phrase("al fin y al cabo", ES, &pack, &KnowledgeState::new());
        let lemmas: Vec<&str> = phrase.tokens.iter().map(|t| t.lemma.as_str()).collect();
        assert_eq!(lemmas, ["a", "el", "fin", "y", "a", "el", "cabo"]);
        assert_eq!(
            spanish_spans("al fin y al cabo"),
            [span(0, 7, "al fin y al cabo")]
        );
    }

    #[test]
    fn spec_scenario_the_status_follows_a_spanish_name() {
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(ES, "tener en cuenta", Status::Known(KnownSource::Manual));
        let phrase = gloss_phrase(
            "tener en cuenta",
            ES,
            &spanish_expression_pack(),
            &knowledge,
        );
        assert_eq!(phrase.expressions[0].key, "tener en cuenta");
        assert_eq!(phrase.expressions[0].class, TokenClass::Known);
    }

    #[test]
    fn a_spanish_selection_s_tokens_do_not_move_with_the_table() {
        // The tokens are what they are without the table; the matches sit beside them.
        let bare = build_pack_for(ES, SPANISH_FORMS, SPANISH_LEMMAS, &[], &[], &[]);
        let knowledge = KnowledgeState::new();
        for text in [
            "al menos",
            "después del",
            "al fin y al cabo",
            "tener en cuenta",
        ] {
            let with_table = gloss_phrase(text, ES, &spanish_expression_pack(), &knowledge);
            let without = gloss_phrase(text, ES, &bare, &knowledge);
            assert_eq!(with_table.tokens, without.tokens, "{text}");
            assert!(without.expressions.is_empty());
            assert!(!with_table.expressions.is_empty(), "{text}");
        }
    }

    #[test]
    fn a_french_match_ending_on_the_a_of_au_or_aux_covers_its_article() {
        // add-lingua-french-word-card D12, the owner's answer to
        // add-lingua-spanish-expression-keys' open question 3: French's « au » and « aux » share
        // their span between `à` and `le`/`les` as « al » does, and a match ending on that `à`
        // covers the article too — no match ends inside a written word.
        let french = build_pack_with_names(
            FR,
            &[("murs", "mur")],
            &["face", "à", "le", "les", "mur", "jusque", "soir"],
            &[],
            &[],
            &[("face à", "facing"), ("jusque à", "until, up to")],
            &[("jusque à", "jusqu'à")],
            None,
        );
        let knowledge = KnowledgeState::new();
        let phrase = gloss_phrase("face au mur", FR, &french, &knowledge);
        let surfaces: Vec<&str> = phrase.tokens.iter().map(|t| t.surface.as_str()).collect();
        assert_eq!(surfaces, ["face", "à", "le", "mur"]);
        assert_eq!(spans(&phrase), [(0, 3, "face à")]);
        let phrase = gloss_phrase("face aux murs", FR, &french, &knowledge);
        assert_eq!(phrase.tokens[2].surface, "les");
        assert_eq!(spans(&phrase), [(0, 3, "face à")]);
        // The selection of the written words is answered whole, under the expression's name.
        let phrase = gloss_phrase("jusqu'au", FR, &french, &knowledge);
        assert_eq!(phrase.tokens.len(), 3);
        assert_eq!(spans(&phrase), [(0, 3, "jusqu'à")]);
        let phrase = gloss_phrase("jusqu'au soir", FR, &french, &knowledge);
        assert_eq!(spans(&phrase), [(0, 3, "jusqu'à")]);
        // A match ending elsewhere stops where it stops; the tokens do not move with the table.
        let bare = build_pack_for(
            FR,
            &[("murs", "mur")],
            &["face", "à", "le", "les", "mur"],
            &[],
            &[],
            &[],
        );
        assert_eq!(
            gloss_phrase("face au mur", FR, &bare, &knowledge).tokens,
            gloss_phrase("face au mur", FR, &french, &knowledge).tokens
        );
        let phrase = gloss_phrase("face à le mur", FR, &french, &knowledge);
        assert_eq!(spans(&phrase), [(0, 2, "face à")]);
    }

    #[test]
    fn an_english_match_ending_before_a_split_word_s_second_half_is_not_extended() {
        // English's « don't » shares its span between `do` and `not`; its matches stop where they
        // stop (add-lingua-spanish-expression-keys D5 is Spanish's and French's).
        let english = build_pack_for(
            EN,
            &[],
            &["i", "do", "not"],
            &[],
            &[],
            &[("i do", "je fais")],
        );
        let phrase = gloss_phrase("I don't", EN, &english, &KnowledgeState::new());
        assert_eq!(phrase.tokens.len(), 3);
        assert_eq!(spans(&phrase), [(0, 2, "i do")]);
    }

    // --- Overlapping expressions (choose-lingua-overlapping-expressions) ---

    /// A pack studying `studied` holding `expressions` (headword, gloss), keyed as the builder
    /// keys them — English at its spaces, French and Spanish through their reading — and named by
    /// their headwords where the two differ (add-lingua-french-expression-keys D1, D3,
    /// add-lingua-spanish-expression-keys D1, D3), with the grammar `readings`.
    fn overlap_pack(
        studied: StudiedLanguage,
        forms: &[(&str, &str)],
        lemmas: &[&str],
        expressions: &[(&str, &str)],
        readings: &[Reading],
    ) -> Pack {
        let (bytes, pool) = build_lexicon_blobs(forms, lemmas).expect("build");
        let lexicon = FstLexicon::from_slices(bytes, &pool).expect("load");
        let keyed: Vec<(String, &str, &str)> = expressions
            .iter()
            .map(|&(headword, gloss)| {
                let key = if studied == EN {
                    headword.to_owned()
                } else {
                    reading_expression_key(headword, studied, &lexicon).expect(headword)
                };
                (key, headword, gloss)
            })
            .collect();
        let table: Vec<(&str, &str)> = keyed
            .iter()
            .map(|(key, _, gloss)| (key.as_str(), *gloss))
            .collect();
        let names: Vec<(&str, &str)> = keyed
            .iter()
            .filter(|(key, headword, _)| key != headword)
            .map(|(key, headword, _)| (key.as_str(), *headword))
            .collect();
        build_pack_with_readings(
            studied,
            forms,
            lemmas,
            &[],
            &[],
            &table,
            &names,
            None,
            &[],
            readings,
        )
    }

    /// Each match of `text` read as `studied` on `pack`: the surfaces of the tokens it covers
    /// and what it reports.
    fn overlap_matches(
        pack: &Pack,
        studied: StudiedLanguage,
        text: &str,
    ) -> Vec<(Vec<String>, String)> {
        let phrase = gloss_phrase(text, studied, pack, &KnowledgeState::new());
        phrase
            .expressions
            .iter()
            .map(|m| {
                let covered = phrase.tokens[m.start..m.end]
                    .iter()
                    .map(|token| token.surface.clone())
                    .collect();
                (covered, m.key.clone())
            })
            .collect()
    }

    const OVERLAP_FR_FORMS: &[(&str, &str)] = &[
        ("ai", "avoir"),
        ("est", "être"),
        ("mit", "mettre"),
        ("resta", "rester"),
        ("sont", "être"),
        ("venait", "venir"),
        ("veux", "vouloir"),
        ("viens", "venir"),
    ];

    const OVERLAP_FR_LEMMAS: &[&str] = &[
        "amical",
        "avoir",
        "besoin",
        "cause",
        "ce",
        "courant",
        "de",
        "début",
        "elle",
        "et",
        "étranger",
        "être",
        "fois",
        "il",
        "je",
        "jusque",
        "la",
        "le",
        "lit",
        "mettre",
        "nombre",
        "nuit",
        "pluie",
        "que",
        "rester",
        "se",
        "sept",
        "sévère",
        "toi",
        "tout",
        "tu",
        "venir",
        "vouloir",
        "à",
    ];

    /// The French verbs of the delta's scenarios, each read by the grammar as a verb.
    const OVERLAP_FR_READINGS: &[Reading] = &[
        ("ai", "avoir", "AUX"),
        ("est", "être", "AUX"),
        ("mit", "mettre", "VERB"),
        ("resta", "rester", "VERB"),
        ("sont", "être", "AUX"),
        ("venait", "venir", "VERB"),
        ("veux", "vouloir", "VERB"),
        ("viens", "venir", "VERB"),
    ];

    fn french_overlap(expressions: &[(&str, &str)], text: &str) -> Vec<(Vec<String>, String)> {
        let pack = overlap_pack(
            FR,
            OVERLAP_FR_FORMS,
            OVERLAP_FR_LEMMAS,
            expressions,
            OVERLAP_FR_READINGS,
        );
        overlap_matches(&pack, FR, text)
    }

    #[test]
    fn spec_scenario_a_verb_s_expression_gives_the_au_to_the_expression_starting_on_it() {
        let expressions = [
            ("être à", "to belong to"),
            ("nombre de", "quite a few"),
            ("au nombre de", "numbering"),
        ];
        assert_eq!(
            french_overlap(&expressions, "Elles sont au nombre de sept."),
            [covering(&["à", "le", "nombre", "de"], "au nombre de")]
        );
    }

    #[test]
    fn spec_scenario_a_preposition_written_apart_goes_to_a_longer_expression() {
        let expressions = [("être à", "to belong to"), ("à la fois", "at once")];
        assert_eq!(
            french_overlap(&expressions, "Il est à la fois sévère et amical."),
            [covering(&["à", "la", "fois"], "à la fois")]
        );
    }

    #[test]
    fn spec_scenario_the_giving_expression_is_tried_shorter() {
        let expressions = [
            ("se mettre", "to place oneself"),
            ("se mettre à", "to start"),
            ("au lit", "in bed"),
        ];
        assert_eq!(
            french_overlap(&expressions, "Elle se mit au lit."),
            [
                covering(&["se", "mit"], "se mettre"),
                covering(&["à", "le", "lit"], "au lit")
            ]
        );
    }

    #[test]
    fn spec_scenario_the_verb_itself_goes_to_a_longer_expression() {
        let expressions = [("j'ai", "I have"), ("avoir besoin de", "to need")];
        assert_eq!(
            french_overlap(&expressions, "J'ai besoin de toi."),
            [covering(&["ai", "besoin", "de"], "avoir besoin de")]
        );
    }

    #[test]
    fn spec_scenario_the_question_marker_needs_its_hyphen_to_take_a_word() {
        let expressions = [
            ("c'est", "it is"),
            ("ce que", "what"),
            ("est-ce que", "question marker"),
        ];
        assert_eq!(
            french_overlap(&expressions, "C'est ce que tu veux."),
            [
                covering(&["Ce", "est"], "c'est"),
                covering(&["ce", "que"], "ce que")
            ]
        );
        // Written with its hyphen, the page meets the headword and the verb gives « est » away.
        assert_eq!(
            french_overlap(&expressions, "C'est-ce que tu veux ?"),
            [covering(&["est", "ce", "que"], "est-ce que")]
        );
    }

    #[test]
    fn spec_scenario_outside_an_overlap_the_hyphen_is_not_read() {
        let expressions = [
            ("c'est", "it is"),
            ("ce que", "what"),
            ("est-ce que", "question marker"),
        ];
        assert_eq!(
            french_overlap(&expressions, "Est ce que tu viens ?"),
            [covering(&["Est", "ce", "que"], "est-ce que")]
        );
        assert_eq!(
            french_overlap(&expressions, "Est-ce que tu viens ?"),
            [covering(&["Est", "ce", "que"], "est-ce que")]
        );
    }

    #[test]
    fn spec_scenario_a_compound_preposition_keeps_its_a() {
        let expressions = [("jusqu'à", "until"), ("au début", "at the beginning")];
        assert_eq!(
            french_overlap(&expressions, "Il resta jusqu'au début de la nuit."),
            [covering(&["jusque", "à", "le"], "jusqu'à")]
        );
    }

    #[test]
    fn spec_scenario_an_expression_no_longer_than_the_verb_s_takes_nothing() {
        let expressions = [("venir de", "to have just"), ("de l'", "some")];
        assert_eq!(
            french_overlap(&expressions, "Il venait de l'étranger."),
            [covering(&["venait", "de"], "venir de")]
        );
    }

    #[test]
    fn an_expression_next_made_of_function_words_only_may_be_cut() {
        // G3 keeps a word only for an expression next holding a word that is not a function
        // word: `de la` is cut, and `être à` gives its `à` to `à cause de`.
        let expressions = [
            ("être à", "to belong to"),
            ("à cause de", "because of"),
            ("de la", "some"),
        ];
        assert_eq!(
            french_overlap(&expressions, "Il est à cause de la pluie."),
            [covering(&["à", "cause", "de"], "à cause de")]
        );
    }

    #[test]
    fn a_pack_without_grammar_keeps_today_s_matches() {
        let expressions = [
            ("être à", "to belong to"),
            ("nombre de", "quite a few"),
            ("au nombre de", "numbering"),
        ];
        let pack = overlap_pack(FR, OVERLAP_FR_FORMS, OVERLAP_FR_LEMMAS, &expressions, &[]);
        assert_eq!(
            overlap_matches(&pack, FR, "Elles sont au nombre de sept."),
            [
                covering(&["sont", "à", "le"], "être à"),
                covering(&["nombre", "de"], "nombre de")
            ]
        );
    }

    #[test]
    fn the_hyphen_join_reads_the_euphonic_t_and_lines_up_with_the_headword() {
        let pack = overlap_pack(FR, OVERLAP_FR_FORMS, OVERLAP_FR_LEMMAS, &[], &[]);
        let lexicon = pack.lexicon();
        // `pense-t-il`: one join, the euphonic `t` between, on the page and in the headword.
        assert!(meets_hyphens(&[false, true], "pense-t-il", lexicon));
        assert!(!meets_hyphens(&[false, false], "pense-t-il", lexicon));
        // A headword without a hyphen, or whose reading does not line up, meets.
        assert!(meets_hyphens(&[false, false], "ce que", lexicon));
        assert!(meets_hyphens(&[false, false, false], "est-ce", lexicon));
        // Only the headword's hyphened places are held to the page.
        assert!(meets_hyphens(&[false, true, false], "est-ce que", lexicon));
        assert!(!meets_hyphens(&[false, false, true], "est-ce que", lexicon));
    }

    const OVERLAP_ES_FORMS: &[(&str, &str)] = &[
        ("hizo", "hacer"),
        ("iba", "ir"),
        ("pese", "pesar"),
        ("tengo", "tener"),
    ];

    const OVERLAP_ES_LEMMAS: &[&str] = &[
        "a",
        "de",
        "dinero",
        "día",
        "el",
        "ella",
        "hacer",
        "ir",
        "lo",
        "mal",
        "modos",
        "no",
        "pesar",
        "que",
        "se",
        "siguiente",
        "tener",
        "tiempo",
        "todos",
    ];

    const OVERLAP_ES_READINGS: &[Reading] = &[
        ("hizo", "hacer", "VERB"),
        ("iba", "ir", "VERB"),
        ("pesar", "pesar", "VERB"),
        ("pese", "pesar", "VERB"),
        ("tengo", "tener", "VERB"),
    ];

    fn spanish_overlap(expressions: &[(&str, &str)], text: &str) -> Vec<(Vec<String>, String)> {
        let pack = overlap_pack(
            ES,
            OVERLAP_ES_FORMS,
            OVERLAP_ES_LEMMAS,
            expressions,
            OVERLAP_ES_READINGS,
        );
        overlap_matches(&pack, ES, text)
    }

    #[test]
    fn spec_scenario_a_spanish_verb_s_preposition() {
        let expressions = [("hacer de", "to act as"), ("de todos modos", "anyway")];
        assert_eq!(
            spanish_overlap(&expressions, "Lo hizo de todos modos."),
            [covering(&["de", "todos", "modos"], "de todos modos")]
        );
    }

    #[test]
    fn spec_scenario_a_preposition_followed_by_que_stays() {
        let expressions = [("pese a", "despite"), ("a que no", "I bet you won't")];
        assert_eq!(
            spanish_overlap(&expressions, "Pese a que no tengo dinero"),
            [covering(&["Pese", "a"], "pese a")]
        );
    }

    #[test]
    fn spec_scenario_the_expression_next_is_not_cut() {
        let expressions = [
            ("ir a", "to go to"),
            ("al día", "up to date"),
            ("día siguiente", "next day"),
        ];
        assert_eq!(
            spanish_overlap(&expressions, "Ella se iba al día siguiente."),
            [
                covering(&["iba", "a", "el"], "ir a"),
                covering(&["día", "siguiente"], "día siguiente")
            ]
        );
        // Without the expression next, the verb's « al » goes to `al día`.
        let expressions = [("ir a", "to go to"), ("al día", "up to date")];
        assert_eq!(
            spanish_overlap(&expressions, "Ella se iba al día siguiente."),
            [covering(&["a", "el", "día"], "al día")]
        );
    }

    #[test]
    fn a_run_whose_verb_follows_a_preposition_keeps_its_last_word() {
        // `a pesar de` is built on a preposition: « pesar » after `a` is not its verb.
        let expressions = [("a pesar de", "despite"), ("del tiempo", "of the weather")];
        assert_eq!(
            spanish_overlap(&expressions, "a pesar del mal tiempo"),
            [covering(&["a", "pesar", "de", "el"], "a pesar de")]
        );
        assert_eq!(
            spanish_overlap(&expressions, "a pesar del tiempo"),
            [covering(&["a", "pesar", "de", "el"], "a pesar de")]
        );
    }

    #[test]
    fn spec_scenario_an_english_verb_s_particle() {
        let pack = overlap_pack(
            EN,
            &[("went", "go")],
            &["go", "over", "the", "top", "whole", "thing"],
            &[("go over", "examiner"), ("over the top", "exagéré")],
            &[("went", "go", "VERB")],
        );
        assert_eq!(
            overlap_matches(&pack, EN, "The whole thing went over the top."),
            [covering(&["over", "the", "top"], "over the top")]
        );
    }

    #[test]
    fn an_english_split_word_glosses_as_before() {
        // « don't » shares its span between `do` and `not`; English never covers a second half,
        // and `do not` is no longer than `i do`.
        let pack = overlap_pack(
            EN,
            &[],
            &["i", "do", "not"],
            &[("i do", "je fais"), ("do not", "ne pas")],
            &[("do", "do", "AUX")],
        );
        assert_eq!(
            overlap_matches(&pack, EN, "I don't"),
            [covering(&["I", "do"], "i do")]
        );
    }

    // --- A Spanish expression settled under its lemmas keeps that key (D6) ---

    #[test]
    fn spec_scenario_a_status_set_before() {
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status_at(
            ES,
            "tener en contar",
            Status::Known(KnownSource::Manual),
            1_000,
        );
        let phrase = gloss_phrase(
            "tener en cuenta",
            ES,
            &spanish_expression_pack(),
            &knowledge,
        );
        assert_eq!(phrase.expressions[0].key, "tener en contar");
        assert_eq!(phrase.expressions[0].class, TokenClass::Known);
    }

    #[test]
    fn spec_scenario_a_deck_card_made_before() {
        // « + Deck » sets « learning » on the key the card was made under.
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status_at(ES, "a el vez", Status::Learning, 1_000);
        let phrase = gloss_phrase("a la vez", ES, &spanish_expression_pack(), &knowledge);
        assert_eq!(phrase.expressions[0].key, "a el vez");
        assert_eq!(phrase.expressions[0].class, TokenClass::Learning);
        assert_eq!(
            (phrase.expressions[0].start, phrase.expressions[0].end),
            (0, 3)
        );
    }

    #[test]
    fn spec_scenario_nothing_settled() {
        // A record on another chain, or in another language, is not this one's.
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(ES, "tener en mente", Status::Known(KnownSource::Manual));
        knowledge.set_status(EN, "tener en contar", Status::Known(KnownSource::Manual));
        assert_eq!(
            spanish_spans_for("tener en cuenta", &knowledge),
            [span(0, 3, "tener en cuenta")]
        );
    }

    #[test]
    fn spec_scenario_a_record_on_the_name() {
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status_at(
            ES,
            "tener en contar",
            Status::Known(KnownSource::Manual),
            1_000,
        );
        knowledge.set_status_at(ES, "tener en cuenta", Status::Learning, 2_000);
        let phrase = gloss_phrase(
            "tener en cuenta",
            ES,
            &spanish_expression_pack(),
            &knowledge,
        );
        assert_eq!(phrase.expressions[0].key, "tener en cuenta");
        assert_eq!(phrase.expressions[0].class, TokenClass::Learning);
        // A withdrawn record on the name is one too.
        let mut withdrawn = knowledge.clone();
        withdrawn.clear_status_at(ES, "tener en cuenta", 3_000);
        let phrase = gloss_phrase(
            "tener en cuenta",
            ES,
            &spanish_expression_pack(),
            &withdrawn,
        );
        assert_eq!(phrase.expressions[0].key, "tener en cuenta");
        assert_eq!(phrase.expressions[0].class, TokenClass::Unknown);
    }

    #[test]
    fn a_withdrawn_status_on_the_chain_keeps_the_chain_a_withdrawal_stamped_zero_included() {
        for at in [0, 5_000] {
            let mut knowledge = KnowledgeState::new();
            knowledge.set_status_at(ES, "tener en contar", Status::Learning, at);
            knowledge.clear_status_at(ES, "tener en contar", at);
            let phrase = gloss_phrase(
                "tener en cuenta",
                ES,
                &spanish_expression_pack(),
                &knowledge,
            );
            assert_eq!(phrase.expressions[0].key, "tener en contar", "{at}");
            assert_eq!(phrase.expressions[0].class, TokenClass::Unknown, "{at}");
        }
    }

    #[test]
    fn a_status_pulled_from_another_device_is_read_as_a_local_one() {
        let mut knowledge = KnowledgeState::new();
        assert!(knowledge.apply_status_lww(
            ES,
            "tener en contar",
            Some(Status::Known(KnownSource::Manual)),
            1_000
        ));
        let phrase = gloss_phrase(
            "tener en cuenta",
            ES,
            &spanish_expression_pack(),
            &knowledge,
        );
        assert_eq!(phrase.expressions[0].key, "tener en contar");
        assert_eq!(phrase.expressions[0].class, TokenClass::Known);
        // A pulled withdrawal too.
        let mut pulled = KnowledgeState::new();
        assert!(pulled.apply_status_lww(ES, "a el vez", None, 1_000));
        assert_eq!(
            spanish_spans_for("a la vez", &pulled),
            [span(0, 3, "a el vez")]
        );
    }

    #[test]
    fn the_chain_is_taken_before_the_article_of_a_contraction_is_covered() {
        // « después del » was reported `después de`, over its first two tokens, before names:
        // that is the chain, never `después de el`, which no match ever reported.
        let pack = build_pack_with_names(
            ES,
            SPANISH_FORMS,
            SPANISH_LEMMAS,
            &[],
            &[],
            &[("después de", "Après")],
            &[("después de", "después de algo")],
            None,
        );
        let mut on_chain = KnowledgeState::new();
        on_chain.set_status(ES, "después de", Status::Ignored);
        let phrase = gloss_phrase("después del", ES, &pack, &on_chain);
        assert_eq!(
            (phrase.expressions[0].start, phrase.expressions[0].end),
            (0, 3)
        );
        assert_eq!(phrase.expressions[0].key, "después de");
        assert_eq!(phrase.expressions[0].class, TokenClass::Ignored);
        let mut covered = KnowledgeState::new();
        covered.set_status(ES, "después de el", Status::Ignored);
        let phrase = gloss_phrase("después del", ES, &pack, &covered);
        assert_eq!(phrase.expressions[0].key, "después de algo");
        assert_eq!(phrase.expressions[0].class, TokenClass::Unknown);
        // A contraction inside the run is part of its chain.
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(ES, "a el menos", Status::Learning);
        assert_eq!(
            spanish_spans_for("al menos", &knowledge),
            [span(0, 3, "a el menos")]
        );
    }

    #[test]
    fn french_and_english_matches_never_take_a_chain() {
        // French: the chain of « il y avait » is `il y avoir`, its key; the name is reported.
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(FR, "il y avoir", Status::Known(KnownSource::Manual));
        let phrase = gloss_phrase("il y avait", FR, &french_expression_pack(), &knowledge);
        assert_eq!(phrase.expressions[0].key, "il y a");
        assert_eq!(phrase.expressions[0].class, TokenClass::Unknown);
        // English: a pack naming a key otherwise (no shipped one does) still reports the name.
        let pack = build_pack_with_names(
            EN,
            &[("gave", "give")],
            &["give", "up"],
            &[],
            &[],
            &[("give up", "Abandonner")],
            &[("give up", "gave up")],
            None,
        );
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(EN, "give up", Status::Known(KnownSource::Manual));
        let phrase = gloss_phrase("gave up", EN, &pack, &knowledge);
        assert_eq!(phrase.expressions[0].key, "gave up");
        assert_eq!(phrase.expressions[0].class, TokenClass::Unknown);
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

    /// The grammar tables of a test pack, laid out as the builder lays them: a sorted tag
    /// pool, and paradigms and runs keyed by lemma id, each zstd-compressed.
    fn grammar_blobs(
        lex: &FstLexicon<Vec<u8>>,
        readings: &[Reading],
        also: &[Also],
        runs: &[Runs],
    ) -> [Vec<u8>; 3] {
        use crate::packs::grammar::{
            FormEdit, ParadigmEntry, SenseRun, encode_indexed, encode_paradigm, encode_runs,
            encode_tag_pool,
        };
        use std::collections::BTreeMap;
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
        [
            encode_tag_pool(&tags),
            zstd::encode_all(encode_indexed(&paradigms).as_slice(), 19).unwrap(),
            zstd::encode_all(encode_indexed(&run_entries).as_slice(), 19).unwrap(),
        ]
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
        let (form_bytes, pool) = build_lexicon_blobs(forms, lemmas).expect("lexicon");
        let lex = FstLexicon::from_slices(form_bytes.clone(), &pool).unwrap();
        let id = |lemma: &str| lex.id_of(lemma).expect("listed") as u32;
        let [tag_pool, paradigm_zst, runs_zst] = grammar_blobs(&lex, readings, also, runs);
        let gloss_entries: Vec<(u32, &str)> = glosses.iter().map(|(l, g)| (id(l), *g)).collect();
        let mut freq = vec![0u32; lex.lemma_count()];
        for (lemma, rank) in ranks {
            freq[id(lemma) as usize] = *rank;
        }
        let freq_bytes: Vec<u8> = freq.iter().flat_map(|r| r.to_le_bytes()).collect();
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
        let gloss = build_gloss_zst(&gloss_entries);
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
        assert!(a.contains("\"analyzer_version\":\"1.3.0\""));
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

    // ignore-lingua-soft-hyphens D4: the selection and the card read a word without its soft
    // hyphens (U+00AD), and a page holding them is read as its clean text, spans aside. `~`
    // stands for one, as `‧` does in the spec's scenarios.

    fn shy(text: &str) -> String {
        text.replace('~', "\u{AD}")
    }

    #[test]
    fn spec_scenario_a_selection_and_a_card() {
        const FIRST_PLURAL_IMPERFECT: &str =
            "VERB|Mood=Ind|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin";
        let pack = build_pack_with_grammar_for(
            ES,
            &[("cantábamos", "cantar")],
            &["mañana", "cantar"],
            &[("mañana", 300), ("cantar", 900)],
            &[("mañana", "Matin; Demain"), ("cantar", "Chanter")],
            &[("cantábamos", "cantar", FIRST_PLURAL_IMPERFECT)],
            &[],
            &[],
        );
        let mut knowledge = KnowledgeState::new();
        for status in [None, Some(Status::Known(KnownSource::Manual))] {
            if let Some(status) = status {
                knowledge.set_status(ES, "mañana", status);
            }
            let selection = gloss_phrase(&shy("ma~ña~na"), ES, &pack, &knowledge);
            assert_eq!(selection, gloss_phrase("mañana", ES, &pack, &knowledge));
            assert_eq!(selection.tokens[0].surface, "mañana");
            assert_eq!(selection.tokens[0].lemma, "mañana");
            assert_eq!(selection.tokens[0].gloss.as_deref(), Some("Matin; Demain"));
        }
        let card = word_grammar(&shy("can~tá~ba~mos"), "cantar", ES, &pack);
        assert_eq!(card, word_grammar("cantábamos", "cantar", ES, &pack));
        assert_eq!(ud(&card.readings), [FIRST_PLURAL_IMPERFECT]);
        assert!(card.pieces.is_empty());
    }

    #[test]
    fn a_written_word_that_yields_no_token_is_read_without_its_soft_hyphens() {
        // `x` alone is no token (a single letter the lexicon does not hold): the card falls
        // back on the written word, read as the tokeniser reads every word.
        let pack = build_pack_with_grammar(
            &[],
            &["ex"],
            &[],
            &[("x", "ex", "NOUN|Number=Sing")],
            &[],
            &[],
        );
        assert!(tokenize("x", EN, pack.lexicon()).is_empty());
        let clean = word_grammar("x", "ex", EN, &pack);
        assert_eq!(ud(&clean.readings), ["NOUN|Number=Sing"]);
        assert_eq!(word_grammar(&shy(" x~ "), "ex", EN, &pack), clean);
    }

    /// `blocks` analysed, voted on and glossed with their soft hyphens as without them: every
    /// answer the same, spans aside, and each span covering its word as written.
    fn page_reads_as_clean(blocks: &[&str], studied: StudiedLanguage, pack: &Pack) -> PageAnalysis {
        let written: Vec<String> = blocks.iter().map(|b| shy(b)).collect();
        let clean: Vec<String> = blocks.iter().map(|b| b.replace('~', "")).collect();
        let written: Vec<&str> = written.iter().map(String::as_str).collect();
        let clean: Vec<&str> = clean.iter().map(String::as_str).collect();
        assert_ne!(written, clean);
        let knowledge = KnowledgeState::new();
        let hyphenated = analyse_page(&written, studied, pack, &knowledge);
        let plain = analyse_page(&clean, studied, pack, &knowledge);
        assert!(plain.analysable);
        let aside = |page: &PageAnalysis| {
            let mut page = page.clone();
            for token in &mut page.tokens {
                (token.start, token.end) = (0, 0);
            }
            page
        };
        assert_eq!(aside(&hyphenated), aside(&plain));
        for (h, p) in hyphenated.tokens.iter().zip(&plain.tokens) {
            assert_eq!(
                without_soft_hyphens(&written[h.block][h.start..h.end]),
                &clean[p.block][p.start..p.end]
            );
        }
        assert!(written.iter().any(|b| {
            hyphenated
                .tokens
                .iter()
                .any(|t| b[t.start..t.end].contains('\u{AD}'))
        }));
        for candidates in [
            &[EN, ES][..],
            &[ES, FR],
            &[EN, FR],
            &[EN, ES, FR],
            &[FR, ES, EN],
        ] {
            assert_eq!(
                crate::analysis::language::detect_document_language(&written, candidates, None),
                crate::analysis::language::detect_document_language(&clean, candidates, None),
            );
        }
        for block in blocks {
            assert_eq!(
                gloss_phrase(&shy(block), studied, pack, &knowledge),
                gloss_phrase(&block.replace('~', ""), studied, pack, &knowledge)
            );
        }
        hyphenated
    }

    #[test]
    fn spec_scenario_a_page_holding_soft_hyphens() {
        let english = build_pack(
            &[("teams", "team"), ("went", "go"), ("couldn't", "could")],
            &[
                "the", "team", "go", "home", "early", "and", "code", "could", "not", "answer",
            ],
            &[("the", 1), ("go", 50), ("team", 900)],
            &[("team", "Équipe"), ("answer", "Réponse; Répondre")],
        );
        let page = page_reads_as_clean(
            &[
                "The teams went ho~me ear~ly and the teams co~de the co~de at ho~me.",
                "The team could~n't an~swer, and the teams went ho~me ear~ly.",
            ],
            EN,
            &english,
        );
        assert_eq!(page.analyzer_version, ANALYZER_VERSION);
        let spanish = build_pack_for(
            ES,
            &[("cosas", "cosa"), ("habrá", "haber")],
            &[
                "todo", "el", "mundo", "saber", "que", "mañana", "haber", "mucho", "cosa", "hacer",
            ],
            &[("el", 1), ("que", 2), ("mañana", 300)],
            &[("mañana", "Matin; Demain"), ("cosa", "Chose")],
            &[],
        );
        page_reads_as_clean(
            &[
                "To~do el mun~do sa~be que ma~ña~na ha~brá mu~chas co~sas que ha~cer.",
                "Ma~ña~na ha~brá co~sas que ha~cer, y to~do el mun~do lo sa~be.",
            ],
            ES,
            &spanish,
        );
        let french = build_pack_with_names(
            FR,
            FRENCH_FORMS,
            FRENCH_LEMMAS,
            &[("le", 1), ("de", 2), ("maison", 400)],
            &[("maison", "house"), ("enfant", "child")],
            &[],
            &[],
            None,
        );
        page_reads_as_clean(
            &[
                "L’en~fant a vu la mai~son du bou~lan~ger jus~qu’au soir, puis il a man~gé du pain.",
                "Lors~qu’il est ren~tré à la mai~son, l’en~fant a~vait les mains plei~nes de pain.",
            ],
            FR,
            &french,
        );
    }

    // — a word standing alone in capitals (keep-lingua-french-acronyms-in-capitals) —

    const ACRONYM_FORMS: &[(&str, &str)] = &[
        ("CA", "CA"),
        ("ca", "ça"),
        ("a", "avoir"),
        ("voté", "voter"),
        ("crié", "crier"),
        ("suffit", "suffire"),
        ("fait", "faire"),
        ("réunira", "réunir"),
        ("membres", "membre"),
    ];

    const ACRONYM_LEMMAS: &[&str] = &[
        "le",
        "ça",
        "avoir",
        "voter",
        "budget",
        "il",
        "crier",
        "suffire",
        "faire",
        "du",
        "bien",
        "de",
        "association",
        "se",
        "réunir",
        "demain",
        "avec",
        "les",
        "membre",
        "bureau",
        "onu",
    ];

    const CA_GLOSS: &str = "board of directors; revenue, turnover, net sales; alternating current";
    const CA_CEDILLA_GLOSS: &str = "that (distal demonstrative pronoun); this";

    /// A fr-en pack holding the acronym `CA` as its own lemma — a dictionary word, glossed,
    /// unranked — beside `ca` read as `ça`, and the expression `ça suffit`.
    fn acronym_pack() -> Pack {
        let (bytes, pool) = build_lexicon_blobs(ACRONYM_FORMS, ACRONYM_LEMMAS).expect("build");
        let lexicon = FstLexicon::from_slices(bytes, &pool).expect("load");
        let key = french_expression_key("ça suffit", &lexicon).expect("a key");
        assert_eq!(key, "ça suffire");
        let mut words: Vec<&str> = ACRONYM_LEMMAS.to_vec();
        words.push("CA");
        build_pack_with_names(
            FR,
            ACRONYM_FORMS,
            ACRONYM_LEMMAS,
            &[("le", 1), ("ça", 20), ("budget", 900)],
            &[
                ("CA", CA_GLOSS),
                ("ça", CA_CEDILLA_GLOSS),
                ("budget", "budget"),
            ],
            &[("ça suffire", "that's enough")],
            &[("ça suffire", "ça suffit")],
            Some(&words),
        )
    }

    /// Each token of a French selection: (surface, dictionary form, gloss, function word).
    fn acronym_phrase(text: &str) -> Vec<(String, String, Option<String>, bool)> {
        gloss_phrase(text, FR, &acronym_pack(), &KnowledgeState::new())
            .tokens
            .into_iter()
            .map(|t| (t.surface, t.lemma, t.gloss, t.function_word))
            .collect()
    }

    fn token_of(
        tokens: &[(String, String, Option<String>, bool)],
        surface: &str,
    ) -> (String, Option<String>, bool) {
        let (_, lemma, gloss, function_word) = tokens
            .iter()
            .find(|(written, ..)| written == surface)
            .unwrap_or_else(|| panic!("no {surface:?} in {tokens:?}"))
            .clone();
        (lemma, gloss, function_word)
    }

    #[test]
    fn spec_scenario_the_acronym_in_a_line_of_lowercase_words() {
        let tokens = acronym_phrase("Le CA a voté le budget.");
        assert_eq!(
            token_of(&tokens, "CA"),
            ("CA".to_owned(), Some(CA_GLOSS.to_owned()), false)
        );
        // The words beside it read as before.
        assert_eq!(token_of(&tokens, "a").0, "avoir");
        assert_eq!(token_of(&tokens, "voté").0, "voter");
    }

    #[test]
    fn spec_scenario_a_headline_in_capitals() {
        let tokens = acronym_phrase("LE CA A VOTÉ LE BUDGET");
        assert_eq!(
            token_of(&tokens, "CA"),
            ("ça".to_owned(), Some(CA_CEDILLA_GLOSS.to_owned()), true)
        );
    }

    #[test]
    fn spec_scenario_a_shout_in_a_line_of_lowercase_words() {
        let pack = acronym_pack();
        let phrase = gloss_phrase("Il a crié : CA SUFFIT !", FR, &pack, &KnowledgeState::new());
        let lemmas: Vec<&str> = phrase.tokens.iter().map(|t| t.lemma.as_str()).collect();
        assert_eq!(lemmas, ["il", "avoir", "crier", "ça", "suffire"]);
        assert_eq!(
            phrase
                .expressions
                .iter()
                .map(|m| (m.start, m.end, m.key.as_str(), m.gloss.as_str()))
                .collect::<Vec<_>>(),
            [(3, 5, "ça suffit", "that's enough")]
        );
    }

    #[test]
    fn spec_scenario_a_capital_initial() {
        let tokens = acronym_phrase("Ca fait du bien.");
        assert_eq!(token_of(&tokens, "Ca").0, "ça");
        assert_eq!(token_of(&acronym_phrase("ca va"), "ca").0, "ça");
    }

    #[test]
    fn spec_scenario_a_word_in_capitals_the_table_does_not_hold() {
        let page = analyse_document(
            &["L'ONU a voté le budget avec les membres du bureau demain."],
            FR,
            acronym_pack().lexicon(),
        );
        let DocumentAnalysis::Analysed(tokens) = page else {
            panic!("analysable");
        };
        let onu = tokens.iter().find(|t| t.surface == "ONU").expect("ONU");
        assert_eq!(onu.lemma, "onu");
    }

    #[test]
    fn spec_scenario_the_acronym_on_a_page() {
        let pack = acronym_pack();
        let blocks = [
            "Le CA de l'association se réunira demain avec les membres du bureau.",
            "Demain, les membres du bureau verront le CA de l'association avec plaisir.",
        ];
        let page = analyse_page(&blocks, FR, &pack, &KnowledgeState::new());
        assert!(page.analysable);
        let ca: Vec<_> = page.tokens.iter().filter(|t| t.surface == "CA").collect();
        assert_eq!(ca.len(), 2);
        for token in ca {
            // Read as `CA`, counted as a word to learn — never set aside as a name — and glossed.
            assert_eq!(token.lemma, "CA");
            assert_eq!(token.class, TokenClass::Unknown);
            assert_eq!(token.gloss.as_deref(), Some(CA_GLOSS));
        }
        // Not ranked: the pack holds no rank for it.
        assert_eq!(
            crate::knowledge::state::FrequencyRanks::rank(&pack, "CA"),
            None
        );
        assert!(pack.is_dictionary_word("CA"));
    }

    #[test]
    fn spec_scenario_the_acronym_s_card() {
        let grammar_pack = build_pack_with_grammar_for(
            FR,
            ACRONYM_FORMS,
            ACRONYM_LEMMAS,
            &[("ça", 20)],
            &[("CA", CA_GLOSS), ("ça", CA_CEDILLA_GLOSS)],
            &[],
            &[],
            &[("CA", &[("NOUN", 3)]), ("ça", &[("PRON", 2)])],
        );
        let card = word_grammar("CA", "CA", FR, &grammar_pack);
        assert_eq!(card.gloss.as_deref(), Some(CA_GLOSS));
        assert_eq!(card.senses.len(), 1);
        assert_eq!(card.senses[0].text, CA_GLOSS);
        assert_eq!(
            card.senses[0].tag.as_ref().map(Tag::to_ud).as_deref(),
            Some("NOUN")
        );
        assert!(card.readings.is_empty() && card.others.is_empty() && card.pieces.is_empty());
        // `ça`'s card, for « ca » written in lowercase, is `ça`'s, as before.
        let cedilla = word_grammar("ca", "ça", FR, &grammar_pack);
        assert_eq!(cedilla.gloss.as_deref(), Some(CA_CEDILLA_GLOSS));
        // A dictionary form the pack holds in no capitals is lowercased, as before.
        assert_eq!(
            word_grammar("Ça", "Ça", FR, &grammar_pack).gloss.as_deref(),
            Some(CA_CEDILLA_GLOSS)
        );
    }

    #[test]
    fn the_four_readers_read_ca_alike() {
        // A page's block, a selection, a card's written word and an expression's headword read
        // « CA » through one rule (keep-lingua-french-acronyms-in-capitals D1).
        let pack = acronym_pack();
        let lexicon = pack.lexicon();
        for (text, expected) in [
            (
                "Le CA a voté le budget avec les membres du bureau demain.",
                "CA",
            ),
            (
                "LE CA A VOTÉ LE BUDGET AVEC LES MEMBRES DU BUREAU DEMAIN.",
                "ça",
            ),
        ] {
            let DocumentAnalysis::Analysed(page) = analyse_document(&[text], FR, lexicon) else {
                panic!("{text} is analysable");
            };
            let on_page = &page.iter().find(|t| t.surface == "CA").expect("CA").lemma;
            let phrase = gloss_phrase(text, FR, &pack, &KnowledgeState::new());
            let selected = &phrase
                .tokens
                .iter()
                .find(|t| t.surface == "CA")
                .expect("CA")
                .lemma;
            let reading = headword_reading(text, FR, lexicon).expect("a reading");
            let in_headword = &reading.iter().find(|(t, _)| t.text == "CA").expect("CA").1;
            assert_eq!(
                [on_page.as_str(), selected.as_str(), in_headword.as_str()],
                [expected; 3],
                "{text}"
            );
        }
        // A card's written word: `CA` alone answers the acronym's gloss.
        assert_eq!(
            word_grammar("CA", "CA", FR, &pack).gloss.as_deref(),
            Some(CA_GLOSS)
        );
    }

    #[test]
    fn english_and_spanish_read_capitals_as_before() {
        // A pack listing `US` and `CA` as written: English and Spanish still lowercase every
        // token, and their word grammar too.
        let english = build_pack_with_lexical(
            EN,
            &[("US", "US")],
            &["the", "us", "army", "ca"],
            &[],
            &[("us", "nous")],
            &[],
            None,
        );
        let phrase = gloss_phrase("the US army", EN, &english, &KnowledgeState::new());
        assert_eq!(phrase.tokens[1].lemma, "us");
        assert_eq!(phrase.tokens[1].gloss.as_deref(), Some("nous"));
        let spanish = build_pack_for(ES, &[("CA", "CA")], &["el", "ca", "votar"], &[], &[], &[]);
        let phrase = gloss_phrase("el CA votó", ES, &spanish, &KnowledgeState::new());
        assert_eq!(phrase.tokens[1].lemma, "ca");
    }
}
