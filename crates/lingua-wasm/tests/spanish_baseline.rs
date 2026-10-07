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

//! Spanish invariance baseline (`docs/lingua/language-matrix-programme.md`, change 3).
//!
//! The language matrix generalises the core, the packs and the extension around the reader's
//! native language, and what French readers of Spanish see must come out of it unchanged — the
//! Spanish counterpart of `english_baseline.rs`, over a Spanish corpus (`baseline/pages-es.txt`)
//! and the real es-fr pack built from the committed tables. It is the pair a native-language
//! axis can move: the Spanish names rule and the vocabulary estimate read gloss presence.
//!
//! The engine holds en-fr beside es-fr, as the extension does (it starts on the default pair's
//! pack). The estimated Spanish ladder borrows English's typical vocabularies, which the core
//! freezes (`ENGLISH_TYPICAL_VOCABULARY`, generalise-lingua-native-language D6): the figures this
//! golden recorded when they were read from en-fr.
//!
//! A pull request that changes `baseline/es-fr.golden` changes what readers of Spanish see.
//! Only a dictionary update of es-fr, or a deliberate analyser change that bumps the Spanish
//! analyser version, should do that; an en-fr update moves the `beside en-fr` line alone.
//! Re-bless with
//! `LINGUA_BLESS=1 cargo test -p lingua-wasm --test spanish_baseline` and say why in the pull
//! request; `lingua-pack-update` re-blesses on its own branch.
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use support::{Card, Scenario, first_difference};

/// Dictionary forms whose gloss the card shows: irregular verbs, homographs, apocopes,
/// function words, words from the corpus. Some have none in the pack, on purpose.
const LEMMAS: &[&str] = &[
    "ser", "ir", "estar", "haber", "tener", "hacer", "decir", "poder", "dar", "ver", "casa",
    "vino", "venir", "como", "comer", "para", "parar", "creer", "crear", "banco", "cuenta",
    "contar", "llama", "llamar", "bueno", "grande", "primero", "mucho", "muy", "uno", "el", "de",
    "a", "faro", "marinero", "tormenta", "huelga", "sartén", "ojalá", "hidalgo", "rocín", "adarga",
    "guay", "jajaja",
];

/// Selections a reader glosses: expressions, locutions, sentences, clitics, and blocks that
/// are not Spanish (Catalan, English, French).
const PHRASES: &[&str] = &[
    "tener en cuenta",
    "hay que",
    "sin embargo",
    "a pesar de",
    "por lo tanto",
    "darse cuenta",
    "echar de menos",
    "de vez en cuando",
    "a lo mejor",
    "al aire libre",
    "En un lugar de la Mancha",
    "Fue a la casa de su abuela.",
    "El vino tinto vino de La Rioja.",
    "dámelo",
    "del barrio",
    "The lighthouse stood",
    "Le phare se dressait",
    "La ciutat de Barcelona",
];

/// (word as written, dictionary form) pairs a word card asks the grammar of.
const GRAMMAR: &[(&str, &str)] = &[
    ("fue", "ser"),
    ("fue", "ir"),
    ("era", "ser"),
    ("es", "ser"),
    ("tengo", "tener"),
    ("hizo", "hacer"),
    ("dijo", "decir"),
    ("puedo", "poder"),
    ("vino", "venir"),
    ("vino", "vino"),
    ("como", "comer"),
    ("como", "como"),
    ("casas", "casa"),
    ("buen", "bueno"),
    ("gran", "grande"),
    ("primer", "primero"),
    ("dámelo", "dar"),
    ("levantarme", "levantar"),
    ("hablaba", "hablar"),
    ("hable", "hablar"),
    ("cantaríamos", "cantar"),
    ("tuviera", "tener"),
    ("rápidas", "rápido"),
    ("al", "a"),
    ("del", "de"),
    ("mesas", "mesa"),
    ("fríelas", "freír"),
    ("hubieran", "haber"),
];

/// The pages analysed again for the reader with a history.
const READER_PAGES: &[&str] = &["noticias", "homografos", "nombres", "mixto"];

const PAGE_NAMES: &[&str] = &[
    "noticias",
    "ficcion",
    "quijote",
    "homografos",
    "encliticos",
    "contracciones",
    "apocopes",
    "nombres",
    "informal",
    "receta",
    "tecnico",
    "mixto",
    "gramatica",
];

const SPANISH: Scenario = Scenario {
    pair: "es-fr",
    beside: &["en-fr"],
    test: "spanish_baseline",
    pages: "pages-es.txt",
    page_names: PAGE_NAMES,
    lemmas: LEMMAS,
    phrases: PHRASES,
    grammar: GRAMMAR,
    reader_pages: READER_PAGES,
    statuses: &[
        ("faro", "known", 1.0),
        ("marinero", "learning", 2.0),
        ("ojalá", "ignored", 3.0),
        ("ventana", "known", 4.0),
        ("pueblo", "known", 5.0),
        // Withdrawn: « Remettre à apprendre ».
        ("pueblo", "", 6.0),
    ],
    exposures: ["barco", "mar", "tormenta", "viaje", "paciencia"],
    cards: [
        Card {
            lemma: "viaje",
            form: "viaje",
            sentence: "Ella dirigirá el viaje.",
            url: "https://example.com/homografos",
            gloss: Some("voyage"),
        },
        Card {
            lemma: "retraso",
            form: "retraso",
            sentence: "Tenemos que ocuparnos del retraso.",
            url: "https://example.com/noticias",
            gloss: None,
        },
        Card {
            lemma: "sonreír",
            form: "sonriendo",
            sentence: "Entró en la sala sonriendo.",
            url: "",
            gloss: None,
        },
    ],
    reader_phrase: "Ella dirigirá el viaje",
};

#[test]
fn spanish_output_has_not_moved() {
    let actual = SPANISH.render(Some("es"));
    let Some(expected) = SPANISH.bless_or_read(&actual) else {
        return;
    };
    assert!(
        actual == expected,
        "Spanish output moved (docs/lingua/language-matrix-programme.md: es-fr does not move).\n\
         First difference — {}\n\
         If this pull request means to change Spanish output (a dictionary update of es-fr, or \
         an analyser change that bumps the Spanish analyser version), re-bless with\n  \
         LINGUA_BLESS=1 cargo test -p lingua-wasm --test spanish_baseline\n\
         and say why in the pull request. Otherwise the change is wrong.",
        first_difference(&expected, &actual)
    );
}

#[test]
fn the_corpus_reads_as_expected() {
    SPANISH.check_corpus();
}

/// Spanish levels are derived from frequency (add-lingua-spanish-levels, D1): the real pack
/// calls them estimated.
#[test]
fn spanish_levels_are_estimated() {
    let engine = SPANISH.loaded();
    let es = || Some("es".to_owned());
    assert!(engine.has_levels(es()).expect("levels"));
    assert!(engine.levels_estimated(es()).expect("estimated"));
}
