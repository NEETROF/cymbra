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

//! Italian invariance baseline (`docs/lingua/italian-programme.md`, change 5,
//! add-lingua-italian-baseline).
//!
//! Italian is a studied language served by the baseline analysis (analyser `0.1.0`): the rules
//! that belong to no language and the pack's forms, nothing else — `l'uomo`, `dell'acqua`, `c'è`,
//! `del`, `nel`, `col` and `dirmelo` stay whole, `po'` is `po` by the edge trim every language
//! has, nothing is a function word, there is no names rule, no NFC and no detection guard, so the
//! `misto` page's Catalan and Galician blocks are analysed as Italian. This freezes what the
//! engine makes of raw Italian text today, over a thirteen-page corpus (`baseline/pages-it.txt`,
//! its sources in `support/italian.rs`), so that each later change of the Italian stage shows its
//! effect as the diff of a re-bless. It is not Italian support: no listed pair studies Italian.
//!
//! The pack is the hand-written it-fr fixture (`scripts/lingua-data/testdata/it-fr/`) until the
//! Italian tables are committed (change 16); the engine starts on the real en-fr pack, built from
//! the committed tables, as a French-native reader's engine does.
//!
//! A pull request that changes `baseline/it-fr.golden` re-blesses it with
//! `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline` and says why: an Italian
//! rule that bumps Italian's analyser version (and the fixture's manifest with it), the fixture
//! replaced by the committed tables, or an en-fr update, which moves the `beside en-fr` line
//! alone. `lingua-pack-update` re-blesses on its own branch.
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use std::sync::OnceLock;

use support::italian::ITALIAN;
use support::{PackSource, first_difference, testdata_pack};
use unicode_normalization::UnicodeNormalization;

/// The `tecnico` page's block committed in NFD.
const NFD_BLOCK: usize = 3;

/// The `misto` page's blocks: Italian, English, Spanish, French, Catalan, Galician, Latin, Italian.
const MISTO_ITALIAN: [usize; 2] = [0, 7];
const MISTO_ENGLISH_SPANISH_FRENCH: [usize; 3] = [1, 2, 3];
const MISTO_CATALAN_GALICIAN: [usize; 2] = [4, 5];
const MISTO_LATIN: usize = 6;

fn it() -> Option<String> {
    Some("it".to_owned())
}

/// The golden as this build renders it, once for every test of the target.
fn rendered() -> &'static str {
    static RENDERED: OnceLock<String> = OnceLock::new();
    RENDERED.get_or_init(|| ITALIAN.render(Some("it")))
}

#[test]
fn italian_output_has_not_moved() {
    let actual = rendered();
    let Some(expected) = ITALIAN.bless_or_read(actual) else {
        return;
    };
    assert!(
        actual == expected,
        "Italian output moved (add-lingua-italian-baseline: the Italian golden moves only with a \
         reason).\n\
         First difference — {}\n\
         If this pull request means to change Italian output (an Italian rule that bumps Italian's \
         analyser version, the fixture replaced by the committed tables, or an en-fr update), \
         re-bless with\n  \
         LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline\n\
         and say why in the pull request. Otherwise the change is wrong.",
        first_difference(&expected, actual)
    );
}

/// The blocks of the corpus page `name`.
fn page(name: &str) -> Vec<String> {
    ITALIAN
        .pages()
        .into_iter()
        .find(|(page, _)| page == name)
        .map(|(_, blocks)| blocks)
        .unwrap_or_else(|| panic!("the {name} page"))
}

#[test]
fn the_corpus_reads_as_expected() {
    ITALIAN.check_corpus();
    assert_eq!(ITALIAN.pages().len(), 13);
    // One block of the technical page is committed in NFD on purpose: an editor or a formatter
    // that composes the file must fail here, not move the golden unnoticed.
    let tecnico = page("tecnico");
    let block = &tecnico[NFD_BLOCK];
    assert!(
        block.contains('\u{0300}'),
        "the tecnico page's NFD block was composed"
    );
    assert_ne!(
        block.nfc().collect::<String>(),
        *block,
        "NFC composes the NFD block"
    );
    assert_eq!(
        block.nfd().collect::<String>(),
        *block,
        "the NFD block is wholly decomposed"
    );
    assert!(block.contains("citta\u{0300}"), "{block}");
    // Its `città` is the corpus's only one, so its gloss appears in the golden with NFC alone.
    let corpus = ITALIAN
        .pages()
        .into_iter()
        .flat_map(|(_, blocks)| blocks)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !corpus.contains("citt\u{e0}"),
        "a composed `città` elsewhere"
    );
    // Every other block is in NFC.
    for (name, blocks) in ITALIAN.pages() {
        for (i, block) in blocks.iter().enumerate() {
            if (name.as_str(), i) != ("tecnico", NFD_BLOCK) {
                assert_eq!(block.nfc().collect::<String>(), *block, "{name} {i}");
            }
        }
    }
    // The narrativa and manzoni pages are set in Italian punctuation; narrativa has « », the
    // dialogue dash and the accented capital.
    for name in ["narrativa", "manzoni"] {
        let text = page(name).concat();
        assert!(
            text.contains('\u{2019}'),
            "{name}: the typographic apostrophe"
        );
        assert!(!text.contains('\''), "{name}: no straight apostrophe");
    }
    let narrativa = page("narrativa").concat();
    assert!(narrativa.contains("«È") && narrativa.contains('»') && narrativa.contains('—'));
    // The informale page writes the apostrophe for an accent, in ASCII.
    let informale = page("informale").concat();
    for written in ["perche'", "citta'", "piu'", "E'"] {
        assert!(informale.contains(written), "{written}");
    }
    // The excerpts open their pages.
    assert!(page("manzoni")[0].starts_with("Quel ramo del lago di Como, che volge a mezzogiorno,"));
    assert!(page("nomi")[0].starts_with("— C’era una volta...."));
}

/// The page's blocks, analysed as Italian by `engine`, as JSON.
fn analysed(engine: &lingua_wasm::LinguaEngine, name: &str) -> serde_json::Value {
    serde_json::from_str(&engine.analyse(page(name), it()).expect("analysed")).expect("JSON")
}

/// The (surface, lemma, block) of every token of an analysed page.
fn tokens(page: &serde_json::Value) -> Vec<(String, String, u64)> {
    page["tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .map(|t| {
            (
                t["surface"].as_str().expect("a surface").to_owned(),
                t["lemma"].as_str().expect("a lemma").to_owned(),
                t["block"].as_u64().expect("a block"),
            )
        })
        .collect()
}

fn has(tokens: &[(String, String, u64)], surface: &str, lemma: &str) -> bool {
    tokens.iter().any(|(s, l, _)| s == surface && l == lemma)
}

#[test]
fn what_the_baseline_shows_today() {
    let engine = ITALIAN.loaded();
    assert_eq!(engine.languages(), r#"["en","it"]"#, "en-fr beside it-fr");
    assert_eq!(engine.native_language(), "fr");

    // Every page reports Italian's own version, the baseline's.
    for name in ITALIAN.page_names {
        assert_eq!(
            analysed(&engine, name)["analyzer_version"],
            "0.1.0",
            "{name}"
        );
    }

    // An elided word is one token, its lemma itself, lowercased; so is `c'è`.
    let elisioni = tokens(&analysed(&engine, "elisioni"));
    assert!(has(&elisioni, "L'uomo", "l'uomo"), "{elisioni:?}");
    assert!(has(&elisioni, "un'amica", "un'amica"), "{elisioni:?}");
    assert!(has(&elisioni, "dell'acqua", "dell'acqua"), "{elisioni:?}");
    assert!(has(&elisioni, "c'è", "c'è"), "{elisioni:?}");
    // A truncation loses its trailing apostrophe, as any word's edge apostrophe does.
    assert!(has(&elisioni, "po", "po"), "{elisioni:?}");
    assert!(has(&elisioni, "fa", "fare"), "{elisioni:?}");

    // Articulated prepositions are whole, in both readings: no written word gives two tokens.
    let page_json = analysed(&engine, "preposizioni");
    let spans: Vec<_> = page_json["tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .map(|t| (t["block"].clone(), t["start"].clone(), t["end"].clone()))
        .collect();
    for (i, span) in spans.iter().enumerate() {
        assert!(!spans[..i].contains(span), "a split word at {span:?}");
    }
    let preposizioni = tokens(&page_json);
    for written in [
        "del", "dello", "della", "nel", "al", "dal", "sul", "col", "coi",
    ] {
        assert!(
            preposizioni
                .iter()
                .any(|(s, l, _)| s.to_lowercase() == written && l == written),
            "{written}: {preposizioni:?}"
        );
    }

    // An enclitic form is one token, its lemma itself.
    let clitici = tokens(&analysed(&engine, "clitici"));
    for written in [
        "dirmelo", "dammelo", "vattene", "eccolo", "farcela", "scrivimi",
    ] {
        assert!(has(&clitici, written, written), "{written}: {clitici:?}");
    }

    // No token of any page is a function word: Italian has no table yet.
    for (name, blocks) in ITALIAN.pages() {
        for block in blocks {
            let glossed = engine.phrase_gloss(&block, it()).expect("glossed");
            assert!(
                !glossed.contains(r#""function_word":true"#),
                "{name}: {glossed}"
            );
        }
    }

    // The misto page: its Italian blocks, and its Catalan and Galician ones — whichlang reads them
    // as Italian and Italian has no guard yet —, are analysed; its English, Spanish, French and
    // Latin ones (whichlang reads the Latin as French) are not.
    let misto = tokens(&analysed(&engine, "misto"));
    let analysed_blocks =
        |block: usize| misto.iter().filter(|(_, _, b)| *b == block as u64).count();
    for block in MISTO_ITALIAN.into_iter().chain(MISTO_CATALAN_GALICIAN) {
        assert!(analysed_blocks(block) > 5, "block {block}: {misto:?}");
    }
    for block in MISTO_ENGLISH_SPANISH_FRENCH
        .into_iter()
        .chain([MISTO_LATIN])
    {
        assert_eq!(analysed_blocks(block), 0, "block {block}: {misto:?}");
    }
    assert!(has(&misto, "s'alçava", "s'alçava"), "{misto:?}");
    assert!(has(&misto, "unha", "unha"), "{misto:?}");
}

#[test]
fn the_readers_backup_is_version_4() {
    // The engine starts on en-fr, as a French-native reader's: the default profile, English
    // studied, at version 1. The reader's records are Italian, so the backup names Italian and
    // is schema version 4, and the golden's `backup` probe records it.
    let mut engine = ITALIAN.loaded();
    assert_eq!(engine.studied_languages(), r#"["en"]"#);
    assert!(engine.backup().starts_with("{\n  \"schema_version\": 1,"));
    engine
        .set_status_at("faro", "known", 1_790_000_000_000.0, it())
        .expect("an Italian status");
    let backup = engine.backup();
    assert!(
        backup.starts_with("{\n  \"schema_version\": 4,"),
        "{backup}"
    );
    assert!(backup.contains("\"Italian\": {"), "{backup}");
    let golden = rendered()
        .split("\n### backup\n")
        .nth(1)
        .expect("the backup probe");
    assert!(
        golden.starts_with("{\n  \"schema_version\": 4,"),
        "{golden}"
    );
    assert!(golden.contains("\"Italian\": {"), "{golden}");
}

#[test]
fn the_nfd_block_s_citta_is_glossed_once_italian_composes_it() {
    // The fixture lists `città`, glossed `ville`, a word the corpus has only in its NFD block
    // (D5): read as it came, decomposed, the word is not the pack's and has no gloss; change 7's
    // NFC makes the gloss appear in the golden, not only bytes no one sees move.
    let engine = ITALIAN.loaded();
    let tecnico = analysed(&engine, "tecnico");
    let decomposed = "citta\u{300}";
    let token = tecnico["tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .find(|t| t["block"] == NFD_BLOCK && t["surface"] == decomposed)
        .expect("the NFD block's `città`, as it came");
    assert_eq!(token["lemma"], decomposed, "lowercased, not composed");
    assert!(token["gloss"].is_null(), "{token}");
    assert_eq!(
        engine.gloss("citt\u{e0}", it()).expect("glossed"),
        Some("ville".to_owned()),
        "the pack glosses the composed form"
    );
}

#[test]
fn a_fixture_left_behind_its_analyser_names_its_manifest() {
    // An Italian rule bumps Italian's analyser version; a fixture manifest left at the old one is
    // refused by the core. The harness names the manifest to bump, not wasm-bindgen's host panic.
    let committed = PackSource::Testdata.manifest_dir("it-fr");
    let behind = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("it-fr-fixture-behind");
    let _ = std::fs::remove_dir_all(&behind);
    std::fs::create_dir_all(&behind).expect("a scratch fixture");
    for entry in std::fs::read_dir(&committed).expect("the it-fr fixture") {
        let entry = entry.expect("a fixture file");
        std::fs::copy(entry.path(), behind.join(entry.file_name())).expect("copied");
    }
    let manifest_path = behind.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).expect("manifest"))
            .expect("JSON");
    assert_eq!(
        manifest["meta"]["analyzer_version"],
        lingua_core::analysis::ITALIAN_ANALYZER_VERSION
    );
    manifest["meta"]["analyzer_version"] = "0.0.9".into();
    std::fs::write(&manifest_path, manifest.to_string()).expect("written");

    let Err(refused) = std::panic::catch_unwind(|| testdata_pack(&behind, "it-fr")) else {
        panic!("a fixture behind Italian's analyser is refused");
    };
    let message = refused.downcast_ref::<String>().expect("a formatted panic");
    assert!(
        message.contains("pack built for analyzer 0.0.9 but this core is 0.1.0"),
        "{message}"
    );
    assert!(
        message.contains("bump `analyzer_version` in")
            && message.contains("it-fr-fixture-behind/manifest.json"),
        "{message}"
    );
    assert!(!message.contains("wasm-bindgen"), "{message}");
    // The committed fixture loads.
    assert!(!testdata_pack(&committed, "it-fr").is_empty());
}
