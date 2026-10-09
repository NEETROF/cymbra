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

//! Non-regression fixtures for French's tokenisation pre-pass
//! (add-lingua-french-tokenisation, task 1.7 — at least 80 cases), grouped by
//! rule: the narrow no-break space, each elided form, the words that stay whole,
//! `au`/`aux` and `du`/`des`, the hyphenated inversions and the runs that are
//! none. Every case states each token beside the source text its span covers, so
//! every split is checked with its spans. Any change to a fixture's expectation
//! is a behavioural change of the French analyser and demands a
//! `FRENCH_ANALYZER_VERSION` bump; English's and Spanish's fixtures and
//! baselines do not move.
//!
//! The lexicon is a small stand-in for the forms tables
//! (add-lingua-french-forms-tables), which measure these rules on real text.

use lingua_core::analysis::FRENCH_ANALYZER_VERSION;
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::lexicon::{FstLexicon, build_lexicon_blobs};
use lingua_core::analysis::tokenize::{Token, tokenize};
use lingua_core::engine::word_grammar;
use lingua_core::packs::pack::section;
use lingua_core::packs::{Pack, PackMeta, write_container};

const FR: StudiedLanguage = StudiedLanguage::French;

/// The forms the stand-in pack lists, with their lemmas: the one-letter words the
/// single-letter rule keeps, and the hyphenated runs it lists whole.
const FORMS: &[(&str, &str)] = &[
    ("a", "avoir"),
    ("dit", "dire"),
    ("rendez-vous", "rendez-vous"),
    ("peut-être", "peut-être"),
    ("arc-en-ciel", "arc-en-ciel"),
    ("c'est-à-dire", "c'est-à-dire"),
    ("porte-monnaie", "porte-monnaie"),
];

/// Lemmas the stand-in pack lists as their own forms.
const LEMMAS: &[&str] = &[
    "à", "y", "le", "de", "que", "homme", "avoir", "dire", "être", "ce",
];

/// A narrow no-break space, as French sets it.
const NNBSP: &str = "\u{202F}";

/// (text tokenised as French, the tokens as (text, the source text the span covers)).
type Case = (&'static str, &'static [(&'static str, &'static str)]);

const CASES: &[Case] = &[
    // --- U+202F, the narrow no-break space: a space (D2) ---
    (
        "«\u{202F}C’est fini\u{202F}!\u{202F}»",
        &[("Ce", "C’"), ("est", "est"), ("fini", "fini")],
    ),
    ("pas\u{202F}?", &[("pas", "pas")]),
    ("«\u{202F}Je", &[("Je", "Je")]),
    ("lumière\u{202F};", &[("lumière", "lumière")]),
    ("tous\u{202F}!", &[("tous", "tous")]),
    ("pas\u{202F}encore", &[("pas", "pas"), ("encore", "encore")]),
    (
        "«\u{202F}Qu’est-ce\u{202F}?",
        &[("Que", "Qu’"), ("est", "est"), ("ce", "ce")],
    ),
    ("Note\u{A0}: fini", &[("Note", "Note"), ("fini", "fini")]),
    // --- each elided form, read as the word it stands for (D3) ---
    ("c'est", &[("ce", "c'"), ("est", "est")]),
    ("C’était", &[("Ce", "C’"), ("était", "était")]),
    ("ç'a", &[("ça", "ç'"), ("a", "a")]),
    ("Ç’aurait", &[("Ça", "Ç’"), ("aurait", "aurait")]),
    ("d'abord", &[("de", "d'"), ("abord", "abord")]),
    ("D’abord", &[("De", "D’"), ("abord", "abord")]),
    ("j'ai", &[("je", "j'"), ("ai", "ai")]),
    ("J’entends", &[("Je", "J’"), ("entends", "entends")]),
    ("l'homme", &[("le", "l'"), ("homme", "homme")]),
    ("L'homme", &[("Le", "L'"), ("homme", "homme")]),
    ("l’aube", &[("le", "l’"), ("aube", "aube")]),
    ("m'appelle", &[("me", "m'"), ("appelle", "appelle")]),
    ("M’entendez", &[("Me", "M’"), ("entendez", "entendez")]),
    ("n'est", &[("ne", "n'"), ("est", "est")]),
    ("N’importe", &[("Ne", "N’"), ("importe", "importe")]),
    ("qu'il", &[("que", "qu'"), ("il", "il")]),
    ("Qu’on", &[("Que", "Qu’"), ("on", "on")]),
    ("s'en", &[("se", "s'"), ("en", "en")]),
    ("s'il", &[("si", "s'"), ("il", "il")]),
    ("S’ils", &[("Si", "S’"), ("ils", "ils")]),
    ("s'illumine", &[("se", "s'"), ("illumine", "illumine")]),
    ("t'aime", &[("te", "t'"), ("aime", "aime")]),
    ("T’es", &[("Te", "T’"), ("es", "es")]),
    ("jusqu'ici", &[("jusque", "jusqu'"), ("ici", "ici")]),
    ("Jusqu’alors", &[("Jusque", "Jusqu’"), ("alors", "alors")]),
    ("lorsqu'il", &[("lorsque", "lorsqu'"), ("il", "il")]),
    ("Lorsqu’elle", &[("Lorsque", "Lorsqu’"), ("elle", "elle")]),
    ("puisqu'on", &[("puisque", "puisqu'"), ("on", "on")]),
    ("Puisqu’elle", &[("Puisque", "Puisqu’"), ("elle", "elle")]),
    ("quoiqu'il", &[("quoique", "quoiqu'"), ("il", "il")]),
    ("Quoiqu’on", &[("Quoique", "Quoiqu’"), ("on", "on")]),
    ("QU'IL", &[("Que", "QU'"), ("IL", "IL")]),
    // --- the rule again on what follows; an elided word on its own (D3) ---
    (
        "jusqu'au",
        &[("jusque", "jusqu'"), ("à", "au"), ("le", "au")],
    ),
    (
        "qu'aujourd'hui",
        &[("que", "qu'"), ("aujourd'hui", "aujourd'hui")],
    ),
    ("l’", &[("le", "l’")]),
    ("l’ homme", &[("le", "l’"), ("homme", "homme")]),
    ("d' ", &[("de", "d'")]),
    ("Qu’", &[("Que", "Qu’")]),
    ("d'1", &[]),
    // --- the words that stay whole ---
    ("aujourd'hui", &[("aujourd'hui", "aujourd'hui")]),
    ("Aujourd’hui", &[("Aujourd'hui", "Aujourd’hui")]),
    ("presqu'île", &[("presqu'île", "presqu'île")]),
    ("quelqu'un", &[("quelqu'un", "quelqu'un")]),
    ("quelqu’une", &[("quelqu'une", "quelqu’une")]),
    ("prud'homme", &[("prud'homme", "prud'homme")]),
    ("entr'ouvert", &[("entr'ouvert", "entr'ouvert")]),
    ("grand'mère", &[("grand'mère", "grand'mère")]),
    ("lʼhomme", &[("lʼhomme", "lʼhomme")]),
    ("homme", &[("homme", "homme")]),
    ("Don't", &[("Don't", "Don't")]),
    // --- `au`/`aux` split, sharing the span; `du`/`des` whole (D4) ---
    (
        "au marché",
        &[("à", "au"), ("le", "au"), ("marché", "marché")],
    ),
    (
        "aux halles",
        &[("à", "aux"), ("les", "aux"), ("halles", "halles")],
    ),
    (
        "Au revoir",
        &[("À", "Au"), ("le", "Au"), ("revoir", "revoir")],
    ),
    ("AU", &[("À", "AU"), ("le", "AU")]),
    ("AUX", &[("À", "AUX"), ("les", "AUX")]),
    (
        "Aux armes",
        &[("À", "Aux"), ("les", "Aux"), ("armes", "armes")],
    ),
    ("du pain", &[("du", "du"), ("pain", "pain")]),
    ("des pommes", &[("des", "des"), ("pommes", "pommes")]),
    ("Du", &[("Du", "Du")]),
    ("auquel", &[("auquel", "auquel")]),
    ("auxquels", &[("auxquels", "auxquels")]),
    ("duquel", &[("duquel", "duquel")]),
    ("desquels", &[("desquels", "desquels")]),
    ("au-dessus", &[("au-dessus", "au-dessus")]),
    ("au-delà", &[("au-delà", "au-delà")]),
    (
        "jusqu'au-delà",
        &[("jusque", "jusqu'"), ("au-delà", "au-delà")],
    ),
    // --- hyphenated inversions: each piece a word with its own span (D5) ---
    ("dit-il", &[("dit", "dit"), ("il", "il")]),
    ("a-t-il", &[("a", "a"), ("il", "il")]),
    ("pense-t-elle", &[("pense", "pense"), ("elle", "elle")]),
    (
        "demanda-t-elle",
        &[("demanda", "demanda"), ("elle", "elle")],
    ),
    ("va-t-on", &[("va", "va"), ("on", "on")]),
    (
        "Viendront-ils",
        &[("Viendront", "Viendront"), ("ils", "ils")],
    ),
    ("viendras-tu", &[("viendras", "viendras"), ("tu", "tu")]),
    ("Est-ce", &[("Est", "Est"), ("ce", "ce")]),
    ("Qu'est-ce", &[("Que", "Qu'"), ("est", "est"), ("ce", "ce")]),
    ("Va-t'en", &[("Va", "Va"), ("toi", "t'"), ("en", "en")]),
    (
        "Donne-m'en",
        &[("Donne", "Donne"), ("moi", "m'"), ("en", "en")],
    ),
    (
        "allez-vous-en",
        &[("allez", "allez"), ("vous", "vous"), ("en", "en")],
    ),
    ("coupez-les", &[("coupez", "coupez"), ("les", "les")]),
    (
        "dis-le-moi",
        &[("dis", "dis"), ("le", "le"), ("moi", "moi")],
    ),
    ("Vas-y", &[("Vas", "Vas"), ("y", "y")]),
    ("Prends-en", &[("Prends", "Prends"), ("en", "en")]),
    ("mets-l'y", &[("mets", "mets"), ("le", "l'"), ("y", "y")]),
    ("donnez-leur", &[("donnez", "donnez"), ("leur", "leur")]),
    ("dites-lui", &[("dites", "dites"), ("lui", "lui")]),
    ("sommes-nous", &[("sommes", "sommes"), ("nous", "nous")]),
    ("ai-je", &[("ai", "ai"), ("je", "je")]),
    (
        "Qu’en dit-on",
        &[("Que", "Qu’"), ("en", "en"), ("dit", "dit"), ("on", "on")],
    ),
    // --- runs the pack lists whole, and an elision before one ---
    ("rendez-vous", &[("rendez-vous", "rendez-vous")]),
    ("peut-être", &[("peut-être", "peut-être")]),
    ("c'est-à-dire", &[("c'est-à-dire", "c'est-à-dire")]),
    ("porte-monnaie", &[("porte-monnaie", "porte-monnaie")]),
    (
        "l'arc-en-ciel",
        &[("le", "l'"), ("arc-en-ciel", "arc-en-ciel")],
    ),
    (
        "d’arc-en-ciel",
        &[("de", "d’"), ("arc-en-ciel", "arc-en-ciel")],
    ),
    // --- runs that are no inversion: the compound rule ---
    ("celui-ci", &[("celui-ci", "celui-ci")]),
    ("celle-là", &[("celle-là", "celle-là")]),
    ("jour-là", &[("jour-là", "jour-là")]),
    ("moi-même", &[("moi-même", "moi-même")]),
    ("demi-heure", &[("demi-heure", "demi-heure")]),
    ("Jean-Pierre", &[("Jean-Pierre", "Jean-Pierre")]),
    ("Saint-Y", &[("Saint-Y", "Saint-Y")]),
    ("DIT-IL", &[("DIT-IL", "DIT-IL")]),
    ("dit-t-nous", &[("dit-t-nous", "dit-t-nous")]),
    ("rendez-moi-ça", &[("rendez-moi-ça", "rendez-moi-ça")]),
    (
        "Qu’est-ce que",
        &[("Que", "Qu’"), ("est", "est"), ("ce", "ce"), ("que", "que")],
    ),
    (
        "qu'est-ce-là",
        &[("que", "qu'"), ("est-ce-là", "est-ce-là")],
    ),
    // --- a digit in a run: each piece a French word ---
    ("l'an-2000", &[("le", "l'"), ("an", "an")]),
    ("au-10", &[("à", "au"), ("le", "au")]),
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
fn french_tokenisation_fixtures() {
    assert!(
        CASES.len() >= 80,
        "task 1.7 asks for at least 80 cases, got {}",
        CASES.len()
    );
    let lex = lexicon();
    let wrong: Vec<String> = CASES
        .iter()
        .filter_map(|(text, expected)| {
            let tokens = tokenize(text, FR, &lex);
            let got = read(text, &tokens);
            (got != *expected).then(|| format!("{text:?}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn french_spans_follow_the_text() {
    // Every span lies inside the text, starts no earlier than the last one, and holds no
    // narrow no-break space; the pieces of one written word never overlap but at a shared span.
    let lex = lexicon();
    for (text, _) in CASES {
        let tokens = tokenize(text, FR, &lex);
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
            assert!(!text[token.start..token.end].contains(NNBSP), "{text:?}");
            assert!(!token.text.contains(NNBSP), "{text:?}");
            last = (token.start, token.end);
        }
    }
}

#[test]
fn english_and_spanish_read_the_fixtures_as_before() {
    // French's pre-pass is French's arm: no elision, contraction or inversion of it runs on
    // English or Spanish text.
    let lex = lexicon();
    for language in [StudiedLanguage::English, StudiedLanguage::Spanish] {
        let texts = |text: &str| -> Vec<String> {
            tokenize(text, language, &lex)
                .into_iter()
                .map(|t| t.text)
                .collect()
        };
        assert_eq!(texts("l'homme"), ["l'homme"]);
        assert_eq!(texts("Au revoir"), ["Au", "revoir"]);
        assert_eq!(texts("dit-il"), ["dit-il"]);
        assert_eq!(texts("pas\u{202F}?"), ["pas\u{202F}"]);
    }
}

/// A French pack holding the stand-in lexicon, at French's analyser version.
fn pack() -> Pack {
    let (forms, pool) = build_lexicon_blobs(FORMS, LEMMAS).expect("lexicon");
    let lemma_count = FstLexicon::from_slices(forms.clone(), &pool)
        .expect("load")
        .lemma_count();
    let freq = vec![0u8; lemma_count * 4];
    let meta = serde_json::to_vec(&PackMeta {
        studied: "fr".into(),
        native: "en".into(),
        pack_version: "fixture".into(),
        analyzer_version: FRENCH_ANALYZER_VERSION.into(),
        licences: vec![],
        levels_estimated: false,
    })
    .expect("meta");
    let sections: [(&str, &[u8]); 3] = [
        (section::FORMS, &forms),
        (section::LEMMAS, pool.as_bytes()),
        (section::FREQ, &freq),
    ];
    Pack::load(&write_container(&meta, &sections)).expect("a French pack loads")
}

#[test]
fn a_word_card_reads_the_written_word_through_the_pre_pass() {
    let pack = pack();
    let pieces = |written: &str, lemma: &str| word_grammar(written, lemma, FR, &pack).pieces;
    // An elided word on its own is one token, `le`: no pieces.
    assert_eq!(
        read("l’", &tokenize("l’", FR, pack.lexicon())),
        [("le", "l’")]
    );
    assert!(pieces("l’", "le").is_empty());
    // A contraction answers its two pieces.
    assert_eq!(pieces("au", "à"), ["à", "le"]);
    // An elision's pieces, each a word.
    assert_eq!(pieces("l'homme", "homme"), ["le", "homme"]);
    assert_eq!(pieces("L’homme", "homme"), ["Le", "homme"]);
    // A word the pre-pass leaves whole has none.
    assert!(pieces("dit", "dire").is_empty());
    assert!(pieces("aujourd'hui", "aujourd'hui").is_empty());
}
