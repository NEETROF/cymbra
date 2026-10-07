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

//! English invariance baseline (`docs/lingua/spanish-programme.md`, spike S0).
//!
//! The Spanish programme generalises the core, the wasm engine and the pack build around a
//! second studied language, and English must come out of it unchanged. This freezes what the
//! extension receives from the engine today over a fixed corpus (`baseline/pages.txt`) and the
//! real en-fr pack, built here from the committed tables:
//! - page analyses, for a new reader and for one with statuses, a level, exposures and cards;
//! - glosses, phrase glosses and word grammar;
//! - levels and the vocabulary estimate;
//! - the sync exports, the backup and a review session.
//!
//! A pull request that changes `baseline/en-fr.golden` changes what English readers see. Only
//! a dictionary update, or a deliberate analyser change that bumps the English analyser
//! version, should do that. Re-bless with
//! `LINGUA_BLESS=1 cargo test -p lingua-wasm --test english_baseline` and say why in the pull
//! request; `lingua-pack-update` re-blesses on its own branch.
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use support::english::ENGLISH;
use support::first_difference;

#[test]
fn english_output_has_not_moved() {
    let actual = ENGLISH.render(None);
    let Some(expected) = ENGLISH.bless_or_read(&actual) else {
        return;
    };
    assert!(
        actual == expected,
        "English output moved (docs/lingua/spanish-programme.md: English does not move).\n\
         First difference — {}\n\
         If this pull request means to change English output (a dictionary update, or an \
         analyser change that bumps the English analyser version), re-bless with\n  \
         LINGUA_BLESS=1 cargo test -p lingua-wasm --test english_baseline\n\
         and say why in the pull request. Otherwise the change is wrong.",
        first_difference(&expected, &actual)
    );
    // The extension names the language on every language-bound call
    // (generalise-lingua-extension-port): naming English must answer exactly as naming none.
    let named = ENGLISH.render(Some("en"));
    assert!(
        named == expected,
        "Naming the language `en` moved English output. First difference — {}",
        first_difference(&expected, &named)
    );
}

#[test]
fn the_corpus_reads_as_expected() {
    ENGLISH.check_corpus();
}

/// English's levels come from CEFR-J and Octanove, never from frequency
/// (add-lingua-spanish-levels): the real pack does not call them estimated.
#[test]
fn english_levels_are_not_estimated() {
    let engine = ENGLISH.loaded();
    assert!(engine.has_levels(None).expect("levels"));
    assert!(!engine.levels_estimated(None).expect("estimated"));
}
