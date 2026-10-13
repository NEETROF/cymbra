// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The translation providers of « Traduction en ligne » (change: add-lingua-online-translation,
//! D2): their configuration, the request each one is sent and how its reply is read — all pure
//! and tested here; the HTTPS calls themselves are the thin adapters in [`http`].

pub mod http;

use serde_json::{Value, json};

/// Read from the environment; `get` is `std::env::var(..).ok()` in production.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConfig {
    pub name: &'static str,
    pub api_key: String,
    pub model: String,
}

pub const GEMINI: &str = "gemini";
pub const ANTHROPIC: &str = "anthropic";
pub const DEFAULT_GEMINI_MODEL: &str = "gemini-3.1-flash-lite";
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-haiku-5-5";

/// The configured providers, primary first: `CYMBRA_LINGUA_TRANSLATION_PRIMARY` (`gemini` by
/// default) picks the order; a provider without its API key is left out. Empty when neither
/// is configured — the service then answers unavailable.
pub fn from_env(get: impl Fn(&str) -> Option<String>) -> Vec<ProviderConfig> {
    let key = |k: &str| {
        get(k)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    let gemini = key("CYMBRA_LINGUA_TRANSLATION_GEMINI_API_KEY").map(|api_key| ProviderConfig {
        name: GEMINI,
        api_key,
        model: key("CYMBRA_LINGUA_TRANSLATION_GEMINI_MODEL")
            .unwrap_or_else(|| DEFAULT_GEMINI_MODEL.into()),
    });
    let anthropic =
        key("CYMBRA_LINGUA_TRANSLATION_ANTHROPIC_API_KEY").map(|api_key| ProviderConfig {
            name: ANTHROPIC,
            api_key,
            model: key("CYMBRA_LINGUA_TRANSLATION_ANTHROPIC_MODEL")
                .unwrap_or_else(|| DEFAULT_ANTHROPIC_MODEL.into()),
        });
    let anthropic_first = key("CYMBRA_LINGUA_TRANSLATION_PRIMARY").as_deref() == Some(ANTHROPIC);
    let ordered = if anthropic_first {
        [anthropic, gemini]
    } else {
        [gemini, anthropic]
    };
    ordered.into_iter().flatten().collect()
}

/// The structured answer every provider is held to.
fn answer_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "translation": { "type": "string" },
            "highlighted": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["translation", "highlighted"],
        "additionalProperties": false
    })
}

/// The Gemini `generateContent` body (its schema dialect has no `additionalProperties`).
pub fn gemini_body(system: &str, user: &str) -> Value {
    json!({
        "systemInstruction": { "parts": [{ "text": system }] },
        "contents": [{ "role": "user", "parts": [{ "text": user }] }],
        "generationConfig": {
            "responseMimeType": "application/json",
            "responseSchema": {
                "type": "OBJECT",
                "properties": {
                    "translation": { "type": "STRING" },
                    "highlighted": { "type": "ARRAY", "items": { "type": "STRING" } }
                },
                "required": ["translation", "highlighted"]
            }
        }
    })
}

/// The Anthropic Messages body: structured output and low effort. Thinking is left to the
/// model — turning it off made it write its own corrections into the answer (D4).
pub fn anthropic_body(model: &str, system: &str, user: &str) -> Value {
    json!({
        "model": model,
        "max_tokens": 1024,
        "system": system,
        "messages": [{ "role": "user", "content": user }],
        "output_config": {
            "effort": "low",
            "format": { "type": "json_schema", "schema": answer_schema() }
        }
    })
}

/// `(text, input tokens, output tokens)` of a Gemini reply; thinking tokens are billed as output.
pub fn read_gemini(reply: &Value) -> Option<(String, i64, i64)> {
    let text = reply
        .pointer("/candidates/0/content/parts")?
        .as_array()?
        .iter()
        .filter_map(|p| p.get("text")?.as_str())
        .collect::<String>();
    let usage = reply.get("usageMetadata");
    let n = |k: &str| {
        usage
            .and_then(|u| u.get(k))
            .and_then(Value::as_i64)
            .unwrap_or(0)
    };
    Some((
        text,
        n("promptTokenCount"),
        n("candidatesTokenCount") + n("thoughtsTokenCount"),
    ))
}

/// `(text, input tokens, output tokens)` of an Anthropic reply: its text blocks, joined.
pub fn read_anthropic(reply: &Value) -> Option<(String, i64, i64)> {
    if reply.get("stop_reason").and_then(Value::as_str) == Some("refusal") {
        return None;
    }
    let text = reply
        .get("content")?
        .as_array()?
        .iter()
        .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|b| b.get("text")?.as_str())
        .collect::<String>();
    let n = |k: &str| {
        reply
            .pointer(&format!("/usage/{k}"))
            .and_then(Value::as_i64)
            .unwrap_or(0)
    };
    Some((text, n("input_tokens"), n("output_tokens")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let m: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| m.get(k).cloned()
    }

    #[test]
    fn gemini_is_primary_by_default_and_the_order_is_configuration() {
        let both = [
            ("CYMBRA_LINGUA_TRANSLATION_GEMINI_API_KEY", "g"),
            ("CYMBRA_LINGUA_TRANSLATION_ANTHROPIC_API_KEY", "a"),
        ];
        let names = |v: Vec<ProviderConfig>| v.into_iter().map(|p| p.name).collect::<Vec<_>>();
        assert_eq!(names(from_env(env(&both))), vec![GEMINI, ANTHROPIC]);
        let mut swapped = both.to_vec();
        swapped.push(("CYMBRA_LINGUA_TRANSLATION_PRIMARY", "anthropic"));
        assert_eq!(names(from_env(env(&swapped))), vec![ANTHROPIC, GEMINI]);
        let only = from_env(env(&[(
            "CYMBRA_LINGUA_TRANSLATION_ANTHROPIC_API_KEY",
            " a ",
        )]));
        assert_eq!(only.len(), 1);
        assert_eq!(only[0].api_key, "a");
        assert_eq!(only[0].model, DEFAULT_ANTHROPIC_MODEL);
        assert!(from_env(env(&[("CYMBRA_LINGUA_TRANSLATION_GEMINI_API_KEY", "")])).is_empty());
        let model = from_env(env(&[
            ("CYMBRA_LINGUA_TRANSLATION_GEMINI_API_KEY", "g"),
            ("CYMBRA_LINGUA_TRANSLATION_GEMINI_MODEL", "gemini-9"),
        ]));
        assert_eq!(model[0].model, "gemini-9");
    }

    #[test]
    fn each_body_asks_for_the_structured_answer() {
        let g = gemini_body("sys", "user");
        assert_eq!(
            g.pointer("/systemInstruction/parts/0/text"),
            Some(&json!("sys"))
        );
        assert_eq!(
            g.pointer("/generationConfig/responseMimeType"),
            Some(&json!("application/json"))
        );
        let a = anthropic_body("claude-haiku-5-5", "sys", "user");
        assert_eq!(a["model"], json!("claude-haiku-5-5"));
        assert_eq!(a.pointer("/output_config/effort"), Some(&json!("low")));
        assert_eq!(
            a.pointer("/output_config/format/schema/required/1"),
            Some(&json!("highlighted"))
        );
        assert!(a.get("thinking").is_none());
    }

    #[test]
    fn replies_are_read_with_their_token_counts() {
        let g = json!({
            "candidates": [{ "content": { "parts": [{ "text": "{\"a\":" }, { "text": "1}" }] } }],
            "usageMetadata": { "promptTokenCount": 270, "candidatesTokenCount": 40, "thoughtsTokenCount": 5 }
        });
        assert_eq!(read_gemini(&g), Some(("{\"a\":1}".into(), 270, 45)));
        assert_eq!(read_gemini(&json!({})), None);
        let a = json!({
            "content": [{ "type": "thinking", "thinking": "" }, { "type": "text", "text": "{}" }],
            "usage": { "input_tokens": 634, "output_tokens": 60 }
        });
        assert_eq!(read_anthropic(&a), Some(("{}".into(), 634, 60)));
        assert_eq!(
            read_anthropic(&json!({ "stop_reason": "refusal", "content": [] })),
            None
        );
    }
}
