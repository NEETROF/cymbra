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
//! (add-lingua-french-analysis D1). Adding a studied language means adding a
//! pre-pass, not touching the tokeniser.
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
/// (add-lingua-french-tokenisation D6). The text is the word without its soft
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
/// inversions. All share the rules that belong to no language: segmentation,
/// the hyphen run, the compound rule, the digit drop, the edge-apostrophe trim
/// and the single-letter rule.
pub fn tokenize(
    text: &str,
    language: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> Vec<Token> {
    let words: Vec<(usize, &str)> = match language {
        StudiedLanguage::French => french_words(text),
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
        if language == StudiedLanguage::French {
            if j - i >= 2 {
                push_french_run(&mut tokens, text, &words[i..j], lexicon);
            } else {
                push_french_word(&mut tokens, text, first, start, lexicon);
            }
        } else if j - i >= 2 {
            push_compound(
                &mut tokens,
                &text[start..end],
                &words[i..j],
                start,
                end,
                language,
                lexicon,
            );
        } else {
            push_word(&mut tokens, first, start, end, language, lexicon);
        }
        i = j;
    }
    tokens
}

/// Emits an ordinary single-word token, applying the pre-pass: apostrophe
/// normalisation/trimming, NFC (Spanish, French), the language's contraction split
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
/// combining marks included. Every rule reads the word without its soft hyphens,
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

/// The word in NFC for Spanish and French, so a decomposed accent reads as the
/// pack's precomposed one (add-lingua-spanish-analysis D1,
/// add-lingua-french-analysis D1); English text as it came — its output must not
/// move.
fn nfc_for(word: &str, language: StudiedLanguage) -> String {
    match language {
        StudiedLanguage::English => word.to_owned(),
        StudiedLanguage::Spanish | StudiedLanguage::French => word.nfc().collect(),
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
/// takes a two-byte `à`; `du` and `des` stay whole (M21).
fn split_contraction(lower: &str, language: StudiedLanguage) -> Option<(&str, &'static str)> {
    match language {
        StudiedLanguage::English => split_english_contraction(lower),
        StudiedLanguage::Spanish => match lower {
            "al" => Some(("a", "el")),
            "del" => Some(("de", "el")),
            _ => None,
        },
        StudiedLanguage::French => None,
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
/// « lors‧qu » the elided `lorsqu` (ignore-lingua-soft-hyphens D2).
fn french_lowercase(written: &str) -> String {
    without_soft_hyphens(written).to_lowercase().nfc().collect()
}

/// The straight and the typographic apostrophe, the two French text is written
/// with (D3). U+02BC, U+2018 and U+FF07 are not read as apostrophes.
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
/// cannot take. A soft hyphen before that letter is no letter.
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
        ];
        for &(language, line) in lines {
            let lexicon: &FstLexicon<Vec<u8>> = if language == FR { &fr } else { &lex };
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
    }
}
