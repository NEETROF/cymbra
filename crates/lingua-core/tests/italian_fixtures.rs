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

//! Non-regression fixtures for Italian's analysis, grouped by rule.
//!
//! Its tokenisation pre-pass (add-lingua-italian-tokenisation, task 1.6 — at least
//! 80 cases): each elided form, the rule again on what follows, the words that stay
//! whole, the `di` family, the other articulated prepositions, the truncations, the
//! apostrophe for an accent and the quotes, the hyphenated runs. Every case states
//! each token beside the source text its span covers, so every split is checked
//! with its spans.
//!
//! The rules of its own analysis (add-lingua-italian-analysis, task 6.1 — at least
//! 70 cases): NFC in the pre-pass (decomposed words, elisions and articulated
//! prepositions), the cascade (listed forms, the enclitic rule by each base shape,
//! the combined clitics, the five monosyllabic imperatives with and without
//! doubling, `ecco`, a past participle, the words never split, a base the pack does
//! not list, an unlisted plural left as written), the closed classes in a phrase
//! (each class, each word left out) — each asserting every token's lemma and
//! function-word flag through the phrase gloss — and the names rule over a page,
//! through `analyse_page` (each branch of its design's D4).
//!
//! The apostrophe that stands for an accent (read-lingua-italian-unmarked-spellings, task 1.2,
//! `ACCENT_CASES` over a stand-in listing the accented words): each vowel, after an elided
//! piece, on a capital, with two accents listed, beside change 6's truncations, the apocopes the
//! rule never reads, a word the pack does not accent and the quotes.
//!
//! A word standing alone in capitals (keep-lingua-italian-acronyms-and-names, task 1.2,
//! `ACRONYM_CASES` through the phrase gloss over a stand-in listing `NATO` and `USA` as their own
//! lemmas): in a line of lowercase words, in a headline, beside an articulated preposition or an
//! elided piece in either case, a contraction in capitals, an acronym the pack lists no row for,
//! and the participle in lowercase.
//!
//! Any change to a fixture's expectation is a behavioural change of the Italian
//! analyser and demands an `ITALIAN_ANALYZER_VERSION` bump; English's, Spanish's
//! and French's fixtures and baselines do not move.
//!
//! The lexicons are small stand-ins for the forms tables
//! (add-lingua-italian-forms-tables): the tokeniser's lists the single-letter words
//! it drops when unlisted — `è`, `a`, `e`, `o`; the analysis's the verbs, forms and
//! words its cases name.

use lingua_core::analysis::ITALIAN_ANALYZER_VERSION;
use lingua_core::analysis::function_words::is_function_word;
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::lexicon::{FstLexicon, Lexicon, build_lexicon_blobs};
use lingua_core::analysis::percent::TokenClass;
use lingua_core::analysis::tokenize::{Token, tokenize};
use lingua_core::engine::{analyse_page, gloss_phrase, word_grammar};
use lingua_core::knowledge::KnowledgeState;
use lingua_core::packs::pack::section;
use lingua_core::packs::{Pack, PackMeta, write_container};

const IT: StudiedLanguage = StudiedLanguage::Italian;

/// The forms the stand-in pack lists, with their lemmas.
const FORMS: &[(&str, &str)] = &[("è", "essere")];

/// Lemmas the stand-in pack lists as their own forms: the one-letter words the
/// single-letter rule keeps.
const LEMMAS: &[&str] = &["a", "e", "o", "essere"];

/// (text tokenised as Italian, the tokens as (text, the source text the span covers)).
type Case = (&'static str, &'static [(&'static str, &'static str)]);

const CASES: &[Case] = &[
    // --- each elided form, read as the word it stands for, its own span (D3) ---
    ("l'uomo", &[("lo", "l'"), ("uomo", "uomo")]),
    ("L'uomo", &[("Lo", "L'"), ("uomo", "uomo")]),
    ("l’amica", &[("lo", "l’"), ("amica", "amica")]),
    ("un'amica", &[("una", "un'"), ("amica", "amica")]),
    ("Un’ora", &[("Una", "Un’"), ("ora", "ora")]),
    ("d'oro", &[("di", "d'"), ("oro", "oro")]),
    ("D’accordo", &[("Di", "D’"), ("accordo", "accordo")]),
    ("c'è", &[("ci", "c'"), ("è", "è")]),
    ("C’era", &[("Ci", "C’"), ("era", "era")]),
    ("v'è", &[("vi", "v'"), ("è", "è")]),
    ("m'ha", &[("mi", "m'"), ("ha", "ha")]),
    ("t'amo", &[("ti", "t'"), ("amo", "amo")]),
    ("s'era", &[("si", "s'"), ("era", "era")]),
    ("n'è", &[("ne", "n'"), ("è", "è")]),
    ("ch'io", &[("che", "ch'"), ("io", "io")]),
    ("gl'italiani", &[("gli", "gl'"), ("italiani", "italiani")]),
    ("dov'è", &[("dove", "dov'"), ("è", "è")]),
    ("Com’è", &[("Come", "Com’"), ("è", "è")]),
    ("cos'è", &[("cosa", "cos'"), ("è", "è")]),
    ("quand'ero", &[("quando", "quand'"), ("ero", "ero")]),
    ("dev'essere", &[("deve", "dev'"), ("essere", "essere")]),
    ("diss'egli", &[("disse", "diss'"), ("egli", "egli")]),
    ("diss'io", &[("dissi", "diss'"), ("io", "io")]),
    ("quest'anno", &[("questo", "quest'"), ("anno", "anno")]),
    ("Quest’anno", &[("Questo", "Quest’"), ("anno", "anno")]),
    ("quell'amico", &[("quello", "quell'"), ("amico", "amico")]),
    ("anch'io", &[("anche", "anch'"), ("io", "io")]),
    ("senz'altro", &[("senza", "senz'"), ("altro", "altro")]),
    ("tutt'altro", &[("tutto", "tutt'"), ("altro", "altro")]),
    ("bell'uomo", &[("bello", "bell'"), ("uomo", "uomo")]),
    ("buon'anima", &[("buona", "buon'"), ("anima", "anima")]),
    ("grand'uomo", &[("grande", "grand'"), ("uomo", "uomo")]),
    ("pover'uomo", &[("povero", "pover'"), ("uomo", "uomo")]),
    (
        "Sant’Antonio",
        &[("Santo", "Sant’"), ("Antonio", "Antonio")],
    ),
    ("mezz'ora", &[("mezzo", "mezz'"), ("ora", "ora")]),
    ("nient'altro", &[("niente", "nient'"), ("altro", "altro")]),
    ("quant'è", &[("quanto", "quant'"), ("è", "è")]),
    ("cent'anni", &[("cento", "cent'"), ("anni", "anni")]),
    ("vent'anni", &[("venti", "vent'"), ("anni", "anni")]),
    ("sott'acqua", &[("sotto", "sott'"), ("acqua", "acqua")]),
    (
        "nessun'altra",
        &[("nessuna", "nessun'"), ("altra", "altra")],
    ),
    (
        "qualcos'altro",
        &[("qualcosa", "qualcos'"), ("altro", "altro")],
    ),
    (
        "qualcun'altra",
        &[("qualcuna", "qualcun'"), ("altra", "altra")],
    ),
    (
        "ciascun'altra",
        &[("ciascuna", "ciascun'"), ("altra", "altra")],
    ),
    ("alcun'altra", &[("alcuna", "alcun'"), ("altra", "altra")]),
    ("QUEST'ANNO", &[("Questo", "QUEST'"), ("ANNO", "ANNO")]),
    // --- the rule again on what follows; an elided word on its own (D3, D5) ---
    ("un'altr'ora", &[("una", "un'"), ("altr'ora", "altr'ora")]),
    (
        "dall'altro",
        &[("da", "dall'"), ("lo", "dall'"), ("altro", "altro")],
    ),
    ("l’", &[("lo", "l’")]),
    ("l’ uomo", &[("lo", "l’"), ("uomo", "uomo")]),
    ("Quest' ", &[("Questo", "Quest'")]),
    ("d' ", &[("di", "d'")]),
    // Before a number, as written alone, the digits dropped: never a bare `l`, `nell`, `dell`.
    ("l'1", &[("lo", "l'")]),
    (
        "l'11 settembre",
        &[("lo", "l'"), ("settembre", "settembre")],
    ),
    ("nell'800", &[("in", "nell'"), ("lo", "nell'")]),
    ("dell'85", &[("dello", "dell'")]),
    ("all'80%", &[("a", "all'"), ("lo", "all'")]),
    ("dall’8", &[("da", "dall’"), ("lo", "dall’")]),
    // --- the words that stay whole ---
    ("altr'ora", &[("altr'ora", "altr'ora")]),
    ("tal'altro", &[("tal'altro", "tal'altro")]),
    ("po'ero", &[("po'ero", "po'ero")]),
    ("lʼuomo", &[("lʼuomo", "lʼuomo")]),
    ("uomo", &[("uomo", "uomo")]),
    ("dirmelo", &[("dirmelo", "dirmelo")]),
    ("Don't", &[("Don't", "Don't")]),
    // --- the `di` family whole; `dell'` read as `dello` (D4) ---
    ("del pane", &[("del", "del"), ("pane", "pane")]),
    ("Del", &[("Del", "Del")]),
    ("dello", &[("dello", "dello")]),
    ("della", &[("della", "della")]),
    ("dei", &[("dei", "dei")]),
    ("degli", &[("degli", "degli")]),
    ("delle", &[("delle", "delle")]),
    ("dell'acqua", &[("dello", "dell'"), ("acqua", "acqua")]),
    ("Dell’Europa", &[("Dello", "Dell’"), ("Europa", "Europa")]),
    ("dell’", &[("dello", "dell’")]),
    // --- the other articulated prepositions split, sharing the span (D4) ---
    ("al", &[("a", "al"), ("il", "al")]),
    ("allo", &[("a", "allo"), ("lo", "allo")]),
    ("alla", &[("a", "alla"), ("la", "alla")]),
    ("ai", &[("a", "ai"), ("i", "ai")]),
    ("agli", &[("a", "agli"), ("gli", "agli")]),
    ("alle", &[("a", "alle"), ("le", "alle")]),
    ("dal", &[("da", "dal"), ("il", "dal")]),
    ("dallo", &[("da", "dallo"), ("lo", "dallo")]),
    ("dalla", &[("da", "dalla"), ("la", "dalla")]),
    ("dai", &[("da", "dai"), ("i", "dai")]),
    ("dagli", &[("da", "dagli"), ("gli", "dagli")]),
    ("dalle", &[("da", "dalle"), ("le", "dalle")]),
    ("nel", &[("in", "nel"), ("il", "nel")]),
    ("nello", &[("in", "nello"), ("lo", "nello")]),
    ("nella", &[("in", "nella"), ("la", "nella")]),
    ("nei", &[("in", "nei"), ("i", "nei")]),
    ("negli", &[("in", "negli"), ("gli", "negli")]),
    ("nelle", &[("in", "nelle"), ("le", "nelle")]),
    ("sul", &[("su", "sul"), ("il", "sul")]),
    ("sullo", &[("su", "sullo"), ("lo", "sullo")]),
    ("sulla", &[("su", "sulla"), ("la", "sulla")]),
    ("sui", &[("su", "sui"), ("i", "sui")]),
    ("sugli", &[("su", "sugli"), ("gli", "sugli")]),
    ("sulle", &[("su", "sulle"), ("le", "sulle")]),
    ("col", &[("con", "col"), ("il", "col")]),
    ("coi", &[("con", "coi"), ("i", "coi")]),
    (
        "all'alba",
        &[("a", "all'"), ("lo", "all'"), ("alba", "alba")],
    ),
    (
        "dall'alto",
        &[("da", "dall'"), ("lo", "dall'"), ("alto", "alto")],
    ),
    (
        "nell’acqua",
        &[("in", "nell’"), ("lo", "nell’"), ("acqua", "acqua")],
    ),
    (
        "sull'isola",
        &[("su", "sull'"), ("lo", "sull'"), ("isola", "isola")],
    ),
    ("all’", &[("a", "all’"), ("lo", "all’")]),
    ("Al", &[("A", "Al"), ("il", "Al")]),
    ("NEL", &[("In", "NEL"), ("il", "NEL")]),
    (
        "Nel mezzo",
        &[("In", "Nel"), ("il", "Nel"), ("mezzo", "mezzo")],
    ),
    (
        "All’alba",
        &[("A", "All’"), ("lo", "All’"), ("alba", "alba")],
    ),
    ("collo", &[("collo", "collo")]),
    ("colla", &[("colla", "colla")]),
    ("cogli", &[("cogli", "cogli")]),
    ("colle", &[("colle", "colle")]),
    ("pel", &[("pel", "pel")]),
    ("pei", &[("pei", "pei")]),
    ("alto", &[("alto", "alto")]),
    // --- truncations, one token spanning the apostrophe (D5) ---
    ("po'", &[("poco", "po'")]),
    ("po’", &[("poco", "po’")]),
    ("mo'", &[("modo", "mo'")]),
    ("fa'", &[("fai", "fa'")]),
    ("Fa'", &[("Fai", "Fa'")]),
    ("va'", &[("vai", "va'")]),
    ("da'", &[("dai", "da'")]),
    ("sta'", &[("stai", "sta'")]),
    ("di'", &[("dici", "di'")]),
    ("be'", &[("bene", "be'")]),
    (
        "un po' di pane, va' via",
        &[
            ("un", "un"),
            ("poco", "po'"),
            ("di", "di"),
            ("pane", "pane"),
            ("vai", "va'"),
            ("via", "via"),
        ],
    ),
    (
        "c'è un po'",
        &[("ci", "c'"), ("è", "è"), ("un", "un"), ("poco", "po'")],
    ),
    // --- an apostrophe for an accent where the pack lists the accented word, `è` here
    // (read-lingua-italian-unmarked-spellings D1; `ACCENT_CASES` below); outside the word where
    // it lists none (`perché`, `città`); a closing quote outside the word (D5) ---
    ("perche'", &[("perche", "perche")]),
    ("e'", &[("è", "e'")]),
    ("E' tardi", &[("È", "E'"), ("tardi", "tardi")]),
    ("citta'", &[("citta", "citta")]),
    ("‘ciao’", &[("ciao", "ciao")]),
    (
        "‘due anni fa’",
        &[("due", "due"), ("anni", "anni"), ("fa", "fa")],
    ),
    (
        "'due anni fa'",
        &[("due", "due"), ("anni", "anni"), ("fa", "fa")],
    ),
    (
        "‘ciao’ un po'",
        &[("ciao", "ciao"), ("un", "un"), ("poco", "po'")],
    ),
    (
        "'un po' e basta'",
        &[("un", "un"), ("po", "po"), ("e", "e"), ("basta", "basta")],
    ),
    // A quote closed after punctuation is closed; `‘` before a digit opens none.
    (
        "‘Vieni!’ disse, un po'",
        &[
            ("Vieni", "Vieni"),
            ("disse", "disse"),
            ("un", "un"),
            ("poco", "po'"),
        ],
    ),
    (
        "'Sì.' Un po'",
        &[("Sì", "Sì"), ("Un", "Un"), ("poco", "po'")],
    ),
    (
        "anni ‘80, un po'",
        &[("anni", "anni"), ("un", "un"), ("poco", "po'")],
    ),
    // --- hyphenated runs: the compound rule, an elision on the first piece split off ---
    (
        "l'anglo-americano",
        &[("lo", "l'"), ("anglo-americano", "anglo-americano")],
    ),
    ("week-end", &[("week-end", "week-end")]),
    ("al-Qaida", &[("al-Qaida", "al-Qaida")]),
    (
        "dell'arco-baleno",
        &[("dello", "dell'"), ("arco-baleno", "arco-baleno")],
    ),
    // --- a digit in a run: each piece an Italian word ---
    ("l'anno-2020", &[("lo", "l'"), ("anno", "anno")]),
    ("nel-10", &[("in", "nel"), ("il", "nel")]),
];

fn lexicon() -> FstLexicon<Vec<u8>> {
    let (bytes, pool) = build_lexicon_blobs(FORMS, LEMMAS).expect("build");
    FstLexicon::from_slices(bytes, &pool).expect("load")
}

/// The forms the accent rule's stand-in pack lists (read-lingua-italian-unmarked-spellings D1):
/// the accented words of the cases, each filed under itself or its verb; `perchè` and `caffé`
/// filed under `perché` and `caffè`, as the dictionary's spellings are (D2); the truncations'
/// accented look-alikes (`pò`, `dà`) and the apocopes' (`né`, `sù`), which the rule never reads.
const ACCENT_FORMS: &[(&str, &str)] = &[
    ("è", "essere"),
    ("perché", "perché"),
    ("perchè", "perché"),
    ("più", "più"),
    ("città", "città"),
    ("può", "potere"),
    ("così", "così"),
    ("caffè", "caffè"),
    ("caffé", "caffè"),
    ("già", "già"),
    ("lunedì", "lunedì"),
    ("però", "però"),
    ("virtù", "virtù"),
    ("pò", "pò"),
    ("dà", "dare"),
    ("né", "né"),
    ("sù", "su"),
    // The misspelling `é`, which the tables rank as a word of its own: `è` is not filed under it.
    ("é", "é"),
    // Two apocopes' accented look-alikes the tables list (task 2.2's measurement).
    ("qué", "qué"),
    ("cà", "cà"),
    // Two words a quoted phrase ends on, listed so that the quotes are what keeps them as written.
    ("lavorò", "lavorare"),
    ("bastà", "bastare"),
];

/// (text tokenised as Italian with [`ACCENT_FORMS`], the tokens as (text, the source text the span
/// covers)): the apostrophe that stands for an accent (D1), each vowel, after an elided piece, on a
/// capital, with two accents listed, beside change 6's listed words, the apocopes and the quotes.
const ACCENT_CASES: &[Case] = &[
    // --- one token spanning the word and its apostrophe, each vowel ---
    ("perche'", &[("perché", "perche'")]),
    ("perche’", &[("perché", "perche’")]),
    ("piu'", &[("più", "piu'")]),
    ("citta'", &[("città", "citta'")]),
    ("e'", &[("è", "e'")]),
    ("puo'", &[("può", "puo'")]),
    ("cosi'", &[("così", "cosi'")]),
    ("caffe'", &[("caffè", "caffe'")]),
    ("gia'", &[("già", "gia'")]),
    ("lunedi'", &[("lunedì", "lunedi'")]),
    ("pero'", &[("però", "pero'")]),
    ("virtu'", &[("virtù", "virtu'")]),
    ("perche' no?", &[("perché", "perche'"), ("no", "no")]),
    // --- after an elided piece ---
    ("c'e'", &[("ci", "c'"), ("è", "e'")]),
    (
        "non c'e' problema",
        &[
            ("non", "non"),
            ("ci", "c'"),
            ("è", "e'"),
            ("problema", "problema"),
        ],
    ),
    ("dov'e'", &[("dove", "dov'"), ("è", "e'")]),
    ("com'e'", &[("come", "com'"), ("è", "e'")]),
    ("l'e'", &[("lo", "l'"), ("è", "e'")]),
    ("cos'e'?", &[("cosa", "cos'"), ("è", "e'")]),
    ("quant’e’", &[("quanto", "quant’"), ("è", "e’")]),
    // --- a capital kept ---
    ("E' vero", &[("È", "E'"), ("vero", "vero")]),
    ("E’ tardi", &[("È", "E’"), ("tardi", "tardi")]),
    ("Perche'", &[("Perché", "Perche'")]),
    ("PIU'", &[("Più", "PIU'")]),
    ("C'E'", &[("Ci", "C'"), ("È", "E'")]),
    // --- two accents listed: the one filed under itself ---
    (
        "perche' caffe'",
        &[("perché", "perche'"), ("caffè", "caffe'")],
    ),
    // --- change 6's listed words first: never `pò`, `dà` ---
    ("po'", &[("poco", "po'")]),
    ("da'", &[("dai", "da'")]),
    ("Da'", &[("Dai", "Da'")]),
    ("di'", &[("dici", "di'")]),
    // --- the apocopes, never: the apostrophe outside the word ---
    ("ne'", &[("ne", "ne")]),
    ("su'", &[("su", "su")]),
    ("de'", &[("de", "de")]),
    ("co'", &[("co", "co")]),
    ("pe'", &[("pe", "pe")]),
    ("a'", &[("a", "a")]),
    ("que' due", &[("que", "que"), ("due", "due")]),
    ("Ca' Foscari", &[("Ca", "Ca"), ("Foscari", "Foscari")]),
    // --- a word the tables do not accent, or no word to accent ---
    ("ciao'", &[("ciao", "ciao")]),
    ("uomo'", &[("uomo", "uomo")]),
    ("perche'2", &[("perche", "perche")]),
    ("perché'", &[("perché", "perché")]),
    // --- the quotes: no accent read inside one ---
    ("‘perche’", &[("perche", "perche")]),
    ("'perche'", &[("perche", "perche")]),
    (
        "‘ciao’ perche' no",
        &[("ciao", "ciao"), ("perché", "perche'"), ("no", "no")],
    ),
    // At review: a truncation's or an accent spelling's apostrophe closes no quote, and a quote
    // mark after a space opens one before dots or a number.
    (
        "Mi serve 'un po' di lavoro'",
        &[
            ("Mi", "Mi"),
            ("serve", "serve"),
            ("un", "un"),
            ("po", "po"),
            ("di", "di"),
            ("lavoro", "lavoro"),
        ],
    ),
    (
        "Disse: 'c'e' lavoro'",
        &[
            ("Disse", "Disse"),
            ("ci", "c'"),
            ("e", "e"),
            ("lavoro", "lavoro"),
        ],
    ),
    (
        "il cosiddetto '...lavoro'",
        &[
            ("il", "il"),
            ("cosiddetto", "cosiddetto"),
            ("lavoro", "lavoro"),
        ],
    ),
    ("'2 lavoro'", &[("lavoro", "lavoro")]),
    (
        "'un po' e basta'",
        &[("un", "un"), ("po", "po"), ("e", "e"), ("basta", "basta")],
    ),
    // Outside a quote the same words read their accent.
    (
        "un po' di lavoro'",
        &[
            ("un", "un"),
            ("poco", "po'"),
            ("di", "di"),
            ("lavorò", "lavoro'"),
        ],
    ),
];

fn accent_lexicon() -> FstLexicon<Vec<u8>> {
    let lemmas = ["a", "e", "o", "essere", "potere", "dare", "su", "due"];
    let (bytes, pool) = build_lexicon_blobs(ACCENT_FORMS, &lemmas).expect("build");
    FstLexicon::from_slices(bytes, &pool).expect("load")
}

/// Each token's text beside the source text its span covers.
fn read<'t>(text: &'t str, tokens: &'t [Token]) -> Vec<(&'t str, &'t str)> {
    tokens
        .iter()
        .map(|t| (t.text.as_str(), &text[t.start..t.end]))
        .collect()
}

#[test]
fn italian_tokenisation_fixtures() {
    assert!(
        CASES.len() >= 80,
        "task 1.6 asks for at least 80 cases, got {}",
        CASES.len()
    );
    let lex = lexicon();
    let wrong: Vec<String> = CASES
        .iter()
        .filter_map(|(text, expected)| {
            let tokens = tokenize(text, IT, &lex);
            let got = read(text, &tokens);
            (got != *expected).then(|| format!("{text:?}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn italian_accent_fixtures() {
    // read-lingua-italian-unmarked-spellings, task 1.2: the apostrophe that stands for an accent.
    let lex = accent_lexicon();
    let wrong: Vec<String> = ACCENT_CASES
        .iter()
        .filter_map(|(text, expected)| {
            let tokens = tokenize(text, IT, &lex);
            let got = read(text, &tokens);
            (got != *expected).then(|| format!("{text:?}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn italian_spans_follow_the_text() {
    // Every span lies inside the text and starts no earlier than the last one; the pieces of one
    // written word never overlap but at a shared span; no token's text holds a typographic
    // apostrophe; no piece is a compound but a hyphenated run's.
    let (lex, accents) = (lexicon(), accent_lexicon());
    let cases = CASES
        .iter()
        .map(|case| (case, &lex))
        .chain(ACCENT_CASES.iter().map(|case| (case, &accents)));
    for ((text, _), lex) in cases {
        let tokens = tokenize(text, IT, lex);
        let mut last = (0, 0);
        for token in &tokens {
            assert!(
                token.start < token.end && token.end <= text.len(),
                "{text:?}"
            );
            assert!(
                (token.start, token.end) == last || token.start >= last.1,
                "{text:?}: {token:?} overlaps {last:?}"
            );
            assert!(!token.text.contains('\u{2019}'), "{text:?}");
            assert_eq!(
                token.parts.is_empty(),
                !token.text.contains('-'),
                "{text:?}"
            );
            last = (token.start, token.end);
        }
    }
}

#[test]
fn english_spanish_and_french_read_the_fixtures_as_before() {
    // Italian's pre-pass is Italian's arm: no elision, articulated preposition or truncation of it
    // runs on English, Spanish or French text.
    let lex = lexicon();
    let texts = |text: &str, language: StudiedLanguage| -> Vec<String> {
        tokenize(text, language, &lex)
            .into_iter()
            .map(|t| t.text)
            .collect()
    };
    for language in [StudiedLanguage::English, StudiedLanguage::Spanish] {
        assert_eq!(texts("l'uomo", language), ["l'uomo"]);
        assert_eq!(texts("nel col", language), ["nel", "col"]);
        assert_eq!(texts("po'", language), ["po"]);
        assert_eq!(texts("dell'acqua", language), ["dell'acqua"]);
    }
    let french = StudiedLanguage::French;
    assert_eq!(texts("l'uomo", french), ["le", "uomo"]);
    assert_eq!(texts("nel col po'", french), ["nel", "col", "po"]);
    assert_eq!(texts("dell'acqua", french), ["dell'acqua"]);
}

/// An Italian pack holding the stand-in lexicon, at Italian's analyser version.
fn pack() -> Pack {
    italian_pack(FORMS, LEMMAS, &[])
}

/// An Italian pack of `forms` and `lemmas`, its dictionary words `dictionary` in a lexical
/// section (none without one), at Italian's analyser version.
fn italian_pack(forms: &[(&str, &str)], lemmas: &[&str], dictionary: &[&str]) -> Pack {
    let (forms, pool) = build_lexicon_blobs(forms, lemmas).expect("lexicon");
    let lex = FstLexicon::from_slices(forms.clone(), &pool).expect("load");
    let lemma_count = lex.lemma_count();
    let freq = vec![0u8; lemma_count * 4];
    let mut lexical = vec![0u8; lemma_count.div_ceil(8)];
    for word in dictionary {
        let id = lex.id_of(word).expect("a dictionary word is a lemma") as usize;
        lexical[id / 8] |= 1 << (id % 8);
    }
    let meta = serde_json::to_vec(&PackMeta {
        studied: "it".into(),
        native: "fr".into(),
        pack_version: "fixture".into(),
        analyzer_version: ITALIAN_ANALYZER_VERSION.into(),
        licences: vec![],
        levels_estimated: false,
    })
    .expect("meta");
    let mut sections: Vec<(&str, &[u8])> = vec![
        (section::FORMS, &forms),
        (section::LEMMAS, pool.as_bytes()),
        (section::FREQ, &freq),
    ];
    if !dictionary.is_empty() {
        sections.push((section::LEXICAL, &lexical));
    }
    Pack::load(&write_container(&meta, &sections)).expect("an Italian pack loads")
}

#[test]
fn a_word_card_reads_the_written_word_through_the_pre_pass() {
    let pack = pack();
    let pieces = |written: &str, lemma: &str| word_grammar(written, lemma, IT, &pack).pieces;
    // An elided word on its own is one token, `lo`, spanning its apostrophe: no pieces.
    assert_eq!(
        read("l’", &tokenize("l’", IT, pack.lexicon())),
        [("lo", "l’")]
    );
    assert!(pieces("l’", "lo").is_empty());
    assert!(pieces("dell’", "dello").is_empty());
    // A contraction answers its two pieces, an elided one too: the span « all’ » the reader
    // clicked holds both.
    assert_eq!(pieces("al", "a"), ["a", "il"]);
    assert_eq!(pieces("all’", "a"), ["a", "lo"]);
    // An elision's pieces, each a word.
    assert_eq!(pieces("L'uomo", "uomo"), ["Lo", "uomo"]);
    // A truncation is one word; a word the pre-pass leaves whole has none.
    assert!(pieces("po'", "poco").is_empty());
    assert!(pieces("del", "del").is_empty());
    assert!(pieces("dirmelo", "dirmelo").is_empty());
}

// --- add-lingua-italian-analysis (task 6.1) ---

/// The analysis's stand-in pack: the forms its cases need, filed as the forms tables file them
/// (`sto` under `stare`, `parlando` and `parla` under `parlare`, `porte` under `porta`), and
/// `sviluppata` under the adjective `sviluppato`.
const ANALYSIS_FORMS: &[(&str, &str)] = &[
    ("è", "essere"),
    ("È", "essere"),
    ("sarà", "essere"),
    ("ho", "avere"),
    ("so", "sapere"),
    ("sto", "stare"),
    ("voglio", "volere"),
    ("visto", "vedere"),
    ("parlando", "parlare"),
    ("parla", "parlare"),
    ("scrivi", "scrivere"),
    ("prendete", "prendere"),
    ("andiamo", "andare"),
    ("dicendo", "dire"),
    ("facendo", "fare"),
    ("guarda", "guardare"),
    ("porte", "porta"),
    ("vite", "vita"),
    ("sviluppata", "sviluppato"),
    ("sa", "sapere"),
    ("fa", "fare"),
];

/// The analysis's lemmas, each its own form.
const ANALYSIS_LEMMAS: &[&str] = &[
    "a",
    "e",
    "o",
    "i",
    "essere",
    "avere",
    "sapere",
    "stare",
    "volere",
    "vedere",
    "parlare",
    "scrivere",
    "prendere",
    "andare",
    "dire",
    "fare",
    "dare",
    "guardare",
    "porre",
    "condurre",
    "portare",
    "sviluppare",
    "sviluppato",
    "formare",
    "ecco",
    "città",
    "età",
    "università",
    "più",
    "perché",
    "pietà",
    "tardi",
    "poco",
    "di",
    "da",
    "in",
    "con",
    "il",
    "lo",
    "la",
    "gli",
    "le",
    "ci",
    "ne",
    "mi",
    "ti",
    "si",
    "vi",
    "non",
    "né",
    "mai",
    "oggi",
    "cane",
    "marco",
    "secondo",
    "giorno",
    "molto",
    "tempo",
    "mare",
    "uomo",
    "cinema",
    "sistema",
    "tema",
    "vela",
    "mela",
    "porta",
    "vita",
    "pane",
    "del",
    "prima",
    "tutto",
    "che",
    "subito",
    "risultare",
    "vestire",
];

/// (text glossed as Italian, each token as (surface, lemma, function word)).
type AnalysisCase = (&'static str, &'static [(&'static str, &'static str, bool)]);

const ANALYSIS_CASES: &[AnalysisCase] = &[
    // --- NFC in the pre-pass: a decomposed word is the pack's composed one (D1) ---
    ("citta\u{300}", &[("città", "città", false)]),
    ("Citta\u{300}", &[("Città", "città", false)]),
    ("piu\u{300}", &[("più", "più", false)]),
    ("perche\u{301}", &[("perché", "perché", true)]),
    ("sara\u{300}", &[("sarà", "essere", true)]),
    (
        "E\u{300} tardi",
        &[("È", "essere", true), ("tardi", "tardi", false)],
    ),
    ("l'eta\u{300}", &[("lo", "lo", true), ("età", "età", false)]),
    (
        "dall'eta\u{300}",
        &[
            ("da", "da", true),
            ("lo", "lo", true),
            ("età", "età", false),
        ],
    ),
    ("c'e\u{300}", &[("ci", "ci", true), ("è", "essere", true)]),
    (
        "nell'universita\u{300}",
        &[
            ("in", "in", true),
            ("lo", "lo", true),
            ("università", "università", false),
        ],
    ),
    ("attivita\u{300}", &[("attività", "attività", false)]),
    (
        "un po' di pieta\u{300}",
        &[
            ("un", "un", true),
            ("poco", "poco", true),
            ("di", "di", true),
            ("pietà", "pietà", false),
        ],
    ),
    // --- the enclitic rule: an infinitive, its ending restored (D2) ---
    ("parlarmi", &[("parlarmi", "parlare", false)]),
    ("dirlo", &[("dirlo", "dire", false)]),
    ("porlo", &[("porlo", "porre", false)]),
    ("condurlo", &[("condurlo", "condurre", false)]),
    ("vederci", &[("vederci", "vedere", false)]),
    ("esserci", &[("esserci", "essere", true)]),
    ("averne", &[("averne", "avere", true)]),
    // --- a combined clitic, longest first ---
    ("dirmelo", &[("dirmelo", "dire", false)]),
    ("dargliela", &[("dargliela", "dare", false)]),
    ("andarsene", &[("andarsene", "andare", false)]),
    ("vedermene", &[("vedermene", "vedere", false)]),
    ("darcela", &[("darcela", "dare", false)]),
    ("dirvelo", &[("dirvelo", "dire", false)]),
    ("portarglielo", &[("portarglielo", "portare", false)]),
    // --- a gerund, an imperative: a form the pack files under a verb ---
    ("parlandogli", &[("parlandogli", "parlare", false)]),
    ("dicendolo", &[("dicendolo", "dire", false)]),
    ("facendone", &[("facendone", "fare", false)]),
    ("scrivimi", &[("scrivimi", "scrivere", false)]),
    ("prendetelo", &[("prendetelo", "prendere", false)]),
    ("andiamocene", &[("andiamocene", "andare", false)]),
    ("parlagliene", &[("parlagliene", "parlare", false)]),
    ("Guardala", &[("Guardala", "guardare", false)]),
    // --- the five monosyllabic imperatives, the doubling undone, `gli` without it ---
    ("dammi", &[("dammi", "dare", false)]),
    ("fallo", &[("fallo", "fare", false)]),
    ("vattene", &[("vattene", "andare", false)]),
    ("dimmi", &[("dimmi", "dire", false)]),
    ("stacci", &[("stacci", "stare", false)]),
    ("dammelo", &[("dammelo", "dare", false)]),
    ("Dimmelo", &[("Dimmelo", "dire", false)]),
    ("digli", &[("digli", "dire", false)]),
    ("fagli", &[("fagli", "fare", false)]),
    ("diglielo", &[("diglielo", "dire", false)]),
    // --- `ecco` ---
    ("eccolo", &[("eccolo", "ecco", false)]),
    ("eccomi", &[("eccomi", "ecco", false)]),
    ("eccotelo", &[("eccotelo", "ecco", false)]),
    // --- a regular past participle, kept by D6's measurement ---
    ("sviluppatasi", &[("sviluppatasi", "sviluppare", false)]),
    ("formatosi", &[("formatosi", "formare", false)]),
    // --- a listed word is never split; a base the pack does not list reads nothing ---
    ("cinema", &[("cinema", "cinema", false)]),
    ("sistema", &[("sistema", "sistema", false)]),
    ("tema", &[("tema", "tema", false)]),
    ("vela", &[("vela", "vela", false)]),
    ("mela", &[("mela", "mela", false)]),
    ("zorbarmi", &[("zorbarmi", "zorbarmi", false)]),
    // --- a present indicative takes no clitic; an undoubled monosyllable takes only `gli` ---
    ("stone", &[("stone", "stone", false)]),
    ("sami", &[("sami", "sami", false)]),
    ("falo", &[("falo", "falo", false)]),
    ("dami", &[("dami", "dami", false)]),
    // --- an augmentative is no participle before `ne` ---
    ("risultatone", &[("risultatone", "risultatone", false)]),
    ("vestitone", &[("vestitone", "vestitone", false)]),
    ("zorbarlo", &[("zorbarlo", "zorbarlo", false)]),
    // --- an unlisted plural stays as written: D2's step 3 was measured out (D6) ---
    ("tavoli", &[("tavoli", "tavoli", false)]),
    ("amiche", &[("amiche", "amiche", false)]),
    ("botteghe", &[("botteghe", "botteghe", false)]),
    ("fuochi", &[("fuochi", "fuochi", false)]),
    ("vite", &[("vite", "vita", false)]),
    ("porte", &[("porte", "porta", false)]),
    // --- the closed classes in a phrase (D3) ---
    (
        "Non ho visto il cane di Marco",
        &[
            ("Non", "non", true),
            ("ho", "avere", true),
            ("visto", "vedere", false),
            ("il", "il", true),
            ("cane", "cane", false),
            ("di", "di", true),
            ("Marco", "marco", false),
        ],
    ),
    (
        "non lo so",
        &[
            ("non", "non", true),
            ("lo", "lo", true),
            ("so", "sapere", false),
        ],
    ),
    (
        "né oggi né mai",
        &[
            ("né", "né", true),
            ("oggi", "oggi", false),
            ("né", "né", true),
            ("mai", "mai", false),
        ],
    ),
    ("più", &[("più", "più", false)]),
    (
        "sto parlando",
        &[("sto", "stare", false), ("parlando", "parlare", false)],
    ),
    ("voglio", &[("voglio", "volere", false)]),
    (
        "il secondo giorno",
        &[
            ("il", "il", true),
            ("secondo", "secondo", false),
            ("giorno", "giorno", false),
        ],
    ),
    (
        "molto tempo",
        &[("molto", "molto", true), ("tempo", "tempo", false)],
    ),
    (
        "al mare",
        &[
            ("a", "a", true),
            ("il", "il", true),
            ("mare", "mare", false),
        ],
    ),
    ("l'uomo", &[("lo", "lo", true), ("uomo", "uomo", false)]),
    (
        "prima di tutto",
        &[
            ("prima", "prima", false),
            ("di", "di", true),
            ("tutto", "tutto", true),
        ],
    ),
    (
        "del pane, subito",
        &[
            ("del", "del", true),
            ("pane", "pane", false),
            ("subito", "subito", false),
        ],
    ),
    (
        "fino a dove, nonostante tutto",
        &[
            ("fino", "fino", true),
            ("a", "a", true),
            ("dove", "dove", true),
            ("nonostante", "nonostante", true),
            ("tutto", "tutto", true),
        ],
    ),
    (
        "salvo qualunque cosa",
        &[
            ("salvo", "salvo", false),
            ("qualunque", "qualunque", true),
            ("cosa", "cosa", false),
        ],
    ),
    (
        "che ci sia con gli",
        &[
            ("che", "che", true),
            ("ci", "ci", true),
            ("sia", "sia", false),
            ("con", "con", true),
            ("gli", "gli", true),
        ],
    ),
];

#[test]
fn italian_analysis_fixtures() {
    let pack = italian_pack(ANALYSIS_FORMS, ANALYSIS_LEMMAS, &[]);
    let knowledge = KnowledgeState::new();
    let wrong: Vec<String> = ANALYSIS_CASES
        .iter()
        .filter_map(|(text, expected)| {
            let glossed = gloss_phrase(text, IT, &pack, &knowledge);
            let got: Vec<(&str, &str, bool)> = glossed
                .tokens
                .iter()
                .map(|t| (t.surface.as_str(), t.lemma.as_str(), t.function_word))
                .collect();
            (got != *expected).then(|| format!("{text:?}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The names rule's stand-in pack (D4): the names its cases write as lemmas that are no
/// dictionary words, `anna`, `maria`, `marco` and `sale` dictionary words, and the function
/// words of its blocks.
const NAMES_LEMMAS: &[&str] = &[
    "roma", "italia", "europa", "olanda", "firenze", "ecco", "anna", "maria", "marco", "sale",
    "il", "lo", "la", "di", "a", "e", "in", "con", "essere",
];

/// The names pack's dictionary words.
const NAMES_DICTIONARY: &[&str] = &[
    "anna", "maria", "marco", "sale", "il", "lo", "la", "di", "a", "e", "in", "con", "essere",
];

/// (the document's blocks, a surface, the class of each of its occurrences, in order).
type NameCase = (&'static [&'static str], &'static str, &'static [TokenClass]);

const NAME: TokenClass = TokenClass::ProperNounOutOfLexicon;
const WORD: TokenClass = TokenClass::Unknown;

const NAME_CASES: &[NameCase] = &[
    // --- Spanish's rule: a capital in mid-sentence, never in lowercase, no dictionary word ---
    (
        &["Sono andato a Roma con i miei amici per vedere i monumenti antichi della città."],
        "Roma",
        &[NAME],
    ),
    (
        &[
            "Sono andato a Roma con i miei amici per vedere i monumenti antichi della città.",
            "Roma è grande e piena di turisti in ogni stagione dell'anno, anche d'inverno.",
        ],
        "Roma",
        &[NAME, NAME],
    ),
    // --- no evidence: the head of a block, of a sentence ---
    (
        &[
            "Roma è una grande città piena di monumenti, di chiese e di turisti.",
            "Roma attira ogni anno milioni di visitatori da tutto il mondo.",
        ],
        "Roma",
        &[WORD, WORD],
    ),
    (
        &[
            "Ecco. Ecco la casa di mio nonno, con il giardino e la vecchia porta di legno.",
            "Ecco il paese dove sono nato e dove ho passato tutta la mia infanzia.",
        ],
        "Ecco",
        &[WORD, WORD, WORD],
    ),
    // --- a capital right after an elided piece, straight or typographic, even at a head ---
    (
        &["L'Italia è lunga e stretta, con il mare su tre lati e le montagne a nord."],
        "Italia",
        &[NAME],
    ),
    (
        &["Il paese è nel cuore dell'Europa da molti secoli, e la sua storia è antica."],
        "Europa",
        &[NAME],
    ),
    (
        &["Tutti parlano dell\u{2019}Europa con il cuore e con la testa, da molti anni ormai."],
        "Europa",
        &[NAME],
    ),
    (
        &["Questa pianura è come un'Olanda in miniatura, piena di canali e di biciclette."],
        "Olanda",
        &[NAME],
    ),
    // --- a hyphenated run the pack does not list is one form ---
    (
        &["Ieri sera sono uscito con Anna-Maria e con la sua famiglia per una cena in centro."],
        "Anna-Maria",
        &[NAME],
    ),
    (
        &["Anna-Maria è una ragazza molto simpatica, e tutti i suoi amici le vogliono bene."],
        "Anna-Maria",
        &[WORD],
    ),
    // --- a dictionary word stays a word ---
    (
        &["Quando sono arrivato a Firenze, Marco legge il giornale seduto in piazza."],
        "Firenze",
        &[NAME],
    ),
    (
        &["Quando sono arrivato a Firenze, Marco legge il giornale seduto in piazza."],
        "Marco",
        &[WORD],
    ),
    // --- a form the document also writes in lowercase ---
    (
        &["Il fiume si chiama il Sale e il sale si raccoglie sulle sue rive da sempre."],
        "Sale",
        &[WORD],
    ),
];

/// The classes of every token written `surface` on an Italian page.
fn classes(pack: &Pack, blocks: &[&str], surface: &str) -> Vec<TokenClass> {
    let page = analyse_page(blocks, IT, pack, &KnowledgeState::new());
    assert!(page.analysable, "{blocks:?}");
    assert_eq!(page.analyzer_version, ITALIAN_ANALYZER_VERSION);
    page.tokens
        .into_iter()
        .filter(|token| token.surface == surface)
        .map(|token| token.class)
        .collect()
}

#[test]
fn italian_names_fixtures() {
    let pack = italian_pack(&[("è", "essere")], NAMES_LEMMAS, NAMES_DICTIONARY);
    let wrong: Vec<String> = NAME_CASES
        .iter()
        .filter_map(|(blocks, surface, expected)| {
            let got = classes(&pack, blocks, surface);
            (got != *expected)
                .then(|| format!("{surface:?} in {blocks:?}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_name_ending_like_a_clitic_stays_a_name() {
    // A present indicative takes no clitic: « Stone » is no `sto` + `ne` read as `stare`, nor
    // « Sami » `sa` + `mi` read as `sapere`, though the pack files `sto` and `sa` under their
    // verbs; each keeps its own lemma, outside the lexicon, and is set aside as a name.
    let pack = italian_pack(
        &[("è", "essere"), ("sto", "stare"), ("sa", "sapere")],
        &[
            "stare", "sapere", "il", "lo", "la", "di", "a", "e", "in", "con", "essere",
        ],
        &[],
    );
    let blocks = [
        "Ieri sera ho incontrato Sami e Stone al mercato, vicino alla stazione del paese.",
        "Il mercato era pieno di gente e di colori, e la musica suonava in piazza.",
    ];
    let page = analyse_page(&blocks, IT, &pack, &KnowledgeState::new());
    assert!(page.analysable);
    for (surface, lemma) in [("Sami", "sami"), ("Stone", "stone")] {
        let token = page
            .tokens
            .iter()
            .find(|t| t.surface == surface)
            .unwrap_or_else(|| panic!("{surface}"));
        assert_eq!(
            (token.lemma.as_str(), token.class),
            (lemma, NAME),
            "{surface}"
        );
    }
}

// --- keep-lingua-italian-acronyms-and-names (task 1.2) ---

/// The acronym rule's stand-in pack: `NATO` and `USA` its own lemmas, as `tables/it/forms.tsv`
/// writes the reviewed table's rows, beside `nato` read as *nascere* and `usa` as *usare*.
const ACRONYM_FORMS: &[(&str, &str)] = &[
    ("NATO", "NATO"),
    ("USA", "USA"),
    ("nato", "nascere"),
    ("usa", "usare"),
    ("paesi", "paese"),
    ("hanno", "avere"),
    ("ha", "avere"),
    ("votato", "votare"),
    ("sono", "essere"),
    ("è", "essere"),
    ("miei", "mio"),
    ("amici", "amico"),
    ("torna", "tornare"),
    ("lo", "il"),
    ("la", "il"),
    ("gli", "il"),
    ("i", "il"),
];

const ACRONYM_LEMMAS: &[&str] = &[
    "il", "di", "della", "degli", "dello", "in", "a", "e", "da", "paese", "avere", "votare",
    "nascere", "usare", "essere", "mio", "amico", "roma", "onu", "rai", "tornare",
];

/// (text glossed as Italian, each token as (surface, dictionary form)).
type AcronymCase = (&'static str, &'static [(&'static str, &'static str)]);

const ACRONYM_CASES: &[AcronymCase] = &[
    // --- in a line of lowercase words: the acronym, as written ---
    (
        "I paesi della NATO hanno votato",
        &[
            ("I", "il"),
            ("paesi", "paese"),
            ("della", "della"),
            ("NATO", "NATO"),
            ("hanno", "avere"),
            ("votato", "votare"),
        ],
    ),
    ("la NATO", &[("la", "il"), ("NATO", "NATO")]),
    ("NATO", &[("NATO", "NATO")]),
    ("« NATO », dico", &[("NATO", "NATO"), ("dico", "dico")]),
    // --- a headline in capitals: the words around it are in capitals, the cascade reads it ---
    (
        "LA NATO HA VOTATO",
        &[
            ("LA", "il"),
            ("NATO", "nascere"),
            ("HA", "avere"),
            ("VOTATO", "votare"),
        ],
    ),
    (
        "NATO E USA",
        &[("NATO", "nascere"), ("E", "e"), ("USA", "usare")],
    ),
    // --- the participle in lowercase, or with a capital initial ---
    (
        "sono nato a Roma",
        &[
            ("sono", "essere"),
            ("nato", "nascere"),
            ("a", "a"),
            ("Roma", "roma"),
        ],
    ),
    (
        "Nato a Roma",
        &[("Nato", "nascere"), ("a", "a"), ("Roma", "roma")],
    ),
    ("NaTO", &[("NaTO", "nascere")]),
    // --- an articulated preposition beside the acronym, judged by its written word ---
    ("negli USA", &[("in", "in"), ("gli", "il"), ("USA", "USA")]),
    (
        "NEGLI USA",
        &[("In", "in"), ("gli", "il"), ("USA", "usare")],
    ),
    ("Negli USA", &[("In", "in"), ("gli", "il"), ("USA", "USA")]),
    ("degli USA", &[("degli", "degli"), ("USA", "USA")]),
    ("DEGLI USA", &[("DEGLI", "degli"), ("USA", "usare")]),
    ("dagli USA", &[("da", "da"), ("gli", "il"), ("USA", "USA")]),
    (
        "DAGLI USA",
        &[("Da", "da"), ("gli", "il"), ("USA", "usare")],
    ),
    // --- an elided piece: `l'` a single letter passed over, `dell'` a word ---
    ("l'USA", &[("lo", "il"), ("USA", "USA")]),
    ("dell'USA", &[("dello", "dello"), ("USA", "USA")]),
    ("DELL'USA", &[("Dello", "dello"), ("USA", "usare")]),
    // --- a contraction in capitals: single letters sharing one span, never a row ---
    (
        "AI MIEI AMICI",
        &[("A", "a"), ("i", "il"), ("MIEI", "mio"), ("AMICI", "amico")],
    ),
    (
        "Ai miei amici",
        &[("A", "a"), ("i", "il"), ("miei", "mio"), ("amici", "amico")],
    ),
    ("torna AI", &[("torna", "tornare"), ("A", "a"), ("i", "il")]),
    // --- an acronym the pack lists no row for: the cascade, as before ---
    (
        "l'ONU e la RAI",
        &[
            ("lo", "il"),
            ("ONU", "onu"),
            ("e", "e"),
            ("la", "il"),
            ("RAI", "rai"),
        ],
    ),
];

#[test]
fn italian_acronym_fixtures() {
    let pack = italian_pack(ACRONYM_FORMS, ACRONYM_LEMMAS, &[]);
    let knowledge = KnowledgeState::new();
    let wrong: Vec<String> = ACRONYM_CASES
        .iter()
        .filter_map(|(text, expected)| {
            let glossed = gloss_phrase(text, IT, &pack, &knowledge);
            let got: Vec<(&str, &str)> = glossed
                .tokens
                .iter()
                .map(|t| (t.surface.as_str(), t.lemma.as_str()))
                .collect();
            (got != *expected).then(|| format!("{text:?}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_word_card_keeps_the_italian_acronym_as_asked() {
    // The card's dictionary form is kept as asked when the pack holds it as a lemma (`NATO`), and
    // lowercased otherwise, as before.
    let pack = italian_pack(ACRONYM_FORMS, ACRONYM_LEMMAS, &[]);
    assert!(pack.lexicon().contains_lemma("NATO"));
    assert_eq!(pack.lexicon().lemma_of("NATO"), Some("NATO"));
    assert_eq!(pack.lexicon().lemma_of("nato"), Some("nascere"));
    assert!(word_grammar("NATO", "NATO", IT, &pack).pieces.is_empty());
    assert_eq!(
        word_grammar("NEGLI USA", "usare", IT, &pack).pieces,
        ["In", "gli", "USA"]
    );
}

#[test]
fn at_least_seventy_cases_cover_italian_s_own_analysis() {
    // add-lingua-italian-analysis task 6.1.
    let cases = ANALYSIS_CASES.len() + NAME_CASES.len();
    assert!(
        cases >= 70,
        "task 6.1 asks for at least 70 cases, got {cases}"
    );
}

#[test]
fn english_spanish_and_french_keep_their_function_words() {
    // Italian's tables are Italian's: `il`, `di`, `non` flag nothing in English or Spanish.
    for language in [StudiedLanguage::English, StudiedLanguage::Spanish] {
        for lemma in ["il", "di", "non", "gli", "né"] {
            assert!(!is_function_word(lemma, language), "{lemma} {language:?}");
        }
    }
    assert!(is_function_word("the", StudiedLanguage::English));
    assert!(is_function_word("de", StudiedLanguage::Spanish));
    assert!(is_function_word("pas", StudiedLanguage::French));
    assert!(!is_function_word("the", IT) && !is_function_word("de", IT));
}
