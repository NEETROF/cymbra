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

//! Cross-native invariance (add-lingua-pack-lexical-layer D6): an analysis does not depend
//! on the native language its pack is glossed in.
//!
//! For English and for Spanish, the reference pack (en-fr, es-fr) is built from its committed
//! tables, and the second pack is the real one over the same studied tables: en-es, the committed
//! pair glossed in Spanish (add-lingua-pack-en-es), built from `tables/en/` and `tables/en-es/`;
//! es-en, the committed pair glossed in English (add-lingua-pack-es-en), built from `tables/es/`
//! and `tables/es-en/`.
//!
//! Every probe of the language's invariance baseline is answered through both packs, and must
//! be byte for byte alike once glosses, senses and expressions — the native side — are removed
//! (`support::studied_side`, which `es_en_baseline.rs` and `en_es_baseline.rs` also compare the
//! committed goldens through). The studied sections must be byte-equal. The baselines' own
//! goldens are not read here.
//!
//! Host only: the pack builder is native (C zstd).

#![cfg(not(target_arch = "wasm32"))]

mod support;

use lingua_core::packs::Pack;
use lingua_core::packs::format::read_container;
use lingua_core::packs::pack::section;
use lingua_pack::{build_pack, inputs_from_tables};
use support::english::ENGLISH;
use support::spanish::SPANISH;
use support::{Scenario, probes, studied_side};

fn sections(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let (_, sections) = read_container(bytes).expect("a pack");
    sections.into_iter().map(|s| (s.name, s.data)).collect()
}

fn section_of<'a>(sections: &'a [(String, Vec<u8>)], name: &str) -> Option<&'a [u8]> {
    sections
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, data)| data.as_slice())
}

/// The studied sections are byte-equal, the tag pool up to the pinned prefix and the
/// readings' tags; the second pack carries a lexical table, the reference none.
fn assert_studied_sections_alike(pair: &str, reference: &[u8], other: &[u8]) {
    let (a, b) = (sections(reference), sections(other));
    for name in [
        section::FORMS,
        section::LEMMAS,
        section::FREQ,
        section::LEVELS,
        section::PARADIGMS_ZST,
    ] {
        assert!(section_of(&a, name).is_some(), "{pair} carries {name}");
        assert!(
            section_of(&a, name) == section_of(&b, name),
            "{pair}: the {name} section depends on the native language"
        );
    }
    let pool = |s: &[(String, Vec<u8>)]| -> Vec<String> {
        String::from_utf8(section_of(s, section::TAGS).expect("tags").to_vec())
            .unwrap()
            .split('\n')
            .map(str::to_owned)
            .collect()
    };
    let (pinned, grown) = (pool(&a), pool(&b));
    assert_eq!(grown[..pinned.len()], pinned[..], "{pair}: the pinned pool");
    assert!(
        grown[pinned.len()..].iter().all(|tag| !tag.contains('|')),
        "{pair}: only sense parts of speech follow the pin: {:?}",
        &grown[pinned.len()..]
    );
    assert!(section_of(&a, section::LEXICAL).is_none(), "{pair}");
    let lemmas = section_of(&a, section::LEMMAS)
        .unwrap()
        .split(|&b| b == b'\n')
        .count();
    assert_eq!(
        section_of(&b, section::LEXICAL).map(<[u8]>::len),
        Some(lemmas.div_ceil(8)),
        "{pair}: the second pack's lexical table"
    );
}

/// The exported card operations say the language of their glosses before it is stripped: the
/// reference's, French, carry no label; the other pack's every one carry its native's
/// (add-lingua-card-gloss-language D2) — so a label astray elsewhere is not hidden by the
/// strip.
fn assert_card_ops_labelled(reference: &str, other: &str, native: &str) {
    let ops = |body: &str| -> Vec<serde_json::Value> {
        serde_json::from_str(body).expect("an array of card operations")
    };
    let (a, b) = (ops(reference), ops(other));
    assert!(!a.is_empty() && a.len() == b.len(), "the same cards");
    for op in &a {
        assert!(
            op.get("gloss_language").is_none(),
            "a French gloss carries no label: {op}"
        );
    }
    for op in &b {
        assert_eq!(op["gloss_language"], native, "{op}");
    }
}

/// Every probe of `scenario`, through `reference` and through `other`: alike once the native
/// side is removed — the other pack's own credits included. `native` is the other pack's native
/// language, which labels its cards.
fn assert_probes_alike(
    scenario: &Scenario,
    language: Option<&str>,
    reference: &[u8],
    other: &[u8],
    native: &str,
) {
    let render = |pack: &[u8]| {
        // The scenario's pair labels the pack line, which is not compared.
        scenario.render_with(vec![(scenario.pair, pack.to_vec())], language)
    };
    let (a, b) = (probes(&render(reference)), probes(&render(other)));
    assert_eq!(
        a.iter().map(|(n, _)| n).collect::<Vec<_>>(),
        b.iter().map(|(n, _)| n).collect::<Vec<_>>(),
        "the same probes"
    );
    let mut compared = 0;
    for ((name, x), (_, y)) in a.iter().zip(&b) {
        if name == "export-card-ops" {
            assert_card_ops_labelled(x, y, native);
        }
        let (Some(x), Some(y)) = (studied_side(name, x), studied_side(name, y)) else {
            continue;
        };
        compared += 1;
        let at = x.chars().zip(y.chars()).take_while(|(p, q)| p == q).count();
        assert!(
            x == y,
            "{}: probe `{name}` depends on the native language, character {at}:\n  \
             reference: …{}…\n  other:     …{}…",
            scenario.pair,
            x.chars()
                .skip(at.saturating_sub(80))
                .take(240)
                .collect::<String>(),
            y.chars()
                .skip(at.saturating_sub(80))
                .take(240)
                .collect::<String>(),
        );
    }
    // Every probe but the pack line, the cards' glosses and — a pack's own credits — the notice
    // and the licences.
    assert_eq!(
        compared,
        a.len() - 1 - 2 - scenario.lemmas.len(),
        "probes compared"
    );
}

/// The estimate's universe and each level's typical vocabulary, through an engine.
fn sizes(pack: &[u8], language: &str) -> (u64, Vec<u64>) {
    let engine = Scenario::engine(&[("pack", pack.to_vec())]);
    let lang = || Some(language.to_owned());
    let estimate: serde_json::Value =
        serde_json::from_str(&engine.vocabulary_estimate(lang()).unwrap()).unwrap();
    let ladder: serde_json::Value =
        serde_json::from_str(&engine.level_ladder(lang()).unwrap()).unwrap();
    let typical = ladder
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["typicalVocabulary"].as_u64().unwrap())
        .collect();
    (estimate["universe"].as_u64().unwrap(), typical)
}

#[test]
fn spec_scenario_english_through_another_native_language() {
    // en-fr, and en-es: the committed pair glossed in Spanish (add-lingua-pack-en-es), both
    // built from tables/en/ — the real second pack, not a synthetic one.
    let root = Scenario::tables_root();
    let (reference_inputs, other_inputs) = (
        inputs_from_tables(&root, "en-fr").expect("read en-fr"),
        inputs_from_tables(&root, "en-es").expect("read en-es"),
    );
    assert_eq!(
        other_inputs.meta.native, "es",
        "en-es is glossed in Spanish"
    );
    // *Glossed in Spanish with fewer glosses*.
    assert!(other_inputs.glosses.len() < reference_inputs.glosses.len());
    let (reference, other) = (
        build_pack(&reference_inputs).expect("en-fr"),
        build_pack(&other_inputs).expect("en-es"),
    );
    let (en_fr, en_es) = (Pack::load(&reference).unwrap(), Pack::load(&other).unwrap());
    assert_eq!(en_es.pair().key(), "en-es", "glossed in Spanish");
    assert_studied_sections_alike("en-fr", &reference, &other);
    assert_probes_alike(&ENGLISH, None, &reference, &other, "es");
    // Fewer glosses, yet glosses of its own: a lemma this pack glosses and en-fr does not is no
    // dictionary word — the dictionary words are en-fr's, read from tables/en (the lexical
    // section `assert_studied_sections_alike` found).
    let only_here: Vec<&str> = other_inputs
        .glosses
        .iter()
        .map(|(lemma, _)| lemma.as_str())
        .filter(|lemma| en_fr.gloss(lemma).is_none())
        .take(8)
        .collect();
    assert!(!only_here.is_empty(), "en-es glosses lemmas en-fr does not");
    for lemma in &only_here {
        assert!(
            en_es.gloss(lemma).is_some() && !en_es.is_dictionary_word(lemma),
            "{lemma}"
        );
    }

    // *A vocabulary size counts dictionary words*: the universe and B1's typical vocabulary.
    let (universe, typical) = sizes(&reference, "en");
    assert_eq!((universe, typical[2]), (25_372, 3_359), "en-fr, B1");
    assert_eq!(sizes(&other, "en"), (universe, typical));

    // *A sense part of speech the first pack never used*: no en-fr run is tagged NUM; en-es tags
    // the numbers' runs NUM (the Spanish Wiktionary's numeral sections, kaikki's `num`), and the
    // core reads a numeral's runs as the table writes them — « three », a number and a noun.
    assert!(
        reference_inputs
            .senses
            .iter()
            .all(|(_, runs)| runs.iter().all(|(tag, _)| tag != "NUM"))
    );
    let (_, runs) = other_inputs
        .senses
        .iter()
        .find(|(lemma, _)| lemma == "three")
        .expect("three's sense runs");
    let tags: Vec<&str> = runs.iter().map(|(tag, _)| tag.as_str()).collect();
    assert_eq!(tags, ["NUM", "NOUN"], "three, in tables/en-es/senses.tsv");
    let read: Vec<String> = en_es
        .sense_runs("three")
        .into_iter()
        .map(|(tag, _)| tag.unwrap().to_ud())
        .collect();
    assert_eq!(read, tags, "three, as the core reads it");
}

#[test]
fn spec_scenario_spanish_through_another_native_language() {
    // es-fr, and es-en: the committed pair glossed in English (add-lingua-pack-es-en), both
    // built from tables/es/ — the real second pack, not a synthetic one.
    let root = Scenario::tables_root();
    let (reference_inputs, other_inputs) = (
        inputs_from_tables(&root, "es-fr").expect("read es-fr"),
        inputs_from_tables(&root, "es-en").expect("read es-en"),
    );
    assert_eq!(
        other_inputs.meta.native, "en",
        "es-en is glossed in English"
    );
    let (reference, other) = (
        build_pack(&reference_inputs).expect("es-fr"),
        build_pack(&other_inputs).expect("es-en"),
    );
    assert_studied_sections_alike("es-fr", &reference, &other);
    assert_probes_alike(&SPANISH, Some("es"), &reference, &other, "en");

    let (es_fr, es_en) = (Pack::load(&reference).unwrap(), Pack::load(&other).unwrap());
    assert_eq!(es_en.pair().key(), "es-en");
    // *A pack glossed in English*: `casa` is one of its dictionary words, glossed in English,
    // not as es-fr glosses it.
    assert!(es_en.is_dictionary_word("casa"));
    assert_ne!(es_en.gloss("casa"), es_fr.gloss("casa"));
    assert!(es_en.gloss("casa").is_some_and(|g| g.contains("house")));
    // A lemma this pack glosses and es-fr does not is no dictionary word: the dictionary words
    // are es-fr's, read from tables/es (the lexical section `assert_studied_sections_alike`
    // found).
    let only_here: Vec<&str> = other_inputs
        .glosses
        .iter()
        .map(|(lemma, _)| lemma.as_str())
        .filter(|lemma| es_fr.gloss(lemma).is_none())
        .take(8)
        .collect();
    assert!(!only_here.is_empty(), "es-en glosses lemmas es-fr does not");
    for lemma in &only_here {
        assert!(
            es_en.gloss(lemma).is_some() && !es_en.is_dictionary_word(lemma),
            "{lemma}"
        );
    }
    // *Spanish glossed in another language*: the estimate's universe is es-fr's.
    assert_eq!(sizes(&reference, "es").0, 22_755);
    assert_eq!(sizes(&other, "es").0, 22_755);

    // *A pack glossed in English* (names): Augusto and Eugenia are set aside as with es-fr.
    let names = |pack: &[u8]| -> Vec<String> {
        let engine = Scenario::engine(&[("pack", pack.to_vec())]);
        let page: serde_json::Value = serde_json::from_str(
            &engine
                .analyse(
                    vec![
                        "Augusto miró a Eugenia y pensó en la Nela.".to_owned(),
                        "Entonces Augusto salió de la casa.".to_owned(),
                    ],
                    Some("es".to_owned()),
                )
                .unwrap(),
        )
        .unwrap();
        page["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| matches!(t["surface"].as_str(), Some("Augusto" | "Eugenia")))
            .map(|t| t["class"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(names(&reference), ["ProperNounOutOfLexicon"; 3]);
    assert_eq!(names(&other), names(&reference));

    // *Glosses that say no gender*: es-en's sense table carries none (its reducer computes
    // nothing of the studied side, D1), and the noun runs read the readings' gender.
    let heading = |pack: &Pack, lemma: &str| -> Vec<String> {
        pack.sense_runs(lemma)
            .into_iter()
            .map(|(tag, _)| tag.unwrap().to_ud())
            .collect()
    };
    assert!(
        other_inputs
            .senses
            .iter()
            .all(|(_, runs)| runs.iter().all(|(tag, _)| !tag.contains("Gender")))
    );
    assert_eq!(heading(&es_en, "casa")[0], "NOUN|Gender=Fem");
    assert_eq!(heading(&es_fr, "casa")[0], "NOUN|Gender=Fem");
    // *A noun of both genders*: no gender.
    assert_eq!(heading(&es_en, "estudiante")[0], "NOUN");
    assert_eq!(heading(&es_fr, "estudiante")[0], "NOUN");
    // The parts of speech of es-en's runs are the English Wiktionary's, laid out after the
    // pinned pool (`assert_studied_sections_alike`); every one is a Universal Dependencies tag.
    assert!(other_inputs.senses.iter().all(|(_, runs)| {
        runs.iter()
            .all(|(tag, _)| tag.chars().all(|c| c.is_ascii_uppercase()))
    }));
}
