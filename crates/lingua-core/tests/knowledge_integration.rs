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

//! End-to-end wiring of the knowledge model onto the analysis pipeline
//! (change 2 onto change 1): a real analysed document, classified through the
//! knowledge state, folded into a known-token percentage.

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::lexicon::{FstLexicon, build_lexicon_blobs};
use lingua_core::analysis::percent::{TokenClass, coverage};
use lingua_core::analysis::pipeline::{DocumentAnalysis, analyse_document};
use lingua_core::knowledge::state::{KnowledgeState, MapFrequencyRanks};
use lingua_core::knowledge::status::{KnownSource, Status};

const EN: StudiedLanguage = StudiedLanguage::English;

fn lexicon() -> FstLexicon<Vec<u8>> {
    let (bytes, pool) = build_lexicon_blobs(
        &[
            ("teams", "team"),
            ("ship", "ship"),
            ("evenings", "evening"),
            ("regrets", "regret"),
        ],
        &[
            "a", "and", "code", "do", "evening", "friday", "i", "it", "not", "on", "regret", "run",
            "seldom", "ship", "team", "the", "they",
        ],
    )
    .expect("build lexicon");
    FstLexicon::from_slices(bytes, &pool).expect("load lexicon")
}

/// Ranks so that calibration can make the common words known while the two
/// hard words (`seldom`, `regret`) stay out of reach.
fn ranks() -> MapFrequencyRanks {
    MapFrequencyRanks::from_pairs([
        ("the", 1),
        ("they", 60),
        ("and", 3),
        ("do", 20),
        ("not", 30),
        ("on", 25),
        ("it", 10),
        ("a", 2),
        ("friday", 1_800),
        ("evening", 1_500),
        ("team", 900),
        ("ship", 1_200),
        ("code", 2_500),
        ("seldom", 5_100),
        ("regret", 4_200),
    ])
}

/// Classifies a document's tokens through the knowledge state and returns the
/// known percentage — the change-1 pipeline feeding the change-2 model
/// feeding the change-1 percentage.
fn known_percent(doc: &[&str], state: &KnowledgeState) -> Option<u8> {
    let lexicon = lexicon();
    let ranks = ranks();
    let DocumentAnalysis::Analysed(tokens) = analyse_document(doc, EN, &lexicon) else {
        return None;
    };
    let classes = tokens.iter().map(|t| {
        // A single candidate today (the cascade returns one lemma); the
        // classify API already accepts several.
        state.classify(EN, &[t.lemma.as_str()], &ranks)
    });
    coverage(classes).percent_rounded()
}

#[test]
fn calibration_then_marking_moves_the_percentage() {
    let doc = ["Teams do not ship code on Friday evenings, and they seldom regret it."];

    // No primer: only what the (tiny) fixture ranks nothing for is unknown —
    // with calibration 0, nothing is implicitly known, so 0%.
    let empty = KnowledgeState::new();
    assert_eq!(known_percent(&doc, &empty), Some(0));

    // Calibrate to 3,000: the common words become known, the two rare ones
    // (seldom 5,100 / regret 4,200) remain unknown.
    let mut calibrated = KnowledgeState::new();
    calibrated.set_calibration(EN, 3_000);
    let p = known_percent(&doc, &calibrated).expect("analysable");
    assert!(
        (80..=95).contains(&p),
        "expected a high-but-not-full percentage, got {p}"
    );

    // Learn one of the two: the percentage rises.
    let mut learned = calibrated.clone();
    learned.set_status(EN, "seldom", Status::Known(KnownSource::Manual));
    let p2 = known_percent(&doc, &learned).expect("analysable");
    assert!(p2 > p, "learning a word must raise coverage ({p} → {p2})");
}

#[test]
fn learning_status_counts_as_not_known() {
    let doc = ["Teams do not ship code on Friday evenings, and they seldom regret it."];
    let mut state = KnowledgeState::new();
    state.set_calibration(EN, 6_000); // everything ranked is known…
    state.set_status(EN, "regret", Status::Learning); // …except this, in a deck
    let tokens = match analyse_document(&doc, EN, &lexicon()) {
        DocumentAnalysis::Analysed(t) => t,
        DocumentAnalysis::NotAnalysable => panic!("analysable"),
    };
    let regret = tokens
        .iter()
        .find(|t| t.lemma == "regret")
        .expect("regret present");
    assert_eq!(
        state.classify(EN, &[regret.lemma.as_str()], &ranks()),
        TokenClass::Learning
    );
}
