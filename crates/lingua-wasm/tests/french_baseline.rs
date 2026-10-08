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

//! French invariance baseline (`docs/lingua/language-matrix-programme.md`, change 39,
//! add-lingua-french-baseline).
//!
//! French is a studied language served by the baseline analysis (analyser `0.1.0`): the rules that
//! belong to no language and the pack's forms, nothing else — `l'homme`, `aujourd'hui`, `au`, `du`
//! and every elision stay whole, nothing is a function word, there is no names rule and no NFC.
//! This freezes what the engine makes of raw French text today, over a thirteen-page corpus
//! (`baseline/pages-fr.txt`, its sources in `support/french.rs`), so that each later change of
//! the French stage shows its effect as the diff of a re-bless. It is not French support: no pair
//! studying French is listed.
//!
//! The pack is the hand-written fr-en fixture (`scripts/lingua-data/testdata/fr-en/`) until the
//! French tables are committed (change 48); the engine starts on the real es-en pack, built from
//! the committed tables, as an English-native reader's engine does.
//!
//! A pull request that changes `baseline/fr-en.golden` re-blesses it with
//! `LINGUA_BLESS=1 cargo test -p lingua-wasm --test french_baseline` and says why: a French rule
//! that bumps French's analyser version (and the fixture's manifest with it), the fixture
//! replaced by the committed tables, or an es-en update, which moves the `beside es-en` line
//! alone. `lingua-pack-update` re-blesses on its own branch.
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use std::sync::OnceLock;

use support::french::FRENCH;
use support::{PackSource, first_difference, testdata_pack};
use unicode_normalization::UnicodeNormalization;

/// The `technique` page's block committed in NFD.
const NFD_BLOCK: usize = 3;

fn fr() -> Option<String> {
    Some("fr".to_owned())
}

/// The golden as this build renders it, once for every test of the target.
fn rendered() -> &'static str {
    static RENDERED: OnceLock<String> = OnceLock::new();
    RENDERED.get_or_init(|| FRENCH.render(Some("fr")))
}

#[test]
fn french_output_has_not_moved() {
    let actual = rendered();
    let Some(expected) = FRENCH.bless_or_read(actual) else {
        return;
    };
    assert!(
        actual == expected,
        "French output moved (add-lingua-french-baseline: the French golden moves only with a \
         reason).\n\
         First difference — {}\n\
         If this pull request means to change French output (a French rule that bumps French's \
         analyser version, the fixture replaced by the committed tables, or an es-en update), \
         re-bless with\n  \
         LINGUA_BLESS=1 cargo test -p lingua-wasm --test french_baseline\n\
         and say why in the pull request. Otherwise the change is wrong.",
        first_difference(&expected, actual)
    );
}

#[test]
fn the_corpus_reads_as_expected() {
    FRENCH.check_corpus();
    // One block of the technical page is committed in NFD on purpose: an editor or a formatter
    // that composes the file must fail here, not move the golden unnoticed.
    let pages = FRENCH.pages();
    let (_, technique) = pages
        .iter()
        .find(|(name, _)| name == "technique")
        .expect("the technique page");
    let block = &technique[NFD_BLOCK];
    assert!(
        block.contains('\u{0301}'),
        "the technique page's NFD block was composed"
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
    // The fiction and Proust pages are set in French punctuation.
    for name in ["fiction", "proust"] {
        let (_, blocks) = pages.iter().find(|(n, _)| n == name).expect(name);
        let text = blocks.concat();
        assert!(
            text.contains('\u{2019}'),
            "{name}: the typographic apostrophe"
        );
        assert!(
            text.contains("\u{202f}»"),
            "{name}: a narrow no-break space before »"
        );
    }
    let fiction = pages
        .iter()
        .find(|(n, _)| n == "fiction")
        .map(|(_, blocks)| blocks.concat())
        .expect("fiction");
    assert!(fiction.contains("\u{202f}?") && fiction.contains("\u{202f}!"));
}

/// The page's blocks, analysed as French by `engine`, as JSON.
fn analysed(engine: &lingua_wasm::LinguaEngine, page: &str) -> serde_json::Value {
    let (_, blocks) = FRENCH
        .pages()
        .into_iter()
        .find(|(name, _)| name == page)
        .unwrap_or_else(|| panic!("the {page} page"));
    serde_json::from_str(&engine.analyse(blocks, fr()).expect("analysed")).expect("JSON")
}

#[test]
fn french_is_the_baseline_until_its_rules_are_written() {
    let mut engine = FRENCH.loaded();
    assert_eq!(engine.languages(), r#"["es","fr"]"#, "es-en beside fr-en");
    assert_eq!(engine.native_language(), "en");

    let elisions = analysed(&engine, "elisions");
    assert_eq!(elisions["analyzer_version"], "0.1.0");
    let tokens = elisions["tokens"].as_array().expect("tokens");
    let homme = tokens
        .iter()
        .find(|t| t["surface"] == "L'homme")
        .expect("`L'homme` is one token");
    assert_eq!(homme["lemma"], "l'homme");

    let contractions = analysed(&engine, "contractions");
    assert_eq!(contractions["analyzer_version"], "0.1.0");
    assert!(
        contractions["tokens"]
            .as_array()
            .expect("tokens")
            .iter()
            .any(|t| t["surface"] == "au" && t["lemma"] == "au"),
        "`au` is whole"
    );

    // No word of any page is a function word: French has no table yet.
    for (name, blocks) in FRENCH.pages() {
        for block in blocks {
            let glossed = engine.phrase_gloss(&block, fr()).expect("glossed");
            assert!(
                !glossed.contains(r#""function_word":true"#),
                "{name}: {glossed}"
            );
        }
    }

    // The reader's records are French, under the profile the engine started with (Spanish,
    // English native): the backup names French, so it is schema version 3, and the golden's
    // `backup` probe records it.
    assert_eq!(engine.studied_languages(), r#"["es"]"#);
    assert!(engine.backup().starts_with("{\n  \"schema_version\": 2,"));
    engine
        .set_status_at("phare", "known", 1_790_000_000_000.0, fr())
        .expect("a French status");
    assert!(engine.backup().starts_with("{\n  \"schema_version\": 3,"));
    let backup = rendered()
        .split("\n### backup\n")
        .nth(1)
        .expect("the backup probe");
    assert!(
        backup.starts_with("{\n  \"schema_version\": 3,"),
        "{backup}"
    );
}

#[test]
fn the_nfd_block_s_memoire_is_glossed_once_french_composes_it() {
    // The fixture lists `mémoire`, glossed `memory`, a word the corpus has only in its NFD block
    // (D5): read as it came, decomposed, the word is not the pack's and has no gloss; change 41's
    // NFC makes the gloss appear in the golden, not only bytes no one sees move.
    let engine = FRENCH.loaded();
    let technique = analysed(&engine, "technique");
    let decomposed = "me\u{301}moire";
    let token = technique["tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .find(|t| t["block"] == NFD_BLOCK && t["surface"] == decomposed)
        .expect("the NFD block's `mémoire`, as it came");
    assert_eq!(token["lemma"], decomposed, "lowercased, not composed");
    assert!(token["gloss"].is_null(), "{token}");
    assert_eq!(
        engine.gloss("m\u{e9}moire", fr()).expect("glossed"),
        Some("memory".to_owned()),
        "the pack glosses the composed form"
    );
}

#[test]
fn a_fixture_left_behind_its_analyser_names_its_manifest() {
    // A French rule bumps French's analyser version; a fixture manifest left at the old one is
    // refused by the core. The harness names the manifest to bump, not wasm-bindgen's host panic.
    let committed = PackSource::Testdata.manifest_dir("fr-en");
    let behind = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("fr-en-fixture-behind");
    let _ = std::fs::remove_dir_all(&behind);
    std::fs::create_dir_all(&behind).expect("a scratch fixture");
    for entry in std::fs::read_dir(&committed).expect("the fr-en fixture") {
        let entry = entry.expect("a fixture file");
        std::fs::copy(entry.path(), behind.join(entry.file_name())).expect("copied");
    }
    let manifest_path = behind.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).expect("manifest"))
            .expect("JSON");
    assert_eq!(manifest["meta"]["analyzer_version"], "0.1.0");
    manifest["meta"]["analyzer_version"] = "0.0.9".into();
    std::fs::write(&manifest_path, manifest.to_string()).expect("written");

    let Err(refused) = std::panic::catch_unwind(|| testdata_pack(&behind, "fr-en")) else {
        panic!("a fixture behind French's analyser is refused");
    };
    let message = refused.downcast_ref::<String>().expect("a formatted panic");
    assert!(
        message.contains("pack built for analyzer 0.0.9 but this core is 0.1.0"),
        "{message}"
    );
    assert!(
        message.contains("bump `analyzer_version` in")
            && message.contains("fr-en-fixture-behind/manifest.json"),
        "{message}"
    );
    assert!(!message.contains("wasm-bindgen"), "{message}");
    // The committed fixture loads.
    assert!(!testdata_pack(&committed, "fr-en").is_empty());
}
