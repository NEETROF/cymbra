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
    for pair in ["en-fr", "es-fr", "es-en", "en-es"] {
        assert!(pairs.iter().any(|p| p == pair), "{pair} is read");
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

/// es-en's committed pack, built once per test binary.
fn es_en() -> &'static [u8] {
    static ES_EN: OnceLock<Vec<u8>> = OnceLock::new();
    ES_EN.get_or_init(|| {
        let inputs =
            inputs_from_tables(&tables(), "es-en").unwrap_or_else(|e| panic!("es-en: {e}"));
        build_pack(&inputs).unwrap_or_else(|e| panic!("build es-en: {e}"))
    })
}

fn json(path: &Path) -> serde_json::Value {
    serde_json::from_str(
        &std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    )
    .unwrap()
}

#[test]
fn spec_scenario_the_first_reader_pair() {
    // es-en (add-lingua-pack-es-en): the first pair of a studied language's second native. Its
    // folder holds exactly a pair's file set — its native side, pin and README — its manifest
    // studies Spanish glossed in English, and its studied side is tables/es, which es-fr writes.
    let mut own: Vec<String> = PAIR_SIDE.iter().map(|t| (*t).to_owned()).collect();
    own.extend(["pin.json".to_owned(), "README.md".to_owned()]);
    own.sort();
    assert_eq!(files(&tables().join("es-en")), own, "es-en/");
    let manifest = json(&tables().join("es-en/manifest.json"));
    assert_eq!(manifest["meta"]["studied"], "es");
    assert_eq!(manifest["meta"]["native"], "en");
    let pairs = check_committed_tables(&tables()).unwrap_or_else(|e| panic!("{e}"));
    assert!(pairs.iter().any(|p| p == "es-en"), "es-en is read");

    // Its pin records its own pack, and — a reader pair's record (D3) — es-fr as the reference
    // and the sha256 of each of the six studied tables it was built on, as committed.
    let pin = json(&tables().join("es-en/pin.json"));
    let bytes = es_en();
    assert_eq!(
        pin["pack"]["sha256"],
        sha256_hex(bytes).as_str(),
        "es-en: pin.json"
    );
    assert_eq!(pin["pack"]["size"], bytes.len());
    assert_eq!(pin["studied"]["reference"], "es-fr");
    for name in STUDIED_SIDE {
        let committed = std::fs::read(tables().join("es").join(name)).unwrap();
        assert_eq!(
            pin["studied"]["tables"][name],
            sha256_hex(&committed).as_str(),
            "es-en: es/{name}"
        );
    }
    // The reference's pin records nothing of its readers.
    assert!(
        json(&tables().join("es-fr/pin.json"))
            .get("studied")
            .is_none()
    );
    // A reader pair's pack carries a lexical table: its dictionary words are es-fr's, not the
    // lemmas it glosses.
    assert!(section_of(&sections(bytes), section::LEXICAL).is_some());
    let pack = Pack::load(bytes).unwrap();
    assert_eq!(pack.pair().key(), "es-en");
    assert_eq!(
        pack.dictionary_words(),
        Pack::load(shipped("es-fr")).unwrap().dictionary_words(),
        "es-en's dictionary words are es-fr's"
    );
}

#[test]
fn spec_scenario_the_credits() {
    // es-en's notice names both sides' sources — the English Wiktionary's Spanish section, the
    // Spanish Wiktionary's translations, the French Wiktionary (es-fr's glosses decide the
    // dictionary words and which lemmas take a level), wordfreq and UD Spanish-GSD — and its
    // manifest says the levels are estimated (D4). The pack carries both.
    let notice = std::fs::read_to_string(tables().join("es-en/NOTICE")).unwrap();
    // Read as running text: a credit may wrap.
    let notice = notice.split_whitespace().collect::<Vec<_>>().join(" ");
    for credit in [
        "English Wiktionary (enwiktionary), Spanish section",
        "Spanish Wiktionary (eswiktionary)",
        "translations its Spanish entries list",
        "French Wiktionary (frwiktionary)",
        "which lemmas es-fr glosses: the dictionary words and which take a level",
        "wordfreq",
        "Robyn Speer",
        "UD Spanish-GSD",
        "levels are estimated",
    ] {
        assert!(notice.contains(credit), "es-en/NOTICE names {credit:?}");
    }
    let manifest = json(&tables().join("es-en/manifest.json"));
    assert_eq!(manifest["meta"]["levels_estimated"], true);
    assert_eq!(
        manifest["meta"]["licences"][0],
        "kaikki / enwiktionary, eswiktionary, frwiktionary (CC BY-SA 4.0 + GFDL)"
    );
    let pack = Pack::load(es_en()).unwrap();
    assert!(pack.meta().levels_estimated);
    for credit in ["UD Spanish-GSD", "eswiktionary", "frwiktionary"] {
        assert!(
            pack.notice().contains(credit),
            "the pack's notice names {credit:?}"
        );
    }
}

/// en-es's committed pack, built once per test binary.
fn en_es() -> &'static [u8] {
    static EN_ES: OnceLock<Vec<u8>> = OnceLock::new();
    EN_ES.get_or_init(|| {
        let inputs =
            inputs_from_tables(&tables(), "en-es").unwrap_or_else(|e| panic!("en-es: {e}"));
        build_pack(&inputs).unwrap_or_else(|e| panic!("build en-es: {e}"))
    })
}

#[test]
fn spec_scenario_english_glossed_in_spanish() {
    // en-es (add-lingua-pack-en-es): English glossed in Spanish, the first pair of a native
    // language no shipped pack speaks, a reader pair as es-en is. Its folder holds exactly a
    // pair's file set, its manifest studies English glossed in Spanish with English's analyser
    // and CEFR levels (not estimated), and its studied side is tables/en, which en-fr writes.
    let mut own: Vec<String> = PAIR_SIDE.iter().map(|t| (*t).to_owned()).collect();
    own.extend(["pin.json".to_owned(), "README.md".to_owned()]);
    own.sort();
    assert_eq!(files(&tables().join("en-es")), own, "en-es/");
    let manifest = json(&tables().join("en-es/manifest.json"));
    assert_eq!(manifest["meta"]["studied"], "en");
    assert_eq!(manifest["meta"]["native"], "es");
    assert_eq!(
        manifest["meta"]["analyzer_version"],
        lingua_core::analysis::ANALYZER_VERSION
    );
    assert!(manifest["meta"].get("levels_estimated").is_none());

    // Its pin records its own pack and, a reader pair's record (add-lingua-pack-es-en D3), en-fr
    // as the reference and the sha256 of each of the six studied tables it was built on.
    let pin = json(&tables().join("en-es/pin.json"));
    let bytes = en_es();
    assert_eq!(
        pin["pack"]["sha256"],
        sha256_hex(bytes).as_str(),
        "en-es: pin.json"
    );
    assert_eq!(pin["pack"]["size"], bytes.len());
    assert_eq!(pin["studied"]["reference"], "en-fr");
    for name in STUDIED_SIDE {
        let committed = std::fs::read(tables().join("en").join(name)).unwrap();
        assert_eq!(
            pin["studied"]["tables"][name],
            sha256_hex(&committed).as_str(),
            "en-es: en/{name}"
        );
    }
    // Its sources are dumps alone: no extract of its own (D2), the three derived files under its
    // own release.
    assert!(pin["sources"].get("kaikki").is_none(), "no extract");
    let release = pin["sources"]["kaikki-es"]["release"].as_str().unwrap();
    assert!(
        release.starts_with("lingua-pack-sources-en-es-"),
        "{release}"
    );
    assert_eq!(pin["sources"]["kaikki-en"]["release"], release);
    assert!(pin["sources"]["kaikki-es"]["files"]["kaikki-es-English.jsonl"].is_object());
    assert!(pin["sources"]["kaikki-es"]["files"]["kaikki-es-traductions-en.jsonl"].is_object());
    assert!(pin["sources"]["kaikki-en"]["files"]["kaikki-en-traductions-es.jsonl"].is_object());
    assert!(
        json(&tables().join("en-fr/pin.json"))
            .get("studied")
            .is_none(),
        "the reference's pin records nothing of its readers"
    );
    // Its dictionary words are en-fr's, so the pack carries a lexical table; its levels are
    // English's CEFR lists, read as committed.
    let pack = Pack::load(bytes).unwrap();
    assert_eq!(pack.pair().key(), "en-es");
    assert!(!pack.meta().levels_estimated);
    assert!(section_of(&sections(bytes), section::LEXICAL).is_some());
    assert_eq!(
        pack.dictionary_words(),
        Pack::load(shipped("en-fr")).unwrap().dictionary_words(),
        "en-es's dictionary words are en-fr's"
    );
    let house = pack.gloss("house").expect("house: glossed in Spanish");
    assert!(house.starts_with("Casa"), "house: {house}");
}

#[test]
fn spec_scenario_the_credits_of_en_es() {
    // en-es's notice names both sides' sources: the studied side as en-fr's notice credits it
    // (ESDB with its WordNet notice, wordfreq, the French Wiktionary's form links and dictionary
    // words, CEFR-J, Octanove) and the native side (the Spanish Wiktionary's definitions and
    // English translations, the English Wiktionary's Spanish translations). The pack carries it.
    let notice = std::fs::read_to_string(tables().join("en-es/NOTICE")).unwrap();
    let notice = notice.split_whitespace().collect::<Vec<_>>().join(" ");
    for credit in [
        "ESDB (English Speller Database, SCOWLv2, en-wl/wordlist)",
        "WordNet, used by ESDB",
        "wordfreq (English and Spanish frequency lists)",
        "Robyn Speer",
        "Spanish Wiktionary (eswiktionary)",
        "English translations its Spanish entries list",
        "English Wiktionary (enwiktionary)",
        "Spanish translations its English entries list",
        "French Wiktionary (frwiktionary)",
        "which lemmas en-fr glosses: the dictionary words",
        "CEFR-J: The CEFR-J Wordlist Version 1.5",
        "Octanove: Octanove Vocabulary Profile C1/C2 v1.0",
    ] {
        assert!(notice.contains(credit), "en-es/NOTICE names {credit:?}");
    }
    let manifest = json(&tables().join("en-es/manifest.json"));
    assert_eq!(
        manifest["meta"]["licences"][2],
        "kaikki / eswiktionary, enwiktionary, frwiktionary (CC BY-SA 4.0 + GFDL)"
    );
    let pack = Pack::load(en_es()).unwrap();
    for credit in [
        "ESDB",
        "eswiktionary",
        "enwiktionary",
        "frwiktionary",
        "CEFR-J",
        "Octanove",
    ] {
        assert!(
            pack.notice().contains(credit),
            "the pack's notice names {credit:?}"
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
