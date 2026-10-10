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

//! A Spanish expression settled under its lemmas keeps that key
//! (add-lingua-spanish-expression-keys D6), through the engine the extension calls and the real
//! es-fr pack built from the committed tables: es-fr is read, and a reader may hold a status or a
//! deck card on the lemma chain the phrase gloss reported before Spanish expressions were named.
//! Such a reader is still answered under that chain; every other reader under the name.
//!
//! Host only: the pack builder is native (C zstd).

#![cfg(not(target_arch = "wasm32"))]

mod support;

use std::sync::OnceLock;

use lingua_wasm::LinguaEngine;
use support::Scenario;

/// 2026-09-21T13:46:40Z, in epoch seconds: the deck bindings' unit.
const T: f64 = 1_790_000_000.0;

/// The es-fr pack, built once for the file.
fn es_fr() -> &'static [u8] {
    static PACK: OnceLock<Vec<u8>> = OnceLock::new();
    PACK.get_or_init(|| Scenario::real_pack("es-fr"))
}

fn engine() -> LinguaEngine {
    Scenario::engine(&[("es-fr", es_fr().to_vec())])
}

fn es() -> Option<String> {
    Some("es".to_owned())
}

/// The expressions the engine finds in a selection, as (start, end, key, class).
fn expressions(engine: &LinguaEngine, text: &str) -> Vec<(u64, u64, String, String)> {
    let gloss: serde_json::Value = match engine.phrase_gloss(text, es()) {
        Ok(json) => serde_json::from_str(&json).expect("a phrase gloss"),
        Err(_) => panic!("{text} is glossed"),
    };
    gloss["expressions"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|m| {
            (
                m["start"].as_u64().expect("start"),
                m["end"].as_u64().expect("end"),
                m["key"].as_str().expect("key").to_owned(),
                m["class"].as_str().expect("class").to_owned(),
            )
        })
        .collect()
}

fn found(start: u64, end: u64, key: &str, class: &str) -> (u64, u64, String, String) {
    (start, end, key.to_owned(), class.to_owned())
}

#[test]
fn a_reader_who_settled_the_lemma_chain_keeps_it_and_another_reader_sees_the_name() {
    let mut settled = engine();
    settled
        .set_status_at("tener en contar", "known", T * 1000.0, es())
        .unwrap();
    assert_eq!(
        expressions(&settled, "tener en cuenta"),
        [found(0, 3, "tener en contar", "Known")]
    );
    assert_eq!(
        expressions(&engine(), "tener en cuenta"),
        [found(0, 3, "tener en cuenta", "Unknown")]
    );
}

#[test]
fn a_deck_card_made_on_the_lemma_chain_is_still_the_one_acted_on() {
    // « + Deck » on « a la vez » made the card `a el vez` and set it « learning »: the card the
    // match opens is that one, not a second card under the name.
    let mut engine = engine();
    engine
        .add_card(
            "a el vez",
            "a la vez",
            "Habla y come a la vez.",
            "https://example.es",
            None,
            T,
            es(),
        )
        .unwrap();
    assert_eq!(
        expressions(&engine, "a la vez"),
        [found(0, 3, "a el vez", "Learning")]
    );
    assert_eq!(engine.deck_count(Some(vec!["es".to_owned()])), 1);
}

#[test]
fn a_status_pulled_from_another_device_keeps_the_chain_and_a_record_on_the_name_wins() {
    let mut engine = engine();
    let pulled = r#"[{"language":"es","lemma":"tener en contar","status":"known","provenance":"manual","updated_at":1000}]"#;
    assert!(matches!(engine.apply_status_changes(pulled), Ok(1)));
    assert_eq!(
        expressions(&engine, "tener en cuenta"),
        [found(0, 3, "tener en contar", "Known")]
    );
    engine
        .set_status_at("tener en cuenta", "learning", 2_000.0, es())
        .unwrap();
    assert_eq!(
        expressions(&engine, "tener en cuenta"),
        [found(0, 3, "tener en cuenta", "Learning")]
    );
}
