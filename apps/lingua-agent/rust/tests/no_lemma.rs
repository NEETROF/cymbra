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

//! Product-rule lint (task 2.1): the plugin's user-facing output — statusline, `/vocab`,
//! MCP text, the command + manifest copy — must never show the French word "lemme". The
//! English identifier "lemma" is fine (internal); only the whole French word is banned,
//! matched case-insensitively with word boundaries across the whole plugin tree.

use std::fs;
use std::path::Path;

fn walk(dir: &Path, offenders: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            // Skip build output and the test tree itself (this lint file names the word
            // as data; tests are not user-facing output).
            if !matches!(
                name.as_str(),
                "target" | ".git" | "fixtures" | "node_modules" | "tests"
            ) {
                walk(&path, offenders);
            }
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "rs" | "md" | "json" | "sh" | "txt") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            if contains_french_lemme(line) {
                offenders.push(format!("{}:{}", path.display(), i + 1));
            }
        }
    }
}

/// True if the line contains the French word "lemme"/"lemmes" as a whole word
/// (case-insensitive), not English "lemma"/"lemmas" nor a substring like "dilemme".
fn contains_french_lemme(line: &str) -> bool {
    let lower = line.to_lowercase();
    let bytes = lower.as_bytes();
    let mut from = 0;
    while let Some(rel) = lower[from..].find("lemme") {
        let start = from + rel;
        let end = start + "lemme".len();
        let before_ok = start == 0 || !is_word_byte(bytes[start - 1]);
        // Accept an optional trailing "s"; the word must then end.
        let after = if bytes.get(end) == Some(&b's') {
            end + 1
        } else {
            end
        };
        let after_ok = bytes.get(after).is_none_or(|&b| !is_word_byte(b));
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[test]
fn sanity_of_the_matcher() {
    assert!(contains_french_lemme("un lemme"));
    assert!(contains_french_lemme("les lemmes différents"));
    assert!(!contains_french_lemme("the lemma form")); // English identifier, allowed
    assert!(!contains_french_lemme("LemmaStatus")); // identifier, allowed
    assert!(!contains_french_lemme("un dilemme")); // substring, not the word
}

#[test]
fn plugin_tree_never_shows_the_word_lemme() {
    // CARGO_MANIFEST_DIR = apps/lingua-agent/rust; scan the whole plugin app dir.
    let app_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let mut offenders = Vec::new();
    walk(&app_dir, &mut offenders);
    assert!(
        offenders.is_empty(),
        "found the word « lemme » at: {offenders:?}"
    );
}
