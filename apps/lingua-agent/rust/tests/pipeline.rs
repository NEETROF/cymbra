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

//! The ingest → statusline → /vocab pipeline over the committed fixture pack and
//! synthetic Claude Code transcripts.

use std::io::Write;

use lingua_agent::engine::{load_pack, pack_path};
use lingua_agent::ingest::{ingest_texts, run_ingest};
use lingua_agent::source::{ClaudeCodeSource, extract_assistant_texts};
use lingua_agent::statusline::statusline_text;
use lingua_agent::store::Store;
use lingua_agent::vocab::{add_to_deck, vocab_words};
use lingua_core::packs::Pack;

const PACK_BYTES: &[u8] = include_bytes!("fixtures/pack.lingua");

/// English text over the fixture vocabulary (run/city known below 3000; seldom/conundrum unknown).
const REPLY: &str = "The runner runs through the city every morning and they seldom face a strange conundrum in the code.";

fn pack() -> Pack {
    Pack::load(PACK_BYTES).expect("fixture pack loads")
}

fn calibrated_store() -> Store {
    let store = Store::open_in_memory().unwrap();
    store.set_calibration(3000).unwrap();
    store
}

#[test]
fn extract_assistant_texts_handles_claude_code_shapes() {
    let jsonl = concat!(
        r#"{"type":"user","message":{"role":"user","content":"hello"}}"#,
        "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"first reply"},{"type":"tool_use","name":"x"}]}}"#,
        "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":"second reply"}}"#,
        "\n",
        "not json at all",
        "\n",
    );
    let texts = extract_assistant_texts(jsonl);
    assert_eq!(
        texts,
        vec!["first reply".to_string(), "second reply".to_string()]
    );
}

#[test]
fn ingest_counts_lemmas_but_no_transcript_content() {
    let store = calibrated_store();
    let counted = ingest_texts(&store, &pack(), &[REPLY.to_string()], "claude-code", 1000).unwrap();
    assert!(counted > 0);
    // Exposures recorded for the fixture vocabulary.
    assert!(store.exposure("run").unwrap() >= 1);
    assert!(store.exposure("city").unwrap() >= 1);
    assert!(store.exposure("seldom").unwrap() >= 1);
    // The store holds only lemmas — never a sentence from the transcript.
    assert_eq!(store.deck_len().unwrap(), 0);
}

#[test]
fn run_ingest_is_idempotent_on_the_transcript_offset() {
    let dir = std::env::temp_dir().join(format!("lingua-agent-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("transcript.jsonl");
    let line = format!(
        "{{\"type\":\"assistant\",\"message\":{{\"role\":\"assistant\",\"content\":\"{REPLY}\"}}}}\n"
    );
    std::fs::write(&path, &line).unwrap();

    let store = calibrated_store();
    let source = ClaudeCodeSource { path: path.clone() };
    let first = run_ingest(&store, &pack(), &source, "t1", 1000).unwrap();
    assert!(first > 0);
    let before = store.exposure("seldom").unwrap();

    // Re-running over the same (unchanged) transcript ingests nothing new.
    let second = run_ingest(&store, &pack(), &source, "t1", 2000).unwrap();
    assert_eq!(second, 0);
    assert_eq!(store.exposure("seldom").unwrap(), before);

    // Appending a new turn ingests only the new turn.
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    f.write_all(line.as_bytes()).unwrap();
    let third = run_ingest(&store, &pack(), &source, "t1", 3000).unwrap();
    assert!(third > 0);
    assert_eq!(store.exposure("seldom").unwrap(), before + 1);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn statusline_reports_percentage_and_new_words() {
    let store = calibrated_store();
    let ks = store.knowledge_state().unwrap();
    let line = statusline_text(&pack(), &ks, REPLY).expect("analysable");
    assert!(line.starts_with("📖 "));
    assert!(line.contains("%"));
    assert!(line.contains("nouveau"), "line was: {line}");
}

#[test]
fn statusline_is_mute_on_non_analysable_input() {
    let store = calibrated_store();
    let ks = store.knowledge_state().unwrap();
    assert!(statusline_text(&pack(), &ks, "hi").is_none());
}

#[test]
fn vocab_lists_unknown_words_and_add_creates_cards_with_the_sentence() {
    let store = calibrated_store();
    let ks = store.knowledge_state().unwrap();
    let words = vocab_words(&pack(), &ks, &[REPLY.to_string()], 3000);
    let lemmas: Vec<&str> = words.iter().map(|w| w.lemma.as_str()).collect();
    assert!(lemmas.contains(&"seldom"));
    assert!(lemmas.contains(&"conundrum"));
    // seldom carries a gloss from the pack and the source sentence.
    let seldom = words.iter().find(|w| w.lemma == "seldom").unwrap();
    assert_eq!(seldom.gloss.as_deref(), Some("rarement"));
    assert!(seldom.sentence.contains("seldom"));

    let added = add_to_deck(&store, &words, &["seldom".into(), "conundrum".into()], 500).unwrap();
    assert_eq!(added, 2);
    let card = store.card("seldom").unwrap().unwrap();
    assert!(card.provenance.sentence.contains("seldom"));
    assert_eq!(store.deck_len().unwrap(), 2);
}

#[test]
fn engine_path_resolution_honours_env_and_defaults() {
    // The only test that touches these process-global env vars — kept in one function so
    // it never races another test's env access. Exercises every path branch.
    use lingua_agent::engine::lingua_home;

    // SAFETY: single-threaded within this test; no other test reads LINGUA_HOME/PACK.
    unsafe {
        std::env::remove_var("LINGUA_HOME");
        std::env::remove_var("LINGUA_PACK");
    }
    // Defaults: home ends in `.lingua`, pack lives under it.
    assert!(lingua_home().ends_with(".lingua"));
    assert_eq!(pack_path(), lingua_home().join("pack.lingua"));

    let dir = std::env::temp_dir().join(format!("lingua-agent-env-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    unsafe { std::env::set_var("LINGUA_HOME", &dir) };
    assert_eq!(lingua_home(), dir);
    assert_eq!(pack_path(), dir.join("pack.lingua"));

    // LINGUA_PACK overrides everything, and load_pack reads it.
    let pack = dir.join("custom.lingua");
    std::fs::write(&pack, PACK_BYTES).unwrap();
    unsafe { std::env::set_var("LINGUA_PACK", &pack) };
    assert_eq!(pack_path(), pack);
    assert!(load_pack().is_some());

    unsafe {
        std::env::remove_var("LINGUA_HOME");
        std::env::remove_var("LINGUA_PACK");
    }
    std::fs::remove_dir_all(&dir).ok();
}
