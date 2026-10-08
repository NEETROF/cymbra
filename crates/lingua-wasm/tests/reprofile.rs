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

//! The reader chooses their native language (add-lingua-native-language-choice D2): the store's
//! owner rewrites the backup's profile with `reprofileBackup`, a pure function, and an engine of
//! the new native language restores it. Success paths only, on the host: a refusal creates a
//! `JsError`, which panics off wasm — the refusal logic is unit-tested in `src/lib.rs`, the binding's
//! refusal in `tests/languages_wasm.rs`.

#![cfg(not(target_arch = "wasm32"))]

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::packs::{PackMeta, read_container, write_container};
use lingua_wasm::{LinguaEngine, reprofile_backup};

const PACK: &[u8] = include_bytes!("fixtures/pack.lingua");
/// 2026-09-21T13:46:40Z, in milliseconds (status bindings take them).
const T_MS: f64 = 1_790_000_000_000.0;

/// The fixture pack, its metadata rewritten to study `studied` glossed in `native`: the analyser
/// only reads the forms, so English ones serve here.
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

fn tags(tags: &[&str]) -> Vec<String> {
    tags.iter().map(|tag| (*tag).to_owned()).collect()
}

/// A French-native reader of English and Spanish, as an engine on en-fr and es-fr holds one.
fn french_reader_of_two() -> LinguaEngine {
    let mut engine = LinguaEngine::new(PACK).unwrap();
    engine
        .add_pack(&rewritten(StudiedLanguage::Spanish, "fr"))
        .unwrap();
    engine.set_studied_languages(tags(&["en", "es"])).unwrap();
    engine
        .set_status_at("city", "known", T_MS, Some("es".to_owned()))
        .unwrap();
    engine
}

#[test]
fn spec_scenario_a_reprofiled_backup_restores_on_an_engine_of_the_new_native() {
    let french = french_reader_of_two();
    let reprofiled = reprofile_backup(&french.backup(), "en", tags(&["es"])).unwrap();

    // An engine of the new native language, as every port rebuilds for it (D3).
    let mut english = LinguaEngine::new(&rewritten(StudiedLanguage::Spanish, "en")).unwrap();
    english.restore(&reprofiled).unwrap();
    assert_eq!(english.native_language(), "en");
    assert_eq!(english.profile_native_language(), "en");
    assert_eq!(english.studied_languages(), r#"["es"]"#);
    // The reader's records stay: the Spanish status set as a French reader is there.
    let ops: serde_json::Value = serde_json::from_str(&english.export_status_ops()).unwrap();
    assert_eq!(ops[0]["lemma"], "city");
    assert_eq!(ops[0]["language"], "es");
    assert_eq!(
        english.backup(),
        reprofiled,
        "the engine restores the backup whole"
    );
}

#[test]
fn spec_scenario_a_full_reset_keeps_the_native_language() {
    // An English-native reader erases their data: the fresh state is English-native, studying the
    // engine's first pack's language — the first shipped pair's of English (M3).
    let reprofiled =
        reprofile_backup(&french_reader_of_two().backup(), "en", tags(&["es"])).unwrap();
    let mut english = LinguaEngine::new(&rewritten(StudiedLanguage::Spanish, "en")).unwrap();
    english.restore(&reprofiled).unwrap();
    english.reset();
    assert_eq!(english.profile_native_language(), "en");
    assert_eq!(english.studied_languages(), r#"["es"]"#);
    assert_eq!(english.export_status_ops(), "[]");
}

#[test]
fn a_reader_returned_to_french_is_written_as_every_installed_reader_is() {
    let english = LinguaEngine::new(&rewritten(StudiedLanguage::Spanish, "en")).unwrap();
    let french = reprofile_backup(&english.backup(), "fr", tags(&["en"])).unwrap();
    assert_eq!(french, LinguaEngine::new(PACK).unwrap().backup());
    assert!(french.starts_with("{\n  \"schema_version\": 1,"));
}
