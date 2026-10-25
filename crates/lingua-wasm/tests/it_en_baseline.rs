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

//! The it-en baseline (`docs/lingua/italian-programme.md`, change 22 — add-lingua-pack-it-en D9):
//! what an English-native reader of Italian receives from the engine, over the real it-en pack built
//! from the committed tables (`tables/it/` and `tables/it-en/`).
//!
//! The scenario is the Italian baseline's (`support/italian.rs`: its corpus, lemmas, phrases, grammar
//! probes — the reference's 32, the card's 24 and the 40 lemmas — and reader), the engine holding
//! it-en alone — an engine holds one native's packs, and en-fr, which the Italian baseline's engine
//! holds beside it-fr, is French-native. The extension's `test/word-card-it-en.spec.ts` reads every
//! grammar and phrase probe of `baseline/it-en.golden` and pins the English lines the card renders
//! from them (add-lingua-italian-word-card D8); nothing there reads the tables.
//!
//! On the probes they share, up to the reader's level seeding, this golden and it-fr's differ only in
//! the native side — glosses, senses, expressions, the lines naming the packs, the notice, the
//! licences and the backup's profile: `the_golden_is_the_italian_one_on_the_studied_side` compares the
//! two committed goldens through `support::studied_side`, as `cross_native.rs` compares the engines.
//!
//! A pull request that changes `baseline/it-en.golden` changes what English-native readers of Italian
//! will see. Only a dictionary update of it-en or of it-fr (which moves the studied tables), or a
//! deliberate analyser change that bumps the Italian analyser version, should do that — or a builder
//! change that moves the pack, and the probes a change adds, with the owner's approval. Re-bless with
//! `LINGUA_BLESS=1 cargo test -p lingua-wasm --test it_en_baseline` — then the extension's snapshot,
//! `yarn vitest run test/word-card-it-en.spec.ts -u` — and say why in the pull request;
//! `lingua-pack-update` re-blesses both on its own branch.
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use support::italian::ITALIAN;
use support::{PackSource, Scenario, first_difference, follows_seeding, probes, studied_side};

/// The first page of it-en's gloss of « orizzonte », the card the reader adds first: the whole gloss.
const ORIZZONTE_FIRST_PAGE: &str = "horizon";

/// The Italian scenario, glossed in English: it-en alone, over the committed tables, its cards'
/// shown gloss the it-en pack's first page for their lemma.
pub const ITALIAN_IN_ENGLISH: Scenario = Scenario {
    pair: "it-en",
    pack: PackSource::Tables,
    beside: &[],
    test: "it_en_baseline",
    ..ITALIAN
};

#[test]
fn it_en_output_has_not_moved() {
    let actual = ITALIAN_IN_ENGLISH.render(Some("it"));
    let Some(expected) = ITALIAN_IN_ENGLISH.bless_or_read(&actual) else {
        return;
    };
    assert!(
        actual == expected,
        "it-en output moved (docs/lingua/italian-programme.md, change 22: the card of English-native \
         readers of Italian is pinned on the real pack).\n\
         First difference — {}\n\
         If this pull request means to change it-en output (a dictionary update of it-en or of it-fr, \
         or an analyser change that bumps the Italian analyser version), re-bless with\n  \
         LINGUA_BLESS=1 cargo test -p lingua-wasm --test it_en_baseline\n  \
         cd apps/lingua-extension && yarn vitest run test/word-card-it-en.spec.ts -u\n\
         and say why in the pull request. Otherwise the change is wrong.",
        first_difference(&expected, &actual)
    );
}

#[test]
fn the_corpus_reads_as_expected() {
    ITALIAN_IN_ENGLISH.check_corpus();
}

/// The card's shown gloss is the it-en pack's first page for its lemma: whole senses, up to 160
/// characters as the extension's `glossPages` counts them — UTF-16 code units — which for
/// « orizzonte » is the whole gloss.
#[test]
fn the_cards_gloss_is_the_packs_first_page() {
    assert_eq!(ITALIAN_IN_ENGLISH.cards[0].lemma, "orizzonte");
    assert_eq!(
        ITALIAN_IN_ENGLISH.cards[0].gloss,
        Some(ORIZZONTE_FIRST_PAGE)
    );
    let engine = ITALIAN_IN_ENGLISH.loaded();
    let gloss = engine
        .gloss("orizzonte", Some("it".to_owned()))
        .unwrap()
        .expect("it-en glosses orizzonte");
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
    assert_eq!(page, ORIZZONTE_FIRST_PAGE);
    assert_eq!(page, gloss, "one page");
}

/// Italian's levels are derived from frequency (add-lingua-italian-levels D3), whatever the native
/// language: the real it-en pack calls them estimated too.
#[test]
fn it_en_levels_are_estimated() {
    let engine = ITALIAN_IN_ENGLISH.loaded();
    let it = || Some("it".to_owned());
    assert!(engine.has_levels(it()).expect("levels"));
    assert!(engine.levels_estimated(it()).expect("estimated"));
}

/// The committed it-en golden is the committed it-fr golden on the studied side (D9): on every probe
/// they share — every probe of it-en's — the two differ only in the native side — glosses, senses,
/// expressions —, the lines naming the packs (it-fr's engine holds en-fr beside it, this one
/// nothing), the notice, the licences and the backup's profile.
#[test]
fn the_golden_is_the_italian_one_on_the_studied_side() {
    // The goldens are being rewritten by the baseline tests, in this binary and in
    // `italian_baseline`: nothing to compare until both are blessed.
    if std::env::var_os("LINGUA_BLESS").is_some() {
        return;
    }
    let (it_fr, it_en) = (
        probes(&ITALIAN.committed_golden()),
        probes(&ITALIAN_IN_ENGLISH.committed_golden()),
    );
    let shared: Vec<&str> = it_fr
        .iter()
        .map(|(name, _)| name.as_str())
        .filter(|name| !name.starts_with("beside "))
        .collect();
    let own: Vec<&str> = it_en
        .iter()
        .map(|(name, _)| name.as_str())
        .filter(|name| !shared.contains(name))
        .collect();
    assert!(
        own.is_empty(),
        "it-en asks no probe it-fr does not: {own:?}"
    );
    let mut compared = 0;
    let after = shared
        .iter()
        .filter(|name| follows_seeding(&shared, name))
        .count();
    for (name, body) in &it_fr {
        // The `about` line names the test that generated the golden; the probes that follow the
        // reader's level seeding are compared through engines (cross_native.rs).
        if name == "about" || follows_seeding(&shared, name) {
            continue;
        }
        let Some((_, other)) = it_en.iter().find(|(n, _)| n == name) else {
            assert!(name.starts_with("beside "), "it-en answers `{name}` too");
            continue;
        };
        let (Some(x), Some(y)) = (studied_side(name, body), studied_side(name, other)) else {
            continue;
        };
        compared += 1;
        let at = x.chars().zip(y.chars()).take_while(|(p, q)| p == q).count();
        assert!(
            x == y,
            "probe `{name}` differs between it-fr.golden and it-en.golden beyond the native side, \
             character {at}:\n  it-fr: …{}…\n  it-en: …{}…",
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
        shared.len() - 2 - 2 - ITALIAN.lemmas.len() - after,
        "probes compared"
    );
}
