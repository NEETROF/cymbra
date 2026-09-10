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

//! Runs the pack pipeline over the committed tiny testdata (no download):
//! proves it builds from real on-disk tables, is reproducible, stays under
//! budget, and — with the analyser version aligned — round-trips through the
//! `lingua-core` reader. The full EN→FR pack uses the same code path over the
//! real AGID/wordfreq/kaikki tables in CI/dev.

use std::path::PathBuf;

use lingua_core::analysis::ANALYZER_VERSION;
use lingua_core::analysis::lexicon::Lexicon;
use lingua_core::knowledge::state::FrequencyRanks;
use lingua_core::packs::Pack;
use lingua_pack::{MAX_PACK_BYTES, build_pack, inputs_from_dir};

fn testdata_dir() -> PathBuf {
    // crates/lingua-pack/ -> ../../scripts/lingua-data/testdata/en-fr
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/testdata/en-fr")
}

#[test]
fn pipeline_build_is_reproducible_and_under_budget() {
    let inputs = inputs_from_dir(&testdata_dir()).expect("read testdata");
    let a = build_pack(&inputs).expect("build a");
    let b = build_pack(&inputs).expect("build b");
    assert_eq!(
        a, b,
        "two builds over the same sources must be byte-identical"
    );
    assert!(a.len() < MAX_PACK_BYTES);
}

#[test]
fn pipeline_output_round_trips_through_the_reader() {
    let mut inputs = inputs_from_dir(&testdata_dir()).expect("read testdata");
    // Align the fixture with whatever analyser this build declares, so the
    // reader accepts it regardless of version bumps.
    inputs.meta.analyzer_version = ANALYZER_VERSION.to_owned();
    let bytes = build_pack(&inputs).expect("build");

    let pack = Pack::load(&bytes).expect("load");
    assert_eq!(pack.meta().pair_key(), "en->fr");
    assert_eq!(pack.lexicon().lemma_of("running"), Some("run"));
    assert_eq!(pack.rank("run"), Some(500));
    assert_eq!(pack.gloss("conundrum"), Some("casse-tête"));
    assert!(pack.notice().contains("CC BY-SA"));
}
