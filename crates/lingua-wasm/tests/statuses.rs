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

//! Native exercise of the "Remettre à apprendre" path the extension's marked-words
//! list drives: `setStatusAt(lemma, "clear", now)` on a word the reader no longer
//! wants treated as known. Runs on the host over the committed fixture pack, which
//! carries no CEFR levels — so these use the frequency calibration, where `city`
//! (ranked below 3,000) is presumed known. The declared-level and exposure-promotion
//! variants are covered in lingua-core's `knowledge::state` tests.

use lingua_wasm::LinguaEngine;

const PACK: &[u8] = include_bytes!("fixtures/pack.lingua");
/// What the extension's engine adapter sends for a `null` status.
const CLEAR: &str = "clear";

fn engine() -> LinguaEngine {
    match LinguaEngine::new(PACK) {
        Ok(e) => e,
        Err(_) => panic!("fixture pack loads"),
    }
}

/// A calibrated reader, as the extension sets one up by default.
fn calibrated() -> LinguaEngine {
    let mut e = engine();
    e.set_calibration(3_000);
    e
}

/// The highlight class the page would paint for `lemma` in a sentence that uses it.
fn class_of(e: &LinguaEngine, lemma: &str) -> String {
    let analysis: serde_json::Value = serde_json::from_str(&e.analyse(vec![
        "The runner runs through many cities every morning before work.".to_owned(),
    ]))
    .expect("analysis json");
    analysis["tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .find(|t| t["lemma"] == lemma)
        .and_then(|t| t["class"].as_str())
        .expect("the lemma is in the sentence")
        .to_owned()
}

#[test]
fn putting_back_a_known_word_the_calibration_presumes_resurfaces_it() {
    let mut e = calibrated();
    e.set_status_at("city", "known", 10.0);
    assert_eq!(class_of(&e, "city"), "Known");

    e.set_status_at("city", CLEAR, 20.0);
    assert_eq!(
        class_of(&e, "city"),
        "Unknown",
        "the word is highlighted again"
    );
}

#[test]
fn putting_a_word_back_syncs_to_other_devices() {
    let mut phone = calibrated();
    phone.set_status_at("city", "known", 10.0);
    phone.set_status_at("city", CLEAR, 20.0);

    // The undo is pushed, stamped with the time it was made.
    let ops: serde_json::Value =
        serde_json::from_str(&phone.export_status_ops()).expect("status ops json");
    let city = ops
        .as_array()
        .expect("an array")
        .iter()
        .find(|o| o["lemma"] == "city")
        .expect("the undo is exported");
    assert_eq!(city["status"], "cleared");
    assert_eq!(city["updated_at"], 20);

    // A laptop that only saw the earlier "known" converges on the undo.
    let mut laptop = calibrated();
    laptop.set_status_at("city", "known", 10.0);
    let pulled = r#"[{"language":"en","lemma":"city","status":"cleared","provenance":"manual","updated_at":20}]"#;
    assert!(matches!(laptop.apply_status_changes(pulled), Ok(1)));
    assert_eq!(class_of(&laptop, "city"), "Unknown");
}
