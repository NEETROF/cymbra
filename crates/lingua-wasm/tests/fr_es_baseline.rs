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

//! The fr-es baseline (`docs/lingua/language-matrix-programme.md`, change 51 —
//! add-lingua-french-word-card D9): what a Spanish-native reader of French receives from the
//! engine, over the real fr-es pack built from the committed tables (`tables/fr/` and
//! `tables/fr-es/`, add-lingua-pack-fr-es).
//!
//! The scenario is the French baseline's (`support/french.rs`: its corpus, lemmas, phrases, grammar
//! probes — the reference's, the card's 21 and the 40 lemmas — and reader), the engine started on
//! the real en-es pack as a Spanish-native reader's is, as the French scenario starts on es-en. The
//! extension's `test/word-card-fr-es.spec.ts` reads every grammar and phrase probe of
//! `baseline/fr-es.golden` and pins the Spanish lines the card renders from them; nothing there
//! reads the tables.
//!
//! On every probe up to the reader's level seeding, this golden and fr-en's differ only in the
//! native side — glosses, senses, expressions, the lines naming the packs, the notice, the licences
//! and the backup's profile: `the_golden_is_the_french_one_on_the_studied_side` compares the two
//! committed goldens through `support::studied_side`, as `es_en_baseline.rs` compares es-en's with
//! es-fr's. The probes from `start-review` on follow the deck the seeding left, whose cards are the
//! lemmas each pack glosses (seed-lingua-decks-with-glossed-lemmas D4): `cross_native.rs` compares
//! them through engines, the reader answered without the seeding, and checks the seeding apart.
//!
//! A pull request that changes `baseline/fr-es.golden` changes what Spanish-native readers of
//! French will see, once a package lists fr-es (change 52). Only a dictionary update of fr-es or
//! of fr-en (which moves the studied tables), a deliberate analyser change that bumps French's
//! analyser version, probes the French scenario gains, or an en-es update (its `beside en-es` line
//! alone) should do that. Re-bless with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test
//! fr_es_baseline` — then the extension's snapshot, `yarn vitest run test/word-card-fr-es.spec.ts
//! -u` — and say why in the pull request; `lingua-pack-update` re-blesses both on its own branch.
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use support::french::FRENCH;
use support::{Scenario, first_difference, follows_seeding, probes, studied_side};

/// The French scenario, glossed in Spanish: fr-es beside the real en-es pack the engine starts on.
pub const FRENCH_IN_SPANISH: Scenario = Scenario {
    pair: "fr-es",
    beside: &["en-es"],
    test: "fr_es_baseline",
    ..FRENCH
};

fn fr() -> Option<String> {
    Some("fr".to_owned())
}

#[test]
fn fr_es_output_has_not_moved() {
    let actual = FRENCH_IN_SPANISH.render(Some("fr"));
    let Some(expected) = FRENCH_IN_SPANISH.bless_or_read(&actual) else {
        return;
    };
    assert!(
        actual == expected,
        "fr-es output moved (docs/lingua/language-matrix-programme.md, change 51: the card of \
         Spanish-native readers of French is pinned on the real pack).\n\
         First difference — {}\n\
         If this pull request means to change fr-es output (a dictionary update of fr-es or of \
         fr-en, an analyser change that bumps French's analyser version, probes the French \
         scenario gains, or an en-es update), re-bless with\n  \
         LINGUA_BLESS=1 cargo test -p lingua-wasm --test fr_es_baseline\n  \
         cd apps/lingua-extension && yarn vitest run test/word-card-fr-es.spec.ts -u\n\
         and say why in the pull request. Otherwise the change is wrong.",
        first_difference(&expected, &actual)
    );
}

#[test]
fn the_corpus_reads_as_expected() {
    FRENCH_IN_SPANISH.check_corpus();
}

/// A Spanish-native engine: the reader's profile names Spanish, and French is held beside it.
#[test]
fn the_engine_is_a_spanish_native_reader_s() {
    let engine = FRENCH_IN_SPANISH.loaded();
    assert_eq!(engine.native_language(), "es");
    assert_eq!(engine.languages(), r#"["en","fr"]"#, "en-es beside fr-es");
}

/// French's levels are the studied language's, whatever the native language: the real fr-es pack
/// says of them what fr-en's says.
#[test]
fn fr_es_levels_are_french_s() {
    let (fr_es, fr_en) = (FRENCH_IN_SPANISH.loaded(), FRENCH.loaded());
    assert!(fr_es.has_levels(fr()).expect("levels"));
    assert_eq!(
        fr_es.has_levels(fr()).expect("levels"),
        fr_en.has_levels(fr()).expect("levels")
    );
    assert_eq!(
        fr_es.levels_estimated(fr()).expect("estimated"),
        fr_en.levels_estimated(fr()).expect("estimated")
    );
    assert_eq!(
        fr_es.level_ladder(fr()).expect("ladder"),
        fr_en.level_ladder(fr()).expect("ladder")
    );
}

/// The committed fr-es golden is the committed fr-en golden on the studied side (D9): on every
/// probe before `start-review`, both seeding counts included, the two differ only in the native
/// side — glosses, senses, expressions —, the lines naming the packs (fr-en's engine holds es-en
/// beside it, this one en-es), the notice, the licences and the backup's profile. They ask the same
/// probes. The probes after the seeding are `cross_native.rs`'s, through engines: the cards a level
/// seeds are those each pack glosses (seed-lingua-decks-with-glossed-lemmas D4).
#[test]
fn the_golden_is_the_french_one_on_the_studied_side() {
    // The goldens are being rewritten by the baseline tests, in this binary and in
    // `french_baseline`: nothing to compare until both are blessed.
    if std::env::var_os("LINGUA_BLESS").is_some() {
        return;
    }
    let (fr_en, fr_es) = (
        probes(&FRENCH.committed_golden()),
        probes(&FRENCH_IN_SPANISH.committed_golden()),
    );
    let names = |golden: &[(String, String)]| -> Vec<String> {
        golden
            .iter()
            .map(|(name, _)| name.clone())
            .filter(|name| !name.starts_with("beside "))
            .collect()
    };
    assert_eq!(names(&fr_en), names(&fr_es), "the same probes, in order");
    let shared = names(&fr_en);
    let after = shared
        .iter()
        .filter(|name| follows_seeding(&shared, name))
        .count();
    let mut compared = 0;
    for ((name, body), (_, other)) in fr_en
        .iter()
        .filter(|(name, _)| !name.starts_with("beside "))
        .zip(
            fr_es
                .iter()
                .filter(|(name, _)| !name.starts_with("beside ")),
        )
    {
        // The `about` line names the test that generated the golden; the probes that follow the
        // reader's level seeding are compared through engines (cross_native.rs).
        if name == "about" || follows_seeding(&shared, name) {
            continue;
        }
        let (Some(x), Some(y)) = (studied_side(name, body), studied_side(name, other)) else {
            continue;
        };
        compared += 1;
        let at = x.chars().zip(y.chars()).take_while(|(p, q)| p == q).count();
        assert!(
            x == y,
            "probe `{name}` differs between fr-en.golden and fr-es.golden beyond the native side, \
             character {at}:\n  fr-en: …{}…\n  fr-es: …{}…",
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
    // Every probe but the `about`, pack and beside lines, the cards' glosses, the notice, the
    // licences and the probes that follow the seeding.
    assert!(after > 0, "the reader's probes from `start-review` on");
    assert_eq!(
        compared,
        fr_en.len() - 3 - 2 - FRENCH.lemmas.len() - after,
        "probes compared"
    );
}
