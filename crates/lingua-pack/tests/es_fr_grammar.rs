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
use lingua_core::analysis::percent::TokenClass;
use lingua_core::engine::{analyse_page, gloss_phrase, word_grammar};
use lingua_core::knowledge::state::KnowledgeState;
use lingua_core::packs::Pack;
use lingua_pack::{build_pack, inputs_from_tables};

/// The pack's bytes, built once for every test of this file.
fn es_fr_pack() -> Pack {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    let bytes = BYTES.get_or_init(|| {
        let tables =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/tables");
        let inputs = inputs_from_tables(&tables, "es-fr").expect("read the committed es-fr tables");
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

    // A letter's name gives no reading of its plural (add-lingua-spanish-word-card D7): `Es` is
    // ser's alone, not also the plural of the letter E, nor `des` the plural of the letter D.
    assert_eq!(
        card(&pack, "Es", "ser"),
        (
            vec!["VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin".to_owned()],
            vec![]
        )
    );
    assert_eq!(
        card(&pack, "des", "dar").0,
        ["VERB|Mood=Sub|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin"]
    );
}

#[test]
fn a_spanish_card_reads_its_french_gloss() {
    let pack = es_fr_pack();
    let gloss = |word: &str| word_grammar(word, word, StudiedLanguage::Spanish, &pack);

    // The French Wiktionary's Spanish entry.
    let heading = |word: &str| gloss(word).senses[0].tag.as_ref().map(|tag| tag.to_ud());
    let casa = gloss("casa");
    assert_eq!(casa.gloss.as_deref(), Some("Maison"));
    // The run carries the noun's gender (add-lingua-spanish-word-card), for « nom féminin ».
    assert_eq!(heading("casa").as_deref(), Some("NOUN|Gender=Fem"));
    // No Spanish entry there: the Spanish Wiktionary's French translation.
    let sector = gloss("sector");
    assert_eq!(sector.gloss.as_deref(), Some("Secteur"));
    assert_eq!(heading("sector").as_deref(), Some("NOUN|Gender=Masc"));
    // A noun of both genders keeps a run without one.
    assert_eq!(heading("estudiante").as_deref(), Some("NOUN"));

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

/// Each token of the page analysis with its class, in document order.
fn classes(pack: &Pack, blocks: &[&str]) -> Vec<(String, TokenClass)> {
    analyse_page(
        blocks,
        StudiedLanguage::Spanish,
        pack,
        &KnowledgeState::default(),
    )
    .tokens
    .into_iter()
    .map(|token| (token.surface, token.class))
    .collect()
}

fn class_of(classes: &[(String, TokenClass)], surface: &str) -> Vec<TokenClass> {
    classes
        .iter()
        .filter(|(seen, _)| seen == surface)
        .map(|(_, class)| *class)
        .collect()
}

#[test]
fn a_spanish_document_sets_its_names_aside() {
    let pack = es_fr_pack();
    // add-lingua-spanish-names: Augusto, Eugenia and la Nela are words of the lexicon with no gloss,
    // written as names throughout. A capital in mid-sentence is the evidence; a block-initial
    // `Augusto` follows it.
    let page = classes(
        &pack,
        &[
            "Augusto miró a Eugenia y pensó en la Nela.",
            "Entonces Augusto salió de la casa.",
        ],
    );
    assert_eq!(
        class_of(&page, "Augusto"),
        [TokenClass::ProperNounOutOfLexicon; 2]
    );
    assert_eq!(
        class_of(&page, "Eugenia"),
        [TokenClass::ProperNounOutOfLexicon]
    );
    assert_eq!(
        class_of(&page, "Nela"),
        [TokenClass::ProperNounOutOfLexicon]
    );
    assert_eq!(class_of(&page, "casa"), [TokenClass::Unknown]);

    // A page long enough to be analysed as Spanish, around the sentence under test.
    const AROUND: &str = "Todos los nobles de la corte miraban la escena con mucha atención.";
    // The same form in lowercase in the document: a word, not a name.
    let page = classes(
        &pack,
        &[
            "El augusto monarca recibió a Augusto en el palacio.",
            AROUND,
        ],
    );
    assert_eq!(class_of(&page, "Augusto"), [TokenClass::Unknown]);
    // A capital at the head of every sentence says nothing.
    let page = classes(&pack, &["Augusto calla. Augusto mira el jardín.", AROUND]);
    assert_eq!(class_of(&page, "Augusto"), [TokenClass::Unknown; 2]);
    // A name the pack glosses stays a word to learn.
    assert!(pack.gloss("dios").is_some());
    let page = classes(&pack, &["Gracias a Dios, dijo, y rezó a Dios.", AROUND]);
    assert_eq!(class_of(&page, "Dios"), [TokenClass::Unknown; 2]);
}

#[test]
fn a_spanish_apocope_reads_as_its_full_word() {
    let pack = es_fr_pack();
    let token = |text: &str| {
        let phrase = gloss_phrase(
            text,
            StudiedLanguage::Spanish,
            &pack,
            &KnowledgeState::default(),
        );
        let first = &phrase.tokens[0];
        (first.lemma.clone(), first.gloss.clone().unwrap_or_default())
    };

    // fix-lingua-spanish-apocopes: `buen` is *bueno* before a noun, and `bueno` no form of `buen`,
    // so the commonest adjective has its gloss.
    for written in ["buen", "bueno", "buenos"] {
        let (lemma, gloss) = token(written);
        assert_eq!(lemma, "bueno", "{written}");
        assert!(gloss.starts_with("Bon"), "{written}: {gloss}");
    }
    let (lemma, gloss) = token("malo");
    assert_eq!(lemma, "malo");
    assert!(gloss.starts_with("Mauvais"), "{gloss}");
    assert_eq!(token("gran").0, "grande");
    assert_eq!(token("algún").0, "alguno");
    // An adverb kaikki calls apocopic stays a word of its own.
    let (lemma, gloss) = token("muy");
    assert_eq!(lemma, "muy");
    assert!(gloss.starts_with("Très"), "{gloss}");
}

#[test]
fn a_spanish_card_shows_no_letter_and_no_surname_for_a_word() {
    let pack = es_fr_pack();
    let lemma = |text: &str| {
        gloss_phrase(
            text,
            StudiedLanguage::Spanish,
            &pack,
            &KnowledgeState::default(),
        )
        .tokens[0]
            .lemma
            .clone()
    };

    // fix-lingua-spanish-card-noise D2: a letter's sense is no gloss.
    assert!(pack.gloss("a").is_some_and(|gloss| gloss.starts_with("À")));
    assert_eq!(pack.gloss("de"), Some("De"));
    // D1: between a proper name and a word, the commoner reading wins. `miró` is mirar's preterite,
    // not the surname, and `dolores` dolor's plural; `argentina` stays the country.
    assert_eq!(lemma("miró"), "mirar");
    assert_eq!(card(&pack, "miró", "mirar").0, [PRETERITE_3SG]);
    assert_eq!(lemma("dolores"), "dolor");
    assert_eq!(lemma("argentina"), "argentina");
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
