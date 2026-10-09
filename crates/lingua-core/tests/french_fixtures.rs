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

//! Non-regression fixtures for French's analysis, grouped by rule.
//!
//! Its tokenisation pre-pass (add-lingua-french-tokenisation, task 1.7 — at
//! least 80 cases): the narrow no-break space, each elided form, the words that
//! stay whole, `au`/`aux` and `du`/`des`, the hyphenated inversions and the runs
//! that are none. Every case states each token beside the source text its span
//! covers, so every split is checked with its spans.
//!
//! The rules of its own analysis (add-lingua-french-analysis, task 6.1 — at
//! least 70 cases): NFC in the pre-pass (decomposed words, elisions, runs and
//! capitals, each span the source's), the cascade (listed forms, each plural
//! ending, each exclusion, the guard on a listed singular), the closed classes
//! (each class, each word left out) and the names rule, through `analyse_page`
//! on a small synthetic pack (each branch of its design's D4).
//!
//! Any change to a fixture's expectation is a behavioural change of the French
//! analyser and demands a `FRENCH_ANALYZER_VERSION` bump; English's and
//! Spanish's fixtures and baselines do not move.
//!
//! The lexicons are small stand-ins for the forms tables
//! (add-lingua-french-forms-tables), which measure these rules on real text.

use lingua_core::analysis::FRENCH_ANALYZER_VERSION;
use lingua_core::analysis::function_words::is_function_word;
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::lemmatize::lemmatize;
use lingua_core::analysis::lexicon::{FstLexicon, build_lexicon_blobs};
use lingua_core::analysis::percent::TokenClass;
use lingua_core::analysis::tokenize::{Token, tokenize};
use lingua_core::engine::{analyse_page, word_grammar};
use lingua_core::knowledge::{KnowledgeState, KnownSource, Status};
use lingua_core::packs::pack::section;
use lingua_core::packs::{Pack, PackMeta, write_container};
use unicode_normalization::UnicodeNormalization;

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

// --- add-lingua-french-analysis (task 6.1) ---

/// NFC in the pre-pass (D1): (text tokenised as French, the tokens as (text, the source text the
/// span covers)). Every word is composed before it is compared and written composed; every span
/// covers the decomposed letters and their combining marks.
const NFC_CASES: &[Case] = &[
    ("me\u{301}moire", &[("mémoire", "me\u{301}moire")]),
    ("E\u{301}cole", &[("École", "E\u{301}cole")]),
    ("E\u{301}TAT", &[("ÉTAT", "E\u{301}TAT")]),
    ("a\u{300}", &[("à", "a\u{300}")]),
    (
        "A\u{300} demain",
        &[("À", "A\u{300}"), ("demain", "demain")],
    ),
    ("c\u{327}'a", &[("ça", "c\u{327}'"), ("a", "a")]),
    (
        "C\u{327}\u{2019}e\u{301}tait",
        &[("Ça", "C\u{327}\u{2019}"), ("était", "e\u{301}tait")],
    ),
    ("c\u{327}\u{2019} ", &[("ça", "c\u{327}\u{2019}")]),
    (
        "l'e\u{301}te\u{301}",
        &[("le", "l'"), ("été", "e\u{301}te\u{301}")],
    ),
    (
        "qu\u{2019}E\u{301}lise",
        &[("que", "qu\u{2019}"), ("Élise", "E\u{301}lise")],
    ),
    ("jusqu'a\u{300}", &[("jusque", "jusqu'"), ("à", "a\u{300}")]),
    ("presqu'i\u{302}le", &[("presqu'île", "presqu'i\u{302}le")]),
    (
        "Au cafe\u{301}",
        &[("À", "Au"), ("le", "Au"), ("café", "cafe\u{301}")],
    ),
    ("peut-e\u{302}tre", &[("peut-être", "peut-e\u{302}tre")]),
    (
        "c'est-a\u{300}-dire",
        &[("c'est-à-dire", "c'est-a\u{300}-dire")],
    ),
    (
        "l'arc-en-ciel e\u{301}tait",
        &[
            ("le", "l'"),
            ("arc-en-ciel", "arc-en-ciel"),
            ("était", "e\u{301}tait"),
        ],
    ),
    (
        "Saint-E\u{301}tienne",
        &[("Saint-Étienne", "Saint-E\u{301}tienne")],
    ),
    (
        "re\u{301}pondit-il",
        &[("répondit", "re\u{301}pondit"), ("il", "il")],
    ),
    (
        "e\u{301}tait-il",
        &[("était", "e\u{301}tait"), ("il", "il")],
    ),
];

/// The forms the cascade's stand-in pack lists, with their lemmas: one lemma per form, as the
/// tables choose (M8).
const CASCADE_FORMS: &[(&str, &str)] = &[
    ("as", "avoir"),
    ("a", "avoir"),
    ("été", "être"),
    ("étais", "être"),
    ("est", "être"),
    ("peut", "pouvoir"),
    ("doit", "devoir"),
    ("porte", "porter"),
    ("portes", "porter"),
    ("vivant", "vivre"),
    ("sort", "sortir"),
    ("vienne", "venir"),
    ("mes", "mon"),
    ("travaux", "travail"),
    ("endors", "endormir"),
    ("yeux", "œil"),
    ("fait", "faire"),
    ("ces", "ce"),
];

/// Lemmas the cascade's stand-in pack lists as their own forms.
const CASCADE_LEMMAS: &[&str] = &[
    "avoir",
    "être",
    "pouvoir",
    "devoir",
    "porter",
    "vivre",
    "sortir",
    "venir",
    "mon",
    "travail",
    "endormir",
    "œil",
    "cheval",
    "maison",
    "printemps",
    "faire",
    "ce",
    "personne",
];

/// (form, expected lemma), grouped by the step of the cascade that resolves them (D2).
const CASCADE_CASES: &[(&str, &str)] = &[
    // --- the pack's forms: the tables decide ---
    ("porte", "porter"),
    ("Portes", "porter"),
    ("été", "être"),
    ("ÉTÉ", "être"),
    ("étais", "être"),
    ("as", "avoir"),
    ("mes", "mon"),
    ("travaux", "travail"),
    ("endors", "endormir"),
    ("yeux", "œil"),
    ("printemps", "printemps"),
    ("Vienne", "venir"),
    ("e\u{301}te\u{301}", "être"),
    // --- an unlisted lowercase plural in `-s`, its singular unlisted too ---
    ("mégalithes", "mégalithe"),
    ("vicissitudes", "vicissitude"),
    ("auspices", "auspice"),
    ("belgicismes", "belgicisme"),
    ("félibres", "félibre"),
    ("comarques", "comarque"),
    ("alluvions", "alluvion"),
    ("ramures", "ramure"),
    ("patoisants", "patoisant"),
    ("me\u{301}galithes", "mégalithe"),
    // --- `-aux` → `-al` ---
    ("chenaux", "chenal"),
    ("bocaux", "bocal"),
    ("végétaux", "végétal"),
    // --- `-eaux` → `-eau`, before `-aux` ---
    ("perdreaux", "perdreau"),
    ("lionceaux", "lionceau"),
    ("bateaux", "bateau"),
    // --- the guard: a singular the pack lists keeps its plural itself (M8) ---
    ("étés", "étés"),
    ("vivants", "vivants"),
    ("sorts", "sorts"),
    ("chevaux", "chevaux"),
    ("maisons", "maisons"),
    ("personnes", "personnes"),
    // --- exclusions: `-x` that is no `-aux`, singulars in `-s`, short words ---
    ("heureux", "heureux"),
    ("bijoux", "bijoux"),
    ("campus", "campus"),
    ("souris", "souris"),
    ("succès", "succès"),
    ("repos", "repos"),
    ("stress", "stress"),
    ("gens", "gens"),
    ("mois", "mois"),
    ("bras", "bras"),
    // --- exclusions: hyphenated and elided words ---
    ("arc-en-ciels", "arc-en-ciels"),
    ("prud'hommes", "prud'hommes"),
    ("grand'mères", "grand'mères"),
    // --- exclusions: the passé simple ---
    ("dormîmes", "dormîmes"),
    ("cessâmes", "cessâmes"),
    ("reçûmes", "reçûmes"),
    ("cessâtes", "cessâtes"),
    ("dormîtes", "dormîtes"),
    ("reçûtes", "reçûtes"),
    // --- a capitalised form outside the pack: the form, lowercased ---
    ("Belfons", "belfons"),
    ("Wisigoths", "wisigoths"),
    ("MÉGALITHES", "mégalithes"),
    ("ME\u{301}GALITHES", "mégalithes"),
    // --- the form itself ---
    ("maison", "maison"),
    ("hier", "hier"),
];

/// (form, whether its lemma is a function word), each class and each word left out (D3). The
/// forms go through the cascade first, as a phrase gloss's do.
const CLOSED_CLASS_CASES: &[(&str, bool)] = &[
    // --- determiners ---
    ("le", true),
    ("la", true),
    ("les", true),
    ("un", true),
    ("une", true),
    ("du", true),
    ("des", true),
    ("cet", true),
    ("ces", true),
    ("mon", true),
    ("mes", true),
    ("ton", true),
    ("son", true),
    ("leur", true),
    ("chaque", true),
    ("tout", true),
    ("beaucoup", true),
    // --- pronouns ---
    ("je", true),
    ("il", true),
    ("on", true),
    ("nous", true),
    ("y", true),
    ("en", true),
    ("ça", true),
    ("qui", true),
    ("dont", true),
    ("lequel", true),
    ("quelqu'un", true),
    ("rien", true),
    // --- prepositions ---
    ("à", true),
    ("de", true),
    ("dans", true),
    ("pour", true),
    ("jusque", true),
    ("malgré", true),
    // --- conjunctions ---
    ("et", true),
    ("mais", true),
    ("que", true),
    ("si", true),
    ("lorsque", true),
    // --- auxiliaries and modals, through their forms ---
    ("été", true),
    ("as", true),
    ("peut", true),
    ("doit", true),
    // --- negation, « pas » by M21 ---
    ("ne", true),
    ("pas", true),
    ("non", true),
    // --- left out: words that look closed but carry meaning ---
    ("personne", false),
    ("point", false),
    ("or", false),
    ("plus", false),
    ("jamais", false),
    ("guère", false),
    ("certain", false),
    ("fait", false),
    ("falloir", false),
    ("vouloir", false),
    ("aller", false),
    ("voici", false),
    ("voilà", false),
    // --- content words ---
    ("maison", false),
    ("porte", false),
    ("hier", false),
];

fn cascade_lexicon() -> FstLexicon<Vec<u8>> {
    let (bytes, pool) = build_lexicon_blobs(CASCADE_FORMS, CASCADE_LEMMAS).expect("build");
    FstLexicon::from_slices(bytes, &pool).expect("load")
}

#[test]
fn french_nfc_fixtures() {
    let lex = lexicon();
    let wrong: Vec<String> = NFC_CASES
        .iter()
        .filter_map(|(text, expected)| {
            let tokens = tokenize(text, FR, &lex);
            let got = read(text, &tokens);
            (got != *expected).then(|| format!("{text:?}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    // English reads the same text as it came.
    for (text, _) in NFC_CASES {
        for token in tokenize(text, StudiedLanguage::English, &lex) {
            assert_eq!(
                token.text,
                text[token.start..token.end].replace('\u{2019}', "'")
            );
        }
    }
}

#[test]
fn french_cascade_fixtures() {
    let lex = cascade_lexicon();
    let wrong: Vec<String> = CASCADE_CASES
        .iter()
        .filter_map(|(form, expected)| {
            let got = lemmatize(form, FR, &lex);
            (got != *expected).then(|| format!("{form:?}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    // Every lemma is lowercase and composed.
    for (form, _) in CASCADE_CASES {
        let got = lemmatize(form, FR, &lex);
        assert_eq!(got, got.to_lowercase(), "{form:?}");
        assert!(!got.contains('\u{301}'), "{form:?}");
    }
}

#[test]
fn french_closed_class_fixtures() {
    let lex = cascade_lexicon();
    let wrong: Vec<String> = CLOSED_CLASS_CASES
        .iter()
        .filter_map(|(form, expected)| {
            let lemma = lemmatize(form, FR, &lex);
            let got = is_function_word(&lemma, FR);
            (got != *expected).then(|| format!("{form:?} → {lemma:?}: expected {expected}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    // French's tables are French's: neither English nor Spanish reads `ne` or `pas` as theirs.
    for language in [StudiedLanguage::English, StudiedLanguage::Spanish] {
        assert!(!is_function_word("ne", language));
        assert!(!is_function_word("pas", language));
    }
}

/// The names rule's stand-in pack (D4): every word the cases write that the pack lists, the
/// dictionary words named by a lexical section, `vienne` a form of the dictionary word `venir`.
const NAMES_FORMS: &[(&str, &str)] = &[("vienne", "venir")];

/// Lemmas of the names pack.
const NAMES_LEMMAS: &[&str] = &[
    "paris", "lot", "aube", "espagne", "saint", "pierre", "orange", "venir", "mme", "garonne",
    "durand", "le", "la", "de",
];

/// The names pack's dictionary words; `paris`, `lot`, `aube`, `espagne`, `mme`, `garonne` and
/// `durand` are lemmas but no dictionary words.
const NAMES_DICTIONARY: &[&str] = &["saint", "pierre", "orange", "venir", "le", "la", "de"];

/// (the document's blocks, a surface, the class of each of its occurrences, in order).
type NameCase = (&'static [&'static str], &'static str, &'static [TokenClass]);

const NAME: TokenClass = TokenClass::ProperNounOutOfLexicon;
const WORD: TokenClass = TokenClass::Unknown;

const NAME_CASES: &[NameCase] = &[
    // --- Spanish's rule: a capital in mid-sentence, never in lowercase, no dictionary word ---
    (
        &["Nous partons de Paris demain matin avec toute la famille et le chien."],
        "Paris",
        &[NAME],
    ),
    (
        &[
            "Nous partons de Paris demain matin avec toute la famille et le chien.",
            "Paris est loin de notre petit village de montagne, mais le voyage est beau.",
        ],
        "Paris",
        &[NAME, NAME],
    ),
    (
        &["Il pleut encore, Paris dort sous les nuages gris de ce long matin d'hiver."],
        "Paris",
        &[NAME],
    ),
    // --- no evidence: the head of a block, of a sentence, after `M.`'s full stop ---
    (
        &[
            "Paris est une grande ville avec beaucoup de musées et de jardins.",
            "Paris attire chaque année des millions de visiteurs du monde entier.",
        ],
        "Paris",
        &[WORD, WORD],
    ),
    (
        &["Il est arrivé hier soir. Paris dormait déjà sous la pluie froide de novembre."],
        "Paris",
        &[WORD],
    ),
    (
        &["Nous avons rencontré M. Durand près de la gare du village hier soir."],
        "Durand",
        &[WORD],
    ),
    (
        &[
            "Mme Durand ouvre la boutique tous les matins à huit heures précises.",
            "Mme Durand vend du pain, des gâteaux et des croissants au beurre frais.",
        ],
        "Mme",
        &[WORD, WORD],
    ),
    // --- a capital right after an elided piece, straight or typographic, even at a head ---
    (
        &["La rivière traverse l'Aube avant de rejoindre la Seine près du village."],
        "Aube",
        &[NAME],
    ),
    (
        &["L'Aube rejoint la Seine dans la plaine, après un long voyage vers le nord."],
        "Aube",
        &[NAME],
    ),
    (
        &["Ils reviennent d\u{2019}Espagne avec leurs amis après un long voyage en voiture."],
        "Espagne",
        &[NAME],
    ),
    (
        &["Le train part demain. D'Espagne, il rejoindra la mer avant la fin du mois."],
        "Espagne",
        &[NAME],
    ),
    // --- a hyphenated run the pack does not list is one form ---
    (
        &["Les gendarmes ont retrouvé Jean-Pierre près de la gare hier soir avec son chien."],
        "Jean-Pierre",
        &[NAME],
    ),
    (
        &["Nous avons visité Saint-Étienne pendant les vacances avec toute la famille."],
        "Saint-Étienne",
        &[NAME],
    ),
    (
        &["Nous avons visité Saint-E\u{301}tienne pendant les vacances avec toute la famille."],
        "Saint-Étienne",
        &[NAME],
    ),
    (
        &["Saint-Étienne est une grande ville industrielle au sud de la région."],
        "Saint-Étienne",
        &[WORD],
    ),
    // --- a dictionary word stays a word, by its lemma or by the lemma its form reads as ---
    (
        &["Le train roule entre Orange et la mer pendant toute la matinée."],
        "Orange",
        &[WORD],
    ),
    (
        &["Nous avons passé l'été entre la mer et Vienne pendant les vacances."],
        "Vienne",
        &[WORD],
    ),
    (
        &["Nous avons vu Pierre hier soir près de la gare avec son chien."],
        "Pierre",
        &[WORD],
    ),
    // --- a form the document also writes in lowercase ---
    (
        &[
            "La rivière du Lot traverse le village avant de rejoindre la Garonne.",
            "Il a acheté un lot de livres anciens au marché de la place.",
        ],
        "Lot",
        &[WORD],
    ),
    (
        &[
            "La rivière du Lot traverse le village avant de rejoindre la Garonne.",
            "Il a acheté un lot de livres anciens au marché de la place.",
        ],
        "Garonne",
        &[NAME],
    ),
];

/// A French pack holding the names lexicon, its dictionary words in a lexical section, at French's
/// analyser version.
fn names_pack() -> Pack {
    let (forms, pool) = build_lexicon_blobs(NAMES_FORMS, NAMES_LEMMAS).expect("lexicon");
    let lex = FstLexicon::from_slices(forms.clone(), &pool).expect("load");
    let freq = vec![0u8; lex.lemma_count() * 4];
    let mut lexical = vec![0u8; lex.lemma_count().div_ceil(8)];
    for word in NAMES_DICTIONARY {
        let id = lex.id_of(word).expect("a dictionary word is a lemma") as usize;
        lexical[id / 8] |= 1 << (id % 8);
    }
    let meta = serde_json::to_vec(&PackMeta {
        studied: "fr".into(),
        native: "en".into(),
        pack_version: "fixture".into(),
        analyzer_version: FRENCH_ANALYZER_VERSION.into(),
        licences: vec![],
        levels_estimated: false,
    })
    .expect("meta");
    let sections: [(&str, &[u8]); 4] = [
        (section::FORMS, &forms),
        (section::LEMMAS, pool.as_bytes()),
        (section::FREQ, &freq),
        (section::LEXICAL, &lexical),
    ];
    Pack::load(&write_container(&meta, &sections)).expect("the names pack loads")
}

/// The classes of every token written `surface` on a French page.
fn classes(
    pack: &Pack,
    blocks: &[&str],
    surface: &str,
    knowledge: &KnowledgeState,
) -> Vec<TokenClass> {
    let page = analyse_page(blocks, FR, pack, knowledge);
    assert!(page.analysable, "{blocks:?}");
    page.tokens
        .into_iter()
        .filter(|token| token.surface.nfc().collect::<String>() == surface)
        .map(|token| token.class)
        .collect()
}

#[test]
fn french_names_fixtures() {
    let pack = names_pack();
    let knowledge = KnowledgeState::new();
    let wrong: Vec<String> = NAME_CASES
        .iter()
        .filter_map(|(blocks, surface, expected)| {
            let got = classes(&pack, blocks, surface, &knowledge);
            (got != *expected)
                .then(|| format!("{surface:?} in {blocks:?}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_name_the_reader_marked_keeps_its_status() {
    let pack = names_pack();
    let mut knowledge = KnowledgeState::new();
    knowledge.set_status(FR, "paris", Status::Learning);
    knowledge.set_status(FR, "jean-pierre", Status::Known(KnownSource::Manual));
    let blocks = [
        "Nous partons de Paris demain matin avec toute la famille et le chien.",
        "Les gendarmes ont retrouvé Jean-Pierre près de la gare hier soir.",
    ];
    assert_eq!(
        classes(&pack, &blocks, "Paris", &knowledge),
        [TokenClass::Learning]
    );
    assert_eq!(
        classes(&pack, &blocks, "Jean-Pierre", &knowledge),
        [TokenClass::Known]
    );
}

#[test]
fn a_glossed_lemma_is_a_dictionary_word_of_a_pack_without_a_lexical_section() {
    // The fixture's case (`is_dictionary_word`'s fallback): `orange` glossed stays a word,
    // `paris` unglossed goes.
    let (forms, pool) =
        build_lexicon_blobs(&[], &["orange", "paris", "le", "la", "de"]).expect("lexicon");
    let lex = FstLexicon::from_slices(forms.clone(), &pool).expect("load");
    let freq = vec![0u8; lex.lemma_count() * 4];
    let orange = lex.id_of("orange").expect("orange") as u32;
    let mut raw = Vec::new();
    raw.extend_from_slice(&1u32.to_le_bytes());
    raw.extend_from_slice(&orange.to_le_bytes());
    raw.extend_from_slice(&0u32.to_le_bytes());
    raw.extend_from_slice(&6u32.to_le_bytes());
    raw.extend_from_slice(b"orange");
    let gloss = zstd::encode_all(raw.as_slice(), 19).expect("zstd");
    let meta = serde_json::to_vec(&PackMeta {
        studied: "fr".into(),
        native: "en".into(),
        pack_version: "fixture".into(),
        analyzer_version: FRENCH_ANALYZER_VERSION.into(),
        licences: vec![],
        levels_estimated: false,
    })
    .expect("meta");
    let sections: [(&str, &[u8]); 4] = [
        (section::FORMS, &forms),
        (section::LEMMAS, pool.as_bytes()),
        (section::FREQ, &freq),
        (section::GLOSS_ZST, &gloss),
    ];
    let pack = Pack::load(&write_container(&meta, &sections)).expect("loads");
    let blocks = ["Le train roule entre Orange et Paris pendant toute la matinée."];
    let knowledge = KnowledgeState::new();
    assert_eq!(classes(&pack, &blocks, "Orange", &knowledge), [WORD]);
    assert_eq!(classes(&pack, &blocks, "Paris", &knowledge), [NAME]);
}

#[test]
fn the_new_rules_hold_at_least_seventy_cases() {
    let cases = NFC_CASES.len() + CASCADE_CASES.len() + CLOSED_CLASS_CASES.len() + NAME_CASES.len();
    assert!(
        cases >= 70,
        "task 6.1 asks for at least 70 cases, got {cases}"
    );
}
