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

use lingua_agent::store::{DEFAULT_CALIBRATION, SCHEMA_VERSION, Store};
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::decks::card::{Card, EncounterSource, Provenance};
use lingua_core::knowledge::status::{KnownSource, Status};

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
    )
}

#[test]
fn fresh_store_is_at_the_current_schema_with_defaults() {
    let store = Store::open_in_memory().unwrap();
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    assert_eq!(store.calibration().unwrap(), DEFAULT_CALIBRATION);
    assert_eq!(store.tracked_lemmas().unwrap(), 0);
    assert_eq!(store.deck_len().unwrap(), 0);
}

#[test]
fn exposures_accumulate() {
    let store = Store::open_in_memory().unwrap();
    store
        .record_exposure("seldom", 1, "claude-code", 100)
        .unwrap();
    store
        .record_exposure("seldom", 2, "claude-code", 200)
        .unwrap();
    store.record_exposure("run", 1, "claude-code", 100).unwrap();
    assert_eq!(store.exposure("seldom").unwrap(), 3);
    assert_eq!(store.exposure("run").unwrap(), 1);
    assert_eq!(store.exposure("never").unwrap(), 0);
    assert_eq!(store.tracked_lemmas().unwrap(), 2);
}

#[test]
fn statuses_and_calibration_build_a_knowledge_state() {
    let store = Store::open_in_memory().unwrap();
    store.set_calibration(2500).unwrap();
    store
        .set_status("run", Status::Known(KnownSource::Manual))
        .unwrap();
    store.set_status("seldom", Status::Learning).unwrap();
    let ks = store.knowledge_state().unwrap();
    assert_eq!(ks.calibration(StudiedLanguage::English), 2500);
    assert_eq!(
        ks.explicit_status(StudiedLanguage::English, "run"),
        Some(Status::Known(KnownSource::Manual))
    );
    assert_eq!(
        ks.explicit_status(StudiedLanguage::English, "seldom"),
        Some(Status::Learning)
    );
}

#[test]
fn cards_round_trip_and_report_due() {
    let store = Store::open_in_memory().unwrap();
    store.upsert_card(&card("seldom")).unwrap();
    store.upsert_card(&card("conundrum")).unwrap();
    assert_eq!(store.deck_len().unwrap(), 2);
    let got = store.card("seldom").unwrap().unwrap();
    assert_eq!(got.lemma, "seldom");
    assert_eq!(got.gloss.as_deref(), Some("gloss-seldom"));
    // Fresh cards are due now.
    assert_eq!(store.due_cards(1000).unwrap().len(), 2);
    assert!(store.card("absent").unwrap().is_none());
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
