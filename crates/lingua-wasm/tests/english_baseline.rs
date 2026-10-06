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

use support::{Card, Scenario, first_difference};

/// Dictionary forms whose gloss the card shows. Some have none in the pack, on purpose.
const LEMMAS: &[&str] = &[
    "go",
    "be",
    "have",
    "do",
    "get",
    "make",
    "take",
    "run",
    "set",
    "put",
    "house",
    "light",
    "bank",
    "bat",
    "fair",
    "lead",
    "wind",
    "tear",
    "read",
    "close",
    "live",
    "present",
    "record",
    "object",
    "minute",
    "bass",
    "dove",
    "refuse",
    "more",
    "eat",
    "son",
    "are",
    "has",
    "sea",
    "lighthouse",
    "landlord",
    "expedition",
    "backlog",
    "betwixt",
    "councillor",
    "flashcard",
    "the",
    "of",
    "whilst",
    "oughta",
    "lol",
];

/// Selections a reader glosses: expressions, phrasal verbs, sentences, clitics, and blocks
/// that are not English.
const PHRASES: &[&str] = &[
    "give up",
    "look forward to",
    "run out of",
    "put up with",
    "break the ice",
    "by and large",
    "in spite of",
    "as a matter of fact",
    "take care of",
    "on the other hand",
    "once in a blue moon",
    "a piece of cake",
    "make up your mind",
    "get over it",
    "turn off the lights",
    "in order to",
    "He has more patience than anyone I know.",
    "We ate dinner early.",
    "The wind was too strong.",
    "don't",
    "won't they",
    "state-of-the-art",
    "Le phare se dressait",
    "El faro se alzaba",
];

/// (word as written, dictionary form) pairs a word card asks the grammar of.
const GRAMMAR: &[(&str, &str)] = &[
    ("went", "go"),
    ("gone", "go"),
    ("goes", "go"),
    ("has", "have"),
    ("had", "have"),
    ("are", "be"),
    ("was", "be"),
    ("better", "good"),
    ("best", "good"),
    ("worse", "bad"),
    ("children", "child"),
    ("mice", "mouse"),
    ("leaves", "leave"),
    ("leaves", "leaf"),
    ("read", "read"),
    ("lead", "lead"),
    ("led", "lead"),
    ("ate", "eat"),
    ("more", "more"),
    ("thought", "think"),
    ("thought", "thought"),
    ("running", "run"),
    ("ran", "run"),
    ("feet", "foot"),
    ("lying", "lie"),
    ("lay", "lie"),
    ("lay", "lay"),
    ("saw", "see"),
    ("saw", "saw"),
    ("don't", "do"),
    ("won't", "will"),
    ("children's", "child"),
];

/// The pages analysed again for the reader with a history.
const READER_PAGES: &[&str] = &["news", "homographs", "mixed", "register"];

const PAGE_NAMES: &[&str] = &[
    "news",
    "fiction",
    "phrasal",
    "technical",
    "academic",
    "recipe",
    "informal",
    "homographs",
    "mixed",
    "register",
    "grammar",
];

const ENGLISH: Scenario = Scenario {
    pair: "en-fr",
    beside: &[],
    test: "english_baseline",
    pages: "pages.txt",
    page_names: PAGE_NAMES,
    lemmas: LEMMAS,
    phrases: PHRASES,
    grammar: GRAMMAR,
    reader_pages: READER_PAGES,
    statuses: &[
        ("lighthouse", "known", 1.0),
        ("landlord", "learning", 2.0),
        ("betwixt", "ignored", 3.0),
        ("ceiling", "known", 4.0),
        ("councillor", "known", 5.0),
        // Withdrawn: « Remettre à apprendre ».
        ("councillor", "", 6.0),
    ],
    exposures: ["sailor", "shore", "storm", "expedition", "patience"],
    cards: [
        Card {
            lemma: "expedition",
            form: "expedition",
            sentence: "She will lead the expedition.",
            url: "https://example.com/homographs",
            gloss: Some("expédition"),
        },
        Card {
            lemma: "backlog",
            form: "backlog",
            sentence: "We really need to take care of the backlog.",
            url: "https://example.com/phrasal",
            gloss: None,
        },
        Card {
            lemma: "grin",
            form: "grinning",
            sentence: "He couldn't stop grinning.",
            url: "",
            gloss: None,
        },
    ],
    reader_phrase: "She will lead the expedition",
};

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
