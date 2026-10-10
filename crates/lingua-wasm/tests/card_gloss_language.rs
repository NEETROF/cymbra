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

//! A card says the language of its gloss, and review shows one the reader can read
//! (add-lingua-card-gloss-language, `lingua-decks-review`).
//!
//! A card is created on an engine glossed in English — es-en, the committed pair glossed in
//! English (add-lingua-pack-es-en), built over es-fr's studied tables — with the gloss its
//! surface showed, and pulled as a card operation into a French-native engine holding es-fr.
//! Reviewed there, the view shows es-fr's gloss for the lemma: a word's, or an expression's from
//! the pack's expression table; and the card's own text when es-fr has none. The card keeps its
//! English text and label. The baselines pin the view of every French card.
//!
//! A French expression card is named by its headword (add-lingua-french-expression-keys D3):
//! reviewed in another language than its gloss's, it is read in the table at the key its name
//! reads as, in the pack built from the committed French tables (add-lingua-pack-fr-en; the fr-en
//! fixture before them). A Spanish expression card is named by its headword too
//! (add-lingua-spanish-expression-keys D3), and one made before was made under its key: review
//! reads es-fr's table at the card's lemma, then at the key the lemma reads as (D7). A French
//! expression named without a space (`d'abord`) is read at the key its name reads as (`de abord`),
//! and a French word written with an apostrophe (`aujourd'hui`, one token) as a word
//! (add-lingua-french-word-card D8).
//!
//! Host only: the pack builder is native (C zstd).

#![cfg(not(target_arch = "wasm32"))]

mod support;

use lingua_core::packs::Pack;
use lingua_wasm::LinguaEngine;
use support::english::ENGLISH;
use support::french::FRENCH;
use support::spanish::SPANISH;
use support::{PackSource, Scenario};

/// 2026-09-21T13:46:40Z, in epoch seconds: the deck bindings' unit.
const T: f64 = 1_790_000_000.0;
const DAY: f64 = 86_400.0;

/// The reference es-fr pack, and es-en: the same studied tables, glossed in English — both from
/// the committed tables.
fn packs() -> (Vec<u8>, Vec<u8>) {
    (Scenario::real_pack("es-fr"), Scenario::real_pack("es-en"))
}

/// An engine glossed in English, holding one Spanish card created with the gloss its surface
/// showed.
fn glossed_in_english(es_en: &[u8], lemma: &str, gloss: &str) -> LinguaEngine {
    let mut engine = Scenario::engine(&[("es-en", es_en.to_vec())]);
    assert_eq!(engine.native_language(), "en");
    engine
        .add_card(
            lemma,
            lemma,
            "Una frase con la palabra.",
            "https://example.es/pagina",
            Some(gloss.to_owned()),
            T,
            Some("es".to_owned()),
        )
        .unwrap();
    engine
}

/// A French-native engine holding es-fr, the English-native engine's card pulled in as a card
/// operation and under review.
fn french_native_reviewing(es_fr: &[u8], from: &LinguaEngine) -> LinguaEngine {
    let mut engine = Scenario::engine(&[("es-fr", es_fr.to_vec())]);
    assert_eq!(engine.native_language(), "fr");
    match engine.apply_card_ops(&from.export_card_ops()) {
        Ok(changed) => assert_eq!(changed, 1),
        Err(_) => panic!("the pulled card applies"),
    }
    assert_eq!(engine.start_review(T + DAY, None), 1);
    engine
}

fn view(engine: &LinguaEngine) -> serde_json::Value {
    match engine
        .review_current()
        .map(|json| serde_json::from_str(&json))
    {
        Some(Ok(view)) => view,
        _ => panic!("a card is under review"),
    }
}

fn ops(engine: &LinguaEngine) -> serde_json::Value {
    serde_json::from_str(&engine.export_card_ops()).expect("card ops")
}

/// The card's own text and label, as the engine exports them.
fn text_and_label(engine: &LinguaEngine) -> (String, String) {
    let ops = ops(engine);
    (
        ops[0]["gloss"].as_str().unwrap_or_default().to_owned(),
        ops[0]["gloss_language"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
    )
}

#[test]
fn spec_scenario_a_card_created_on_an_engine_glossed_in_english() {
    let (es_fr, es_en) = packs();
    let engine = glossed_in_english(&es_en, "casa", "house");
    // Its gloss language is `en`, which the export emits…
    assert_eq!(
        text_and_label(&engine),
        ("house".to_owned(), "en".to_owned())
    );
    // …and the backup carries, at schema version 2, the version the previous build reads: the
    // label is one more field a card does not deny.
    let backup = engine.backup();
    assert!(backup.contains("\"gloss_language\": \"en\""), "{backup}");
    let file: serde_json::Value = serde_json::from_str(&backup).expect("a backup");
    assert_eq!(file["schema_version"], 2);
    // A French-native engine of this build restores it whole.
    let mut restored = Scenario::engine(&[("es-fr", es_fr)]);
    match restored.restore(&backup) {
        Ok(()) => {}
        Err(_) => panic!("the backup restores"),
    }
    assert_eq!(
        text_and_label(&restored),
        ("house".to_owned(), "en".to_owned())
    );
    assert_eq!(restored.backup(), backup, "the backup is a fixpoint");
}

#[test]
fn a_card_seeded_from_the_pack_is_labelled_with_the_engines_native_language() {
    // The other way a card is created on an engine: seeded from the pack's level lists, with
    // the pack's gloss — in the engine's native language, which labels every seeded card.
    let (_, es_en) = packs();
    let mut engine = Scenario::engine(&[("es-en", es_en)]);
    assert_eq!(engine.native_language(), "en");
    let seeded = engine
        .seed_level("A1", 5, "common", T, Some("es".to_owned()))
        .unwrap();
    assert_eq!(seeded, 5);
    let ops = ops(&engine);
    let ops = ops.as_array().expect("an array of card operations");
    assert_eq!(ops.len(), 5);
    for op in ops {
        assert_eq!(op["gloss_language"], "en", "{}", op["lemma"]);
    }
    assert_eq!(
        engine
            .backup()
            .matches("\"gloss_language\": \"en\"")
            .count(),
        5
    );
}

#[test]
fn review_reads_the_pack_of_the_cards_language_not_the_first_one_held() {
    // A French-native engine holding en-fr first and es-fr beside it, as the extension does,
    // reviews a Spanish card glossed in English: the gloss is es-fr's for the lemma, not
    // en-fr's, which does not gloss the Spanish word that way.
    let (es_fr, es_en) = packs();
    let es_fr_gloss = Pack::load(&es_fr)
        .unwrap()
        .gloss("casa")
        .expect("es-fr glosses casa")
        .to_owned();
    let en_fr_gloss = Pack::load(&Scenario::real_pack("en-fr"))
        .unwrap()
        .gloss("casa")
        .map(str::to_owned);
    assert_ne!(en_fr_gloss.as_ref(), Some(&es_fr_gloss));

    let mut engine = SPANISH.loaded();
    assert_eq!(engine.native_language(), "fr");
    match engine.apply_card_ops(&glossed_in_english(&es_en, "casa", "house").export_card_ops()) {
        Ok(changed) => assert_eq!(changed, 1),
        Err(_) => panic!("the pulled card applies"),
    }
    assert_eq!(engine.start_review(T + DAY, None), 1);
    assert_eq!(engine.review_current_language().as_deref(), Some("es"));
    assert_eq!(
        view(&engine)["gloss"],
        es_fr_gloss,
        "es-fr's gloss, not en-fr's"
    );
    assert_eq!(
        text_and_label(&engine),
        ("house".to_owned(), "en".to_owned())
    );
}

#[test]
fn spec_scenario_a_word_glossed_in_another_native_language() {
    let (es_fr, es_en) = packs();
    let pack_gloss = Pack::load(&es_fr)
        .unwrap()
        .gloss("casa")
        .expect("es-fr glosses casa")
        .to_owned();
    assert_ne!(pack_gloss, "house");

    let mut engine = french_native_reviewing(&es_fr, &glossed_in_english(&es_en, "casa", "house"));
    let shown = view(&engine);
    assert_eq!(shown["headword"], "casa");
    assert_eq!(shown["gloss"], pack_gloss, "es-fr's gloss for the lemma");
    assert_eq!(shown["sentence"], "Una frase con la palabra.");
    // The card keeps its English text and label, under review and once graded.
    assert_eq!(
        text_and_label(&engine),
        ("house".to_owned(), "en".to_owned())
    );
    engine.review_reveal();
    assert_eq!(view(&engine)["gloss"], pack_gloss);
    engine.review_grade("good", T + DAY);
    assert_eq!(
        text_and_label(&engine),
        ("house".to_owned(), "en".to_owned())
    );
    assert!(engine.backup().contains("\"gloss\": \"house\""));
}

#[test]
fn spec_scenario_an_expression_glossed_in_another_native_language() {
    let (es_fr, es_en) = packs();
    // The expression table is keyed by Spanish's reading — `tener en cuenta` is held as `tener
    // en contar` —, the lemma the surface created the card with before Spanish expressions were
    // named: a card made before (add-lingua-spanish-expression-keys D7). The surface now creates
    // it with the name, `tener en cuenta` (the next scenario).
    let key = "tener en contar";
    let pack_gloss = Pack::load(&es_fr)
        .unwrap()
        .expression(key)
        .expect("es-fr's expression table holds the key")
        .to_owned();
    assert_ne!(pack_gloss, "to take into account");

    let engine = french_native_reviewing(
        &es_fr,
        &glossed_in_english(&es_en, key, "to take into account"),
    );
    let shown = view(&engine);
    assert_eq!(shown["headword"], key);
    assert_eq!(
        shown["gloss"], pack_gloss,
        "es-fr's expression gloss for the key"
    );
    assert_eq!(
        text_and_label(&engine),
        ("to take into account".to_owned(), "en".to_owned())
    );
}

#[test]
fn spec_scenario_a_named_spanish_expression_card_glossed_in_another_language() {
    let (es_fr, es_en) = packs();
    let pack = Pack::load(&es_fr).unwrap();
    // `tener en cuenta` is the card's name, no key of the table; `tener en contar` is its key.
    assert_eq!(pack.expression("tener en cuenta"), None);
    assert_eq!(
        pack.expression_name("tener en contar"),
        Some("tener en cuenta")
    );
    let pack_gloss = pack
        .expression("tener en contar")
        .expect("es-fr holds the key")
        .to_owned();

    let mut engine = french_native_reviewing(
        &es_fr,
        &glossed_in_english(&es_en, "tener en cuenta", "to take into account"),
    );
    let shown = view(&engine);
    assert_eq!(shown["headword"], "tener en cuenta");
    assert_eq!(
        shown["gloss"], pack_gloss,
        "es-fr's gloss for `tener en contar`"
    );
    // The card keeps its English text and label, under review and once graded.
    assert_eq!(
        text_and_label(&engine),
        ("to take into account".to_owned(), "en".to_owned())
    );
    engine.review_grade("good", T + DAY);
    assert_eq!(
        text_and_label(&engine),
        ("to take into account".to_owned(), "en".to_owned())
    );
}

#[test]
fn a_spanish_card_is_read_at_its_lemma_then_at_its_name_s_key() {
    let (es_fr, es_en) = packs();
    let pack = Pack::load(&es_fr).unwrap();
    let gloss_of = |key: &str| pack.expression(key).expect(key).to_owned();
    // `al menos` is read at `a el menos`, `a la vez` is its own key; a card made before on a
    // chain the determiners now split (`a el vez`, D2) is no key any more and shows its own text,
    // as a gloss the pack has not.
    assert_eq!(pack.expression("a el vez"), None);
    for (lemma, text, shown) in [
        ("al menos", "at least", gloss_of("a el menos")),
        ("a la vez", "at the same time", gloss_of("a la vez")),
        (
            "a el vez",
            "at the same time",
            "at the same time".to_owned(),
        ),
    ] {
        let engine = french_native_reviewing(&es_fr, &glossed_in_english(&es_en, lemma, text));
        assert_eq!(view(&engine)["gloss"], shown, "{lemma}");
    }
}

#[test]
fn spec_scenario_a_gloss_the_pack_has_not() {
    let (es_fr, es_en) = packs();
    let pack = Pack::load(&es_fr).unwrap();
    // A word es-en glosses and es-fr does not, and an expression neither holds: the view shows
    // the card's own text, in its language.
    assert!(pack.gloss("augusto").is_none());
    assert!(Pack::load(&es_en).unwrap().gloss("augusto").is_some());
    assert!(pack.expression("hablar de casa").is_none());
    for (lemma, text) in [
        ("augusto", "Gloss of augusto"),
        ("hablar de casa", "to talk about home"),
    ] {
        let engine = french_native_reviewing(&es_fr, &glossed_in_english(&es_en, lemma, text));
        let shown = view(&engine);
        assert_eq!(shown["headword"], lemma);
        assert_eq!(shown["gloss"], text, "{lemma}: the card's own text");
        assert_eq!(text_and_label(&engine), (text.to_owned(), "en".to_owned()));
    }
}

#[test]
fn a_card_glossed_in_the_engines_native_language_is_shown_as_before() {
    // The label is the engine's native: the card's text, not the pack's, even where they differ.
    let (es_fr, _) = packs();
    let mut engine = Scenario::engine(&[("es-fr", es_fr)]);
    engine
        .add_card(
            "casa",
            "casa",
            "La casa.",
            "https://example.es",
            Some("demeure".to_owned()),
            T,
            Some("es".to_owned()),
        )
        .unwrap();
    assert_eq!(engine.start_review(T + DAY, None), 1);
    assert_eq!(view(&engine)["gloss"], "demeure");
    assert!(
        ops(&engine)[0].get("gloss_language").is_none(),
        "fr is left out"
    );
}

#[test]
fn a_card_of_a_language_the_engine_holds_no_pack_for_keeps_its_text() {
    // Restored on a French-native engine holding en-fr alone, a Spanish card glossed in English
    // has no pack to be read through: its own text is shown.
    let (_, es_en) = packs();
    let backup = glossed_in_english(&es_en, "casa", "house").backup();
    let mut engine = ENGLISH.loaded();
    match engine.restore(&backup) {
        Ok(()) => {}
        Err(_) => panic!("the backup restores"),
    }
    assert_eq!(engine.start_review(T + DAY, None), 1);
    assert_eq!(engine.review_current_language().as_deref(), Some("es"));
    assert_eq!(view(&engine)["gloss"], "house");
}

/// A French card glossed in Spanish, as a Spanish-native engine creates it, written as the card
/// operation it syncs as: no pack glossed in Spanish studies French before change 49, so the
/// operation is the wire's.
fn french_card_glossed_in_spanish(lemma: &str, surface: &str, gloss: &str) -> String {
    serde_json::json!([{
        "client_id": lemma,
        "language": "fr",
        "lemma": lemma,
        "surface_form": surface,
        "source_sentence": "Au revoir, et à demain.",
        "source": "",
        "gloss": gloss,
        "gloss_language": "es",
        "fsrs_state": "",
        "deleted": false,
        "client_ts": (T as i64) * 1000,
        "device_id": "",
    }])
    .to_string()
}

/// An English-native engine holding fr-en's committed pack beside es-en, as the French baseline's
/// reader does, reviewing the French card the operation creates.
fn english_native_reviewing_french(op: &str) -> LinguaEngine {
    let mut engine = FRENCH.loaded();
    assert_eq!(engine.native_language(), "en");
    match engine.apply_card_ops(op) {
        Ok(changed) => assert_eq!(changed, 1),
        Err(_) => panic!("the French card applies"),
    }
    assert_eq!(engine.start_review(T + DAY, None), 1);
    assert_eq!(engine.review_current_language().as_deref(), Some("fr"));
    engine
}

#[test]
fn spec_scenario_a_french_expression_card_glossed_in_another_language() {
    let pack = Pack::load(&PackSource::Tables.pack("fr-en")).unwrap();
    // `au revoir` is the card's name, no key of the table; `à le revoir` is its key.
    assert_eq!(pack.expression("au revoir"), None);
    assert_eq!(pack.expression_name("à le revoir"), Some("au revoir"));
    let pack_gloss = pack
        .expression("à le revoir")
        .expect("the pack holds the key")
        .to_owned();

    let mut engine = english_native_reviewing_french(&french_card_glossed_in_spanish(
        "au revoir",
        "Au revoir",
        "adiós",
    ));
    let shown = view(&engine);
    assert_eq!(shown["headword"], "au revoir");
    assert_eq!(
        shown["gloss"], pack_gloss,
        "the pack's gloss for `à le revoir`"
    );
    // The card keeps its Spanish text and label, under review and once graded.
    assert_eq!(
        text_and_label(&engine),
        ("adiós".to_owned(), "es".to_owned())
    );
    engine.review_grade("good", T + DAY);
    assert_eq!(
        text_and_label(&engine),
        ("adiós".to_owned(), "es".to_owned())
    );
}

#[test]
fn a_french_card_is_read_at_its_name_s_key_or_shows_its_own_text() {
    let pack = Pack::load(&PackSource::Tables.pack("fr-en")).unwrap();
    let gloss_of = |key: &str| pack.expression(key).expect(key).to_owned();
    for (lemma, surface, text, shown) in [
        // Named by its headword, read at its key; a name that is its own key.
        ("il y a", "il y avait", "hay", gloss_of("il y avoir")),
        (
            "tout de suite",
            "tout de suite",
            "enseguida",
            gloss_of("tout de suite"),
        ),
        // A card made under the key itself, a lemma chain, reads as that key too.
        ("à le revoir", "Au revoir", "adiós", gloss_of("à le revoir")),
        // An expression the pack does not hold shows the card's own text; a French word is read
        // as before.
        (
            "au bout du quai",
            "au bout du quai",
            "al final del muelle",
            "al final del muelle".to_owned(),
        ),
        (
            "maison",
            "maison",
            "casa",
            pack.gloss("maison").expect("maison").to_owned(),
        ),
    ] {
        let engine =
            english_native_reviewing_french(&french_card_glossed_in_spanish(lemma, surface, text));
        assert_eq!(view(&engine)["gloss"], shown, "{lemma}");
    }
}

/// add-lingua-french-word-card D8: a French expression named without a space, as « + Deck »
/// stores it from the whole-selection card « D’abord » answers, is read in the table at the key
/// its name reads as; the card keeps its text and label.
#[test]
fn spec_scenario_an_expression_named_without_a_space() {
    let pack = Pack::load(&PackSource::Tables.pack("fr-en")).unwrap();
    // `d'abord` is the card's name, no word of the gloss table; `de abord` is its key.
    assert_eq!(pack.gloss("d'abord"), None);
    assert_eq!(pack.expression_name("de abord"), Some("d'abord"));
    let pack_gloss = pack
        .expression("de abord")
        .expect("the pack holds the key")
        .to_owned();

    let mut engine = english_native_reviewing_french(&french_card_glossed_in_spanish(
        "d'abord",
        "D\u{2019}abord",
        "primero",
    ));
    let shown = view(&engine);
    assert_eq!(shown["headword"], "d'abord");
    assert_eq!(
        shown["gloss"], pack_gloss,
        "the pack's gloss for `de abord`"
    );
    assert_eq!(
        text_and_label(&engine),
        ("primero".to_owned(), "es".to_owned())
    );
    engine.review_grade("good", T + DAY);
    assert_eq!(
        text_and_label(&engine),
        ("primero".to_owned(), "es".to_owned())
    );
}

/// A French word written with an apostrophe reads as one token: it is looked up as a word, as
/// before (add-lingua-french-word-card D8).
#[test]
fn spec_scenario_a_word_written_with_an_apostrophe() {
    let pack = Pack::load(&PackSource::Tables.pack("fr-en")).unwrap();
    let word = pack
        .gloss("aujourd'hui")
        .expect("the pack glosses aujourd'hui")
        .to_owned();
    let engine = english_native_reviewing_french(&french_card_glossed_in_spanish(
        "aujourd'hui",
        "Aujourd'hui",
        "hoy",
    ));
    assert_eq!(view(&engine)["gloss"], word);
}
