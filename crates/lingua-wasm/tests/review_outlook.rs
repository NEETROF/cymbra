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

//! One figure for the cards waiting (add-lingua-comeback-hints D1, D2): the engine's
//! `reviewableCount` is what `startReview` with the same allowance and no limit holds, and the
//! standalone `reviewOutlook` over the engine's backup counts the same at later times. Compiled
//! for the host (`cargo test`) and for wasm (`wasm-pack test --node crates/lingua-wasm`), so both
//! targets agree. A refused backup throws, which only wasm can show (`languages_wasm.rs`).

use lingua_core::decks::NewCardAllowance;
use lingua_core::decks::outlook::ReviewOutlook;
use lingua_wasm::{LinguaEngine, review_outlook};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const PACK: &[u8] = include_bytes!("fixtures/pack.lingua");

const DAY: f64 = 86_400.0;
const HOUR: f64 = 3_600.0;
/// A local midnight, Unix-epoch seconds.
const DAY_START: f64 = 20_000.0 * DAY;

/// An English deck of 30 new words captured yesterday, 6 of them answered this morning, some
/// recalled, one missed.
fn engine_with_a_morning_session() -> LinguaEngine {
    let mut engine = LinguaEngine::new(PACK).unwrap();
    for i in 0..30 {
        let lemma = format!("word{i:02}");
        engine
            .add_card(
                &lemma,
                &lemma,
                "",
                "",
                None,
                DAY_START - DAY + f64::from(i),
                None,
            )
            .unwrap();
    }
    let morning = DAY_START + 8.0 * HOUR;
    assert_eq!(
        engine.start_review(morning, None, Some(6), Some(10), Some(DAY_START), None),
        6
    );
    let mut first = true;
    while engine.review_remaining() > 0 {
        engine.review_reveal();
        engine.review_grade(if first { "again" } else { "good" }, morning);
        first = false;
    }
    engine
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn the_count_is_what_a_review_would_hold_not_every_new_word() {
    let mut engine = engine_with_a_morning_session();
    let noon = DAY_START + 12.0 * HOUR;
    // 24 new words wait, 4 of today's allowance of 10 are left; the old figure counts them all.
    assert_eq!(engine.due_count(noon, None), 24);
    let count = engine.reviewable_count(noon, None, Some(10), Some(DAY_START));
    assert_eq!(count, 4);
    assert_eq!(
        engine.reviewable_count(noon, Some(vec!["en".to_owned()]), Some(10), Some(DAY_START)),
        count
    );
    assert_eq!(
        engine.reviewable_count(noon, None, None, None),
        24,
        "no allowance"
    );
    assert_eq!(
        engine.start_review(noon, None, None, Some(10), Some(DAY_START), None),
        count,
        "what a session holds"
    );
    // Tomorrow: a new allowance, and today's cards come due.
    let tomorrow = DAY_START + DAY;
    for at in [tomorrow + HOUR, tomorrow + 3.0 * DAY] {
        let expected = engine.start_review(at, None, None, Some(10), Some(tomorrow), None);
        assert_eq!(
            engine.reviewable_count(at, None, Some(10), Some(tomorrow)),
            expected
        );
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn the_outlook_of_the_backup_counts_what_the_engine_counts() {
    let engine = engine_with_a_morning_session();
    let now = DAY_START + 9.0 * HOUR;
    let json = review_outlook(&engine.backup(), now, DAY_START).unwrap();
    let outlook: ReviewOutlook = serde_json::from_str(&json).unwrap();
    assert_eq!(outlook.languages.len(), 1);
    assert_eq!(outlook.languages[0].language, "en");
    assert_eq!(outlook.languages[0].introduced, 6);
    assert_eq!(outlook.languages[0].fresh, 24);
    assert!(!outlook.languages[0].upcoming.is_empty());
    for (at, day_start) in [
        (now, DAY_START),
        (DAY_START + 11.0 * HOUR, DAY_START),
        (DAY_START + DAY + HOUR, DAY_START + DAY),
        (DAY_START + 4.0 * DAY, DAY_START + 4.0 * DAY),
    ] {
        for per_day in [5, 10, 20] {
            assert_eq!(
                outlook.count_at(
                    at as i64,
                    &[],
                    Some(NewCardAllowance {
                        per_day,
                        day_start: day_start as i64,
                    })
                ),
                engine.reviewable_count(at, None, Some(per_day as u32), Some(day_start)),
                "at {at}, {per_day} a day"
            );
        }
    }
}
