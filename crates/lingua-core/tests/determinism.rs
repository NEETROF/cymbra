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

//! Determinism over a fixture corpus (task 2.6, design D5): at an equal
//! `ANALYZER_VERSION` the full pipeline must produce byte-for-byte identical
//! serialised output across runs and across lexicon reloads. The native/WASM
//! cross-target parity test builds on this in `add-lingua-wasm`.

use lingua_core::analysis::ANALYZER_VERSION;
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::lexicon::{FstLexicon, build_lexicon_blobs};
use lingua_core::analysis::pipeline::{DocumentAnalysis, analyse_document};

const CORPUS: &[&str] = &[
    "The quick brown fox jumps over the lazy dog every single morning.",
    "Teams don't ship code on Friday evenings, and they seldom regret it.",
    "Les équipes ne livrent jamais le vendredi soir, c'est une règle ancienne.",
    "In hindsight, the endeavors to automate reviews were never about replacing engineers.",
    "She thought the biggest conundrums were the pitfalls nobody wrote down.",
    "OK",
];

fn fixture_lexicon() -> (Vec<u8>, String) {
    build_lexicon_blobs(
        &[
            ("teams", "team"),
            ("jumps", "jump"),
            ("evenings", "evening"),
            ("reviews", "review"),
        ],
        &[
            "a", "and", "about", "automate", "brown", "code", "dog", "down", "engineer", "every",
            "fox", "friday", "i", "in", "lazy", "morning", "never", "nobody", "not", "on", "over",
            "quick", "regret", "replace", "she", "single", "ship", "the", "they", "to", "were",
            "write",
        ],
    )
    .expect("build fixture lexicon")
}

fn run_once() -> String {
    let (bytes, pool) = fixture_lexicon();
    let lexicon = FstLexicon::from_slices(bytes, &pool).expect("load lexicon");
    let analysis = analyse_document(CORPUS, StudiedLanguage::English, &lexicon);
    serde_json::to_string(&analysis).expect("serialise analysis")
}

#[test]
fn analyzer_version_is_exposed_and_semver_shaped() {
    let parts: Vec<&str> = ANALYZER_VERSION.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "expected MAJOR.MINOR.PATCH, got {ANALYZER_VERSION:?}"
    );
    for part in parts {
        part.parse::<u32>().expect("numeric semver component");
    }
}

#[test]
fn double_run_is_byte_for_byte_identical() {
    let first = run_once();
    let second = run_once();
    assert_eq!(
        first, second,
        "same corpus, same version, same output — the contract"
    );
}

#[test]
fn corpus_analysis_shape_is_stable() {
    let (bytes, pool) = fixture_lexicon();
    let lexicon = FstLexicon::from_slices(bytes, &pool).expect("load lexicon");
    match analyse_document(CORPUS, StudiedLanguage::English, &lexicon) {
        DocumentAnalysis::Analysed(tokens) => {
            // The French block (index 2) and the tiny block (index 5) are
            // excluded; English blocks are analysed in document order.
            assert!(tokens.iter().all(|t| t.block != 2 && t.block != 5));
            let blocks: Vec<usize> = tokens.iter().map(|t| t.block).collect();
            let mut sorted = blocks.clone();
            sorted.sort_unstable();
            assert_eq!(blocks, sorted, "tokens must stay in document order");
            // Spot-checks pinning the pipeline's behaviour on the corpus.
            assert!(tokens.iter().any(|t| t.surface == "do" && t.lemma == "do"));
            assert!(
                tokens
                    .iter()
                    .any(|t| t.surface == "endeavors" && t.lemma == "endeavor")
            );
            assert!(
                tokens
                    .iter()
                    .any(|t| t.surface == "thought" && t.lemma == "think")
            );
        }
        DocumentAnalysis::NotAnalysable => panic!("the fixture corpus must be analysable"),
    }
}
