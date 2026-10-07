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

//! The committed tables of every pair hold to their studied language
//! (add-lingua-pack-lexical-layer D3, D4): each pair carries its studied language's pinned
//! tag pool, and two pairs of one studied language agree on it, on their studied tables and on
//! their dictionary words.

use std::path::PathBuf;

use lingua_pack::tables::check_committed_tables;
use lingua_pack::{LEXICAL_TABLE, TAG_POOL_TABLE};

fn tables() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/tables")
}

#[test]
fn every_committed_pair_holds_to_its_studied_language() {
    let pairs = check_committed_tables(&tables()).unwrap_or_else(|e| panic!("{e}"));
    for reference in ["en-fr", "es-fr"] {
        assert!(pairs.iter().any(|p| p == reference), "{reference} is read");
    }
}

#[test]
fn the_reference_pairs_pin_their_pool_and_name_no_dictionary_words() {
    // en-fr and es-fr are their studied languages' reference packs: their glossed lemmas are
    // the dictionary words, so they carry no lexical table and keep their bytes.
    for reference in ["en-fr", "es-fr"] {
        let dir = tables().join(reference);
        assert!(
            dir.join(TAG_POOL_TABLE).is_file(),
            "{reference}/{TAG_POOL_TABLE}"
        );
        assert!(
            !dir.join(LEXICAL_TABLE).exists(),
            "{reference}/{LEXICAL_TABLE}"
        );
    }
}
