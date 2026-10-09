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

//! A French paragraph and the tokens French's pre-pass reads it as, with their byte spans
//! (add-lingua-french-tokenisation D6): the host (`languages.rs`) and the wasm target
//! (`languages_wasm.rs`, `wasm-pack test --node`) both analyse it and assert these spans, the
//! first French check on the wasm target with multi-byte spans inside a word. Included by path
//! by both, as `fixture_pack.rs` is.

/// Long enough to be analysed as French, holding `L’`, `qu’`, `au`, an inversion with its
/// euphonic `t` and a narrow no-break space — and no one-letter word: the single-letter rule reads
/// the pack, and the two targets' packs differ.
pub const PARAGRAPH: &str = "L\u{2019}homme pense qu\u{2019}elle viendra demain au marché du village, \
                             mais pense-t-elle vraiment revenir ce soir\u{202F}?";

/// The tokens of [`PARAGRAPH`]: (surface, start, end), in bytes. The elided pieces span their
/// apostrophe (three bytes), `à` and `le` share the span of `au`, the euphonic `t`, the hyphens
/// and the narrow no-break space are in no span.
pub const TOKENS: &[(&str, usize, usize)] = &[
    ("Le", 0, 4),
    ("homme", 4, 9),
    ("pense", 10, 15),
    ("que", 16, 21),
    ("elle", 21, 25),
    ("viendra", 26, 33),
    ("demain", 34, 40),
    ("à", 41, 43),
    ("le", 41, 43),
    ("marché", 44, 51),
    ("du", 52, 54),
    ("village", 55, 62),
    ("mais", 64, 68),
    ("pense", 69, 74),
    ("elle", 77, 81),
    ("vraiment", 82, 90),
    ("revenir", 91, 98),
    ("ce", 99, 101),
    ("soir", 102, 106),
];

/// A page analysis's tokens as (surface, start, end).
pub fn surfaces_and_spans(page: &str) -> Vec<(String, usize, usize)> {
    let page: serde_json::Value = serde_json::from_str(page).unwrap();
    assert_eq!(page["analysable"], true, "{page}");
    page["tokens"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            (
                t["surface"].as_str().unwrap().to_owned(),
                t["start"].as_u64().unwrap() as usize,
                t["end"].as_u64().unwrap() as usize,
            )
        })
        .collect()
}

/// [`TOKENS`], owned, to compare with [`surfaces_and_spans`].
pub fn expected() -> Vec<(String, usize, usize)> {
    TOKENS
        .iter()
        .map(|&(surface, start, end)| (surface.to_owned(), start, end))
        .collect()
}
