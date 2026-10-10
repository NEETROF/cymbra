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

//! What a French word card says, from the fr-en pack built here from the committed tables
//! (add-lingua-french-grammar-tables D11): the English Wiktionary's tags as Universal Dependencies
//! tags, one reading per mood, person and number, and the other dictionary forms a form names. The
//! card's French — its tense names, the merge of moods — is change 51's; this pins what it reads.
//! Every probe asks a lemma the tables keep (design D12).

use std::path::PathBuf;
use std::sync::OnceLock;

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::engine::word_grammar;
use lingua_core::packs::Pack;
use lingua_pack::{build_pack, inputs_from_tables};

/// The pack's bytes, built once for every test of this file.
fn fr_en_pack() -> Pack {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    let bytes = BYTES.get_or_init(|| {
        let tables =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/tables");
        let inputs = inputs_from_tables(&tables, "fr-en").expect("read the committed fr-en tables");
        build_pack(&inputs).expect("build the fr-en pack")
    });
    Pack::load(bytes).expect("load the fr-en pack")
}

/// The other dictionary forms a card names, each with its readings.
type Others = Vec<(String, Vec<String>)>;

/// The card's readings of `written` as `lemma`, and the other dictionary forms with theirs, every
/// tag in UD notation.
fn card(pack: &Pack, written: &str, lemma: &str) -> (Vec<String>, Others) {
    let grammar = word_grammar(written, lemma, StudiedLanguage::French, pack);
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

fn tags(list: &[&str]) -> Vec<String> {
    list.iter().map(|tag| (*tag).to_owned()).collect()
}

/// A card's readings and no other dictionary form.
fn alone(list: &[&str]) -> (Vec<String>, Others) {
    (tags(list), vec![])
}

const PRESENT_3SG: &str = "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin";
/// `parle`'s five readings, in the pinned pool's order: imperative, indicative, subjunctive
/// (design D7, D8).
const FIVE: [&str; 5] = [
    "VERB|Mood=Imp|Number=Sing|Person=2|VerbForm=Fin",
    "VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin",
    PRESENT_3SG,
    "VERB|Mood=Sub|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin",
    "VERB|Mood=Sub|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin",
];

#[test]
fn spec_scenario_a_verb_form_says_what_it_is() {
    let pack = fr_en_pack();
    // The passé simple is the indicative past.
    assert_eq!(
        card(&pack, "fut", "être"),
        alone(&["VERB|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin"])
    );
    assert_eq!(card(&pack, "est", "être"), alone(&[PRESENT_3SG]));
    // `été` is être's past participle; the noun « summer » is no lemma of the tables (M8).
    assert_eq!(
        card(&pack, "été", "être"),
        alone(&["VERB|Gender=Masc|Number=Sing|Tense=Past|VerbForm=Part"])
    );
    // `va` names no *vader*, a Louisiana and Swiss verb (D6).
    assert_eq!(
        card(&pack, "va", "aller"),
        alone(&[
            "VERB|Mood=Imp|Number=Sing|Person=2|VerbForm=Fin",
            PRESENT_3SG
        ])
    );
    // `venait` names no *came*, an English gloss the French section holds as a noun (D5).
    assert_eq!(
        card(&pack, "venait", "venir"),
        alone(&["VERB|Mood=Ind|Number=Sing|Person=3|Tense=Imp|VerbForm=Fin"])
    );
}

#[test]
fn spec_scenario_a_present_of_five_readings() {
    let pack = fr_en_pack();
    assert_eq!(card(&pack, "parle", "parler"), alone(&FIVE));
    // `porte` reads as *porter*, and no reading names the noun, no lemma of the tables (M8).
    assert_eq!(card(&pack, "porte", "porter"), alone(&FIVE));
}

#[test]
fn spec_scenario_the_conditional_and_the_participles() {
    let pack = fr_en_pack();
    assert_eq!(
        card(&pack, "parlerait", "parler"),
        alone(&["VERB|Mood=Cnd|Number=Sing|Person=3|VerbForm=Fin"])
    );
    assert_eq!(
        card(&pack, "parlant", "parler"),
        alone(&["VERB|Tense=Pres|VerbForm=Part"])
    );
    assert_eq!(
        card(&pack, "dirigée", "diriger"),
        alone(&["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"])
    );
    // `faites`, faire's present and imperative, and its feminine plural participle through `fait`.
    assert_eq!(
        card(&pack, "faites", "faire"),
        alone(&[
            "VERB|Gender=Fem|Number=Plur|Tense=Past|VerbForm=Part",
            "VERB|Mood=Imp|Number=Plur|Person=2|VerbForm=Fin",
            "VERB|Mood=Ind|Number=Plur|Person=2|Tense=Pres|VerbForm=Fin",
        ])
    );
    // `fatiguée`, whose verb entry reads « feminine singular of parlé », names no *parler*.
    assert_eq!(
        card(&pack, "fatiguée", "fatiguer"),
        alone(&["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"])
    );
}

#[test]
fn spec_scenario_a_pronominal_verb() {
    let pack = fr_en_pack();
    // The dictionary writes `s'évanouit`; the bare form takes the reading, present and passé simple.
    assert_eq!(
        card(&pack, "évanouit", "évanouir"),
        alone(&[
            "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin",
            PRESENT_3SG,
        ])
    );
    assert_eq!(
        card(&pack, "évanouissaient", "évanouir"),
        alone(&["VERB|Mood=Ind|Number=Plur|Person=3|Tense=Imp|VerbForm=Fin"])
    );
    // `évanouis-toi`'s imperative, among `évanouis`'s readings.
    let (readings, _) = card(&pack, "évanouis", "évanouir");
    assert!(
        readings
            .iter()
            .any(|tag| tag == "VERB|Mood=Imp|Number=Sing|Person=2|VerbForm=Fin"),
        "{readings:?}"
    );
}

#[test]
fn spec_scenario_a_noun_says_its_gender() {
    let pack = fr_en_pack();
    assert_eq!(
        card(&pack, "maisons", "maison"),
        alone(&["NOUN|Gender=Fem|Number=Plur"])
    );
    // A noun the dictionary gives one form for, both numbers.
    assert_eq!(
        card(&pack, "temps", "temps"),
        alone(&[
            "NOUN|Gender=Masc|Number=Plur",
            "NOUN|Gender=Masc|Number=Sing"
        ])
    );
    // An article is a determiner, and `la` is le's as the article and as the pronoun.
    assert_eq!(
        card(&pack, "la", "le"),
        alone(&["DET|Gender=Fem|Number=Sing", "PRON|Gender=Fem|Number=Sing"])
    );
}

#[test]
fn spec_scenario_a_homograph_names_its_other_dictionary_form() {
    let pack = fr_en_pack();
    assert_eq!(
        card(&pack, "couvent", "couvent"),
        (
            tags(&["NOUN|Gender=Masc|Number=Sing"]),
            vec![(
                "couver".to_owned(),
                tags(&[
                    "VERB|Mood=Ind|Number=Plur|Person=3|Tense=Pres|VerbForm=Fin",
                    "VERB|Mood=Sub|Number=Plur|Person=3|Tense=Pres|VerbForm=Fin",
                ])
            )]
        )
    );
    assert_eq!(
        card(&pack, "fils", "fils"),
        (
            tags(&[
                "NOUN|Gender=Masc|Number=Plur",
                "NOUN|Gender=Masc|Number=Sing"
            ]),
            vec![("fil".to_owned(), tags(&["NOUN|Gender=Masc|Number=Plur"]))]
        )
    );
    // `vis`, the screw, is vivre's present and imperative and voir's passé simple.
    assert_eq!(
        card(&pack, "vis", "vis"),
        (
            tags(&["NOUN|Gender=Fem|Number=Plur", "NOUN|Gender=Fem|Number=Sing"]),
            vec![
                (
                    "vivre".to_owned(),
                    tags(&[
                        "VERB|Mood=Imp|Number=Sing|Person=2|VerbForm=Fin",
                        "VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin",
                        "VERB|Mood=Ind|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin",
                    ])
                ),
                (
                    "voir".to_owned(),
                    tags(&[
                        "VERB|Mood=Ind|Number=Sing|Person=1|Tense=Past|VerbForm=Fin",
                        "VERB|Mood=Ind|Number=Sing|Person=2|Tense=Past|VerbForm=Fin",
                    ])
                ),
            ]
        )
    );
}

#[test]
fn spec_scenario_a_participle_filed_under_a_noun_s_spelling() {
    let pack = fr_en_pack();
    // The forms table files `cités` under the noun *cité*: the card reads the noun's plural, no
    // verb reading of *cité*, and names *citer*, whose masculine plural past participle it is.
    assert_eq!(
        card(&pack, "cités", "cité"),
        (
            tags(&["NOUN|Gender=Fem|Number=Plur"]),
            vec![(
                "citer".to_owned(),
                tags(&["VERB|Gender=Masc|Number=Plur|Tense=Past|VerbForm=Part"])
            )]
        )
    );
    // `citée`, which change 43 files under *citer*, reads as its participle and names no noun.
    assert_eq!(
        card(&pack, "citée", "citer"),
        alone(&["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"])
    );
    // Under an adjective's spelling too: `privée` is privé's feminine, and priver's participle.
    assert_eq!(
        card(&pack, "privée", "privé"),
        (
            tags(&["ADJ|Gender=Fem|Number=Sing"]),
            vec![(
                "priver".to_owned(),
                tags(&["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"])
            )]
        )
    );
}
