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

//! The calls the engine refuses (`generalise-lingua-wasm-engine`). They throw, which
//! only a wasm target can show: creating a `JsError` off wasm panics.
//! `wasm-pack test --node crates/lingua-wasm`.

#![cfg(target_arch = "wasm32")]

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::packs::{PackMeta, read_container, write_container};
use lingua_wasm::LinguaEngine;
use wasm_bindgen::{JsError, JsValue};
use wasm_bindgen_test::wasm_bindgen_test;

const PACK: &[u8] = include_bytes!("fixtures/pack.lingua");

/// The fixture pack, its metadata rewritten to study `studied` glossed in `native`
/// (generalise-lingua-native-language): the analyser only reads the forms, so English ones
/// serve here.
fn rewritten(studied: StudiedLanguage, native: &str) -> Vec<u8> {
    let (meta, sections) = read_container(PACK).unwrap();
    let mut meta: PackMeta = serde_json::from_slice(&meta).unwrap();
    meta.studied = studied.tag().into();
    meta.analyzer_version = studied.analyzer_version().into();
    meta.native = native.into();
    let sections: Vec<(&str, &[u8])> = sections
        .iter()
        .map(|s| (s.name.as_str(), s.data.as_slice()))
        .collect();
    write_container(&serde_json::to_vec(&meta).unwrap(), &sections)
}

/// What a refusal says, as the extension reads it (`Error: <message>`, then its stack).
fn message(error: JsError) -> String {
    format!("{:?}", JsValue::from(error))
}

#[wasm_bindgen_test]
fn spec_scenario_a_language_without_a_pack_is_refused() {
    let mut engine = LinguaEngine::new(PACK).unwrap();
    assert!(
        engine
            .analyse(vec!["Hola".to_owned()], Some("es".to_owned()))
            .is_err()
    );
    assert!(
        engine
            .set_status("haber", "known", Some("es".to_owned()))
            .is_err()
    );
    assert!(
        engine
            .frequency_rank("haber", Some("es".to_owned()))
            .is_err()
    );
    assert!(
        engine
            .analyse(vec!["Olá".to_owned()], Some("pt".to_owned()))
            .is_err()
    );
    assert_eq!(
        engine.export_status_ops(),
        "[]",
        "a refused call changes nothing"
    );
}

#[wasm_bindgen_test]
fn spec_scenario_an_invalid_choice_of_studied_languages_is_refused() {
    let mut engine = LinguaEngine::new(PACK).unwrap();
    engine
        .set_studied_languages(vec!["es".to_owned(), "en".to_owned()])
        .unwrap();
    assert!(engine.set_studied_languages(vec![]).is_err());
    assert!(
        engine
            .set_studied_languages(vec!["en".to_owned(), "en".to_owned()])
            .is_err()
    );
    assert!(engine.set_studied_languages(vec!["pt".to_owned()]).is_err());
    assert_eq!(
        engine.studied_languages(),
        r#"["es","en"]"#,
        "a refused choice leaves the current one"
    );
}

#[wasm_bindgen_test]
fn spec_scenario_a_detection_needs_known_candidates() {
    let engine = LinguaEngine::new(PACK).unwrap();
    assert!(
        engine
            .detect_language(vec!["Hola".to_owned()], vec![], None)
            .is_err()
    );
    assert!(
        engine
            .detect_language(vec!["Olá".to_owned()], vec!["pt".to_owned()], None)
            .is_err()
    );
}

#[wasm_bindgen_test]
fn spec_scenario_one_pack_per_language() {
    let mut engine = LinguaEngine::new(PACK).unwrap();
    assert!(engine.add_pack(PACK).is_err());
    assert_eq!(engine.languages(), r#"["en"]"#);
}

#[wasm_bindgen_test]
fn spec_scenario_a_pack_of_another_native_language_is_refused() {
    let mut engine = LinguaEngine::new(PACK).unwrap();
    let backup = engine.backup();
    let Err(refused) = engine.add_pack(&rewritten(StudiedLanguage::Spanish, "en")) else {
        panic!("a pack glossed in English was added to an engine serving French");
    };
    // The error names both languages.
    let said = message(refused);
    assert!(said.contains("\"fr\"") && said.contains("\"en\""), "{said}");
    assert_eq!(engine.languages(), r#"["en"]"#, "the packs held stay");
    assert_eq!(engine.native_language(), "fr");
    assert_eq!(engine.backup(), backup, "the profile stays");
    // The same pair glossed in French is added.
    assert_eq!(
        engine
            .add_pack(&rewritten(StudiedLanguage::Spanish, "fr"))
            .unwrap(),
        "es"
    );
    // A pack glossed in a language the core does not know, or in the one it studies, never loads.
    assert!(LinguaEngine::new(&rewritten(StudiedLanguage::English, "de")).is_err());
    assert!(LinguaEngine::new(&rewritten(StudiedLanguage::English, "en")).is_err());
}

#[wasm_bindgen_test]
fn spec_scenario_a_native_language_no_pack_serves_is_refused() {
    let mut engine = LinguaEngine::new(PACK).unwrap();
    engine
        .add_pack(&rewritten(StudiedLanguage::Spanish, "fr"))
        .unwrap();
    engine
        .set_studied_languages(vec!["es".to_owned(), "en".to_owned()])
        .unwrap();
    let backup = engine.backup();
    let Err(refused) = engine.set_profile("en", vec!["es".to_owned()]) else {
        panic!("a native language no pack serves was set");
    };
    assert!(message(refused).contains("no pack the engine holds is glossed in \"en\""));
    assert!(engine.set_profile("de", vec!["es".to_owned()]).is_err());
    assert!(engine.set_profile("fr", vec!["pt".to_owned()]).is_err());
    assert!(engine.set_profile("fr", vec![]).is_err());
    assert_eq!(engine.profile_native_language(), "fr");
    assert_eq!(engine.studied_languages(), r#"["es","en"]"#);
    assert_eq!(engine.backup(), backup, "a refused profile changes nothing");
}

#[wasm_bindgen_test]
fn spec_scenario_the_native_language_is_never_studied() {
    let mut engine = LinguaEngine::new(&rewritten(StudiedLanguage::Spanish, "en")).unwrap();
    assert_eq!(engine.native_language(), "en");
    assert!(
        engine
            .set_studied_languages(vec!["es".to_owned(), "en".to_owned()])
            .is_err()
    );
    assert!(
        engine
            .set_profile("en", vec!["en".to_owned(), "es".to_owned()])
            .is_err()
    );
    assert_eq!(engine.studied_languages(), r#"["es"]"#);
    assert_eq!(engine.profile_native_language(), "en");
}

#[wasm_bindgen_test]
fn spec_scenario_a_french_native_reader_cannot_study_french() {
    // add-lingua-french-baseline: French is a studied language now, and never a French reader's.
    let mut engine = LinguaEngine::new(PACK).unwrap();
    assert_eq!(engine.native_language(), "fr");
    let backup = engine.backup();
    let Err(refused) = engine.set_profile("fr", vec!["fr".to_owned()]) else {
        panic!("a French-native reader was set to study French");
    };
    assert!(message(refused).contains("\"fr\" is the native language"));
    assert!(
        engine
            .set_studied_languages(vec!["en".to_owned(), "fr".to_owned()])
            .is_err()
    );
    assert_eq!(engine.studied_languages(), r#"["en"]"#);
    assert_eq!(engine.backup(), backup, "a refused profile changes nothing");
}
