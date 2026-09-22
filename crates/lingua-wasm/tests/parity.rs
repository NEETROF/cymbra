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
//! MUST produce identical analysis output on every target. These tests are
//! compiled twice — once for the host (`cargo test`) and once for wasm
//! (`wasm-pack test --node`) — and both assert the engine's JSON equals the
//! same committed golden. If the two targets diverged, one of the runs would
//! fail the shared golden, so passing on both *is* the parity proof.
//!
//! The pack and the goldens are embedded with `include_bytes!`/`include_str!`,
//! which resolve at compile time and need no filesystem — so the wasm run
//! under Node validates the very same bytes as the host run.
//!
//! Two goldens, each under its own update variable, so refreshing one can never
//! rewrite the other in the same run (`add-lingua-phrase-gloss`, design D7):
//! `golden.json` matching with no diff is the proof that the page analysis did
//! not move when the phrase gloss was added.
//!
//! Refresh a golden after an intended analyser change with:
//! `LINGUA_UPDATE_GOLDEN=1 cargo test -p lingua-wasm --test parity` for the
//! page analysis, `LINGUA_UPDATE_PHRASE_GOLDEN=1 …` for the phrase gloss.

use lingua_wasm::LinguaEngine;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

/// The fixture pack, built from `scripts/lingua-data/testdata/en-fr` by
/// `lingua-pack` and committed so the test is hermetic.
const PACK: &[u8] = include_bytes!("fixtures/pack.lingua");

/// The expected canonical JSON of the page analysis. Regenerate with
/// `LINGUA_UPDATE_GOLDEN=1`.
const GOLDEN: &str = include_str!("fixtures/golden.json");

/// The expected phrase glosses of [`PHRASES`], one JSON line per selection.
/// Regenerate with `LINGUA_UPDATE_PHRASE_GOLDEN=1`.
const PHRASE_GOLDEN: &str = include_str!("fixtures/phrase_golden.json");

/// Selections made of the fixture pack's own words: a known word (`city`, ranked
/// below the calibration), an inflected form (`cities`), a known word's
/// irregular form (`ran` → `run`), an unlisted hyphenated compound whose
/// parts the pack glosses (`run-seldom`), a sentence-cased name the lexicon
/// holds no form of, and a multi-word selection mixing pack words with words
/// the pack has never heard of.
///
/// One selection MUST reach the fixture pack's expression table once that
/// table is there — see [`the_phrase_golden_exercises_the_expression_table`],
/// which fails until one does. The testdata's keys are `city conundrum`,
/// `city run` and `seldom run`, so a selection such as `the city runs daily`
/// reaches one on its dictionary forms, which is the point.
const PHRASES: &[&str] = &[
    "city",
    "cities",
    "ran",
    "run-seldom",
    "Jenkins",
    "she seldom meets such a strange conundrum",
    // Reaches the expression table on DICTIONARY FORMS, not on the words as written:
    // `runs` lemmatises to `run`, so the key `city run` matches where a surface
    // comparison would not.
    "the city runs daily",
];

/// A calibrated reader with one lemma forced to `learning`: the state both
/// goldens are produced under.
fn fixture_engine() -> LinguaEngine {
    let mut engine = LinguaEngine::new(PACK).expect("fixture pack loads");
    engine.set_calibration(3_000);
    engine.set_status("seldom", "learning");
    engine
}

/// Builds the exact scenario both targets analyse: a calibrated reader, one
/// lemma forced to `learning`, over English prose that mixes known words
/// (`run`, `city` — ranked below the threshold), an above-threshold word
/// (`seldom`), an unranked word (`conundrum`), out-of-lexicon words, and two
/// hyphenated compounds — `city-run` (both parts known → one Known token) and
/// `run-seldom` (a part above the threshold → weakest-link Unknown).
fn analyse_fixture() -> String {
    fixture_engine().analyse(vec![
        "The runner runs through many cities every morning before work.".to_owned(),
        "She ran again today, yet she seldom meets such a strange conundrum.".to_owned(),
        "The city-run service runs well, yet the run-seldom rule holds here.".to_owned(),
    ])
}

/// Glosses every selection of [`PHRASES`] under the same reader as the page
/// analysis. Each line is a JSON object holding the selection and the engine's
/// JSON verbatim, so the golden stays readable and the comparison byte-exact.
fn gloss_fixture_phrases() -> String {
    let engine = fixture_engine();
    let lines: Vec<String> = PHRASES
        .iter()
        .map(|text| {
            let key = serde_json::to_string(text).expect("a JSON string");
            format!("{{\"text\":{key},\"gloss\":{}}}", engine.phrase_gloss(text))
        })
        .collect();
    format!("[\n{}\n]", lines.join(",\n"))
}

/// Asserts the produced JSON equals the committed golden `file` (embedded as
/// `expected`), or rewrites that file when `update_var` is set (host only —
/// wasm has no filesystem here).
#[cfg(not(target_arch = "wasm32"))]
fn assert_matches_golden(actual: &str, file: &str, expected: &str, update_var: &str) {
    if std::env::var_os(update_var).is_some() {
        let path = format!("{}/tests/fixtures/{file}", env!("CARGO_MANIFEST_DIR"));
        std::fs::write(path, format!("{actual}\n")).expect("write golden");
        return;
    }
    assert_eq!(
        actual,
        expected.trim_end_matches('\n'),
        "output drifted from {file} — if intended, refresh with \
         {update_var}=1 cargo test -p lingua-wasm --test parity",
    );
}

#[cfg(target_arch = "wasm32")]
fn assert_matches_golden(actual: &str, _file: &str, expected: &str, _update_var: &str) {
    assert_eq!(actual, expected.trim_end_matches('\n'));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn analysis_matches_golden() {
    assert_matches_golden(
        &analyse_fixture(),
        "golden.json",
        GOLDEN,
        "LINGUA_UPDATE_GOLDEN",
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn phrase_gloss_matches_golden() {
    assert_matches_golden(
        &gloss_fixture_phrases(),
        "phrase_golden.json",
        PHRASE_GOLDEN,
        "LINGUA_UPDATE_PHRASE_GOLDEN",
    );
}

/// A table no selection reaches is a table the golden would stop covering
/// without anyone noticing, so the phrase golden must exercise the fixture
/// pack's expressions (`add-lingua-expression-table`).
///
/// The assertion waits for the fixture: the committed `pack.lingua` predates
/// the expression table, and rebuilding it from the testdata belongs to the
/// data pipeline, not to this test. The day it carries one, this fails until
/// [`PHRASES`] holds a selection that reaches it.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn the_phrase_golden_exercises_the_expression_table() {
    let pack = lingua_core::packs::Pack::load(PACK).expect("fixture pack loads");
    if !pack.has_expressions() {
        return;
    }
    assert!(
        gloss_fixture_phrases().contains(r#""expressions":["#),
        "the fixture pack carries an expression table no selection of PHRASES \
         reaches — add one made of its words, then refresh the phrase golden",
    );
}
