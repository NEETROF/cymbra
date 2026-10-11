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

//! A soft hyphen is not part of a word (ignore-lingua-soft-hyphens D8), over the committed
//! corpora and the real packs built from the committed tables: every page of the English,
//! Spanish and French baselines, hyphenated in the test at fixed breaks — a rule written here,
//! so that no hyphenation dictionary enters the repository —, analyses, votes, glosses and
//! answers its word cards exactly as its clean text, spans aside, and each span covers its word
//! as written, soft hyphens included. With the spec's scenarios that need a real pack.
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use lingua_wasm::LinguaEngine;
use serde_json::Value;
use support::Scenario;
use support::english::ENGLISH;
use support::french::FRENCH;
use support::spanish::SPANISH;

/// U+00AD SOFT HYPHEN.
const SHY: char = '\u{AD}';

fn is_vowel(c: char) -> bool {
    "aeiouyáéíóúàèìòùâêîôûäëïöüœæ".contains(c.to_lowercase().next().unwrap_or(c))
}

/// `text` with a soft hyphen at fixed breaks: inside a run of letters, before a consonant that
/// follows a vowel and precedes one (`vi‧da`, `ma‧ña‧na`, `go‧vernment`, `che‧min`). Not a
/// language's hyphenation, nor needing one: a break anywhere inside a word must read as none.
fn hyphenate(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() * 2);
    for (i, &c) in chars.iter().enumerate() {
        let inside = i > 0
            && i + 1 < chars.len()
            && chars[i - 1].is_alphabetic()
            && chars[i + 1].is_alphabetic();
        if inside && is_vowel(chars[i - 1]) && !is_vowel(c) && is_vowel(chars[i + 1]) {
            out.push(SHY);
        }
        out.push(c);
    }
    out
}

/// An engine holding `pairs`' packs, built from the committed tables — one native language,
/// as a reader's —, each studied language calibrated at the 5,000 commonest dictionary forms.
fn engine(pairs: &[&'static str]) -> LinguaEngine {
    let packs: Vec<(&str, Vec<u8>)> = pairs
        .iter()
        .map(|pair| (*pair, Scenario::real_pack(pair)))
        .collect();
    let mut engine = Scenario::engine(&packs);
    for pair in pairs {
        let language = &pair[..2];
        if engine
            .set_calibration(5_000, Some(language.into()))
            .is_err()
        {
            panic!("calibrate {language}");
        }
    }
    engine
}

fn json(text: &str) -> Value {
    serde_json::from_str(text).expect("the engine's JSON")
}

/// A page analysis with every token's span set aside.
fn spans_aside(analysis: &Value) -> Value {
    let mut analysis = analysis.clone();
    for token in analysis["tokens"].as_array_mut().expect("tokens") {
        let token = token.as_object_mut().expect("a token");
        token.remove("start");
        token.remove("end");
    }
    analysis
}

fn ok<T>(result: Result<T, wasm_bindgen::JsError>) -> T {
    match result {
        Ok(value) => value,
        Err(_) => panic!("the engine refused"),
    }
}

/// Every page of `scenario`'s corpus, hyphenated, read as its clean text by `engine`. Returns
/// how many tokens' spans hold a soft hyphen.
fn corpus_reads_as_clean(engine: &LinguaEngine, scenario: &Scenario, language: &str) -> usize {
    let lang = || Some(language.to_owned());
    let mut hyphenated_tokens = 0;
    for (name, blocks) in scenario.pages() {
        assert!(
            blocks.iter().all(|b| !b.contains(SHY)),
            "{name}: the corpus holds no soft hyphen"
        );
        let written: Vec<String> = blocks.iter().map(|b| hyphenate(b)).collect();
        assert_ne!(written, blocks, "{name}");
        let clean = json(&ok(engine.analyse(blocks.clone(), lang())));
        let shy = json(&ok(engine.analyse(written.clone(), lang())));
        assert_eq!(spans_aside(&shy), spans_aside(&clean), "{language} {name}");
        let tokens = shy["tokens"].as_array().expect("tokens");
        for (h, c) in tokens
            .iter()
            .zip(clean["tokens"].as_array().expect("tokens"))
        {
            let span = |t: &Value, text: &[String]| -> String {
                let block = &text[t["block"].as_u64().unwrap() as usize];
                block[t["start"].as_u64().unwrap() as usize..t["end"].as_u64().unwrap() as usize]
                    .to_owned()
            };
            let (as_written, as_clean) = (span(h, &written), span(c, &blocks));
            assert_eq!(as_written.replace(SHY, ""), as_clean, "{language} {name}");
            if as_written.contains(SHY) {
                hyphenated_tokens += 1;
                // The word card, asked of the word as written and of the clean word.
                let lemma = h["lemma"].as_str().unwrap();
                assert_eq!(
                    ok(engine.word_grammar(&as_written, lemma, lang())),
                    ok(engine.word_grammar(&as_clean, lemma, lang())),
                    "{language} {name}: {as_written}"
                );
            }
        }
        for candidates in [
            vec!["en", "es"],
            vec!["es", "fr"],
            vec!["en", "fr"],
            vec!["en", "es", "fr"],
            vec!["fr", "es", "en"],
        ] {
            let candidates: Vec<String> = candidates.into_iter().map(str::to_owned).collect();
            assert_eq!(
                ok(engine.detect_language(written.clone(), candidates.clone(), None)),
                ok(engine.detect_language(blocks.clone(), candidates, None)),
                "{language} {name}"
            );
        }
        for (w, c) in written.iter().zip(&blocks) {
            assert_eq!(
                ok(engine.phrase_gloss(w, lang())),
                ok(engine.phrase_gloss(c, lang())),
                "{language} {name}: {c}"
            );
        }
    }
    hyphenated_tokens
}

#[test]
fn spec_scenario_a_page_holding_soft_hyphens() {
    // A reader of French studying English and Spanish, and a reader of English studying French.
    let french_reader = engine(&["en-fr", "es-fr"]);
    let english_reader = engine(&["es-en", "fr-en"]);
    for (engine, scenario, language) in [
        (&french_reader, &ENGLISH, "en"),
        (&french_reader, &SPANISH, "es"),
        (&english_reader, &FRENCH, "fr"),
    ] {
        let hyphenated = corpus_reads_as_clean(engine, scenario, language);
        assert!(
            hyphenated > 100,
            "{language}: {hyphenated} hyphenated tokens"
        );
    }
}

#[test]
fn the_spec_scenarios_with_the_real_packs() {
    let engine = engine(&["en-fr", "es-fr"]);
    let es = || Some("es".to_owned());
    // A Spanish word read whole: `vida`, its dictionary form `vida`, over the six bytes of
    // « vi‧da » as written.
    let block = "La vi\u{AD}da em\u{AD}pie\u{AD}za cuan\u{AD}do te das cuen\u{AD}ta de quién eres.";
    let page = json(&ok(engine.analyse(vec![block.into()], es())));
    assert_eq!(page["analysable"], true, "{page}");
    let vida = &page["tokens"][1];
    assert_eq!(
        (&vida["surface"], &vida["lemma"]),
        (&"vida".into(), &"vida".into())
    );
    assert_eq!((&vida["start"], &vida["end"]), (&3.into(), &9.into()));
    let selection = json(&ok(engine.phrase_gloss("vi\u{AD}da", es())));
    assert_eq!(selection["tokens"][0]["surface"], "vida");
    assert_eq!(selection["tokens"][0]["lemma"], "vida");
    // A selection and a card: `mañana`, glossed, and the readings of « cantábamos ».
    let selection = ok(engine.phrase_gloss("ma\u{AD}ña\u{AD}na", es()));
    assert_eq!(selection, ok(engine.phrase_gloss("mañana", es())));
    let selection = json(&selection);
    assert_eq!(selection["tokens"][0]["surface"], "mañana");
    assert!(selection["tokens"][0]["gloss"].is_string(), "{selection}");
    let card = ok(engine.word_grammar("can\u{AD}tá\u{AD}ba\u{AD}mos", "cantar", es()));
    assert_eq!(card, ok(engine.word_grammar("cantábamos", "cantar", es())));
    assert!(
        !json(&card)["readings"].as_array().unwrap().is_empty(),
        "{card}"
    );
    // English words and a contraction, and a French elision, as their clean text.
    let en = || Some("en".to_owned());
    let english = "The gov\u{AD}ern\u{AD}ment could\u{AD}n't an\u{AD}swer the ques\u{AD}tion \
                   yes\u{AD}ter\u{AD}day af\u{AD}ter\u{AD}noon.";
    assert_eq!(
        ok(engine.phrase_gloss(english, en())),
        ok(engine.phrase_gloss(&english.replace(SHY, ""), en()))
    );
    let engine = self::engine(&["fr-en"]);
    let fr = || Some("fr".to_owned());
    assert_eq!(
        ok(engine.phrase_gloss("lors\u{AD}qu’il", fr())),
        ok(engine.phrase_gloss("lorsqu’il", fr()))
    );
    let lorsque = json(&ok(engine.phrase_gloss("lors\u{AD}qu’il", fr())));
    assert_eq!(lorsque["tokens"][0]["lemma"], "lorsque");
    assert_eq!(lorsque["tokens"][1]["lemma"], "il");
}
