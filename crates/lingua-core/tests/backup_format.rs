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

//! The backup file format, pinned (add-lingua-studied-language-profile). An English reader's
//! backup is written as schema version 1, byte for byte what builds wrote before the profile
//! was stored: a build released before it must keep reading it. `fixtures/backup-v1-english.json`
//! was written by the build before that change; `LINGUA_BLESS=1` rewrites it, which only a
//! deliberate format change may do.
//!
//! A Spanish reader's backup, and an English-native reader's, are written as schema version 2,
//! byte for byte what builds wrote before schema version 3 existed (add-lingua-french-baseline
//! D7): `fixtures/backup-v2-spanish.json` and `fixtures/backup-v2-english-native.json` were
//! written by the build before that change.

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::decks::{Card, EncounterSource, LinguaState, Provenance};
use lingua_core::knowledge::{CefrLevel, KnownSource, NativeLanguage, Status};

const EN: StudiedLanguage = StudiedLanguage::English;
const ES: StudiedLanguage = StudiedLanguage::Spanish;
const FIXTURE: &str = "tests/fixtures/backup-v1-english.json";
const SPANISH_FIXTURE: &str = "tests/fixtures/backup-v2-spanish.json";
const ENGLISH_NATIVE_FIXTURE: &str = "tests/fixtures/backup-v2-english-native.json";

/// An English reader with a record in every per-language map: statuses and their times, a
/// calibration, a declared level and its time, an exposure counter and a card.
fn english_state() -> LinguaState {
    let mut state = LinguaState::default();
    state.knowledge.set_calibration(EN, 3_000);
    state.knowledge.set_status_at(
        EN,
        "lighthouse",
        Status::Known(KnownSource::Manual),
        1_700_000_000_000,
    );
    state
        .knowledge
        .set_status_at(EN, "harbour", Status::Learning, 1_700_000_001_000);
    state
        .knowledge
        .set_declared_level_at(EN, Some(CefrLevel::B1), 1_700_000_002_000);
    state
        .exposure
        .record(EN, "keeper", 3, "web:example.com", 1_700_000_003);
    state.deck.upsert(
        EN,
        Card::new(
            "harbour",
            "harbours",
            Provenance {
                sentence: "Ships rest in the harbours.".to_owned(),
                source: EncounterSource::Web {
                    url: "https://example.com/sea".to_owned(),
                },
                captured_at: 1_700_000_004,
            },
            Some("port".to_owned()),
            "fr",
        ),
    );
    state
}

#[test]
fn an_english_backup_is_written_as_before() {
    let written = english_state().to_backup();
    if std::env::var_os("LINGUA_BLESS").is_some() {
        std::fs::write(FIXTURE, &written).expect("bless the fixture");
    }
    let pinned = std::fs::read_to_string(FIXTURE).expect("the pinned fixture");
    assert_eq!(written, pinned, "an English backup changed: see {FIXTURE}");
}

/// `state`'s backup, held to the fixture at `path` (rewritten under `LINGUA_BLESS`).
fn pinned(state: &LinguaState, path: &str) -> (String, String) {
    let written = state.to_backup();
    if std::env::var_os("LINGUA_BLESS").is_some() {
        std::fs::write(path, &written).expect("bless the fixture");
    }
    let pinned = std::fs::read_to_string(path).expect("the pinned fixture");
    (written, pinned)
}

/// A Spanish card met on `url`, its gloss written in `gloss_language`.
fn faro(gloss: &str, gloss_language: &str) -> Card {
    Card::new(
        "faro",
        "faros",
        Provenance {
            sentence: "Los faros brillan.".to_owned(),
            source: EncounterSource::Web {
                url: "https://example.es/mar".to_owned(),
            },
            captured_at: 1_700_000_004,
        },
        Some(gloss.to_owned()),
        gloss_language,
    )
}

/// A reader of Spanish then English, French native, with a Spanish record in every
/// per-language map beside the English ones.
fn spanish_state() -> LinguaState {
    let mut state = english_state();
    state
        .profile
        .set_studied_languages(vec![ES, EN])
        .expect("two languages");
    state.knowledge.set_calibration(ES, 1_500);
    state
        .knowledge
        .set_status_at(ES, "faro", Status::Learning, 1_700_000_000_500);
    state
        .knowledge
        .set_declared_level_at(ES, Some(CefrLevel::A2), 1_700_000_002_500);
    state
        .exposure
        .record(ES, "marinero", 2, "web:example.es", 1_700_000_003);
    state.deck.upsert(ES, faro("phare", "fr"));
    state
}

/// An English-native reader of Spanish, as an es-en engine writes them: their cards glossed in
/// English.
fn english_native_state() -> LinguaState {
    let mut state = LinguaState::default();
    state
        .profile
        .set(NativeLanguage::English, vec![ES])
        .expect("Spanish for an English reader");
    state
        .knowledge
        .set_status_at(ES, "faro", Status::Learning, 1_700_000_000_500);
    state
        .exposure
        .record(ES, "marinero", 2, "web:example.es", 1_700_000_003);
    state.deck.upsert(ES, faro("lighthouse", "en"));
    state
}

#[test]
fn a_spanish_backup_is_written_as_before() {
    let state = spanish_state();
    assert_eq!(state.backup_version(), 2);
    let (written, pinned) = pinned(&state, SPANISH_FIXTURE);
    assert_eq!(
        written, pinned,
        "a Spanish backup changed: see {SPANISH_FIXTURE}"
    );
}

#[test]
fn an_english_native_backup_is_written_as_before() {
    let state = english_native_state();
    assert_eq!(state.backup_version(), 2);
    let (written, pinned) = pinned(&state, ENGLISH_NATIVE_FIXTURE);
    assert_eq!(
        written, pinned,
        "an English-native backup changed: see {ENGLISH_NATIVE_FIXTURE}"
    );
}
