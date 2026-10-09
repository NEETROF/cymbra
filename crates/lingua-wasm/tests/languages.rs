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

//! One engine, a pack per studied language (`generalise-lingua-wasm-engine`): an engine
//! built from the testdata en-fr pack, plus a small Spanish pack built here with the
//! pack builder.
//!
//! Success paths only. Creating a `JsError` off wasm panics, so the refused calls run
//! in `tests/languages_wasm.rs` under `wasm-pack test`, and the refusal logic itself is
//! unit-tested in `lingua_core::packs::set`.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::knowledge::level::CefrLevel;
use lingua_core::knowledge::vocabulary::{ENGLISH_TYPICAL_VOCABULARY, level_vocabulary};
use lingua_core::packs::{Pack, PackMeta, read_container, write_container};
use lingua_wasm::LinguaEngine;

/// 2026-09-21T13:46:40Z: deck bindings take epoch seconds, status and level bindings
/// take milliseconds.
const T_SECS: f64 = 1_790_000_000.0;
const T_MS: f64 = T_SECS * 1000.0;

const SPANISH: &str =
    "Los equipos nunca entregan el viernes, y tú has trabajado toda la semana por la noche.";
const ENGLISH: &str = "The city runs seldom, and the conundrum went on for a very long time.";

fn testdata() -> lingua_pack::PackInputs {
    let dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/testdata/en-fr");
    lingua_pack::inputs_from_dir(&dir).unwrap_or_else(|e| panic!("read the testdata tables: {e}"))
}

fn english_pack() -> Vec<u8> {
    lingua_pack::build_pack(&testdata()).unwrap_or_else(|e| panic!("build the en pack: {e}"))
}

/// A small es→fr pack: the testdata's sources and notice, so the licence checks pass,
/// with Spanish forms, ranks and glosses, and no CEFR levels.
fn spanish_pack() -> Vec<u8> {
    lingua_pack::build_pack(&spanish_inputs()).unwrap_or_else(|e| panic!("build the es pack: {e}"))
}

fn spanish_inputs() -> lingua_pack::PackInputs {
    let mut inputs = testdata();
    inputs.meta.studied = "es".into();
    inputs.meta.analyzer_version = StudiedLanguage::Spanish.analyzer_version().into();
    inputs.meta.pack_version = "0.0.1-es-test".into();
    inputs.form_lemma = vec![
        ("has".into(), "haber".into()),
        ("equipos".into(), "equipo".into()),
        ("trabajado".into(), "trabajar".into()),
    ];
    inputs.ranks = vec![
        ("haber".into(), 20),
        ("trabajar".into(), 300),
        ("equipo".into(), 900),
    ];
    inputs.glosses = vec![
        ("haber".into(), "Avoir (auxiliaire)".into()),
        ("equipo".into(), "Équipe".into()),
    ];
    inputs.levels = Vec::new();
    inputs.expressions = Vec::new();
    inputs.readings = Vec::new();
    inputs.senses = Vec::new();
    inputs.notice = format!("{}\nSpanish test pack.", inputs.notice);
    inputs
}

fn testdata_of(pair: &str) -> lingua_pack::PackInputs {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/lingua-data/testdata")
        .join(pair);
    lingua_pack::inputs_from_dir(&dir)
        .unwrap_or_else(|e| panic!("read the {pair} testdata tables: {e}"))
}

/// A small fr→en pack: the es-en testdata's sources and notice, re-stamped `fr` as the Spanish
/// one is, with French forms, ranks and glosses, at French's analyser version
/// (add-lingua-french-baseline).
fn french_pack() -> Vec<u8> {
    let mut inputs = testdata_of("es-en");
    inputs.meta.studied = "fr".into();
    inputs.meta.analyzer_version = StudiedLanguage::French.analyzer_version().into();
    inputs.meta.pack_version = "0.0.1-fr-test".into();
    inputs.meta.levels_estimated = false;
    inputs.form_lemma = vec![
        ("est".into(), "être".into()),
        ("maisons".into(), "maison".into()),
    ];
    inputs.ranks = vec![("être".into(), 3), ("maison".into(), 400)];
    inputs.glosses = vec![("maison".into(), "house".into())];
    inputs.levels = Vec::new();
    inputs.expressions = Vec::new();
    inputs.readings = Vec::new();
    inputs.senses = Vec::new();
    inputs.notice = format!("{}\nFrench test pack.", inputs.notice);
    lingua_pack::build_pack(&inputs).unwrap_or_else(|e| panic!("build the fr pack: {e}"))
}

/// An English-native reader's engine: the testdata es-en pack, the small French one added.
fn english_native_engine() -> LinguaEngine {
    let es_en = lingua_pack::build_pack(&testdata_of("es-en"))
        .unwrap_or_else(|e| panic!("build the es-en pack: {e}"));
    let mut engine = LinguaEngine::new(&es_en).unwrap();
    assert_eq!(engine.add_pack(&french_pack()).unwrap(), "fr");
    engine
}

/// `pack` with its metadata rewritten to name `native` as the language of its glosses.
fn glossed_in(pack: &[u8], native: &str) -> Vec<u8> {
    let (meta, sections) = read_container(pack).unwrap_or_else(|e| panic!("a container: {e}"));
    let mut meta: PackMeta = serde_json::from_slice(&meta).unwrap();
    meta.native = native.into();
    let sections: Vec<(&str, &[u8])> = sections
        .iter()
        .map(|s| (s.name.as_str(), s.data.as_slice()))
        .collect();
    write_container(&serde_json::to_vec(&meta).unwrap(), &sections)
}

/// The small Spanish pack, glossed in English.
fn spanish_for_english_pack() -> Vec<u8> {
    glossed_in(&spanish_pack(), "en")
}

fn english_engine() -> LinguaEngine {
    LinguaEngine::new(&english_pack()).unwrap()
}

fn two_language_engine() -> LinguaEngine {
    let mut engine = english_engine();
    assert_eq!(engine.add_pack(&spanish_pack()).unwrap(), "es");
    engine
}

fn es() -> Option<String> {
    Some("es".to_owned())
}

/// The ladder's rows of `engine` in `language`.
fn ladder(engine: &LinguaEngine, language: Option<String>) -> Vec<serde_json::Value> {
    serde_json::from_str(&engine.level_ladder(language).unwrap()).unwrap()
}

/// The small Spanish pack with estimated levels.
fn estimated_spanish_pack() -> Vec<u8> {
    let mut inputs = spanish_inputs();
    inputs.levels = vec![
        ("haber".into(), CefrLevel::A1),
        ("trabajar".into(), CefrLevel::A2),
        ("equipo".into(), CefrLevel::B1),
    ];
    inputs.meta.levels_estimated = true;
    lingua_pack::build_pack(&inputs).unwrap()
}

/// A pack whose levels are estimated from frequency borrows English's typical
/// vocabularies for its ladder, as the core freezes them, and says so; English's own ladder
/// computes its own figures (`fix-lingua-spanish-ladder-estimates`,
/// `generalise-lingua-native-language` D6).
#[test]
fn an_estimated_ladder_borrows_english_typical_vocabularies() {
    let mut engine = english_engine();
    engine.add_pack(&estimated_spanish_pack()).unwrap();

    let spanish = ladder(&engine, es());
    assert_eq!(spanish.len(), ENGLISH_TYPICAL_VOCABULARY.len());
    for (row, typical) in spanish.iter().zip(ENGLISH_TYPICAL_VOCABULARY) {
        assert_eq!(row["typicalFrom"], "en");
        assert_eq!(row["typicalVocabulary"], typical);
    }
    // Its own levels' counts stay its own.
    assert_eq!(spanish[0]["total"], 1);

    // English's rows are the testdata pack's own figures, never the frozen ones.
    let pack = Pack::load(&english_pack()).unwrap();
    let words = pack.dictionary_words();
    let own = CefrLevel::ALL.map(|level| level_vocabulary(level, words.iter().copied(), &pack));
    assert_ne!(own, ENGLISH_TYPICAL_VOCABULARY);
    let english = ladder(&engine, None);
    assert!(english.iter().all(|row| row.get("typicalFrom").is_none()));
    for (row, typical) in english.iter().zip(own) {
        assert_eq!(row["typicalVocabulary"], typical);
    }
}

/// An engine that holds no English pack — an English reader's — shows the same figures.
#[test]
fn spec_scenario_an_estimated_ladder_without_an_english_pack() {
    let engine = LinguaEngine::new(&estimated_spanish_pack()).unwrap();
    assert_eq!(engine.languages(), r#"["es"]"#);
    let alone = ladder(&engine, None);
    let mut beside = english_engine();
    beside.add_pack(&estimated_spanish_pack()).unwrap();
    assert_eq!(alone, ladder(&beside, es()));
    for (row, typical) in alone.iter().zip(ENGLISH_TYPICAL_VOCABULARY) {
        assert_eq!(row["typicalFrom"], "en");
        assert_eq!(row["typicalVocabulary"], typical);
    }

    let english_reader = LinguaEngine::new(&glossed_in(&estimated_spanish_pack(), "en")).unwrap();
    assert_eq!(ladder(&english_reader, None), alone);
}

/// Every engine today: built on en-fr, Spanish added, French for the engine and the reader
/// (`generalise-lingua-native-language`).
#[test]
fn spec_scenario_every_engine_today_serves_french() {
    let engine = two_language_engine();
    assert_eq!(engine.languages(), r#"["en","es"]"#);
    assert_eq!(engine.native_language(), "fr");
    assert_eq!(engine.profile_native_language(), "fr");
    assert_eq!(engine.studied_languages(), r#"["en"]"#);
    assert!(engine.backup().starts_with("{\n  \"schema_version\": 1,"));
}

#[test]
fn a_profile_set_whole_round_trips_through_the_backup() {
    let mut engine = english_engine();
    engine
        .set_profile("fr", vec!["es".to_owned(), "en".to_owned()])
        .unwrap();
    assert_eq!(engine.studied_languages(), r#"["es","en"]"#);
    let backup = engine.backup();
    assert!(backup.starts_with("{\n  \"schema_version\": 2,"));
    assert!(
        backup.contains("\"native_language\": \"French\""),
        "{backup}"
    );

    let mut restored = english_engine();
    restored.restore(&backup).unwrap();
    assert_eq!(restored.studied_languages(), r#"["es","en"]"#);
    assert_eq!(restored.profile_native_language(), "fr");

    // Back to the default: the backup is version 1 again.
    restored.set_profile("fr", vec!["en".to_owned()]).unwrap();
    assert!(restored.backup().starts_with("{\n  \"schema_version\": 1,"));
}

/// An engine whose first pack studies Spanish glossed in English serves English, and a full
/// reset keeps its reader there, studying Spanish alone (`generalise-lingua-native-language` D5).
#[test]
fn spec_scenario_a_full_reset_on_an_engine_glossed_in_english() {
    let mut engine = LinguaEngine::new(&spanish_for_english_pack()).unwrap();
    assert_eq!(engine.native_language(), "en");
    assert_eq!(engine.profile_native_language(), "en");
    assert_eq!(engine.studied_languages(), r#"["es"]"#);
    let backup = engine.backup();
    assert!(backup.starts_with("{\n  \"schema_version\": 2,"));
    assert!(
        backup.contains("\"native_language\": \"English\""),
        "{backup}"
    );

    engine.set_status_at("haber", "known", T_MS, None).unwrap();
    engine.set_calibration(1_000, None).unwrap();
    engine.reset_statuses();
    assert_eq!(engine.profile_native_language(), "en");
    engine.reset();
    assert_eq!(engine.native_language(), "en");
    assert_eq!(engine.profile_native_language(), "en");
    assert_eq!(engine.studied_languages(), r#"["es"]"#);
    assert_eq!(engine.export_status_ops(), "[]");
    assert_eq!(
        engine.backup(),
        backup,
        "a reset gives the new engine's state"
    );
}

/// A restore keeps a backup of another native language whole (design D4): the engine still
/// serves its own, and the reader's shows the mismatch.
#[test]
fn a_backup_of_another_native_language_is_restored_whole() {
    let english_reader = LinguaEngine::new(&spanish_for_english_pack()).unwrap();
    let mut engine = english_engine();
    engine.restore(&english_reader.backup()).unwrap();
    assert_eq!(engine.native_language(), "fr");
    assert_eq!(engine.profile_native_language(), "en");
    assert_eq!(engine.studied_languages(), r#"["es"]"#);
    // A full reset gives the engine's own again.
    engine.reset();
    assert_eq!(engine.profile_native_language(), "fr");
    assert_eq!(engine.studied_languages(), r#"["en"]"#);
}

/// A dictionary form's frequency rank comes from its language's own pack
/// (`add-lingua-card-frequency`); a lemma the pack does not rank has none.
#[test]
fn a_rank_comes_from_the_language_s_own_pack() {
    let engine = two_language_engine();
    assert_eq!(engine.frequency_rank("haber", es()).unwrap(), Some(20));
    assert_eq!(engine.frequency_rank("equipo", es()).unwrap(), Some(900));
    assert_eq!(engine.frequency_rank("run", None).unwrap(), Some(500));
    assert_eq!(engine.frequency_rank("haber", None).unwrap(), None);
    assert_eq!(engine.frequency_rank("run", es()).unwrap(), None);
}

#[test]
fn spec_scenario_a_second_language_is_served_by_its_own_pack() {
    let mut engine = english_engine();
    assert_eq!(engine.languages(), r#"["en"]"#);
    assert_eq!(engine.add_pack(&spanish_pack()).unwrap(), "es");
    assert_eq!(engine.languages(), r#"["en","es"]"#);

    let spanish = engine.analyse(vec![SPANISH.to_owned()], es()).unwrap();
    assert!(
        spanish.contains(r#""analyzer_version":"1.2.0""#),
        "{spanish}"
    );
    assert!(
        spanish.contains(r#""surface":"has","lemma":"haber""#),
        "{spanish}"
    );
    let english = engine.analyse(vec![ENGLISH.to_owned()], None).unwrap();
    assert!(
        english.contains(r#""analyzer_version":"1.1.0""#),
        "{english}"
    );

    assert_eq!(
        engine.gloss("haber", es()).unwrap().as_deref(),
        Some("Avoir (auxiliaire)")
    );
    assert_eq!(engine.gloss("haber", None).unwrap(), None);
    assert_eq!(
        engine.gloss("city", None).unwrap().as_deref(),
        Some("ville")
    );
    let phrase = engine.phrase_gloss("tú has trabajado", es()).unwrap();
    assert!(phrase.contains(r#""lemma":"haber""#), "{phrase}");
}

#[test]
fn spec_scenario_a_spanish_record_exports_as_spanish() {
    let mut engine = two_language_engine();
    engine
        .set_status_at("haber", "learning", T_MS, es())
        .unwrap();
    engine
        .set_status_at("city", "known", T_MS + 1.0, None)
        .unwrap();
    let statuses = engine.export_status_ops();
    assert!(
        statuses.contains(r#""language":"es","lemma":"haber""#),
        "{statuses}"
    );
    assert!(
        statuses.contains(r#""language":"en","lemma":"city""#),
        "{statuses}"
    );

    engine
        .add_card(
            "equipo",
            "equipos",
            "Los equipos nunca entregan el viernes.",
            "",
            None,
            T_SECS,
            es(),
        )
        .unwrap();
    let cards = engine.export_card_ops();
    assert!(
        cards.contains(r#""language":"es","lemma":"equipo""#),
        "{cards}"
    );

    engine.set_declared_level_at("A2", T_MS, es()).unwrap();
    let levels = engine.export_declared_levels();
    assert!(
        levels.contains(r#"{"language":"es","level":"A2""#),
        "{levels}"
    );
}

#[test]
fn spec_scenario_records_of_a_language_the_engine_does_not_study_are_skipped() {
    let status = r#"[{"language":"es","lemma":"haber","status":"known","provenance":"manual","updated_at":5}]"#;
    let mut english_only = english_engine();
    assert_eq!(english_only.apply_status_changes(status).unwrap(), 0);
    assert_eq!(english_only.export_status_ops(), "[]");
    let mut both = two_language_engine();
    assert_eq!(both.apply_status_changes(status).unwrap(), 1);
    assert!(
        both.export_status_ops()
            .contains(r#""language":"es","lemma":"haber""#)
    );

    // A record that names no language is English.
    let unlabelled = r#"[{"lemma":"city","status":"known","updated_at":6}]"#;
    assert_eq!(english_only.apply_status_changes(unlabelled).unwrap(), 1);
    assert!(
        english_only
            .export_status_ops()
            .contains(r#""language":"en","lemma":"city""#)
    );

    let card = r#"[{"client_id":"equipo","language":"es","lemma":"equipo","surface_form":"equipos","source_sentence":"","source":"","gloss":"","fsrs_state":"","deleted":false,"client_ts":1000,"device_id":""}]"#;
    assert_eq!(english_engine().apply_card_ops(card).unwrap(), 0);
    assert_eq!(two_language_engine().apply_card_ops(card).unwrap(), 1);

    let level = r#"[{"language":"es","level":"B1","updated_at":7}]"#;
    assert_eq!(
        english_engine()
            .apply_declared_level_changes(level)
            .unwrap(),
        0
    );
    let mut levels = two_language_engine();
    assert_eq!(levels.apply_declared_level_changes(level).unwrap(), 1);
    assert_eq!(levels.declared_level(es()).unwrap().as_deref(), Some("B1"));
    assert_eq!(levels.declared_level(None).unwrap(), None);
}

#[test]
fn pack_bound_answers_follow_the_language() {
    let mut engine = two_language_engine();
    assert!(engine.has_levels(None).unwrap());
    assert!(!engine.has_levels(es()).unwrap());
    assert_eq!(engine.level_ladder(es()).unwrap(), "[]");
    assert_ne!(engine.level_ladder(None).unwrap(), "[]");
    assert!(engine.notice(es()).unwrap().contains("Spanish test pack."));
    assert!(!engine.notice(None).unwrap().contains("Spanish test pack."));
    assert_eq!(
        engine.licences(es()).unwrap(),
        engine.licences(None).unwrap()
    );
    assert!(
        engine
            .vocabulary_estimate(es())
            .unwrap()
            .contains("\"universe\"")
    );
    assert_eq!(
        engine.seed_level("A1", 5, "common", T_SECS, es()).unwrap(),
        0
    );
    assert!(engine.seed_level("A1", 5, "common", T_SECS, None).unwrap() > 0);

    engine.set_calibration(1_000, es()).unwrap();
    assert_eq!(engine.calibration(es()).unwrap(), 1_000);
    assert_eq!(engine.calibration(None).unwrap(), 0);
    engine.set_declared_level("B2", es()).unwrap();
    assert_eq!(engine.declared_level(es()).unwrap().as_deref(), Some("B2"));
    engine.set_status("equipo", "known", es()).unwrap();
    engine
        .add_card("haber", "has", "", "", None, T_SECS, es())
        .unwrap();
    engine.retire_card("haber", T_SECS, es()).unwrap();
    engine
        .record_exposures(vec!["trabajar".to_owned()], "page", T_MS, es())
        .unwrap();
    assert_eq!(engine.promote_by_exposure(2, T_MS, es()).unwrap(), 0);
    let grammar = engine.word_grammar("has", "haber", es()).unwrap();
    assert!(grammar.contains("Avoir (auxiliaire)"), "{grammar}");
}

#[test]
fn spec_scenario_a_document_is_read_in_its_own_language() {
    let engine = english_engine(); // a choice needs no pack: English alone is held
    let spanish = vec![
        "Los equipos nunca entregan el viernes por la noche, es una regla antigua.".to_owned(),
        "El faro se alza sobre las rocas desde hace más de un siglo.".to_owned(),
    ];
    let both = vec!["en".to_owned(), "es".to_owned()];
    assert_eq!(
        engine
            .detect_language(spanish.clone(), both.clone(), None)
            .unwrap(),
        "es"
    );
    // Nothing to detect: the declared language, then the first candidate.
    let menu = vec!["Menu".to_owned(), "OK".to_owned()];
    assert_eq!(
        engine
            .detect_language(menu.clone(), both.clone(), Some("es-ES".to_owned()))
            .unwrap(),
        "en"
    );
    assert_eq!(
        engine
            .detect_language(menu.clone(), both.clone(), Some("es".to_owned()))
            .unwrap(),
        "es"
    );
    assert_eq!(
        engine
            .detect_language(menu, both, Some("fr".to_owned()))
            .unwrap(),
        "en"
    );
    // One candidate is chosen without detection.
    assert_eq!(
        engine
            .detect_language(spanish, vec!["en".to_owned()], None)
            .unwrap(),
        "en"
    );
}

#[test]
fn spec_scenario_a_review_across_languages_or_within_one() {
    let mut engine = two_language_engine();
    engine
        .add_card("city", "city", "The city sleeps.", "", None, 1.0, None)
        .unwrap();
    engine
        .add_card("faro", "faro", "El faro brilla.", "", None, 1.0, es())
        .unwrap();
    assert_eq!(engine.due_count(10.0, None), 2);
    assert_eq!(engine.due_count(10.0, Some(vec!["es".to_owned()])), 1);
    // A tag the core does not know filters nothing.
    assert_eq!(engine.due_count(10.0, Some(vec!["pt".to_owned()])), 2);
    // The deck's size by language, read the same way (refine-lingua-review-language D4).
    assert_eq!(engine.deck_count(None), 2);
    assert_eq!(engine.deck_count(Some(vec!["es".to_owned()])), 1);
    assert_eq!(engine.deck_count(Some(vec!["en".to_owned()])), 1);
    assert_eq!(engine.deck_count(Some(vec!["pt".to_owned()])), 2);

    assert_eq!(engine.start_review(10.0, Some(vec!["es".to_owned()])), 1);
    assert_eq!(engine.review_current_language().as_deref(), Some("es"));
    assert_eq!(engine.start_review(10.0, None), 2);
    assert!(engine.review_current_language().is_some());
    engine.review_mark_known(10.0);
    engine.review_mark_known(10.0);
    assert_eq!(
        engine.review_current_language(),
        None,
        "no card shown, no language"
    );
}

#[test]
fn spec_scenario_the_studied_languages_live_in_the_backup() {
    let mut engine = english_engine();
    assert_eq!(engine.studied_languages(), r#"["en"]"#);
    assert!(engine.backup().starts_with("{\n  \"schema_version\": 1,"));

    // A reader may study a language whose pack the engine does not hold.
    engine
        .set_studied_languages(vec!["es".to_owned(), "en".to_owned()])
        .unwrap();
    assert_eq!(engine.studied_languages(), r#"["es","en"]"#);
    assert_eq!(engine.languages(), r#"["en"]"#);
    let backup = engine.backup();
    assert!(backup.starts_with("{\n  \"schema_version\": 2,"));

    let mut restored = english_engine();
    restored.restore(&backup).unwrap();
    assert_eq!(restored.studied_languages(), r#"["es","en"]"#);

    // Resetting statuses keeps the profile; erasing everything returns it to English.
    restored.reset_statuses();
    assert_eq!(restored.studied_languages(), r#"["es","en"]"#);
    restored.reset();
    assert_eq!(restored.studied_languages(), r#"["en"]"#);
    assert!(restored.backup().starts_with("{\n  \"schema_version\": 1,"));
}

#[test]
fn a_two_language_state_round_trips_and_an_engine_without_the_pack_keeps_it() {
    let mut engine = two_language_engine();
    engine.set_status_at("haber", "known", T_MS, es()).unwrap();
    engine
        .set_status_at("city", "learning", T_MS, None)
        .unwrap();
    let backup = engine.backup();

    let mut restored = two_language_engine();
    restored.restore(&backup).unwrap();
    assert_eq!(restored.backup(), backup);

    // Without the Spanish pack, the Spanish records stay (the backup is lossless)
    // and export under their own language.
    let mut english_only = english_engine();
    english_only.restore(&backup).unwrap();
    assert!(
        english_only
            .export_status_ops()
            .contains(r#""language":"es","lemma":"haber""#)
    );
}

/// A reader of French has a backup of schema version 3, which restores whole; a reader of
/// Spanish on the same engine keeps version 2 (add-lingua-french-baseline D7).
#[test]
fn spec_scenario_a_french_reader_s_backup_is_version_3() {
    let mut engine = english_native_engine();
    assert_eq!(engine.languages(), r#"["es","fr"]"#);
    assert_eq!(engine.native_language(), "en");
    // The engine starts on es-en: a reader of Spanish, at version 2.
    assert_eq!(engine.studied_languages(), r#"["es"]"#);
    assert!(engine.backup().starts_with("{\n  \"schema_version\": 2,"));

    engine.set_profile("en", vec!["fr".to_owned()]).unwrap();
    assert_eq!(engine.studied_languages(), r#"["fr"]"#);
    let fr = || Some("fr".to_owned());
    engine
        .set_status_at("maison", "learning", T_MS, fr())
        .unwrap();
    engine
        .add_card(
            "maison",
            "maisons",
            "Les maisons dorment.",
            "",
            None,
            T_SECS,
            fr(),
        )
        .unwrap();
    let page = engine
        .analyse(
            vec!["L'homme est dans la maison, et les maisons dorment encore ce matin.".to_owned()],
            fr(),
        )
        .unwrap();
    // French's pre-pass splits the elision, each piece with its own span
    // (add-lingua-french-tokenisation), and the page reports French's own version, no longer one
    // of the baseline's `0.x` (add-lingua-french-analysis D5).
    let french = StudiedLanguage::French.analyzer_version();
    assert!(!french.starts_with("0."), "{french}");
    assert!(
        page.contains(&format!(r#""analyzer_version":"{french}""#)),
        "{page}"
    );
    assert!(
        page.contains(r#""start":0,"end":2,"surface":"Le","lemma":"le""#),
        "{page}"
    );
    assert!(
        page.contains(r#""start":2,"end":7,"surface":"homme","lemma":"homme""#),
        "{page}"
    );
    let backup = engine.backup();
    assert!(
        backup.starts_with("{\n  \"schema_version\": 3,"),
        "{backup}"
    );

    let mut restored = english_native_engine();
    restored.restore(&backup).unwrap();
    assert_eq!(restored.backup(), backup, "a version 3 restore is lossless");
    assert_eq!(restored.studied_languages(), r#"["fr"]"#);
    assert!(
        restored
            .export_status_ops()
            .contains(r#""language":"fr","lemma":"maison""#)
    );

    // A reader studying Spanish alone on the same engine stays at version 2.
    let mut spanish = english_native_engine();
    spanish.set_profile("en", vec!["es".to_owned()]).unwrap();
    spanish.set_status_at("casa", "known", T_MS, es()).unwrap();
    assert!(spanish.backup().starts_with("{\n  \"schema_version\": 2,"));
}

#[path = "support/french_spans.rs"]
mod french_spans;

/// French's pieces have spans of their own, the same on the host as on the wasm target
/// (`languages_wasm.rs` asserts these against the fixture pack re-stamped French), whatever the
/// pack (add-lingua-french-tokenisation D6).
#[test]
fn spec_scenario_french_pieces_keep_their_spans_on_the_host() {
    let mut engine = english_native_engine();
    engine.set_profile("en", vec!["fr".to_owned()]).unwrap();
    let page = engine
        .analyse(
            vec![french_spans::PARAGRAPH.to_owned()],
            Some("fr".to_owned()),
        )
        .unwrap();
    assert_eq!(
        french_spans::surfaces_and_spans(&page),
        french_spans::expected()
    );
    for (surface, start, end) in french_spans::TOKENS {
        let source = &french_spans::PARAGRAPH[*start..*end];
        assert!(
            !source.contains('\u{202F}') && !source.contains('-'),
            "{surface}: {source:?}"
        );
    }
}
