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

//! Cross-native invariance (add-lingua-pack-lexical-layer D6): an analysis does not depend
//! on the native language its pack is glossed in.
//!
//! For English, Spanish and French, the reference pack (en-fr, es-fr, fr-en) is built from its
//! committed tables, and the second pack is the real one over the same studied tables: en-es, the
//! committed pair glossed in Spanish (add-lingua-pack-en-es), built from `tables/en/` and
//! `tables/en-es/`; es-en, the committed pair glossed in English (add-lingua-pack-es-en), built
//! from `tables/es/` and `tables/es-en/`; fr-es, French glossed in Spanish
//! (add-lingua-pack-fr-es), built from `tables/fr/` and `tables/fr-es/`.
//!
//! Every probe of the language's invariance baseline is answered through both packs, and must
//! be byte for byte alike once glosses, senses and expressions — the native side — are removed
//! (`support::studied_side`, which `es_en_baseline.rs` and `en_es_baseline.rs` also compare the
//! committed goldens through). The studied sections must be byte-equal. The baselines' own
//! goldens are not read here.
//!
//! The cards a level seeds are the one part of a reader's history that follows the native
//! language (seed-lingua-decks-with-glossed-lemmas D4): each pack seeds the lemmas of the level it
//! glosses, and the two packs of a language do not gloss the same ones. So the baseline is
//! answered here without the reader's level seeding (`Scenario::render_unseeded`) — every probe
//! after it, the review, the statuses, the pages, the exports and the backup, still compared —
//! and `seeding_follows_each_packs_glosses` checks the seeding apart, through both packs, every
//! level, both orders: the gloss filter is its one native dependency.
//!
//! Host only: the pack builder is native (C zstd).

#![cfg(not(target_arch = "wasm32"))]

mod support;

use std::cmp::Reverse;
use std::collections::BTreeSet;

use lingua_core::knowledge::level::{CefrLevel, CefrLevels};
use lingua_core::knowledge::state::FrequencyRanks;
use lingua_core::packs::Pack;
use lingua_core::packs::format::read_container;
use lingua_core::packs::pack::section;
use lingua_pack::{build_pack, inputs_from_tables};
use support::english::ENGLISH;
use support::french::FRENCH;
use support::spanish::SPANISH;
use support::{Scenario, probes, studied_side};

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

/// The studied sections are byte-equal, the tag pool up to the pinned prefix and the
/// readings' tags; the second pack carries a lexical table, the reference none.
fn assert_studied_sections_alike(pair: &str, reference: &[u8], other: &[u8]) {
    let (a, b) = (sections(reference), sections(other));
    for name in [
        section::FORMS,
        section::LEMMAS,
        section::FREQ,
        section::LEVELS,
        section::PARADIGMS_ZST,
    ] {
        assert!(section_of(&a, name).is_some(), "{pair} carries {name}");
        assert!(
            section_of(&a, name) == section_of(&b, name),
            "{pair}: the {name} section depends on the native language"
        );
    }
    let pool = |s: &[(String, Vec<u8>)]| -> Vec<String> {
        String::from_utf8(section_of(s, section::TAGS).expect("tags").to_vec())
            .unwrap()
            .split('\n')
            .map(str::to_owned)
            .collect()
    };
    let (pinned, grown) = (pool(&a), pool(&b));
    assert_eq!(grown[..pinned.len()], pinned[..], "{pair}: the pinned pool");
    assert!(
        grown[pinned.len()..].iter().all(|tag| !tag.contains('|')),
        "{pair}: only sense parts of speech follow the pin: {:?}",
        &grown[pinned.len()..]
    );
    assert!(section_of(&a, section::LEXICAL).is_none(), "{pair}");
    let lemmas = section_of(&a, section::LEMMAS)
        .unwrap()
        .split(|&b| b == b'\n')
        .count();
    assert_eq!(
        section_of(&b, section::LEXICAL).map(<[u8]>::len),
        Some(lemmas.div_ceil(8)),
        "{pair}: the second pack's lexical table"
    );
}

/// The exported card operations say the language of their glosses before it is stripped: a
/// French gloss carries no label — en-fr's and es-fr's, the references' —, every other carries
/// its native's: fr-en's `en` (`reference_native`), the other pack's `native`
/// (add-lingua-card-gloss-language D2) — so a label astray elsewhere is not hidden by the
/// strip.
fn assert_card_ops_labelled(reference: &str, other: &str, reference_native: &str, native: &str) {
    let ops = |body: &str| -> Vec<serde_json::Value> {
        serde_json::from_str(body).expect("an array of card operations")
    };
    let (a, b) = (ops(reference), ops(other));
    assert!(!a.is_empty() && a.len() == b.len(), "the same cards");
    for op in &a {
        if reference_native == "fr" {
            assert!(
                op.get("gloss_language").is_none(),
                "a French gloss carries no label: {op}"
            );
        } else {
            assert_eq!(op["gloss_language"], reference_native, "{op}");
        }
    }
    for op in &b {
        assert_eq!(op["gloss_language"], native, "{op}");
    }
}

/// Every probe of `scenario`, through `reference` and through `other`: alike once the native
/// side is removed — the other pack's own credits included. `reference_native` and `native` are
/// the two packs' native languages, which label their cards.
fn assert_probes_alike(
    scenario: &Scenario,
    language: Option<&str>,
    reference: &[u8],
    other: &[u8],
    reference_native: &str,
    native: &str,
) {
    let render = |pack: &[u8]| {
        // The scenario's pair labels the pack line, which is not compared. The reader's level
        // seeding is left out: its cards follow the pack's glosses, which
        // `seeding_follows_each_packs_glosses` checks (seed-lingua-decks-with-glossed-lemmas D4).
        scenario.render_unseeded(vec![(scenario.pair, pack.to_vec())], language)
    };
    let (a, b) = (probes(&render(reference)), probes(&render(other)));
    assert_eq!(
        a.iter().map(|(n, _)| n).collect::<Vec<_>>(),
        b.iter().map(|(n, _)| n).collect::<Vec<_>>(),
        "the same probes"
    );
    let mut compared = 0;
    for ((name, x), (_, y)) in a.iter().zip(&b) {
        if name == "export-card-ops" {
            assert_card_ops_labelled(x, y, reference_native, native);
        }
        let (Some(x), Some(y)) = (studied_side(name, x), studied_side(name, y)) else {
            continue;
        };
        compared += 1;
        let at = x.chars().zip(y.chars()).take_while(|(p, q)| p == q).count();
        assert!(
            x == y,
            "{}: probe `{name}` depends on the native language, character {at}:\n  \
             reference: …{}…\n  other:     …{}…",
            scenario.pair,
            x.chars()
                .skip(at.saturating_sub(80))
                .take(240)
                .collect::<String>(),
            y.chars()
                .skip(at.saturating_sub(80))
                .take(240)
                .collect::<String>(),
        );
    }
    // Every probe but the pack line, the cards' glosses and — a pack's own credits — the notice
    // and the licences.
    assert_eq!(
        compared,
        a.len() - 1 - 2 - scenario.lemmas.len(),
        "probes compared"
    );
}

/// The estimate's universe and each level's typical vocabulary, through an engine.
fn sizes(pack: &[u8], language: &str) -> (u64, Vec<u64>) {
    let engine = Scenario::engine(&[("pack", pack.to_vec())]);
    let lang = || Some(language.to_owned());
    let estimate: serde_json::Value =
        serde_json::from_str(&engine.vocabulary_estimate(lang()).unwrap()).unwrap();
    let ladder: serde_json::Value =
        serde_json::from_str(&engine.level_ladder(lang()).unwrap()).unwrap();
    let typical = ladder
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["typicalVocabulary"].as_u64().unwrap())
        .collect();
    (estimate["universe"].as_u64().unwrap(), typical)
}

#[test]
fn spec_scenario_english_through_another_native_language() {
    // en-fr, and en-es: the committed pair glossed in Spanish (add-lingua-pack-en-es), both
    // built from tables/en/ — the real second pack, not a synthetic one.
    let root = Scenario::tables_root();
    let (reference_inputs, other_inputs) = (
        inputs_from_tables(&root, "en-fr").expect("read en-fr"),
        inputs_from_tables(&root, "en-es").expect("read en-es"),
    );
    assert_eq!(
        other_inputs.meta.native, "es",
        "en-es is glossed in Spanish"
    );
    // *Glossed in Spanish with fewer glosses*.
    assert!(other_inputs.glosses.len() < reference_inputs.glosses.len());
    let (reference, other) = (
        build_pack(&reference_inputs).expect("en-fr"),
        build_pack(&other_inputs).expect("en-es"),
    );
    let (en_fr, en_es) = (Pack::load(&reference).unwrap(), Pack::load(&other).unwrap());
    assert_eq!(en_es.pair().key(), "en-es", "glossed in Spanish");
    assert_studied_sections_alike("en-fr", &reference, &other);
    assert_probes_alike(&ENGLISH, None, &reference, &other, "fr", "es");
    // Fewer glosses, yet glosses of its own: a lemma this pack glosses and en-fr does not is no
    // dictionary word — the dictionary words are en-fr's, read from tables/en (the lexical
    // section `assert_studied_sections_alike` found).
    let only_here: Vec<&str> = other_inputs
        .glosses
        .iter()
        .map(|(lemma, _)| lemma.as_str())
        .filter(|lemma| en_fr.gloss(lemma).is_none())
        .take(8)
        .collect();
    assert!(!only_here.is_empty(), "en-es glosses lemmas en-fr does not");
    for lemma in &only_here {
        assert!(
            en_es.gloss(lemma).is_some() && !en_es.is_dictionary_word(lemma),
            "{lemma}"
        );
    }

    // *A vocabulary size counts dictionary words*: the universe and B1's typical vocabulary.
    let (universe, typical) = sizes(&reference, "en");
    assert_eq!((universe, typical[2]), (25_372, 3_359), "en-fr, B1");
    assert_eq!(sizes(&other, "en"), (universe, typical));

    // *A sense part of speech the first pack never used*: no en-fr run is tagged NUM; en-es tags
    // the numbers' runs NUM (the Spanish Wiktionary's numeral sections, kaikki's `num`), and the
    // core reads a numeral's runs as the table writes them — « three », a number and a noun.
    assert!(
        reference_inputs
            .senses
            .iter()
            .all(|(_, runs)| runs.iter().all(|(tag, _)| tag != "NUM"))
    );
    let (_, runs) = other_inputs
        .senses
        .iter()
        .find(|(lemma, _)| lemma == "three")
        .expect("three's sense runs");
    let tags: Vec<&str> = runs.iter().map(|(tag, _)| tag.as_str()).collect();
    assert_eq!(tags, ["NUM", "NOUN"], "three, in tables/en-es/senses.tsv");
    let read: Vec<String> = en_es
        .sense_runs("three")
        .into_iter()
        .map(|(tag, _)| tag.unwrap().to_ud())
        .collect();
    assert_eq!(read, tags, "three, as the core reads it");
}

#[test]
fn spec_scenario_spanish_through_another_native_language() {
    // es-fr, and es-en: the committed pair glossed in English (add-lingua-pack-es-en), both
    // built from tables/es/ — the real second pack, not a synthetic one.
    let root = Scenario::tables_root();
    let (reference_inputs, other_inputs) = (
        inputs_from_tables(&root, "es-fr").expect("read es-fr"),
        inputs_from_tables(&root, "es-en").expect("read es-en"),
    );
    assert_eq!(
        other_inputs.meta.native, "en",
        "es-en is glossed in English"
    );
    let (reference, other) = (
        build_pack(&reference_inputs).expect("es-fr"),
        build_pack(&other_inputs).expect("es-en"),
    );
    assert_studied_sections_alike("es-fr", &reference, &other);
    assert_probes_alike(&SPANISH, Some("es"), &reference, &other, "fr", "en");

    let (es_fr, es_en) = (Pack::load(&reference).unwrap(), Pack::load(&other).unwrap());
    assert_eq!(es_en.pair().key(), "es-en");
    // *A pack glossed in English*: `casa` is one of its dictionary words, glossed in English,
    // not as es-fr glosses it.
    assert!(es_en.is_dictionary_word("casa"));
    assert_ne!(es_en.gloss("casa"), es_fr.gloss("casa"));
    assert!(es_en.gloss("casa").is_some_and(|g| g.contains("house")));
    // A lemma this pack glosses and es-fr does not is no dictionary word: the dictionary words
    // are es-fr's, read from tables/es (the lexical section `assert_studied_sections_alike`
    // found).
    let only_here: Vec<&str> = other_inputs
        .glosses
        .iter()
        .map(|(lemma, _)| lemma.as_str())
        .filter(|lemma| es_fr.gloss(lemma).is_none())
        .take(8)
        .collect();
    assert!(!only_here.is_empty(), "es-en glosses lemmas es-fr does not");
    for lemma in &only_here {
        assert!(
            es_en.gloss(lemma).is_some() && !es_en.is_dictionary_word(lemma),
            "{lemma}"
        );
    }
    // *Spanish glossed in another language*: the estimate's universe is es-fr's.
    assert_eq!(sizes(&reference, "es").0, 22_755);
    assert_eq!(sizes(&other, "es").0, 22_755);

    // *A pack glossed in English* (names): Augusto and Eugenia are set aside as with es-fr.
    let names = |pack: &[u8]| -> Vec<String> {
        let engine = Scenario::engine(&[("pack", pack.to_vec())]);
        let page: serde_json::Value = serde_json::from_str(
            &engine
                .analyse(
                    vec![
                        "Augusto miró a Eugenia y pensó en la Nela.".to_owned(),
                        "Entonces Augusto salió de la casa.".to_owned(),
                    ],
                    Some("es".to_owned()),
                )
                .unwrap(),
        )
        .unwrap();
        page["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| matches!(t["surface"].as_str(), Some("Augusto" | "Eugenia")))
            .map(|t| t["class"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(names(&reference), ["ProperNounOutOfLexicon"; 3]);
    assert_eq!(names(&other), names(&reference));

    // *Glosses that say no gender*: es-en's sense table carries none (its reducer computes
    // nothing of the studied side, D1), and the noun runs read the readings' gender.
    let heading = |pack: &Pack, lemma: &str| -> Vec<String> {
        pack.sense_runs(lemma)
            .into_iter()
            .map(|(tag, _)| tag.unwrap().to_ud())
            .collect()
    };
    assert!(
        other_inputs
            .senses
            .iter()
            .all(|(_, runs)| runs.iter().all(|(tag, _)| !tag.contains("Gender")))
    );
    assert_eq!(heading(&es_en, "casa")[0], "NOUN|Gender=Fem");
    assert_eq!(heading(&es_fr, "casa")[0], "NOUN|Gender=Fem");
    // *A noun of both genders*: no gender.
    assert_eq!(heading(&es_en, "estudiante")[0], "NOUN");
    assert_eq!(heading(&es_fr, "estudiante")[0], "NOUN");
    // The parts of speech of es-en's runs are the English Wiktionary's, laid out after the
    // pinned pool (`assert_studied_sections_alike`); every one is a Universal Dependencies tag.
    assert!(other_inputs.senses.iter().all(|(_, runs)| {
        runs.iter()
            .all(|(tag, _)| tag.chars().all(|c| c.is_ascii_uppercase()))
    }));
}

#[test]
fn spec_scenario_french_through_another_native_language() {
    // fr-en, French's reference, and fr-es: the committed pair glossed in Spanish
    // (add-lingua-pack-fr-es D10), both built from tables/fr/ — the real second pack. The scenario's
    // engine holds the one pack, so its native language is the pack's (`render_with`): fr-en's
    // cards are labelled `en`, fr-es's `es`.
    let root = Scenario::tables_root();
    let (reference_inputs, other_inputs) = (
        inputs_from_tables(&root, "fr-en").expect("read fr-en"),
        inputs_from_tables(&root, "fr-es").expect("read fr-es"),
    );
    assert_eq!(
        other_inputs.meta.native, "es",
        "fr-es is glossed in Spanish"
    );
    // *Glossed in Spanish with fewer glosses*.
    assert!(other_inputs.glosses.len() < reference_inputs.glosses.len());
    let (reference, other) = (
        build_pack(&reference_inputs).expect("fr-en"),
        build_pack(&other_inputs).expect("fr-es"),
    );
    let (fr_en, fr_es) = (Pack::load(&reference).unwrap(), Pack::load(&other).unwrap());
    assert_eq!(fr_es.pair().key(), "fr-es", "glossed in Spanish");
    // French's levels (add-lingua-french-levels) and paradigms (add-lingua-french-grammar-tables)
    // are among the sections compared: both have landed.
    assert_studied_sections_alike("fr-en", &reference, &other);
    assert_probes_alike(&FRENCH, Some("fr"), &reference, &other, "en", "es");
    // A lemma fr-es glosses and fr-en does not is no dictionary word: the dictionary words are
    // fr-en's, read from tables/fr (`quant`, which the Spanish Wiktionary glosses).
    assert!(fr_en.gloss("quant").is_none());
    assert!(fr_es.gloss("quant").is_some() && !fr_es.is_dictionary_word("quant"));
    let only_here: Vec<&str> = other_inputs
        .glosses
        .iter()
        .map(|(lemma, _)| lemma.as_str())
        .filter(|lemma| fr_en.gloss(lemma).is_none())
        .collect();
    assert!(only_here.len() > 100, "{}", only_here.len());
    for lemma in &only_here {
        assert!(!fr_es.is_dictionary_word(lemma), "{lemma}");
    }
    assert_eq!(fr_es.gloss("maison"), Some("Casa"));
    assert!(fr_es.is_dictionary_word("maison"));
    // *A vocabulary size counts dictionary words*: the universe and the ladder are fr-en's.
    assert_eq!(sizes(&other, "fr"), sizes(&reference, "fr"));
}

/// How many cards a seeding asks for in `seeding_follows_each_packs_glosses`: the engine's cap,
/// the extension's `SEED_CAP`.
const SEEDED: usize = 50;

/// A level's lemmas in the order `seedLevel` takes them: by frequency rank, commonest or rarest
/// first, unranked last, ties in lemma-id order — read from `pack`'s studied sections.
fn level_in_order<'p>(pack: &'p Pack, level: CefrLevel, order: &str) -> Vec<&'p str> {
    let mut lemmas: Vec<&str> = pack
        .lemmas_at_level(level)
        .into_iter()
        .map(|(lemma, _)| lemma)
        .collect();
    // Each rank read once: a lookup goes through the lexicon.
    if order == "rare" {
        lemmas.sort_by_cached_key(|l| {
            let rank = pack.rank(l);
            (rank.is_none(), Reverse(rank.unwrap_or(0)))
        });
    } else {
        lemmas.sort_by_cached_key(|l| {
            let rank = pack.rank(l);
            (rank.is_none(), rank.unwrap_or(0))
        });
    }
    lemmas
}

/// The deck's cards, as the engine exports them for sync.
fn card_ops(engine: &lingua_wasm::LinguaEngine) -> Vec<serde_json::Value> {
    serde_json::from_str(&engine.export_card_ops()).expect("an array of card operations")
}

/// Through `bytes` (whose loaded pack is `pack`), from a fresh deck, one seeding of each level in
/// each order: the cards are the first [`SEEDED`] lemmas of the level, in that order, that the pack
/// glosses — `order` read from `reference`, whose studied sections every pack of the language
/// shares —, each with the pack's gloss and labelled with its native language, the reader's level
/// ladder and estimate untouched. Answers each level-order's cards, and how many lemmas the gloss
/// filter passed over among the [`SEEDED`] the seeding would have drawn without it.
fn seedings(
    language: &str,
    reference: &Pack,
    pack: &Pack,
    bytes: &[u8],
) -> (Vec<BTreeSet<String>>, usize) {
    let lang = || Some(language.to_owned());
    let native = pack.native().tag();
    // One engine, its state restored to a fresh reader's before each seeding: loading a pack is
    // the slow part, and a fresh state is a fresh deck.
    let mut engine = Scenario::engine(&[("pack", bytes.to_vec())]);
    let fresh = engine.backup();
    let (ladder, estimate) = (
        engine.level_ladder(lang()).unwrap(),
        engine.vocabulary_estimate(lang()).unwrap(),
    );
    let mut cards = Vec::new();
    let mut skipped = 0;
    for level in CefrLevel::ALL {
        for order in ["common", "rare"] {
            let what = format!("{} {} {order}", pack.pair().key(), level.label());
            let lemmas = level_in_order(reference, level, order);
            skipped += lemmas
                .iter()
                .take(SEEDED)
                .filter(|l| pack.gloss(l).is_none())
                .count();
            let expected: BTreeSet<String> = lemmas
                .iter()
                .filter(|l| pack.gloss(l).is_some())
                .take(SEEDED)
                .map(|l| (*l).to_owned())
                .collect();
            // *Seeded cards through another native language*: as many as asked — every level of
            // every pair holds more glossed lemmas than a seeding asks for.
            assert_eq!(expected.len(), SEEDED, "{what}: glossed lemmas");
            if engine.restore(&fresh).is_err() {
                panic!("{what}: a fresh reader's state restores");
            }
            assert_eq!(
                engine
                    .seed_level(level.label(), SEEDED, order, 1.0, lang())
                    .unwrap(),
                SEEDED,
                "{what}: added"
            );
            let ops = card_ops(&engine);
            let seeded: BTreeSet<String> = ops
                .iter()
                .map(|op| op["lemma"].as_str().unwrap().to_owned())
                .collect();
            assert_eq!(
                seeded, expected,
                "{what}: the lemmas the pack glosses, in order"
            );
            for op in &ops {
                let lemma = op["lemma"].as_str().unwrap();
                assert_eq!(op["gloss"].as_str(), pack.gloss(lemma), "{what}: {op}");
                if native == "fr" {
                    // A French gloss carries no label (add-lingua-card-gloss-language D2).
                    assert!(op.get("gloss_language").is_none(), "{what}: {op}");
                } else {
                    assert_eq!(op["gloss_language"], native, "{what}: {op}");
                }
            }
            // A seeding writes no status: the ladder and the estimate do not move.
            assert_eq!(engine.export_status_ops(), "[]", "{what}: statuses");
            assert_eq!(engine.level_ladder(lang()).unwrap(), ladder, "{what}");
            assert_eq!(
                engine.vocabulary_estimate(lang()).unwrap(),
                estimate,
                "{what}"
            );
            cards.push(seeded);
        }
    }
    (cards, skipped)
}

/// [`seedings`] through a language's two packs, built from the committed tables: each pack's cards
/// and the lemmas its gloss filter passed over, then the two packs' bytes.
fn seed_through(
    language: &str,
    reference_pair: &str,
    other_pair: &str,
) -> (
    (Vec<BTreeSet<String>>, usize),
    (Vec<BTreeSet<String>>, usize),
    Vec<u8>,
    Vec<u8>,
) {
    let root = Scenario::tables_root();
    let build = |pair: &str| -> Vec<u8> {
        build_pack(&inputs_from_tables(&root, pair).unwrap_or_else(|e| panic!("{pair}: {e}")))
            .unwrap_or_else(|e| panic!("build {pair}: {e}"))
    };
    let (reference_bytes, other_bytes) = (build(reference_pair), build(other_pair));
    let (reference, other) = (
        Pack::load(&reference_bytes).unwrap(),
        Pack::load(&other_bytes).unwrap(),
    );
    let a = seedings(language, &reference, &reference, &reference_bytes);
    let b = seedings(language, &reference, &other, &other_bytes);
    (a, b, reference_bytes, other_bytes)
}

#[test]
fn seeding_follows_each_packs_glosses() {
    // *Seeded cards through another native language* (seed-lingua-decks-with-glossed-lemmas D4):
    // each language's reference pack and the real second one, built from the committed tables —
    // the three languages side by side, building a pack being the slow part.
    let languages = [
        ("en", "en-fr", "en-es"),
        ("es", "es-fr", "es-en"),
        ("fr", "fr-en", "fr-es"),
    ];
    let checked: Vec<_> = std::thread::scope(|scope| {
        let threads: Vec<_> = languages
            .iter()
            .map(|&(language, reference, other)| {
                scope.spawn(move || seed_through(language, reference, other))
            })
            .collect();
        threads
            .into_iter()
            .map(|thread| {
                thread
                    .join()
                    .unwrap_or_else(|e| std::panic::resume_unwind(e))
            })
            .collect()
    });
    let mut french = None;
    for ((language, reference_pair, other_pair), ((a, skipped_a), (b, skipped_b), r, o)) in
        languages.into_iter().zip(checked)
    {
        // The measure the design quotes (*What the tests show*), not pinned: a gloss update moves it.
        println!(
            "{language}: lemmas without a gloss among the first {SEEDED} of the 12 level-orders, \
             {reference_pair} {skipped_a}, {other_pair} {skipped_b}"
        );
        match language {
            // *A pack that glosses every levelled lemma*: es-fr seeds what it always seeded.
            "es" => assert_eq!(skipped_a, 0, "es-fr glosses every levelled lemma"),
            // The C1 lemmas en-fr seeds rarest first are not en-es's: C1 is the fifth level,
            // `rare` its second order.
            "en" => assert_ne!(a[4 * 2 + 1], b[4 * 2 + 1], "C1, rarest first"),
            _ => french = Some((r, o)),
        }
    }

    // *A French level with no Spanish gloss*: seeded until it is spent, A1 gives `part` a card
    // through fr-en and none through fr-es, which does not gloss it; `part` keeps its level, its
    // place in the A1 row of the ladder and in the estimate, and gets no status. The last seeding
    // adds nothing (*Only lemmas without a gloss are left*).
    let (fr_en, fr_es) = french.expect("the French packs");
    let fr = || Some("fr".to_owned());
    let spent = |bytes: &[u8]| -> (Vec<serde_json::Value>, usize) {
        let mut engine = Scenario::engine(&[("pack", bytes.to_vec())]);
        let (ladder, estimate) = (
            engine.level_ladder(fr()).unwrap(),
            engine.vocabulary_estimate(fr()).unwrap(),
        );
        let mut added = 0;
        loop {
            match engine
                .seed_level("A1", SEEDED, "common", 1.0, fr())
                .unwrap()
            {
                0 => break,
                n => added += n,
            }
        }
        assert_eq!(engine.export_status_ops(), "[]");
        assert_eq!(engine.level_ladder(fr()).unwrap(), ladder);
        assert_eq!(engine.vocabulary_estimate(fr()).unwrap(), estimate);
        (card_ops(&engine), added)
    };
    let carded = |ops: &[serde_json::Value], lemma: &str| ops.iter().any(|op| op["lemma"] == lemma);
    let (en_pack, es_pack) = (Pack::load(&fr_en).unwrap(), Pack::load(&fr_es).unwrap());
    assert_eq!(en_pack.level("part"), Some(CefrLevel::A1));
    assert!(en_pack.gloss("part").is_some(), "fr-en glosses `part`");
    assert!(
        es_pack.gloss("part").is_none(),
        "fr-es glosses `part` now: name another A1 lemma it leaves out"
    );
    let a1 = |pack: &Pack| {
        pack.lemmas_at_level(CefrLevel::A1)
            .iter()
            .filter(|(_, gloss)| gloss.is_some())
            .count()
    };
    let (ops_en, added_en) = spent(&fr_en);
    let (ops_es, added_es) = spent(&fr_es);
    assert_eq!((added_en, added_es), (a1(&en_pack), a1(&es_pack)));
    assert!(carded(&ops_en, "part"));
    assert!(!carded(&ops_es, "part"));
    assert_eq!(es_pack.level("part"), Some(CefrLevel::A1), "its level kept");
}
