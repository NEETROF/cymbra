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

//! The English invariance baseline's scenario (`english_baseline.rs`): its corpus, probes and
//! reader, shared with the cross-native invariance test (`cross_native.rs`).

use super::{Card, PackSource, Scenario};

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

pub const ENGLISH: Scenario = Scenario {
    pair: "en-fr",
    pack: PackSource::Tables,
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
