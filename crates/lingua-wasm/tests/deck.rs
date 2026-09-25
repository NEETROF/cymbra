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
fn review_card_shows_where_the_word_was_met() {
    let mut e = engine();
    e.add_card(
        "seldom",
        "seldom",
        "They seldom ship.",
        "The Hound of the Baskervilles · I: Mr. Sherlock Holmes",
        None,
        0.0,
    );
    e.start_review(10.0);
    let view: serde_json::Value = match e.review_current().map(|j| serde_json::from_str(&j)) {
        Some(Ok(v)) => v,
        _ => panic!("a card is under review"),
    };
    assert_eq!(
        view["source"],
        "The Hound of the Baskervilles · I: Mr. Sherlock Holmes"
    );
    assert_eq!(view["sentence"], "They seldom ship.");
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

#[test]
fn a_card_syncs_without_its_page_address_and_keeps_it_locally() {
    // add-lingua-privacy-controls: the page stays on the device.
    let mut e = engine();
    e.add_card(
        "seldom",
        "seldom",
        "They seldom ship.",
        "https://example.com/article",
        Some("rarement".into()),
        100.0,
    );
    let ops: serde_json::Value = serde_json::from_str(&e.export_card_ops()).unwrap();
    assert_eq!(ops[0]["source"], "");
    assert_eq!(ops[0]["source_sentence"], "They seldom ship.");

    // A later edit from another device arrives without an address…
    let mut edited = ops[0].clone();
    edited["surface_form"] = "Seldom".into();
    edited["client_ts"] = (ops[0]["client_ts"].as_i64().unwrap() + 60_000).into();
    let pulled = serde_json::Value::Array(vec![edited]).to_string();
    match e.apply_card_ops(&pulled) {
        Ok(changed) => assert_eq!(changed, 1),
        Err(_) => panic!("pulled card applies"),
    }
    // …and the local backup still knows where the word was captured.
    assert!(e.backup().contains("https://example.com/article"));
    let again: serde_json::Value = serde_json::from_str(&e.export_card_ops()).unwrap();
    assert_eq!(again[0]["surface_form"], "Seldom");
    assert_eq!(again[0]["source"], "");
}

#[test]
fn adding_to_the_deck_stamps_the_learning_decision() {
    // A word put in the deck is a dated decision: unstamped, it would lose last-write-wins
    // to any decision made on another device, even an older one.
    let mut e = engine();
    e.add_card("seldom", "seldom", "They seldom ship.", "", None, 1_700.0);

    let ops: serde_json::Value = serde_json::from_str(&e.export_status_ops()).unwrap();
    let op = ops
        .as_array()
        .and_then(|a| a.iter().find(|o| o["lemma"] == "seldom"))
        .expect("the status went to the outbox");
    assert_eq!(op["status"], "learning");
    assert_eq!(op["updated_at"], 1_700_000); // the capture time, in millis
}

#[test]
fn a_word_marked_known_elsewhere_retires_its_card_here() {
    // The device that marked it known had no card to retire; this one does.
    let mut e = engine();
    e.add_card("seldom", "seldom", "They seldom ship.", "", None, 1_700.0);
    assert_eq!(e.due_count(2_000.0), 1);

    let pulled = r#"[{"language":"en","lemma":"seldom","status":"known","updated_at":1800000}]"#;
    match e.apply_status_changes(pulled) {
        Ok(changed) => assert_eq!(changed, 1),
        Err(_) => panic!("pulled status applies"),
    }

    assert_eq!(e.deck_count(), 1); // the card is kept…
    assert_eq!(e.due_count(f64::from(i32::MAX)), 0); // …and never comes due again
    let card: serde_json::Value = serde_json::from_str(&e.export_card_ops()).unwrap();
    assert!(card[0]["client_ts"].as_i64().unwrap() >= 1_800_000); // the retirement syncs back
}

#[test]
fn retiring_on_a_pulled_known_never_dates_the_card_backwards() {
    // The card was edited here (a review) AFTER the status was last stamped, so a known
    // pulled in between wins the status while being older than the card.
    let mut e = engine();
    e.add_card("seldom", "seldom", "They seldom ship.", "", None, 1_000.0);
    e.start_review(9_000.0);
    e.review_reveal();
    e.review_grade("good", 9_000.0);
    let before: serde_json::Value = serde_json::from_str(&e.export_card_ops()).unwrap();
    assert_eq!(before[0]["client_ts"], 9_000_000);

    let pulled = r#"[{"language":"en","lemma":"seldom","status":"known","updated_at":2000000}]"#;
    match e.apply_status_changes(pulled) {
        Ok(changed) => assert_eq!(changed, 1), // the status is newer than the local stamp
        Err(_) => panic!("pulled status applies"),
    }

    let after: serde_json::Value = serde_json::from_str(&e.export_card_ops()).unwrap();
    assert_eq!(after[0]["client_ts"], 9_000_000); // the card keeps its own, later date
    assert_eq!(e.due_count(f64::from(i32::MAX)), 0); // and it is retired all the same
}

#[test]
fn reading_counts_nothing_without_a_declared_level() {
    // Exposure counters exist only to confirm a presumed word by repeated reading, which
    // never happens without a declared level. Counting anyway filled the browser's storage.
    let mut e = engine();
    let read: Vec<String> = (0..500).map(|i| format!("word{i}")).collect();

    e.record_exposures(
        read.clone(),
        "reading:en.wikipedia.org",
        1_700_000_000_000.0,
    );

    assert!(
        e.backup().len() < 2_000,
        "backup: {} bytes",
        e.backup().len()
    );
}

#[test]
fn restoring_a_bloated_backup_prunes_it() {
    // What an older build wrote: a counter for every word ever met, none of which any level
    // presumed. Restoring must shrink it, so a saturated store heals on the first load.
    let counters: serde_json::Map<String, serde_json::Value> = (0..2_000)
        .map(|i| {
            (
                format!("word{i}"),
                serde_json::json!({
                    "occurrences": 3,
                    "last_source": "reading:en.wikipedia.org",
                    "last_seen": 1_700_000_000_i64,
                    "last_day": 19_675,
                    "distinct_days": 2
                }),
            )
        })
        .collect();
    let mut backup: serde_json::Value = serde_json::from_str(&engine().backup()).unwrap();
    backup["exposure"]["counters"] = serde_json::json!({ "English": counters });
    let bloated = backup.to_string();
    assert!(
        bloated.len() > 200_000,
        "the fixture is big: {}",
        bloated.len()
    );

    let mut e = engine();
    match e.restore(&bloated) {
        Ok(()) => {}
        Err(_) => panic!("the backup restores"),
    }

    assert!(
        e.backup().len() < 5_000,
        "pruned: {} bytes",
        e.backup().len()
    );
}
