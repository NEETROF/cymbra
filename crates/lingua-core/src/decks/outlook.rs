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

//! The review outlook (add-lingua-comeback-hints D2): what the extension's toolbar icon needs to
//! count, at any later time, the cards a review would hold — worked out once per change of the
//! reader's data, read on every event.
//!
//! The count changes when the data changes, and when time passes: a reviewed card comes due, a
//! new day refills the allowance of new words. So the outlook keeps, per studied language, the
//! reviewed cards due when it was worked out, the due times of the next ones to come, the
//! never-reviewed cards waiting and the new words already let in today — and the language's cards,
//! which the popup shows beside the count away from a page (D8). Counts and timestamps only — no
//! word, no page. [`ReviewOutlook::count_at`] is the one reading of it; the extension
//! repeats it in TypeScript, held to this one by a test against the engine.

use serde::{Deserialize, Serialize};

use crate::analysis::language::StudiedLanguage;

use super::review::{Deck, NewCardAllowance};

/// The outlook's format. A reader that finds another version works a new one out.
pub const OUTLOOK_VERSION: u32 = 1;

/// The most upcoming due times kept per language: enough to tell 99 cards from more, which is
/// all the icon shows (« 99+ »).
pub const UPCOMING_MAX: usize = 100;

/// One studied language's outlook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageOutlook {
    /// The language, as its ISO 639-1 tag.
    pub language: String,
    /// Every card of the language, retired ones included — the deck the popup shows away from a
    /// page (D8).
    pub cards: usize,
    /// The reviewed cards due when the outlook was worked out.
    pub due: usize,
    /// The due times (Unix-epoch seconds) of the next reviewed cards to come due, ascending, at
    /// most [`UPCOMING_MAX`]; retired cards never come due and are not there.
    pub upcoming: Vec<i64>,
    /// The never-reviewed cards waiting (due, not retired).
    pub fresh: usize,
    /// The cards first answered since the outlook's `day_start`.
    pub introduced: usize,
}

/// The outlook of every studied language, the primary first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewOutlook {
    /// [`OUTLOOK_VERSION`].
    pub version: u32,
    /// When it was worked out (Unix-epoch seconds).
    pub computed_at: i64,
    /// The reader's local midnight at `computed_at` (Unix-epoch seconds): the day `introduced`
    /// counts.
    pub day_start: i64,
    /// One entry per studied language, in the reader's order.
    pub languages: Vec<LanguageOutlook>,
}

impl LanguageOutlook {
    /// Worked out from `deck` at `now` for `language`, `introduced` counted from `day_start`.
    pub fn of(deck: &Deck, language: StudiedLanguage, now: i64, day_start: i64) -> Self {
        let mut cards = 0;
        let mut due = 0;
        let mut fresh = 0;
        let mut upcoming = Vec::new();
        for (_, card) in deck.iter().filter(|(l, _)| *l == language) {
            cards += 1;
            let review = &card.review;
            if review.is_due(now) {
                if review.memory.is_some() {
                    due += 1;
                } else {
                    fresh += 1;
                }
            } else if let Some(at) = review.due
                && review.memory.is_some()
                && at != i64::MAX
            {
                upcoming.push(at);
            }
        }
        upcoming.sort_unstable();
        upcoming.truncate(UPCOMING_MAX);
        Self {
            language: language.tag().to_owned(),
            cards,
            due,
            upcoming,
            fresh,
            introduced: deck.introduced_since(language, day_start),
        }
    }

    /// What a review started at `at` would hold in this language, as
    /// [`Deck::reviewable_count_for`] counts it, while the data is unchanged. `allowance` is the
    /// day's allowance at `at`: its `day_start` later than the outlook's means another day, on
    /// which no new word has been let in yet. Exact while the upcoming times are not used up
    /// ([`Self::used_up`]).
    pub fn count_at(
        &self,
        at: i64,
        outlook_day_start: i64,
        allowance: Option<NewCardAllowance>,
    ) -> usize {
        let come_due = self.upcoming.iter().take_while(|&&t| t <= at).count();
        let admitted = allowance.map_or(self.fresh, |allowance| {
            let introduced = if allowance.day_start > outlook_day_start {
                0
            } else {
                self.introduced
            };
            self.fresh.min(allowance.per_day.saturating_sub(introduced))
        });
        self.due + come_due + admitted
    }

    /// Whether the upcoming times kept are used up at `at`: the list was cut at
    /// [`UPCOMING_MAX`] and its last time has passed, so a card beyond it may be due — only a
    /// new outlook knows.
    pub fn used_up(&self, at: i64) -> bool {
        self.upcoming.len() >= UPCOMING_MAX && self.upcoming.last().is_some_and(|&t| t <= at)
    }
}

impl ReviewOutlook {
    /// Worked out from `deck` at `now` for `languages` (the reader's studied languages, in
    /// order), `day_start` being the reader's local midnight at `now`.
    pub fn of(deck: &Deck, languages: &[StudiedLanguage], now: i64, day_start: i64) -> Self {
        Self {
            version: OUTLOOK_VERSION,
            computed_at: now,
            day_start,
            languages: languages
                .iter()
                .map(|&language| LanguageOutlook::of(deck, language, now, day_start))
                .collect(),
        }
    }

    /// What a review started at `at` would hold in `languages` (tags; every language of the
    /// outlook when empty), as [`Deck::reviewable_count_for`] counts it while the data is
    /// unchanged.
    pub fn count_at(
        &self,
        at: i64,
        languages: &[StudiedLanguage],
        allowance: Option<NewCardAllowance>,
    ) -> usize {
        self.languages
            .iter()
            .filter(|outlook| {
                languages.is_empty()
                    || languages
                        .iter()
                        .any(|language| language.tag() == outlook.language)
            })
            .map(|outlook| outlook.count_at(at, self.day_start, allowance))
            .sum()
    }

    /// The outlook as JSON, the shape the extension keeps.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("an outlook is always serialisable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decks::card::{Card, EncounterSource, Provenance};
    use crate::decks::fsrs::{FsrsParams, Memory, Rating};
    use crate::decks::review::{ReviewSession, SessionOptions};

    const EN: StudiedLanguage = StudiedLanguage::English;
    const ES: StudiedLanguage = StudiedLanguage::Spanish;
    const DAY: i64 = 86_400;
    const HOUR: i64 = 3_600;

    fn captured(lemma: &str, captured_at: i64) -> Card {
        Card::new(
            lemma,
            lemma,
            Provenance {
                sentence: format!("A sentence with {lemma}."),
                source: EncounterSource::Web {
                    url: "https://example.com".to_owned(),
                },
                captured_at,
            },
            None,
            "fr",
        )
    }

    fn reviewed(lemma: &str, last: i64, due: i64) -> Card {
        let mut card = captured(lemma, last - DAY);
        card.review.memory = Some(Memory {
            stability: 3.0,
            difficulty: 5.0,
        });
        card.review.last_review = Some(last);
        card.review.due = Some(due);
        card.review.reps = 2;
        card
    }

    /// English and Spanish: reviewed cards due and coming due over two days, new words, a
    /// retired card of each kind.
    fn deck(day_start: i64) -> Deck {
        let mut deck = Deck::new();
        for (language, prefix) in [(EN, "e"), (ES, "s")] {
            for i in 0..3 {
                deck.upsert(
                    language,
                    reviewed(
                        &format!("{prefix}due{i}"),
                        day_start - 3 * DAY,
                        day_start - i,
                    ),
                );
            }
            for i in 0..6 {
                let due = day_start + (2 + 6 * i) * HOUR;
                deck.upsert(
                    language,
                    reviewed(&format!("{prefix}soon{i}"), day_start - 3 * DAY, due),
                );
            }
            for i in 0..13 {
                deck.upsert(
                    language,
                    captured(&format!("{prefix}new{i:02}"), day_start - DAY + i),
                );
            }
            let mut known = reviewed(&format!("{prefix}known"), day_start - DAY, day_start);
            known.review.retire(day_start);
            deck.upsert(language, known);
            let mut hidden = captured(&format!("{prefix}hidden"), day_start - DAY);
            hidden.review.retire(day_start);
            deck.upsert(language, hidden);
        }
        deck
    }

    fn allowance(per_day: usize, day_start: i64) -> Option<NewCardAllowance> {
        Some(NewCardAllowance { per_day, day_start })
    }

    /// The outlook's count against the deck's at `at`, `day_start` being the local midnight at
    /// `at`, in every language selection.
    fn assert_counts_agree(outlook: &ReviewOutlook, deck: &Deck, at: i64, day_start: i64) {
        for languages in [&[][..], &[EN], &[ES]] {
            for allowance in [None, allowance(10, day_start), allowance(4, day_start)] {
                assert_eq!(
                    outlook.count_at(at, languages, allowance),
                    deck.reviewable_count_for(at, languages, allowance),
                    "at {at}, in {languages:?}, with {allowance:?}"
                );
            }
        }
    }

    #[test]
    fn an_outlook_counts_what_the_deck_counts_now_later_and_the_next_day() {
        let day_start = 200 * DAY;
        let mut deck = deck(day_start);
        let now = day_start + HOUR;
        let outlook = ReviewOutlook::of(&deck, &[EN, ES], now, day_start);
        let english = &outlook.languages[0];
        assert_eq!(english.language, "en");
        assert_eq!(
            (
                english.cards,
                english.due,
                english.fresh,
                english.introduced
            ),
            (24, 3, 13, 0)
        );
        assert_eq!(
            english.upcoming.len(),
            6,
            "the retired card never comes due"
        );
        assert!(english.upcoming.is_sorted());

        // Now, past some of the upcoming times, and on the next local day.
        for at in [now, day_start + 9 * HOUR, day_start + 20 * HOUR] {
            assert_counts_agree(&outlook, &deck, at, day_start);
        }
        assert_counts_agree(&outlook, &deck, day_start + DAY + 2 * HOUR, day_start + DAY);

        // New words answered this morning: the outlook worked out after it knows them.
        let params = FsrsParams::default();
        let mut session = ReviewSession::start_with(
            &deck,
            &params,
            now,
            &[EN],
            SessionOptions {
                practice: false,
                limit: Some(8),
                new_cards: allowance(10, day_start),
            },
        );
        while session.current_key().is_some() {
            session.grade(&mut deck, &params, Rating::Good, now);
        }
        assert!(deck.introduced_since(EN, day_start) > 0);
        let after = now + 60;
        let outlook = ReviewOutlook::of(&deck, &[EN, ES], after, day_start);
        for at in [after, day_start + 9 * HOUR, day_start + 23 * HOUR] {
            assert_counts_agree(&outlook, &deck, at, day_start);
        }
        // The next day, the allowance is whole again.
        assert_counts_agree(&outlook, &deck, day_start + DAY + 9 * HOUR, day_start + DAY);
        assert_counts_agree(&outlook, &deck, day_start + 3 * DAY, day_start + 3 * DAY);
    }

    #[test]
    fn an_outlook_keeps_a_hundred_upcoming_times_and_says_when_they_are_used_up() {
        let day_start = 200 * DAY;
        let mut deck = Deck::new();
        for i in 0..130 {
            deck.upsert(
                EN,
                reviewed(
                    &format!("w{i:03}"),
                    day_start - DAY,
                    day_start + (i + 1) * 60,
                ),
            );
        }
        let outlook = ReviewOutlook::of(&deck, &[EN], day_start, day_start);
        let english = &outlook.languages[0];
        assert_eq!(english.upcoming.len(), UPCOMING_MAX);
        assert_eq!(english.upcoming.last(), Some(&(day_start + 100 * 60)));
        assert!(!english.used_up(day_start + 99 * 60));
        assert_eq!(outlook.count_at(day_start + 99 * 60, &[], None), 99);
        assert!(english.used_up(day_start + 100 * 60));
        // Exact up to the last time kept; beyond it only a new outlook knows.
        assert_eq!(
            outlook.count_at(day_start + 100 * 60, &[], None),
            deck.reviewable_count_for(day_start + 100 * 60, &[], None)
        );
        assert!(
            outlook.count_at(day_start + DAY, &[], None)
                < deck.reviewable_count_for(day_start + DAY, &[], None)
        );

        let short = ReviewOutlook::of(&deck, &[EN], day_start + 50 * 60, day_start);
        assert!(
            !short.languages[0].used_up(day_start + DAY),
            "a list not cut is never used up"
        );
        assert_eq!(short.languages[0].upcoming.len(), 80);
    }

    #[test]
    fn an_outlook_is_written_as_plain_json_with_its_version() {
        let mut deck = Deck::new();
        deck.upsert(EN, captured("seldom", 0));
        deck.upsert(EN, reviewed("ship", 0, 5 * DAY));
        let outlook = ReviewOutlook::of(&deck, &[EN, ES], DAY, DAY);
        assert_eq!(
            outlook.to_json(),
            format!(
                r#"{{"version":{OUTLOOK_VERSION},"computedAt":{DAY},"dayStart":{DAY},"languages":[{{"language":"en","cards":2,"due":0,"upcoming":[{}],"fresh":1,"introduced":0}},{{"language":"es","cards":0,"due":0,"upcoming":[],"fresh":0,"introduced":0}}]}}"#,
                5 * DAY
            )
        );
        let back: ReviewOutlook = serde_json::from_str(&outlook.to_json()).unwrap();
        assert_eq!(back, outlook);
    }
}
