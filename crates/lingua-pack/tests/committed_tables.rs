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
//! tag pool and names its reference pair's glossed lemmas as its dictionary words — French's less
//! those fr-en glosses by a proper noun's senses alone (refine-lingua-fr-en-glosses D2). Moving the
//! tables there left every shipped pack byte for byte as its pin records.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::lexicon::Lexicon;
use lingua_core::analysis::pipeline::headword_reading;
use lingua_core::analysis::tokenize::{
    FRENCH_ELISIONS, FRENCH_INVERSION_PRONOUNS, is_elided, tokenize,
};
use lingua_core::engine::french_expression_key;
use lingua_core::knowledge::level::CefrLevel;
use lingua_core::packs::Pack;
use lingua_core::packs::format::read_container;
use lingua_core::packs::grammar::Tag;
use lingua_core::packs::pack::section;
use lingua_pack::studied_of;
use lingua_pack::tables::{STUDIED_RECORD, check_committed_tables};
use lingua_pack::{
    MAX_PACK_BYTES, PAIR_SIDE, STUDIED_SIDE, build_pack, inputs_from_dirs, inputs_from_tables,
};
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
    for pair in ["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"] {
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

/// fr-en's committed pack, built once per test binary.
fn fr_en() -> &'static [u8] {
    static FR_EN: OnceLock<Vec<u8>> = OnceLock::new();
    FR_EN.get_or_init(|| {
        let inputs =
            inputs_from_tables(&tables(), "fr-en").unwrap_or_else(|e| panic!("fr-en: {e}"));
        build_pack(&inputs).unwrap_or_else(|e| panic!("build fr-en: {e}"))
    })
}

/// A committed two-column table, `key → value`, as the builder reads it.
fn tsv(path: &Path) -> BTreeMap<String, String> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .lines()
        .map(|line| {
            let (key, value) = line.split_once('\t').unwrap_or_else(|| panic!("{line:?}"));
            (key.to_owned(), value.to_owned())
        })
        .collect()
}

/// The lemmas fr-en glosses by a proper noun's senses alone: every run of their `senses.tsv` row
/// is `PROPN` (refine-lingua-fr-en-glosses D2).
fn fr_en_names() -> BTreeSet<String> {
    tsv(&tables().join("fr-en/senses.tsv"))
        .into_iter()
        .filter(|(_, runs)| runs.split('\t').all(|run| run.starts_with("PROPN:")))
        .map(|(lemma, _)| lemma)
        .collect()
}

#[test]
fn spec_scenario_the_reference_pair_writes_french_s_folder() {
    // fr-en (add-lingua-french-forms-tables D1, D10), French's reference pair: tables/fr/ holds
    // French's forms and ranks, its readings and their pinned tag pool
    // (add-lingua-french-grammar-tables), its estimated levels (add-lingua-french-levels), its record
    // naming fr-en, and the dictionary words, fr-en's glossed lemmas (add-lingua-pack-fr-en) less
    // those it glosses by a proper noun's senses alone (refine-lingua-fr-en-glosses D2).
    let fr = tables().join("fr");
    assert_eq!(
        files(&fr),
        [
            "forms.tsv",
            "freq.tsv",
            "grammar.tsv",
            "level.tsv",
            "lexical.tsv",
            "studied.json",
            "tags.tsv"
        ]
    );
    assert_eq!(json(&fr.join(STUDIED_RECORD))["reference"], "fr-en");
    // tables/fr-en/ holds a pair's native side — its glosses, sense runs and expressions — its
    // notice, manifest, pin and README, and nothing else (add-lingua-pack-fr-en: *The same
    // snapshot*).
    let fr_en_dir = tables().join("fr-en");
    assert_eq!(
        files(&fr_en_dir),
        [
            "NOTICE",
            "README.md",
            "gloss.tsv",
            "manifest.json",
            "mwe.tsv",
            "pin.json",
            "senses.tsv"
        ]
    );
    // French's dictionary words are exactly the lemmas fr-en glosses less its names — `paris`,
    // `durand` —, byte-sorted, one per line (*French's dictionary words*, *French's names*): a word
    // with a common sense beside a name's stays one (`lot`, `marche`, `nice`).
    let glossed: Vec<String> = tsv(&fr_en_dir.join("gloss.tsv")).into_keys().collect();
    assert!(glossed.len() > 30_000, "{}", glossed.len());
    let names = fr_en_names();
    assert!(names.len() > 3_000, "{}", names.len());
    for name in ["paris", "durand", "lyon", "france", "coran"] {
        assert!(names.contains(name), "{name} is glossed as a name alone");
    }
    let words: Vec<&String> = glossed.iter().filter(|l| !names.contains(*l)).collect();
    let lexical = std::fs::read_to_string(fr.join("lexical.tsv")).unwrap();
    assert_eq!(lexical.lines().collect::<Vec<_>>(), words, "fr/lexical.tsv");
    assert!(lexical.ends_with('\n'));
    for word in ["lot", "marche", "nice", "le", "des", "maison"] {
        assert!(lexical.lines().any(|l| l == word), "{word} is a word");
    }
    let manifest = json(&fr_en_dir.join("manifest.json"));
    assert_eq!(manifest["meta"]["studied"], "fr");
    assert_eq!(manifest["meta"]["native"], "en");
    assert_eq!(
        manifest["meta"]["analyzer_version"],
        lingua_core::analysis::FRENCH_ANALYZER_VERSION
    );
    // Every ranked lemma is the lemma of at least one form.
    let forms = tsv(&fr.join("forms.tsv"));
    let lemmas: BTreeSet<&String> = forms.values().collect();
    let ranks = tsv(&fr.join("freq.tsv"));
    assert_eq!(ranks.len(), 60_000);
    let unreached: Vec<&String> = ranks.keys().filter(|l| !lemmas.contains(l)).collect();
    assert!(
        unreached.is_empty(),
        "ranked, no form's lemma: {unreached:?}"
    );
    // The reference's pin records its sources and pack, and nothing of a studied record.
    let pin = json(&fr_en_dir.join("pin.json"));
    assert!(pin.get("studied").is_none());
}

#[test]
fn spec_scenario_the_pack_builds_where_the_others_do() {
    // fr-en's pack, built from tables/fr/ and tables/fr-en/, has the sha256 its pin records, carries
    // a lexical section (its dictionary words leave out the lemmas it glosses as names alone,
    // refine-lingua-fr-en-glosses D2), fits the budget, and no package lists it
    // (add-lingua-pack-fr-en: *The pack is built, not shipped*).
    let bytes = fr_en();
    let pin = json(&tables().join("fr-en/pin.json"));
    assert_eq!(
        pin["pack"]["sha256"],
        sha256_hex(bytes).as_str(),
        "fr-en: pin.json"
    );
    assert_eq!(pin["pack"]["size"], bytes.len());
    assert!(bytes.len() < MAX_PACK_BYTES);
    assert!(section_of(&sections(bytes), section::LEXICAL).is_some());
    let pack = Pack::load(bytes).unwrap();
    assert_eq!(pack.pair().key(), "fr-en");
    // A vocabulary estimate counts the ranked lemmas that are dictionary words or carry a level
    // (`dictionary_words`): tables/fr/lexical.tsv's, every one of them ranked, since every levelled
    // lemma is one of them.
    let lexical: BTreeSet<String> = std::fs::read_to_string(tables().join("fr/lexical.tsv"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    let ranks = tsv(&tables().join("fr/freq.tsv"));
    assert!(lexical.iter().all(|l| ranks.contains_key(l)));
    assert_eq!(pack.dictionary_words().len(), lexical.len());
    assert!(pack.is_dictionary_word("maison"));
    assert!(pack.gloss("maison").is_some_and(|g| g.starts_with("house")));
    // A name keeps its gloss but is no word to learn.
    assert!(!pack.is_dictionary_word("paris"));
    assert!(pack.gloss("paris").is_some_and(|g| g.starts_with("Paris")));
    let packs = json(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/lingua-extension/packs.json"),
    );
    assert!(
        !packs["pairs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == "fr-en"),
        "no package lists fr-en"
    );
}

#[test]
fn spec_scenario_a_gloss_is_its_lemma_s() {
    // Every lemma fr-en glosses is a ranked lemma the forms table reads as itself, so the builder,
    // which files a gloss under the lemma its key reads as, lends no gloss to another word
    // (add-lingua-pack-fr-en D1): the pack answers each lemma with its own row. The prototype, on
    // tables ranking `venue` (read as venir), glossed venir « coming, arrival ».
    let forms = tsv(&tables().join("fr/forms.tsv"));
    let ranks = tsv(&tables().join("fr/freq.tsv"));
    let glossed = tsv(&tables().join("fr-en/gloss.tsv"));
    let senses = tsv(&tables().join("fr-en/senses.tsv"));
    let elsewhere: Vec<(&String, Option<&String>)> = glossed
        .keys()
        .map(|lemma| (lemma, forms.get(lemma)))
        .filter(|(lemma, read)| *read != Some(*lemma) || !ranks.contains_key(*lemma))
        .take(10)
        .collect();
    assert!(
        elsewhere.is_empty(),
        "glossed, no ranked lemma read as itself: {elsewhere:?}"
    );
    // A run per gloss, on the same lemmas.
    assert!(senses.keys().eq(glossed.keys()), "senses.tsv's lemmas");
    let pack = Pack::load(fr_en()).unwrap();
    let lent: Vec<&String> = glossed
        .iter()
        .filter(|(lemma, gloss)| pack.gloss(lemma) != Some(gloss.as_str()))
        .map(|(lemma, _)| lemma)
        .take(10)
        .collect();
    assert!(
        lent.is_empty(),
        "a lemma answered with another row: {lent:?}"
    );
    let venir = pack.gloss("venir").expect("venir is glossed");
    assert!(venir.starts_with("to come"), "{venir}");
    assert!(!venir.contains("arrival"), "{venir}");
    assert!(!glossed.contains_key("venue"));
}

#[test]
fn spec_scenario_english_s_sizes() {
    // French's levels keep English's six sizes and go to French's dictionary words alone
    // (refine-lingua-fr-en-glosses D3, *A French level is given only to a French dictionary word*):
    // a lemma the dictionary words do not list — `parce`, met only in « parce que », `coran`, glossed
    // as a name alone — gives its slot to the next one. The checks name a levelled lemma that is no
    // word (*A level for a word with no gloss*).
    let levels = tsv(&tables().join("fr/level.tsv"));
    let lexical: BTreeSet<String> = std::fs::read_to_string(tables().join("fr/lexical.tsv"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    let unlisted: Vec<&String> = levels.keys().filter(|l| !lexical.contains(*l)).collect();
    assert!(
        unlisted.is_empty(),
        "levelled, no dictionary word of French: {unlisted:?}"
    );
    let mut sizes = BTreeMap::new();
    for level in levels.values() {
        *sizes.entry(level.as_str()).or_insert(0) += 1;
    }
    assert_eq!(
        sizes,
        BTreeMap::from([
            ("A1", 1020),
            ("A2", 1158),
            ("B1", 2015),
            ("B2", 2347),
            ("C1", 886),
            ("C2", 876)
        ])
    );
    for lemma in ["parce", "coran", "quant"] {
        assert!(!levels.contains_key(lemma), "{lemma} takes no level");
    }
    // Every levelled lemma carries a gloss, so a card seeded from a level shows one.
    let pack = Pack::load(fr_en()).unwrap();
    let bare: Vec<&String> = levels
        .keys()
        .filter(|l| pack.gloss(l).is_none())
        .take(10)
        .collect();
    assert!(bare.is_empty(), "levelled with no gloss: {bare:?}");
}

/// fr-es's committed pack, built once per test binary.
fn fr_es() -> &'static [u8] {
    static FR_ES: OnceLock<Vec<u8>> = OnceLock::new();
    FR_ES.get_or_init(|| {
        let inputs =
            inputs_from_tables(&tables(), "fr-es").unwrap_or_else(|e| panic!("fr-es: {e}"));
        build_pack(&inputs).unwrap_or_else(|e| panic!("build fr-es: {e}"))
    })
}

#[test]
fn spec_scenario_french_glossed_in_spanish() {
    // fr-es (add-lingua-pack-fr-es D1, D10), a reader pair of French: its folder holds exactly a
    // pair's file set, its manifest studies French glossed in Spanish with French's analyser and
    // estimated levels, and its studied side is tables/fr, which fr-en writes.
    let dir = tables().join("fr-es");
    assert_eq!(
        files(&dir),
        [
            "NOTICE",
            "README.md",
            "gloss.tsv",
            "manifest.json",
            "mwe.tsv",
            "pin.json",
            "senses.tsv"
        ]
    );
    let manifest = json(&dir.join("manifest.json"));
    assert_eq!(manifest["meta"]["studied"], "fr");
    assert_eq!(manifest["meta"]["native"], "es");
    assert_eq!(
        manifest["meta"]["analyzer_version"],
        lingua_core::analysis::FRENCH_ANALYZER_VERSION
    );
    assert_eq!(manifest["meta"]["levels_estimated"], true);

    // Its pin records its own pack and, a reader pair's record (add-lingua-pack-es-en D3), fr-en
    // as the reference and the sha256 of each of the six studied tables it was built on, as
    // committed; its pack_version names their digest.
    let pin = json(&dir.join("pin.json"));
    assert_eq!(pin["studied"]["reference"], "fr-en");
    let recorded = pin["studied"]["tables"].as_object().unwrap();
    assert_eq!(recorded.len(), 6, "{recorded:?}");
    for name in STUDIED_SIDE {
        let committed = std::fs::read(tables().join("fr").join(name)).unwrap();
        assert_eq!(
            recorded[name],
            sha256_hex(&committed).as_str(),
            "fr-es: fr/{name}"
        );
    }
    let version = manifest["meta"]["pack_version"].as_str().unwrap();
    assert!(
        version.starts_with(&format!("{}+", pin["snapshot"].as_str().unwrap()))
            && version.matches('.').count() == 3,
        "a reader pair's pack_version names its snapshot, rules and studied tables: {version}"
    );
    // Its sources are dumps alone, the three derived files under its own release (D3): the Spanish
    // edition's French section and French translations, the French edition's Spanish ones. No
    // English dump: tables/fr is read as committed.
    assert!(pin["sources"].get("kaikki").is_none(), "no extract");
    assert!(pin["sources"].get("kaikki-en").is_none(), "no English dump");
    let release = pin["sources"]["kaikki-es"]["release"].as_str().unwrap();
    assert!(
        release.starts_with("lingua-pack-sources-fr-es-"),
        "{release}"
    );
    assert_eq!(pin["sources"]["kaikki-fr"]["release"], release);
    assert!(pin["sources"]["kaikki-es"]["files"]["kaikki-es-Frances.jsonl"].is_object());
    assert!(pin["sources"]["kaikki-es"]["files"]["kaikki-es-traductions.jsonl"].is_object());
    assert!(pin["sources"]["kaikki-fr"]["files"]["kaikki-fr-traductions.jsonl"].is_object());
    assert!(
        json(&tables().join("fr-en/pin.json"))
            .get("studied")
            .is_none(),
        "the reference's pin records nothing of its readers"
    );
}

#[test]
fn spec_scenario_french_s_dictionary_words() {
    // fr-es's pack carries a lexical table listing exactly the lemmas tables/fr/lexical.tsv lists —
    // French's dictionary words, fr-en's glossed lemmas less its names alone
    // (refine-lingua-fr-en-glosses D2) — so a lemma fr-es glosses and fr-en does not is no dictionary
    // word (`quant`, glossed by the Spanish Wiktionary; *French's dictionary words*), nor one fr-en
    // glosses as a name alone.
    let bytes = fr_es();
    assert!(section_of(&sections(bytes), section::LEXICAL).is_some());
    let (fr_es, fr_en) = (Pack::load(bytes).unwrap(), Pack::load(fr_en()).unwrap());
    assert_eq!(fr_es.pair().key(), "fr-es");
    assert_eq!(
        fr_es.dictionary_words(),
        fr_en.dictionary_words(),
        "fr-es's dictionary words are fr-en's"
    );
    let lexical: BTreeSet<String> =
        std::fs::read_to_string(tables().join("fr").join("lexical.tsv"))
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect();
    let glossed = tsv(&tables().join("fr-es/gloss.tsv"));
    let only_here: Vec<&String> = glossed
        .keys()
        .filter(|lemma| !lexical.contains(*lemma))
        .collect();
    assert!(only_here.iter().any(|l| *l == "quant"), "quant");
    for lemma in &only_here {
        assert!(
            fr_es.gloss(lemma).is_some() && !fr_es.is_dictionary_word(lemma),
            "{lemma}"
        );
    }
    assert!(fr_en.gloss("quant").is_none());
    // Every row answers its own lemma (the builder files a gloss under the lemma its key reads
    // as), with a run per gloss.
    let senses = tsv(&tables().join("fr-es/senses.tsv"));
    assert!(senses.keys().eq(glossed.keys()), "senses.tsv's lemmas");
    let lent: Vec<&String> = glossed
        .iter()
        .filter(|(lemma, gloss)| fr_es.gloss(lemma) != Some(gloss.as_str()))
        .map(|(lemma, _)| lemma)
        .take(10)
        .collect();
    assert!(
        lent.is_empty(),
        "a lemma answered with another row: {lent:?}"
    );
    assert_eq!(fr_es.gloss("maison"), Some("Casa"));
    assert_eq!(fr_es.gloss("et"), Some("Y, e"));
}

#[test]
fn spec_scenario_the_pack_is_built_not_shipped() {
    // fr-es's pack, built from tables/fr/ and tables/fr-es/, has the sha256 its pin records, fits
    // the budget, and no package lists it: change 52 decides, by the floor (add-lingua-pack-fr-es
    // D8).
    let bytes = fr_es();
    let pin = json(&tables().join("fr-es/pin.json"));
    assert_eq!(
        pin["pack"]["sha256"],
        sha256_hex(bytes).as_str(),
        "fr-es: pin.json"
    );
    assert_eq!(pin["pack"]["size"], bytes.len());
    assert!(bytes.len() < MAX_PACK_BYTES);
    let packs = json(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/lingua-extension/packs.json"),
    );
    assert!(
        !packs["pairs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == "fr-es"),
        "no package lists fr-es"
    );
}

#[test]
fn spec_scenario_the_credits_of_fr_es() {
    // fr-es's notice credits both sides: French's tables and dictionary words as fr-en's notice
    // credits them (the English Wiktionary's French section, wordfreq, UD French-GSD), the Spanish
    // Wiktionary's definitions and French translations, the French Wiktionary's Spanish
    // translations, wordfreq for the Spanish words' order. The pack carries it.
    let notice = std::fs::read_to_string(tables().join("fr-es/NOTICE")).unwrap();
    let notice = notice.split_whitespace().collect::<Vec<_>>().join(" ");
    for credit in [
        "English Wiktionary (enwiktionary), French section",
        "which lemmas fr-en glosses: the dictionary words and which take a level",
        "Spanish Wiktionary (eswiktionary)",
        "French translations its Spanish entries list",
        "French Wiktionary (frwiktionary)",
        "Spanish translations its French entries list",
        "wordfreq (French and Spanish frequency lists)",
        "Robyn Speer",
        "UD French-GSD",
        "The levels are estimated, not taken from a CEFR list",
    ] {
        assert!(notice.contains(credit), "fr-es/NOTICE names {credit:?}");
    }
    let manifest = json(&tables().join("fr-es/manifest.json"));
    assert_eq!(
        manifest["meta"]["licences"][0],
        "kaikki / enwiktionary, eswiktionary, frwiktionary (CC BY-SA 4.0 + GFDL)"
    );
    let pack = Pack::load(fr_es()).unwrap();
    assert!(pack.meta().levels_estimated);
    for credit in [
        "enwiktionary",
        "eswiktionary",
        "frwiktionary",
        "UD French-GSD",
    ] {
        assert!(
            pack.notice().contains(credit),
            "the pack's notice names {credit:?}"
        );
    }
}

#[test]
fn spec_scenario_the_reduction_never_reads_the_treebanks_it_is_measured_on() {
    // fr-en's pin records the English Wiktionary's French section, derived from the edition's dump
    // under fr-en's own release, GSD's training and development sections at a commit, and wordfreq —
    // never GSD's test section nor PUD, which measure/fr-ud.sh holds out.
    let pin = json(&tables().join("fr-en/pin.json"));
    let sources: Vec<&String> = pin["sources"].as_object().unwrap().keys().collect();
    // serde_json's map is sorted by key.
    assert_eq!(sources, ["gsd-dev", "gsd-train", "kaikki-en", "wordfreq"]);
    for (name, file) in [
        ("gsd-train", "fr_gsd-ud-train.conllu"),
        ("gsd-dev", "fr_gsd-ud-dev.conllu"),
    ] {
        let url = pin["sources"][name]["url"].as_str().unwrap();
        assert!(
            url.starts_with(
                "https://raw.githubusercontent.com/UniversalDependencies/UD_French-GSD/"
            ) && url.ends_with(&format!("/{file}")),
            "{name}: {url}"
        );
    }
    let kaikki = &pin["sources"]["kaikki-en"];
    let release = kaikki["release"].as_str().unwrap();
    assert!(
        release.starts_with("lingua-pack-sources-fr-en-"),
        "{release}"
    );
    assert_eq!(
        kaikki["url"],
        "https://kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz"
    );
    let derived: Vec<&String> = kaikki["files"].as_object().unwrap().keys().collect();
    assert_eq!(derived, ["kaikki-French.jsonl"]);
    let text = pin.to_string();
    for held_out in ["fr_gsd-ud-test", "fr_pud", "UD_French-PUD"] {
        assert!(!text.contains(held_out), "fr-en's pin names {held_out}");
    }
}

// French's pinned tag pool (add-lingua-french-grammar-tables D8): the tags French's readings carry,
// each once, written by a person — no reducer writes it — and held to the readings here.

/// Where a pin and the readings' tags part: the canonical, distinct lines of `pin` against the
/// third column of `grammar` (`grammar.tsv`), each difference named.
fn pin_against_readings(pin: &str, grammar: &str) -> Result<(), String> {
    let mut pinned = BTreeSet::new();
    for line in pin.lines() {
        let canonical = Tag::parse_strict(line)
            .map_err(|e| format!("tags.tsv: {line:?} is no tag ({e})"))?
            .to_ud();
        if canonical != line {
            return Err(format!(
                "tags.tsv: {line:?} is not written as {canonical:?}"
            ));
        }
        if !pinned.insert(line) {
            return Err(format!("tags.tsv: {line:?} is listed twice"));
        }
    }
    let read: BTreeSet<&str> = grammar
        .lines()
        .map(|row| row.split('\t').nth(2).unwrap_or_else(|| panic!("{row:?}")))
        .collect();
    let missing: Vec<&&str> = read.difference(&pinned).collect();
    if !missing.is_empty() {
        return Err(format!(
            "a reading carries {missing:?}, which tags.tsv does not pin: append it, after the pinned tags"
        ));
    }
    let left: Vec<&&str> = pinned.difference(&read).collect();
    if !left.is_empty() {
        return Err(format!("tags.tsv pins {left:?}, which no reading carries"));
    }
    Ok(())
}

#[test]
fn spec_scenario_the_pin_is_the_readings_tags() {
    let fr = tables().join("fr");
    let pin = std::fs::read_to_string(fr.join("tags.tsv")).unwrap();
    let grammar = std::fs::read_to_string(fr.join("grammar.tsv")).unwrap();
    pin_against_readings(&pin, &grammar).unwrap_or_else(|e| panic!("fr/{e}"));
    // The first reduction's tags, in byte order: 79, as the design counted them.
    let lines: Vec<&str> = pin.lines().collect();
    assert_eq!(lines.len(), 79);
    assert!(lines.is_sorted(), "fr/tags.tsv is not in byte order");
    // The pack lays its pool out as the pin, then the tags only fr-en's sense runs carry
    // (add-lingua-pack-fr-en: `ADV`, a noun run's `NOUN|Gender=Fem`…), each once, sorted.
    let (_, sections) = read_container(fr_en()).unwrap();
    let pool = sections
        .iter()
        .find(|s| s.name == section::TAGS)
        .map(|s| String::from_utf8(s.data.clone()).unwrap())
        .expect("a tag pool");
    let pool: Vec<&str> = pool.split('\n').collect();
    assert_eq!(pool[..lines.len()], lines[..]);
    let senses = &pool[lines.len()..];
    assert!(senses.contains(&"ADV"), "{senses:?}");
    assert!(senses.is_sorted(), "{senses:?}");
    assert!(senses.iter().all(|tag| !lines.contains(tag)), "{senses:?}");
}

#[test]
fn spec_scenario_a_later_reduction_carries_a_new_tag() {
    let pin = "NOUN|Gender=Fem|Number=Sing\nVERB|VerbForm=Inf\n";
    let grammar =
        "maison\tmaison\tNOUN|Gender=Fem|Number=Sing\t-\nparler\tparler\tVERB|VerbForm=Inf\t-\n";
    assert_eq!(pin_against_readings(pin, grammar), Ok(()));
    // A reading's new tag fails, named, until it is appended after the pinned ones.
    let newer = format!("{grammar}parlant\tparler\tVERB|Tense=Pres|VerbForm=Part\t-\n");
    let err = pin_against_readings(pin, &newer).unwrap_err();
    assert!(err.contains("VERB|Tense=Pres|VerbForm=Part"), "{err}");
    let appended = format!("{pin}VERB|Tense=Pres|VerbForm=Part\n");
    assert_eq!(pin_against_readings(&appended, &newer), Ok(()));
    // A pinned tag no reading carries any more fails, named.
    let err = pin_against_readings(&appended, grammar).unwrap_err();
    assert!(err.contains("which no reading carries"), "{err}");
    assert!(err.contains("VERB|Tense=Pres|VerbForm=Part"), "{err}");
    // A line not written as the vocabulary writes it, or listed twice, fails, named.
    let err = pin_against_readings("NOUN|Number=Sing|Gender=Fem\n", "").unwrap_err();
    assert!(err.contains("is not written as"), "{err}");
    let err = pin_against_readings("VERB|VerbForm=Inf\nVERB|VerbForm=Inf\n", "").unwrap_err();
    assert!(err.contains("listed twice"), "{err}");
    assert!(pin_against_readings("Gender=Fem\n", "").is_err());
}

#[test]
fn spec_scenario_senses_another_pack_s_glosses_carry() {
    // A pack studying French, its glosses carrying a sense run tagged INTJ, a tag no reading
    // carries: its readings are stored byte for byte as without it, the pool only grows at its end,
    // and the core reads the run as INTJ. French's studied tables, fr-en's own native side left
    // out — its runs carry INTJ already (add-lingua-pack-fr-en).
    let studied_only = || {
        let mut inputs = inputs_from_tables(&tables(), "fr-en").expect("fr-en");
        inputs.glosses.clear();
        inputs.senses.clear();
        inputs
    };
    let build = |runs: bool| {
        let mut inputs = studied_only();
        inputs.glosses.push(("ah".into(), "Ah!".into()));
        if runs {
            inputs.senses.push(("ah".into(), vec![("INTJ".into(), 1)]));
        }
        build_pack(&inputs).expect("build fr-en")
    };
    let (without, with) = (build(false), build(true));
    let (a, b) = (sections(&without), sections(&with));
    assert!(section_of(&a, section::PARADIGMS_ZST).is_some());
    assert!(
        section_of(&a, section::PARADIGMS_ZST) == section_of(&b, section::PARADIGMS_ZST),
        "the INTJ run moved the readings"
    );
    let pool = |s: &[(String, Vec<u8>)]| -> Vec<String> {
        String::from_utf8(section_of(s, section::TAGS).unwrap().to_vec())
            .unwrap()
            .split('\n')
            .map(str::to_owned)
            .collect()
    };
    let (pool_a, pool_b) = (pool(&a), pool(&b));
    assert_eq!(pool_b[..pool_a.len()], pool_a[..]);
    assert_eq!(pool_b[pool_a.len()..], ["INTJ"]);
    let pack = Pack::load(&with).unwrap();
    let runs: Vec<String> = pack
        .sense_runs("ah")
        .into_iter()
        .map(|(tag, _)| tag.unwrap().to_ud())
        .collect();
    assert_eq!(runs, ["INTJ"]);
    // Built without a pin — one sorted pool —, INTJ falls among the readings' tags and moves them.
    let mut unpinned = studied_only();
    unpinned.glosses.push(("ah".into(), "Ah!".into()));
    unpinned
        .senses
        .push(("ah".into(), vec![("INTJ".into(), 1)]));
    unpinned.tag_pool = None;
    let moved = sections(&build_pack(&unpinned).unwrap());
    assert!(section_of(&moved, section::PARADIGMS_ZST) != section_of(&a, section::PARADIGMS_ZST));
}

#[test]
fn the_pin_moves_no_byte_of_today_s_pack() {
    // The readings' tags in byte order are what the empty pin laid out too: fr-en built with an
    // empty pin is the pack its pin records (design D8).
    let mut empty = inputs_from_tables(&tables(), "fr-en").expect("fr-en");
    assert_eq!(empty.tag_pool.as_ref().map(Vec::len), Some(79));
    empty.tag_pool = Some(vec![]);
    assert!(
        build_pack(&empty).unwrap() == fr_en(),
        "the pin moved fr-en's bytes"
    );
}

// French's pre-pass (add-lingua-french-tokenisation D3–D5): the tables must hold every word it
// hands the lookup (add-lingua-french-forms-tables D4). Its lists are read from lingua-core, and the
// words it writes from its own tokeniser, so a pull request that moves them moves this check.

/// The words French's pre-pass writes, read by lingua-core's tokeniser with fr-en's lexicon: each
/// elided piece before a word, and in its special cases (`s'il`, `donne-m'en`, `va-t'en`); `au` and
/// `aux`; and an inversion with each of its pronouns, the euphonic `t` too. The words they are
/// written beside (`avoir`, `dit`, …) left out.
fn pre_pass_words(lexicon: &(impl Lexicon + ?Sized)) -> BTreeSet<String> {
    let mut text: Vec<String> = FRENCH_ELISIONS
        .iter()
        .map(|(written, _)| format!("{written}'avoir"))
        .collect();
    text.extend(["s'il", "donne-m'en", "va-t'en", "au", "aux", "a-t-il"].map(String::from));
    text.extend(FRENCH_INVERSION_PRONOUNS.iter().map(|p| format!("dit-{p}")));
    tokenize(&text.join(" "), StudiedLanguage::French, lexicon)
        .into_iter()
        .map(|token| token.text.to_lowercase())
        .filter(|word| !["avoir", "donne", "va", "a", "dit"].contains(&word.as_str()))
        .collect()
}

#[test]
fn spec_scenario_every_word_the_pre_pass_writes_is_a_form() {
    let forms = tsv(&tables().join("fr/forms.tsv"));
    let ranks = tsv(&tables().join("fr/freq.tsv"));
    let pack = Pack::load(fr_en()).unwrap();
    let written = pre_pass_words(pack.lexicon());
    // The 17 words of the elisions, `à`, `le`, `les` and the inversion's pronouns: 32, as the
    // design counted them — a word the tokeniser dropped would be missing here.
    assert_eq!(written.len(), 32, "{written:?}");
    for word in &written {
        let lemma = forms
            .get(word)
            .unwrap_or_else(|| panic!("{word}: the pre-pass writes it, and fr/forms.tsv lacks it"));
        assert!(
            ranks.contains_key(lemma),
            "{word} → {lemma}: no ranked lemma"
        );
    }
    // Each elided piece maps to the word the pre-pass reads it as outside its special cases.
    for (written, read) in FRENCH_ELISIONS {
        let piece = format!("{written}'");
        assert_eq!(
            forms.get(&piece).map(String::as_str),
            Some(*read),
            "{piece}"
        );
    }
}

#[test]
fn spec_scenario_the_contracted_articles() {
    // `au` and `aux`, which the pre-pass always splits, are neither forms nor ranks; `du` and `des`,
    // which it keeps whole, are words of their own.
    let forms = tsv(&tables().join("fr/forms.tsv"));
    let ranks = tsv(&tables().join("fr/freq.tsv"));
    for contraction in ["au", "aux"] {
        assert!(!forms.contains_key(contraction), "{contraction} is a form");
        assert!(!ranks.contains_key(contraction), "{contraction} is ranked");
    }
    for word in ["du", "des"] {
        assert_eq!(forms.get(word).map(String::as_str), Some(word));
        assert!(ranks.contains_key(word), "{word} is no ranked lemma");
    }
    // A noun ending in a pronoun is listed whole, so the inversion rule never splits it; a verb and
    // its pronoun is not, and no plain word beginning with a piece is a form.
    for whole in [
        "rendez-vous",
        "qu'en-dira-t-on",
        "c'est-à-dire",
        "aujourd'hui",
    ] {
        assert!(forms.contains_key(whole), "{whole} is no form");
    }
    for split in [
        "est-il",
        "allez-y",
        "souviens-toi",
        "c'est",
        "d'abord",
        "l'on",
        "jusqu'à",
    ] {
        assert!(!forms.contains_key(split), "{split} is a form");
    }
}

#[test]
fn spec_scenario_the_committed_pairs() {
    // fix-lingua-lemma-lookup D4, D6: each committed pair builds — no form of its forms table has
    // two lemmas, every lemma of its pack reads as itself — to the sha256 its pin records, so
    // filing every section at the lemma's own place moved no byte.
    for (pair, bytes) in [
        ("en-fr", shipped("en-fr")),
        ("es-fr", shipped("es-fr")),
        ("es-en", es_en()),
        ("en-es", en_es()),
        ("fr-en", fr_en()),
        ("fr-es", fr_es()),
    ] {
        let pin = json(&tables().join(pair).join("pin.json"));
        assert_eq!(pin["pack"]["sha256"], sha256_hex(bytes).as_str(), "{pair}");
        let pack = Pack::load(bytes).unwrap();
        let sections = sections(bytes);
        let pool = std::str::from_utf8(section_of(&sections, section::LEMMAS).unwrap()).unwrap();
        let elsewhere: Vec<(&str, Option<&str>)> = pool
            .lines()
            .map(|lemma| (lemma, pack.lexicon().lemma_of(lemma)))
            .filter(|(lemma, read)| *read != Some(*lemma))
            .take(10)
            .collect();
        assert!(
            elsewhere.is_empty(),
            "{pair}: lemmas read as another: {elsewhere:?}"
        );
    }
}

#[test]
fn every_rank_lands_on_its_own_lemma_in_the_built_pack() {
    // A ranked lemma whose own form reads as another word kept no rank of its own and lent it to
    // that word while the builder filed a rank through the form lookup
    // (add-lingua-french-forms-tables: `donnée`, read as donner, gave donner its 1,711). The
    // builder now files it at the lemma's own place and refuses such a lemma
    // (fix-lingua-lemma-lookup D4); this holds the result on the packs as built: every committed
    // pair's pack holds each rank of its studied language's freq.tsv on that very lemma, and no
    // other.
    for (pair, bytes) in [
        ("en-fr", shipped("en-fr")),
        ("es-fr", shipped("es-fr")),
        ("es-en", es_en()),
        ("en-es", en_es()),
        ("fr-en", fr_en()),
    ] {
        let pack = Pack::load(bytes).unwrap();
        let ranks: BTreeMap<String, u32> = tsv(&tables().join(studied_of(pair)).join("freq.tsv"))
            .into_iter()
            .map(|(lemma, rank)| (lemma, rank.parse().unwrap()))
            .collect();
        let elsewhere: Vec<(&String, Option<&str>)> = ranks
            .keys()
            .map(|lemma| (lemma, pack.lexicon().lemma_of(lemma)))
            .filter(|(lemma, read)| *read != Some(lemma.as_str()))
            .take(10)
            .collect();
        assert!(
            elsewhere.is_empty(),
            "{pair}: ranked lemmas whose own form reads elsewhere: {elsewhere:?}"
        );
        // Band by band, the pack's ranked lemmas are exactly freq.tsv's.
        let top = ranks.values().copied().max().unwrap();
        for lo in (1..=top).step_by(1000) {
            let hi = lo + 999;
            let got: BTreeSet<&str> = pack
                .lemmas_in_rank_band(lo, hi)
                .into_iter()
                .map(|(lemma, _)| lemma)
                .collect();
            let want: BTreeSet<&str> = ranks
                .iter()
                .filter(|(_, rank)| (lo..=hi).contains(*rank))
                .map(|(lemma, _)| lemma.as_str())
                .collect();
            assert_eq!(got, want, "{pair}: ranks {lo}–{hi}");
        }
    }
}

// French's estimated levels (add-lingua-french-levels, M7): fr-en's reduction writes
// `tables/fr/level.tsv`, English's level sizes given to French's commonest lemmas in rank order,
// which lemmas take one read from the English Wiktionary's French section.

/// English's CEFR level sizes, which French's estimated levels borrow, as Spanish's do
/// (`ENGLISH_BANDS` in reduce-fr-en.py and reduce-es-fr.py).
const ENGLISH_LEVEL_SIZES: [(&str, usize); 6] = [
    ("A1", 1_020),
    ("A2", 1_158),
    ("B1", 2_015),
    ("B2", 2_347),
    ("C1", 886),
    ("C2", 876),
];

#[test]
fn spec_scenario_six_levels_of_english_s_sizes() {
    let fr = tables().join("fr");
    let levels = tsv(&fr.join("level.tsv"));
    let ranks = tsv(&fr.join("freq.tsv"));
    let forms = tsv(&fr.join("forms.tsv"));
    assert_eq!(levels.len(), 8_302);
    // Each a ranked lemma whose own form reads as itself: a pack finds a lemma by its own form.
    for lemma in levels.keys() {
        assert!(ranks.contains_key(lemma), "{lemma}: levelled, not ranked");
        assert_eq!(
            forms.get(lemma),
            Some(lemma),
            "{lemma}: its own form reads elsewhere"
        );
    }
    // As many at each level as English's lists hold, given in rank order: each level's ranks come
    // after the level below's.
    let mut below = 0;
    for (label, size) in ENGLISH_LEVEL_SIZES {
        let at: Vec<u32> = levels
            .iter()
            .filter(|(_, level)| *level == label)
            .map(|(lemma, _)| ranks[lemma].parse().unwrap())
            .collect();
        assert_eq!(at.len(), size, "fr/level.tsv: {label}");
        let (first, last) = (*at.iter().min().unwrap(), *at.iter().max().unwrap());
        assert!(
            first > below,
            "{label} starts at rank {first}, the level below ends at {below}"
        );
        below = last;
    }
}

#[test]
fn spec_scenario_the_commonest_words_are_a1() {
    let levels = tsv(&tables().join("fr/level.tsv"));
    let ranks = tsv(&tables().join("fr/freq.tsv"));
    for word in ["de", "le", "et", "à", "être", "avoir", "du", "des", "y"] {
        assert_eq!(levels.get(word).map(String::as_str), Some("A1"), "{word}");
    }
    // `au` and `aux` are no words of French's tables.
    for word in ["au", "aux"] {
        assert!(!levels.contains_key(word), "{word} has a level");
    }
    // What a CEFR list leaves out (design D2), all ranked: a name, a word the section does not
    // know, a letter, another word's spelling.
    for word in ["paris", "france", "the", "b", "e", "etre"] {
        assert!(ranks.contains_key(word), "{word} is no ranked lemma");
        assert!(!levels.contains_key(word), "{word} has a level");
    }
}

#[test]
fn spec_scenario_every_french_pack_says_so() {
    // Every pack studying French built from the committed tables — fr-en today — says its levels
    // are estimated, in its manifest and its pack, and its NOTICE says so, citing no CEFR list.
    let pairs = check_committed_tables(&tables()).unwrap_or_else(|e| panic!("{e}"));
    let french: Vec<&String> = pairs.iter().filter(|p| studied_of(p) == "fr").collect();
    assert!(french.iter().any(|p| *p == "fr-en"), "{french:?}");
    for pair in french {
        let dir = tables().join(pair);
        assert_eq!(
            json(&dir.join("manifest.json"))["meta"]["levels_estimated"],
            true,
            "{pair}/manifest.json"
        );
        let pack = Pack::load(&committed_pack(pair)).unwrap();
        assert!(pack.has_levels(), "{pair}");
        assert!(pack.levels_estimated(), "{pair}");
        let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
        assert!(
            notice.contains("The levels are estimated, not taken from a CEFR list"),
            "{pair}/NOTICE"
        );
        for list in ["FLELex", "CEFRLex", "CEFR-J", "Octanove"] {
            assert!(!notice.contains(list), "{pair}/NOTICE names {list}");
        }
    }
}

/// A committed pair's pack, built once per test binary for the six pairs the tests above build.
fn committed_pack(pair: &str) -> std::borrow::Cow<'static, [u8]> {
    use std::borrow::Cow;
    match pair {
        "en-fr" | "es-fr" => Cow::Borrowed(shipped(pair)),
        "es-en" => Cow::Borrowed(es_en()),
        "en-es" => Cow::Borrowed(en_es()),
        "fr-en" => Cow::Borrowed(fr_en()),
        "fr-es" => Cow::Borrowed(fr_es()),
        _ => Cow::Owned(
            build_pack(&inputs_from_tables(&tables(), pair).unwrap_or_else(|e| panic!("{e}")))
                .unwrap_or_else(|e| panic!("build {pair}: {e}")),
        ),
    }
}

/// A studied language's level table, `lemma → level`; empty when it has none.
fn level_table(studied: &Path) -> BTreeMap<String, String> {
    let path = studied.join("level.tsv");
    if path.exists() {
        tsv(&path)
    } else {
        BTreeMap::new()
    }
}

/// Each level of a studied language's `level.tsv` on the lemma it is written for, in a pair's
/// pack (add-lingua-french-levels D3). The builder keyed a level by looking the lemma up as a form
/// (`FstLexicon::id_of`), so a lemma whose own form reads as another handed that other lemma its
/// level; it now files a level at the lemma's own place and a level written for a string that is
/// no lemma under none (fix-lingua-lemma-lookup D4), which this check sees as a level the pack
/// does not carry. Every lemma of the table must carry the table's level in the pack, and every
/// lemma the pack gives a level the table's — the second half catches a level landing on a lemma
/// the table leaves without one. The levels carried, or the pair and each lemma off, with both
/// levels.
fn levels_on_their_lemmas(
    pair: &str,
    bytes: &[u8],
    table: &BTreeMap<String, String>,
) -> Result<usize, String> {
    let pack = Pack::load(bytes).map_err(|e| format!("{pair}: {e}"))?;
    let mut carried: BTreeMap<&str, &str> = BTreeMap::new();
    for level in CefrLevel::ALL {
        for (lemma, _) in pack.lemmas_at_level(level) {
            carried.insert(lemma, level.label());
        }
    }
    let lemmas: BTreeSet<&str> = table
        .keys()
        .map(String::as_str)
        .chain(carried.keys().copied())
        .collect();
    let off: Vec<String> = lemmas
        .into_iter()
        .filter_map(|lemma| {
            let written = table.get(lemma).map(String::as_str);
            let landed = carried.get(lemma).copied();
            (written != landed).then(|| {
                format!(
                    "{lemma} (table {}, pack {})",
                    written.unwrap_or("none"),
                    landed.unwrap_or("none")
                )
            })
        })
        .collect();
    if off.is_empty() {
        Ok(carried.len())
    } else {
        Err(format!(
            "{pair}: {} level(s) not on the lemma they are written for: {}",
            off.len(),
            off.join(", ")
        ))
    }
}

#[test]
fn spec_scenario_the_committed_pairs_give_each_level_to_its_own_lemma() {
    // Every committed pair's pack gives every lemma of its studied language's level.tsv exactly
    // that level, and no other lemma one: 8,302 levels each.
    let pairs = check_committed_tables(&tables()).unwrap_or_else(|e| panic!("{e}"));
    for pair in &pairs {
        let table = level_table(&tables().join(studied_of(pair)));
        let carried = levels_on_their_lemmas(pair, &committed_pack(pair), &table)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(carried, table.len(), "{pair}");
    }
    for pair in ["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"] {
        assert_eq!(
            level_table(&tables().join(studied_of(pair))).len(),
            8_302,
            "{pair}"
        );
    }
}

#[test]
fn spec_scenario_a_level_written_for_a_lemma_whose_form_reads_as_another() {
    // French's forms table maps the form `donnée` to *donner* (change 43: the noun leaves the
    // pack). A level table giving `donnée` B1 and `donner` A1 — sorted, as a reducer writes it —
    // builds a pack in which donner keeps its A1 and the level written for `donnée`, no lemma of
    // the pack, lands on no lemma (fix-lingua-lemma-lookup D4; the builder looked `donnée` up as a
    // form and gave donner its B1): the check fails, naming the pair and donnée.
    assert_eq!(
        tsv(&tables().join("fr/forms.tsv"))
            .get("donnée")
            .map(String::as_str),
        Some("donner")
    );
    let scratch = std::env::temp_dir().join(format!("lingua-levels-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let fr = studied_copy(&scratch, "fr");
    std::fs::write(fr.join("level.tsv"), "donner\tA1\ndonnée\tB1\n").unwrap();
    let inputs = inputs_from_dirs(&fr, &tables().join("fr-en")).unwrap_or_else(|e| panic!("{e}"));
    let bytes = build_pack(&inputs).unwrap_or_else(|e| panic!("build fr-en: {e}"));
    let table = level_table(&fr);
    std::fs::remove_dir_all(&scratch).unwrap();
    let pack = Pack::load(&bytes).unwrap();
    assert!(!pack.lexicon().contains_lemma("donnée"));
    let at = |level: CefrLevel| -> Vec<&str> {
        pack.lemmas_at_level(level)
            .into_iter()
            .map(|(lemma, _)| lemma)
            .filter(|lemma| lemma.starts_with("donn"))
            .collect()
    };
    assert_eq!(at(CefrLevel::A1), ["donner"]);
    assert!(at(CefrLevel::B1).is_empty());
    let failure = levels_on_their_lemmas("fr-en", &bytes, &table).unwrap_err();
    assert_eq!(
        failure,
        "fr-en: 1 level(s) not on the lemma they are written for: donnée (table B1, pack none)"
    );
}

// The French phrase gloss holds an expression's elided pieces to the page's
// (match-lingua-french-elided-pieces): it reads them from the expression's name, which only a pack
// built from these tables can show it carries.

#[test]
fn every_french_headword_holding_an_elided_piece_is_named() {
    // D3: a key no longer says how its pieces were written (`de l'` is `de le`), so the matcher
    // reads them from the headword that won the key, which the pack names wherever the two
    // differ. Every winning headword holding an elided piece is named, and reads as many pieces
    // as its key, so the check lines it up with a run; a key the pack does not name is its own
    // headword, written without an elided piece. Measured on the tables of the proposal: fr-en
    // 1,541 such headwords, fr-es 869.
    for (pair, bytes, among) in [
        ("fr-en", fr_en(), ["de l'", "d'abord", "c'est"].as_slice()),
        ("fr-es", fr_es(), ["coup d'œil", "qu'est-ce que"].as_slice()),
    ] {
        let inputs = inputs_from_tables(&tables(), pair).unwrap_or_else(|e| panic!("{e}"));
        let pack = Pack::load(bytes).unwrap();
        let lexicon = pack.lexicon();
        let keys: BTreeSet<String> = inputs
            .expressions
            .iter()
            .filter_map(|(headword, _)| french_expression_key(headword, lexicon))
            .filter(|key| pack.expression(key).is_some())
            .collect();
        let mut elided = BTreeSet::new();
        for key in &keys {
            let name = pack.expression_name(key);
            let headword = name.unwrap_or(key);
            let reading = headword_reading(headword, StudiedLanguage::French, lexicon)
                .unwrap_or_else(|| panic!("{pair}: {headword} has a key and no reading"));
            if reading.iter().any(|(token, _)| is_elided(headword, token)) {
                assert_eq!(name, Some(headword), "{pair}: {key} is not named");
                elided.insert(headword.to_owned());
            }
            assert_eq!(
                reading.len(),
                key.split(' ').count(),
                "{pair}: {headword} does not read as its key {key}"
            );
        }
        for headword in among {
            assert!(elided.contains(*headword), "{pair}: {headword}");
        }
        assert!(elided.len() > 500, "{pair}: {}", elided.len());
    }
}
