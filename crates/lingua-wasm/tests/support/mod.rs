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

//! Invariance baselines: what the engine gives the extension over a fixed corpus, with a real
//! pack built from the committed tables, frozen in `tests/baseline/<pair>.golden`.
//!
//! One scenario per shipped pair (`english.rs`, `spanish.rs` here, frozen by `english_baseline.rs`
//! and `spanish_baseline.rs`; es-en's, the Spanish scenario glossed in English, and en-es's, the
//! English scenario glossed in Spanish, are declared by `es_en_baseline.rs` and `en_es_baseline.rs`
//! themselves), one harness: the probes and the reader's history are the same for every pair, only
//! the words differ. `cross_native.rs` answers the English and Spanish scenarios through the real
//! pack of the same studied tables glossed in another native language (en-es, es-en); `probes`
//! and `studied_side` here are how it, `es_en_baseline.rs` and `en_es_baseline.rs` read a golden
//! probe by probe and remove the native side.
//! A golden is re-blessed with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test <test>`, and the
//! pull request says why (docs/lingua/language-matrix-programme.md: en-fr and es-fr do not move).

#![allow(dead_code)]

pub mod english;
pub mod french;
pub mod spanish;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use lingua_wasm::LinguaEngine;

/// 2026-09-21T13:46:40Z, in the unit each binding takes: deck bindings count epoch seconds,
/// status, level and exposure bindings count milliseconds.
const T_SECS: f64 = 1_790_000_000.0;
const T_MS: f64 = T_SECS * 1000.0;
const DAY_SECS: f64 = 86_400.0;
const DAY_MS: f64 = DAY_SECS * 1000.0;

/// A card the reader adds: dictionary form, form as written, sentence, page, gloss shown.
pub struct Card {
    pub lemma: &'static str,
    pub form: &'static str,
    pub sentence: &'static str,
    pub url: &'static str,
    pub gloss: Option<&'static str>,
}

/// Where a scenario's own pack is built from (add-lingua-french-baseline D6). The packs loaded
/// beside it are always built from the committed tables. `french.rs` (`french_baseline.rs`)
/// freezes a pair that does not ship yet, over a fixture, until its tables are committed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackSource {
    /// The committed tables: `tables/<studied>/` and `tables/<pair>/`, as the extension ships.
    Tables,
    /// A hand-written fixture, `scripts/lingua-data/testdata/<pair>/`, until the pair's tables
    /// are committed.
    Testdata,
}

impl PackSource {
    /// The folder holding the pair's `manifest.json`.
    pub fn manifest_dir(self, pair: &str) -> PathBuf {
        match self {
            PackSource::Tables => Scenario::tables_dir(pair),
            PackSource::Testdata => Scenario::crate_dir()
                .join("../../scripts/lingua-data/testdata")
                .join(pair),
        }
    }

    /// The pair's pack, built from this source (the build is deterministic), once the core
    /// loads it ([`loadable`]).
    pub fn pack(self, pair: &str) -> Vec<u8> {
        let dir = self.manifest_dir(pair);
        match self {
            PackSource::Tables => loadable(Scenario::real_pack(pair), pair, &dir),
            PackSource::Testdata => testdata_pack(&dir, pair),
        }
    }
}

/// The pack built from the fixture tables in `dir` (`scripts/lingua-data/testdata/<pair>/`, or a
/// copy of it), once the core loads it ([`loadable`]).
pub fn testdata_pack(dir: &Path, pair: &str) -> Vec<u8> {
    let inputs = lingua_pack::inputs_from_dir(dir)
        .unwrap_or_else(|e| panic!("read the {pair} testdata tables: {e}"));
    let pack = lingua_pack::build_pack(&inputs)
        .unwrap_or_else(|e| panic!("build the {pair} testdata pack: {e}"));
    loadable(pack, pair, dir)
}

/// `pack`, once the core loads it; else a panic naming the core's reason and the manifest to
/// bump. The builder stamps the manifest's `analyzer_version` without checking it (the core
/// compares it at load), and on the host the engine cannot say why it refuses a pack: its
/// `JsError` is wasm-bindgen's, and building one outside wasm panics with « cannot call
/// wasm-bindgen imported functions on non-wasm targets ». So a fixture whose manifest falls
/// behind its language's analyser version (add-lingua-french-baseline D5) is named here.
fn loadable(pack: Vec<u8>, pair: &str, manifest_dir: &Path) -> Vec<u8> {
    if let Err(e) = lingua_core::packs::Pack::load(&pack) {
        let manifest = manifest_dir.join("manifest.json");
        let repo = Scenario::crate_dir().join("../..").canonicalize().ok();
        let shown = manifest
            .canonicalize()
            .ok()
            .zip(repo)
            .and_then(|(m, r)| m.strip_prefix(r).ok().map(Path::to_path_buf))
            .unwrap_or(manifest);
        panic!(
            "the {pair} pack does not load: {e}.\n\
             If its language's analyser version moved, bump `analyzer_version` in {} to the \
             core's, in the pull request that moves it, and re-bless the golden.",
            shown.display()
        );
    }
    pack
}

/// What one pair's baseline asks the engine.
pub struct Scenario {
    /// The pair whose output is frozen, e.g. `en-fr`.
    pub pair: &'static str,
    /// Where the pair's pack is built from: the committed tables, or a fixture until they are.
    pub pack: PackSource,
    /// Pairs loaded first, as the extension does: it starts with the default pair's pack and
    /// adds another listed pair's the first time its language is needed.
    pub beside: &'static [&'static str],
    /// The test target, for the messages (`english_baseline`).
    pub test: &'static str,
    /// The corpus under `tests/baseline/`: `=== name` opens a page, every other non-empty line
    /// is one block.
    pub pages: &'static str,
    /// The pages, in order, that the corpus must hold.
    pub page_names: &'static [&'static str],
    /// Dictionary forms whose gloss the card shows. Some have none in the pack, on purpose.
    pub lemmas: &'static [&'static str],
    /// Selections a reader glosses: expressions, sentences, and blocks in other languages.
    pub phrases: &'static [&'static str],
    /// (word as written, dictionary form) pairs a word card asks the grammar of.
    pub grammar: &'static [(&'static str, &'static str)],
    /// The pages analysed again for the reader with a history.
    pub reader_pages: &'static [&'static str],
    /// (lemma, status, offset in ms): every kind of status, the last one withdrawn.
    pub statuses: &'static [(&'static str, &'static str, f64)],
    /// Words recorded as read on three days.
    pub exposures: [&'static str; 5],
    /// Three cards, added a minute apart; the second is retired.
    pub cards: [Card; 3],
    /// A selection glossed by the reader with a history.
    pub reader_phrase: &'static str,
}

impl Scenario {
    fn crate_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    /// The committed tables, `scripts/lingua-data/tables`: a pair's native side in
    /// `<pair>/`, its studied language's side in `<studied>/`.
    pub fn tables_root() -> PathBuf {
        Self::crate_dir().join("../../scripts/lingua-data/tables")
    }

    /// A pair's committed native side, `scripts/lingua-data/tables/<pair>`.
    pub fn tables_dir(pair: &str) -> PathBuf {
        Self::tables_root().join(pair)
    }

    fn golden_path(&self) -> PathBuf {
        Self::crate_dir().join(format!("tests/baseline/{}.golden", self.pair))
    }

    /// The committed golden, as the repository holds it.
    pub fn committed_golden(&self) -> String {
        let path = self.golden_path();
        std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "{} is missing: run LINGUA_BLESS=1 cargo test -p lingua-wasm --test {}",
                path.display(),
                self.test
            )
        })
    }

    /// The pack the extension ships, built from the committed tables (the build is
    /// deterministic; `lingua-extension-check` holds the tables to `pin.json`).
    pub fn real_pack(pair: &str) -> Vec<u8> {
        let inputs = lingua_pack::inputs_from_tables(&Self::tables_root(), pair)
            .unwrap_or_else(|e| panic!("read the committed {pair} tables: {e}"));
        lingua_pack::build_pack(&inputs).unwrap_or_else(|e| panic!("build the {pair} pack: {e}"))
    }

    fn manifest(pair: &str, source: PackSource) -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(source.manifest_dir(pair).join("manifest.json"))
                .unwrap_or_else(|e| panic!("read the {pair} manifest: {e}")),
        )
        .unwrap_or_else(|e| panic!("parse the {pair} manifest: {e}"))
    }

    /// Where `pair`'s pack comes from: this scenario's source for its own pair, the committed
    /// tables for the pairs beside it.
    fn source_of(&self, pair: &str) -> PackSource {
        if pair == self.pair {
            self.pack
        } else {
            PackSource::Tables
        }
    }

    /// `pair, N bytes, pack_version …, analyzer_version …`.
    fn pack_line(pair: &str, pack: &[u8], source: PackSource) -> String {
        let manifest = Self::manifest(pair, source);
        format!(
            "{pair}, {} bytes, pack_version {}, analyzer_version {}",
            pack.len(),
            manifest["meta"]["pack_version"],
            manifest["meta"]["analyzer_version"],
        )
    }

    /// Every pack the extension holds for this pair, the beside pairs first.
    fn packs(&self) -> Vec<(&'static str, Vec<u8>)> {
        self.beside
            .iter()
            .chain(std::iter::once(&self.pair))
            .map(|pair| (*pair, self.source_of(pair).pack(pair)))
            .collect()
    }

    /// An engine started on the first pack, the others added in order.
    pub fn engine(packs: &[(&str, Vec<u8>)]) -> LinguaEngine {
        let (first, rest) = packs.split_first().expect("at least one pack");
        let mut engine = match LinguaEngine::new(&first.1) {
            Ok(e) => e,
            Err(_) => panic!("the {} pack loads", first.0),
        };
        for (pair, pack) in rest {
            if engine.add_pack(pack).is_err() {
                panic!("the {pair} pack is added");
            }
        }
        engine
    }

    /// An engine holding this pair's packs, for single assertions beside the golden.
    pub fn loaded(&self) -> LinguaEngine {
        Self::engine(&self.packs())
    }

    pub fn pages(&self) -> Vec<(String, Vec<String>)> {
        let path = Self::crate_dir().join("tests/baseline").join(self.pages);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read baseline/{}: {e}", self.pages));
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

    /// The corpus holds the pages the scenario names, each with a block, and the reader's
    /// pages are among them.
    pub fn check_corpus(&self) {
        let pages = self.pages();
        let names: Vec<&str> = pages.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, self.page_names);
        assert!(pages.iter().all(|(_, blocks)| !blocks.is_empty()));
        assert!(
            self.reader_pages.iter().all(|p| names.contains(p)),
            "every reader page exists"
        );
    }

    /// The golden text: every probe, in order, with the binding's output exactly as returned.
    pub fn render(&self, language: Option<&str>) -> String {
        self.render_with(self.packs(), language)
    }

    /// [`Scenario::render`] through the packs given, the pair's last: every probe of the
    /// scenario, answered by an engine holding them (the cross-native invariance test).
    pub fn render_with(&self, packs: Vec<(&str, Vec<u8>)>, language: Option<&str>) -> String {
        let lang = || language.map(str::to_owned);
        let pages = self.pages();
        let mut g = Golden::default();
        g.probe(
            "about",
            format!(
                "Generated by crates/lingua-wasm/tests/{}.rs. Re-bless, never edit by hand.",
                self.test
            ),
        );
        let (pair, pack) = packs.last().expect("the pair's pack");
        g.probe("pack", Self::pack_line(pair, pack, self.source_of(pair)));
        for (beside, pack) in &packs[..packs.len() - 1] {
            g.probe(
                &format!("beside {beside}"),
                Self::pack_line(beside, pack, self.source_of(beside)),
            );
        }

        // A new reader: no level, no calibration, no history.
        let fresh = Self::engine(&packs);
        for (name, blocks) in &pages {
            g.probe(
                &format!("analyse new-reader {name}"),
                fresh.analyse(blocks.clone(), lang()).unwrap(),
            );
        }
        for lemma in self.lemmas {
            g.probe(
                &format!("gloss {lemma}"),
                shown(fresh.gloss(lemma, lang()).unwrap()),
            );
        }
        for phrase in self.phrases {
            g.probe(
                &format!("phrase-gloss {phrase}"),
                fresh.phrase_gloss(phrase, lang()).unwrap(),
            );
        }
        for (written, lemma) in self.grammar {
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
        let mut reader = Self::engine(&packs);
        reader.set_calibration(3_000, lang()).unwrap();
        reader.set_declared_level_at("B1", T_MS, lang()).unwrap();
        for (lemma, status, offset_ms) in self.statuses {
            reader
                .set_status_at(lemma, status, T_MS + offset_ms, lang())
                .unwrap();
        }
        for day in 0..3 {
            reader
                .record_exposures(
                    self.exposures.map(str::to_owned).to_vec(),
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
        for (i, card) in self.cards.iter().enumerate() {
            reader
                .add_card(
                    card.lemma,
                    card.form,
                    card.sentence,
                    card.url,
                    card.gloss.map(str::to_owned),
                    T_SECS + 60.0 * i as f64,
                    lang(),
                )
                .unwrap();
        }
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
            .retire_card(self.cards[1].lemma, T_SECS + 300.0, lang())
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
        g.probe("deck-count", reader.deck_count(None));
        g.probe(
            "due-count in 7 days",
            reader.due_count(now + 7.0 * DAY_SECS, None),
        );
        for (name, blocks) in pages
            .iter()
            .filter(|(name, _)| self.reader_pages.contains(&name.as_str()))
        {
            g.probe(
                &format!("analyse reader {name}"),
                reader.analyse(blocks.clone(), lang()).unwrap(),
            );
        }
        g.probe(
            &format!("phrase-gloss reader {}", self.reader_phrase),
            reader.phrase_gloss(self.reader_phrase, lang()).unwrap(),
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

    /// With `LINGUA_BLESS` set, writes `actual` as the golden and answers `None`; otherwise
    /// answers the committed golden.
    pub fn bless_or_read(&self, actual: &str) -> Option<String> {
        let path = self.golden_path();
        if std::env::var_os("LINGUA_BLESS").is_some() {
            std::fs::write(&path, actual)
                .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
            return None;
        }
        Some(self.committed_golden())
    }
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

/// A golden's probes, by name, in order.
pub fn probes(text: &str) -> Vec<(String, String)> {
    format!("\n{text}")
        .split("\n### ")
        .skip(1)
        .map(|chunk| {
            let (name, body) = chunk.split_once('\n').unwrap_or((chunk, ""));
            (name.to_owned(), body.to_owned())
        })
        .collect()
}

/// Removes the native side of an output: glosses, their language, their senses, and expressions.
fn strip(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for key in ["gloss", "gloss_language", "senses", "expressions"] {
                map.remove(key);
            }
            map.values_mut().for_each(strip);
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(strip),
        _ => {}
    }
}

/// A probe's output with its native side removed; `None` for the probes that are native or
/// pack identity by definition: the pack lines, a card's gloss, the notice and the licences — a
/// real pack's attributions credit the sources of its glosses, so es-en's name the English and
/// Spanish Wiktionaries where es-fr's name the French one (add-lingua-pack-es-en D4), and en-es's
/// the Spanish one where en-fr's name the French one.
///
/// An engine's native language is its reader's (generalise-lingua-native-language), so the
/// backup records the reader's profile, and writes it in the schema version a profile other
/// than the default needs: both name the reader, not what the pack analyses.
pub fn studied_side(name: &str, body: &str) -> Option<String> {
    if name == "pack"
        || matches!(name, "notice" | "licences")
        || name.starts_with("beside ")
        || name.starts_with("gloss ")
    {
        return None;
    }
    Some(match serde_json::from_str::<serde_json::Value>(body) {
        Ok(mut value) => {
            strip(&mut value);
            if name == "backup"
                && let Some(backup) = value.as_object_mut()
            {
                backup.remove("profile");
                backup.remove("schema_version");
            }
            value.to_string()
        }
        Err(_) => body.to_owned(),
    })
}

/// The first probe that differs, with a window around the first character that does.
pub fn first_difference(expected: &str, actual: &str) -> String {
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
