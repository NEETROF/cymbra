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

//! The en-es baseline (`docs/lingua/language-matrix-programme.md`, change 24 —
//! add-lingua-spanish-card-wording D1): what a Spanish-native reader of English receives from
//! the engine, over the real en-es pack built from the committed tables (`tables/en/` and
//! `tables/en-es/`, add-lingua-pack-en-es).
//!
//! The scenario is the English baseline's (`support/english.rs`: its corpus, lemmas, phrases,
//! grammar probes and reader), the engine holding en-es alone — an engine holds one native's
//! packs, and en-fr is French-native — and 40 more lemmas asked as `word-grammar <lemma> <lemma>`
//! probes after the reference's, so that their glosses and sense runs are the engine's answer and
//! not the table's line. The extension's `test/word-card-en-es.spec.ts` reads every grammar probe
//! of `baseline/en-es.golden` and pins the Spanish lines the card renders from them; nothing there
//! reads the tables.
//!
//! On the probes they share, up to the reader's level seeding, this golden and en-fr's differ only
//! in the native side — glosses, senses, expressions, the line naming the pack, the notice, the
//! licences and the backup's profile: `the_golden_is_the_english_one_on_the_studied_side` compares
//! the two committed goldens through `support::studied_side`, as `cross_native.rs` compares the
//! engines. The probes from `start-review` on follow the deck the seeding left, whose cards are the
//! lemmas each pack glosses (seed-lingua-decks-with-glossed-lemmas D4): `cross_native.rs` compares
//! them through engines, the reader answered without the seeding, and checks the seeding apart.
//!
//! A pull request that changes `baseline/en-es.golden` changes what Spanish-native readers of
//! English will see. Only a dictionary update of en-es or of en-fr (which moves the studied
//! tables), or a deliberate analyser change that bumps the English analyser version, should do
//! that. Re-bless with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test en_es_baseline` — then
//! the extension's snapshot, `yarn vitest run test/word-card-en-es.spec.ts -u` — and say why in
//! the pull request; `lingua-pack-update` re-blesses both on its own branch.
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use support::english::ENGLISH;
use support::{Card, Scenario, first_difference, follows_seeding, probes, studied_side};

/// 40 more dictionary forms asked as grammar probes, `word-grammar <lemma> <lemma>`: the most
/// frequent lemmas of `tables/en/freq.tsv` whose en-es gloss has two sense runs or more
/// (`tables/en-es/senses.tsv`), taken in the frequency order. None of them is a probe the
/// reference asks as itself (`read`, `lead`, `more`, `thought`, `lay`, `saw` rank lower or have
/// one run). Chosen once, on the tables of 2026-10-08: a re-reduction may give one of them a
/// single run, which moves nothing but what the golden says of it.
const MORE_LEMMAS: &[&str] = &[
    "to", "a", "in", "for", "that", "have", "by", "but", "or", "all", "one", "can", "will", "just",
    "like", "about", "up", "out", "no", "her", "time", "new", "some", "now", "other", "its",
    "good", "only", "after", "first", "see", "over", "think", "any", "back", "want", "go", "well",
    "way", "much",
];

/// The reference's grammar probes, then the 40 lemmas as themselves.
const GRAMMAR: [(&str, &str); ENGLISH.grammar.len() + MORE_LEMMAS.len()] = {
    let mut out = [("", ""); ENGLISH.grammar.len() + MORE_LEMMAS.len()];
    let mut i = 0;
    while i < ENGLISH.grammar.len() {
        out[i] = ENGLISH.grammar[i];
        i += 1;
    }
    let mut j = 0;
    while j < MORE_LEMMAS.len() {
        out[i + j] = (MORE_LEMMAS[j], MORE_LEMMAS[j]);
        j += 1;
    }
    out
};

/// The first page of en-es's gloss of « expedition »: the whole gloss, which fits one page.
const EXPEDITION_FIRST_PAGE: &str = "Expedición";

/// The English scenario, glossed in Spanish: en-es alone, the reference's cards with the shown
/// gloss the en-es pack's first page for its lemma, and the 40 more grammar probes.
pub const ENGLISH_IN_SPANISH: Scenario = Scenario {
    pair: "en-es",
    beside: &[],
    test: "en_es_baseline",
    grammar: &GRAMMAR,
    cards: [
        Card {
            lemma: "expedition",
            form: "expedition",
            sentence: "She will lead the expedition.",
            url: "https://example.com/homographs",
            gloss: Some(EXPEDITION_FIRST_PAGE),
        },
        Card {
            lemma: "backlog",
            form: "backlog",
            sentence: "We really need to take care of the backlog.",
            url: "https://example.com/phrasal",
            gloss: None,
        },
        Card {
            lemma: "grin",
            form: "grinning",
            sentence: "He couldn't stop grinning.",
            url: "",
            gloss: None,
        },
    ],
    ..ENGLISH
};

#[test]
fn en_es_output_has_not_moved() {
    // The extension names the language on every language-bound call
    // (generalise-lingua-extension-port); `english_baseline` holds naming `en` to naming none.
    let actual = ENGLISH_IN_SPANISH.render(Some("en"));
    let Some(expected) = ENGLISH_IN_SPANISH.bless_or_read(&actual) else {
        return;
    };
    assert!(
        actual == expected,
        "en-es output moved (docs/lingua/language-matrix-programme.md, change 24: the card of \
         Spanish-native readers of English is pinned on the real pack).\n\
         First difference — {}\n\
         If this pull request means to change en-es output (a dictionary update of en-es or of \
         en-fr, or an analyser change that bumps the English analyser version), re-bless with\n  \
         LINGUA_BLESS=1 cargo test -p lingua-wasm --test en_es_baseline\n  \
         cd apps/lingua-extension && yarn vitest run test/word-card-en-es.spec.ts -u\n\
         and say why in the pull request. Otherwise the change is wrong.",
        first_difference(&expected, &actual)
    );
}

#[test]
fn the_corpus_reads_as_expected() {
    ENGLISH_IN_SPANISH.check_corpus();
}

/// The probes are the reference's, each once, then the 40 lemmas, each once and none of them a
/// probe the reference already asks.
#[test]
fn the_probes_are_the_references_and_forty_more() {
    let (reference, more) = GRAMMAR.split_at(ENGLISH.grammar.len());
    assert_eq!(reference, ENGLISH.grammar);
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

/// The card's shown gloss is the en-es pack's first page for its lemma: whole senses, up to
/// 160 characters as the extension's `glossPages` counts them — UTF-16 code units, JavaScript's
/// `length` — which for « expedition » is the whole gloss.
#[test]
fn the_cards_gloss_is_the_packs_first_page() {
    let engine = ENGLISH_IN_SPANISH.loaded();
    let gloss = engine
        .gloss("expedition", Some("en".to_owned()))
        .unwrap()
        .expect("en-es glosses expedition");
    let units = |text: &str| text.encode_utf16().count();
    let mut page = String::new();
    for sense in gloss.split("; ") {
        if !page.is_empty() && units(&page) + 2 + units(sense) > 160 {
            break;
        }
        if !page.is_empty() {
            page.push_str("; ");
        }
        page.push_str(sense);
    }
    assert_eq!(page, EXPEDITION_FIRST_PAGE);
    assert_eq!(page, gloss, "one page");
}

/// English's levels come from CEFR-J and Octanove, whatever the native language: the real en-es
/// pack does not call them estimated.
#[test]
fn en_es_levels_are_not_estimated() {
    let engine = ENGLISH_IN_SPANISH.loaded();
    let en = || Some("en".to_owned());
    assert!(engine.has_levels(en()).expect("levels"));
    assert!(!engine.levels_estimated(en()).expect("estimated"));
}

/// The committed en-es golden is the committed en-fr golden on the studied side (D1): on every
/// probe they share before `start-review`, both seeding counts included, the two differ only in
/// the native side — glosses, senses, expressions — the line naming the pack, the notice, the
/// licences and the backup's profile. Both engines hold their pair alone. The 40 more probes are
/// en-es's alone. The probes after the seeding are `cross_native.rs`'s, through engines: the cards
/// a level seeds are those each pack glosses (seed-lingua-decks-with-glossed-lemmas D4).
#[test]
fn the_golden_is_the_english_one_on_the_studied_side() {
    // The goldens are being rewritten by the baseline tests, in this binary and in
    // `english_baseline`: nothing to compare until both are blessed.
    if std::env::var_os("LINGUA_BLESS").is_some() {
        return;
    }
    let (en_fr, en_es) = (
        probes(&ENGLISH.committed_golden()),
        probes(&ENGLISH_IN_SPANISH.committed_golden()),
    );
    let shared: Vec<&str> = en_fr.iter().map(|(name, _)| name.as_str()).collect();
    let own: Vec<&str> = en_es
        .iter()
        .map(|(name, _)| name.as_str())
        .filter(|name| !shared.contains(name))
        .collect();
    let more: Vec<String> = MORE_LEMMAS
        .iter()
        .map(|lemma| format!("word-grammar {lemma} {lemma}"))
        .collect();
    assert_eq!(own, more, "en-es's own probes are the 40 lemmas");
    let mut compared = 0;
    let after = shared
        .iter()
        .filter(|name| follows_seeding(&shared, name))
        .count();
    for (name, body) in &en_fr {
        // The `about` line names the test that generated the golden; the probes that follow the
        // reader's level seeding are compared through engines (cross_native.rs).
        if name == "about" || follows_seeding(&shared, name) {
            continue;
        }
        let (_, other) = en_es
            .iter()
            .find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("en-es answers `{name}` too"));
        let (Some(x), Some(y)) = (studied_side(name, body), studied_side(name, other)) else {
            continue;
        };
        compared += 1;
        let at = x.chars().zip(y.chars()).take_while(|(p, q)| p == q).count();
        assert!(
            x == y,
            "probe `{name}` differs between en-fr.golden and en-es.golden beyond the native side, \
             character {at}:\n  en-fr: …{}…\n  en-es: …{}…",
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
    // Every shared probe but the `about` and pack lines, the cards' glosses, the notice, the
    // licences and the probes that follow the seeding.
    assert!(after > 0, "the reader's probes from `start-review` on");
    assert_eq!(
        compared,
        shared.len() - 2 - 2 - ENGLISH.lemmas.len() - after,
        "probes compared"
    );
}
