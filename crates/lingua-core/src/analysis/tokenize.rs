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

//! UAX #29 tokenisation with a per-studied-language pre-pass (design D2).
//!
//! `unicode-segmentation` finds word candidates; the English pre-pass then
//! absorbs surface quirks: `n't` contractions expand to their two words
//! (`don't` → `do` + `not`), edge apostrophes are stripped, and
//! single-letter tokens only survive when the lexicon knows them ("I", "a").
//! The Spanish pre-pass reads its text in NFC and splits `al`/`del` into
//! `a`/`de` + `el` (add-lingua-spanish-analysis D1). The French pre-pass
//! (add-lingua-french-tokenisation) reads the narrow no-break space (U+202F) as
//! a space, splits an elided word off the word it is joined to (`l'homme` →
//! `le` + `homme`, each piece with its own span; `aujourd'hui` stays whole),
//! splits `au`/`aux` into `à` + `le`/`les` and reads a hyphenated inversion as
//! words (`dit-il` → `dit` + `il`); it reads every word in NFC before comparing
//! it with anything — the elided forms, `au`/`aux`, a run the pack lists — and
//! writes every token's text composed, each span still the source's
//! (add-lingua-french-analysis D1). The Italian pre-pass
//! (add-lingua-italian-tokenisation) splits an elided word off the word it is
//! joined to (`l'uomo` → `lo` + `uomo`, each piece with its own span), keeps the
//! articulated prepositions of `di` whole (`del`, `della`; `dell'` read as
//! `dello`) and splits the others into the preposition and the article
//! (`nel` → `in` + `il`, sharing the span), and reads a truncation as its word
//! (`po'` → `poco`), its apostrophe kept in its span, and an apostrophe that stands for
//! an accent as the accented word the pack lists (`perche'` → `perché`, `E'` → `È`,
//! read-lingua-italian-unmarked-spellings D1); like French's, it reads every
//! word in NFC before comparing it with anything and writes every token's text
//! composed, each span still the source's (add-lingua-italian-analysis D1). Adding a
//! studied language means adding a pre-pass, not touching the tokeniser.
//!
//! Every language reads a word without the soft hyphens (U+00AD) an e-book or a
//! page hides inside it to let a line break there (« vi‧da », ‧ standing for the
//! invisible character): UAX #29 keeps them inside their word, and each rule then
//! reads the word through [`without_soft_hyphens`] — edge apostrophes, digits,
//! NFC, `n't`, `al`/`del`, French's elisions, `au`/`aux`, listed runs and
//! inversions —, so a token's text, and its parts, hold none. Its span stays the
//! source's, soft hyphens included, as a composed French token's does
//! (ignore-lingua-soft-hyphens D1–D2).

use std::borrow::Cow;

use serde::Serialize;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use super::language::StudiedLanguage;
use super::lexicon::Lexicon;

/// A countable token: the surface text (case preserved — the proper-noun
/// heuristic needs it) plus the byte span of the source text it came from.
/// The two halves of an expanded contraction share the written word's span
/// (`don't`, `del`, `au`): two letters cannot be shared out. The pieces of a
/// French elision and the words of a French inversion each have their own
/// (`L'homme` → `Le` [0, 2) + `homme` [2, 7); `a-t-il` → `a` [0, 1) + `il`
/// [4, 6)), so a click lands on the piece under the pointer
/// (add-lingua-french-tokenisation D6), and so do an Italian elision's (`L'uomo` →
/// `Lo` [0, 2) + `uomo` [2, 6)); an Italian articulated preposition's two tokens
/// share theirs (`nel`), and a truncation is one token whose span holds its
/// apostrophe (`po'` → `poco` [0, 3), add-lingua-italian-tokenisation D6), as is a
/// word whose apostrophe stands for its accent (`perche'` → `perché` [0, 7),
/// read-lingua-italian-unmarked-spellings D1). The text is the word without its soft
/// hyphens, the span the source's with them: « vi‧da » is `vida` [0, 6)
/// (ignore-lingua-soft-hyphens D2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Token {
    /// Surface text after the pre-pass (e.g. `do`, `not`, `Teams`). For a
    /// hyphenated compound it is the whole run, hyphens included (`repo-wide`).
    /// It never holds a soft hyphen (U+00AD).
    pub text: String,
    /// Byte offset of the token's source in the analysed text.
    pub start: usize,
    /// Byte end (exclusive) of the token's source in the analysed text.
    pub end: usize,
    /// The surfaces of a hyphenated compound's pieces (`["repo", "wide"]`),
    /// empty for an ordinary single-word token. A compound the lexicon does not
    /// know as a unit is classified from these — the reader understands it only
    /// as well as its weakest part.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<String>,
}

/// U+00AD SOFT HYPHEN: an invisible break point an e-book or a page may hide
/// inside a word (calibre's *Add soft hyphens*, HTML's `&shy;`).
pub const SOFT_HYPHEN: char = '\u{AD}';

/// `text` without its soft hyphens (ignore-lingua-soft-hyphens D2): borrowed when
/// it holds none, which is what every text without them pays; a copy without them
/// otherwise. The tokeniser reads every word through it, and detection every
/// block (`language::block_is_studied`, `language::detect_document_language`).
///
/// U+00AD is `C2 AD` in UTF-8: a text without the byte `AD` holds none, which one
/// byte search tells — every English word, and most others — before the
/// character search confirms one (`í` is `C3 AD`).
pub fn without_soft_hyphens(text: &str) -> Cow<'_, str> {
    if text.as_bytes().contains(&0xAD) && text.contains(SOFT_HYPHEN) {
        Cow::Owned(text.replace(SOFT_HYPHEN, ""))
    } else {
        Cow::Borrowed(text)
    }
}

/// Whether the text between two words is the single hyphen of a compound, a soft
/// hyphen UAX #29 glued to it read as nowhere (`-‧` is three bytes; the common
/// gap, a space, is answered by its length).
fn is_hyphen(gap: &str) -> bool {
    gap == "-" || (gap.len() > 1 && without_soft_hyphens(gap) == "-")
}

/// Whether the text between two tokens joins them with a hyphen as French's pre-pass
/// splits a hyphenated inversion: one hyphen (soft hyphens read as nowhere, as
/// [`is_hyphen`] reads them), or a hyphen, the euphonic `t` and a hyphen (`-t-`, any
/// case), which the pre-pass drops (« a-t-il » → `a` + `il`). The phrase gloss reads it
/// on a page and on a French expression's headword, holding the headword's hyphens to
/// the page where its expression would take a word (choose-lingua-overlapping-expressions
/// D3, G1).
pub fn is_hyphen_join(gap: &str) -> bool {
    is_hyphen(gap) || without_soft_hyphens(gap).eq_ignore_ascii_case("-t-")
}

/// Contractions whose base changes when `n't` is peeled off. Everything else
/// follows the regular rule `Xn't` → `X` + `not`.
const IRREGULAR_CONTRACTIONS: &[(&str, &str)] = &[
    ("won't", "will"),
    ("can't", "can"),
    ("shan't", "shall"),
    ("ain't", "be"),
];

/// Tokenises `text` for the studied language.
///
/// Tokens containing a digit are dropped (identifiers, quantities, "3D"):
/// they are not vocabulary. Apostrophe variants (`’`) are normalised to `'`
/// before the pre-pass so typographic text behaves like plain text.
///
/// Each language has its own pre-pass. English expands `n't`; Spanish reads
/// its tokens in NFC and splits `al`/`del` (add-lingua-spanish-analysis D1);
/// French takes an arm of its own (add-lingua-french-tokenisation): its words
/// are cut at U+202F, then read in NFC (add-lingua-french-analysis) by
/// `push_french_word` and `push_french_run` — elisions, `au`/`aux`, hyphenated
/// inversions. Italian takes an arm of its own (add-lingua-italian-tokenisation):
/// its words keep a listed trailing apostrophe (`italian_words`), then
/// `push_italian_word` and `push_italian_run` read its elisions, its articulated
/// prepositions, its truncations and an apostrophe that stands for an accent
/// (read-lingua-italian-unmarked-spellings D1), each word read in NFC (add-lingua-italian-analysis
/// D1); it has no U+202F rule (D2), and `dirmelo` stays whole — the enclitic rule is
/// its cascade's. All share the rules that belong to no language:
/// segmentation, the hyphen run, the compound rule, the digit drop, the
/// edge-apostrophe trim and the single-letter rule.
pub fn tokenize(
    text: &str,
    language: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> Vec<Token> {
    let words: Vec<(usize, &str)> = match language {
        StudiedLanguage::French => french_words(text),
        StudiedLanguage::Italian => italian_words(text, lexicon),
        StudiedLanguage::English | StudiedLanguage::Spanish => {
            text.unicode_word_indices().collect()
        }
    };
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let (start, first) = words[i];
        let mut end = start + first.len();
        // Absorb pieces joined to this one by a single hyphen: `repo-wide`,
        // `state-of-the-art`. Exactly one `-` is the hyphenation mark — a space
        // ends the run, and so does a `--`/`---` run (the ASCII em-dash / range,
        // not a compound). A soft hyphen UAX #29 glued to the hard one is read
        // without, as everywhere (ignore-lingua-soft-hyphens D2).
        let mut j = i + 1;
        while j < words.len() {
            let (next_start, next) = words[j];
            if !is_hyphen(&text[end..next_start]) {
                break;
            }
            end = next_start + next.len();
            j += 1;
        }
        let run = j - i >= 2;
        match language {
            StudiedLanguage::French if run => {
                push_french_run(&mut tokens, text, &words[i..j], lexicon);
            }
            StudiedLanguage::French => push_french_word(&mut tokens, text, first, start, lexicon),
            StudiedLanguage::Italian if run => {
                push_italian_run(&mut tokens, text, &words[i..j], lexicon);
            }
            StudiedLanguage::Italian => push_italian_word(&mut tokens, text, first, start, lexicon),
            StudiedLanguage::English | StudiedLanguage::Spanish if run => push_compound(
                &mut tokens,
                &text[start..end],
                &words[i..j],
                start,
                end,
                language,
                lexicon,
            ),
            StudiedLanguage::English | StudiedLanguage::Spanish => {
                push_word(&mut tokens, first, start, end, language, lexicon);
            }
        }
        i = j;
    }
    tokens
}

/// The readings of `text`, the one a page gives the written word first: the tokens
/// [`tokenize`] gives, then — where Italian's pre-pass reads a word of it by its context, `dai`,
/// `da'` or `mo'` (refine-lingua-italian-reading D4) — the same tokens with each such word read
/// the other way: `dai` as the verb or as `da` + `i`, `mo'` as `modo` or `mò`. A word card is
/// handed the written word alone, which ends the text — `dai` reads the verb there, `mo'` `mò`
/// —, and answers the reading that holds its dictionary form (`engine::word_grammar`): a card
/// keyed by `da` asks for the preposition. `a'`, `co'`, `ne'` and `su'` written with their
/// apostrophe (D3, at review) read first as the preposition and `i`, the only reading whose span
/// holds the apostrophe — the one a page that read them so hands the card —, then as the
/// apocope [`tokenize`] gives them alone. Every other text has one reading, [`tokenize`]'s.
pub fn tokenize_readings(
    text: &str,
    language: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> Vec<Vec<Token>> {
    let tokens = tokenize(text, language, lexicon);
    let other = match language {
        StudiedLanguage::Italian => italian_other_reading(text, &tokens),
        StudiedLanguage::English | StudiedLanguage::Spanish | StudiedLanguage::French => None,
    };
    match other {
        Some((other, true)) => vec![other, tokens],
        Some((other, false)) => vec![tokens, other],
        None => vec![tokens],
    }
}

/// Emits an ordinary single-word token, applying the pre-pass: apostrophe
/// normalisation/trimming, NFC (Spanish, French, Italian), the language's contraction split
/// (English `n't`, Spanish `al`/`del`), the digit drop and the
/// single-letter-needs-the-lexicon rule.
///
/// A contraction's base keeps the written word's capital by slicing the base
/// at the written first letter's byte length — one byte for every English and
/// Spanish base (`D`/`d`, `A`/`a`). French reads its elisions and `au`/`aux`
/// before this ([`push_french_word`]) and capitalises the read word's own first
/// letter ([`french_cased`]): `à` is two bytes where `A` is one, and the slice
/// would panic. For French, what reaches this is the word that remains — the
/// rules every language shares, its text composed; its span stays the source's,
/// combining marks included. Italian reads its elisions, articulated prepositions
/// and truncations before this ([`push_italian_word`]), with French's casing, and
/// what reaches this is likewise composed (add-lingua-italian-analysis D1).
/// Every rule reads the word without its soft hyphens,
/// so « could‧n't » is `could` + `not` (ignore-lingua-soft-hyphens D2).
fn push_word(
    tokens: &mut Vec<Token>,
    word: &str,
    start: usize,
    end: usize,
    language: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) {
    let normalized = without_soft_hyphens(word).replace('\u{2019}', "'");
    let trimmed = normalized.trim_matches('\'');
    if trimmed.is_empty() || trimmed.chars().any(|c| c.is_ascii_digit()) {
        return;
    }
    let composed = nfc_for(trimmed, language);
    let trimmed = composed.as_str();
    let lower = trimmed.to_lowercase();
    if let Some((base, second)) = split_contraction(&lower, language) {
        // Preserve the original casing on the base's first letter so the
        // proper-noun heuristic still sees "Don't" as sentence-cased.
        let base_cased = match trimmed.chars().next() {
            Some(first) if first.is_uppercase() => {
                let mut s = String::new();
                s.extend(first.to_uppercase());
                s.push_str(&base[first.len_utf8().min(base.len())..]);
                s
            }
            _ => base.to_owned(),
        };
        tokens.push(Token {
            text: base_cased,
            start,
            end,
            parts: Vec::new(),
        });
        tokens.push(Token {
            text: second.to_owned(),
            start,
            end,
            parts: Vec::new(),
        });
        return;
    }
    if single_letter_outside_lexicon(trimmed, &lower, lexicon) {
        return;
    }
    tokens.push(Token {
        text: trimmed.to_owned(),
        start,
        end,
        parts: Vec::new(),
    });
}

/// The word in NFC for Spanish, French and Italian, so a decomposed accent reads as
/// the pack's precomposed one (add-lingua-spanish-analysis D1,
/// add-lingua-french-analysis D1, add-lingua-italian-analysis D1); English text as it
/// came — its output must not move.
fn nfc_for(word: &str, language: StudiedLanguage) -> String {
    match language {
        StudiedLanguage::English => word.to_owned(),
        StudiedLanguage::Spanish | StudiedLanguage::French | StudiedLanguage::Italian => {
            word.nfc().collect()
        }
    }
}

/// Emits one token spanning a whole hyphenated compound, recording its pieces'
/// surfaces. Contractions and the single-letter rule do not apply inside a
/// compound — it stands or falls as a unit (so `x-ray`, `e-mail` survive).
///
/// A digit anywhere in the run means an identifier/quantity, not a compound
/// word (`utf-8`, `well-being-2`). Rather than drop the whole run — which would
/// swallow clean neighbours like `well`/`being` — it degrades to per-piece
/// tokenisation, identical to no fusion: the clean pieces survive, the
/// digit-bearing ones are dropped by [`push_word`]'s own digit rule. The
/// compound's text and its parts are read without their soft hyphens.
fn push_compound(
    tokens: &mut Vec<Token>,
    whole: &str,
    pieces: &[(usize, &str)],
    start: usize,
    end: usize,
    language: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) {
    if whole.chars().any(|c| c.is_ascii_digit()) {
        for (p_start, word) in pieces {
            push_word(
                tokens,
                word,
                *p_start,
                p_start + word.len(),
                language,
                lexicon,
            );
        }
        return;
    }
    tokens.push(Token {
        text: nfc_for(&without_soft_hyphens(whole), language),
        start,
        end,
        parts: pieces
            .iter()
            .map(|(_, w)| nfc_for(&without_soft_hyphens(w), language))
            .collect(),
    });
}

/// The language's contraction split, if `lower` is one: English `n't`, Spanish
/// `al`/`del` (add-lingua-spanish-analysis D1). French has none here: its
/// `au`/`aux` and its elisions are read by its own pre-pass
/// ([`push_french_word`], add-lingua-french-tokenisation D3–D4), whose casing
/// takes a two-byte `à`; `du` and `des` stay whole (M21). Italian has none here
/// either: its articulated prepositions are read by its own pre-pass
/// ([`push_italian_word`], add-lingua-italian-tokenisation D4) — `nel` → `in` +
/// `il`, `del` whole —, and neither English's `n't` nor Spanish's `al`/`del` ever
/// runs on it.
fn split_contraction(lower: &str, language: StudiedLanguage) -> Option<(&str, &'static str)> {
    match language {
        StudiedLanguage::English => split_english_contraction(lower),
        StudiedLanguage::Spanish => match lower {
            "al" => Some(("a", "el")),
            "del" => Some(("de", "el")),
            _ => None,
        },
        StudiedLanguage::French | StudiedLanguage::Italian => None,
    }
}

/// `don't` → (`do`, `not`) — irregular table first, then the regular rule.
fn split_english_contraction(lower: &str) -> Option<(&str, &'static str)> {
    for (contraction, base) in IRREGULAR_CONTRACTIONS {
        if lower == *contraction {
            return Some((base, "not"));
        }
    }
    let base = lower.strip_suffix("n't")?;
    if base.is_empty() {
        None
    } else {
        Some((base, "not"))
    }
}

fn single_letter_outside_lexicon(
    trimmed: &str,
    lower: &str,
    lexicon: &(impl Lexicon + ?Sized),
) -> bool {
    trimmed.chars().count() == 1 && !lexicon.contains(lower)
}

// French's pre-pass (add-lingua-french-tokenisation). The pieces it writes are
// lemmatised as any French word is.

const FRENCH: StudiedLanguage = StudiedLanguage::French;

/// The narrow no-break space, which French sets before `?`, `!`, `;` and `»`
/// and after `«`. UAX #29 gives it the word-break class ExtendNumLet, which
/// glues it to the word beside it; French reads it as a space (D2).
const NARROW_NO_BREAK_SPACE: char = '\u{202F}';

/// French's elided words — what is written before the apostrophe, lowercase —
/// and the word each stands for (D3), sorted byte-wise by the written form.
/// Two readings depend on the context ([`french_elided_word`]): `s'` is `si`
/// before `il` and `ils`, and `m'`, `t'` are `moi`, `toi` right after a hyphen.
/// `presqu'` and `quelqu'` are left off on purpose: they elide only in
/// lexicalised words (`presqu'île`, `quelqu'un`), which stay whole. Public for
/// the check that French's committed tables hold every word it reads
/// (`lingua-pack`'s `committed_tables.rs`, add-lingua-french-forms-tables D4).
pub const FRENCH_ELISIONS: &[(&str, &str)] = &[
    // `c'est` → `ce` + `est`
    ("c", "ce"),
    // `d'abord` → `de` + `abord`
    ("d", "de"),
    // `j'ai` → `je` + `ai`
    ("j", "je"),
    // `jusqu'ici` → `jusque` + `ici`
    ("jusqu", "jusque"),
    // `l'homme` → `le` + `homme` (`la` would need the next word's gender)
    ("l", "le"),
    // `lorsqu'il` → `lorsque` + `il`
    ("lorsqu", "lorsque"),
    // `m'appelle` → `me` + `appelle`; `donne-m'en` → `moi` + `en`
    ("m", "me"),
    // `n'est` → `ne` + `est`
    ("n", "ne"),
    // `puisqu'elle` → `puisque` + `elle`
    ("puisqu", "puisque"),
    // `qu'on` → `que` + `on`
    ("qu", "que"),
    // `quoiqu'il` → `quoique` + `il`
    ("quoiqu", "quoique"),
    // `s'en` → `se` + `en`; `s'il` → `si` + `il`
    ("s", "se"),
    // `t'aime` → `te` + `aime`; `va-t'en` → `toi` + `en`
    ("t", "te"),
    // `ç'a été` → `ça` + `a`
    ("ç", "ça"),
];

/// The pronouns the pieces of a French hyphenated inversion after the first
/// are (D5), sorted: `dit-il`, `allez-vous-en`, `coupez-les`, `dis-le-moi`.
/// Public for the same check as [`FRENCH_ELISIONS`].
pub const FRENCH_INVERSION_PRONOUNS: &[&str] = &[
    "ce", "elle", "elles", "en", "il", "ils", "je", "la", "le", "les", "leur", "lui", "moi",
    "nous", "on", "toi", "tu", "vous", "y",
];

/// The pronouns the euphonic `t` is written before: `a-t-il`, `pense-t-elle`.
const AFTER_EUPHONIC_T: &[&str] = &["elle", "elles", "il", "ils", "on"];

/// A French written piece as the pre-pass compares it: without its soft hyphens,
/// lowercase and in NFC, so a decomposed `ç'` is the elided `ç'`, a decomposed
/// `peut-être` the run the pack lists (add-lingua-french-analysis D1) and
/// « lors‧qu » the elided `lorsqu` (ignore-lingua-soft-hyphens D2). Italian's
/// pre-pass compares its written pieces the same way (add-lingua-italian-analysis
/// D1): its elided forms, its articulated prepositions, its truncations.
fn french_lowercase(written: &str) -> String {
    without_soft_hyphens(written).to_lowercase().nfc().collect()
}

/// The straight and the typographic apostrophe, the two French text is written
/// with (D3), and Italian text (add-lingua-italian-tokenisation D2). U+02BC,
/// U+2018 and U+FF07 are not read as apostrophes.
fn is_apostrophe(c: char) -> bool {
    c == '\'' || c == '\u{2019}'
}

/// The words of French text with their byte offsets: UAX #29's, each cut at
/// every U+202F, the space in no part (D2). A part keeps UAX #29's own filter: it
/// holds a letter or a digit.
fn french_words(text: &str) -> Vec<(usize, &str)> {
    let mut words = Vec::new();
    for (start, word) in text.unicode_word_indices() {
        let mut at = start;
        for part in word.split(NARROW_NO_BREAK_SPACE) {
            if part.chars().any(char::is_alphanumeric) {
                words.push((at, part));
            }
            at += part.len() + NARROW_NO_BREAK_SPACE.len_utf8();
        }
    }
    words
}

/// `read` with the capital of `written`'s first letter put on its own first
/// letter: `L'` → `Le`, `Qu'` → `Que`, `Au` → `À` (D4). [`push_word`]'s casing
/// slices its base at the written letter's byte length, which `à` under `A`
/// cannot take. A soft hyphen before that letter is no letter. Italian's pre-pass
/// cases its read words with it too: `L'` → `Lo`, `NEL` → `In`, `Fa'` → `Fai`
/// (add-lingua-italian-tokenisation D3).
fn french_cased(read: &str, written: &str) -> String {
    let mut letters = read.chars();
    match (written.chars().find(|&c| c != SOFT_HYPHEN), letters.next()) {
        (Some(capital), Some(first)) if capital.is_uppercase() => {
            first.to_uppercase().chain(letters).collect()
        }
        _ => read.to_owned(),
    }
}

/// The word an elided form stands for, or `None` when `written` (lowercase,
/// without its apostrophe) is not one (D3). `next` is the word that follows,
/// lowercase; `after_hyphen`, whether the elided form starts a piece after a
/// hyphen.
fn french_elided_word(written: &str, next: &str, after_hyphen: bool) -> Option<&'static str> {
    let index = FRENCH_ELISIONS
        .binary_search_by_key(&written, |&(form, _)| form)
        .ok()?;
    Some(match FRENCH_ELISIONS[index].1 {
        "se" if matches!(next, "il" | "ils") => "si",
        "me" if after_hyphen => "moi",
        "te" if after_hyphen => "toi",
        read => read,
    })
}

/// Splits the elided words off the front of a French word written at `start`
/// (D3): each is a token of its own, read as the word it stands for and spanning
/// its letters and its apostrophe — a combining mark among them, the written piece
/// being looked up composed —, and the rule runs again on what follows, as long as
/// a letter follows the apostrophe. Returns where the rest starts and the rest.
/// `after_hyphen`: the word is a piece after a hyphen. A soft hyphen is read as
/// nowhere, in the elided piece, in the word after it and right after the
/// apostrophe (ignore-lingua-soft-hyphens D2).
fn push_french_elisions<'a>(
    tokens: &mut Vec<Token>,
    word: &'a str,
    start: usize,
    after_hyphen: bool,
) -> (usize, &'a str) {
    let mut at = start;
    let mut rest = word;
    while let Some((i, apostrophe)) = rest.char_indices().find(|&(_, c)| is_apostrophe(c)) {
        let written = &rest[..i];
        let after = &rest[i + apostrophe.len_utf8()..];
        if !after
            .trim_start_matches(SOFT_HYPHEN)
            .starts_with(char::is_alphabetic)
        {
            break;
        }
        let next: String = after
            .chars()
            .filter(|&c| c != SOFT_HYPHEN)
            .take_while(|c| c.is_alphabetic())
            .flat_map(char::to_lowercase)
            .collect();
        let first_piece = at == start && after_hyphen;
        let Some(read) = french_elided_word(&french_lowercase(written), &next, first_piece) else {
            break;
        };
        let end = at + i + apostrophe.len_utf8();
        tokens.push(Token {
            text: french_cased(read, written),
            start: at,
            end,
            parts: Vec::new(),
        });
        at = end;
        rest = after;
    }
    (at, rest)
}

/// The byte length of the apostrophe at `end` of `text` when it ends an elided
/// word written on its own — neither a letter nor a digit after it (D3): `l’`
/// alone, as a word card is handed it, or `l’ homme`.
fn apostrophe_ending_a_word(text: &str, end: usize) -> Option<usize> {
    let mut after = text[end..].chars();
    let apostrophe = after.next().filter(|&c| is_apostrophe(c))?;
    match after.next() {
        Some(c) if c.is_alphanumeric() => None,
        _ => Some(apostrophe.len_utf8()),
    }
}

/// Whether `token`, read from `text`, is an elided piece, a truncation or an Italian
/// word written with an apostrophe for its accent — `l’` of « l’homme », `qu'` of
/// « qu'il », `l’` written alone, Italian's `l'` of « l'uomo », both tokens of its
/// « all'alba »'s `all'`, its `po'`, its `perche'` read as `perché`: its source span ends
/// on a straight or typographic apostrophe (match-lingua-french-elided-pieces D2,
/// add-lingua-italian-tokenisation D6, read-lingua-italian-unmarked-spellings D1).
/// French's pre-pass gives an elided piece, and it alone, a span holding its apostrophe
/// ([`push_french_elisions`], and an elided word written on its own); Italian's gives
/// one to an elided piece, to a truncation and to a word whose apostrophe stands for
/// its accent ([`push_italian_word`]). No other token's span can end on one: every
/// other span is a UAX #29 word's, or a hyphenated run's ending on one, and UAX #29
/// never ends a word on an apostrophe — it joins one only between two letters
/// (`aujourd'hui`, `don't`), and an edge apostrophe (`vélib'`, `sailors'`, Italian's
/// `ciao'`, or a `perche'` the pack lists no `perché` for) is outside the word's span.
/// The token's text cannot say it: an elided piece reads as the word it stands for
/// (`le`), a truncation as its word (`poco`), an accent spelling as its accented word
/// (`perché`). So the flag says yes for an accent spelling too, which is no elided
/// piece: an expression written with `perché` is not met where the page writes
/// `perche'` — named for Italian's phrase gloss (change 12), which may read the flag
/// by the token's text. English and Spanish tokens are never elided.
///
/// The phrase gloss reads it on a selection's tokens and on an expression's headword
/// (`engine::match_expressions`), so the two say it the same way.
pub fn is_elided(text: &str, token: &Token) -> bool {
    text.get(token.start..token.end)
        .is_some_and(|written| written.ends_with(is_apostrophe))
}

/// One French word written at `start` of `text` — outside a hyphenated run, a
/// piece of a run holding a digit, or an inversion's first piece: its elisions
/// (D3); then `au`/`aux`, split into `à` + `le`/`les` sharing the span,
/// `du`/`des` whole (D4); an elided word written on its own (D3); or the rules
/// every language shares ([`push_word`]). Each check reads the word composed and
/// each token's text is composed (add-lingua-french-analysis D1); the spans are
/// the source's.
fn push_french_word(
    tokens: &mut Vec<Token>,
    text: &str,
    word: &str,
    start: usize,
    lexicon: &(impl Lexicon + ?Sized),
) {
    let (at, rest) = push_french_elisions(tokens, word, start, false);
    let end = at + rest.len();
    let lower = french_lowercase(rest);
    let article = match lower.as_str() {
        "au" => Some("le"),
        "aux" => Some("les"),
        _ => None,
    };
    if let Some(article) = article {
        for read in [french_cased("à", rest), article.to_owned()] {
            tokens.push(Token {
                text: read,
                start: at,
                end,
                parts: Vec::new(),
            });
        }
        return;
    }
    if let Some(apostrophe) = apostrophe_ending_a_word(text, end)
        && let Some(read) = french_elided_word(&lower, "", false)
    {
        tokens.push(Token {
            text: french_cased(read, rest),
            start: at,
            end: end + apostrophe,
            parts: Vec::new(),
        });
        return;
    }
    push_word(tokens, rest, at, end, FRENCH, lexicon);
}

/// Whether the pack lists a hyphenated run whole, as `resolve_lemmas`
/// (`pipeline.rs`) reads it: `peut-être`, `rendez-vous` — the run composed, so a
/// decomposed `peut-être` is found (add-lingua-french-analysis D1).
fn listed_whole(run: &str, lexicon: &(impl Lexicon + ?Sized)) -> bool {
    lexicon
        .lemma_of(&french_lowercase(&run.replace('\u{2019}', "'")))
        .is_some()
}

/// Whether the pieces after a run's first are an inversion's (D5): each a
/// pronoun written in lowercase, or an elided `m'`, `t'` or `l'` before `en` or
/// `y`, the euphonic `t` allowed right before `il`, `elle`, `on`, `ils` or
/// `elles`. A capital anywhere is a name or text set in capitals. Each piece is
/// read without its soft hyphens.
fn is_french_inversion(tail: &[(usize, &str)]) -> bool {
    !tail.is_empty()
        && tail.iter().enumerate().all(|(k, &(_, piece))| {
            if piece.chars().any(char::is_uppercase) {
                return false;
            }
            let piece = without_soft_hyphens(piece).replace('\u{2019}', "'");
            match piece.as_str() {
                "t" => tail.get(k + 1).is_some_and(|&(_, next)| {
                    AFTER_EUPHONIC_T.contains(&without_soft_hyphens(next).as_ref())
                }),
                pronoun if FRENCH_INVERSION_PRONOUNS.binary_search(&pronoun).is_ok() => true,
                elided => matches!(elided.split_once('\''), Some(("m" | "t" | "l", "en" | "y"))),
            }
        })
}

/// A French hyphenated run — pieces joined by single hyphens — read in order
/// (D5): a digit makes each piece a word of its own; a run the pack lists whole
/// is one token; an elision on the first piece is split off and the rest read
/// again; an inversion's pieces are words, each with its own span, the euphonic
/// `t` and the hyphens in none; any other run follows the compound rule.
fn push_french_run(
    tokens: &mut Vec<Token>,
    text: &str,
    pieces: &[(usize, &str)],
    lexicon: &(impl Lexicon + ?Sized),
) {
    let start = pieces[0].0;
    let end = pieces[pieces.len() - 1].0 + pieces[pieces.len() - 1].1.len();
    if text[start..end].chars().any(|c| c.is_ascii_digit()) {
        for &(at, piece) in pieces {
            push_french_word(tokens, text, piece, at, lexicon);
        }
        return;
    }
    if listed_whole(&text[start..end], lexicon) {
        push_compound(
            tokens,
            &text[start..end],
            pieces,
            start,
            end,
            FRENCH,
            lexicon,
        );
        return;
    }
    let (head_start, head) = push_french_elisions(tokens, pieces[0].1, start, false);
    let mut rest = pieces.to_vec();
    rest[0] = (head_start, head);
    let whole = &text[head_start..end];
    if !listed_whole(whole, lexicon) && is_french_inversion(&rest[1..]) {
        push_french_word(tokens, text, head, head_start, lexicon);
        for &(at, piece) in &rest[1..] {
            if without_soft_hyphens(piece) == "t" {
                continue;
            }
            let (at, piece) = push_french_elisions(tokens, piece, at, true);
            push_word(tokens, piece, at, at + piece.len(), FRENCH, lexicon);
        }
        return;
    }
    push_compound(tokens, whole, &rest, head_start, end, FRENCH, lexicon);
}

// Italian's pre-pass (add-lingua-italian-tokenisation, I7). The pieces it writes
// are lemmatised as any Italian word is, by Italian's cascade
// (add-lingua-italian-analysis, `analysis/italian.rs`); every comparison it makes
// reads the written word in NFC, and every token's text is composed (D1).

const ITALIAN: StudiedLanguage = StudiedLanguage::Italian;

/// Italian's elided words — what is written before the apostrophe, lowercase —
/// and the word each stands for (D3), sorted byte-wise by the written form; the
/// eighteen an Italian-specialist review found read as one unknown word — `trent'`,
/// `qual'`, `tant'`, `gliel'`, `ultim'` … — since refine-lingua-italian-reading (D2). One
/// reading depends on the context ([`italian_elided`]): `diss'` is `dissi` before
/// `io`. A form the list leaves out is read as written (`altr'ora` stays whole, and so
/// does `oltr'alpe`, which Italian writes `oltralpe`). Public for the checks that a pack
/// holds every word Italian's pre-pass reads.
pub const ITALIAN_ELISIONS: &[(&str, &str)] = &[
    // `alcun'altra` → `alcuna` + `altra` (the masculine elides without an apostrophe)
    ("alcun", "alcuna"),
    // `anch'io` → `anche` + `io`
    ("anch", "anche"),
    // `bell'uomo` → `bello` + `uomo`
    ("bell", "bello"),
    // `buon'anima` → `buona` + `anima` (the masculine elides without one: `buon uomo`)
    ("buon", "buona"),
    // `c'è` → `ci` + `è`
    ("c", "ci"),
    // `cent'anni` → `cento` + `anni`
    ("cent", "cento"),
    // `ch'io` → `che` + `io`
    ("ch", "che"),
    // `ciascun'altra` → `ciascuna` + `altra`
    ("ciascun", "ciascuna"),
    // `cinquant'anni` → `cinquanta` + `anni` (refine-lingua-italian-reading D2)
    ("cinquant", "cinquanta"),
    // `com'è` → `come` + `è`
    ("com", "come"),
    // `cos'è` → `cosa` + `è`
    ("cos", "cosa"),
    // `d'olio` → `di` + `olio` (also `da` in « d'ora in poi »; `di` is the commoner)
    ("d", "di"),
    // `dev'essere` → `deve` + `essere`
    ("dev", "deve"),
    // `diciott'anni` → `diciotto` + `anni`
    ("diciott", "diciotto"),
    // `diss'egli` → `disse` + `egli`; `diss'io` → `dissi` + `io`
    ("diss", "disse"),
    // `dov'è` → `dove` + `è`
    ("dov", "dove"),
    // `fin'ora` → `fino` + `ora` (the plain `fin` is an apocope, a word of its own)
    ("fin", "fino"),
    // `gl'italiani` → `gli` + `italiani`
    ("gl", "gli"),
    // `gliel'ho` → `glielo` + `ho`
    ("gliel", "glielo"),
    // `grand'uomo` → `grande` + `uomo`
    ("grand", "grande"),
    // `l'uomo` → `lo` + `uomo` (`la` would need the next word's gender; `il` never elides)
    ("l", "lo"),
    // `m'ha` → `mi` + `ha`
    ("m", "mi"),
    // `mezz'ora` → `mezzo` + `ora` (also `mezza`; the tables file both under one lemma)
    ("mezz", "mezzo"),
    // `n'è` → `ne` + `è`
    ("n", "ne"),
    // `neanch'io` → `neanche` + `io`
    ("neanch", "neanche"),
    // `nessun'altra` → `nessuna` + `altra` (the masculine elides without an apostrophe)
    ("nessun", "nessuna"),
    // `nient'altro` → `niente` + `altro`
    ("nient", "niente"),
    // `novant'anni` → `novanta` + `anni`
    ("novant", "novanta"),
    // `null'altro` → `nulla` + `altro`
    ("null", "nulla"),
    // `ogn'uno` → `ogni` + `uno` (literary)
    ("ogn", "ogni"),
    // `ottant'anni` → `ottanta` + `anni`
    ("ottant", "ottanta"),
    // `pover'uomo` → `povero` + `uomo` (also `povera`; the masculine, as `l'`)
    ("pover", "povero"),
    // `prim'ordine` → `primo` + `ordine` (the masculine, as `l'`)
    ("prim", "primo"),
    // `qual'è` → `quale` + `è` (a misspelling of « qual è », the web's commonest)
    ("qual", "quale"),
    // `qualcos'altro` → `qualcosa` + `altro`
    ("qualcos", "qualcosa"),
    // `qualcun'altra` → `qualcuna` + `altra`
    ("qualcun", "qualcuna"),
    // `quand'ero` → `quando` + `ero`
    ("quand", "quando"),
    // `quant'è` → `quanto` + `è`
    ("quant", "quanto"),
    // `quarant'anni` → `quaranta` + `anni`
    ("quarant", "quaranta"),
    // `quell'amico` → `quello` + `amico`
    ("quell", "quello"),
    // `quest'anno` → `questo` + `anno`
    ("quest", "questo"),
    // `s'era` → `si` + `era` (also `se` in literary « s'io fossi »; `si` is the commoner)
    ("s", "si"),
    // `sant'Antonio` → `santo` + `Antonio`
    ("sant", "santo"),
    // `senz'altro` → `senza` + `altro`
    ("senz", "senza"),
    // `sessant'anni` → `sessanta` + `anni`
    ("sessant", "sessanta"),
    // `settant'anni` → `settanta` + `anni`
    ("settant", "settanta"),
    // `sott'acqua` → `sotto` + `acqua`
    ("sott", "sotto"),
    // `t'amo` → `ti` + `amo`
    ("t", "ti"),
    // `tant'è` → `tanto` + `è`
    ("tant", "tanto"),
    // `trent'anni` → `trenta` + `anni`
    ("trent", "trenta"),
    // `tutt'altro` → `tutto` + `altro`
    ("tutt", "tutto"),
    // `l'ultim'ora` → `lo` + `ultima` + `ora` (its one common use, feminine)
    ("ultim", "ultima"),
    // `un'amica` → `una` + `amica`
    ("un", "una"),
    // `v'è` → `vi` + `è`
    ("v", "vi"),
    // `vent'anni` → `venti` + `anni`
    ("vent", "venti"),
    // `ventiquattr'ore` → `ventiquattro` + `ore`
    ("ventiquattr", "ventiquattro"),
];

/// Italian's articulated prepositions read as two words (D4, I7) — written
/// lowercase — and the preposition and the article each is read as, the two
/// sharing the written word's span, sorted byte-wise. The `di` family (`del`,
/// `dello`, `della`, `dei`, `degli`, `delle`) is not here: each is also the
/// partitive article and stays whole, as French's `du` and `des` do (M21); nor
/// are the archaic `collo`, `colla`, `cogli`, `colle` (also nouns), `pel` and
/// `pei`. Public for the same checks as [`ITALIAN_ELISIONS`].
pub const ITALIAN_CONTRACTIONS: &[(&str, &str, &str)] = &[
    ("agli", "a", "gli"),
    ("ai", "a", "i"),
    ("al", "a", "il"),
    ("alla", "a", "la"),
    ("alle", "a", "le"),
    ("allo", "a", "lo"),
    ("coi", "con", "i"),
    ("col", "con", "il"),
    ("dagli", "da", "gli"),
    ("dai", "da", "i"),
    ("dal", "da", "il"),
    ("dalla", "da", "la"),
    ("dalle", "da", "le"),
    ("dallo", "da", "lo"),
    ("negli", "in", "gli"),
    ("nei", "in", "i"),
    ("nel", "in", "il"),
    ("nella", "in", "la"),
    ("nelle", "in", "le"),
    ("nello", "in", "lo"),
    ("sugli", "su", "gli"),
    ("sui", "su", "i"),
    ("sul", "su", "il"),
    ("sulla", "su", "la"),
    ("sulle", "su", "le"),
    ("sullo", "su", "lo"),
];

/// Italian's elided articulated prepositions — what is written before the
/// apostrophe, lowercase — and the words each is read as (D4), sorted byte-wise:
/// `dell'` and `degl'` are the `di` family's and read whole as `dello` and `degli`, no
/// article after them (« degl'Italiani »); the others are the preposition and `lo`,
/// sharing the span, apostrophe included — the literary `coll'` too (« coll'aiuto »;
/// both since refine-lingua-italian-reading D2). Public for the same checks as
/// [`ITALIAN_ELISIONS`].
pub const ITALIAN_ELIDED_PREPOSITIONS: &[(&str, &str, Option<&str>)] = &[
    ("all", "a", Some("lo")),
    ("coll", "con", Some("lo")),
    ("dall", "da", Some("lo")),
    ("degl", "degli", None),
    ("dell", "dello", None),
    ("nell", "in", Some("lo")),
    ("sull", "su", Some("lo")),
];

/// Italian's truncations — the word written before its trailing apostrophe,
/// lowercase — and the word each stands for (D5), sorted byte-wise. The truncated
/// imperatives are read as their verb's present second person (`fa'` → `fai`,
/// `di'` → `dici`, of `dire`), a form the page could have written, which the forms
/// table files under the verb; none of them is a word with its apostrophe. The
/// books' `ca'`, `de'` and `que'` (« Ca' Foscari », « Lorenzo de' Medici », « que'
/// due ») are read as `casa`, `dei` — the `di` family's, whole — and `quei` since
/// refine-lingua-italian-reading (D3), and `da'` has left for
/// [`ITALIAN_TRUNCATED_PREPOSITIONS`]. A truncation's read word is final, and one
/// reading depends on the context ([`italian_truncation_read`]): `mo'` is `modo`
/// before `di` and `mò` otherwise (D4). Public for the same checks as
/// [`ITALIAN_ELISIONS`].
pub const ITALIAN_TRUNCATIONS: &[(&str, &str)] = &[
    ("be", "bene"),
    ("ca", "casa"),
    ("de", "dei"),
    ("di", "dici"),
    ("fa", "fai"),
    ("mo", "modo"),
    ("po", "poco"),
    ("que", "quei"),
    ("sta", "stai"),
    ("va", "vai"),
];

/// Italian's truncated articulated prepositions (refine-lingua-italian-reading D3) — the
/// word written before its trailing apostrophe, lowercase — and the preposition and the
/// article each is read as, the two sharing the written word's span, apostrophe included,
/// the first keeping the written capital, as an articulated preposition is read (I7): the
/// books write `ai`, `coi`, `dai`, `nei` and `sui` truncated before a consonant. Sorted
/// byte-wise. `a'`, `co'`, `ne'` and `su'` are read so only when a word follows them with
/// nothing but a space between — the preposition always has its noun phrase after it —,
/// and as apocopes otherwise, their apostrophe outside the word ([`ITALIAN_APOCOPES`]);
/// `da'` wherever it stands, the verb `dai` where its context says so
/// ([`italian_dai_is_the_verb`], D4). Public for the same checks as [`ITALIAN_ELISIONS`].
pub const ITALIAN_TRUNCATED_PREPOSITIONS: &[(&str, &str, &str)] = &[
    // `a' piedi` → `a` + `i` + `piedi`
    ("a", "a", "i"),
    // `co' suoi` → `con` + `i` + `suoi`
    ("co", "con", "i"),
    // `da' primi suoi anni` → `da` + `i` (Manzoni); `mi da' una mano` → the verb `dai`
    ("da", "da", "i"),
    // `ne' campi` → `in` + `i` + `campi`
    ("ne", "in", "i"),
    // `su' colli` → `su` + `i` + `colli`
    ("su", "su", "i"),
];

/// The words an Italian elided form or articulated preposition is read as: one,
/// or a preposition and an article sharing the written word's span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItalianReading {
    Word(&'static str),
    Contraction(&'static str, &'static str),
}

/// What an elided form is read as, or `None` when `written` (lowercase, without
/// its apostrophe) is not one (D3, D4): D3's words, then D4's elided articulated
/// prepositions. `next` is the word that follows, lowercase.
fn italian_elided(written: &str, next: &str) -> Option<ItalianReading> {
    if let Ok(index) = ITALIAN_ELISIONS.binary_search_by_key(&written, |&(form, _)| form) {
        return Some(ItalianReading::Word(match ITALIAN_ELISIONS[index].1 {
            "disse" if next == "io" => "dissi",
            read => read,
        }));
    }
    let index = ITALIAN_ELIDED_PREPOSITIONS
        .binary_search_by_key(&written, |&(form, _, _)| form)
        .ok()?;
    Some(match ITALIAN_ELIDED_PREPOSITIONS[index] {
        (_, preposition, Some(article)) => ItalianReading::Contraction(preposition, article),
        (_, read, None) => ItalianReading::Word(read),
    })
}

/// The articulated preposition `lower` is split into (D4), if it is one.
fn italian_contraction(lower: &str) -> Option<ItalianReading> {
    let index = ITALIAN_CONTRACTIONS
        .binary_search_by_key(&lower, |&(form, _, _)| form)
        .ok()?;
    let (_, preposition, article) = ITALIAN_CONTRACTIONS[index];
    Some(ItalianReading::Contraction(preposition, article))
}

/// The word a truncation stands for, if `lower` (without its apostrophe) is one (D5), as
/// its table writes it.
fn italian_truncation(lower: &str) -> Option<&'static str> {
    let index = ITALIAN_TRUNCATIONS
        .binary_search_by_key(&lower, |&(form, _)| form)
        .ok()?;
    Some(ITALIAN_TRUNCATIONS[index].1)
}

/// `mo'`, the one truncation read by the word after it (refine-lingua-italian-reading D4):
/// its table's `modo` before `di` (« a mo' di esempio »), and this, the Roman and southern
/// *now* (« mo' arrivo », « e mo'? »), otherwise.
const ITALIAN_MO: (&str, &str) = ("mo", "mò");

/// The word a truncation ending at byte `end` of `text` is read as (D5): its table's, and
/// for `mo'` its context's — `modo` when the next word, as read and with nothing but a space
/// between, is `di` (`di`, or the elided `d'` of « a mo' d'esempio »), `mò` otherwise
/// (refine-lingua-italian-reading D4).
fn italian_truncation_read(lower: &str, text: &str, end: usize) -> Option<&'static str> {
    let read = italian_truncation(lower)?;
    Some(
        if lower == ITALIAN_MO.0 && !italian_next_word_reads_di(text, end) {
            ITALIAN_MO.1
        } else {
            read
        },
    )
}

/// Whether the word after byte `end` of `text`, nothing but a space between, reads `di`: the
/// word `di`, or the elided `d'` before a letter (D4).
fn italian_next_word_reads_di(text: &str, end: usize) -> bool {
    let rest = &text[end..];
    let after = rest.trim_start_matches(char::is_whitespace);
    if after.len() == rest.len() {
        return false;
    }
    let letters = after
        .char_indices()
        .find(|&(_, c)| !c.is_alphabetic())
        .map_or(after.len(), |(at, _)| at);
    let word = french_lowercase(&after[..letters]);
    let mut tail = after[letters..].chars();
    if tail.next().is_some_and(is_apostrophe) && tail.next().is_some_and(char::is_alphabetic) {
        italian_elided(&word, "") == Some(ItalianReading::Word("di"))
    } else {
        word == "di"
    }
}

/// The preposition and the article a truncated articulated preposition is read as, if
/// `lower` (without its apostrophe) is one of the table's (refine-lingua-italian-reading D3),
/// whatever follows it.
fn italian_truncated_preposition(lower: &str) -> Option<(&'static str, &'static str)> {
    let index = ITALIAN_TRUNCATED_PREPOSITIONS
        .binary_search_by_key(&lower, |&(form, _, _)| form)
        .ok()?;
    let (_, preposition, article) = ITALIAN_TRUNCATED_PREPOSITIONS[index];
    Some((preposition, article))
}

/// `da'`, the truncated articulated preposition read wherever it stands, by its context
/// (D4); the other four only before a word (D3).
const ITALIAN_DA: &str = "da";

/// Whether Italian's pre-pass reads `lower` (without its apostrophe) as a truncated
/// articulated preposition with the apostrophe that ends at byte `end` of `text` (D3): `da'`
/// wherever it stands; `a'`, `co'`, `ne'` and `su'` only when a word follows them with nothing
/// but a space between ([`italian_word_follows`]).
fn italian_reads_truncated_preposition(lower: &str, text: &str, end: usize) -> bool {
    italian_truncated_preposition(lower).is_some()
        && (lower == ITALIAN_DA || italian_word_follows(text, end))
}

/// Whether a word follows byte `end` of `text` with nothing but a space between: one or more
/// whitespace characters, then a letter.
fn italian_word_follows(text: &str, end: usize) -> bool {
    let rest = &text[end..];
    let after = rest.trim_start_matches(char::is_whitespace);
    after.len() < rest.len() && after.starts_with(char::is_alphabetic)
}

/// What a word the seam kept its trailing apostrophe for — a truncation, a truncated
/// articulated preposition — reads as, written from `start` to `end` of `text`, the apostrophe
/// included: a truncation's word (`mo'` by the word after it, [`italian_truncation_read`]); a
/// truncated articulated preposition's preposition and `i` — but `da'`, the verb `dai`, one
/// word, where [`italian_dai_is_the_verb`] says so (refine-lingua-italian-reading D3, D4).
fn italian_truncated_reading(
    lower: &str,
    text: &str,
    start: usize,
    end: usize,
) -> Option<ItalianReading> {
    if let Some(read) = italian_truncation_read(lower, text, end) {
        return Some(ItalianReading::Word(read));
    }
    let (preposition, article) = italian_truncated_preposition(lower)?;
    Some(
        if lower == ITALIAN_DA && italian_dai_is_the_verb(text, start, end) {
            ItalianReading::Word(ITALIAN_DAI)
        } else {
            ItalianReading::Contraction(preposition, article)
        },
    )
}

/// `dai`, the articulated preposition `da` + `i` (I7) and the verb *dare*'s present second
/// person, and the word the truncated `da'` is read as where it is the verb
/// (refine-lingua-italian-reading D4).
const ITALIAN_DAI: &str = "dai";

/// The clitic pronouns after which `dai` and `da'` are the verb (D4): a clitic is never
/// followed by a preposition (« mi dai una mano? », « me lo dai? », « gliene dai »). Sorted
/// for the search. Measured (D5): the verb in each of the 79 occurrences it read as the verb
/// on the corpus — `mi` 50, `gli` 8, `la` 5, `lo` 4, `ci`, `le`, `ti` 3 each, `glielo` 2,
/// `gliela` 1. `me` and `te`, clitics only before another pronoun and stressed pronouns
/// elsewhere (« vieni con me dai nonni? »), were a trigger of their own, kept only on their own
/// figures: the corpus wrote neither before `dai` or `da'`, and they were left out.
const ITALIAN_DAI_CLITICS: &[&str] = &[
    "ce", "ci", "gli", "gliela", "gliele", "glieli", "glielo", "gliene", "la", "le", "lo", "mi",
    "ne", "ti",
];

/// The word right before byte `start` of `text` when nothing but a space separates the two —
/// one or more whitespace characters after its letters —, lowercase and in NFC; `None` at the
/// start of the text or after punctuation.
fn italian_word_before(text: &str, start: usize) -> Option<String> {
    let before = &text[..start];
    let word = before.trim_end_matches(char::is_whitespace);
    if word.len() == before.len() {
        return None;
    }
    let from = word
        .char_indices()
        .rev()
        .take_while(|&(_, c)| {
            c.is_alphabetic()
                || c == SOFT_HYPHEN
                || unicode_normalization::char::is_combining_mark(c)
        })
        .last()?
        .0;
    Some(french_lowercase(&word[from..]))
}

/// D4's clitic trigger: a clitic pronoun right before ([`ITALIAN_DAI_CLITICS`]).
fn italian_dai_after_a_clitic(before: Option<&str>) -> bool {
    before.is_some_and(|word| ITALIAN_DAI_CLITICS.binary_search(&word).is_ok())
}

/// D4's punctuation trigger: right after the word, `,`, `!`, `?`, `.`, `…` or a closing
/// quotation mark (`"`, `”`, `»`, `'`, `’`, `›`), or the end of the text — nothing, or nothing
/// but spaces, after it: the preposition is never followed by punctuation (« Dai, andiamo! »,
/// « Ma dai! », « “Ma dai” disse »). Measured (D5): the verb in 252 of the 253 occurrences it
/// read as the verb on the corpus (99.6 %), the other a name (« dell'etnia Dai »).
fn italian_dai_before_punctuation(text: &str, end: usize) -> bool {
    let rest = &text[end..];
    match rest.chars().next() {
        None => true,
        Some(c) if c.is_whitespace() => rest.trim_start_matches(char::is_whitespace).is_empty(),
        Some(c) => matches!(
            c,
            ',' | '!' | '?' | '.' | '…' | '"' | '\u{201D}' | '»' | '\'' | '\u{2019}' | '›'
        ),
    }
}

/// Whether `dai` or `da'`, written from `start` to `end` of `text`, is the verb *dare*
/// rather than the preposition `da` + `i` (refine-lingua-italian-reading D4, each trigger
/// measured on its own by D5): a clitic pronoun right before it
/// ([`italian_dai_after_a_clitic`]), or punctuation, a closing quotation mark or the end of the
/// text right after it ([`italian_dai_before_punctuation`]). The design's third trigger, `non`
/// right before, was measured out (D5: the verb in 7 of its 8 readings, short of 95 % — « fatte
/// dagli uomini e non dai computer » is the preposition), and so were `me` and `te`
/// ([`ITALIAN_DAI_CLITICS`]). The verb with no sign — « tu dai il libro a Maria », « non dai
/// retta », « Dai una mano a Tom » — reads as the preposition, as before.
fn italian_dai_is_the_verb(text: &str, start: usize, end: usize) -> bool {
    let before = italian_word_before(text, start);
    italian_dai_after_a_clitic(before.as_deref()) || italian_dai_before_punctuation(text, end)
}

/// The literary apocopes and truncations the apostrophe for an accent is never read on
/// (read-lingua-italian-unmarked-spellings D1), written lowercase without their apostrophe,
/// sorted byte-wise for the search: those Italian's pre-pass does not read. The truncated
/// articulated prepositions `a'` (*ai*), `co'` (*coi*), `ne'` (*nei*) and `su'` (*sui*) with no
/// word after them, which the pre-pass reads as two words only before one
/// ([`ITALIAN_TRUNCATED_PREPOSITIONS`], refine-lingua-italian-reading D3) — with nothing after
/// them `ne'` would read `né` and `su'` `sù`, wrong every time in a book —; `pe'` (*pei*); and the
/// eight the measurement of change 10's task 2.2 found read as no word of theirs in the
/// public-domain books and the Wikipedia sample, each a word the tables list with the accent and
/// Italian does not write so: `ve'` (*vedi*, `vè`), `mi'` (*mio*, `mì`), `fra'` (*frate*,
/// `frà`), `to'` (*togli*, `tò`, the interjection « tiens ! »), `i'` (*io*, `ì`), `me'`
/// (*meglio*, `mè`), `vo'` (*voglio*, `vò`) and `fe'` (*fece*, `fè`). `de'`, `que'` and `ca'`
/// have left the list: the pre-pass reads them wherever they stand (`dei`, `quei`, `casa`;
/// [`ITALIAN_TRUNCATIONS`]). Each stays as an unlisted trailing apostrophe is read, its
/// apostrophe outside the word. Private: the rule's own list, which no table holds.
const ITALIAN_APOCOPES: &[&str] = &[
    "a", "co", "fe", "fra", "i", "me", "mi", "ne", "pe", "su", "to", "ve", "vo",
];

/// The accented form an Italian word reads as when its apostrophe stands for its accent
/// (read-lingua-italian-unmarked-spellings D1), or `None`: `lower` — the word, or what remains
/// of it after its elided pieces, lowercase, in NFC and without the apostrophe — ending in a
/// vowel, that vowel accented (`a` → `à`, `e` → `è` or `é`, `i` → `ì`, `o` → `ò`, `u` → `ù`),
/// as the pack lists it (`perche` → `perché`, `e` → `è`, `citta` → `città`) and ranks it within
/// [`ITALIAN_ACCENT_RANK`]: a form whose word the pack ranks lower is a rare token the tables
/// list (`vità`, `comè`, `dovè`, `chì`, `nò`, `luì`), and the apostrophe a stray one. Where the
/// pack lists both of `e`'s, a form it files under the other is the other's spelling, and the
/// other wins (`perché`, beside `perchè` a form of it; `caffè`, beside `caffé`); where neither is
/// filed under the other, the one whose word the pack ranks higher (`sé` 655 before `sè` 2,911;
/// `è`, a form of *essere*, before the misspelling `é`), and `è` before `é` where the pack ranks
/// neither: the tables decide, never a frequency read at run time. Change 6's truncations are
/// never tried — `po'` is `poco`, never `pò` —, nor the truncated articulated prepositions
/// (`da'` is `da` + `i` or the verb `dai`, never `dà`; refine-lingua-italian-reading D3), nor the
/// apocopes ([`ITALIAN_APOCOPES`]). A lexicon without ranks (a test's stand-in) passes every
/// listed form.
fn italian_accented(lower: &str, lexicon: &(impl Lexicon + ?Sized)) -> Option<String> {
    if italian_truncation(lower).is_some()
        || italian_truncated_preposition(lower).is_some()
        || ITALIAN_APOCOPES.binary_search(&lower).is_ok()
    {
        return None;
    }
    let vowel = lower.chars().next_back()?;
    let accents: &[char] = match vowel {
        'a' => &['à'],
        'e' => &['è', 'é'],
        'i' => &['ì'],
        'o' => &['ò'],
        'u' => &['ù'],
        _ => return None,
    };
    let stem = &lower[..lower.len() - vowel.len_utf8()];
    let listed: Vec<(String, &str)> = accents
        .iter()
        .map(|accent| format!("{stem}{accent}"))
        .filter_map(|form| {
            let lemma = lexicon.lemma_of(&form)?;
            Some((form, lemma))
        })
        .collect();
    let spells_another = |(form, lemma): &(String, &str)| {
        listed
            .iter()
            .any(|(other, _)| other != form && other.as_str() == *lemma)
    };
    let own: Vec<&(String, &str)> = listed.iter().filter(|c| !spells_another(c)).collect();
    let candidates = if own.is_empty() {
        listed.iter().collect()
    } else {
        own
    };
    candidates
        .into_iter()
        .map(|(form, lemma)| (form, lexicon.lemma_rank(lemma)))
        .filter(|(_, rank)| rank.is_none_or(|rank| rank <= ITALIAN_ACCENT_RANK))
        .enumerate()
        .min_by_key(|&(order, (_, rank))| (rank.is_none(), rank, order))
        .map(|(_, (form, _))| form.clone())
}

/// The rank within which an accented form's word is read for an apostrophe
/// (read-lingua-italian-unmarked-spellings, at review): measured on it-fr's tables between the
/// rarest word a real accent spelling reads as (`piè` 15,547, of « pie' ») and the commonest rare
/// token a stray apostrophe would read (`vità` 33,926), at their geometric middle rounded to
/// thousands; `governò` (*governare*, 3,722) and `terrà` (*tenere*, 684) are kept.
const ITALIAN_ACCENT_RANK: u32 = 23_000;

/// Whether an Italian word is an elided form (D3's, or D4's `dell'`, `all'`,
/// `dall'`, `nell'`, `sull'` …), and whether it is a truncation or a truncated articulated
/// preposition the pre-pass reads with the apostrophe ending at byte `end` of `text`
/// ([`italian_reads_truncated_preposition`], refine-lingua-italian-reading D3), read
/// lowercase, in NFC and without its soft hyphens: `(elided, truncation)`.
fn italian_listed_before_apostrophe(word: &str, text: &str, end: usize) -> (bool, bool) {
    let lower = french_lowercase(word);
    (
        italian_elided(&lower, "").is_some(),
        italian_truncation(&lower).is_some()
            || italian_reads_truncated_preposition(&lower, text, end),
    )
}

/// The character before byte `at` of `text`, a soft hyphen read as nowhere.
fn italian_char_before(text: &str, at: usize) -> Option<char> {
    text[..at].chars().rev().find(|&c| c != SOFT_HYPHEN)
}

/// Whether a single quote opened before byte `start` of `text` is still open there,
/// so that an apostrophe after the word starting at `start` closes it (D5, as
/// implemented). The nearest quote mark before the word — `'`, `’` or `‘` — that
/// opens or closes one decides: a mark with a letter after it and no letter or digit
/// before it opens one (`‘due`, `'Sì`), and so does one after a space or the start
/// before `.`, `…` or a number (`'...lavoro'`, `'2 lavoro'`) but an elided decade
/// (`‘80`, two digits and no letter or digit after them); a mark with no letter after
/// it and something other than a space before it closes one (`fa’`, `qui!’`, `Sì.'`),
/// unless the word before it is one Italian's pre-pass reads with its apostrophe — a
/// truncation (`po'`), the truncated `da'` (refine-lingua-italian-reading D3) or a word whose
/// apostrophe stands for an accent (`e'`, [`italian_accented`]; read-lingua-italian-unmarked-spellings,
/// at review) — and no quote mark opened right before it ([`reads_its_apostrophe`]), whose
/// apostrophe is its own. A mark between two letters is inside a word (`c'è`, a
/// mistyped `l‘uomo`), and one between a space and anything else opens nothing: the
/// scan goes on past them, and the start of the text is outside any quote. So
/// « ‘due anni fa’ » reads `fa`, « ‘Vieni!’ disse, un po' », « anni ‘80, un po' » and
/// « un po' di pane, va' via » `poco`, and « 'un po' di lavoro' » and « 'c'e' lavoro' »
/// leave `lavoro` as written.
fn inside_a_quote(text: &str, start: usize, lexicon: &(impl Lexicon + ?Sized)) -> bool {
    for (at, c) in text[..start].char_indices().rev() {
        if !is_apostrophe(c) && c != '\u{2018}' {
            continue;
        }
        let before = italian_char_before(text, at);
        let mut after = text[at + c.len_utf8()..]
            .chars()
            .filter(|&c| c != SOFT_HYPHEN);
        let next = after.next();
        let letter_after = next.is_some_and(char::is_alphabetic);
        let letter_before = before.is_some_and(char::is_alphanumeric);
        let space_before = before.is_none_or(char::is_whitespace);
        if letter_after && !letter_before {
            return true;
        }
        if space_before && opens_before(next, after) {
            return true;
        }
        if !letter_after && !space_before {
            if reads_its_apostrophe(text, at, lexicon) {
                continue;
            }
            return false;
        }
    }
    false
}

/// Whether a quote mark after a space opens a quote by what follows it: `.` or `…`
/// (`'...lavoro'`), or a number but an elided decade — two digits and neither a letter
/// nor a digit after them (`‘80`, `'68`).
fn opens_before(next: Option<char>, mut after: impl Iterator<Item = char>) -> bool {
    match next {
        Some('.' | '…') => true,
        Some(first) if first.is_ascii_digit() => {
            let decade = after.next().is_some_and(|c| c.is_ascii_digit())
                && !after.next().is_some_and(char::is_alphanumeric);
            !decade
        }
        _ => false,
    }
}

/// Whether the word ending at byte `at` of `text`, right before an apostrophe, is one
/// Italian's pre-pass reads with its apostrophe, so that the apostrophe closes no quote: a
/// truncation, the truncated `da'` (refine-lingua-italian-reading D3), or a word whose
/// apostrophe stands for an accent ([`italian_accented`]). The word is the letters right
/// before the apostrophe (an elided piece's rest: `e` of `c'e'`), soft hyphens read as
/// nowhere. Two cases close the quote all the same (at review): a word a quote mark opens right
/// before — the whole quoted span ([`italian_quoted_alone`]: « la particella 'ne' », « si dice
/// 'po' ») —; and `a'`, `co'`, `ne'` and `su'`, which the pre-pass reads with their apostrophe
/// only before a word and with no quote open before them — where it does, no quote was open
/// for the apostrophe to close, so closing it changes nothing, and where a quote is open, the
/// apostrophe is that quote's (« 'Vieni su' gli disse »).
fn reads_its_apostrophe(text: &str, at: usize, lexicon: &(impl Lexicon + ?Sized)) -> bool {
    let from = text[..at]
        .char_indices()
        .rev()
        .take_while(|&(_, c)| {
            c.is_alphabetic()
                || c == SOFT_HYPHEN
                || unicode_normalization::char::is_combining_mark(c)
        })
        .last()
        .map_or(at, |(i, _)| i);
    let lower = french_lowercase(&text[from..at]);
    if lower.is_empty() || italian_quoted_alone(text, from) {
        return false;
    }
    italian_truncation(&lower).is_some()
        || lower == ITALIAN_DA
        || italian_accented(&lower, lexicon).is_some()
}

/// Whether a quote mark opens right before byte `from` of `text`: `'`, `’` or `‘` right before
/// it, after no letter or digit — so that the word starting there is a whole quoted span (`'ne'`,
/// `‘po’`), and the apostrophe after it that quote's closing mark. The apostrophe of `c'` in
/// `c'e'` follows a letter and opens nothing. Soft hyphens are read as nowhere.
fn italian_quoted_alone(text: &str, from: usize) -> bool {
    let mut before = text[..from].chars().rev().filter(|&c| c != SOFT_HYPHEN);
    before
        .next()
        .is_some_and(|c| is_apostrophe(c) || c == '\u{2018}')
        && !before.next().is_some_and(char::is_alphanumeric)
}

/// The words of Italian text with their byte offsets (D5, D2's counterpart of
/// [`french_words`]): UAX #29's, a word followed by an apostrophe and no letter
/// taking that apostrophe into its own when it is listed
/// ([`italian_listed_before_apostrophe`]) — an elided form written alone (`l’`, as a
/// word card is handed it) or before a number (`l'11`, `nell'800`, whose digits
/// the digit rule then drops), a truncation (`po'`) or a truncated articulated preposition
/// the pre-pass reads there (`da'`; `ne'` before a word, refine-lingua-italian-reading D3)
/// before neither a letter nor a digit —, and, unless a digit follows, no quote opened
/// before it is left open
/// ([`inside_a_quote`]). Any other trailing apostrophe stays outside the word here,
/// as the rule of no language leaves it — `‘ciao’` is `ciao` —; [`push_italian_word`]
/// then reads one that stands for an accent, taking it into the token's span
/// (`perche'` is `perche` here and `perché` there, read-lingua-italian-unmarked-spellings
/// D1).
fn italian_words<'t>(text: &'t str, lexicon: &(impl Lexicon + ?Sized)) -> Vec<(usize, &'t str)> {
    text.unicode_word_indices()
        .map(|(start, word)| {
            let end = start + word.len();
            let mut after = text[end..].chars();
            let Some(apostrophe) = after.next().filter(|&c| is_apostrophe(c)) else {
                return (start, word);
            };
            let (elided, truncation) =
                italian_listed_before_apostrophe(word, text, end + apostrophe.len_utf8());
            let keeps = match after.next() {
                Some(c) if c.is_alphabetic() => false,
                Some(c) if c.is_alphanumeric() => elided,
                _ => (elided || truncation) && !inside_a_quote(text, start, lexicon),
            };
            if keeps {
                (start, &text[start..end + apostrophe.len_utf8()])
            } else {
                (start, word)
            }
        })
        .collect()
}

/// Pushes the token or the two tokens `reading` makes of `written`, each spanning
/// `start..end`: the first word takes `written`'s capital on its own first letter
/// (`L'` → `Lo`, `Nel` → `In`), an article never does.
fn push_italian_reading(
    tokens: &mut Vec<Token>,
    reading: ItalianReading,
    written: &str,
    start: usize,
    end: usize,
) {
    let (first, article) = match reading {
        ItalianReading::Word(read) => (read, None),
        ItalianReading::Contraction(preposition, article) => (preposition, Some(article)),
    };
    for read in std::iter::once(french_cased(first, written)).chain(article.map(str::to_owned)) {
        tokens.push(Token {
            text: read,
            start,
            end,
            parts: Vec::new(),
        });
    }
}

/// Splits the elided words off the front of an Italian word written at `start`
/// (D3, D4): each is read as the word it stands for — `dell'` as `dello`, `all'`
/// as `a` + `lo` sharing the span — spanning its letters and its apostrophe, and
/// the rule runs again on what follows, as long as a letter follows the
/// apostrophe. Returns where the rest starts and the rest. A soft hyphen is read
/// as nowhere, in the elided piece, in the word after it and right after the
/// apostrophe (ignore-lingua-soft-hyphens D2); the written piece and the word after
/// it are compared in NFC (add-lingua-italian-analysis D1).
fn push_italian_elisions<'a>(
    tokens: &mut Vec<Token>,
    word: &'a str,
    start: usize,
) -> (usize, &'a str) {
    let mut at = start;
    let mut rest = word;
    while let Some((i, apostrophe)) = rest.char_indices().find(|&(_, c)| is_apostrophe(c)) {
        let written = &rest[..i];
        let after = &rest[i + apostrophe.len_utf8()..];
        if !after
            .trim_start_matches(SOFT_HYPHEN)
            .starts_with(char::is_alphabetic)
        {
            break;
        }
        let next: String = after
            .chars()
            .filter(|&c| c != SOFT_HYPHEN)
            .take_while(|c| c.is_alphabetic())
            .flat_map(char::to_lowercase)
            .nfc()
            .collect();
        let Some(reading) = italian_elided(&french_lowercase(written), &next) else {
            break;
        };
        let end = at + i + apostrophe.len_utf8();
        push_italian_reading(tokens, reading, written, at, end);
        at = end;
        rest = after;
    }
    (at, rest)
}

/// One Italian word written at `start` of `text` — outside a hyphenated run, or a
/// piece of a run holding a digit: its elisions (D3); then a word the seam kept its
/// apostrophe for, a truncation read as its word (`mo'` by the word after it), a truncated
/// articulated preposition read as its preposition and `i` sharing the span (`da'` the verb
/// `dai` by its context; [`italian_truncated_reading`], refine-lingua-italian-reading D3, D4)
/// or an elided form written alone read as the word it stands for, spanning the apostrophe
/// (D5); then a
/// word, or the rest of one after its elisions, whose apostrophe — neither a letter
/// nor a digit after it, no quote open before the written word ([`inside_a_quote`])
/// — stands for the accent of a form the pack lists, read as that form, its written
/// capital kept, one token spanning the apostrophe ([`italian_accented`],
/// read-lingua-italian-unmarked-spellings D1: `perche'` → `perché`, `E'` → `È`, `c'e'`
/// → `ci` + `è`); then `dai` read as the verb where its context says so
/// ([`italian_dai_is_the_verb`], refine-lingua-italian-reading D4), else an articulated
/// preposition split into the preposition and the article sharing the span, the `di` family
/// whole (D4); or the rules every language shares ([`push_word`]). Each check reads the word composed and each token's text
/// is composed (add-lingua-italian-analysis D1); the spans are the source's.
fn push_italian_word(
    tokens: &mut Vec<Token>,
    text: &str,
    word: &str,
    start: usize,
    lexicon: &(impl Lexicon + ?Sized),
) {
    let (at, rest) = push_italian_elisions(tokens, word, start);
    let end = at + rest.len();
    let lower = french_lowercase(rest);
    // The seam kept the apostrophe of a listed word only: a truncation, or an elided form alone.
    if let Some(bare) = lower.strip_suffix(is_apostrophe) {
        let reading =
            italian_truncated_reading(bare, text, at, end).or_else(|| italian_elided(bare, ""));
        if let Some(reading) = reading {
            push_italian_reading(tokens, reading, rest, at, end);
            return;
        }
    } else if let Some(apostrophe) = apostrophe_ending_a_word(text, end)
        && !inside_a_quote(text, start, lexicon)
        && let Some(accented) = italian_accented(&lower, lexicon)
    {
        tokens.push(Token {
            text: french_cased(&accented, rest),
            start: at,
            end: end + apostrophe,
            parts: Vec::new(),
        });
        return;
    }
    if lower == ITALIAN_DAI && italian_dai_is_the_verb(text, at, end) {
        push_italian_reading(tokens, ItalianReading::Word(ITALIAN_DAI), rest, at, end);
        return;
    }
    match italian_contraction(&lower) {
        Some(reading) => push_italian_reading(tokens, reading, rest, at, end),
        None => push_word(tokens, rest, at, end, ITALIAN, lexicon),
    }
}

/// `tokens`, read from `text`, with each word Italian's pre-pass reads by its context read the
/// other way (D4): the verb `dai` as `da` + `i` sharing its span and back, `modo` of `mo'` as
/// `mò` and back, and the apocope `a`, `co`, `ne` or `su` with an apostrophe right after it as
/// the preposition and `i`, the two spanning the word and its apostrophe (D3, at review); each
/// first word keeping the written capital. `None` when `text` holds no such word — `da'` inside a
/// quote, read `da` as written, is none —; else the other reading, and whether it goes first: it
/// does where it reads a truncated articulated preposition, whose apostrophe only it spans.
fn italian_other_reading(text: &str, tokens: &[Token]) -> Option<(Vec<Token>, bool)> {
    let mut other = Vec::with_capacity(tokens.len() + 1);
    let mut flipped = false;
    let mut first = false;
    let mut i = 0;
    while i < tokens.len() {
        let span = (tokens[i].start, tokens[i].end);
        let group = tokens[i..]
            .iter()
            .take_while(|t| (t.start, t.end) == span)
            .count();
        let written = &text[span.0..span.1];
        let lower = french_lowercase(written).replace('\u{2019}', "'");
        let read: Vec<String> = tokens[i..i + group]
            .iter()
            .map(|t| t.text.to_lowercase())
            .collect();
        let read: Vec<&str> = read.iter().map(String::as_str).collect();
        // An apocope with its apostrophe right after it: the truncated articulated preposition
        // a page reads before a word, spanning the apostrophe too.
        let apostrophe = text[span.1..]
            .chars()
            .next()
            .filter(|&c| is_apostrophe(c))
            .map_or(0, char::len_utf8);
        if apostrophe > 0
            && let [apocope] = read.as_slice()
            && *apocope == lower
            && *apocope != ITALIAN_DA
            && let Some((preposition, article)) = italian_truncated_preposition(apocope)
        {
            let end = span.1 + apostrophe;
            let reading = ItalianReading::Contraction(preposition, article);
            push_italian_reading(&mut other, reading, &text[span.0..end], span.0, end);
            flipped = true;
            first = true;
            i += group;
            continue;
        }
        let flip = match (lower.as_str(), read.as_slice()) {
            ("dai" | "da'", [ITALIAN_DAI]) => Some(ItalianReading::Contraction(ITALIAN_DA, "i")),
            ("dai" | "da'", [ITALIAN_DA, "i"]) => Some(ItalianReading::Word(ITALIAN_DAI)),
            ("mo'", ["modo"]) => Some(ItalianReading::Word(ITALIAN_MO.1)),
            ("mo'", [mo]) if *mo == ITALIAN_MO.1 => Some(ItalianReading::Word("modo")),
            _ => None,
        };
        match flip {
            Some(reading) => {
                push_italian_reading(&mut other, reading, written, span.0, span.1);
                flipped = true;
            }
            None => other.extend_from_slice(&tokens[i..i + group]),
        }
        i += group;
    }
    flipped.then_some((other, first))
}

/// An Italian hyphenated run — pieces joined by single hyphens — read in order
/// (D3, D4): a digit makes each piece a word of its own, as French's run does; an
/// elision on the first piece is split off (`l'anglo-americano` → `lo` +
/// `anglo-americano`); the rest follows the compound rule, nothing split inside
/// it (`al-Qaida` stays whole). Italian has no hyphenated inversion.
fn push_italian_run(
    tokens: &mut Vec<Token>,
    text: &str,
    pieces: &[(usize, &str)],
    lexicon: &(impl Lexicon + ?Sized),
) {
    let start = pieces[0].0;
    let end = pieces[pieces.len() - 1].0 + pieces[pieces.len() - 1].1.len();
    if text[start..end].chars().any(|c| c.is_ascii_digit()) {
        for &(at, piece) in pieces {
            push_italian_word(tokens, text, piece, at, lexicon);
        }
        return;
    }
    let (head_start, head) = push_italian_elisions(tokens, pieces[0].1, start);
    let mut rest = pieces.to_vec();
    rest[0] = (head_start, head);
    push_compound(
        tokens,
        &text[head_start..end],
        &rest,
        head_start,
        end,
        ITALIAN,
        lexicon,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lexicon::{FstLexicon, build_lexicon_blobs};

    fn lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) =
            build_lexicon_blobs(&[], &["a", "i", "do", "not", "ship", "code", "team"])
                .expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    fn texts(tokens: &[Token]) -> Vec<&str> {
        tokens.iter().map(|t| t.text.as_str()).collect()
    }

    #[test]
    fn spec_scenario_english_contractions_are_englishs() {
        let lex = lexicon();
        assert_eq!(
            texts(&tokenize("Don't", StudiedLanguage::English, &lex)),
            ["Do", "not"]
        );
        assert_eq!(
            texts(&tokenize("Don't", StudiedLanguage::Spanish, &lex)),
            ["Don't"]
        );
    }

    #[test]
    fn the_rules_that_belong_to_no_language_apply_to_spanish_too() {
        // Compounds, the digit drop, edge apostrophes and the single-letter rule.
        let lex = lexicon();
        let text = "x-ray abc123 'team' a b well-being-2 code";
        assert_eq!(
            texts(&tokenize(text, StudiedLanguage::Spanish, &lex)),
            texts(&tokenize(text, StudiedLanguage::English, &lex))
        );
    }

    #[test]
    fn spec_scenario_two_spanish_contractions() {
        let lex = lexicon();
        let tokens = tokenize(
            "Vengo del mercado al centro.",
            StudiedLanguage::Spanish,
            &lex,
        );
        assert_eq!(
            texts(&tokens),
            ["Vengo", "de", "el", "mercado", "a", "el", "centro"]
        );
        // The two halves share the source span, as `don't` does.
        assert_eq!((tokens[1].start, tokens[1].end), (6, 9));
        assert_eq!((tokens[2].start, tokens[2].end), (6, 9));
        assert_eq!((tokens[4].start, tokens[4].end), (18, 20));
    }

    #[test]
    fn spec_scenario_a_capitalised_spanish_contraction() {
        let lex = lexicon();
        let tokens = tokenize("Del mar. AL fin.", StudiedLanguage::Spanish, &lex);
        assert_eq!(texts(&tokens), ["De", "el", "mar", "A", "el", "fin"]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, 3));
        assert_eq!((tokens[1].start, tokens[1].end), (0, 3));
    }

    #[test]
    fn spanish_contractions_are_not_split_inside_a_compound_nor_in_english() {
        let lex = lexicon();
        assert_eq!(
            texts(&tokenize("al-andalus", StudiedLanguage::Spanish, &lex)),
            ["al-andalus"]
        );
        assert_eq!(
            texts(&tokenize("al del", StudiedLanguage::English, &lex)),
            ["al", "del"]
        );
    }

    #[test]
    fn spec_scenario_a_decomposed_accent_is_read_composed() {
        let lex = lexicon();
        // `está` written as `esta` + U+0301 COMBINING ACUTE ACCENT.
        let text = "Esta\u{0301} aquí.";
        let tokens = tokenize(text, StudiedLanguage::Spanish, &lex);
        assert_eq!(texts(&tokens), ["Está", "aquí"]);
        // The span still points into the source text, combining mark included.
        assert_eq!(&text[tokens[0].start..tokens[0].end], "Esta\u{0301}");
        // English text is never normalised: its output must not move.
        assert_eq!(
            texts(&tokenize(text, StudiedLanguage::English, &lex)),
            ["Esta\u{0301}", "aquí"]
        );
    }

    const FR: StudiedLanguage = StudiedLanguage::French;

    /// A stand-in French pack: the one-letter words it keeps, and the hyphenated runs it lists
    /// whole.
    fn french_lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) = build_lexicon_blobs(
            &[
                ("a", "avoir"),
                ("rendez-vous", "rendez-vous"),
                ("peut-être", "peut-être"),
                ("arc-en-ciel", "arc-en-ciel"),
                ("c'est-à-dire", "c'est-à-dire"),
            ],
            &["à", "y", "le", "de", "homme"],
        )
        .expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    /// Each token's text beside the source text its span covers.
    fn read<'t>(text: &'t str, tokens: &'t [Token]) -> Vec<(&'t str, &'t str)> {
        tokens
            .iter()
            .map(|t| (t.text.as_str(), &text[t.start..t.end]))
            .collect()
    }

    fn french(text: &str) -> Vec<Token> {
        tokenize(text, FR, &french_lexicon())
    }

    #[test]
    fn spec_scenario_french_punctuation() {
        // D2: UAX #29 glues U+202F (ExtendNumLet) to the word beside it; French reads it as a space.
        let text = "«\u{202F}C’est fini\u{202F}!\u{202F}»";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [("Ce", "C’"), ("est", "est"), ("fini", "fini")]
        );
        assert_eq!((tokens[0].start, tokens[0].end), (5, 9));
        assert_eq!((tokens[1].start, tokens[1].end), (9, 12));
        assert!(tokens.iter().all(|t| !t.text.contains('\u{202F}')
            && !text[t.start..t.end].contains('\u{202F}')));
        // A narrow no-break space inside a UAX #29 word cuts it, the space in no part.
        let inside = "pas\u{202F}encore";
        assert_eq!(
            read(inside, &french(inside)),
            [("pas", "pas"), ("encore", "encore")]
        );
        // The no-break space U+00A0, set before `:`, never glued: nothing to do.
        let colon = "Note\u{A0}: fini";
        assert_eq!(
            read(colon, &french(colon)),
            [("Note", "Note"), ("fini", "fini")]
        );
    }

    #[test]
    fn english_and_spanish_keep_the_narrow_no_break_space_as_they_read_it() {
        // U+202F is French typography: English and Spanish output must not move (D2).
        let lex = lexicon();
        let text = "Ready\u{202F}? Go";
        for language in [StudiedLanguage::English, StudiedLanguage::Spanish] {
            assert_eq!(
                texts(&tokenize(text, language, &lex)),
                ["Ready\u{202F}", "Go"]
            );
        }
    }

    #[test]
    fn spec_scenario_an_elided_article() {
        let text = "L'homme";
        let tokens = french(text);
        assert_eq!(read(text, &tokens), [("Le", "L'"), ("homme", "homme")]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, 2));
        assert_eq!((tokens[1].start, tokens[1].end), (2, 7));
        // The typographic apostrophe is three bytes, all in the elided piece's span.
        let text = "l’horizon";
        let tokens = french(text);
        assert_eq!(read(text, &tokens), [("le", "l’"), ("horizon", "horizon")]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, 4));
        assert_eq!((tokens[1].start, tokens[1].end), (4, 11));
        assert!(tokens.iter().all(|t| t.parts.is_empty()));
    }

    #[test]
    fn every_elided_form_reads_as_the_word_it_stands_for() {
        // D3: every entry, written in lowercase and with a capital, with either apostrophe.
        for &(written, read_as) in FRENCH_ELISIONS {
            for apostrophe in ["'", "\u{2019}"] {
                for capital in [false, true] {
                    let written = if capital {
                        french_cased(written, "X")
                    } else {
                        written.to_owned()
                    };
                    let text = format!("{written}{apostrophe}avoir");
                    let tokens = french(&text);
                    let elided = format!("{written}{apostrophe}");
                    let expected = if capital {
                        french_cased(read_as, "X")
                    } else {
                        read_as.to_owned()
                    };
                    assert_eq!(
                        read(&text, &tokens),
                        [(expected.as_str(), elided.as_str()), ("avoir", "avoir")],
                        "{text}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_elision_table_is_sorted_for_its_search() {
        assert!(FRENCH_ELISIONS.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(FRENCH_INVERSION_PRONOUNS.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn spec_scenario_elided_words_read_as_the_words_they_stand_for() {
        let pairs =
            |text: &str| -> Vec<String> { french(text).into_iter().map(|t| t.text).collect() };
        assert_eq!(pairs("s'il"), ["si", "il"]);
        assert_eq!(pairs("s'ils"), ["si", "ils"]);
        assert_eq!(pairs("S’il"), ["Si", "il"]);
        assert_eq!(pairs("s'en"), ["se", "en"]);
        // `si` only before the pronoun itself, not a word that starts like it.
        assert_eq!(pairs("s'illumine"), ["se", "illumine"]);
        assert_eq!(pairs("qu'on"), ["que", "on"]);
        assert_eq!(pairs("n'est"), ["ne", "est"]);
        assert_eq!(pairs("lorsqu'elle"), ["lorsque", "elle"]);
        // `m'` and `t'` read `me` and `te` anywhere but right after a hyphen.
        assert_eq!(pairs("m'appelle t'aime"), ["me", "appelle", "te", "aime"]);
    }

    #[test]
    fn the_rule_runs_again_on_what_follows_an_elision() {
        let text = "jusqu'au";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [("jusque", "jusqu'"), ("à", "au"), ("le", "au")]
        );
        let text = "qu'aujourd'hui";
        assert_eq!(
            read(text, &french(text)),
            [("que", "qu'"), ("aujourd'hui", "aujourd'hui")]
        );
    }

    #[test]
    fn spec_scenario_a_word_whose_elision_is_part_of_it() {
        // D3: what precedes the apostrophe is not on the list, so the word stays whole.
        for word in [
            "aujourd'hui",
            "presqu'île",
            "quelqu'un",
            "prud'homme",
            "entr'ouvert",
            "grand'mère",
            "aujourd’hui",
        ] {
            let tokens = french(word);
            assert_eq!(tokens.len(), 1, "{word}");
            assert_eq!((tokens[0].start, tokens[0].end), (0, word.len()), "{word}");
            assert!(tokens[0].parts.is_empty(), "{word}");
        }
        // U+02BC MODIFIER LETTER APOSTROPHE is a letter to UAX #29, and no apostrophe here.
        assert_eq!(texts(&french("lʼhomme")), ["lʼhomme"]);
    }

    #[test]
    fn an_elided_word_written_on_its_own() {
        // D3: what a word card is handed for the elided piece, its apostrophe included.
        let text = "l’";
        let tokens = french(text);
        assert_eq!(read(text, &tokens), [("le", "l’")]);
        let text = "l’ homme";
        assert_eq!(
            read(text, &french(text)),
            [("le", "l’"), ("homme", "homme")]
        );
        let text = "Qu' ";
        assert_eq!(read(text, &french(text)), [("Que", "Qu'")]);
        // A digit is no letter after an elision, and `d` alone falls to the single-letter rule.
        assert!(french("d'1").is_empty());
        // A word that is no elided form keeps the shared rules: its apostrophe is trimmed.
        assert_eq!(texts(&french("homme’ ")), ["homme"]);
    }

    /// Each token's text beside whether it is an elided piece.
    fn elided(text: &str, language: StudiedLanguage) -> Vec<(String, bool)> {
        let lexicon = if language == FR {
            french_lexicon()
        } else {
            lexicon()
        };
        tokenize(text, language, &lexicon)
            .iter()
            .map(|token| (token.text.clone(), is_elided(text, token)))
            .collect()
    }

    fn pieces(read: &[(&str, bool)]) -> Vec<(String, bool)> {
        read.iter()
            .map(|&(text, elided)| (text.to_owned(), elided))
            .collect()
    }

    const IT: StudiedLanguage = StudiedLanguage::Italian;

    /// A stand-in Italian pack: the one-letter words the single-letter rule keeps (`è`, `a`, `e`,
    /// `o`), as the spec's scenarios list them.
    fn italian_lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) =
            build_lexicon_blobs(&[("è", "essere")], &["a", "e", "o", "essere"]).expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    fn italian(text: &str) -> Vec<Token> {
        tokenize(text, IT, &italian_lexicon())
    }

    #[test]
    fn spec_scenario_an_elided_article_in_italian() {
        // add-lingua-italian-tokenisation D3: each piece has its own span, the elided one holding
        // its apostrophe; the read word takes the written capital on its own first letter.
        let text = "L'uomo";
        let tokens = italian(text);
        assert_eq!(read(text, &tokens), [("Lo", "L'"), ("uomo", "uomo")]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, 2));
        assert_eq!((tokens[1].start, tokens[1].end), (2, 6));
        // The typographic apostrophe is three bytes, all in the elided piece's span.
        let text = "l’amica";
        let tokens = italian(text);
        assert_eq!(read(text, &tokens), [("lo", "l’"), ("amica", "amica")]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, 4));
        assert!(tokens.iter().all(|t| t.parts.is_empty()));
    }

    #[test]
    fn every_italian_elided_form_reads_as_the_word_it_stands_for() {
        // D3: every entry, written in lowercase and with a capital, with either apostrophe.
        for &(written, read_as) in ITALIAN_ELISIONS {
            for apostrophe in ["'", "\u{2019}"] {
                for capital in [false, true] {
                    let written = if capital {
                        french_cased(written, "X")
                    } else {
                        written.to_owned()
                    };
                    let text = format!("{written}{apostrophe}uomo");
                    let elided = format!("{written}{apostrophe}");
                    let expected = if capital {
                        french_cased(read_as, "X")
                    } else {
                        read_as.to_owned()
                    };
                    assert_eq!(
                        read(&text, &italian(&text)),
                        [(expected.as_str(), elided.as_str()), ("uomo", "uomo")],
                        "{text}"
                    );
                }
            }
        }
        // Change 6's 38 and the 18 of refine-lingua-italian-reading D2.
        assert_eq!(ITALIAN_ELISIONS.len(), 56);
    }

    #[test]
    fn the_italian_tables_are_sorted_for_their_search() {
        assert!(ITALIAN_ELISIONS.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(ITALIAN_CONTRACTIONS.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(
            ITALIAN_ELIDED_PREPOSITIONS
                .windows(2)
                .all(|w| w[0].0 < w[1].0)
        );
        assert!(ITALIAN_TRUNCATIONS.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(
            ITALIAN_TRUNCATED_PREPOSITIONS
                .windows(2)
                .all(|w| w[0].0 < w[1].0)
        );
        assert!(ITALIAN_DAI_CLITICS.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(ITALIAN_CONTRACTIONS.len(), 26);
        // refine-lingua-italian-reading: `coll'` and `degl'` join the five (D2); `ca'`, `de'` and
        // `que'` join the truncations, `da'` leaves them for the five truncated articulated
        // prepositions (D3).
        assert_eq!(ITALIAN_ELIDED_PREPOSITIONS.len(), 7);
        assert_eq!(ITALIAN_TRUNCATIONS.len(), 10);
        assert_eq!(ITALIAN_TRUNCATED_PREPOSITIONS.len(), 5);
        // No written form is in two tables: each word has one reading.
        for &(form, _, _) in ITALIAN_CONTRACTIONS {
            assert!(italian_elided(form, "").is_none(), "{form}");
        }
        for &(form, _) in ITALIAN_TRUNCATIONS {
            assert!(italian_elided(form, "").is_none(), "{form}");
            assert!(italian_truncated_preposition(form).is_none(), "{form}");
        }
        for &(form, _, _) in ITALIAN_TRUNCATED_PREPOSITIONS {
            assert!(italian_elided(form, "").is_none(), "{form}");
        }
    }

    #[test]
    fn spec_scenario_italian_elided_words_read_as_the_words_they_stand_for() {
        let pairs =
            |text: &str| -> Vec<String> { italian(text).into_iter().map(|t| t.text).collect() };
        for (text, expected) in [
            ("c'è", ["ci", "è"]),
            ("un'amica", ["una", "amica"]),
            ("dov'è", ["dove", "è"]),
            ("quest'anno", ["questo", "anno"]),
            ("anch'io", ["anche", "io"]),
            ("senz'altro", ["senza", "altro"]),
            ("cos'è", ["cosa", "è"]),
            ("quand'ero", ["quando", "ero"]),
            ("dev'essere", ["deve", "essere"]),
            ("cent'anni", ["cento", "anni"]),
            ("nessun'altra", ["nessuna", "altra"]),
            ("diss'egli", ["disse", "egli"]),
            ("diss'io", ["dissi", "io"]),
            ("Diss’io", ["Dissi", "io"]),
        ] {
            assert_eq!(pairs(text), expected, "{text}");
        }
        // `dissi` before the pronoun itself only.
        assert_eq!(pairs("diss'ioni"), ["disse", "ioni"]);
    }

    #[test]
    fn the_italian_rule_runs_again_on_what_follows_an_elision() {
        let text = "dall'altro";
        let tokens = italian(text);
        assert_eq!(
            read(text, &tokens),
            [("da", "dall'"), ("lo", "dall'"), ("altro", "altro")]
        );
        // A form outside the list stays whole: `altr'` is not elided.
        let text = "un'altr'ora";
        assert_eq!(
            read(text, &italian(text)),
            [("una", "un'"), ("altr'ora", "altr'ora")]
        );
        for word in ["altr'ora", "prud'uomo", "po'ero"] {
            assert_eq!(texts(&italian(word)), [word], "{word}");
        }
        // U+02BC MODIFIER LETTER APOSTROPHE is a letter to UAX #29, and no apostrophe here.
        assert_eq!(texts(&italian("lʼuomo")), ["lʼuomo"]);
    }

    #[test]
    fn spec_scenario_the_di_family_stays_whole() {
        let text = "del pane, della casa, dei libri, dell'acqua";
        let tokens = italian(text);
        assert_eq!(
            texts(&tokens),
            [
                "del", "pane", "della", "casa", "dei", "libri", "dello", "acqua"
            ]
        );
        assert_eq!(&text[tokens[6].start..tokens[6].end], "dell'");
        for word in ["dello", "degli", "delle", "Del"] {
            assert_eq!(texts(&italian(word)), [word], "{word}");
        }
    }

    #[test]
    fn spec_scenario_the_other_articulated_prepositions_split() {
        let text = "al mare, nella casa, sul tavolo, dagli amici, col treno";
        let tokens = italian(text);
        assert_eq!(
            read(text, &tokens),
            [
                ("a", "al"),
                ("il", "al"),
                ("mare", "mare"),
                ("in", "nella"),
                ("la", "nella"),
                ("casa", "casa"),
                ("su", "sul"),
                ("il", "sul"),
                ("tavolo", "tavolo"),
                ("da", "dagli"),
                ("gli", "dagli"),
                ("amici", "amici"),
                ("con", "col"),
                ("il", "col"),
                ("treno", "treno"),
            ]
        );
        // The two tokens share the span, as Spanish's `al` and French's `au` do.
        assert_eq!(
            (tokens[0].start, tokens[0].end),
            (tokens[1].start, tokens[1].end)
        );
        // Every written form of the table, before a word: written alone, `dai` ends the text and
        // is the verb (refine-lingua-italian-reading D4).
        for &(written, preposition, article) in ITALIAN_CONTRACTIONS {
            let text = format!("{written} casa");
            assert_eq!(
                texts(&italian(&text)),
                [preposition, article, "casa"],
                "{text}"
            );
        }
        // The archaic forms, and nouns, stay whole.
        for word in ["collo", "colla", "cogli", "colle", "pel", "pei"] {
            assert_eq!(texts(&italian(word)), [word], "{word}");
        }
    }

    #[test]
    fn spec_scenario_an_elided_articulated_preposition() {
        let text = "all'alba, nell'acqua";
        let tokens = italian(text);
        assert_eq!(
            read(text, &tokens),
            [
                ("a", "all'"),
                ("lo", "all'"),
                ("alba", "alba"),
                ("in", "nell'"),
                ("lo", "nell'"),
                ("acqua", "acqua"),
            ]
        );
        let text = "Sull’isola dall’alto";
        assert_eq!(
            read(text, &italian(text)),
            [
                ("Su", "Sull’"),
                ("lo", "Sull’"),
                ("isola", "isola"),
                ("da", "dall’"),
                ("lo", "dall’"),
                ("alto", "alto"),
            ]
        );
    }

    #[test]
    fn spec_scenario_a_capitalised_italian_contraction() {
        let text = "Nel mezzo";
        let tokens = italian(text);
        assert_eq!(
            read(text, &tokens),
            [("In", "Nel"), ("il", "Nel"), ("mezzo", "mezzo")]
        );
        assert_eq!(texts(&italian("Al")), ["A", "il"]);
        assert_eq!(texts(&italian("NEL")), ["In", "il"]);
        assert_eq!(texts(&italian("DELL'ACQUA")), ["Dello", "ACQUA"]);
    }

    #[test]
    fn spec_scenario_italian_truncations() {
        let text = "un po' di pane, va' via";
        let tokens = italian(text);
        assert_eq!(
            read(text, &tokens),
            [
                ("un", "un"),
                ("poco", "po'"),
                ("di", "di"),
                ("pane", "pane"),
                ("vai", "va'"),
                ("via", "via"),
            ]
        );
        for (text, read_as, written) in [
            ("di' ", "dici", "di'"),
            ("Fa'", "Fai", "Fa'"),
            ("po’", "poco", "po’"),
            ("be', ", "bene", "be'"),
            // refine-lingua-italian-reading D4: `mo'` is `modo` before `di` only.
            ("mo'.", "mò", "mo'"),
            ("sta' zitto", "stai", "sta'"),
        ] {
            let tokens = italian(text);
            assert_eq!(read(text, &tokens)[0], (read_as, written), "{text}");
        }
        // A truncation's read word is final. `da'` and `dai` written alone end the text, and are
        // the verb `dai` there; before a word with no sign, the preposition `da` + `i` (I7;
        // refine-lingua-italian-reading D4).
        assert_eq!(read("da'", &italian("da'")), [("dai", "da'")]);
        assert_eq!(texts(&italian("dai")), ["dai"]);
        assert_eq!(texts(&italian("dai nonni")), ["da", "i", "nonni"]);
        // Every entry of the table, alone: `mo'`, with no `di` after it, is `mò`.
        for &(written, read_as) in ITALIAN_TRUNCATIONS {
            let text = format!("{written}'");
            let read_as = if written == "mo" { "mò" } else { read_as };
            assert_eq!(read(&text, &italian(&text)), [(read_as, text.as_str())]);
        }
    }

    // --- refine-lingua-italian-reading: the elided forms, the truncations, the context rules ---

    #[test]
    fn spec_scenario_elided_numbers_and_quantities() {
        // D2: each elided piece spans its letters and its apostrophe.
        for (text, expected) in [
            ("trent'anni", [("trenta", "trent'"), ("anni", "anni")]),
            (
                "cinquant'anni",
                [("cinquanta", "cinquant'"), ("anni", "anni")],
            ),
            (
                "ventiquattr'ore",
                [("ventiquattro", "ventiquattr'"), ("ore", "ore")],
            ),
            ("tant'è", [("tanto", "tant'"), ("è", "è")]),
            ("null'altro", [("nulla", "null'"), ("altro", "altro")]),
            ("ogn'uno", [("ogni", "ogn'"), ("uno", "uno")]),
        ] {
            assert_eq!(read(text, &italian(text)), expected, "{text}");
        }
    }

    #[test]
    fn spec_scenario_more_elided_words() {
        for (text, expected) in [
            ("Qual'è", vec!["Quale", "è"]),
            ("qual’è", vec!["quale", "è"]),
            ("gliel'ho", vec!["glielo", "ho"]),
            ("neanch'io", vec!["neanche", "io"]),
            ("fin'ora", vec!["fino", "ora"]),
            ("prim'ordine", vec!["primo", "ordine"]),
            ("quarant'anni", vec!["quaranta", "anni"]),
            ("sessant'anni", vec!["sessanta", "anni"]),
            ("settant'anni", vec!["settanta", "anni"]),
            ("ottant'anni", vec!["ottanta", "anni"]),
            ("novant'anni", vec!["novanta", "anni"]),
            ("diciott'anni", vec!["diciotto", "anni"]),
        ] {
            assert_eq!(texts(&italian(text)), expected, "{text}");
        }
        // The rule runs again on what follows: « l'ultim'ora » is three words, each its own span.
        let text = "l'ultim'ora";
        assert_eq!(
            read(text, &italian(text)),
            [("lo", "l'"), ("ultima", "ultim'"), ("ora", "ora")]
        );
        // `oltr'` is left out: Italian writes « oltralpe » whole.
        assert_eq!(texts(&italian("oltr'alpe")), ["oltr'alpe"]);
        // The plain `fin` and `qual` are apocopes, words of their own, read as written.
        assert_eq!(
            texts(&italian("fin qui, qual è")),
            ["fin", "qui", "qual", "è"]
        );
    }

    #[test]
    fn spec_scenario_two_more_elided_prepositions() {
        // D2: `coll'` is `con` + `lo` sharing its span, as `all'` is; `degl'` the `di` family's,
        // whole, as `dell'` is.
        let text = "coll'aiuto e degl'Italiani";
        let tokens = italian(text);
        assert_eq!(
            read(text, &tokens),
            [
                ("con", "coll'"),
                ("lo", "coll'"),
                ("aiuto", "aiuto"),
                ("e", "e"),
                ("degli", "degl'"),
                ("Italiani", "Italiani"),
            ]
        );
        assert_eq!(texts(&italian("Coll’ago")), ["Con", "lo", "ago"]);
        assert_eq!(texts(&italian("DEGL'INNI")), ["Degli", "INNI"]);
        // Written alone, as a word card hands them.
        assert_eq!(read("degl'", &italian("degl'")), [("degli", "degl'")]);
        assert_eq!(texts(&italian("coll'")), ["con", "lo"]);
    }

    #[test]
    fn spec_scenario_truncations_a_book_writes() {
        // D3: `ca'`, `de'` and `que'` are one word each, spanning the apostrophe, wherever they
        // stand.
        for (text, expected) in [
            ("Ca' Foscari", vec![("Casa", "Ca'"), ("Foscari", "Foscari")]),
            (
                "Lorenzo de' Medici",
                vec![("Lorenzo", "Lorenzo"), ("dei", "de'"), ("Medici", "Medici")],
            ),
            ("que' due", vec![("quei", "que'"), ("due", "due")]),
            ("de’ suoi", vec![("dei", "de’"), ("suoi", "suoi")]),
            ("Ca'.", vec![("Casa", "Ca'")]),
        ] {
            assert_eq!(read(text, &italian(text)), expected, "{text}");
        }
    }

    #[test]
    fn spec_scenario_a_truncated_articulated_preposition() {
        // D3: before a word, the preposition and `i`, sharing the written word's span, apostrophe
        // included; the first keeps the written capital.
        for (text, expected) in [
            ("a' piedi", [("a", "a'"), ("i", "a'"), ("piedi", "piedi")]),
            ("co' suoi", [("con", "co'"), ("i", "co'"), ("suoi", "suoi")]),
            (
                "ne' campi",
                [("in", "ne'"), ("i", "ne'"), ("campi", "campi")],
            ),
            (
                "su' colli",
                [("su", "su'"), ("i", "su'"), ("colli", "colli")],
            ),
            ("Co' suoi", [("Con", "Co'"), ("i", "Co'"), ("suoi", "suoi")]),
            ("A’ tuoi", [("A", "A’"), ("i", "A’"), ("tuoi", "tuoi")]),
        ] {
            let tokens = italian(text);
            assert_eq!(read(text, &tokens), expected, "{text}");
            assert_eq!(
                (tokens[0].start, tokens[0].end),
                (tokens[1].start, tokens[1].end)
            );
        }
        // Every entry of the table, before a word: `da'` before one with no sign is `da` + `i` too.
        for &(written, preposition, article) in ITALIAN_TRUNCATED_PREPOSITIONS {
            let text = format!("{written}' monti");
            assert_eq!(
                texts(&italian(&text)),
                [preposition, article, "monti"],
                "{text}"
            );
        }
    }

    #[test]
    fn spec_scenario_a_truncated_preposition_with_no_word_after_it() {
        // D3: `a'`, `co'`, `ne'` and `su'` with no word after them are apocopes, their apostrophe
        // outside the word.
        assert_eq!(read("ne'", &italian("ne'")), [("ne", "ne")]);
        let text = "Vieni su'. ";
        assert_eq!(
            read(text, &italian(text)),
            [("Vieni", "Vieni"), ("su", "su")]
        );
        for written in ["a", "co", "ne", "su"] {
            for after in ["", ".", ", e", "!", " 3"] {
                let text = format!("{written}'{after}");
                assert_eq!(
                    read(&text, &italian(&text))[0],
                    (written, written),
                    "{text}"
                );
            }
        }
        // The quote opened before `Vieni` closes at the apostrophe after `su`: `po'` after it is
        // read as `poco`.
        assert_eq!(
            texts(&italian("'Vieni su', disse, e un po'")),
            ["Vieni", "su", "disse", "e", "un", "poco"]
        );
        // A quoted `a` keeps its quotes outside.
        let text = "si dice 'a', non 'e'";
        assert_eq!(
            read(text, &italian(text)),
            [
                ("si", "si"),
                ("dice", "dice"),
                ("a", "a"),
                ("non", "non"),
                ("e", "e")
            ]
        );
        // Inside a quote, before a word, they are read as written, as change 6's truncations are.
        assert_eq!(texts(&italian("'ne' campi'")), ["ne", "campi"]);
    }

    #[test]
    fn a_truncated_preposition_s_apostrophe_and_the_quotes() {
        // D3, at review: inside an open quote the pre-pass reads `su'` as written, and its
        // apostrophe closes the quote, as the apocope's did: `po'` after it is read as `poco`.
        assert_eq!(
            texts(&italian("'Vieni su' gli disse, un po'")),
            ["Vieni", "su", "gli", "disse", "un", "poco"]
        );
        assert_eq!(
            texts(&italian("'Mi siedo su' gradini, un po'")),
            ["Mi", "siedo", "su", "gradini", "un", "poco"]
        );
        // With no quote open, the four are read before a word, and nothing is left open.
        assert_eq!(
            texts(&italian("Andò ne' campi, un po' stanco")),
            ["Andò", "in", "i", "campi", "un", "poco", "stanco"]
        );
        // `da'`, read wherever it stands, closes none, as a truncation's apostrophe.
        assert_eq!(
            texts(&italian("'Vengo da' monti, un po'")),
            ["Vengo", "da", "monti", "un", "po"]
        );
    }

    #[test]
    fn spec_scenario_a_quoted_short_word() {
        // At review: a word a quote mark opens right before is the whole quoted span, and the
        // apostrophe after it closes that quote — a truncated preposition's, a truncation's or an
        // accent spelling's alike —, so the words after it read their own apostrophes.
        for (text, last) in [
            (
                "la particella 'ne' indica il partitivo, un po' di pane",
                "poco",
            ),
            ("la preposizione 'a' regge, un po'", "poco"),
            ("la forma 'de' è antica, un po'", "poco"),
            ("si scriveva 'que' allora, un po'", "poco"),
            ("si dice ‘po’ e basta, un po'", "poco"),
            ("la congiunzione 'e' unisce, un po'", "poco"),
        ] {
            let tokens = italian(text);
            let po = tokens
                .iter()
                .rfind(|t| text[t.start..].starts_with("po"))
                .unwrap_or_else(|| panic!("{text}: {tokens:?}"));
            assert_eq!(
                (po.text.as_str(), &text[po.start..po.end]),
                (last, "po'"),
                "{text}"
            );
        }
        // The quoted word itself is read as written, its quotes outside it.
        assert_eq!(
            read(
                "la particella 'ne' indica",
                &italian("la particella 'ne' indica")
            )[2],
            ("ne", "ne")
        );
        // An apostrophe after an elided piece opens nothing: « 'c'e' lavoro' » keeps its quote.
        assert_eq!(
            texts(&accented("Disse: 'c'e' lavoro'", &[])),
            ["Disse", "ci", "e", "lavoro"]
        );
    }

    #[test]
    fn spec_scenario_mo_by_the_word_after_it() {
        // D4: `modo` when the next word, as read, is `di` — the elided `d'` too —, `mò` otherwise;
        // each read word spans « mo' ».
        assert_eq!(
            read("a mo' di esempio", &italian("a mo' di esempio")),
            [
                ("a", "a"),
                ("modo", "mo'"),
                ("di", "di"),
                ("esempio", "esempio")
            ]
        );
        assert_eq!(
            read("a mo' d'esempio", &italian("a mo' d'esempio")),
            [
                ("a", "a"),
                ("modo", "mo'"),
                ("di", "d'"),
                ("esempio", "esempio")
            ]
        );
        assert_eq!(
            read("mo' arrivo", &italian("mo' arrivo")),
            [("mò", "mo'"), ("arrivo", "arrivo")]
        );
        assert_eq!(
            texts(&italian("Mo’ basta, e mo'?")),
            ["Mò", "basta", "e", "mò"]
        );
        assert_eq!(
            texts(&italian("a mo' DI esempio")),
            ["a", "modo", "DI", "esempio"]
        );
        // Only a space between: `di` after a comma, or joined to another word, is no next word.
        assert_eq!(texts(&italian("a mo', di")), ["a", "mò", "di"]);
        assert_eq!(texts(&italian("mo' dire")), ["mò", "dire"]);
        // `d'` is the elided `di` only before a letter.
        assert_eq!(texts(&italian("mo' d' ")), ["mò", "di"]);
    }

    #[test]
    fn spec_scenario_dai_the_verb() {
        // D4, as D5 measured it: after a clitic pronoun, or before punctuation, a closing quote or
        // the end of the text, `dai` is the verb, one token spanning the written word.
        for text in [
            "Dai, andiamo!",
            "mi dai una mano?",
            "Me lo dai?",
            "Ma dai!",
            "\u{201C}Ma dai\u{201D} disse",
            "«Ma dai» disse",
            "'Ma dai' disse",
            "Ma dai.",
            "Ma dai…",
            "Proviamoci, dai",
            "Ma dai  ",
        ] {
            let tokens = italian(text);
            let written: Vec<&str> = tokens
                .iter()
                .filter(|t| t.text.eq_ignore_ascii_case("dai"))
                .map(|t| &text[t.start..t.end])
                .collect();
            assert_eq!(written.len(), 1, "{text}: {tokens:?}");
            assert!(written[0].eq_ignore_ascii_case("dai"), "{text}");
            assert!(
                !tokens
                    .iter()
                    .any(|t| t.text.eq_ignore_ascii_case("da") || t.text == "i"),
                "{text}: {tokens:?}"
            );
        }
        assert_eq!(texts(&italian("Dai, andiamo!")), ["Dai", "andiamo"]);
        // One clitic at a time.
        for clitic in ITALIAN_DAI_CLITICS {
            let text = format!("{clitic} dai una mano");
            assert_eq!(texts(&italian(&text))[1], "dai", "{text}");
            let text = format!("Tu {clitic} dai il libro");
            assert_eq!(texts(&italian(&text))[2], "dai", "{text}");
        }
    }

    #[test]
    fn spec_scenario_dai_the_preposition() {
        // D4: anywhere else, the preposition `da` + `i` sharing the span, as I7 reads it.
        for text in [
            "Torno dai nonni",
            "dai molti suoi cocuzzoli",
            "Dai nonni si mangia bene",
            // D5 measured `non`, `me` and `te` out: the verb with no sign reads as before.
            "non dai retta",
            "vieni con me dai nonni?",
            "e te dai cugini",
            // A comma between a clitic and `dai` is no clitic right before it.
            "gli, dai nonni",
            // `!` after a space is not right after the word.
            "dai !",
        ] {
            let tokens = italian(text);
            let at = tokens
                .iter()
                .position(|t| t.text.eq_ignore_ascii_case("da"))
                .unwrap_or_else(|| panic!("{text}: {tokens:?}"));
            assert_eq!(tokens[at + 1].text, "i", "{text}");
            assert_eq!(
                (tokens[at].start, tokens[at].end),
                (tokens[at + 1].start, tokens[at + 1].end),
                "{text}"
            );
            assert!(
                !tokens.iter().any(|t| t.text.eq_ignore_ascii_case("dai")),
                "{text}"
            );
        }
        // « dai e dai, … »: the preposition, then the verb before the comma.
        assert_eq!(
            texts(&italian("dai e dai, ce l'ha fatta")),
            ["da", "i", "e", "dai", "ce", "lo", "ha", "fatta"]
        );
    }

    #[test]
    fn spec_scenario_da_by_its_context() {
        // D4: « da' primi suoi anni » is `da` + `i`, apostrophe included; « mi da' una mano? » the
        // verb `dai`, one token spanning « da' ».
        let text = "da' primi suoi anni";
        let tokens = italian(text);
        assert_eq!(read(text, &tokens)[..2], [("da", "da'"), ("i", "da'")]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, 3));
        let text = "mi da' una mano?";
        assert_eq!(read(text, &italian(text))[1], ("dai", "da'"));
        assert_eq!(read("Da' ", &italian("Da' ")), [("Dai", "Da'")]);
        assert_eq!(
            texts(&italian("Da’ suoi monti")),
            ["Da", "i", "suoi", "monti"]
        );
    }

    #[test]
    fn each_trigger_is_its_own_check() {
        // D5 measures each trigger on its own: the word before, with nothing but a space between.
        assert_eq!(italian_word_before("mi dai", 3).as_deref(), Some("mi"));
        assert_eq!(italian_word_before("Mi  dai", 4).as_deref(), Some("mi"));
        assert_eq!(italian_word_before("mi, dai", 4), None);
        assert_eq!(italian_word_before("dai", 0), None);
        assert_eq!(italian_word_before("po' dai", 4), None);
        assert!(italian_dai_after_a_clitic(Some("gliene")));
        assert!(!italian_dai_after_a_clitic(Some("me")));
        assert!(!italian_dai_after_a_clitic(Some("non")));
        assert!(!italian_dai_after_a_clitic(None));
        // What follows: punctuation, a closing quote, the end of the text, or only spaces to it.
        for (text, verb) in [
            ("dai", true),
            ("dai ", true),
            ("dai,", true),
            ("dai!", true),
            ("dai?", true),
            ("dai.", true),
            ("dai…", true),
            ("dai\"", true),
            ("dai\u{201D}", true),
            ("dai»", true),
            ("dai'", true),
            ("dai\u{2019}", true),
            ("dai›", true),
            ("dai x", false),
            ("dai ,", false),
            ("dai;", false),
            ("dai:", false),
            ("dai)", false),
        ] {
            assert_eq!(italian_dai_before_punctuation(text, 3), verb, "{text:?}");
        }
    }

    #[test]
    fn the_readings_of_a_word_read_by_its_context() {
        // D4: a word card asks for both readings of a word the pre-pass reads by its context, the
        // one as it reads alone first.
        let readings = |text: &str| -> Vec<Vec<String>> {
            tokenize_readings(text, IT, &italian_lexicon())
                .into_iter()
                .map(|tokens| tokens.into_iter().map(|t| t.text).collect())
                .collect()
        };
        assert_eq!(readings("dai"), [vec!["dai"], vec!["da", "i"]]);
        assert_eq!(readings("Dai"), [vec!["Dai"], vec!["Da", "i"]]);
        assert_eq!(readings("da’"), [vec!["dai"], vec!["da", "i"]]);
        assert_eq!(readings("mo'"), [vec!["mò"], vec!["modo"]]);
        assert_eq!(
            readings("a mo' di"),
            [vec!["a", "modo", "di"], vec!["a", "mò", "di"]]
        );
        assert_eq!(
            readings("dai nonni"),
            [vec!["da", "i", "nonni"], vec!["dai", "nonni"]]
        );
        // The other reading keeps the written word's span.
        let tokens = &tokenize_readings("Torno dai", IT, &italian_lexicon())[1];
        assert_eq!(
            tokens.iter().map(|t| (t.start, t.end)).collect::<Vec<_>>(),
            [(0, 5), (6, 9), (6, 9)]
        );
        // At review: `a'`, `co'`, `ne'` and `su'` written with their apostrophe read first as the
        // preposition and `i`, spanning the apostrophe, then as the apocope.
        assert_eq!(readings("ne'"), [vec!["in", "i"], vec!["ne"]]);
        assert_eq!(readings("Su’"), [vec!["Su", "i"], vec!["Su"]]);
        assert_eq!(readings("a'"), [vec!["a", "i"], vec!["a"]]);
        assert_eq!(readings("co'"), [vec!["con", "i"], vec!["co"]]);
        let tokens = &tokenize_readings("ne'", IT, &italian_lexicon())[0];
        assert_eq!(
            tokens.iter().map(|t| (t.start, t.end)).collect::<Vec<_>>(),
            [(0, 3), (0, 3)]
        );
        // Without an apostrophe, or already split before a word, they have one reading.
        assert_eq!(readings("ne"), [vec!["ne"]]);
        assert_eq!(readings("ne' campi"), [vec!["in", "i", "campi"]]);
        // Every other word has one reading, and so has every other language.
        assert_eq!(readings("al mare"), [vec!["a", "il", "mare"]]);
        assert_eq!(readings("'da' "), [vec!["da"]]);
        for language in [
            StudiedLanguage::English,
            StudiedLanguage::Spanish,
            StudiedLanguage::French,
        ] {
            assert_eq!(
                tokenize_readings("dai mo'", language, &italian_lexicon()).len(),
                1
            );
        }
    }

    #[test]
    fn spec_scenario_an_apostrophe_for_an_accent_and_a_closing_quote() {
        // D5: any other trailing apostrophe stays outside the word, as the rule of no language
        // leaves it; a quoted truncation's base is read as written (the opening-quote scan,
        // recorded in the design's implementation notes). Change 10 reads an apostrophe for an
        // accent where the pack lists the accented word (`e'` → `è` below; its own tests,
        // `spec_scenario_an_apostrophe_for_an_accent`): this stand-in lists no `perché`.
        for (text, expected) in [
            ("perche'", vec!["perche"]),
            ("‘ciao’", vec!["ciao"]),
            ("‘due anni fa’", vec!["due", "anni", "fa"]),
            ("'due anni fa'", vec!["due", "anni", "fa"]),
            ("‘fa’", vec!["fa"]),
            ("'po' ", vec!["po"]),
        ] {
            let tokens = italian(text);
            assert_eq!(texts(&tokens), expected, "{text}");
            assert!(
                tokens
                    .iter()
                    .all(|t| !text[t.start..t.end].ends_with(is_apostrophe)),
                "{text}"
            );
        }
        // A closed quote before the word leaves it outside any: the truncation is read.
        assert_eq!(texts(&italian("‘ciao’ un po'")), ["ciao", "un", "poco"]);
        assert_eq!(
            texts(&italian("un po' e va' via")),
            ["un", "poco", "e", "vai", "via"]
        );
        // An elision before the word is inside a word, not a quote.
        assert_eq!(texts(&italian("c'è un po'")), ["ci", "è", "un", "poco"]);
        // The cost: a truncation inside a straight-quoted phrase reads as at 0.1.0, and so does
        // one after an aphaeresis (`'na`), read as an opening quote.
        assert_eq!(
            texts(&italian("'un po' e basta'")),
            ["un", "po", "e", "basta"]
        );
        assert_eq!(
            texts(&italian("'na cosa, un po'")),
            ["na", "cosa", "un", "po"]
        );
        // A quote closed after punctuation is closed: what follows reads its truncations.
        for text in [
            "‘Vieni qui!’ disse. Un po' di pane",
            "'Vieni qui!' disse. Un po' di pane",
            "'Dov'è?' chiese, un po' stanco",
            "'Sì.' Un po'",
        ] {
            assert!(texts(&italian(text)).contains(&"poco"), "{text}");
        }
        // `‘` opens a quote only as `'` does, before a letter and after no letter or digit: not
        // the elided `‘` of « anni ‘80 », nor a mistyped « l‘uomo ».
        assert_eq!(texts(&italian("anni ‘80, un po'")), ["anni", "un", "poco"]);
        assert_eq!(texts(&italian("l‘uomo, un po'")), ["l‘uomo", "un", "poco"]);
    }

    /// A stand-in Italian pack for the apostrophe that stands for an accent
    /// (read-lingua-italian-unmarked-spellings D1): the accented words the spec's scenarios
    /// list, each filed under itself or its verb, the truncations' accented look-alikes (`pò`,
    /// `dà`) and the apocopes' (`né`, `sù`), and `forms` beside them.
    fn accent_lexicon(forms: &[(&str, &str)]) -> FstLexicon<Vec<u8>> {
        let mut pairs = vec![
            ("è", "essere"),
            ("più", "più"),
            ("città", "città"),
            ("può", "potere"),
            ("così", "così"),
            ("perché", "perché"),
            ("caffè", "caffè"),
            ("pò", "pò"),
            ("dà", "dare"),
            ("né", "né"),
            ("sù", "su"),
            ("problema", "problema"),
            // The accented look-alikes of the other apocopes the tables list (task 2.2).
            ("qué", "qué"),
            ("cà", "cà"),
            ("vè", "vè"),
            ("mì", "mì"),
            ("frà", "fra"),
            ("tò", "tò"),
            ("ì", "ì"),
            ("mè", "mè"),
            ("vò", "vò"),
            ("fè", "fè"),
            ("à", "à"),
            ("cò", "cò"),
            ("dè", "dè"),
            ("pè", "pè"),
        ];
        pairs.extend_from_slice(forms);
        let (bytes, pool) =
            build_lexicon_blobs(&pairs, &["a", "e", "o", "essere", "su", "dare", "potere"])
                .expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    fn accented(text: &str, forms: &[(&str, &str)]) -> Vec<Token> {
        tokenize(text, IT, &accent_lexicon(forms))
    }

    #[test]
    fn spec_scenario_an_apostrophe_for_an_accent() {
        // D1: one token spanning the word and its apostrophe, its text the accented form the pack
        // lists, straight or typographic.
        for (text, read_as) in [
            ("perche'", "perché"),
            ("piu'", "più"),
            ("citta'", "città"),
            ("e'", "è"),
            ("puo'", "può"),
            ("cosi'", "così"),
            ("caffe'", "caffè"),
            ("perche’", "perché"),
        ] {
            let tokens = accented(text, &[]);
            assert_eq!(read(text, &tokens), [(read_as, text)], "{text}");
            assert!(is_elided(text, &tokens[0]), "{text}");
        }
        // In running text, before punctuation, a space or the end.
        let text = "E' troppo tardi perche' la citta' e' piu' lontana, cosi'.";
        assert_eq!(
            read(text, &accented(text, &[])),
            [
                ("È", "E'"),
                ("troppo", "troppo"),
                ("tardi", "tardi"),
                ("perché", "perche'"),
                ("la", "la"),
                ("città", "citta'"),
                ("è", "e'"),
                ("più", "piu'"),
                ("lontana", "lontana"),
                ("così", "cosi'"),
            ]
        );
    }

    #[test]
    fn spec_scenario_an_accent_after_an_elided_piece() {
        // D1: the rule reads what remains after change 6's elided pieces are split off.
        for (text, expected) in [
            (
                "non c'e' problema",
                vec![
                    ("non", "non"),
                    ("ci", "c'"),
                    ("è", "e'"),
                    ("problema", "problema"),
                ],
            ),
            ("dov'e'", vec![("dove", "dov'"), ("è", "e'")]),
            ("com'e'", vec![("come", "com'"), ("è", "e'")]),
            ("l'e'", vec![("lo", "l'"), ("è", "e'")]),
            ("C’e’", vec![("Ci", "C’"), ("è", "e’")]),
        ] {
            assert_eq!(read(text, &accented(text, &[])), expected, "{text}");
        }
    }

    #[test]
    fn spec_scenario_an_accent_on_a_capital() {
        // D1: the written capital is kept on the accented letter, as `french_cased` puts it.
        for (text, expected) in [
            ("E' vero", vec![("È", "E'"), ("vero", "vero")]),
            ("PERCHE'", vec![("Perché", "PERCHE'")]),
            ("Citta'", vec![("Città", "Citta'")]),
        ] {
            assert_eq!(read(text, &accented(text, &[])), expected, "{text}");
        }
    }

    #[test]
    fn spec_scenario_two_accents_listed() {
        // D1: a form the pack files under the other is the other's spelling, and the other wins.
        let tables = [("perchè", "perché"), ("caffé", "caffè")];
        assert_eq!(
            texts(&accented("perche' caffe'", &tables)),
            ["perché", "caffè"]
        );
        // Neither filed under the other: `è` before `é`.
        let neither = [("benché", "sebbene"), ("benchè", "sebbene")];
        assert_eq!(texts(&accented("benche'", &neither)), ["benchè"]);
        let both = [("tè", "tè"), ("té", "té")];
        assert_eq!(texts(&accented("te'", &both)), ["tè"]);
        // `è` filed under `é`: `é`.
        let own = [("benchè", "benché"), ("benché", "benché")];
        assert_eq!(texts(&accented("benche'", &own)), ["benché"]);
        // The other's spelling loses whichever accent it carries.
        assert_eq!(
            texts(&accented(
                "ventitre'",
                &[("ventitré", "ventitré"), ("ventitrè", "ventitré")]
            )),
            ["ventitré"]
        );
        // Only `é` listed: `é`.
        assert_eq!(
            texts(&accented("poiche'", &[("poiché", "poiché")])),
            ["poiché"]
        );
        // The tables' own case: `è` a form of *essere*, the misspelling `é` ranked as a word of its
        // own. Neither is filed under the other: `è` before `é`, « E' » is `È`.
        let misspelt = [("é", "é")];
        assert_eq!(
            texts(&accented("E' vero, e'", &misspelt)),
            ["È", "vero", "è"]
        );
    }

    #[test]
    fn spec_scenario_a_listed_word_with_its_apostrophe() {
        // D1: change 6's truncations keep their reading, though the pack lists `pò` and `dà`; since
        // refine-lingua-italian-reading D3–D4 `da'` is a truncated articulated preposition, `da` + `i`
        // before « qua », and the verb `dai` after a clitic, never `dà`.
        let text = "un po' e da' qua";
        assert_eq!(
            read(text, &accented(text, &[])),
            [
                ("un", "un"),
                ("poco", "po'"),
                ("e", "e"),
                ("da", "da'"),
                ("i", "da'"),
                ("qua", "qua")
            ]
        );
        // The spec's scenario: « un po' », « da' primi anni » and « mi da' una mano ».
        assert_eq!(read("un po'", &accented("un po'", &[]))[1], ("poco", "po'"));
        let text = "da' primi anni";
        assert_eq!(
            read(text, &accented(text, &[]))[..2],
            [("da", "da'"), ("i", "da'")]
        );
        let text = "mi da' una mano";
        assert_eq!(read(text, &accented(text, &[]))[1], ("dai", "da'"));
        for &(written, read_as) in ITALIAN_TRUNCATIONS {
            let text = format!("{written}'");
            let read_as = if written == "mo" { "mò" } else { read_as };
            assert_eq!(texts(&accented(&text, &[])), [read_as], "{text}");
        }
        // Quoted, a truncation is read as written, never for an accent.
        assert_eq!(
            texts(&accented("'un po' e basta'", &[])),
            ["un", "po", "e", "basta"]
        );
        // An elided form written alone keeps its reading.
        assert_eq!(read("l' ", &accented("l' ", &[])), [("lo", "l'")]);
    }

    #[test]
    fn spec_scenario_an_apocope() {
        // D1: never read for an accent, though the pack lists `né` and `sù`: the apostrophe stays
        // outside the word.
        for apocope in ITALIAN_APOCOPES {
            let text = format!("{apocope}' ");
            let tokens = accented(&text, &[]);
            assert!(
                tokens.iter().all(|t| t.end <= apocope.len()),
                "{text}: {tokens:?}"
            );
        }
        // refine-lingua-italian-reading D3: before a word, `ne'` and `su'` are two words each, and
        // `que'` and `ca'` read wherever they stand; `ve'` is still an apocope.
        assert_eq!(
            texts(&accented("ne' campi, su' monti", &[])),
            ["in", "i", "campi", "su", "i", "monti"]
        );
        assert_eq!(texts(&accented("Ne'", &[])), ["Ne"]);
        assert_eq!(
            texts(&accented("que' due, Ca' Foscari, ve' ve'", &[])),
            ["quei", "due", "Casa", "Foscari", "ve", "ve"]
        );
        // The spec's scenario: « to' », « ve' », « mi' » and « ne' », with `tò`, `vè`, `mì` and `né`
        // listed.
        for (text, word) in [("to'", "to"), ("ve'", "ve"), ("mi'", "mi"), ("ne'", "ne")] {
            assert_eq!(read(text, &accented(text, &[])), [(word, word)], "{text}");
        }
        assert_eq!(
            ITALIAN_APOCOPES.windows(2).find(|pair| pair[0] >= pair[1]),
            None,
            "sorted for the search"
        );
    }

    #[test]
    fn spec_scenario_a_book_s_truncations_are_not_accents() {
        // refine-lingua-italian-reading D3: before a word, `ne'` and `su'` are two words and `que'`
        // is `quei`, each spanning its apostrophe, though the pack lists `né`, `sù` and `qué`.
        for (text, expected) in [
            (
                "ne' campi",
                vec![("in", "ne'"), ("i", "ne'"), ("campi", "campi")],
            ),
            (
                "su' colli",
                vec![("su", "su'"), ("i", "su'"), ("colli", "colli")],
            ),
            ("que' due", vec![("quei", "que'"), ("due", "due")]),
        ] {
            assert_eq!(read(text, &accented(text, &[])), expected, "{text}");
        }
    }

    #[test]
    fn spec_scenario_a_word_the_tables_do_not_accent() {
        // D1: no `ciaò` listed — the apostrophe outside the word, as before.
        for (text, expected) in [
            ("ciao'", vec![("ciao", "ciao")]),
            ("Ciao' ", vec![("Ciao", "Ciao")]),
            // A consonant before the apostrophe is no vowel to accent.
            ("sailors'", vec![("sailors", "sailors")]),
            // Neither a letter nor a digit may follow the apostrophe.
            ("perche'2", vec![("perche", "perche")]),
            // A word already accented keeps its accent and drops the apostrophe.
            ("perché'", vec![("perché", "perché")]),
        ] {
            assert_eq!(read(text, &accented(text, &[])), expected, "{text}");
        }
        // A word the tokeniser joins at an inner apostrophe is read as before.
        assert_eq!(texts(&accented("perche'x", &[])), ["perche'x"]);
    }

    #[test]
    fn spec_scenario_an_accent_and_a_closing_quote() {
        // D1: an apostrophe after a vowel is read for an accent only when no quote is open before
        // the written word (change 6's scan, `inside_a_quote`).
        for (text, expected) in [
            ("‘perche’", vec!["perche"]),
            ("'perche'", vec!["perche"]),
            ("‘e’", vec!["e"]),
            ("‘Ma e' vero’", vec!["Ma", "e", "vero"]),
        ] {
            let tokens = accented(text, &[]);
            assert_eq!(texts(&tokens), expected, "{text}");
            assert!(
                tokens
                    .iter()
                    .all(|t| !text[t.start..t.end].ends_with(is_apostrophe)),
                "{text}"
            );
        }
        // A quote closed before the word leaves it outside any.
        assert_eq!(
            texts(&accented("‘ciao’ perche' no", &[])),
            ["ciao", "perché", "no"]
        );
        // At review: so does a quoted short word, the whole quoted span, before an accent spelling.
        assert_eq!(
            texts(&accented("la preposizione 'a' regge, perche' no", &[])),
            ["la", "preposizione", "a", "regge", "perché", "no"]
        );
        // The spec's scenario: « perche' », « c'e' », « ‘ciao’ » and « ‘due anni fa’ ».
        assert_eq!(
            texts(&accented("perche' c'e' ‘ciao’ ‘due anni fa’", &[])),
            ["perché", "ci", "è", "ciao", "due", "anni", "fa"]
        );
    }

    /// The accent stand-in with the pack's ranks, `ranks` naming lemmas and their rank.
    fn ranked_accent_lexicon(forms: &[(&str, &str)], ranks: &[(&str, u32)]) -> FstLexicon<Vec<u8>> {
        let lexicon = accent_lexicon(forms);
        let mut table = vec![0u32; lexicon.lemma_count()];
        for &(lemma, rank) in ranks {
            table[lexicon.lemma_id(lemma).expect("a lemma") as usize] = rank;
        }
        lexicon.with_ranks(table)
    }

    #[test]
    fn a_stray_apostrophe_reads_no_rare_token() {
        // At review: the tables rank rare tokens a stray apostrophe would read (`vità` 33,926,
        // `comè` 55,162 …); a form whose word the pack ranks beyond ITALIAN_ACCENT_RANK is not
        // read, and the apostrophe stays outside the word.
        let forms = [
            ("vità", "vità"),
            ("comè", "comè"),
            ("dovè", "dovè"),
            ("chì", "chì"),
            ("nò", "nò"),
            ("luì", "luì"),
            ("governò", "governare"),
            ("terrà", "tenere"),
            ("piè", "piè"),
        ];
        let ranks = [
            ("vità", 33_926),
            ("comè", 55_162),
            ("dovè", 44_355),
            ("chì", 50_303),
            ("nò", 49_990),
            ("luì", 42_486),
            ("governare", 3_722),
            ("tenere", 684),
            ("piè", 15_547),
            ("essere", 29),
            ("perché", 43),
            ("più", 16),
        ];
        let lexicon = ranked_accent_lexicon(&forms, &ranks);
        let read = |text: &str| -> Vec<String> {
            tokenize(text, IT, &lexicon)
                .into_iter()
                .map(|t| t.text)
                .collect()
        };
        for (text, word) in [
            ("vita'", "vita"),
            ("come'", "come"),
            ("dove'", "dove"),
            ("chi'", "chi"),
            ("no'", "no"),
            ("lui'", "lui"),
        ] {
            assert_eq!(read(text), [word], "{text}");
        }
        for (text, word) in [
            ("governo'", "governò"),
            ("terra'", "terrà"),
            ("pie'", "piè"),
            ("perche'", "perché"),
            ("piu'", "più"),
            ("E'", "È"),
        ] {
            assert_eq!(read(text), [word], "{text}");
        }
        // The bound itself is read; one beyond it is not. A form the pack ranks not at all is read
        // (the stand-ins rank nothing).
        let edge = ranked_accent_lexicon(
            &[("bontà", "bontà"), ("onestà", "onestà")],
            &[
                ("bontà", ITALIAN_ACCENT_RANK),
                ("onestà", ITALIAN_ACCENT_RANK + 1),
            ],
        );
        assert_eq!(
            texts(&tokenize("bonta' onesta'", IT, &edge)),
            ["bontà", "onesta"]
        );
        assert_eq!(
            texts(&accented("bonta' onesta'", &[("bontà", "bontà")])),
            ["bontà", "onesta"]
        );
    }

    #[test]
    fn the_pack_s_ranks_choose_between_two_accents() {
        // At review: neither filed under the other, the one whose word the pack ranks higher —
        // `sé` 655 before `sè` 2,911, `è` (*essere*) before the misspelling `é`.
        let lexicon = ranked_accent_lexicon(
            &[("sè", "sè"), ("sé", "sé"), ("é", "é")],
            &[("sè", 2_911), ("sé", 655), ("é", 1_500), ("essere", 29)],
        );
        assert_eq!(
            texts(&tokenize("da se' E' vero e'", IT, &lexicon)),
            ["da", "sé", "È", "vero", "è"]
        );
        // Without ranks, `è` before `é`.
        assert_eq!(
            texts(&accented("se'", &[("sè", "sè"), ("sé", "sé")])),
            ["sè"]
        );
    }

    #[test]
    fn a_quote_the_accent_spellings_do_not_close() {
        // At review: a truncation's or an accent spelling's apostrophe is its own, not a closing
        // quote; a quote mark after a space before `.`, `…` or a number opens one, but an elided
        // decade (`‘80`). `lavorò` is listed, so each probe tests the scan.
        let forms = [("lavorò", "lavorare"), ("bastà", "bastare")];
        for (text, expected) in [
            (
                "Mi serve 'un po' di lavoro'",
                vec!["Mi", "serve", "un", "po", "di", "lavoro"],
            ),
            ("Disse: 'c'e' lavoro'", vec!["Disse", "ci", "e", "lavoro"]),
            (
                "il cosiddetto '...lavoro'",
                vec!["il", "cosiddetto", "lavoro"],
            ),
            (
                "il cosiddetto '…lavoro'",
                vec!["il", "cosiddetto", "lavoro"],
            ),
            ("'2 lavoro'", vec!["lavoro"]),
            ("'un po' e basta'", vec!["un", "po", "e", "basta"]),
        ] {
            assert_eq!(texts(&accented(text, &forms)), expected, "{text}");
        }
        // Outside any quote, the same words read their accent; a closed quote closes.
        assert_eq!(
            texts(&accented("un po' di lavoro' e basta'", &forms)),
            ["un", "poco", "di", "lavorò", "e", "bastà"]
        );
        assert_eq!(
            texts(&accented("‘Vieni!’ lavoro'", &forms)),
            ["Vieni", "lavorò"]
        );
        // An elided decade opens nothing: the truncation after it reads.
        for text in ["anni ‘80, un po'", "nel '68 un po'", "anni ‘80 lavoro'"] {
            let tokens = accented(text, &forms);
            let read = texts(&tokens);
            assert!(
                read.contains(&"poco") || read.contains(&"lavorò"),
                "{text}: {read:?}"
            );
        }
        // A digit run that is no decade opens one.
        assert_eq!(texts(&accented("'2024 lavoro'", &forms)), ["lavoro"]);
        assert_eq!(texts(&accented("'8 lavoro'", &forms)), ["lavoro"]);
    }

    #[test]
    fn the_accent_reads_a_word_without_its_soft_hyphens() {
        // ignore-lingua-soft-hyphens D2: « per‧che' » is `perché`, its span the source's.
        let text = "per\u{AD}che'";
        let tokens = accented(text, &[]);
        assert_eq!(texts(&tokens), ["perché"]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, text.len()));
        // A decomposed vowel before the apostrophe is no plain vowel: `perché'` composed.
        assert_eq!(texts(&accented("perche\u{301}'", &[])), ["perché"]);
    }

    #[test]
    fn spec_scenario_an_italian_hyphenated_compound() {
        let text = "l'anglo-americano";
        let tokens = italian(text);
        assert_eq!(
            read(text, &tokens),
            [("lo", "l'"), ("anglo-americano", "anglo-americano")]
        );
        assert_eq!(tokens[1].parts, ["anglo", "americano"]);
        let tokens = italian("week-end");
        assert_eq!(texts(&tokens), ["week-end"]);
        assert_eq!(tokens[0].parts, ["week", "end"]);
        // Nothing is split inside a compound: its `al` and `nel` stay part of it.
        for (run, parts) in [
            ("al-Qaida", ["al", "Qaida"]),
            ("nel-mezzo", ["nel", "mezzo"]),
        ] {
            let tokens = italian(run);
            assert_eq!(texts(&tokens), [run], "{run}");
            assert_eq!(tokens[0].parts, parts, "{run}");
        }
        // A digit in a run: each piece an Italian word.
        let text = "l'anno-2020 nel-10";
        assert_eq!(
            read(text, &italian(text)),
            [("lo", "l'"), ("anno", "anno"), ("in", "nel"), ("il", "nel")]
        );
    }

    #[test]
    fn spec_scenario_an_italian_elided_word_on_its_own() {
        // D5: what a word card is handed for an elided piece, its apostrophe included.
        for (text, expected) in [
            ("l’", vec![("lo", "l’")]),
            ("L' ", vec![("Lo", "L'")]),
            ("l’ uomo", vec![("lo", "l’"), ("uomo", "uomo")]),
            ("dell’", vec![("dello", "dell’")]),
            ("all’", vec![("a", "all’"), ("lo", "all’")]),
            ("quest'", vec![("questo", "quest'")]),
        ] {
            assert_eq!(read(text, &italian(text)), expected, "{text}");
        }
        // Before a number, an elided form is read as written alone, the digits dropped as any
        // are: never a bare `l`, `nell` or `dell`.
        for (text, expected) in [
            ("l'1", vec![("lo", "l'")]),
            (
                "l'11 settembre",
                vec![("lo", "l'"), ("settembre", "settembre")],
            ),
            ("nell'800", vec![("in", "nell'"), ("lo", "nell'")]),
            ("dell'85", vec![("dello", "dell'")]),
            ("all'80%", vec![("a", "all'"), ("lo", "all'")]),
            ("dall’8", vec![("da", "dall’"), ("lo", "dall’")]),
        ] {
            assert_eq!(read(text, &italian(text)), expected, "{text}");
        }
        // A truncation before a digit is no truncation: its apostrophe stays outside it.
        assert_eq!(read("po'2", &italian("po'2")), [("po", "po")]);
        // A word that is on no list keeps the shared rules: its apostrophe is trimmed.
        assert_eq!(texts(&italian("uomo’ ")), ["uomo"]);
    }

    #[test]
    fn spec_scenario_other_languages_keep_their_words() {
        // The pre-pass is Italian's arm: English, Spanish and French read these words as before.
        let lex = lexicon();
        for language in [StudiedLanguage::English, StudiedLanguage::Spanish] {
            assert_eq!(
                texts(&tokenize("l'uomo nel po' dell'acqua", language, &lex)),
                ["l'uomo", "nel", "po", "dell'acqua"],
                "{language:?}"
            );
        }
        assert_eq!(
            texts(&french("l'uomo nel po' dell'acqua")),
            ["le", "uomo", "nel", "po", "dell'acqua"]
        );
        // And no rule of another language runs on Italian.
        assert_eq!(texts(&italian("don't")), ["don't"]);
        assert_eq!(
            texts(&italian("l'homme au du")),
            ["lo", "homme", "au", "du"]
        );
        // Italian has no U+202F rule (D2): the narrow no-break space is read as English reads it.
        assert_eq!(
            texts(&italian("Pronto\u{202F}?")),
            texts(&tokenize("Pronto\u{202F}?", StudiedLanguage::English, &lex))
        );
    }

    #[test]
    fn spec_scenario_a_decomposed_italian_accent_is_read_composed() {
        // add-lingua-italian-analysis D1: `città` written as `citta` + U+0300 COMBINING GRAVE
        // ACCENT is the composed word, its span the decomposed source, combining mark included.
        let lex = lexicon();
        let text = "citta\u{0300} perche\u{0301} l'attivita\u{0300}";
        let tokens = tokenize(text, IT, &lex);
        assert_eq!(texts(&tokens), ["città", "perché", "lo", "attività"]);
        assert_eq!(&text[tokens[0].start..tokens[0].end], "citta\u{0300}");
        assert_eq!(&text[tokens[3].start..tokens[3].end], "attivita\u{0300}");
        // English text is never normalised: its output must not move.
        assert_eq!(
            texts(&tokenize("citta\u{0300}", StudiedLanguage::English, &lex)),
            ["citta\u{0300}"]
        );
    }

    #[test]
    fn spec_scenario_a_decomposed_italian_elision_and_articulated_preposition() {
        // « l'età » and « dall'età », the accent of `età` a combining mark: the pieces are read
        // as before, and the word after them is composed, its span the source's.
        let text = "l'eta\u{0300} dall'eta\u{0300}";
        let tokens = italian(text);
        assert_eq!(
            read(text, &tokens),
            [
                ("lo", "l'"),
                ("età", "eta\u{0300}"),
                ("da", "dall'"),
                ("lo", "dall'"),
                ("età", "eta\u{0300}")
            ]
        );
        // A decomposed truncation's neighbour and an articulated preposition before a decomposed
        // word: each comparison reads the word composed.
        let text = "Nell'universita\u{0300} c'e\u{0300} un po' di pieta\u{0300}";
        assert_eq!(
            texts(&italian(text)),
            [
                "In",
                "lo",
                "università",
                "ci",
                "è",
                "un",
                "poco",
                "di",
                "pietà"
            ]
        );
    }

    #[test]
    fn spec_scenario_a_decomposed_italian_run_the_pack_lists() {
        // A hyphenated run is one token whose text and parts are composed, so the pack's
        // composed run is found (`pipeline::resolve_lemmas` reads it whole).
        let text = "piu\u{0300}-che-perfetto";
        let tokens = italian(text);
        assert_eq!(texts(&tokens), ["più-che-perfetto"]);
        assert_eq!(tokens[0].parts, ["più", "che", "perfetto"]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, text.len()));
        let (bytes, pool) =
            build_lexicon_blobs(&[("più-che-perfetto", "più-che-perfetto")], &[]).expect("build");
        let listed = FstLexicon::from_slices(bytes, &pool).expect("load");
        let (lemma, parts) =
            crate::analysis::pipeline::resolve_lemmas(&tokens[0], false, IT, &listed);
        assert_eq!((lemma.as_str(), parts.len()), ("più-che-perfetto", 0));
    }

    #[test]
    fn italian_shares_the_rules_of_no_language() {
        let lex = lexicon();
        let text = "x-ray abc123 'team' a b well-being-2 code";
        assert_eq!(
            texts(&tokenize(text, IT, &lex)),
            texts(&tokenize(text, StudiedLanguage::English, &lex))
        );
    }

    #[test]
    fn an_italian_elided_piece_or_truncation_is_told_by_its_span() {
        // D6: `is_elided` widens to a truncation; a contraction's span ends on no apostrophe.
        let flags = |text: &str| -> Vec<(String, bool)> {
            italian(text)
                .iter()
                .map(|token| (token.text.clone(), is_elided(text, token)))
                .collect()
        };
        assert_eq!(
            flags("L’uomo nel po' all'alba"),
            pieces(&[
                ("Lo", true),
                ("uomo", false),
                ("in", false),
                ("il", false),
                ("poco", true),
                ("a", true),
                ("lo", true),
                ("alba", false)
            ])
        );
        assert_eq!(flags("perche' "), pieces(&[("perche", false)]));
        // refine-lingua-italian-reading D3: a truncated articulated preposition's two tokens span
        // its apostrophe, and so does `da'` read as the verb.
        assert_eq!(
            flags("ne' campi mi da' "),
            pieces(&[
                ("in", true),
                ("i", true),
                ("campi", false),
                ("mi", false),
                ("dai", true)
            ])
        );
        // read-lingua-italian-unmarked-spellings D1 widens it to a word whose apostrophe stands for
        // its accent: the stand-in lists `è`, not `perché`.
        assert_eq!(
            flags("c'e' perche' "),
            pieces(&[("ci", true), ("è", true), ("perche", false)])
        );
    }

    #[test]
    fn an_elided_piece_is_told_by_its_span() {
        // match-lingua-french-elided-pieces D2: the span holds the apostrophe; the text, the word
        // the piece stands for, does not.
        assert_eq!(
            elided("L’homme", FR),
            pieces(&[("Le", true), ("homme", false)])
        );
        assert_eq!(
            elided("C'était", FR),
            pieces(&[("Ce", true), ("était", false)])
        );
        // Written on its own, the elided piece keeps its apostrophe.
        assert_eq!(
            elided("l’ homme", FR),
            pieces(&[("le", true), ("homme", false)])
        );
        // The rule runs again after an elision; the two pieces of `au` share a span, which ends on
        // no apostrophe.
        assert_eq!(
            elided("jusqu'au", FR),
            pieces(&[("jusque", true), ("à", false), ("le", false)])
        );
        // A word whose elision is part of it, an edge apostrophe and English's `n't` end on none.
        assert_eq!(elided("aujourd'hui", FR), pieces(&[("aujourd'hui", false)]));
        assert_eq!(elided("presqu'île", FR), pieces(&[("presqu'île", false)]));
        assert_eq!(elided("vélib' ", FR), pieces(&[("vélib", false)]));
        assert_eq!(
            elided("don't", StudiedLanguage::English),
            pieces(&[("do", false), ("not", false)])
        );
    }

    #[test]
    fn spec_scenario_contracted_articles() {
        let text = "au marché, aux halles, du pain, des pommes";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [
                ("à", "au"),
                ("le", "au"),
                ("marché", "marché"),
                ("à", "aux"),
                ("les", "aux"),
                ("halles", "halles"),
                ("du", "du"),
                ("pain", "pain"),
                ("des", "des"),
                ("pommes", "pommes"),
            ]
        );
        // The two tokens of a contraction share its span, as `don't` and `del` do.
        assert_eq!((tokens[0].start, tokens[0].end), (0, 2));
        assert_eq!((tokens[1].start, tokens[1].end), (0, 2));
        // Other words that start like them stay whole, and Spanish's `al`/`del` is Spanish's.
        assert_eq!(
            texts(&french("auquel auxquels duquel desquels al del")),
            ["auquel", "auxquels", "duquel", "desquels", "al", "del"]
        );
    }

    #[test]
    fn spec_scenario_a_capitalised_contraction() {
        // D4: the capital goes on the read word's own first letter; `à` is two bytes where `A` is
        // one, which the casing English and Spanish share would slice through.
        let text = "Au revoir";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [("À", "Au"), ("le", "Au"), ("revoir", "revoir")]
        );
        assert_eq!(texts(&french("AU")), ["À", "le"]);
        assert_eq!(texts(&french("AUX")), ["À", "les"]);
        assert_eq!(texts(&french("Aux")), ["À", "les"]);
    }

    #[test]
    fn a_contraction_inside_a_compound_stays_part_of_it() {
        for (run, parts) in [
            ("au-delà", ["au", "delà"].as_slice()),
            ("au-dessus", ["au", "dessus"].as_slice()),
        ] {
            let tokens = french(run);
            assert_eq!(tokens.len(), 1, "{run}");
            assert_eq!(tokens[0].text, run);
            assert_eq!(tokens[0].parts, parts, "{run}");
        }
        // An elision on the run's first piece is split off; the compound keeps its `au`.
        let text = "jusqu'au-delà";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [("jusque", "jusqu'"), ("au-delà", "au-delà")]
        );
        assert_eq!(tokens[1].parts, ["au", "delà"]);
    }

    #[test]
    fn spec_scenario_a_subject_pronoun_after_its_verb() {
        let text = "dit-il";
        let tokens = french(text);
        assert_eq!(read(text, &tokens), [("dit", "dit"), ("il", "il")]);
        assert_eq!((tokens[1].start, tokens[1].end), (4, 6));
        assert!(tokens.iter().all(|t| t.parts.is_empty()));
    }

    #[test]
    fn spec_scenario_the_euphonic_t() {
        let text = "a-t-il";
        let tokens = french(text);
        assert_eq!(read(text, &tokens), [("a", "a"), ("il", "il")]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, 1));
        assert_eq!((tokens[1].start, tokens[1].end), (4, 6));
        for (text, expected) in [
            ("pense-t-elle", ["pense", "elle"]),
            ("va-t-on", ["va", "on"]),
            ("Viendront-ils", ["Viendront", "ils"]),
        ] {
            assert_eq!(texts(&french(text)), expected, "{text}");
        }
        // The euphonic `t` only before a subject pronoun: elsewhere the run is a compound.
        assert_eq!(texts(&french("dit-t-nous")), ["dit-t-nous"]);
        assert_eq!(texts(&french("dit-t")), ["dit-t"]);
    }

    #[test]
    fn spec_scenario_a_question_with_an_elision() {
        let text = "Qu’est-ce";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [("Que", "Qu’"), ("est", "est"), ("ce", "ce")]
        );
        assert_eq!((tokens[0].start, tokens[0].end), (0, 5));
        assert_eq!((tokens[2].start, tokens[2].end), (9, 11));
        assert_eq!(texts(&french("Est-ce")), ["Est", "ce"]);
    }

    #[test]
    fn spec_scenario_an_imperative_with_its_pronouns() {
        let text = "Donne-m'en";
        assert_eq!(
            read(text, &french(text)),
            [("Donne", "Donne"), ("moi", "m'"), ("en", "en")]
        );
        let text = "Va-t'en";
        assert_eq!(
            read(text, &french(text)),
            [("Va", "Va"), ("toi", "t'"), ("en", "en")]
        );
        let text = "allez-vous-en";
        assert_eq!(
            read(text, &french(text)),
            [("allez", "allez"), ("vous", "vous"), ("en", "en")]
        );
        for (text, expected) in [
            ("coupez-les", ["coupez", "les"].as_slice()),
            ("dis-le-moi", ["dis", "le", "moi"].as_slice()),
            ("Vas-y", ["Vas", "y"].as_slice()),
            ("Prends-en", ["Prends", "en"].as_slice()),
            ("mets-l'y", ["mets", "le", "y"].as_slice()),
        ] {
            assert_eq!(texts(&french(text)), expected, "{text}");
        }
    }

    #[test]
    fn spec_scenario_a_compound_the_pack_lists() {
        for run in ["rendez-vous", "peut-être", "c'est-à-dire"] {
            let tokens = french(run);
            assert_eq!(texts(&tokens), [run], "{run}");
            assert_eq!((tokens[0].start, tokens[0].end), (0, run.len()));
        }
        // Not listed, `rendez-vous` reads as an inversion (D5's risk, checked on the real tables).
        let lex = lexicon();
        assert_eq!(
            texts(&tokenize("rendez-vous", FR, &lex)),
            ["rendez", "vous"]
        );
    }

    #[test]
    fn spec_scenario_an_elision_before_a_compound() {
        let text = "l'arc-en-ciel";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [("le", "l'"), ("arc-en-ciel", "arc-en-ciel")]
        );
        assert_eq!(tokens[1].parts, ["arc", "en", "ciel"]);
        // Not listed, the rest of the run is a compound judged by its parts.
        let text = "d'arc-en-terre";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [("de", "d'"), ("arc-en-terre", "arc-en-terre")]
        );
        assert_eq!(tokens[1].parts, ["arc", "en", "terre"]);
    }

    #[test]
    fn spec_scenario_runs_that_are_no_inversion() {
        // D5: a piece that is no pronoun, or a capital, keeps the compound rule.
        for run in [
            "celui-ci",
            "celle-là",
            "moi-même",
            "Jean-Pierre",
            "Saint-Y",
            "DIT-IL",
        ] {
            let tokens = french(run);
            assert_eq!(texts(&tokens), [run], "{run}");
            assert_eq!(tokens[0].parts, run.split('-').collect::<Vec<_>>(), "{run}");
        }
    }

    #[test]
    fn a_digit_in_a_french_run_reads_each_piece_as_a_word() {
        // The rule that belongs to no language, each piece read as a French word.
        let text = "l'an-2000 au-10";
        assert_eq!(
            read(text, &french(text)),
            [("le", "l'"), ("an", "an"), ("à", "au"), ("le", "au")]
        );
    }

    #[test]
    fn spec_scenario_no_english_rule_runs_on_french() {
        let lex = lexicon();
        assert_eq!(texts(&tokenize("Don't", FR, &lex)), ["Don't"]);
        assert_eq!(texts(&tokenize("won't", FR, &lex)), ["won't"]);
    }

    #[test]
    fn french_rules_never_run_on_english_or_spanish() {
        // The pre-pass is French's arm: the other languages read these words as before.
        let lex = lexicon();
        let text = "l'homme au dit-il";
        assert_eq!(
            texts(&tokenize(text, StudiedLanguage::English, &lex)),
            ["l'homme", "au", "dit-il"]
        );
        assert_eq!(
            texts(&tokenize(text, StudiedLanguage::Spanish, &lex)),
            ["l'homme", "au", "dit-il"]
        );
    }

    #[test]
    fn french_text_is_read_in_nfc() {
        // add-lingua-french-analysis D1: a decomposed accent is read composed.
        let lex = lexicon();
        let text = "e\u{0301}te\u{0301} chaud";
        assert_eq!(texts(&tokenize(text, FR, &lex)), ["été", "chaud"]);
        // And the rules that belong to no language apply as they do to English.
        let shared = "x-ray abc123 'team' a b well-being-2 code";
        assert_eq!(
            texts(&tokenize(shared, FR, &lex)),
            texts(&tokenize(shared, StudiedLanguage::English, &lex))
        );
    }

    #[test]
    fn spec_scenario_a_decomposed_accent_in_french() {
        // `mémoire` written with `e` + U+0301: one token, composed, spanning the decomposed bytes.
        let text = "me\u{301}moire vive";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [("mémoire", "me\u{301}moire"), ("vive", "vive")]
        );
        assert_eq!((tokens[0].start, tokens[0].end), (0, 9));
    }

    #[test]
    fn spec_scenario_a_decomposed_elision() {
        // `ç'a` written with `c` + U+0327: the elided `ç'` is found, `ça` spanning the `c`, its
        // cedilla and the apostrophe.
        let text = "c\u{327}'a e\u{301}te\u{301}";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [
                ("ça", "c\u{327}'"),
                ("a", "a"),
                ("été", "e\u{301}te\u{301}")
            ]
        );
        assert_eq!((tokens[0].start, tokens[0].end), (0, 4));
        // Capitalised, the read word takes the capital on its own letter.
        let text = "C\u{327}\u{2019}e\u{301}tait";
        assert_eq!(
            read(text, &french(text)),
            [("Ça", "C\u{327}\u{2019}"), ("était", "e\u{301}tait")]
        );
    }

    #[test]
    fn spec_scenario_a_decomposed_run_the_pack_lists() {
        // `peut-être` written with `e` + U+0302 is the run the pack lists: one token.
        let text = "peut-e\u{302}tre";
        let tokens = french(text);
        assert_eq!(read(text, &tokens), [("peut-être", text)]);
        assert!(tokens[0].parts.iter().all(|p| p == "peut" || p == "être"));
        // A run the pack does not list keeps its parts, each composed.
        let text = "Saint-E\u{301}tienne";
        let tokens = french(text);
        assert_eq!(read(text, &tokens), [("Saint-Étienne", text)]);
        assert_eq!(tokens[0].parts, ["Saint", "Étienne"]);
    }

    #[test]
    fn a_decomposed_capital_keeps_its_capital_composed() {
        let text = "E\u{301}cole de l'E\u{301}tat";
        let tokens = french(text);
        assert_eq!(
            read(text, &tokens),
            [
                ("École", "E\u{301}cole"),
                ("de", "de"),
                ("le", "l'"),
                ("État", "E\u{301}tat")
            ]
        );
    }

    #[test]
    fn a_decomposed_contraction_and_elided_word_alone_are_read_composed() {
        // `au` is never decomposed, but the word after it is; `ç’` alone reads `ça`.
        let text = "au cafe\u{301} c\u{327}\u{2019} ";
        assert_eq!(
            read(text, &french(text)),
            [
                ("à", "au"),
                ("le", "au"),
                ("café", "cafe\u{301}"),
                ("ça", "c\u{327}\u{2019}")
            ]
        );
    }

    #[test]
    fn english_text_holding_a_combining_mark_is_not_composed() {
        let lex = lexicon();
        let text = "cafe\u{301} re\u{301}sume\u{301}";
        assert_eq!(
            texts(&tokenize(text, StudiedLanguage::English, &lex)),
            ["cafe\u{301}", "re\u{301}sume\u{301}"]
        );
        assert_eq!(texts(&tokenize(text, FR, &lex)), ["café", "résumé"]);
    }

    #[test]
    fn spec_scenario_everyday_english() {
        let tokens = tokenize(
            "Teams don't ship code.",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(texts(&tokens), ["Teams", "do", "not", "ship", "code"]);
    }

    #[test]
    fn contraction_halves_share_the_source_span() {
        let text = "Teams don't ship.";
        let tokens = tokenize(text, StudiedLanguage::English, &lexicon());
        let dont_start = text.find("don't").expect("present");
        assert_eq!(tokens[1].text, "do");
        assert_eq!(tokens[2].text, "not");
        assert_eq!(
            (tokens[1].start, tokens[1].end),
            (dont_start, dont_start + "don't".len())
        );
        assert_eq!(
            (tokens[2].start, tokens[2].end),
            (tokens[1].start, tokens[1].end)
        );
    }

    #[test]
    fn irregular_contractions_change_their_base() {
        let tokens = tokenize(
            "They won't and can't.",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(texts(&tokens), ["They", "will", "not", "and", "can", "not"]);
    }

    #[test]
    fn cased_contraction_keeps_sentence_case() {
        let tokens = tokenize("Don't stop.", StudiedLanguage::English, &lexicon());
        assert_eq!(texts(&tokens), ["Do", "not", "stop"]);
    }

    #[test]
    fn typographic_apostrophes_behave_like_plain_ones() {
        let tokens = tokenize(
            "Teams don\u{2019}t ship.",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(texts(&tokens), ["Teams", "do", "not", "ship"]);
    }

    #[test]
    fn edge_apostrophes_are_stripped_but_internal_ones_kept() {
        let tokens = tokenize(
            "'tis the sailors' o'clock",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(texts(&tokens), ["tis", "the", "sailors", "o'clock"]);
    }

    #[test]
    fn single_letters_need_the_lexicon() {
        // "I" and "a" are in the lexicon; a stray "x" is not.
        let tokens = tokenize("I read a book x", StudiedLanguage::English, &lexicon());
        assert_eq!(texts(&tokens), ["I", "read", "a", "book"]);
    }

    #[test]
    fn digit_bearing_tokens_are_dropped() {
        let tokens = tokenize("2026 saw 3D movies", StudiedLanguage::English, &lexicon());
        assert_eq!(texts(&tokens), ["saw", "movies"]);
    }

    #[test]
    fn hyphenated_compound_is_one_token_spanning_the_whole_run() {
        let text = "The read-only flag is set.";
        let tokens = tokenize(text, StudiedLanguage::English, &lexicon());
        assert_eq!(texts(&tokens), ["The", "read-only", "flag", "is", "set"]);
        let compound = &tokens[1];
        assert_eq!(compound.text, "read-only");
        assert_eq!(compound.parts, ["read", "only"]);
        // The span covers the whole compound, hyphen included.
        assert_eq!(&text[compound.start..compound.end], "read-only");
    }

    #[test]
    fn multi_hyphen_compound_absorbs_every_piece() {
        let tokens = tokenize(
            "A state-of-the-art design.",
            StudiedLanguage::English,
            &lexicon(),
        );
        let compound = tokens
            .iter()
            .find(|t| t.text.contains('-'))
            .expect("compound present");
        assert_eq!(compound.text, "state-of-the-art");
        assert_eq!(compound.parts, ["state", "of", "the", "art"]);
    }

    #[test]
    fn a_hyphen_between_spaces_is_not_a_compound() {
        // "code - team" is two words and a stray dash, not "code-team".
        let tokens = tokenize("ship code - team", StudiedLanguage::English, &lexicon());
        assert_eq!(texts(&tokens), ["ship", "code", "team"]);
        assert!(tokens.iter().all(|t| t.parts.is_empty()));
    }

    #[test]
    fn a_double_hyphen_is_an_em_dash_not_a_compound() {
        // "wait--what" / "cost--benefit" are em-dashes: two words, not one.
        let tokens = tokenize(
            "ship code--team and code---ship now",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(
            texts(&tokens),
            ["ship", "code", "team", "and", "code", "ship", "now"]
        );
        assert!(tokens.iter().all(|t| t.parts.is_empty()));
    }

    #[test]
    fn a_digit_bearing_run_degrades_to_its_clean_pieces() {
        // A digit means an identifier: keep the clean pieces (as if unfused),
        // drop only the digit-bearing one — never swallow `well`/`being`.
        let tokens = tokenize(
            "ship well-being-2 and utf-8 code",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(
            texts(&tokens),
            ["ship", "well", "being", "and", "utf", "code"]
        );
        // The salvaged pieces are ordinary single-word tokens, not compounds.
        assert!(tokens.iter().all(|t| t.parts.is_empty()));
    }

    #[test]
    fn a_single_letter_piece_survives_inside_a_compound() {
        // "x" alone is dropped, but "x-ray" is a word — the compound stands as
        // a unit rather than being pruned piece by piece.
        let tokens = tokenize("an x-ray scan", StudiedLanguage::English, &lexicon());
        let compound = tokens
            .iter()
            .find(|t| t.text.contains('-'))
            .expect("compound present");
        assert_eq!(compound.text, "x-ray");
        assert_eq!(compound.parts, ["x", "ray"]);
    }

    // ignore-lingua-soft-hyphens: a soft hyphen (U+00AD) is not part of a word. In these
    // tests `~` stands for one, as `‧` does in the spec's scenarios.

    /// `text` with a soft hyphen for each `~`.
    fn shy(text: &str) -> String {
        text.replace('~', "\u{AD}")
    }

    /// `text` tokenised with its soft hyphens and without them: the same texts and parts, and
    /// each span covering, as written, the word the clean span covers. Returns the hyphenated
    /// text and its tokens.
    fn reads_as_clean(
        text: &str,
        language: StudiedLanguage,
        lexicon: &(impl Lexicon + ?Sized),
    ) -> (String, Vec<Token>) {
        let written = shy(text);
        let clean = text.replace('~', "");
        assert_ne!(written, clean, "{text} holds no soft hyphen");
        let hyphenated = tokenize(&written, language, lexicon);
        let plain = tokenize(&clean, language, lexicon);
        assert_eq!(texts(&hyphenated), texts(&plain), "{text}");
        for (h, p) in hyphenated.iter().zip(&plain) {
            assert_eq!(h.parts, p.parts, "{text}");
            assert!(
                !h.text.contains(SOFT_HYPHEN) && h.parts.iter().all(|w| !w.contains(SOFT_HYPHEN)),
                "{text}"
            );
            assert_eq!(
                without_soft_hyphens(&written[h.start..h.end]),
                &clean[p.start..p.end],
                "{text}: {}",
                h.text
            );
        }
        (written, hyphenated)
    }

    #[test]
    fn without_soft_hyphens_borrows_when_there_is_nothing_to_strip() {
        assert!(matches!(
            without_soft_hyphens("vida"),
            Cow::Borrowed("vida")
        ));
        assert!(matches!(without_soft_hyphens(""), Cow::Borrowed("")));
        // `í` is `C3 AD`: its byte `AD` is no soft hyphen.
        assert!(matches!(
            without_soft_hyphens("aquí"),
            Cow::Borrowed("aquí")
        ));
        assert!(!is_hyphen(" ") && is_hyphen("-") && is_hyphen("-\u{AD}") && !is_hyphen("--"));
        let stripped = without_soft_hyphens("\u{AD}vi\u{AD}\u{AD}da\u{AD}");
        assert!(matches!(stripped, Cow::Owned(_)));
        assert_eq!(stripped, "vida");
    }

    #[test]
    fn a_hyphen_or_a_euphonic_t_joins_two_tokens() {
        for gap in ["-", "-\u{AD}", "\u{AD}-", "-t-", "-T-", "-\u{AD}t-"] {
            assert!(is_hyphen_join(gap), "{gap:?} joins");
        }
        for gap in [" ", "--", ", ", "", "t", "-t", "- ", " -t- "] {
            assert!(!is_hyphen_join(gap), "{gap:?} does not join");
        }
    }

    #[test]
    fn spec_scenario_a_spanish_word_read_whole() {
        let (written, tokens) = reads_as_clean("vi~da", StudiedLanguage::Spanish, &lexicon());
        assert_eq!(texts(&tokens), ["vida"]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, 6));
        assert_eq!(&written[0..6], written);
    }

    #[test]
    fn spec_scenario_a_french_line_is_read_as_its_words() {
        let line = "On a chan~té en~semble jus~qu’au ma~tin, per~sonne n’a vou~lu dor~mir.";
        let (written, tokens) = reads_as_clean(line, FR, &french_lexicon());
        assert_eq!(
            texts(&tokens),
            [
                "On", "a", "chanté", "ensemble", "jusque", "à", "le", "matin", "personne", "ne",
                "a", "voulu", "dormir"
            ]
        );
        // `jusque` spans « jus‧qu’ » as written, `à` and `le` the « au » they share.
        let spans: Vec<&str> = tokens.iter().map(|t| &written[t.start..t.end]).collect();
        assert_eq!(spans[4], shy("jus~qu’"));
        assert_eq!((spans[5], spans[6]), ("au", "au"));
        assert_eq!(spans[2], shy("chan~té"));
    }

    #[test]
    fn spec_scenario_english_words_and_a_contraction() {
        let line = "The gov~ern~ment could~n't an~swer the ques~tion yes~ter~day af~ter~noon.";
        let (written, tokens) = reads_as_clean(line, StudiedLanguage::English, &lexicon());
        assert_eq!(
            texts(&tokens),
            [
                "The",
                "government",
                "could",
                "not",
                "answer",
                "the",
                "question",
                "yesterday",
                "afternoon"
            ]
        );
        // `could` and `not` share the span of « could‧n't ».
        assert_eq!(
            (tokens[2].start, tokens[2].end),
            (tokens[3].start, tokens[3].end)
        );
        assert_eq!(&written[tokens[2].start..tokens[2].end], shy("could~n't"));
    }

    #[test]
    fn spec_scenario_a_french_elision() {
        let (written, tokens) = reads_as_clean("lors~qu’il", FR, &french_lexicon());
        assert_eq!(texts(&tokens), ["lorsque", "il"]);
        assert_eq!((tokens[0].start, tokens[0].end), (0, 11));
        assert_eq!((tokens[1].start, tokens[1].end), (11, 13));
        assert_eq!(&written[0..11], shy("lors~qu’"));
    }

    #[test]
    fn a_soft_hyphen_opening_a_block_ending_a_word_or_doubled() {
        // UAX #29 gives a soft hyphen opening the text a segment of its own, and glues one after
        // a space to that space: the word starts after it either way.
        for language in StudiedLanguage::ALL {
            let lex = if language == FR {
                french_lexicon()
            } else {
                lexicon()
            };
            let (_, tokens) = reads_as_clean("~vida es~ ~bella", language, &lex);
            assert_eq!(texts(&tokens), ["vida", "es", "bella"], "{language:?}");
            assert_eq!((tokens[0].start, tokens[0].end), (2, 6));
            assert_eq!((tokens[1].start, tokens[1].end), (7, 11), "{language:?}");
            assert_eq!((tokens[2].start, tokens[2].end), (14, 19), "{language:?}");
            let (_, tokens) = reads_as_clean("vi~~da", language, &lex);
            assert_eq!(texts(&tokens), ["vida"]);
            assert_eq!((tokens[0].start, tokens[0].end), (0, 8));
        }
    }

    #[test]
    fn every_rule_reads_the_word_without_its_soft_hyphens() {
        let lex = lexicon();
        let fr = french_lexicon();
        let lines: &[(StudiedLanguage, &str)] = &[
            // Edge apostrophes, a digit, a single letter, compounds, a digit-bearing run.
            (
                StudiedLanguage::English,
                "'te~am' ab~c123 a~ b~ x-r~ay well-be~ing-2 co~de",
            ),
            // The irregular contractions keep their table.
            (StudiedLanguage::English, "Won~'t can~'t ain~'t Do~n’t"),
            // A soft hyphen right after a hard one belongs to the hyphen run.
            (StudiedLanguage::English, "re~po-~wide state-of-~the-art"),
            // `al`/`del`, NFC, a compound.
            (
                StudiedLanguage::Spanish,
                "De~l mar a~l cen~tro Esta\u{301}~ al-an~da~lus",
            ),
            // Elisions, `au`/`aux`, an elided word alone, listed runs, inversions, the
            // euphonic `t`, an elision before a compound, NFC.
            (
                FR,
                "L’hom~me qu’on Au~x jus~qu’i~ci l’ s’~il ren~dez-vous peut-~être \
                 dit-~il a-t~-il al~lez-vous-en mets-l’~y l’arc-en-ci~el c\u{327}~'a",
            ),
            // A soft hyphen right after the apostrophe of an elision.
            (FR, "l’~homme d’~abord qu’~on"),
            // Italian's elisions, the `di` family, a split preposition, an elided one, a
            // truncation, an elided word alone, `diss'io`, an elision before a compound.
            (
                IT,
                "L’uo~mo del~l’ac~qua ne~l po~' Fa~' al~l’al~ba l’ di~ss’io l’an~glo-ame~ri~ca~no",
            ),
            (IT, "l’~uomo c’~è"),
        ];
        let it = italian_lexicon();
        for &(language, line) in lines {
            let lexicon: &FstLexicon<Vec<u8>> = match language {
                FR => &fr,
                IT => &it,
                _ => &lex,
            };
            reads_as_clean(line, language, lexicon);
        }
        // Spot checks of what those lines read.
        let (_, tokens) = reads_as_clean("Won~'t", StudiedLanguage::English, &lex);
        assert_eq!(texts(&tokens), ["Will", "not"]);
        let (_, tokens) = reads_as_clean("De~l mar", StudiedLanguage::Spanish, &lex);
        assert_eq!(texts(&tokens), ["De", "el", "mar"]);
        let (_, tokens) = reads_as_clean("re~po-~wide", StudiedLanguage::English, &lex);
        assert_eq!(texts(&tokens), ["repo-wide"]);
        assert_eq!(tokens[0].parts, ["repo", "wide"]);
        let (_, tokens) = reads_as_clean("peut-~être a-t~-il", FR, &fr);
        assert_eq!(texts(&tokens), ["peut-être", "a", "il"]);
        let (_, tokens) = reads_as_clean("l’~homme s’~il", FR, &fr);
        assert_eq!(texts(&tokens), ["le", "homme", "si", "il"]);
        let (_, tokens) = reads_as_clean("del~l’ac~qua ne~l po~' di~ss’io", IT, &it);
        assert_eq!(
            texts(&tokens),
            ["dello", "acqua", "in", "il", "poco", "dissi", "io"]
        );
    }
}
