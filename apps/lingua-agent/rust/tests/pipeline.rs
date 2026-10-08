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

//! The ingest → statusline → /vocab pipeline over the committed fixture packs (English, and
//! Spanish for add-lingua-agent-languages) and synthetic Claude Code transcripts.

use std::io::Write;

use lingua_agent::engine::{Library, pack_path, prose_lines};
use lingua_agent::ingest::{ingest_texts, run_ingest};
use lingua_agent::source::{ClaudeCodeSource, extract_assistant_texts};
use lingua_agent::statusline::statusline_text;
use lingua_agent::store::Store;
use lingua_agent::vocab::{
    VocabWord, add_to_deck, language_name, listing, skipped_notice, vocab_words,
};
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::knowledge::profile::NativeLanguage;
use lingua_core::packs::{Pack, PackMeta, read_container, write_container};

const EN: StudiedLanguage = StudiedLanguage::English;
const ES: StudiedLanguage = StudiedLanguage::Spanish;

const PACK_BYTES: &[u8] = include_bytes!("fixtures/pack.lingua");
/// Built by `scripts/lingua-data/build.sh --testdata es-fr`.
const ES_PACK_BYTES: &[u8] = include_bytes!("fixtures/es-fr.lingua");

/// English text over the fixture vocabulary (run/city known below 3000; seldom/conundrum unknown).
const REPLY: &str = "The runner runs through the city every morning and they seldom face a strange conundrum in the code.";

/// Spanish over the fixture vocabulary (casa known below 400; hablar and vino unknown), with a
/// block of code that must not vote.
const SPANISH: &str = "Hablaba de la casa y el vino. Hablo de la casa, es el vino de la casa.\n```\nlet city = run(city);\n```";

fn pack() -> Pack {
    Pack::load(PACK_BYTES).expect("fixture pack loads")
}

/// `bytes` with its metadata rewritten to name `native` as the language of its glosses.
fn glossed_in(bytes: &[u8], native: &str) -> Vec<u8> {
    let (meta, sections) = read_container(bytes).expect("a container");
    let mut meta: PackMeta = serde_json::from_slice(&meta).expect("metadata");
    meta.native = native.into();
    let sections: Vec<(&str, &[u8])> = sections
        .iter()
        .map(|s| (s.name.as_str(), s.data.as_slice()))
        .collect();
    write_container(&serde_json::to_vec(&meta).expect("json"), &sections)
}

/// The Spanish fixture, glossed in English.
fn es_en_bytes() -> Vec<u8> {
    glossed_in(ES_PACK_BYTES, "en")
}

/// A fresh directory holding `files`, and their paths in file-name order, as `installed` reads
/// them.
fn installed(name: &str, files: &[(&str, &[u8])]) -> (std::path::PathBuf, Vec<std::path::PathBuf>) {
    let dir = std::env::temp_dir().join(format!("lingua-agent-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    for (file, bytes) in files {
        std::fs::write(dir.join(file), bytes).unwrap();
    }
    let mut paths: Vec<_> = files.iter().map(|(file, _)| dir.join(file)).collect();
    paths.sort();
    (dir, paths)
}

fn english() -> Library {
    Library::from_packs([pack()])
}

/// English and Spanish followed.
fn both() -> Library {
    Library::from_packs([
        Pack::load(ES_PACK_BYTES).expect("Spanish fixture loads"),
        pack(),
    ])
}

fn calibrated_store() -> Store {
    let store = Store::open_in_memory().unwrap();
    store.set_calibration(EN, 3000).unwrap();
    store.set_calibration(ES, 400).unwrap();
    store
}

#[test]
fn extract_assistant_texts_handles_claude_code_shapes() {
    let jsonl = concat!(
        r#"{"type":"user","message":{"role":"user","content":"hello"}}"#,
        "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"first reply"},{"type":"tool_use","name":"x"}]}}"#,
        "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":"second reply"}}"#,
        "\n",
        "not json at all",
        "\n",
    );
    let texts = extract_assistant_texts(jsonl);
    assert_eq!(
        texts,
        vec!["first reply".to_string(), "second reply".to_string()]
    );
}

#[test]
fn ingest_counts_lemmas_but_no_transcript_content() {
    let store = calibrated_store();
    let counted = ingest_texts(
        &store,
        &mut english(),
        &[REPLY.to_string()],
        "claude-code",
        1000,
    )
    .unwrap();
    assert!(counted > 0);
    // Exposures recorded for the fixture vocabulary.
    assert!(store.exposure(EN, "run").unwrap() >= 1);
    assert!(store.exposure(EN, "city").unwrap() >= 1);
    assert!(store.exposure(EN, "seldom").unwrap() >= 1);
    // The store holds only lemmas — never a sentence from the transcript.
    assert_eq!(store.deck_len(EN).unwrap(), 0);
}

#[test]
fn run_ingest_is_idempotent_on_the_transcript_offset() {
    let dir = std::env::temp_dir().join(format!("lingua-agent-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("transcript.jsonl");
    let line = format!(
        "{{\"type\":\"assistant\",\"message\":{{\"role\":\"assistant\",\"content\":\"{REPLY}\"}}}}\n"
    );
    std::fs::write(&path, &line).unwrap();

    let store = calibrated_store();
    let mut library = english();
    let source = ClaudeCodeSource { path: path.clone() };
    let first = run_ingest(&store, &mut library, &source, "t1", 1000).unwrap();
    assert!(first > 0);
    let before = store.exposure(EN, "seldom").unwrap();

    // Re-running over the same (unchanged) transcript ingests nothing new.
    let second = run_ingest(&store, &mut library, &source, "t1", 2000).unwrap();
    assert_eq!(second, 0);
    assert_eq!(store.exposure(EN, "seldom").unwrap(), before);

    // Appending a new turn ingests only the new turn.
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    f.write_all(line.as_bytes()).unwrap();
    let third = run_ingest(&store, &mut library, &source, "t1", 3000).unwrap();
    assert!(third > 0);
    assert_eq!(store.exposure(EN, "seldom").unwrap(), before + 1);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn statusline_reports_percentage_and_new_words() {
    let store = calibrated_store();
    let ks = store.knowledge_state().unwrap();
    let line = statusline_text(&mut english(), &ks, REPLY).expect("analysable");
    assert!(line.starts_with("📖 "));
    assert!(
        !line.starts_with("📖 EN"),
        "one language names none: {line}"
    );
    assert!(line.contains("%"));
    assert!(line.contains("nouveau"), "line was: {line}");
}

#[test]
fn statusline_is_mute_on_non_analysable_input() {
    let store = calibrated_store();
    let ks = store.knowledge_state().unwrap();
    assert!(statusline_text(&mut english(), &ks, "hi").is_none());
    assert!(statusline_text(&mut Library::from_packs([]), &ks, REPLY).is_none());
}

#[test]
fn vocab_lists_unknown_words_and_add_creates_cards_with_the_sentence() {
    let store = calibrated_store();
    let ks = store.knowledge_state().unwrap();
    let words = vocab_words(&mut english(), &ks, &[REPLY.to_string()], |_| 3000);
    let lemmas: Vec<&str> = words.iter().map(|w| w.lemma.as_str()).collect();
    assert!(lemmas.contains(&"seldom"));
    assert!(lemmas.contains(&"conundrum"));
    // seldom carries a gloss from the pack and the source sentence.
    let seldom = words.iter().find(|w| w.lemma == "seldom").unwrap();
    assert_eq!(seldom.gloss.as_deref(), Some("rarement"));
    assert!(seldom.sentence.contains("seldom"));

    assert!(seldom.rarity.contains("3000"));
    // One language: no heading.
    let text = listing(&words, false);
    assert!(text.contains("• seldom — rarement"));
    assert!(!text.contains("Anglais"));

    let added = add_to_deck(
        &store,
        &words,
        &["seldom".into(), "conundrum".into()],
        None,
        NativeLanguage::French,
        500,
    )
    .unwrap();
    assert_eq!(added.added, 2);
    assert!(added.ambiguous.is_empty());
    let card = store.card(EN, "seldom").unwrap().unwrap();
    assert!(card.provenance.sentence.contains("seldom"));
    assert_eq!(store.deck_len(EN).unwrap(), 2);
}

#[test]
fn engine_path_resolution_honours_env_and_defaults() {
    // The only test that touches these process-global env vars — kept in one function so
    // it never races another test's env access. Exercises every path branch.
    use lingua_agent::engine::lingua_home;

    // SAFETY: single-threaded within this test; no other test reads LINGUA_HOME/PACK.
    unsafe {
        std::env::remove_var("LINGUA_HOME");
        std::env::remove_var("LINGUA_PACK");
    }
    // Defaults: home ends in `.lingua`, pack lives under it.
    assert!(lingua_home().ends_with(".lingua"));
    assert_eq!(pack_path(), lingua_home().join("pack.lingua"));

    let dir = std::env::temp_dir().join(format!("lingua-agent-env-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    unsafe { std::env::set_var("LINGUA_HOME", &dir) };
    assert_eq!(lingua_home(), dir);
    assert_eq!(pack_path(), dir.join("pack.lingua"));

    // The packs installed are the languages followed (add-lingua-agent-languages D1): every
    // `*.lingua` file of the data directory, a file that is not a pack skipped.
    assert!(Library::installed().is_empty());
    std::fs::write(dir.join("pack.lingua"), PACK_BYTES).unwrap();
    std::fs::write(dir.join("es-fr.lingua"), ES_PACK_BYTES).unwrap();
    std::fs::write(dir.join("broken.lingua"), b"not a pack").unwrap();
    std::fs::write(dir.join("notes.txt"), PACK_BYTES).unwrap();
    let mut library = Library::installed();
    assert_eq!(library.languages(), vec![EN, ES]);
    assert!(library.several());
    assert_eq!(library.pack(ES).map(|p| p.studied()), Some(ES));
    assert!(
        library.skipped().is_empty(),
        "today's install skips nothing"
    );

    // A pack glossed in another native language is skipped, though it sorts first
    // (generalise-lingua-native-language D9).
    std::fs::write(dir.join("es-en.lingua"), es_en_bytes()).unwrap();
    let mut library = Library::installed();
    assert_eq!(library.languages(), vec![EN, ES]);
    assert_eq!(library.native(), Some(NativeLanguage::French));
    assert_eq!(
        library.pack(ES).map(|p| p.native()),
        Some(NativeLanguage::French)
    );
    assert_eq!(
        library
            .skipped()
            .iter()
            .map(|s| s.file.clone())
            .collect::<Vec<_>>(),
        vec![dir.join("es-en.lingua")]
    );

    // LINGUA_PACK overrides everything: that pack alone is followed.
    let pack = dir.join("custom.lingua");
    std::fs::write(&pack, PACK_BYTES).unwrap();
    unsafe { std::env::set_var("LINGUA_PACK", &pack) };
    assert_eq!(pack_path(), pack);
    let mut library = Library::installed();
    assert_eq!(library.languages(), vec![EN]);
    assert!(library.pack(EN).is_some());
    assert!(library.pack(ES).is_none());
    assert!(library.skipped().is_empty());

    // Whatever it is glossed in: its native language is the plugin's.
    let es_en = dir.join("custom-es-en.lingua");
    std::fs::write(&es_en, es_en_bytes()).unwrap();
    unsafe { std::env::set_var("LINGUA_PACK", &es_en) };
    let mut library = Library::installed();
    assert_eq!(library.languages(), vec![ES]);
    assert_eq!(library.native(), Some(NativeLanguage::English));
    assert!(library.pack(ES).is_some());
    assert!(library.skipped().is_empty());

    unsafe {
        std::env::remove_var("LINGUA_HOME");
        std::env::remove_var("LINGUA_PACK");
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn spec_scenario_a_pack_of_another_native_language_is_skipped_and_named() {
    let es_en = es_en_bytes();
    let (dir, files) = installed(
        "other-native",
        &[
            ("pack.lingua", PACK_BYTES),
            ("es-fr.lingua", ES_PACK_BYTES),
            ("es-en.lingua", &es_en),
        ],
    );
    let mut library = Library::from_files(files);
    // English and Spanish still followed, with French glosses.
    assert_eq!(library.languages(), vec![EN, ES]);
    assert_eq!(library.native(), Some(NativeLanguage::French));
    let hablar = library
        .pack(ES)
        .and_then(|p| p.gloss("hablar").map(str::to_owned));
    assert_eq!(hablar.as_deref(), Some("Parler"));
    // `/vocab` names the file it skipped, after its listing.
    assert_eq!(library.skipped().len(), 1);
    assert_eq!(library.skipped()[0].native, NativeLanguage::English);
    assert_eq!(
        skipped_notice(library.skipped(), library.native()),
        "Pack ignoré : es-en.lingua — traduit en anglais, alors que le plugin suit les packs \
         traduits en français.\n"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn spec_scenario_today_s_installs_show_no_notice() {
    for (name, files) in [
        ("english-alone", vec![("pack.lingua", PACK_BYTES)]),
        (
            "english-and-spanish",
            vec![("pack.lingua", PACK_BYTES), ("es-fr.lingua", ES_PACK_BYTES)],
        ),
    ] {
        let (dir, files) = installed(name, &files);
        let library = Library::from_files(files);
        assert!(library.skipped().is_empty());
        assert_eq!(skipped_notice(library.skipped(), library.native()), "");
        std::fs::remove_dir_all(&dir).ok();
    }
    let none = Library::from_files(Vec::new());
    assert_eq!(none.native(), None);
    assert_eq!(skipped_notice(none.skipped(), none.native()), "");
}

#[test]
fn without_a_readable_pack_lingua_the_first_readable_file_gives_the_native_language() {
    let es_en = es_en_bytes();
    let (dir, files) = installed(
        "no-anchor",
        &[
            ("broken.lingua", b"not a pack"),
            ("es-en.lingua", &es_en),
            ("es-fr.lingua", ES_PACK_BYTES),
            ("pack.lingua", b"damaged"),
        ],
    );
    let library = Library::from_files(files);
    assert_eq!(library.native(), Some(NativeLanguage::English));
    assert_eq!(library.languages(), vec![ES]);
    assert_eq!(library.skipped().len(), 1);
    assert_eq!(library.skipped()[0].native, NativeLanguage::French);
    assert!(
        skipped_notice(library.skipped(), library.native())
            .starts_with("Pack ignoré : es-fr.lingua — traduit en français"),
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn loaded_packs_follow_the_first_one_s_native_language() {
    let library = Library::from_packs([pack(), Pack::load(&es_en_bytes()).expect("es-en loads")]);
    assert_eq!(library.languages(), vec![EN]);
    assert_eq!(library.native(), Some(NativeLanguage::French));
    assert!(Library::from_packs([]).native().is_none());
}

#[test]
fn a_spanish_reply_is_counted_in_spanish_and_its_code_does_not_vote() {
    let store = calibrated_store();
    let mut library = both();
    assert_eq!(library.language_of(SPANISH), Some(ES));
    ingest_texts(
        &store,
        &mut library,
        &[SPANISH.to_string()],
        "claude-code",
        1000,
    )
    .unwrap();
    assert!(store.exposure(ES, "hablar").unwrap() >= 2);
    assert!(store.exposure(ES, "casa").unwrap() >= 1);
    assert_eq!(store.exposure(EN, "city").unwrap(), 0); // no English counter moves
    assert_eq!(store.exposure(EN, "run").unwrap(), 0);
}

#[test]
fn an_english_reply_stays_english_with_two_languages_followed() {
    let store = calibrated_store();
    ingest_texts(
        &store,
        &mut both(),
        &[REPLY.to_string()],
        "claude-code",
        1000,
    )
    .unwrap();
    assert!(store.exposure(EN, "seldom").unwrap() >= 1);
    assert_eq!(store.exposure(ES, "seldom").unwrap(), 0);
}

#[test]
fn fenced_code_is_left_out_of_the_vote() {
    assert_eq!(
        prose_lines("Una frase.\n```rust\nlet x = 1;\n```\n\nOtra frase."),
        vec!["Una frase.", "Otra frase."]
    );
}

#[test]
fn the_statusline_names_the_language_with_two_followed() {
    let store = calibrated_store();
    let ks = store.knowledge_state().unwrap();
    let mut library = both();
    let spanish = statusline_text(&mut library, &ks, SPANISH).expect("analysable");
    assert!(spanish.starts_with("📖 ES "), "{spanish}");
    let english = statusline_text(&mut library, &ks, REPLY).expect("analysable");
    assert!(english.starts_with("📖 EN "), "{english}");
}

#[test]
fn vocab_lists_each_word_under_its_language() {
    let store = calibrated_store();
    let ks = store.knowledge_state().unwrap();
    let words = vocab_words(
        &mut both(),
        &ks,
        &[SPANISH.to_string(), REPLY.to_string()],
        |language| store.calibration(language).unwrap(),
    );
    let of = |language| -> Vec<&str> {
        words
            .iter()
            .filter(|w| w.language == language)
            .map(|w| w.lemma.as_str())
            .collect()
    };
    assert!(of(EN).contains(&"seldom"));
    assert!(of(ES).contains(&"hablar"));
    assert!(of(ES).contains(&"vino"));
    assert!(!of(ES).contains(&"casa")); // known below Spanish's 400
    let hablar = words.iter().find(|w| w.lemma == "hablar").unwrap();
    assert_eq!(hablar.gloss.as_deref(), Some("Parler"));
    assert!(hablar.rarity.contains("400"));
    // English first, as the plugin follows them, each under its heading.
    let text = listing(&words, true);
    let (en, es) = (
        text.find("Anglais :").unwrap(),
        text.find("Espagnol :").unwrap(),
    );
    assert!(en < es, "{text}");
    assert!(text.contains("--language en|es"));
}

#[test]
fn a_word_listed_in_two_languages_needs_one() {
    let store = calibrated_store();
    let word = |language| VocabWord {
        language,
        lemma: "no".into(),
        gloss: None,
        rarity: String::new(),
        sentence: "No.".into(),
    };
    let available = [word(EN), word(ES)];
    let added = add_to_deck(
        &store,
        &available,
        &["No".into()],
        None,
        NativeLanguage::French,
        0,
    )
    .unwrap();
    assert_eq!((added.added, added.ambiguous), (0, vec!["no".to_string()]));
    let added = add_to_deck(
        &store,
        &available,
        &["no".into()],
        Some(ES),
        NativeLanguage::French,
        0,
    )
    .unwrap();
    assert_eq!(added.added, 1);
    assert!(store.card(ES, "no").unwrap().is_some());
    assert!(store.card(EN, "no").unwrap().is_none());
}

#[test]
fn no_vocabulary_says_so() {
    assert_eq!(
        listing(&[], true),
        "Aucun mot inconnu dans cette session.\n"
    );
}

#[test]
fn spec_scenario_a_card_created_on_an_engine_glossed_in_english() {
    // lingua-decks-review (add-lingua-card-gloss-language D2): `/vocab --add` labels the card
    // with the native language the packs followed are glossed in; its gloss is the pack's.
    let store = calibrated_store();
    let word = |language, lemma: &str, gloss: &str| VocabWord {
        language,
        lemma: lemma.into(),
        gloss: Some(gloss.into()),
        rarity: String::new(),
        sentence: format!("{lemma}."),
    };
    let available = [word(ES, "faro", "lighthouse")];
    let added = add_to_deck(
        &store,
        &available,
        &["faro".into()],
        None,
        NativeLanguage::English,
        0,
    )
    .unwrap();
    assert_eq!(added.added, 1);
    let faro = store.card(ES, "faro").unwrap().unwrap();
    assert_eq!(
        (faro.gloss_language.as_str(), faro.gloss.as_deref()),
        ("en", Some("lighthouse"))
    );

    let available = [word(EN, "harbour", "port")];
    add_to_deck(
        &store,
        &available,
        &["harbour".into()],
        None,
        NativeLanguage::French,
        0,
    )
    .unwrap();
    assert_eq!(
        store.card(EN, "harbour").unwrap().unwrap().gloss_language,
        "fr"
    );
}

#[test]
fn every_studied_language_has_a_name_in_the_plugin_s_copy() {
    // add-lingua-french-baseline D8: French is named, though no French pack is published.
    assert_eq!(
        StudiedLanguage::ALL.map(language_name),
        ["Anglais", "Espagnol", "Français"]
    );
}
