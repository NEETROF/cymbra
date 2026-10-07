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

use lingua_agent::store::{DEFAULT_CALIBRATION, SCHEMA_VERSION, Store, StoreError};
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::decks::card::{Card, EncounterSource, Provenance};
use lingua_core::knowledge::status::{KnownSource, Status};
use rusqlite::{Connection, params};

const EN: StudiedLanguage = StudiedLanguage::English;
const ES: StudiedLanguage = StudiedLanguage::Spanish;

fn card(lemma: &str) -> Card {
    Card::new(
        lemma,
        lemma,
        Provenance {
            sentence: format!("A sentence with {lemma}."),
            source: EncounterSource::AgentSession {
                session: "claude-code".into(),
            },
            captured_at: 0,
        },
        Some(format!("gloss-{lemma}")),
        "fr",
    )
}

/// A fresh file for one test's store.
fn temp_db(name: &str) -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("lingua-agent-store-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("lingua.db");
    let _ = std::fs::remove_file(&path);
    path
}

#[test]
fn fresh_store_is_at_the_current_schema_with_defaults() {
    let store = Store::open_in_memory().unwrap();
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    assert_eq!(store.calibration(EN).unwrap(), DEFAULT_CALIBRATION);
    assert_eq!(store.calibration(ES).unwrap(), DEFAULT_CALIBRATION);
    assert_eq!(store.tracked_lemmas().unwrap(), 0);
    assert_eq!(store.deck_len(EN).unwrap(), 0);
}

#[test]
fn exposures_accumulate_per_language() {
    let store = Store::open_in_memory().unwrap();
    store
        .record_exposure(EN, "seldom", 1, "claude-code", 100)
        .unwrap();
    store
        .record_exposure(EN, "seldom", 2, "claude-code", 200)
        .unwrap();
    store
        .record_exposure(EN, "run", 1, "claude-code", 100)
        .unwrap();
    store
        .record_exposure(ES, "no", 4, "claude-code", 100)
        .unwrap();
    store
        .record_exposure(EN, "no", 1, "claude-code", 100)
        .unwrap();
    assert_eq!(store.exposure(EN, "seldom").unwrap(), 3);
    assert_eq!(store.exposure(EN, "run").unwrap(), 1);
    assert_eq!(store.exposure(EN, "never").unwrap(), 0);
    assert_eq!(store.exposure(ES, "seldom").unwrap(), 0);
    assert_eq!(
        (
            store.exposure(ES, "no").unwrap(),
            store.exposure(EN, "no").unwrap()
        ),
        (4, 1)
    );
    assert_eq!(store.tracked_lemmas().unwrap(), 4);
}

#[test]
fn statuses_and_calibration_build_a_knowledge_state_per_language() {
    let store = Store::open_in_memory().unwrap();
    store.set_calibration(EN, 2500).unwrap();
    store.set_calibration(ES, 800).unwrap();
    store
        .set_status(EN, "run", Status::Known(KnownSource::Manual))
        .unwrap();
    store.set_status(EN, "seldom", Status::Learning).unwrap();
    store.set_status(ES, "casa", Status::Learning).unwrap();
    let ks = store.knowledge_state().unwrap();
    assert_eq!(ks.calibration(EN), 2500);
    assert_eq!(ks.calibration(ES), 800);
    assert_eq!(
        ks.explicit_status(EN, "run"),
        Some(Status::Known(KnownSource::Manual))
    );
    assert_eq!(ks.explicit_status(EN, "seldom"), Some(Status::Learning));
    assert_eq!(ks.explicit_status(ES, "casa"), Some(Status::Learning));
    assert_eq!(ks.explicit_status(EN, "casa"), None);
}

#[test]
fn cards_round_trip_and_report_due_per_language() {
    let store = Store::open_in_memory().unwrap();
    store.upsert_card(EN, &card("seldom")).unwrap();
    store.upsert_card(EN, &card("conundrum")).unwrap();
    store.upsert_card(ES, &card("casa")).unwrap();
    assert_eq!(
        (store.deck_len(EN).unwrap(), store.deck_len(ES).unwrap()),
        (2, 1)
    );
    let got = store.card(EN, "seldom").unwrap().unwrap();
    assert_eq!(got.lemma, "seldom");
    assert_eq!(got.gloss.as_deref(), Some("gloss-seldom"));
    // Fresh cards are due now, each in its own deck.
    assert_eq!(store.due_cards(EN, 1000).unwrap().len(), 2);
    assert_eq!(store.due_cards(ES, 1000).unwrap().len(), 1);
    assert!(store.card(EN, "absent").unwrap().is_none());
    assert!(store.card(ES, "seldom").unwrap().is_none());
}

#[test]
fn ingest_offsets_persist_per_transcript() {
    let store = Store::open_in_memory().unwrap();
    assert_eq!(store.ingest_offset("/t/a.jsonl").unwrap(), 0);
    store.set_ingest_offset("/t/a.jsonl", 4096).unwrap();
    store.set_ingest_offset("/t/b.jsonl", 10).unwrap();
    assert_eq!(store.ingest_offset("/t/a.jsonl").unwrap(), 4096);
    assert_eq!(store.ingest_offset("/t/b.jsonl").unwrap(), 10);
}

#[test]
fn a_schema_1_store_is_kept_whole_as_english() {
    // add-lingua-agent-languages D3: what the plugin stored before languages, as it stored it.
    let path = temp_db("v1");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE meta (k TEXT PRIMARY KEY, v TEXT NOT NULL);
             CREATE TABLE exposures (
               lemma TEXT PRIMARY KEY, occurrences INTEGER NOT NULL,
               last_source TEXT NOT NULL, last_seen INTEGER NOT NULL);
             CREATE TABLE statuses (lemma TEXT PRIMARY KEY, status TEXT NOT NULL);
             CREATE TABLE cards (lemma TEXT PRIMARY KEY, json TEXT NOT NULL);
             CREATE TABLE ingest (transcript TEXT PRIMARY KEY, offset INTEGER NOT NULL);
             INSERT INTO meta VALUES ('schema_version', '1'), ('calibration', '2500');
             INSERT INTO exposures VALUES ('seldom', 3, 't1', 100);
             INSERT INTO statuses VALUES ('run', 'known');
             INSERT INTO ingest VALUES ('/t/a.jsonl', 4096);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO cards VALUES ('seldom', ?1)",
            params![serde_json::to_string(&card("seldom")).unwrap()],
        )
        .unwrap();
    }

    let store = Store::open(&path).unwrap();
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    assert_eq!(store.exposure(EN, "seldom").unwrap(), 3);
    assert_eq!(store.calibration(EN).unwrap(), 2500);
    assert_eq!(store.calibration(ES).unwrap(), DEFAULT_CALIBRATION);
    assert!(store.card(EN, "seldom").unwrap().is_some());
    assert_eq!(
        store.knowledge_state().unwrap().explicit_status(EN, "run"),
        Some(Status::Known(KnownSource::Manual))
    );
    assert_eq!(store.ingest_offset("/t/a.jsonl").unwrap(), 4096);
    // Opening it again finds schema 2 and changes nothing.
    drop(store);
    let again = Store::open(&path).unwrap();
    assert_eq!(again.exposure(EN, "seldom").unwrap(), 3);
}

#[test]
fn a_store_from_a_later_plugin_is_left_untouched() {
    let path = temp_db("newer");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE meta (k TEXT PRIMARY KEY, v TEXT NOT NULL);
             INSERT INTO meta VALUES ('schema_version', '3');
             CREATE TABLE future (x TEXT);",
        )
        .unwrap();
    }
    let Err(err) = Store::open(&path) else {
        panic!("a later schema must not open");
    };
    assert!(matches!(err, StoreError::Newer(3)));
    assert!(err.to_string().contains('3'));
    let conn = Connection::open(&path).unwrap();
    let tables: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('future', 'exposures')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tables, 1); // its own table only: nothing created
}
