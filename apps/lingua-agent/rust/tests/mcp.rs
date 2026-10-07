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

use lingua_agent::mcp::{Followed, dispatch_tool, handle_request};
use lingua_agent::store::Store;
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::knowledge::profile::NativeLanguage;
use serde_json::{Value, json};

const DAY: i64 = 86_400;
/// One language followed, glossed in French: the tools need no `language`.
const EN: &Followed = &Followed {
    languages: &[StudiedLanguage::English],
    native: NativeLanguage::French,
};
/// Two followed: the tools ask which (add-lingua-agent-languages D6).
const BOTH: &Followed = &Followed {
    languages: &[StudiedLanguage::English, StudiedLanguage::Spanish],
    native: NativeLanguage::French,
};

#[test]
fn add_words_then_list_and_due_reflect_them() {
    let store = Store::open_in_memory().unwrap();
    let added = dispatch_tool(
        &store,
        EN,
        "add_words",
        &json!({ "words": ["Seldom", " conundrum ", ""] }),
        0,
    )
    .unwrap();
    assert_eq!(added["added"], 2); // empty entry skipped; lemmas normalised

    let decks = dispatch_tool(&store, EN, "list_decks", &json!({}), 0).unwrap();
    assert_eq!(decks["cards"], 2);
    assert_eq!(decks["due"], 2);

    let due = dispatch_tool(&store, EN, "due_cards", &json!({}), 0).unwrap();
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
    dispatch_tool(&store, EN, "add_words", &json!({ "words": ["seldom"] }), 0).unwrap();
    assert_eq!(
        dispatch_tool(&store, EN, "list_decks", &json!({}), 0).unwrap()["due"],
        1
    );

    // A "good" grade schedules the card into the future — not due now, due later.
    let res = dispatch_tool(
        &store,
        EN,
        "answer_card",
        &json!({ "word": "seldom", "rating": "good" }),
        0,
    )
    .unwrap();
    assert!(res["due_in_days"].as_i64().unwrap() >= 0);
    assert_eq!(
        dispatch_tool(&store, EN, "list_decks", &json!({}), 0).unwrap()["due"],
        0
    );
    assert_eq!(
        dispatch_tool(&store, EN, "list_decks", &json!({}), 100 * DAY).unwrap()["due"],
        1
    );
}

#[test]
fn invalid_inputs_are_rejected() {
    let store = Store::open_in_memory().unwrap();
    assert!(
        dispatch_tool(
            &store,
            EN,
            "add_words",
            &json!({ "words": "not-an-array" }),
            0
        )
        .is_err()
    );
    assert!(
        dispatch_tool(
            &store,
            EN,
            "answer_card",
            &json!({ "word": "x", "rating": "meh" }),
            0
        )
        .is_err()
    );
    assert!(
        dispatch_tool(
            &store,
            EN,
            "answer_card",
            &json!({ "word": "absent", "rating": "good" }),
            0
        )
        .is_err()
    );
    assert!(dispatch_tool(&store, EN, "nonsense", &json!({}), 0).is_err());
}

#[test]
fn jsonrpc_handshake_and_tool_call() {
    let store = Store::open_in_memory().unwrap();

    let init = handle_request(
        &store,
        EN,
        &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" }),
        0,
    )
    .unwrap();
    assert_eq!(init["result"]["serverInfo"]["name"], "cymbra-lingua");
    assert!(init["result"]["capabilities"]["tools"].is_object());

    let list = handle_request(
        &store,
        EN,
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
        EN,
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
            EN,
            &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
            0
        )
        .is_none()
    );
}

#[test]
fn with_two_languages_a_tool_asks_which_and_keeps_to_one() {
    let store = Store::open_in_memory().unwrap();
    let refused = dispatch_tool(&store, BOTH, "due_cards", &json!({}), 0).unwrap_err();
    assert!(refused.contains("en, es"), "{refused}");
    assert!(dispatch_tool(&store, BOTH, "add_words", &json!({ "words": ["casa"] }), 0).is_err());

    dispatch_tool(
        &store,
        BOTH,
        "add_words",
        &json!({ "words": ["casa", "vino"], "language": "es" }),
        0,
    )
    .unwrap();
    dispatch_tool(
        &store,
        BOTH,
        "add_words",
        &json!({ "words": ["seldom"], "language": "en" }),
        0,
    )
    .unwrap();
    let due = dispatch_tool(&store, BOTH, "due_cards", &json!({ "language": "es" }), 0).unwrap();
    let words: Vec<&str> = due["due"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["word"].as_str().unwrap())
        .collect();
    assert_eq!(words, vec!["casa", "vino"]); // the Spanish cards only
    assert_eq!(due["language"], "es");

    let graded = dispatch_tool(
        &store,
        BOTH,
        "answer_card",
        &json!({ "word": "casa", "rating": "good", "language": "es" }),
        0,
    );
    assert!(graded.is_ok());
    assert!(
        dispatch_tool(
            &store,
            BOTH,
            "answer_card",
            &json!({ "word": "casa", "rating": "good", "language": "en" }),
            0
        )
        .is_err()
    );

    let decks = dispatch_tool(&store, BOTH, "list_decks", &json!({}), 0).unwrap();
    assert_eq!(decks["cards"], 3);
    assert_eq!(
        decks["languages"][0],
        json!({ "language": "en", "cards": 1, "due": 1 })
    );
    assert_eq!(
        decks["languages"][1],
        json!({ "language": "es", "cards": 2, "due": 1 })
    );
    let spanish =
        dispatch_tool(&store, BOTH, "list_decks", &json!({ "language": "es" }), 0).unwrap();
    assert_eq!(spanish["cards"], 2);
}

#[test]
fn a_language_not_followed_is_refused() {
    let store = Store::open_in_memory().unwrap();
    let refused =
        dispatch_tool(&store, EN, "due_cards", &json!({ "language": "es" }), 0).unwrap_err();
    assert!(refused.contains("not followed"), "{refused}");
    assert!(dispatch_tool(&store, BOTH, "list_decks", &json!({ "language": "fr" }), 0).is_err());
}

#[test]
fn every_tool_says_it_takes_a_language() {
    let store = Store::open_in_memory().unwrap();
    let list = handle_request(
        &store,
        BOTH,
        &json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }),
        0,
    )
    .unwrap();
    for tool in list["result"]["tools"].as_array().unwrap() {
        assert!(
            tool["inputSchema"]["properties"]["language"].is_object(),
            "{}",
            tool["name"]
        );
    }
}

#[test]
fn spec_scenario_a_card_created_on_an_engine_glossed_in_english() {
    // lingua-decks-review (add-lingua-card-gloss-language D2): the agent labels a card with the
    // native language the packs it follows are glossed in.
    const GLOSSED_IN_ENGLISH: &Followed = &Followed {
        languages: &[StudiedLanguage::English],
        native: NativeLanguage::English,
    };
    let store = Store::open_in_memory().unwrap();
    dispatch_tool(
        &store,
        GLOSSED_IN_ENGLISH,
        "add_words",
        &json!({ "words": ["seldom"] }),
        0,
    )
    .unwrap();
    dispatch_tool(
        &store,
        EN,
        "add_words",
        &json!({ "words": ["conundrum"] }),
        0,
    )
    .unwrap();
    let label = |word: &str| {
        store
            .card(StudiedLanguage::English, word)
            .unwrap()
            .unwrap()
            .gloss_language
    };
    assert_eq!(label("seldom"), "en");
    assert_eq!(label("conundrum"), "fr");
}
