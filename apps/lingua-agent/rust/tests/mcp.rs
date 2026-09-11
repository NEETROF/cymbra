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

use lingua_agent::mcp::{dispatch_tool, handle_request};
use lingua_agent::store::Store;
use serde_json::{Value, json};

const DAY: i64 = 86_400;

#[test]
fn add_words_then_list_and_due_reflect_them() {
    let store = Store::open_in_memory().unwrap();
    let added = dispatch_tool(
        &store,
        "add_words",
        &json!({ "words": ["Seldom", " conundrum ", ""] }),
        0,
    )
    .unwrap();
    assert_eq!(added["added"], 2); // empty entry skipped; lemmas normalised

    let decks = dispatch_tool(&store, "list_decks", &json!({}), 0).unwrap();
    assert_eq!(decks["cards"], 2);
    assert_eq!(decks["due"], 2);

    let due = dispatch_tool(&store, "due_cards", &json!({}), 0).unwrap();
    let words: Vec<&str> = due["due"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["word"].as_str().unwrap())
        .collect();
    assert!(words.contains(&"seldom"));
    assert!(words.contains(&"conundrum"));
}

#[test]
fn answer_card_grade_reschedules_it_out() {
    let store = Store::open_in_memory().unwrap();
    dispatch_tool(&store, "add_words", &json!({ "words": ["seldom"] }), 0).unwrap();
    assert_eq!(
        dispatch_tool(&store, "list_decks", &json!({}), 0).unwrap()["due"],
        1
    );

    // A "good" grade schedules the card into the future — not due now, due later.
    let res = dispatch_tool(
        &store,
        "answer_card",
        &json!({ "word": "seldom", "rating": "good" }),
        0,
    )
    .unwrap();
    assert!(res["due_in_days"].as_i64().unwrap() >= 0);
    assert_eq!(
        dispatch_tool(&store, "list_decks", &json!({}), 0).unwrap()["due"],
        0
    );
    assert_eq!(
        dispatch_tool(&store, "list_decks", &json!({}), 100 * DAY).unwrap()["due"],
        1
    );
}

#[test]
fn invalid_inputs_are_rejected() {
    let store = Store::open_in_memory().unwrap();
    assert!(dispatch_tool(&store, "add_words", &json!({ "words": "not-an-array" }), 0).is_err());
    assert!(
        dispatch_tool(
            &store,
            "answer_card",
            &json!({ "word": "x", "rating": "meh" }),
            0
        )
        .is_err()
    );
    assert!(
        dispatch_tool(
            &store,
            "answer_card",
            &json!({ "word": "absent", "rating": "good" }),
            0
        )
        .is_err()
    );
    assert!(dispatch_tool(&store, "nonsense", &json!({}), 0).is_err());
}

#[test]
fn jsonrpc_handshake_and_tool_call() {
    let store = Store::open_in_memory().unwrap();

    let init = handle_request(
        &store,
        &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" }),
        0,
    )
    .unwrap();
    assert_eq!(init["result"]["serverInfo"]["name"], "cymbra-lingua");
    assert!(init["result"]["capabilities"]["tools"].is_object());

    let list = handle_request(
        &store,
        &json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        0,
    )
    .unwrap();
    let names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["list_decks", "add_words", "due_cards", "answer_card"]
    );

    let call = handle_request(
        &store,
        &json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call",
                 "params": { "name": "add_words", "arguments": { "words": ["seldom"] } } }),
        0,
    )
    .unwrap();
    let text = call["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(serde_json::from_str::<Value>(text).unwrap()["added"], 1);

    // A notification (no id) yields no response.
    assert!(
        handle_request(
            &store,
            &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
            0
        )
        .is_none()
    );
}
