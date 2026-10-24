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

//! What an Italian word card says, from the it-fr pack built here from the committed tables
//! (add-lingua-italian-grammar-tables, the Italian programme's change 13, D10): the English
//! Wiktionary's tags as Universal Dependencies tags, one reading per mood, person and number, and the
//! other dictionary forms a form names. The card's Italian — its tense names, the merge of moods — is
//! change 18's; this pins what it reads. Every probe asks a lemma the tables keep: the Italian
//! baseline ran over its hand-written fixture until change 16, so the readings were pinned here, and
//! stay pinned beside it.

use std::path::Path;
use std::sync::OnceLock;

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::engine::word_grammar;
use lingua_core::packs::Pack;
use lingua_core::packs::grammar::Tag;
use lingua_pack::{build_pack, inputs_from_tables};

/// The it-fr pack built from the committed tables, once for every test of this file.
fn pack_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/tables");
        let inputs = inputs_from_tables(&root, "it-fr").expect("the committed it-fr tables");
        build_pack(&inputs).expect("the it-fr pack builds")
    })
}

/// What an Italian word card reads: its own readings, then each other dictionary form it names with
/// that form's readings, every tag in UD notation.
#[derive(Debug, PartialEq)]
struct Card {
    readings: Vec<String>,
    others: Vec<(String, Vec<String>)>,
}

impl Card {
    /// The card of `written` asked as `lemma`.
    fn of(written: &str, lemma: &str) -> Self {
        let pack = Pack::load(pack_bytes()).expect("the it-fr pack loads");
        let grammar = word_grammar(written, lemma, StudiedLanguage::Italian, &pack);
        let notation = |found: &[Tag]| found.iter().map(Tag::to_ud).collect::<Vec<_>>();
        Card {
            readings: notation(&grammar.readings),
            others: grammar
                .others
                .iter()
                .map(|named| (named.lemma.clone(), notation(&named.readings)))
                .collect(),
        }
    }

    /// A card reading `readings` and naming no other dictionary form.
    fn reading(readings: &[&str]) -> Self {
        Card {
            readings: owned(readings),
            others: Vec::new(),
        }
    }

    /// This card, naming `lemma` too, with its readings.
    fn naming(mut self, lemma: &str, readings: &[&str]) -> Self {
        self.others.push((lemma.to_owned(), owned(readings)));
        self
    }
}

fn owned(tags: &[&str]) -> Vec<String> {
    tags.iter().map(ToString::to_string).collect()
}

const PRESENT_3SG: &str = "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin";
const IMPERATIVE_2SG: &str = "VERB|Mood=Imp|Number=Sing|Person=2|VerbForm=Fin";
/// `parli`'s four readings, in the pinned pool's order: the indicative, then the subjunctive
/// (design D7, D8).
const FOUR: [&str; 4] = [
    "VERB|Mood=Ind|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin",
    "VERB|Mood=Sub|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin",
    "VERB|Mood=Sub|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin",
    "VERB|Mood=Sub|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin",
];

#[test]
fn spec_scenario_a_verb_form_says_what_it_is() {
    // The passato remoto is the indicative past, the imperfetto the imperfect.
    assert_eq!(
        Card::of("parlò", "parlare"),
        Card::reading(&["VERB|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin"])
    );
    assert_eq!(
        Card::of("parlavo", "parlare"),
        Card::reading(&["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Imp|VerbForm=Fin"])
    );
}

#[test]
fn spec_scenario_a_present_of_four_readings() {
    // No imperative: the formal one (« parli! », Lei) repeats the subjunctive's third person.
    assert_eq!(Card::of("parli", "parlare"), Card::reading(&FOUR));
}

#[test]
fn spec_scenario_the_conditional_the_imperative_the_gerund_and_the_participles() {
    assert_eq!(
        Card::of("parlerebbe", "parlare"),
        Card::reading(&["VERB|Mood=Cnd|Number=Sing|Person=3|VerbForm=Fin"])
    );
    assert_eq!(
        Card::of("parla", "parlare"),
        Card::reading(&[IMPERATIVE_2SG, PRESENT_3SG])
    );
    assert_eq!(
        Card::of("parlando", "parlare"),
        Card::reading(&["VERB|VerbForm=Ger"])
    );
    // The forms table keeps `parlante` as its own word: its adjective and noun, and parlare's
    // present participle named.
    assert_eq!(
        Card::of("parlante", "parlante"),
        Card::reading(&["ADJ|Number=Sing", "NOUN|Gender=Masc|Number=Sing"])
            .naming("parlare", &["VERB|Number=Sing|Tense=Pres|VerbForm=Part"])
    );
    // `creduta`, the feminine of the participle `creduto`, through it.
    assert_eq!(
        Card::of("creduta", "credere"),
        Card::reading(&["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"])
    );
}

#[test]
fn spec_scenario_the_table_s_stress_marks_and_a_reflexive_row() {
    // The table writes `dièdi`.
    assert_eq!(
        Card::of("diedi", "dare"),
        Card::reading(&["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Past|VerbForm=Fin"])
    );
    // *lavarsi*'s `mi làvo`, filed under *lavare*.
    assert_eq!(
        Card::of("lavo", "lavare"),
        Card::reading(&["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin"])
    );
    // *pentirsi*'s `pentìrsi` and `pèntiti`, read as their bare bases.
    assert_eq!(
        Card::of("pentire", "pentirsi"),
        Card::reading(&["VERB|VerbForm=Inf"])
    );
    let penti = Card::of("penti", "pentirsi");
    assert!(
        penti.readings.iter().any(|tag| tag == IMPERATIVE_2SG),
        "{penti:?}"
    );
}

#[test]
fn spec_scenario_a_noun_says_its_gender() {
    assert_eq!(
        Card::of("case", "casa"),
        Card::reading(&["NOUN|Gender=Fem|Number=Plur"])
    );
    // A noun the dictionary gives one form for, both numbers; an uncountable one, the singular.
    assert_eq!(
        Card::of("città", "città"),
        Card::reading(&["NOUN|Gender=Fem|Number=Plur", "NOUN|Gender=Fem|Number=Sing"])
    );
    assert_eq!(
        Card::of("gas", "gas"),
        Card::reading(&["NOUN|Gender=Masc|Number=Sing"])
    );
    // A plural of another gender.
    assert_eq!(
        Card::of("uova", "uovo"),
        Card::reading(&["NOUN|Gender=Fem|Number=Plur"])
    );
}

#[test]
fn spec_scenario_adjectives_of_four_and_of_two_forms() {
    assert_eq!(
        Card::of("rosse", "rosso"),
        Card::reading(&["ADJ|Gender=Fem|Number=Plur"])
    );
    // A two-form adjective's plural names no gender; *grande* the noun's plural names one.
    assert_eq!(
        Card::of("grandi", "grande"),
        Card::reading(&[
            "ADJ|Number=Plur",
            "NOUN|Gender=Fem|Number=Plur",
            "NOUN|Gender=Masc|Number=Plur"
        ])
    );
    // `minore` names *piccolo*, whose comparative it is.
    assert_eq!(
        Card::of("minore", "minore"),
        Card::reading(&[
            "ADJ|Number=Sing",
            "NOUN|Gender=Fem|Number=Sing",
            "NOUN|Gender=Masc|Number=Sing"
        ])
        .naming("piccolo", &["ADJ|Degree=Cmp|Number=Sing"])
    );
}

#[test]
fn spec_scenario_the_shared_article_table() {
    // An article is a determiner; the table reads under *il*, the lemma the forms table files
    // the articles under.
    assert_eq!(
        Card::of("il", "il"),
        Card::reading(&["DET|Gender=Masc|Number=Sing"])
    );
    assert_eq!(
        Card::of("i", "il"),
        Card::reading(&["DET|Gender=Masc|Number=Plur"])
    );
    assert_eq!(
        Card::of("le", "il"),
        Card::reading(&["DET|Gender=Fem|Number=Plur"])
    );
    // `la` names *lo*, the pronoun, whose feminine it is.
    assert_eq!(
        Card::of("la", "il"),
        Card::reading(&["DET|Gender=Fem|Number=Sing"])
            .naming("lo", &["PRON|Gender=Fem|Number=Sing"])
    );
    // `gli` names *lo*, whose head names it as its plural; `il` names no *lo*.
    assert_eq!(
        Card::of("gli", "il"),
        Card::reading(&["DET|Gender=Masc|Number=Plur"])
            .naming("lo", &["DET|Gender=Masc|Number=Plur"])
    );
}

#[test]
fn spec_scenario_a_homograph_names_its_other_dictionary_form() {
    // `porti`: portare's present, and *porto*'s plural — and *porgere*'s participle (`porto`).
    assert_eq!(
        Card::of("porti", "portare"),
        Card::reading(&FOUR)
            .naming(
                "porgere",
                &["VERB|Gender=Masc|Number=Plur|Tense=Past|VerbForm=Part"]
            )
            .naming("porto", &["NOUN|Gender=Masc|Number=Plur"])
    );
    assert_eq!(
        Card::of("legge", "legge"),
        Card::reading(&["NOUN|Gender=Fem|Number=Sing"]).naming("leggere", &[PRESENT_3SG])
    );
}

#[test]
fn spec_scenario_a_participle_filed_under_a_noun_s_spelling() {
    // The forms table keeps `parlata` as the noun « parlance »: the noun's reading, no verb reading
    // of *parlata*, and *parlare* named, whose feminine past participle it is.
    assert_eq!(
        Card::of("parlata", "parlata"),
        Card::reading(&["NOUN|Gender=Fem|Number=Sing"]).naming(
            "parlare",
            &["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"]
        )
    );
}

#[test]
fn spec_scenario_a_noun_the_forms_table_reads_as_a_verb() {
    // `porta` reads as *portare*; the noun « door », no lemma of the tables (M8), is named nowhere.
    // *porgere* is: its participle `porto` has the feminine `porta`.
    let porta = Card::of("porta", "portare");
    assert_eq!(
        porta,
        Card::reading(&[IMPERATIVE_2SG, PRESENT_3SG]).naming(
            "porgere",
            &["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"]
        )
    );
    assert!(porta.others.iter().all(|(lemma, _)| lemma != "porta"));
}

#[test]
fn change_10_s_spellings_carry_their_word_s_readings() {
    // A dictionary spelling (« misspelling of perché ») and the web's row without its accent.
    assert_eq!(
        Card::of("perchè", "perché"),
        Card::reading(&["NOUN|Gender=Masc|Number=Sing"])
    );
    assert_eq!(Card::of("puo", "potere"), Card::reading(&[PRESENT_3SG]));
}

#[test]
fn spec_scenario_a_dialect_s_table() {
    // *avere*'s Classical Roman Dialect table (`àmo`, `ébbe` a first person) gives no reading: `amo`
    // names *amare* alone, and `ebbe` is the third person singular.
    assert_eq!(
        Card::of("amo", "amo"),
        Card::reading(&["NOUN|Gender=Masc|Number=Sing"]).naming(
            "amare",
            &["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin"]
        )
    );
    assert_eq!(
        Card::of("ebbe", "avere"),
        Card::reading(&["VERB|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin"])
    );
}

#[test]
fn spec_scenario_a_link_the_dictionary_marks_names_no_other_word() {
    // `sono` names no *suonare* (a poetic Latinism), `anno` no *avere* (« misspelling of hanno »),
    // `dire` no *diro* (a literary adjective), and `colli` no *colla* through `colle`, a plural.
    assert_eq!(
        Card::of("sono", "essere"),
        Card::reading(&[
            "VERB|Mood=Ind|Number=Plur|Person=3|Tense=Pres|VerbForm=Fin",
            "VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin"
        ])
    );
    assert_eq!(
        Card::of("anno", "anno"),
        Card::reading(&["NOUN|Gender=Masc|Number=Sing"])
    );
    assert_eq!(
        Card::of("dire", "dire"),
        Card::reading(&["NOUN|Gender=Masc|Number=Sing", "VERB|VerbForm=Inf"])
    );
    assert_eq!(
        Card::of("colli", "collo"),
        Card::reading(&["NOUN|Gender=Masc|Number=Plur"])
            .naming("colle", &["NOUN|Gender=Masc|Number=Plur"])
    );
    // A superlative the head names reads its masculine singular.
    assert_eq!(
        Card::of("bellissimo", "bello"),
        Card::reading(&["ADJ|Degree=Sup|Gender=Masc|Number=Sing"])
    );
}
