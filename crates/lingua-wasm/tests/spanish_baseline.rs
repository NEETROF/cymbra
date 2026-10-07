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

//! Spanish invariance baseline (`docs/lingua/language-matrix-programme.md`, change 3).
//!
//! The language matrix generalises the core, the packs and the extension around the reader's
//! native language, and what French readers of Spanish see must come out of it unchanged — the
//! Spanish counterpart of `english_baseline.rs`, over a Spanish corpus (`baseline/pages-es.txt`)
//! and the real es-fr pack built from the committed tables. It is the pair a native-language
//! axis could move: the Spanish names rule and the vocabulary estimate read the pack's dictionary
//! words, which a pack glossed in another native language keeps (add-lingua-pack-lexical-layer,
//! `cross_native.rs`).
//!
//! The engine holds en-fr beside es-fr, as the extension does (it starts on the default pair's
//! pack). The estimated Spanish ladder borrows English's typical vocabularies, which the core
//! freezes (`ENGLISH_TYPICAL_VOCABULARY`, generalise-lingua-native-language D6): the figures this
//! golden recorded when they were read from en-fr.
//!
//! A pull request that changes `baseline/es-fr.golden` changes what readers of Spanish see.
//! Only a dictionary update of es-fr, or a deliberate analyser change that bumps the Spanish
//! analyser version, should do that; an en-fr update moves the `beside en-fr` line alone.
//! Re-bless with
//! `LINGUA_BLESS=1 cargo test -p lingua-wasm --test spanish_baseline` and say why in the pull
//! request; `lingua-pack-update` re-blesses on its own branch.
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use support::first_difference;
use support::spanish::SPANISH;

#[test]
fn spanish_output_has_not_moved() {
    let actual = SPANISH.render(Some("es"));
    let Some(expected) = SPANISH.bless_or_read(&actual) else {
        return;
    };
    assert!(
        actual == expected,
        "Spanish output moved (docs/lingua/language-matrix-programme.md: es-fr does not move).\n\
         First difference — {}\n\
         If this pull request means to change Spanish output (a dictionary update of es-fr, or \
         an analyser change that bumps the Spanish analyser version), re-bless with\n  \
         LINGUA_BLESS=1 cargo test -p lingua-wasm --test spanish_baseline\n\
         and say why in the pull request. Otherwise the change is wrong.",
        first_difference(&expected, &actual)
    );
}

#[test]
fn the_corpus_reads_as_expected() {
    SPANISH.check_corpus();
}

/// Spanish levels are derived from frequency (add-lingua-spanish-levels, D1): the real pack
/// calls them estimated.
#[test]
fn spanish_levels_are_estimated() {
    let engine = SPANISH.loaded();
    let es = || Some("es".to_owned());
    assert!(engine.has_levels(es()).expect("levels"));
    assert!(engine.levels_estimated(es()).expect("estimated"));
}
