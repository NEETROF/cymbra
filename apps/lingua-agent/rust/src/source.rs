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

//! Session sources (design D3): locating an agent's sessions and extracting the
//! assistant-message text, of which Claude Code is the first implementation. A new
//! agent (Codex, Aider) implements the same trait without touching the analysis
//! pipeline. Parsing is defensive: transcript formats are internal JSONL, not a
//! contract, so an unrecognised line is skipped, never fatal.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

/// A source of AI-agent sessions.
pub trait SessionSource {
    /// Extract the assistant texts appended since `from_offset` bytes. Returns the new
    /// byte offset (a line boundary, so re-runs are idempotent) and the texts in order.
    fn extract(&self, from_offset: u64) -> std::io::Result<(u64, Vec<String>)>;
}

/// A Claude Code transcript (a JSONL file at `transcript_path`).
pub struct ClaudeCodeSource {
    pub path: PathBuf,
}

impl SessionSource for ClaudeCodeSource {
    fn extract(&self, from_offset: u64) -> std::io::Result<(u64, Vec<String>)> {
        let bytes = fs::read(&self.path)?;
        let from = (from_offset as usize).min(bytes.len());
        let tail = &bytes[from..];
        // Only consume complete lines; leave a partial trailing line for next time.
        let last_nl = tail.iter().rposition(|&b| b == b'\n');
        let Some(end) = last_nl else {
            return Ok((from_offset, Vec::new()));
        };
        let complete = &tail[..=end];
        let text = String::from_utf8_lossy(complete);
        let texts = extract_assistant_texts(&text);
        Ok((from as u64 + end as u64 + 1, texts))
    }
}

/// Extract the assistant-message texts from a chunk of Claude Code JSONL. Pure over the
/// input string (unit-tested with synthetic transcripts). Non-assistant or malformed
/// lines are skipped.
pub fn extract_assistant_texts(jsonl: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in jsonl.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if !is_assistant(&value) {
            continue;
        }
        // The message body is at `.message` (nested) or the object itself.
        let body = value.get("message").unwrap_or(&value);
        let content = body.get("content").unwrap_or(&Value::Null);
        if let Some(text) = text_of(content)
            && !text.trim().is_empty()
        {
            out.push(text);
        }
    }
    out
}

fn is_assistant(value: &Value) -> bool {
    let top = value.get("type").and_then(Value::as_str);
    let role = value.get("role").and_then(Value::as_str).or_else(|| {
        value
            .get("message")
            .and_then(|m| m.get("role"))
            .and_then(Value::as_str)
    });
    top == Some("assistant") || role == Some("assistant")
}

/// The plain text of a message `content`: a bare string, or the concatenation of the
/// `text` parts of a content-block array (ignoring tool calls, thinking, etc.).
fn text_of(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => Some(s.clone()),
        Value::Array(items) => {
            let parts: Vec<&str> = items
                .iter()
                .filter(|item| item.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|item| item.get("text").and_then(Value::as_str))
                .collect();
            if parts.is_empty() {
                None
            } else {
                Some(parts.join("\n"))
            }
        }
        _ => None,
    }
}
