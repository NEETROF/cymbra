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

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::decks::{Card, EncounterSource, LinguaState, Provenance};
use lingua_core::knowledge::{CefrLevel, KnownSource, Status};

const EN: StudiedLanguage = StudiedLanguage::English;
const FIXTURE: &str = "tests/fixtures/backup-v1-english.json";

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
