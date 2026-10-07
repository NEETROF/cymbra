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

//! The committed tables are kept once per studied language (split-lingua-pack-tables-by-language,
//! M24): every pair reads its studied side from `tables/<studied>/`, which pins the language's
//! tag pool and names its reference pair's glossed lemmas as its dictionary words. Moving the
//! tables there left every shipped pack byte for byte as its pin records.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use lingua_core::packs::Pack;
use lingua_core::packs::format::read_container;
use lingua_core::packs::pack::section;
use lingua_pack::tables::{STUDIED_RECORD, check_committed_tables};
use lingua_pack::{PAIR_SIDE, STUDIED_SIDE, build_pack, inputs_from_dirs, inputs_from_tables};
use sha2::{Digest, Sha256};

fn tables() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/tables")
}

/// The files of a folder, by name, sorted.
fn files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.'))
        .collect();
    names.sort();
    names
}

/// A shipped pack, built once per test binary from the committed tables.
fn shipped(pair: &str) -> &'static [u8] {
    static EN_FR: OnceLock<Vec<u8>> = OnceLock::new();
    static ES_FR: OnceLock<Vec<u8>> = OnceLock::new();
    let cell = match pair {
        "en-fr" => &EN_FR,
        "es-fr" => &ES_FR,
        _ => unreachable!("{pair} is no shipped pair"),
    };
    cell.get_or_init(|| {
        let inputs = inputs_from_tables(&tables(), pair).unwrap_or_else(|e| panic!("{pair}: {e}"));
        build_pack(&inputs).unwrap_or_else(|e| panic!("build {pair}: {e}"))
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A scratch copy of a committed studied folder, to change one of its tables.
fn studied_copy(scratch: &Path, language: &str) -> PathBuf {
    let (from, to) = (tables().join(language), scratch.join(language));
    std::fs::create_dir_all(&to).unwrap();
    for name in files(&from) {
        std::fs::copy(from.join(&name), to.join(&name)).unwrap();
    }
    to
}

fn sections(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let (_, sections) = read_container(bytes).expect("a pack");
    sections.into_iter().map(|s| (s.name, s.data)).collect()
}

fn section_of<'a>(sections: &'a [(String, Vec<u8>)], name: &str) -> Option<&'a [u8]> {
    sections
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, data)| data.as_slice())
}

#[test]
fn every_committed_pair_holds_to_its_studied_language() {
    let pairs = check_committed_tables(&tables()).unwrap_or_else(|e| panic!("{e}"));
    for reference in ["en-fr", "es-fr"] {
        assert!(pairs.iter().any(|p| p == reference), "{reference} is read");
    }
}

#[test]
fn spec_scenario_one_copy_per_studied_language() {
    // English's and Spanish's studied tables are in tables/en and tables/es alone, written by
    // their reference pairs; en-fr and es-fr hold their native side, pin and README.
    let mut studied: Vec<String> = STUDIED_SIDE.iter().map(|t| (*t).to_owned()).collect();
    studied.push(STUDIED_RECORD.to_owned());
    studied.sort();
    let mut own: Vec<String> = PAIR_SIDE.iter().map(|t| (*t).to_owned()).collect();
    own.extend(["pin.json".to_owned(), "README.md".to_owned()]);
    own.sort();
    for (language, reference) in [("en", "en-fr"), ("es", "es-fr")] {
        assert_eq!(files(&tables().join(language)), studied, "{language}/");
        assert_eq!(files(&tables().join(reference)), own, "{reference}/");
        let record: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tables().join(language).join(STUDIED_RECORD)).unwrap(),
        )
        .unwrap();
        assert_eq!(
            record["reference"], reference,
            "{language}/{STUDIED_RECORD}"
        );
    }
}

#[test]
fn spec_scenario_the_shipped_packs_keep_their_bytes() {
    for pair in ["en-fr", "es-fr"] {
        let bytes = shipped(pair);
        let pin: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tables().join(pair).join("pin.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            pin["pack"]["sha256"],
            sha256_hex(bytes).as_str(),
            "{pair}: pin.json"
        );
        assert_eq!(pin["pack"]["size"], bytes.len(), "{pair}: pin.json");
        // A reference's dictionary words are its glossed lemmas: no lexical section.
        assert!(
            section_of(&sections(bytes), section::LEXICAL).is_none(),
            "{pair} carries no lexical table"
        );
    }
}

#[test]
fn spec_scenario_a_pair_glossed_in_another_native_language_copies_nothing() {
    // A test pair studying Spanish, glossed in English: its folder holds its glosses, senses,
    // expressions, notice and manifest alone, and its studied side is read from tables/es.
    let scratch = std::env::temp_dir().join(format!("lingua-es-en-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let es_en = scratch.join("es-en");
    std::fs::create_dir_all(&es_en).unwrap();
    let es_fr = tables().join("es-fr");
    for name in ["senses.tsv", "mwe.tsv", "NOTICE"] {
        std::fs::copy(es_fr.join(name), es_en.join(name)).unwrap();
    }
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(es_fr.join("manifest.json")).unwrap())
            .unwrap();
    manifest["meta"]["native"] = "en".into();
    std::fs::write(es_en.join("manifest.json"), manifest.to_string()).unwrap();
    // es-fr's glosses, and one of a ranked lemma es-fr does not gloss.
    let mut gloss = std::fs::read_to_string(es_fr.join("gloss.tsv")).unwrap();
    gloss.push_str("augusto\tAugustus\n");
    std::fs::write(es_en.join("gloss.tsv"), gloss).unwrap();
    assert_eq!(
        files(&es_en),
        [
            "NOTICE",
            "gloss.tsv",
            "manifest.json",
            "mwe.tsv",
            "senses.tsv"
        ]
    );

    let inputs = inputs_from_dirs(&tables().join("es"), &es_en).expect("read es-en");
    let bytes = build_pack(&inputs).expect("build es-en");
    std::fs::remove_dir_all(&scratch).unwrap();

    let reference = shipped("es-fr");
    let (a, b) = (sections(reference), sections(&bytes));
    // Its readings are stored byte for byte as es-fr's.
    for name in [
        section::FORMS,
        section::LEMMAS,
        section::FREQ,
        section::LEVELS,
        section::TAGS,
        section::PARADIGMS_ZST,
    ] {
        assert!(section_of(&a, name).is_some(), "es-fr carries {name}");
        assert!(
            section_of(&a, name) == section_of(&b, name),
            "es-en: the {name} section is not es-fr's"
        );
    }
    // Its dictionary words are es-fr's: es-fr's glossed lemmas, read from tables/es, not its own.
    let (es_fr_pack, es_en_pack) = (Pack::load(reference).unwrap(), Pack::load(&bytes).unwrap());
    assert_eq!(es_en_pack.pair().key(), "es-en");
    assert!(
        section_of(&b, section::LEXICAL).is_some(),
        "es-en: a lexical table"
    );
    assert!(es_en_pack.gloss("augusto").is_some() && es_fr_pack.gloss("augusto").is_none());
    assert!(!es_en_pack.is_dictionary_word("augusto"));
    assert_eq!(
        es_en_pack.dictionary_words(),
        es_fr_pack.dictionary_words(),
        "es-en's dictionary words are es-fr's"
    );
    let lexical: BTreeSet<String> = std::fs::read_to_string(tables().join("es/lexical.tsv"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert!(
        lexical
            .iter()
            .all(|lemma| es_en_pack.is_dictionary_word(lemma))
    );
}

#[test]
fn spec_scenario_a_pair_left_behind() {
    // A second pair studying Spanish, its pack recorded from the committed tables/es. A change
    // there — one level, or a lemma es-fr comes to gloss (*A rule of the reference's own
    // edition*) — moves that pair's pack, so its pin no longer records what its tables build and
    // build.sh fails naming it until it is recorded again (test_pack_sources.py,
    // test_build_sh_names_a_pair_left_behind_by_its_studied_tables).
    let scratch = std::env::temp_dir().join(format!("lingua-left-behind-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    // es-fr's glosses, read as another native language's: its dictionary words are its own
    // glossed lemmas, so it carries no lexical section until tables/es names others.
    let es_en = scratch.join("es-en");
    std::fs::create_dir_all(&es_en).unwrap();
    let es_fr = tables().join("es-fr");
    for name in ["gloss.tsv", "senses.tsv", "mwe.tsv", "NOTICE"] {
        std::fs::copy(es_fr.join(name), es_en.join(name)).unwrap();
    }
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(es_fr.join("manifest.json")).unwrap())
            .unwrap();
    manifest["meta"]["native"] = "en".into();
    std::fs::write(es_en.join("manifest.json"), manifest.to_string()).unwrap();
    let build = |studied: &Path| {
        build_pack(&inputs_from_dirs(studied, &es_en).expect("read es-en")).expect("build es-en")
    };
    let recorded = build(&tables().join("es"));
    assert!(section_of(&sections(&recorded), section::LEXICAL).is_none());

    // One level: `a`, A1 → A2.
    let es = studied_copy(&scratch, "es");
    let level = std::fs::read_to_string(es.join("level.tsv")).unwrap();
    assert!(level.starts_with("a\tA1\n"), "es/level.tsv opens on `a`");
    std::fs::write(
        es.join("level.tsv"),
        level.replacen("a\tA1\n", "a\tA2\n", 1),
    )
    .unwrap();
    let moved = build(&es);
    assert_ne!(
        sha256_hex(&moved),
        sha256_hex(&recorded),
        "es-en: es/level.tsv changed, and its pack did not"
    );
    assert_ne!(
        section_of(&sections(&moved), section::LEVELS),
        section_of(&sections(&recorded), section::LEVELS),
        "es-en: its levels are read from tables/es"
    );
    std::fs::write(es.join("level.tsv"), &level).unwrap();

    // es-fr comes to gloss `augusto`: Spanish's dictionary words gain it, and so does es-en.
    let lexical = std::fs::read_to_string(es.join("lexical.tsv")).unwrap();
    let mut words: Vec<&str> = lexical.lines().chain(["augusto"]).collect();
    words.sort_unstable();
    std::fs::write(
        es.join("lexical.tsv"),
        words.iter().map(|w| format!("{w}\n")).collect::<String>(),
    )
    .unwrap();
    let gained = build(&es);
    std::fs::remove_dir_all(&scratch).unwrap();
    assert_ne!(
        sha256_hex(&gained),
        sha256_hex(&recorded),
        "es-en: es/lexical.tsv gained a lemma, and its pack did not move"
    );
    assert!(Pack::load(&gained).unwrap().is_dictionary_word("augusto"));
    assert!(!Pack::load(&recorded).unwrap().is_dictionary_word("augusto"));
}
