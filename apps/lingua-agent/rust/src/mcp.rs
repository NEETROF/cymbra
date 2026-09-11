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

//! The MCP server: deck operations for the agent (`list_decks`, `add_words`,
//! `due_cards`, `answer_card`), over JSON-RPC on stdio. Review on the plugin side is a
//! conversational quiz — `due_cards` → the agent asks → `answer_card` grades. Inputs are
//! validated (lemmas normalised, sizes bounded). The request dispatch is pure over the
//! store (unit-tested); the stdin/stdout loop is the thin transport.

use std::io::{BufRead, Write};

use lingua_core::decks::card::{Card, EncounterSource, Provenance};
use lingua_core::decks::fsrs::{FsrsParams, Rating};
use lingua_core::knowledge::status::Status;
use serde_json::{Value, json};

use crate::store::Store;

/// Maximum words accepted by `add_words` in one call.
const MAX_ADD_WORDS: usize = 100;

const PROTOCOL_VERSION: &str = "2024-11-05";

/// The tool catalogue advertised by `tools/list`.
fn tools() -> Value {
    json!([
        { "name": "list_decks", "description": "Summarise the local deck (card count and how many are due now).",
          "inputSchema": { "type": "object", "properties": {} } },
        { "name": "add_words", "description": "Add dictionary forms to the deck as learning cards.",
          "inputSchema": { "type": "object", "properties": { "words": { "type": "array", "items": { "type": "string" } } }, "required": ["words"] } },
        { "name": "due_cards", "description": "List the cards due for review now (dictionary form, gloss, sentence).",
          "inputSchema": { "type": "object", "properties": {} } },
        { "name": "answer_card", "description": "Grade a due card (again/hard/good/easy), updating its FSRS schedule.",
          "inputSchema": { "type": "object", "properties": { "word": { "type": "string" }, "rating": { "type": "string", "enum": ["again", "hard", "good", "easy"] } }, "required": ["word", "rating"] } }
    ])
}

/// Dispatch a `tools/call` against the store. Returns the tool's structured result, or
/// an `Err(message)` for invalid input. Pure over the store (unit-tested).
pub fn dispatch_tool(store: &Store, name: &str, args: &Value, now: i64) -> Result<Value, String> {
    match name {
        "list_decks" => {
            let cards = store.deck_len().map_err(db)?;
            let due = store.due_cards(now).map_err(db)?.len();
            Ok(json!({ "cards": cards, "due": due }))
        }
        "add_words" => {
            let words = args
                .get("words")
                .and_then(Value::as_array)
                .ok_or("`words` must be an array")?;
            if words.len() > MAX_ADD_WORDS {
                return Err(format!("too many words (max {MAX_ADD_WORDS})"));
            }
            let mut added = 0;
            for w in words {
                let Some(raw) = w.as_str() else { continue };
                let lemma = raw.trim().to_lowercase();
                if lemma.is_empty() {
                    continue;
                }
                store.set_status(&lemma, Status::Learning).map_err(db)?;
                let card = Card::new(
                    &lemma,
                    &lemma,
                    Provenance {
                        sentence: String::new(),
                        source: EncounterSource::AgentSession {
                            session: "claude-code".to_owned(),
                        },
                        captured_at: now,
                    },
                    None,
                );
                store.upsert_card(&card).map_err(db)?;
                added += 1;
            }
            Ok(json!({ "added": added }))
        }
        "due_cards" => {
            let due: Vec<Value> = store
                .due_cards(now)
                .map_err(db)?
                .iter()
                .map(|c| json!({ "word": c.lemma, "gloss": c.gloss, "sentence": c.provenance.sentence }))
                .collect();
            Ok(json!({ "due": due }))
        }
        "answer_card" => {
            let word = args
                .get("word")
                .and_then(Value::as_str)
                .ok_or("`word` is required")?
                .trim()
                .to_lowercase();
            let rating = args
                .get("rating")
                .and_then(Value::as_str)
                .ok_or("`rating` is required")?;
            let rating = parse_rating(rating).ok_or("`rating` must be again/hard/good/easy")?;
            let mut card = store
                .card(&word)
                .map_err(db)?
                .ok_or_else(|| format!("no card for \"{word}\""))?;
            card.review.grade(&FsrsParams::default(), rating, now);
            store.upsert_card(&card).map_err(db)?;
            Ok(json!({ "word": word, "due_in_days": days_until(card.review.due, now) }))
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

/// Handle one JSON-RPC request; `None` for a notification (no response expected).
pub fn handle_request(store: &Store, req: &Value, now: i64) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    match method {
        "initialize" => Some(result(
            id,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "cymbra-lingua", "version": env!("CARGO_PKG_VERSION") }
            }),
        )),
        "tools/list" => Some(result(id, json!({ "tools": tools() }))),
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or(Value::Null);
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match dispatch_tool(store, name, &args, now) {
                Ok(value) => Some(result(id, tool_content(&value))),
                Err(message) => Some(error(id, message)),
            }
        }
        // Notifications (e.g. notifications/initialized) expect no response.
        _ if id.is_none() => None,
        _ => Some(error(id, format!("unknown method: {method}"))),
    }
}

/// Serve MCP over stdin/stdout (newline-delimited JSON-RPC). Thin transport.
pub fn serve(store: &Store, now_fn: impl Fn() -> i64) -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(req) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if let Some(response) = handle_request(store, &req, now_fn()) {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

fn result(id: Option<Value>, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Option<Value>, message: String) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32000, "message": message } })
}

/// Wrap a tool result as MCP tool content (a text block of its JSON).
fn tool_content(value: &Value) -> Value {
    json!({ "content": [{ "type": "text", "text": value.to_string() }] })
}

fn parse_rating(s: &str) -> Option<Rating> {
    match s {
        "again" => Some(Rating::Again),
        "hard" => Some(Rating::Hard),
        "good" => Some(Rating::Good),
        "easy" => Some(Rating::Easy),
        _ => None,
    }
}

fn days_until(due: Option<i64>, now: i64) -> i64 {
    due.map_or(0, |d| (d - now).max(0) / 86_400)
}

fn db(e: rusqlite::Error) -> String {
    format!("store error: {e}")
}
