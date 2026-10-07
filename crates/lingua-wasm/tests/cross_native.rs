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
//! tables, and a second pack over the same studied tables, as a pack glossed in another native
//! language would be (`support/other_native.rs` says how it differs).
//!
//! Every probe of the language's invariance baseline is answered through both packs, and must
//! be byte for byte alike once glosses, senses and expressions — the native side — are removed.
//! The studied sections must be byte-equal. The baselines' own goldens are not read here.
//!
//! Host only: the pack builder is native (C zstd).

#![cfg(not(target_arch = "wasm32"))]

mod support;

use lingua_core::packs::Pack;
use lingua_core::packs::format::read_container;
use lingua_core::packs::pack::section;
use lingua_pack::build_pack;
use support::Scenario;
use support::english::ENGLISH;
use support::other_native::{ENGLISH_IN_SPANISH, SPANISH_IN_ENGLISH};
use support::spanish::SPANISH;

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

/// A golden's probes, by name, in order.
fn probes(text: &str) -> Vec<(String, String)> {
    format!("\n{text}")
        .split("\n### ")
        .skip(1)
        .map(|chunk| {
            let (name, body) = chunk.split_once('\n').unwrap_or((chunk, ""));
            (name.to_owned(), body.to_owned())
        })
        .collect()
}

/// Removes the native side of an output: glosses, their language, their senses, and expressions.
fn strip(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for key in ["gloss", "gloss_language", "senses", "expressions"] {
                map.remove(key);
            }
            map.values_mut().for_each(strip);
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(strip),
        _ => {}
    }
}

/// A probe's output with its native side removed; `None` for the probes that are native or
/// pack identity by definition: the pack line and a card's gloss.
///
/// An engine's native language is its reader's (generalise-lingua-native-language), so the
/// backup records the reader's profile, and writes it in the schema version a profile other
/// than the default needs: both name the reader, not what the pack analyses.
fn studied_side(name: &str, body: &str) -> Option<String> {
    if name == "pack" || name.starts_with("beside ") || name.starts_with("gloss ") {
        return None;
    }
    Some(match serde_json::from_str::<serde_json::Value>(body) {
        Ok(mut value) => {
            strip(&mut value);
            if name == "backup"
                && let Some(backup) = value.as_object_mut()
            {
                backup.remove("profile");
                backup.remove("schema_version");
            }
            value.to_string()
        }
        Err(_) => body.to_owned(),
    })
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
/// side is removed. `native` is the other pack's native language, which labels its cards.
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
    // Every probe but the pack line and the cards' glosses.
    assert_eq!(
        compared,
        a.len() - 1 - scenario.lemmas.len(),
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
    let (reference_inputs, other_inputs) = ENGLISH_IN_SPANISH.inputs();
    let (reference, other) = (
        build_pack(&reference_inputs).expect("en-fr"),
        build_pack(&other_inputs).expect("en-es"),
    );
    let en_es = Pack::load(&other).unwrap();
    assert_eq!(en_es.pair().key(), "en-es", "glossed in Spanish");
    assert_studied_sections_alike("en-fr", &reference, &other);
    assert_probes_alike(
        &ENGLISH,
        None,
        &reference,
        &other,
        ENGLISH_IN_SPANISH.native,
    );

    // *A vocabulary size counts dictionary words*: the universe and B1's typical vocabulary.
    let (universe, typical) = sizes(&reference, "en");
    assert_eq!((universe, typical[2]), (25_372, 3_359), "en-fr, B1");
    assert_eq!(sizes(&other, "en"), (universe, typical));

    // *A sense part of speech the first pack never used*: the core reads the run as NUM.
    let (lemma, _) = other_inputs
        .senses
        .iter()
        .find(|(_, runs)| runs[..] == [("NUM".to_owned(), 1)])
        .expect("a run tagged NUM");
    let runs: Vec<String> = en_es
        .sense_runs(lemma)
        .into_iter()
        .map(|(tag, _)| tag.unwrap().to_ud())
        .collect();
    assert_eq!(runs, ["NUM"], "{lemma}");
}

#[test]
fn spec_scenario_spanish_through_another_native_language() {
    let (reference_inputs, other_inputs) = SPANISH_IN_ENGLISH.inputs();
    let (reference, other) = (
        build_pack(&reference_inputs).expect("es-fr"),
        build_pack(&other_inputs).expect("es-en"),
    );
    assert_studied_sections_alike("es-fr", &reference, &other);
    assert_probes_alike(
        &SPANISH,
        Some("es"),
        &reference,
        &other,
        SPANISH_IN_ENGLISH.native,
    );

    let (es_fr, es_en) = (Pack::load(&reference).unwrap(), Pack::load(&other).unwrap());
    assert_eq!(es_en.pair().key(), "es-en");
    // *A pack glossed in English*: `casa` is one of its dictionary words, and `augusto`, which
    // es-fr does not gloss, is not, though this pack glosses it.
    assert!(es_en.is_dictionary_word("casa"));
    assert!(es_en.gloss("augusto").is_some() && es_fr.gloss("augusto").is_none());
    assert!(!es_en.is_dictionary_word("augusto"));
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

    // *Glosses that say no gender*: the noun runs read the readings' gender.
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
    assert_eq!(heading(&es_en, "casa"), heading(&es_fr, "casa"));
    assert_eq!(heading(&es_en, "casa")[0], "NOUN|Gender=Fem");
    // *A noun of both genders*: no gender.
    assert_eq!(heading(&es_en, "estudiante"), ["NOUN"]);
    // *Fewer sense tags*: no run of the second pack is tagged INTJ, SYM or X.
    assert!(other_inputs.senses.iter().all(|(_, runs)| {
        runs.iter()
            .all(|(tag, _)| !matches!(tag.as_str(), "INTJ" | "SYM" | "X"))
    }));
}
