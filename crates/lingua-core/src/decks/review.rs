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

//! The deck and the review session (designs D2, D3).
//!
//! A [`Deck`] holds one [`Card`] per `(studied language, lemma)`. A
//! [`ReviewSession`] walks the cards due at its start, one at a time, answer
//! hidden until revealed; grading advances the FSRS state, and "I know this"
//! writes `Known(Srs)` into the knowledge model and retires the card from the
//! queue without deleting it — the core substrate the side panel / drawer
//! surfaces will drive (`add-lingua-extension-review`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::analysis::language::StudiedLanguage;
use crate::knowledge::state::KnowledgeState;
use crate::knowledge::status::{KnownSource, Status};

use super::card::Card;
use super::fsrs::{FsrsParams, Rating};

/// A deck of cards, keyed by studied language then lemma (nested maps keep the
/// serialised deck plain-JSON and deterministic, as elsewhere in the core).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Deck {
    cards: BTreeMap<StudiedLanguage, BTreeMap<String, Card>>,
}

impl Deck {
    /// An empty deck.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces the card for a lemma.
    pub fn upsert(&mut self, lang: StudiedLanguage, card: Card) {
        self.cards
            .entry(lang)
            .or_default()
            .insert(card.lemma.clone(), card);
    }

    /// The card for a lemma, if present.
    pub fn get(&self, lang: StudiedLanguage, lemma: &str) -> Option<&Card> {
        self.cards
            .get(&lang)
            .and_then(|per_lang| per_lang.get(lemma))
    }

    /// Total number of cards across all languages.
    pub fn len(&self) -> usize {
        self.cards.values().map(BTreeMap::len).sum()
    }

    /// Whether the deck holds no cards.
    pub fn is_empty(&self) -> bool {
        self.cards.values().all(BTreeMap::is_empty)
    }

    /// The `(language, lemma)` keys of every card due at `now`, in
    /// deterministic order (language then lemma).
    pub fn due_keys(&self, now: i64) -> Vec<(StudiedLanguage, String)> {
        self.cards
            .iter()
            .flat_map(|(&lang, per_lang)| {
                per_lang
                    .iter()
                    .filter(move |(_, card)| card.review.is_due(now))
                    .map(move |(lemma, _)| (lang, lemma.clone()))
            })
            .collect()
    }

    /// The number of cards due at `now`.
    pub fn due_count(&self, now: i64) -> usize {
        self.cards
            .values()
            .flat_map(BTreeMap::values)
            .filter(|card| card.review.is_due(now))
            .count()
    }
}

/// A review session over the cards that were due when it started. Owns its
/// queue of keys, so it does not borrow the deck between steps; each action
/// takes the deck (and, for "I know this", the knowledge state) by reference.
#[derive(Debug, Clone)]
pub struct ReviewSession {
    queue: Vec<(StudiedLanguage, String)>,
    position: usize,
    revealed: bool,
}

impl ReviewSession {
    /// Starts a session over everything due at `now`.
    pub fn start(deck: &Deck, now: i64) -> Self {
        Self {
            queue: deck.due_keys(now),
            position: 0,
            revealed: false,
        }
    }

    /// The key of the card currently under review, or `None` when the session
    /// is finished.
    pub fn current_key(&self) -> Option<&(StudiedLanguage, String)> {
        self.queue.get(self.position)
    }

    /// The card currently under review, looked up in the deck.
    pub fn current<'d>(&self, deck: &'d Deck) -> Option<&'d Card> {
        self.current_key()
            .and_then(|(lang, lemma)| deck.get(*lang, lemma))
    }

    /// Whether the current card's answer has been revealed.
    pub fn is_revealed(&self) -> bool {
        self.revealed
    }

    /// Reveals the current card's answer (the only way past the hidden state).
    pub fn reveal(&mut self) {
        self.revealed = true;
    }

    /// Cards not yet answered in this session (including the current one).
    pub fn remaining(&self) -> usize {
        self.queue.len().saturating_sub(self.position)
    }

    /// Grades the current card and advances. Ignored if the session is
    /// finished. Resets the reveal state for the next card.
    pub fn grade(&mut self, deck: &mut Deck, params: &FsrsParams, rating: Rating, now: i64) {
        let Some((lang, lemma)) = self.current_key().cloned() else {
            return;
        };
        if let Some(card) = deck.cards.get_mut(&lang).and_then(|m| m.get_mut(&lemma)) {
            card.review.grade(params, rating, now);
        }
        self.advance();
    }

    /// Marks the current card's lemma known (provenance `srs`) in the
    /// knowledge model and retires the card from review — the card and its
    /// FSRS history are kept, it simply stops coming due. Advances.
    pub fn mark_known(&mut self, deck: &mut Deck, knowledge: &mut KnowledgeState, now: i64) {
        let Some((lang, lemma)) = self.current_key().cloned() else {
            return;
        };
        knowledge.set_status(lang, &lemma, Status::Known(KnownSource::Srs));
        if let Some(card) = deck.cards.get_mut(&lang).and_then(|m| m.get_mut(&lemma)) {
            card.review.retire(now);
        }
        self.advance();
    }

    fn advance(&mut self) {
        self.position += 1;
        self.revealed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decks::card::{EncounterSource, Provenance};

    const EN: StudiedLanguage = StudiedLanguage::English;
    const DAY: i64 = 86_400;

    fn card(lemma: &str) -> Card {
        Card::new(
            lemma,
            lemma,
            Provenance {
                sentence: format!("A sentence with {lemma}."),
                source: EncounterSource::Web {
                    url: "https://example.com".to_owned(),
                },
                captured_at: 0,
            },
            None,
        )
    }

    fn deck_of(lemmas: &[&str]) -> Deck {
        let mut deck = Deck::new();
        for l in lemmas {
            deck.upsert(EN, card(l));
        }
        deck
    }

    #[test]
    fn new_cards_are_all_due_and_counted() {
        let deck = deck_of(&["run", "ship", "code"]);
        assert_eq!(deck.len(), 3);
        assert_eq!(deck.due_count(0), 3);
        assert_eq!(deck.due_keys(0).len(), 3);
    }

    #[test]
    fn spec_micro_session_answer_hidden_then_revealed_due_count_falls() {
        let mut deck = deck_of(&["run", "ship", "code"]);
        let params = FsrsParams::default();
        let mut session = ReviewSession::start(&deck, 0);
        assert_eq!(session.remaining(), 3);

        // Answer hidden until an explicit reveal.
        assert!(!session.is_revealed());
        assert!(session.current(&deck).is_some());
        session.reveal();
        assert!(session.is_revealed());

        // Grade the three cards; each grade re-hides the next answer.
        session.grade(&mut deck, &params, Rating::Good, 0);
        assert!(!session.is_revealed());
        session.grade(&mut deck, &params, Rating::Good, 0);
        session.grade(&mut deck, &params, Rating::Good, 0);
        assert_eq!(session.remaining(), 0);
        assert!(session.current(&deck).is_none());

        // Graded cards are scheduled into the future: none due now.
        assert_eq!(deck.due_count(0), 0);
        // They come back due once their interval elapses.
        assert!(deck.due_count(30 * DAY) > 0);
    }

    #[test]
    fn spec_mark_known_writes_srs_status_and_retires_card() {
        let mut deck = deck_of(&["seldom"]);
        let mut knowledge = KnowledgeState::new();
        let mut session = ReviewSession::start(&deck, 0);

        session.mark_known(&mut deck, &mut knowledge, 0);

        // The lemma is now known with SRS provenance.
        assert_eq!(
            knowledge.explicit_status(EN, "seldom"),
            Some(Status::Known(KnownSource::Srs))
        );
        // The card is kept (history intact) but never comes due again.
        assert!(deck.get(EN, "seldom").is_some());
        assert_eq!(deck.due_count(i64::MAX - 1), 0);
        assert_eq!(session.remaining(), 0);
    }

    #[test]
    fn grading_on_a_finished_session_is_a_no_op() {
        let mut deck = deck_of(&["run"]);
        let params = FsrsParams::default();
        let mut session = ReviewSession::start(&deck, 0);
        session.grade(&mut deck, &params, Rating::Good, 0);
        // Already past the end: further grades do nothing and do not panic.
        session.grade(&mut deck, &params, Rating::Again, 0);
        assert_eq!(session.remaining(), 0);
        assert_eq!(deck.get(EN, "run").unwrap().review.reps, 1);
    }
}
