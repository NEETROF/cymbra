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

//! Native/WASM parity (spec `lingua-analysis`, change `add-lingua-wasm`).
//!
//! The determinism contract says an equal `analyzer_version` and an equal pack
//! MUST produce identical analysis output on every target. This one test is
//! compiled twice — once for the host (`cargo test`) and once for wasm
//! (`wasm-pack test --node`) — and both assert the engine's JSON equals the
//! same committed golden. If the two targets diverged, one of the runs would
//! fail the shared golden, so passing on both *is* the parity proof.
//!
//! The pack and the golden are embedded with `include_bytes!`/`include_str!`,
//! which resolve at compile time and need no filesystem — so the wasm run
//! under Node validates the very same bytes as the host run.
//!
//! Refresh the golden after an intended analyser change with:
//! `LINGUA_UPDATE_GOLDEN=1 cargo test -p lingua-wasm --test parity`.

use lingua_wasm::LinguaEngine;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

/// The fixture pack, built from `scripts/lingua-data/testdata/en-fr` by
/// `lingua-pack` and committed so the test is hermetic.
const PACK: &[u8] = include_bytes!("fixtures/pack.lingua");

/// The expected canonical JSON. Regenerate with `LINGUA_UPDATE_GOLDEN=1`.
const GOLDEN: &str = include_str!("fixtures/golden.json");

/// Builds the exact scenario both targets analyse: a calibrated reader, one
/// lemma forced to `learning`, over English prose that mixes known words
/// (`run`, `city` — ranked below the threshold), an above-threshold word
/// (`seldom`), an unranked word (`conundrum`), out-of-lexicon words, and two
/// hyphenated compounds — `city-run` (both parts known → one Known token) and
/// `run-seldom` (a part above the threshold → weakest-link Unknown).
fn analyse_fixture() -> String {
    let mut engine = LinguaEngine::new(PACK).expect("fixture pack loads");
    engine.set_calibration(3_000);
    engine.set_status("seldom", "learning");
    engine.analyse(vec![
        "The runner runs through many cities every morning before work.".to_owned(),
        "She ran again today, yet she seldom meets such a strange conundrum.".to_owned(),
        "The city-run service runs well, yet the run-seldom rule holds here.".to_owned(),
    ])
}

/// Asserts the produced JSON equals the committed golden, or rewrites the
/// golden when `LINGUA_UPDATE_GOLDEN` is set (host only — wasm has no
/// filesystem here).
#[cfg(not(target_arch = "wasm32"))]
fn assert_matches_golden(actual: &str) {
    if std::env::var_os("LINGUA_UPDATE_GOLDEN").is_some() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden.json");
        std::fs::write(path, format!("{actual}\n")).expect("write golden");
        return;
    }
    assert_eq!(
        actual,
        GOLDEN.trim_end_matches('\n'),
        "analysis output drifted from the golden — if intended, refresh with \
         LINGUA_UPDATE_GOLDEN=1 cargo test -p lingua-wasm --test parity",
    );
}

#[cfg(target_arch = "wasm32")]
fn assert_matches_golden(actual: &str) {
    assert_eq!(actual, GOLDEN.trim_end_matches('\n'));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn analysis_matches_golden() {
    assert_matches_golden(&analyse_fixture());
}
