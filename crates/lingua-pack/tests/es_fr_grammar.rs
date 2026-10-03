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

//! What a Spanish word card says, from the es-fr pack built here from the
//! committed tables: kaikki's tags as Universal Dependencies tags, a noun's
//! gender, the other dictionary forms a homograph names
//! (add-lingua-spanish-grammar-tables), and the French glosses
//! (add-lingua-spanish-gloss-tables).

use std::path::PathBuf;
use std::sync::OnceLock;

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::engine::{gloss_phrase, word_grammar};
use lingua_core::knowledge::state::KnowledgeState;
use lingua_core::packs::Pack;
use lingua_pack::{build_pack, inputs_from_dir};

/// The pack's bytes, built once for every test of this file.
fn es_fr_pack() -> Pack {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    let bytes = BYTES.get_or_init(|| {
        let tables = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../scripts/lingua-data/tables/es-fr");
        let inputs = inputs_from_dir(&tables).expect("read the committed es-fr tables");
        build_pack(&inputs).expect("build the es-fr pack")
    });
    Pack::load(bytes).expect("load the es-fr pack")
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

#[test]
fn a_spanish_card_reads_its_french_gloss() {
    let pack = es_fr_pack();
    let gloss = |word: &str| word_grammar(word, word, StudiedLanguage::Spanish, &pack);

    // The French Wiktionary's Spanish entry.
    let casa = gloss("casa");
    assert_eq!(casa.gloss.as_deref(), Some("Maison"));
    assert_eq!(
        casa.senses[0]
            .tag
            .as_ref()
            .map(|tag| tag.to_ud())
            .as_deref(),
        Some("NOUN")
    );
    // No Spanish entry there: the Spanish Wiktionary's French translation.
    let sector = gloss("sector");
    assert_eq!(sector.gloss.as_deref(), Some("Secteur"));
    assert_eq!(
        sector.senses[0]
            .tag
            .as_ref()
            .map(|tag| tag.to_ud())
            .as_deref(),
        Some("NOUN")
    );

    // An expression a translation glosses.
    let phrase = gloss_phrase(
        "tener en cuenta",
        StudiedLanguage::Spanish,
        &pack,
        &KnowledgeState::default(),
    );
    assert_eq!(
        phrase
            .expressions
            .iter()
            .map(|found| found.gloss.as_str())
            .collect::<Vec<_>>(),
        ["Prendre en compte, tenir compte de"]
    );
}

#[test]
fn spanish_levels_are_estimated_and_follow_frequency() {
    use lingua_core::knowledge::level::{CefrLevel, CefrLevels};

    let pack = es_fr_pack();
    // Derived from frequency (add-lingua-spanish-levels): the pack says so.
    assert!(pack.levels_estimated());
    assert_eq!(pack.level("de"), Some(CefrLevel::A1));
    assert_eq!(pack.level("casa"), Some(CefrLevel::A1));
    // Glossed only as a place, as a CEFR list would leave it out.
    assert_eq!(pack.level("madrid"), None);
}
