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

//! Non-regression fixtures for Italian's tokenisation pre-pass
//! (add-lingua-italian-tokenisation, task 1.6 — at least 80 cases), grouped by
//! rule: each elided form, the rule again on what follows, the words that stay
//! whole, the `di` family, the other articulated prepositions, the truncations,
//! the apostrophe for an accent and the quotes, the hyphenated runs. Every case
//! states each token beside the source text its span covers, so every split is
//! checked with its spans. Any change to a fixture's expectation is a behavioural
//! change of the Italian analyser and demands an `ITALIAN_ANALYZER_VERSION` bump;
//! English's, Spanish's and French's fixtures and baselines do not move.
//!
//! The lexicon is a small stand-in for the forms tables
//! (add-lingua-italian-forms-tables): it lists the single-letter words the
//! tokeniser drops when unlisted — `è`, `a`, `e`, `o`.

use lingua_core::analysis::ITALIAN_ANALYZER_VERSION;
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::lexicon::{FstLexicon, build_lexicon_blobs};
use lingua_core::analysis::tokenize::{Token, tokenize};
use lingua_core::engine::word_grammar;
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
    // --- an apostrophe for an accent, a closing quote: outside the word, as today (D5) ---
    ("perche'", &[("perche", "perche")]),
    ("e'", &[("e", "e")]),
    ("E' tardi", &[("E", "E"), ("tardi", "tardi")]),
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
fn italian_spans_follow_the_text() {
    // Every span lies inside the text and starts no earlier than the last one; the pieces of one
    // written word never overlap but at a shared span; no token's text holds a typographic
    // apostrophe; no piece is a compound but a hyphenated run's.
    let lex = lexicon();
    for (text, _) in CASES {
        let tokens = tokenize(text, IT, &lex);
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
    let (forms, pool) = build_lexicon_blobs(FORMS, LEMMAS).expect("lexicon");
    let lemma_count = FstLexicon::from_slices(forms.clone(), &pool)
        .expect("load")
        .lemma_count();
    let freq = vec![0u8; lemma_count * 4];
    let meta = serde_json::to_vec(&PackMeta {
        studied: "it".into(),
        native: "fr".into(),
        pack_version: "fixture".into(),
        analyzer_version: ITALIAN_ANALYZER_VERSION.into(),
        licences: vec![],
        levels_estimated: false,
    })
    .expect("meta");
    let sections: [(&str, &[u8]); 3] = [
        (section::FORMS, &forms),
        (section::LEMMAS, pool.as_bytes()),
        (section::FREQ, &freq),
    ];
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
