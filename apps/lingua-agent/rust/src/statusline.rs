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

//! The coverage statusline: for the last assistant reply, the known-word percentage and
//! the number of new (unknown) words, in the language the reply is read in — named first
//! when the plugin follows several (add-lingua-agent-languages D4). Silent degradation —
//! `None` renders nothing.

use std::collections::BTreeSet;

use lingua_core::analysis::percent::TokenClass;
use lingua_core::knowledge::state::KnowledgeState;

use crate::engine::{Library, read_reply};

/// The statusline text for one reply, or `None` when it cannot be analysed (mute).
/// French UI copy: `📖 96 % · 3 nouveaux`, or `📖 ES 91 % · 5 nouveaux` with several languages.
pub fn statusline_text(
    library: &mut Library,
    knowledge: &KnowledgeState,
    last_reply: &str,
) -> Option<String> {
    let several = library.several();
    let (language, analysis) = read_reply(library, knowledge, last_reply)?;
    if !analysis.analysable {
        return None;
    }
    let pct = analysis.percent?;
    let new_words: BTreeSet<&str> = analysis
        .tokens
        .iter()
        .filter(|t| t.class == TokenClass::Unknown)
        .map(|t| t.lemma.as_str())
        .collect();
    let n = new_words.len();
    let noun = if n == 1 { "nouveau" } else { "nouveaux" };
    let tag = if several {
        format!("{} ", language.tag().to_uppercase())
    } else {
        String::new()
    };
    Some(format!("📖 {tag}{pct} % · {n} {noun}"))
}
