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

//! Non-regression fixtures for the lemmatisation cascade (task 2.3 —
//! minimum 100 cases). Any change to a fixture's expectation is a
//! behavioural change of the analyser and demands an `ANALYZER_VERSION`
//! bump.

use lingua_core::analysis::lemmatize::lemmatize;
use lingua_core::analysis::lexicon::{FstLexicon, build_lexicon_blobs};

/// Forms→lemmas pairs exercising the FST path of the cascade.
const FST_PAIRS: &[(&str, &str)] = &[
    ("analyses", "analysis"),
    ("criteria", "criterion"),
    ("data", "datum"),
    ("halves", "half"),
    ("indices", "index"),
    ("knives", "knife"),
    ("leaves", "leaf"),
    ("lives", "life"),
    ("matrices", "matrix"),
    ("oxen", "ox"),
    ("phenomena", "phenomenon"),
    ("shelves", "shelf"),
    ("wolves", "wolf"),
];

/// Lemmas the rule candidates are validated against.
const LEMMAS: &[&str] = &[
    "a", "bad", "bat", "be", "big", "box", "bus", "busy", "carry", "cheap", "child", "church",
    "city", "class", "code", "do", "dog", "early", "easy", "family", "far", "fix", "fly", "focus",
    "go", "good", "happy", "have", "heavy", "hop", "hope", "i", "jump", "know", "large", "late",
    "little", "make", "many", "miss", "nice", "not", "pass", "plan", "play", "quick", "run", "see",
    "ship", "slow", "small", "stop", "story", "study", "take", "team", "tell", "the", "think",
    "try", "use", "walk", "watch", "will", "wish", "write",
];

/// (form, expected lemma) — grouped by the cascade stage that resolves them.
const CASES: &[(&str, &str)] = &[
    // --- irregulars table ---
    ("am", "be"),
    ("are", "be"),
    ("ate", "eat"),
    ("been", "be"),
    ("began", "begin"),
    ("best", "good"),
    ("better", "good"),
    ("bought", "buy"),
    ("broken", "break"),
    ("brought", "bring"),
    ("built", "build"),
    ("came", "come"),
    ("caught", "catch"),
    ("children", "child"),
    ("chosen", "choose"),
    ("did", "do"),
    ("does", "do"),
    ("done", "do"),
    ("drew", "draw"),
    ("driven", "drive"),
    ("feet", "foot"),
    ("felt", "feel"),
    ("flew", "fly"),
    ("fought", "fight"),
    ("found", "find"),
    ("gave", "give"),
    ("geese", "goose"),
    ("gone", "go"),
    ("got", "get"),
    ("grew", "grow"),
    ("had", "have"),
    ("has", "have"),
    ("heard", "hear"),
    ("held", "hold"),
    ("is", "be"),
    ("kept", "keep"),
    ("knew", "know"),
    ("known", "know"),
    ("least", "little"),
    ("left", "leave"),
    ("less", "little"),
    ("lost", "lose"),
    ("made", "make"),
    ("meant", "mean"),
    ("men", "man"),
    ("met", "meet"),
    ("mice", "mouse"),
    ("more", "many"),
    ("most", "many"),
    ("paid", "pay"),
    ("people", "person"),
    ("ran", "run"),
    ("said", "say"),
    ("sat", "sit"),
    ("saw", "see"),
    ("seen", "see"),
    ("sold", "sell"),
    ("spent", "spend"),
    ("spoke", "speak"),
    ("stood", "stand"),
    ("taken", "take"),
    ("taught", "teach"),
    ("teeth", "tooth"),
    ("thought", "think"),
    ("told", "tell"),
    ("took", "take"),
    ("understood", "understand"),
    ("was", "be"),
    ("went", "go"),
    ("were", "be"),
    ("women", "woman"),
    ("worse", "bad"),
    ("worst", "bad"),
    ("written", "write"),
    ("wrote", "write"),
    // --- FST path ---
    ("analyses", "analysis"),
    ("analysis", "analysis"),
    ("criteria", "criterion"),
    ("halves", "half"),
    ("indices", "index"),
    ("knives", "knife"),
    ("leaves", "leaf"),
    ("lives", "life"),
    ("matrices", "matrix"),
    ("oxen", "ox"),
    ("phenomena", "phenomenon"),
    ("shelves", "shelf"),
    ("wolves", "wolf"),
    // --- morphy-style rules validated against the lexicon ---
    ("batted", "bat"),
    ("bigger", "big"),
    ("biggest", "big"),
    ("boxes", "box"),
    ("buses", "bus"),
    ("busiest", "busy"),
    ("carried", "carry"),
    ("carrying", "carry"),
    ("cheapest", "cheap"),
    ("churches", "church"),
    ("cities", "city"),
    ("classes", "class"),
    ("dogs", "dog"),
    ("earliest", "early"),
    ("easier", "easy"),
    ("families", "family"),
    ("fixes", "fix"),
    ("flies", "fly"),
    ("happier", "happy"),
    ("happiest", "happy"),
    ("heavier", "heavy"),
    ("hoped", "hope"),
    ("hoping", "hope"),
    ("jumping", "jump"),
    ("larger", "large"),
    ("largest", "large"),
    ("later", "late"),
    ("making", "make"),
    ("misses", "miss"),
    ("nicer", "nice"),
    ("passes", "pass"),
    ("planned", "plan"),
    ("played", "play"),
    ("quickly", "quick"),
    ("running", "run"),
    ("shipping", "ship"),
    ("slowly", "slow"),
    ("smaller", "small"),
    ("stopped", "stop"),
    ("stories", "story"),
    ("studied", "study"),
    ("studying", "study"),
    ("taking", "take"),
    ("teams", "team"),
    ("tried", "try"),
    ("used", "use"),
    ("walked", "walk"),
    ("watches", "watch"),
    ("wishes", "wish"),
    // --- out-of-lexicon regular-plural fallback ---
    ("berries", "berry"),
    ("blockchains", "blockchain"),
    ("conundrums", "conundrum"),
    ("endeavors", "endeavor"),
    ("memes", "meme"),
    ("pitfalls", "pitfall"),
    ("tokenizers", "tokenizer"),
    ("workflows", "workflow"),
    // --- identity (last resort) and guards ---
    ("albeit", "albeit"),
    ("conundrum", "conundrum"),
    ("focus", "focus"),
    ("hindsight", "hindsight"),
    ("moot", "moot"),
    ("seldom", "seldom"),
    // --- case folding and typographic apostrophes ---
    ("Children", "child"),
    ("Running", "run"),
    ("WENT", "go"),
    ("'re", "be"),
    ("'ve", "have"),
    ("\u{2019}ll", "will"),
];

#[test]
fn lemmatization_non_regression_fixtures() {
    assert!(
        CASES.len() >= 100,
        "the contract is at least 100 cases, got {}",
        CASES.len()
    );
    let (bytes, pool) = build_lexicon_blobs(FST_PAIRS, LEMMAS).expect("build fixture lexicon");
    let lexicon = FstLexicon::from_slices(bytes, &pool).expect("load fixture lexicon");

    let mut failures = Vec::new();
    for (form, expected) in CASES {
        let got = lemmatize(form, &lexicon);
        if got != *expected {
            failures.push(format!("{form:?} → {got:?} (expected {expected:?})"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} regressions:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
