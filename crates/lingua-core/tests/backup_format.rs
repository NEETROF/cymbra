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
//! written by the build before that change. A backup naming French is written as version 3,
//! `fixtures/backup-v3-french.json`, which `decks/backup.rs` holds a build reading version 2 to
//! refuse as unsupported.

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::decks::{Card, EncounterSource, LinguaState, Provenance};
use lingua_core::knowledge::{CefrLevel, KnownSource, NativeLanguage, Status};

const EN: StudiedLanguage = StudiedLanguage::English;
const ES: StudiedLanguage = StudiedLanguage::Spanish;
const FR: StudiedLanguage = StudiedLanguage::French;
const FIXTURE: &str = "tests/fixtures/backup-v1-english.json";
const SPANISH_FIXTURE: &str = "tests/fixtures/backup-v2-spanish.json";
const ENGLISH_NATIVE_FIXTURE: &str = "tests/fixtures/backup-v2-english-native.json";
const FRENCH_FIXTURE: &str = "tests/fixtures/backup-v3-french.json";

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

/// An English-native reader of Spanish with French records, as an engine holding es-en and the
/// French baseline's fr-en fixture writes them: a status and a card glossed in English.
fn french_state() -> LinguaState {
    let mut state = english_native_state();
    state
        .knowledge
        .set_status_at(FR, "phare", Status::Learning, 1_700_000_000_700);
    state.deck.upsert(
        FR,
        Card::new(
            "homme",
            "l'homme",
            Provenance {
                sentence: "L'homme regardait la mer.".to_owned(),
                source: EncounterSource::Web {
                    url: "https://example.fr/mer".to_owned(),
                },
                captured_at: 1_700_000_005,
            },
            Some("man".to_owned()),
            "en",
        ),
    );
    state
}

#[test]
fn a_french_backup_is_written_as_version_3() {
    let state = french_state();
    assert_eq!(state.backup_version(), 3);
    let (written, pinned) = pinned(&state, FRENCH_FIXTURE);
    assert_eq!(
        written, pinned,
        "a French backup changed: see {FRENCH_FIXTURE}"
    );
    let restored = LinguaState::from_backup(&pinned).expect("this build reads version 3");
    assert_eq!(restored, state);
}

/// Every `"French"` of a backup but its native language's.
fn french_outside_the_native_language(backup: &str) -> usize {
    backup.matches("\"French\"").count() - backup.matches("\"native_language\": \"French\"").count()
}

#[test]
fn a_backup_below_version_3_names_french_only_as_the_native_language() {
    // add-lingua-french-baseline D7, held by the files rather than by the list of maps
    // `backup_version` reads: every language name of a backup holding a record in each
    // per-language map, turned into French one at a time and restored, is written again below
    // version 3 only when it was the native language. A per-language map added to the state but
    // not to the version trigger fails here as soon as `english_state` holds a record in it: its
    // French entry would be written in a file that a build reading version 2 reads and cannot
    // parse.
    let mut renamed = 0;
    for state in [english_state(), spanish_state(), english_native_state()] {
        let backup = state.to_backup();
        assert!(state.backup_version() < 3, "{backup}");
        assert_eq!(french_outside_the_native_language(&backup), 0, "{backup}");
        for name in ["\"English\"", "\"Spanish\""] {
            for (at, _) in backup.match_indices(name) {
                let file = format!("{}\"French\"{}", &backup[..at], &backup[at + name.len()..]);
                let restored =
                    LinguaState::from_backup(&file).unwrap_or_else(|e| panic!("{e}:\n{file}"));
                let written = restored.to_backup();
                if restored.backup_version() < 3 {
                    assert_eq!(
                        french_outside_the_native_language(&written),
                        0,
                        "French written below version 3:\n{written}"
                    );
                }
                renamed += 1;
            }
        }
    }
    // Statuses, their times, calibrations, declared levels and their times, exposure counters
    // and cards, in three states, and the profiles.
    assert!(renamed >= 20, "{renamed} names renamed");
}
