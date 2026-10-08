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

//! The es-en baseline (`docs/lingua/language-matrix-programme.md`, change 23 —
//! add-lingua-english-card-wording D1): what an English-native reader of Spanish receives from
//! the engine, over the real es-en pack built from the committed tables (`tables/es/` and
//! `tables/es-en/`, add-lingua-pack-es-en).
//!
//! The scenario is the Spanish baseline's (`support/spanish.rs`: its corpus, lemmas, phrases,
//! grammar probes and reader), the engine holding es-en alone — an engine holds one native's
//! packs, and en-fr is French-native — and 40 more lemmas asked as `word-grammar <lemma> <lemma>`
//! probes after the reference's, so that their glosses and sense runs, genders included, are the
//! engine's answer and not the table's line. The extension's `test/word-card-es-en.spec.ts` reads
//! every grammar probe of `baseline/es-en.golden` and pins the English lines the card renders from
//! them; nothing there reads the tables.
//!
//! On the probes they share, this golden and es-fr's differ only in the native side — glosses,
//! senses, expressions, the lines naming the packs, the notice, the licences and the backup's
//! profile: `the_golden_is_the_spanish_one_on_the_studied_side` compares the two committed goldens
//! through `support::studied_side`, as `cross_native.rs` compares the engines.
//!
//! A pull request that changes `baseline/es-en.golden` changes what English-native readers of
//! Spanish will see. Only a dictionary update of es-en or of es-fr (which moves the studied
//! tables), or a deliberate analyser change that bumps the Spanish analyser version, should do
//! that. Re-bless with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test es_en_baseline` — then
//! the extension's snapshot, `yarn vitest -u test/word-card-es-en.spec.ts` — and say why in the
//! pull request; `lingua-pack-update` re-blesses both on its own branch.
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use support::spanish::SPANISH;
use support::{Card, Scenario, first_difference, probes, studied_side};

/// 40 more dictionary forms asked as grammar probes, `word-grammar <lemma> <lemma>`: the most
/// frequent lemmas of `tables/es/freq.tsv` whose es-en gloss has two sense runs or more
/// (`tables/es-en/senses.tsv`), taken in the frequency order, `como` left out since the
/// reference already asks it as itself. Chosen once, on the tables of 2026-10-08: a re-reduction
/// may give one of them a single run, which moves nothing but what the golden says of it.
const MORE_LEMMAS: &[&str] = &[
    "que", "a", "no", "por", "para", "su", "más", "pero", "o", "este", "todo", "ya", "yo", "ser",
    "sobre", "qué", "dos", "bien", "hasta", "hacer", "ahora", "nada", "ni", "donde", "solo",
    "algo", "mucho", "tan", "ese", "mismo", "cada", "estado", "año", "otro", "menos", "tanto",
    "antes", "poco", "cómo", "hecho",
];

/// The reference's grammar probes, then the 40 lemmas as themselves.
const GRAMMAR: [(&str, &str); SPANISH.grammar.len() + MORE_LEMMAS.len()] = {
    let mut out = [("", ""); SPANISH.grammar.len() + MORE_LEMMAS.len()];
    let mut i = 0;
    while i < SPANISH.grammar.len() {
        out[i] = SPANISH.grammar[i];
        i += 1;
    }
    let mut j = 0;
    while j < MORE_LEMMAS.len() {
        out[i + j] = (MORE_LEMMAS[j], MORE_LEMMAS[j]);
        j += 1;
    }
    out
};

/// The first page of es-en's gloss of « viaje »: the whole gloss, which fits one page.
const VIAJE_FIRST_PAGE: &str = "voyage; journey, trip; A state of hallucination or altered consciousness caused by a narcotic drug; a lot, loads";

/// The Spanish scenario, glossed in English: es-en alone, the reference's cards with the shown
/// gloss the es-en pack's first page for its lemma, and the 40 more grammar probes.
pub const SPANISH_IN_ENGLISH: Scenario = Scenario {
    pair: "es-en",
    beside: &[],
    test: "es_en_baseline",
    grammar: &GRAMMAR,
    cards: [
        Card {
            lemma: "viaje",
            form: "viaje",
            sentence: "Ella dirigirá el viaje.",
            url: "https://example.com/homografos",
            gloss: Some(VIAJE_FIRST_PAGE),
        },
        Card {
            lemma: "retraso",
            form: "retraso",
            sentence: "Tenemos que ocuparnos del retraso.",
            url: "https://example.com/noticias",
            gloss: None,
        },
        Card {
            lemma: "sonreír",
            form: "sonriendo",
            sentence: "Entró en la sala sonriendo.",
            url: "",
            gloss: None,
        },
    ],
    ..SPANISH
};

#[test]
fn es_en_output_has_not_moved() {
    let actual = SPANISH_IN_ENGLISH.render(Some("es"));
    let Some(expected) = SPANISH_IN_ENGLISH.bless_or_read(&actual) else {
        return;
    };
    assert!(
        actual == expected,
        "es-en output moved (docs/lingua/language-matrix-programme.md, change 23: the card of \
         English-native readers of Spanish is pinned on the real pack).\n\
         First difference — {}\n\
         If this pull request means to change es-en output (a dictionary update of es-en or of \
         es-fr, or an analyser change that bumps the Spanish analyser version), re-bless with\n  \
         LINGUA_BLESS=1 cargo test -p lingua-wasm --test es_en_baseline\n  \
         cd apps/lingua-extension && yarn vitest -u test/word-card-es-en.spec.ts\n\
         and say why in the pull request. Otherwise the change is wrong.",
        first_difference(&expected, &actual)
    );
}

#[test]
fn the_corpus_reads_as_expected() {
    SPANISH_IN_ENGLISH.check_corpus();
}

/// The probes are the reference's, each once, then the 40 lemmas, each once and none of them a
/// probe the reference already asks.
#[test]
fn the_probes_are_the_references_and_forty_more() {
    let (reference, more) = GRAMMAR.split_at(SPANISH.grammar.len());
    assert_eq!(reference, SPANISH.grammar);
    assert_eq!(more.len(), 40);
    for (written, lemma) in more {
        assert_eq!(written, lemma);
        assert!(
            !reference.contains(&(written, lemma)),
            "{lemma} is a probe of the reference"
        );
    }
    let mut lemmas: Vec<&str> = more.iter().map(|(_, lemma)| *lemma).collect();
    lemmas.sort_unstable();
    lemmas.dedup();
    assert_eq!(lemmas.len(), 40, "each lemma once");
}

/// The card's shown gloss is the es-en pack's first page for its lemma: whole senses, up to
/// 160 characters (the extension's `glossPages`), which for « viaje » is the whole gloss.
#[test]
fn the_cards_gloss_is_the_packs_first_page() {
    let engine = SPANISH_IN_ENGLISH.loaded();
    let gloss = engine
        .gloss("viaje", Some("es".to_owned()))
        .unwrap()
        .expect("es-en glosses viaje");
    let mut page = String::new();
    for sense in gloss.split("; ") {
        if !page.is_empty() && page.len() + 2 + sense.len() > 160 {
            break;
        }
        if !page.is_empty() {
            page.push_str("; ");
        }
        page.push_str(sense);
    }
    assert_eq!(page, VIAJE_FIRST_PAGE);
    assert_eq!(page, gloss, "one page");
}

/// Spanish levels are derived from frequency (add-lingua-spanish-levels, D1), whatever the
/// native language: the real es-en pack calls them estimated too.
#[test]
fn es_en_levels_are_estimated() {
    let engine = SPANISH_IN_ENGLISH.loaded();
    let es = || Some("es".to_owned());
    assert!(engine.has_levels(es()).expect("levels"));
    assert!(engine.levels_estimated(es()).expect("estimated"));
}

/// The committed es-en golden is the committed es-fr golden on the studied side (D1): on every
/// probe they share, the two differ only in the native side — glosses, senses, expressions —
/// the lines naming the packs (es-fr's engine holds en-fr beside it, this one nothing), the
/// notice, the licences and the backup's profile. The 40 more probes are es-en's alone.
#[test]
fn the_golden_is_the_spanish_one_on_the_studied_side() {
    // The goldens are being rewritten by the baseline tests, in this binary and in
    // `spanish_baseline`: nothing to compare until both are blessed.
    if std::env::var_os("LINGUA_BLESS").is_some() {
        return;
    }
    let (es_fr, es_en) = (
        probes(&SPANISH.committed_golden()),
        probes(&SPANISH_IN_ENGLISH.committed_golden()),
    );
    let shared: Vec<&str> = es_fr
        .iter()
        .map(|(name, _)| name.as_str())
        .filter(|name| !name.starts_with("beside "))
        .collect();
    let own: Vec<&str> = es_en
        .iter()
        .map(|(name, _)| name.as_str())
        .filter(|name| !shared.contains(name))
        .collect();
    assert_eq!(
        own,
        MORE_LEMMAS
            .iter()
            .map(|lemma| format!("word-grammar {lemma} {lemma}"))
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        "es-en's own probes are the 40 lemmas"
    );
    let mut compared = 0;
    for (name, body) in &es_fr {
        // The `about` line names the test that generated the golden.
        if name == "about" {
            continue;
        }
        let Some((_, other)) = es_en.iter().find(|(n, _)| n == name) else {
            assert!(name.starts_with("beside "), "es-en answers `{name}` too");
            continue;
        };
        let (Some(x), Some(y)) = (
            studied_side(name, body, true),
            studied_side(name, other, true),
        ) else {
            continue;
        };
        compared += 1;
        let at = x.chars().zip(y.chars()).take_while(|(p, q)| p == q).count();
        assert!(
            x == y,
            "probe `{name}` differs between es-fr.golden and es-en.golden beyond the native side, \
             character {at}:\n  es-fr: …{}…\n  es-en: …{}…",
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
    // Every shared probe but the `about` and pack lines, the cards' glosses, the notice and the
    // licences.
    assert_eq!(
        compared,
        shared.len() - 2 - 2 - SPANISH.lemmas.len(),
        "probes compared"
    );
}
