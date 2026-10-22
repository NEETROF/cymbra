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

//! Packs built from the committed tables, shared by the integration tests (`support`) and the
//! crate's own unit tests (`src/lib.rs` includes this file by path). Host only: the builder uses
//! C zstd.

use std::path::PathBuf;

/// The committed tables, `scripts/lingua-data/tables`.
pub fn tables_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/tables")
}

/// The inputs of `pair`'s pack, read from the committed tables.
pub fn inputs(pair: &str) -> lingua_pack::PackInputs {
    lingua_pack::inputs_from_tables(&tables_root(), pair)
        .unwrap_or_else(|e| panic!("read the committed {pair} tables: {e}"))
}

/// The pack the extension ships for `pair`, built from the committed tables (the build is
/// deterministic).
pub fn pack(pair: &str) -> Vec<u8> {
    build(&inputs(pair))
}

/// The base and the layer of `pair`'s pack, built from the committed tables in one run
/// (split-lingua-pack-format D8): assembled, the pack [`pack`] builds.
pub fn split(pair: &str) -> lingua_pack::SplitPack {
    lingua_pack::build_split(&inputs(pair)).unwrap_or_else(|e| panic!("split {pair}: {e}"))
}

fn build(inputs: &lingua_pack::PackInputs) -> Vec<u8> {
    lingua_pack::build_pack(inputs).unwrap_or_else(|e| panic!("build the pack: {e}"))
}

/// en-fr built from the committed tables with `four-poster` among its dictionary words, so that
/// its expression table holds `four-poster bed → Lit à baldaquin` (fix-lingua-selection-card-
/// punctuation): the builder keys an English expression only when each of its words is a lemma
/// of the lexicon, and the committed tables do not hold `four-poster`.
pub fn en_fr_holding_four_poster_bed() -> Vec<u8> {
    let mut inputs = inputs("en-fr");
    let word = "four-poster".to_owned();
    inputs
        .glosses
        .push((word.clone(), "Lit à colonnes".to_owned()));
    inputs.form_lemma.push((word.clone(), word.clone()));
    if let Some(lexical) = inputs.lexical.as_mut() {
        lexical.push(word);
    }
    build(&inputs)
}
