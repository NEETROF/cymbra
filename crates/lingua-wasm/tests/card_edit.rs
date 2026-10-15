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

//! The card edit through the bindings the extension drives (add-lingua-card-edit D3, D7, D10):
//! `reviewEdit`, and the removals `exportCardOps` sends and `applyCardOps` applies. Host only:
//! en-fr built from the committed tables, its expression table holding `four-poster bed`.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use lingua_wasm::LinguaEngine;
use serde_json::Value;
use support::Scenario;

const SENTENCE: &str = "She slept in a four-poster bed, alone.";
const OLD: &str = "four-poster bed,";
const NEW: &str = "four-poster bed";
const PACK_GLOSS: &str = "Lit à baldaquin";

fn engine() -> LinguaEngine {
    Scenario::engine(&[("en-fr", support::committed::en_fr_holding_four_poster_bed())])
}

fn json(text: &str) -> Value {
    serde_json::from_str(text).expect("JSON")
}

fn view(engine: &LinguaEngine) -> Value {
    json(&engine.review_current().expect("a card under review"))
}

fn capture(engine: &mut LinguaEngine, lemma: &str, gloss: Option<&str>, at: f64) {
    engine
        .add_card(
            lemma,
            lemma,
            SENTENCE,
            "https://example.com",
            gloss.map(str::to_owned),
            at,
            None,
        )
        .unwrap();
}

/// An engine reviewing the cards `lemmas`, captured in that order, the first revealed.
fn reviewing(lemmas: &[(&str, Option<&str>)]) -> LinguaEngine {
    let mut engine = engine();
    for (i, (lemma, gloss)) in lemmas.iter().enumerate() {
        capture(&mut engine, lemma, *gloss, 100.0 + i as f64);
    }
    engine.start_review(200.0, None, None, None, None, None);
    engine.review_reveal();
    engine
}

fn card_ops(engine: &LinguaEngine) -> Vec<Value> {
    json(&engine.export_card_ops())
        .as_array()
        .expect("an array")
        .clone()
}

#[test]
fn spec_scenario_trailing_punctuation_renames_the_card_and_removes_the_old_one() {
    let mut engine = reviewing(&[(OLD, None)]);
    assert_eq!(
        view(&engine)["gloss"],
        PACK_GLOSS,
        "the pack read at the trimmed lemma"
    );
    let answer = json(&engine.review_edit(OLD, PACK_GLOSS, 300.0));
    assert_eq!(answer, json(r#"{"ok":true,"changed":true}"#));
    let shown = view(&engine);
    assert_eq!(shown["headword"], NEW);
    assert_eq!(shown["surface"], NEW);
    assert_eq!(shown["sentence"], SENTENCE);
    assert_eq!(shown["gloss"], PACK_GLOSS);
    assert_eq!(shown["revealed"], true);

    let ops = card_ops(&engine);
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0]["lemma"], NEW);
    assert_eq!(ops[0]["client_ts"], 300_000);
    assert!(ops[0].get("gloss_by_reader").is_none(), "the pack's gloss");
    assert_eq!(
        ops[1],
        json(
            r#"{"client_id":"four-poster bed,","language":"en","lemma":"four-poster bed,",
                "surface_form":"","source_sentence":"","source":"","gloss":"","fsrs_state":"",
                "deleted":true,"client_ts":300000,"device_id":""}"#
        )
    );
    let statuses = json(&engine.export_status_ops());
    let of = |lemma: &str| {
        statuses
            .as_array()
            .unwrap()
            .iter()
            .find(|op| op["lemma"] == lemma)
            .map(|op| (op["status"].clone(), op["updated_at"].clone()))
    };
    assert_eq!(of(OLD), Some(("cleared".into(), 300_000.into())));
    assert_eq!(of(NEW), Some(("learning".into(), 300_000.into())));
}

#[test]
fn spec_scenario_a_refused_save_changes_nothing() {
    let too_long_form = "a".repeat(121);
    let too_long_gloss = "é".repeat(201);
    let cases = [
        (" , ", "", "empty"),
        (too_long_form.as_str(), "", "form_too_long"),
        (NEW, too_long_gloss.as_str(), "gloss_too_long"),
    ];
    for (form, gloss, error) in cases {
        let mut engine = reviewing(&[(OLD, None)]);
        let before = engine.backup();
        let answer = json(&engine.review_edit(form, gloss, 300.0));
        assert_eq!(answer["ok"], false, "{error}");
        assert_eq!(answer["error"], error);
        assert_eq!(engine.backup(), before, "{error}: nothing changed");
    }
    // Nothing under review: nothing to edit.
    let mut idle = engine();
    assert_eq!(
        json(&idle.review_edit(NEW, "", 300.0)),
        json(r#"{"ok":true,"changed":false}"#)
    );
}

#[test]
fn spec_scenario_writing_a_missing_translation_marks_it_as_the_readers() {
    let mut engine = reviewing(&[("blorft", None)]);
    assert_eq!(view(&engine)["gloss"], Value::Null);
    let answer = json(&engine.review_edit("blorft", " truc ", 300.0));
    assert_eq!(answer["changed"], true);
    assert_eq!(view(&engine)["gloss"], "truc");
    let ops = card_ops(&engine);
    assert_eq!(ops.len(), 1, "nothing removed");
    assert_eq!(ops[0]["gloss"], "truc");
    assert_eq!(ops[0]["gloss_by_reader"], true);
    // A save that changes nothing writes nothing.
    let before = engine.backup();
    assert_eq!(
        json(&engine.review_edit("blorft", "truc", 400.0))["changed"],
        false
    );
    assert_eq!(engine.backup(), before);
}

#[test]
fn spec_scenario_two_cards_for_one_expression_merge() {
    let mut engine = reviewing(&[("colloquially four-poster bed", None), (NEW, None)]);
    assert_eq!(engine.review_remaining(), 2);
    let answer = json(&engine.review_edit("Four-poster bed", "", 300.0));
    assert_eq!(
        answer,
        json(r#"{"ok":true,"changed":true,"merged":"four-poster bed"}"#)
    );
    assert_eq!(engine.deck_count(None), 1);
    assert_eq!(
        engine.review_remaining(),
        1,
        "the card merged into does not come twice"
    );
    assert_eq!(view(&engine)["headword"], NEW);
}

#[test]
fn spec_scenario_a_form_already_known_retires_the_card() {
    let mut engine = reviewing(&[(OLD, None), ("blorft", None)]);
    engine.set_status_at(NEW, "known", 150_000.0, None).unwrap();
    let answer = json(&engine.review_edit(NEW, "", 300.0));
    assert_eq!(answer["retired"], true);
    assert_eq!(view(&engine)["headword"], "blorft", "the review moved on");
    assert_eq!(
        engine.due_count(1e12, None),
        1,
        "the renamed card is retired"
    );
}

#[test]
fn spec_scenario_a_rename_reaches_another_device_and_a_stale_copy_does_not_bring_it_back() {
    let mut phone = reviewing(&[(OLD, None)]);
    let stale = card_ops(&phone)[0].clone();
    phone.review_edit(NEW, PACK_GLOSS, 300.0);
    let renamed = serde_json::to_string(&card_ops(&phone)).unwrap();

    let mut mac = engine();
    capture(&mut mac, OLD, None, 100.0);
    assert_eq!(mac.apply_card_ops(&renamed).unwrap(), 2);
    assert_eq!(mac.deck_count(None), 1);
    mac.start_review(400.0, None, None, None, None, None);
    assert_eq!(view(&mac)["headword"], NEW);
    // The same ops again change nothing; the older copy of the removed card is refused.
    assert_eq!(mac.apply_card_ops(&renamed).unwrap(), 0);
    let stale = serde_json::to_string(&[stale]).unwrap();
    assert_eq!(mac.apply_card_ops(&stale).unwrap(), 0);
    assert_eq!(mac.deck_count(None), 1);
    // Captured again later: the card exists again, and goes up as a card, not a removal.
    capture(&mut mac, OLD, None, 500.0);
    assert_eq!(mac.deck_count(None), 2);
    assert!(card_ops(&mac).iter().all(|op| op["deleted"] == false));

    // A device that graded the card after the removal keeps it.
    let mut tablet = reviewing(&[(OLD, None)]);
    tablet.review_grade("good", 350.0);
    tablet.apply_card_ops(&renamed).unwrap();
    assert_eq!(tablet.deck_count(None), 2, "kept, beside the new one");
}
