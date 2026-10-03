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

//! Non-regression fixtures for Spanish's analysis (add-lingua-spanish-analysis,
//! task 4.1 — at least 100 cases). Any change to a fixture's expectation is a
//! behavioural change of the Spanish analyser and demands a
//! `SPANISH_ANALYZER_VERSION` bump; English's fixtures and baseline do not move.
//!
//! The lexicon is a small stand-in for the forms tables
//! (add-lingua-spanish-forms-tables), which measure these rules on real text.

use lingua_core::analysis::function_words::is_function_word;
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::lemmatize::lemmatize;
use lingua_core::analysis::lexicon::{FstLexicon, build_lexicon_blobs};
use lingua_core::analysis::tokenize::tokenize;

const ES: StudiedLanguage = StudiedLanguage::Spanish;

/// The forms the stand-in pack lists, with their lemmas.
const FORMS: &[(&str, &str)] = &[
    // dar
    ("da", "dar"),
    ("dad", "dar"),
    ("dio", "dar"),
    ("doy", "dar"),
    // decir
    ("di", "decir"),
    ("diga", "decir"),
    ("diciendo", "decir"),
    ("dijo", "decir"),
    // hacer, poner, salir, tener, venir, ir, ver
    ("haz", "hacer"),
    ("haciendo", "hacer"),
    ("hizo", "hacer"),
    ("pon", "poner"),
    ("sal", "salir"),
    ("sale", "salir"),
    ("ten", "tener"),
    ("tiene", "tener"),
    ("ven", "venir"),
    ("vino", "venir"),
    ("ve", "ir"),
    ("id", "ir"),
    ("va", "ir"),
    ("vamos", "ir"),
    ("ved", "ver"),
    ("vio", "ver"),
    // ser, estar, haber
    ("fue", "ser"),
    ("fui", "ser"),
    ("es", "ser"),
    ("está", "estar"),
    ("estaba", "estar"),
    ("has", "haber"),
    ("ha", "haber"),
    // regular verbs
    ("hablo", "hablar"),
    ("habló", "hablar"),
    ("come", "comer"),
    ("comió", "comer"),
    ("comiendo", "comer"),
    ("vivimos", "vivir"),
    ("levanta", "levantar"),
    ("levantad", "levantar"),
    ("sienta", "sentar"),
    ("sentad", "sentar"),
    ("sentemos", "sentar"),
    ("mira", "mirar"),
    ("mirando", "mirar"),
    ("escribiendo", "escribir"),
    ("leyendo", "leer"),
    // nouns and others
    ("casas", "casa"),
    ("libros", "libro"),
    ("pie", "pie"),
    ("solo", "solo"),
    ("este", "este"),
    ("esta", "este"),
    ("cielo", "cielo"),
    ("pelo", "pelo"),
    ("lunes", "lunes"),
    ("a", "a"),
    ("o", "o"),
];

/// Lemmas the stand-in pack lists as their own forms.
const LEMMAS: &[&str] = &[
    "dar", "decir", "hacer", "poner", "salir", "tener", "venir", "ir", "ver", "ser", "estar",
    "haber", "hablar", "comer", "vivir", "levantar", "sentar", "mirar", "escribir", "leer", "reír",
    "oír", "traer", "dormir", "casa", "libro", "el", "de", "mercado", "centro",
];

/// (form, expected lemma), grouped by the stage of the cascade that resolves them.
const CASES: &[(&str, &str)] = &[
    // --- the pack's forms ---
    ("da", "dar"),
    ("Dijo", "decir"),
    ("hizo", "hacer"),
    ("tiene", "tener"),
    ("vino", "venir"),
    ("vamos", "ir"),
    ("es", "ser"),
    ("Estaba", "estar"),
    ("has", "haber"),
    ("hablo", "hablar"),
    ("habló", "hablar"),
    ("comió", "comer"),
    ("vivimos", "vivir"),
    ("casas", "casa"),
    ("libros", "libro"),
    ("lunes", "lunes"),
    ("sale", "salir"),
    ("come", "comer"),
    ("cielo", "cielo"),
    ("pelo", "pelo"),
    // --- old spellings: the form without its acute accents ---
    ("fué", "ser"),
    ("fuí", "ser"),
    ("dió", "dar"),
    ("vió", "ver"),
    ("á", "a"),
    ("ó", "o"),
    ("sólo", "solo"),
    ("Sólo", "solo"),
    ("éste", "este"),
    ("ésta", "este"),
    ("pié", "pie"),
    // --- enclitics: one clitic ---
    ("dime", "decir"),
    ("hazlo", "hacer"),
    ("hazme", "hacer"),
    ("ponte", "poner"),
    ("salte", "salir"),
    ("tenlo", "tener"),
    ("vente", "venir"),
    ("vete", "ir"),
    ("idos", "ir"),
    ("dadme", "dar"),
    ("vedlo", "ver"),
    ("hacerlo", "hacer"),
    ("tenerla", "tener"),
    ("verte", "ver"),
    ("comerse", "comer"),
    ("dormirse", "dormir"),
    ("reírse", "reír"),
    ("oírlo", "oír"),
    ("levántate", "levantar"),
    ("siéntate", "sentar"),
    ("mírame", "mirar"),
    ("cómelo", "comer"),
    ("dígame", "decir"),
    ("diciéndole", "decir"),
    ("haciéndolo", "hacer"),
    ("mirándote", "mirar"),
    ("escribiéndole", "escribir"),
    ("leyéndolo", "leer"),
    // --- enclitics: the restored `-s` and `-d` ---
    ("vámonos", "ir"),
    ("sentémonos", "sentar"),
    ("sentaos", "sentar"),
    ("levantaos", "levantar"),
    ("deciros", "decir"),
    ("diciéndoos", "decir"),
    // --- enclitics: two clitics ---
    ("dámelo", "dar"),
    ("dámela", "dar"),
    ("dáselo", "dar"),
    ("dímelo", "decir"),
    ("póntelo", "poner"),
    ("decírselo", "decir"),
    ("traérmelo", "traer"),
    ("comiéndoselo", "comer"),
    ("hacérmelo", "hacer"),
    ("Dámelo", "dar"),
    // --- enclitic guards: nothing is split ---
    ("vale", "vale"),     // `va` is listed, but no imperative of the closed list
    ("comélo", "comélo"), // `come` stresses its first vowel, not the accented one
    ("carmela", "carmela"),
    ("hola", "hola"),  // `ho` is no base
    ("telas", "tela"), // no base: the plural fallback groups it
    ("damelo", "dar"), // an accent left out is not required: informal text drops it
    ("dios", "dios"),  // `os` never follows a tú imperative: no `di` + `os`
    // --- plurals outside the lexicon ---
    ("luces", "luz"),
    ("peces", "pez"),
    ("voces", "voz"),
    ("cruces", "cruz"),
    ("raíces", "raíz"),
    ("actrices", "actriz"),
    ("dulces", "dulce"),
    ("canciones", "canción"),
    ("naciones", "nación"),
    ("árboles", "árbol"),
    ("papeles", "papel"),
    ("ciudades", "ciudad"),
    ("paredes", "pared"),
    ("reyes", "rey"),
    ("leyes", "ley"),
    ("relojes", "reloj"),
    ("exámenes", "examen"),
    ("jóvenes", "joven"),
    ("madres", "madre"),
    ("padres", "padre"),
    ("nubes", "nube"),
    ("coches", "coche"),
    ("posibles", "posible"),
    ("mesas", "mesa"),
    // --- plural exclusions ---
    ("crisis", "crisis"),
    ("virus", "virus"),
    ("análisis", "análisis"),
    ("tesis", "tesis"),
    ("tres", "tres"),
];

fn lexicon() -> FstLexicon<Vec<u8>> {
    let (bytes, pool) = build_lexicon_blobs(FORMS, LEMMAS).expect("build");
    FstLexicon::from_slices(bytes, &pool).expect("load")
}

#[test]
fn spanish_lemmatisation_fixtures() {
    assert!(
        CASES.len() >= 100,
        "task 4.1 asks for at least 100 cases, got {}",
        CASES.len()
    );
    let lex = lexicon();
    let wrong: Vec<String> = CASES
        .iter()
        .filter_map(|(form, expected)| {
            let got = lemmatize(form, ES, &lex);
            (got != *expected).then(|| format!("{form:?}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn spanish_lemmas_are_lowercase_and_composed() {
    let lex = lexicon();
    // `está` written decomposed: `esta` + U+0301.
    assert_eq!(lemmatize("ESTA\u{0301}", ES, &lex), "estar");
    for (form, _) in CASES {
        let got = lemmatize(form, ES, &lex);
        assert_eq!(got, got.to_lowercase(), "{form:?}");
    }
}

#[test]
fn spanish_contractions_tokenise_into_their_words() {
    let lex = lexicon();
    let texts = |text: &str| -> Vec<String> {
        tokenize(text, ES, &lex)
            .into_iter()
            .map(|t| t.text)
            .collect()
    };
    assert_eq!(
        texts("Vengo del mercado al centro."),
        ["Vengo", "de", "el", "mercado", "a", "el", "centro"]
    );
    assert_eq!(texts("Del mar."), ["De", "el", "mar"]);
    assert_eq!(texts("don't"), ["don't"]);
}

#[test]
fn spanish_closed_classes_are_flagged_and_content_words_are_not() {
    let lex = lexicon();
    for word in [
        "el", "la", "de", "a", "en", "y", "que", "no", "mi", "su", "este", "yo", "nos", "le",
    ] {
        let lemma = lemmatize(word, ES, &lex);
        assert!(is_function_word(&lemma, ES), "{word:?} → {lemma:?}");
    }
    for word in ["casa", "libro", "mercado", "hablar", "nunca", "dijo"] {
        let lemma = lemmatize(word, ES, &lex);
        assert!(!is_function_word(&lemma, ES), "{word:?} → {lemma:?}");
    }
    // `ha` and `es` reach their auxiliaries through the pack's forms.
    assert!(is_function_word(&lemmatize("ha", ES, &lex), ES));
    assert!(is_function_word(&lemmatize("es", ES, &lex), ES));
}
