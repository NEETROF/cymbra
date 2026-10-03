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

//! What a Spanish word card's grammar says, from the es-fr pack built here
//! from the committed tables (add-lingua-spanish-grammar-tables): kaikki's
//! tags as Universal Dependencies tags, a noun's gender, and the other
//! dictionary forms a homograph names.

use std::path::PathBuf;

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::engine::word_grammar;
use lingua_core::packs::Pack;
use lingua_pack::{build_pack, inputs_from_dir};

fn es_fr_pack() -> Pack {
    let tables =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/tables/es-fr");
    let inputs = inputs_from_dir(&tables).expect("read the committed es-fr tables");
    Pack::load(&build_pack(&inputs).expect("build the es-fr pack")).expect("load the es-fr pack")
}

/// The card's readings of `written` as `lemma`, and the other dictionary forms
/// with theirs, every tag in UD notation.
fn card(pack: &Pack, written: &str, lemma: &str) -> (Vec<String>, Vec<(String, Vec<String>)>) {
    let grammar = word_grammar(written, lemma, StudiedLanguage::Spanish, pack);
    let ud = |tags: &[lingua_core::packs::grammar::Tag]| -> Vec<String> {
        tags.iter().map(|tag| tag.to_ud()).collect()
    };
    let others = grammar
        .others
        .iter()
        .map(|other| (other.lemma.clone(), ud(&other.readings)))
        .collect();
    (ud(&grammar.readings), others)
}

const PRETERITE_3SG: &str = "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin";

#[test]
fn a_spanish_card_says_what_the_form_is_and_what_else_it_may_be() {
    let pack = es_fr_pack();

    // A noun's gender, on its own form and on its plural — which is also a form
    // of casar (`tú casas`).
    assert_eq!(
        card(&pack, "casa", "casa").0,
        ["NOUN|Gender=Fem|Number=Sing"]
    );
    assert_eq!(
        card(&pack, "Casas", "casa"),
        (
            vec!["NOUN|Gender=Fem|Number=Plur".to_owned()],
            vec![(
                "casar".to_owned(),
                vec!["VERB|Mood=Ind|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin".to_owned()]
            )]
        )
    );

    // A verb form: mood, tense, person, number.
    assert_eq!(
        card(&pack, "hablábamos", "hablar").0,
        ["VERB|Mood=Ind|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin"]
    );
    // An agreed participle.
    assert_eq!(
        card(&pack, "escrita", "escribir").0,
        ["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"]
    );
    // An adjective's agreement.
    assert_eq!(
        card(&pack, "rápidas", "rápido").0,
        ["ADJ|Gender=Fem|Number=Plur"]
    );

    // A homograph names its other dictionary form: `fue` is ser's, and ir's too.
    assert_eq!(
        card(&pack, "fue", "ser"),
        (
            vec![PRETERITE_3SG.to_owned()],
            vec![("ir".to_owned(), vec![PRETERITE_3SG.to_owned()])]
        )
    );
    // `vino` is the noun, and the preterite of venir.
    assert_eq!(
        card(&pack, "vino", "vino"),
        (
            vec!["NOUN|Gender=Masc|Number=Sing".to_owned()],
            vec![("venir".to_owned(), vec![PRETERITE_3SG.to_owned()])]
        )
    );
}
