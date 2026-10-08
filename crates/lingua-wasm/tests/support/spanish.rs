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

//! The Spanish invariance baseline's scenario (`spanish_baseline.rs`): its corpus, probes and
//! reader, shared with the cross-native invariance test (`cross_native.rs`).

use super::{Card, PackSource, Scenario};

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

pub const SPANISH: Scenario = Scenario {
    pair: "es-fr",
    pack: PackSource::Tables,
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
