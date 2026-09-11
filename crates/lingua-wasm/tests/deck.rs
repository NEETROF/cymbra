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

//! Native exercise of the deck / review / backup bindings the extension drives
//! (`add-lingua-extension-review`). Runs on the host over the committed fixture
//! pack; the wasm surface is the same methods.

use lingua_wasm::LinguaEngine;

const PACK: &[u8] = include_bytes!("fixtures/pack.lingua");

fn engine() -> LinguaEngine {
    match LinguaEngine::new(PACK) {
        Ok(e) => e,
        Err(_) => panic!("fixture pack loads"),
    }
}

#[test]
fn add_card_marks_learning_and_grows_the_deck() {
    let mut e = engine();
    e.set_calibration(3_000);
    e.add_card(
        "seldom",
        "seldom",
        "They seldom ship.",
        "https://example.com",
        Some("rarement".into()),
        100.0,
    );
    assert_eq!(e.deck_count(), 1);
    assert_eq!(e.tracked_count(), 1);
    // A fresh card is due now.
    assert_eq!(e.due_count(200.0), 1);
}

#[test]
fn review_session_walks_due_cards() {
    let mut e = engine();
    e.add_card("seldom", "seldom", "s1", "https://x", None, 0.0);
    e.add_card("conundrum", "conundrum", "s2", "https://x", None, 0.0);
    assert_eq!(e.start_review(10.0), 2);
    assert!(e.review_current().is_some());
    assert_eq!(e.review_remaining(), 2);
    e.review_reveal();
    e.review_grade("good", 10.0);
    assert_eq!(e.review_remaining(), 1);
    e.review_grade("again", 10.0);
    assert_eq!(e.review_remaining(), 0);
    assert!(e.review_current().is_none());
}

#[test]
fn mark_known_retires_the_card() {
    let mut e = engine();
    e.add_card("seldom", "seldom", "s", "https://x", None, 0.0);
    e.start_review(10.0);
    e.review_mark_known(10.0);
    // The card is kept but never comes due again.
    assert_eq!(e.deck_count(), 1);
    assert_eq!(e.due_count(f64::from(i32::MAX)), 0);
}

#[test]
fn backup_restores_losslessly_into_a_fresh_engine() {
    let mut e = engine();
    e.set_calibration(2_500);
    e.set_status("ship", "known");
    e.add_card(
        "seldom",
        "seldom",
        "They seldom ship.",
        "https://example.com",
        Some("rarement".into()),
        100.0,
    );
    let backup = e.backup();

    let mut restored = engine();
    match restored.restore(&backup) {
        Ok(()) => {}
        Err(_) => panic!("restore accepts our own backup"),
    }
    assert_eq!(restored.calibration(), 2_500);
    assert_eq!(restored.deck_count(), 1);
    // The backup is itself a fixpoint.
    assert_eq!(restored.backup(), backup);
}

// Note: the restore error path (malformed / unsupported version) is covered by
// lingua-core's backup.rs tests. It cannot be exercised here because the wrapper
// maps failures to `JsError`, whose constructor panics on a non-wasm target.

#[test]
fn reset_clears_the_whole_state() {
    let mut e = engine();
    e.set_calibration(2_000);
    e.add_card("seldom", "seldom", "s", "https://x", None, 0.0);
    e.reset();
    assert_eq!(e.deck_count(), 0);
    assert_eq!(e.tracked_count(), 0);
    assert_eq!(e.calibration(), 0);
}

#[test]
fn notice_and_licences_come_from_the_pack() {
    let e = engine();
    // The testdata pack ships a NOTICE and three source licences.
    assert!(!e.notice().is_empty());
    let licences: Vec<String> = serde_json::from_str(&e.licences()).expect("licences json");
    assert!(!licences.is_empty());
}
