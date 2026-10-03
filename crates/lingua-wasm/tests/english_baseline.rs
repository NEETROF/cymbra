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

use std::fmt::Write as _;
use std::path::PathBuf;

use lingua_wasm::LinguaEngine;

/// 2026-09-21T13:46:40Z, in the unit each binding takes: deck bindings count epoch seconds,
/// status, level and exposure bindings count milliseconds.
const T_SECS: f64 = 1_790_000_000.0;
const T_MS: f64 = T_SECS * 1000.0;
const DAY_SECS: f64 = 86_400.0;
const DAY_MS: f64 = DAY_SECS * 1000.0;

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

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn tables_dir() -> PathBuf {
    crate_dir().join("../../scripts/lingua-data/tables/en-fr")
}

fn golden_path() -> PathBuf {
    crate_dir().join("tests/baseline/en-fr.golden")
}

/// The pack the extension ships, built from the committed tables (the build is
/// deterministic; `lingua-extension-check` holds the tables to `pin.json`).
fn real_pack() -> Vec<u8> {
    let inputs = lingua_pack::inputs_from_dir(&tables_dir())
        .unwrap_or_else(|e| panic!("read the committed en-fr tables: {e}"));
    lingua_pack::build_pack(&inputs).unwrap_or_else(|e| panic!("build the en-fr pack: {e}"))
}

fn engine(pack: &[u8]) -> LinguaEngine {
    match LinguaEngine::new(pack) {
        Ok(e) => e,
        Err(_) => panic!("the en-fr pack loads"),
    }
}

/// `baseline/pages.txt`: `=== name` opens a page, every other non-empty line is one block.
fn pages() -> Vec<(String, Vec<String>)> {
    let text = std::fs::read_to_string(crate_dir().join("tests/baseline/pages.txt"))
        .unwrap_or_else(|e| panic!("read baseline/pages.txt: {e}"));
    let mut pages: Vec<(String, Vec<String>)> = Vec::new();
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("=== ") {
            pages.push((name.trim().to_owned(), Vec::new()));
        } else if let Some((_, blocks)) = pages.last_mut()
            && !line.trim().is_empty()
        {
            blocks.push(line.to_owned());
        }
    }
    pages
}

/// The golden text: one `### probe` line, then the binding's output exactly as returned.
#[derive(Default)]
struct Golden(String);

impl Golden {
    fn probe(&mut self, name: &str, output: impl std::fmt::Display) {
        let _ = writeln!(self.0, "### {name}\n{output}");
    }
}

fn shown(value: Option<String>) -> String {
    value.unwrap_or_else(|| "(none)".to_owned())
}

fn render(language: Option<&str>) -> String {
    let lang = || language.map(str::to_owned);
    let pack = real_pack();
    let pages = pages();
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(tables_dir().join("manifest.json"))
            .unwrap_or_else(|e| panic!("read the en-fr manifest: {e}")),
    )
    .unwrap_or_else(|e| panic!("parse the en-fr manifest: {e}"));
    let mut g = Golden::default();
    g.probe(
        "about",
        "Generated by crates/lingua-wasm/tests/english_baseline.rs. Re-bless, never edit by hand.",
    );
    g.probe(
        "pack",
        format!(
            "en-fr, {} bytes, pack_version {}, analyzer_version {}",
            pack.len(),
            manifest["meta"]["pack_version"],
            manifest["meta"]["analyzer_version"],
        ),
    );

    // A new reader: no level, no calibration, no history.
    let fresh = engine(&pack);
    for (name, blocks) in &pages {
        g.probe(
            &format!("analyse new-reader {name}"),
            fresh.analyse(blocks.clone(), lang()).unwrap(),
        );
    }
    for lemma in LEMMAS {
        g.probe(
            &format!("gloss {lemma}"),
            shown(fresh.gloss(lemma, lang()).unwrap()),
        );
    }
    for phrase in PHRASES {
        g.probe(
            &format!("phrase-gloss {phrase}"),
            fresh.phrase_gloss(phrase, lang()).unwrap(),
        );
    }
    for (written, lemma) in GRAMMAR {
        g.probe(
            &format!("word-grammar {written} {lemma}"),
            fresh.word_grammar(written, lemma, lang()).unwrap(),
        );
    }
    g.probe("has-levels", fresh.has_levels(lang()).unwrap());
    g.probe("level-ladder", fresh.level_ladder(lang()).unwrap());
    g.probe(
        "vocabulary-estimate new-reader",
        fresh.vocabulary_estimate(lang()).unwrap(),
    );
    g.probe("notice", fresh.notice(lang()).unwrap());
    g.probe("licences", fresh.licences(lang()).unwrap());

    // A reader with a history: calibration, a declared level, every kind of status (one of
    // them withdrawn), exposures on three days, cards added, seeded and retired, a review.
    let mut reader = engine(&pack);
    reader.set_calibration(3_000, lang()).unwrap();
    reader.set_declared_level_at("B1", T_MS, lang()).unwrap();
    for (lemma, status, offset_ms) in [
        ("lighthouse", "known", 1.0),
        ("landlord", "learning", 2.0),
        ("betwixt", "ignored", 3.0),
        ("ceiling", "known", 4.0),
        ("councillor", "known", 5.0),
        // Withdrawn: « Remettre à apprendre ».
        ("councillor", "", 6.0),
    ] {
        reader
            .set_status_at(lemma, status, T_MS + offset_ms, lang())
            .unwrap();
    }
    for day in 0..3 {
        reader
            .record_exposures(
                ["sailor", "shore", "storm", "expedition", "patience"]
                    .map(str::to_owned)
                    .to_vec(),
                "page",
                T_MS + f64::from(day) * DAY_MS,
                lang(),
            )
            .unwrap();
    }
    g.probe(
        "promote-by-exposure 2 days",
        reader
            .promote_by_exposure(2, T_MS + 3.0 * DAY_MS, lang())
            .unwrap(),
    );
    reader
        .add_card(
            "expedition",
            "expedition",
            "She will lead the expedition.",
            "https://example.com/homographs",
            Some("expédition".to_owned()),
            T_SECS,
            lang(),
        )
        .unwrap();
    reader
        .add_card(
            "backlog",
            "backlog",
            "We really need to take care of the backlog.",
            "https://example.com/phrasal",
            None,
            T_SECS + 60.0,
            lang(),
        )
        .unwrap();
    reader
        .add_card(
            "grin",
            "grinning",
            "He couldn't stop grinning.",
            "",
            None,
            T_SECS + 120.0,
            lang(),
        )
        .unwrap();
    g.probe(
        "seed-level A2 5 common",
        reader
            .seed_level("A2", 5, "common", T_SECS + 180.0, lang())
            .unwrap(),
    );
    g.probe(
        "seed-level C1 3 rare",
        reader
            .seed_level("C1", 3, "rare", T_SECS + 240.0, lang())
            .unwrap(),
    );
    reader
        .retire_card("backlog", T_SECS + 300.0, lang())
        .unwrap();
    let now = T_SECS + DAY_SECS;
    g.probe("start-review", reader.start_review(now, None));
    g.probe("review-current first", shown(reader.review_current()));
    reader.review_reveal();
    reader.review_grade("good", now);
    g.probe("review-current second", shown(reader.review_current()));
    reader.review_mark_known(now);
    g.probe("review-remaining", reader.review_remaining());
    g.probe("calibration", reader.calibration(lang()).unwrap());
    g.probe(
        "declared-level",
        shown(reader.declared_level(lang()).unwrap()),
    );
    g.probe("tracked-count", reader.tracked_count());
    g.probe("deck-count", reader.deck_count());
    g.probe(
        "due-count in 7 days",
        reader.due_count(now + 7.0 * DAY_SECS, None),
    );
    for (name, blocks) in pages
        .iter()
        .filter(|(name, _)| READER_PAGES.contains(&name.as_str()))
    {
        g.probe(
            &format!("analyse reader {name}"),
            reader.analyse(blocks.clone(), lang()).unwrap(),
        );
    }
    g.probe(
        "phrase-gloss reader She will lead the expedition",
        reader
            .phrase_gloss("She will lead the expedition", lang())
            .unwrap(),
    );
    g.probe(
        "vocabulary-estimate reader",
        reader.vocabulary_estimate(lang()).unwrap(),
    );
    g.probe("export-status-ops", reader.export_status_ops());
    g.probe("export-card-ops", reader.export_card_ops());
    g.probe("export-declared-levels", reader.export_declared_levels());
    g.probe("backup", reader.backup());
    g.0
}

/// The first probe that differs, with a window around the first character that does.
fn first_difference(expected: &str, actual: &str) -> String {
    let expected: Vec<&str> = expected.split("\n### ").collect();
    let actual: Vec<&str> = actual.split("\n### ").collect();
    for i in 0..expected.len().max(actual.len()) {
        let e = expected.get(i).copied().unwrap_or("(missing)");
        let a = actual.get(i).copied().unwrap_or("(missing)");
        if e == a {
            continue;
        }
        let at = e.chars().zip(a.chars()).take_while(|(x, y)| x == y).count();
        let window =
            |s: &str| -> String { s.chars().skip(at.saturating_sub(60)).take(200).collect() };
        let probe = a.lines().next().unwrap_or("?").trim_start_matches("### ");
        return format!(
            "probe `{probe}`, character {at}:\n  expected: …{}…\n  actual:   …{}…",
            window(e),
            window(a)
        );
    }
    "no probe differs, but the files do (trailing bytes)".to_owned()
}

#[test]
fn english_output_has_not_moved() {
    let actual = render(None);
    let path = golden_path();
    if std::env::var_os("LINGUA_BLESS").is_some() {
        std::fs::write(&path, &actual).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "{} is missing: run LINGUA_BLESS=1 cargo test -p lingua-wasm --test english_baseline",
            path.display()
        )
    });
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
    let named = render(Some("en"));
    assert!(
        named == expected,
        "Naming the language `en` moved English output. First difference — {}",
        first_difference(&expected, &named)
    );
}

#[test]
fn the_corpus_reads_as_expected() {
    let pages = pages();
    let names: Vec<&str> = pages.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [
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
            "grammar"
        ]
    );
    assert!(pages.iter().all(|(_, blocks)| !blocks.is_empty()));
    assert!(
        READER_PAGES.iter().all(|p| names.contains(p)),
        "every reader page exists"
    );
}

/// English's levels come from CEFR-J and Octanove, never from frequency
/// (add-lingua-spanish-levels): the real pack does not call them estimated.
#[test]
fn english_levels_are_not_estimated() {
    let engine = engine(&real_pack());
    assert!(engine.has_levels(None).expect("levels"));
    assert!(!engine.levels_estimated(None).expect("estimated"));
}
