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

    /// Seeds cards for a chosen set of `(lemma, gloss)` — typically the lemmas
    /// of a selected CEFR level, in the caller's order (commonest-first by
    /// default) — for level-targeted feeding (`add-lingua-cefr-levels`). Skips
    /// any lemma that already has a card or an explicit status in `knowledge`
    /// (idempotent), stops after `cap` new cards, and stamps each with the
    /// reserved `Import` source. Returns the number actually added.
    pub fn seed_lemmas<'a>(
        &mut self,
        lang: StudiedLanguage,
        lemmas: impl IntoIterator<Item = (&'a str, Option<&'a str>)>,
        knowledge: &KnowledgeState,
        cap: usize,
        at: i64,
    ) -> usize {
        let mut added = 0;
        for (lemma, gloss) in lemmas {
            if added >= cap {
                break;
            }
            if self.get(lang, lemma).is_some() || knowledge.explicit_status(lang, lemma).is_some() {
                continue;
            }
            self.upsert(lang, Card::seeded(lemma, gloss.map(str::to_owned), at));
            added += 1;
        }
        added
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

    /// Every card with its language, cloned — the outbox source for a card push
    /// (`add-lingua-connected-clients`), in deterministic (language, lemma) order.
    /// The FSRS state travels as the card's `review`.
    pub fn export_cards(&self) -> Vec<(StudiedLanguage, Card)> {
        let mut out = Vec::new();
        for (&lang, per_lang) in &self.cards {
            for card in per_lang.values() {
                out.push((lang, card.clone()));
            }
        }
        out
    }

    /// Apply a pulled card under last-write-wins by its `updated_at`: a
    /// newer-or-equal card wins (upsert), an older one is dropped. Returns whether
    /// local state changed.
    ///
    /// Upsert-only: cards are added, graded and retired but never deleted in the
    /// MVP (mark-known retires a card, it does not remove it), so there is no
    /// deletion to sync. Card deletion — and the tombstones it would need to stay
    /// LWW-correct — is deferred to the change that actually adds it, rather than
    /// shipping a delete path that resurrects on a stale re-add.
    ///
    /// `captured_at` is the immutable encounter time and does not travel on the
    /// wire (the `CardOp` has no field for it), so the local value is preserved on
    /// an update: sync never rewrites when the word was first met.
    pub fn apply_card_lww(&mut self, lang: StudiedLanguage, mut card: Card) -> bool {
        // No existing card → i64::MIN, so any incoming wins.
        let existing_ts = self
            .get(lang, &card.lemma)
            .map_or(i64::MIN, |c| c.updated_at);
        if card.updated_at < existing_ts {
            return false;
        }
        if let Some(captured_at) = self
            .get(lang, &card.lemma)
            .map(|c| c.provenance.captured_at)
        {
            card.provenance.captured_at = captured_at;
        }
        let changed = self.get(lang, &card.lemma) != Some(&card);
        self.upsert(lang, card);
        changed
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
            card.updated_at = now; // sync: a grade is a change
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
        knowledge.set_status_at(lang, &lemma, Status::Known(KnownSource::Srs), now * 1000);
        if let Some(card) = deck.cards.get_mut(&lang).and_then(|m| m.get_mut(&lemma)) {
            card.review.retire(now);
            card.updated_at = now; // sync: retiring is a change
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
    fn seed_lemmas_caps_and_skips_tracked_lemmas() {
        let mut deck = deck_of(&["run"]); // `run` already carded
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(EN, "city", Status::Learning); // `city` has an explicit status
        let candidates = [
            ("run", Some("courir")),    // already carded → skip
            ("city", Some("ville")),    // explicit status → skip
            ("nuance", Some("nuance")), // new → add
            ("quixotic", None),         // new → add
            ("arcane", None),           // would add, but the cap stops us first
        ];
        let added = deck.seed_lemmas(EN, candidates, &knowledge, 2, 5 * DAY);
        assert_eq!(added, 2);
        assert!(deck.get(EN, "nuance").is_some());
        assert!(deck.get(EN, "quixotic").is_some());
        assert!(deck.get(EN, "arcane").is_none()); // capped
        assert_eq!(
            deck.get(EN, "nuance").unwrap().provenance.source,
            EncounterSource::Import
        );
        assert_eq!(deck.get(EN, "nuance").unwrap().updated_at, 5 * DAY);
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

    fn card_at(lemma: &str, updated_at: i64) -> Card {
        let mut c = card(lemma);
        c.updated_at = updated_at;
        c
    }

    #[test]
    fn export_cards_lists_every_card_in_deterministic_order() {
        let deck = deck_of(&["run", "city", "seldom"]);
        let lemmas: Vec<_> = deck
            .export_cards()
            .into_iter()
            .map(|(_, c)| c.lemma)
            .collect();
        assert_eq!(lemmas, ["city", "run", "seldom"]); // BTreeMap (lemma) order
    }

    #[test]
    fn apply_card_lww_upserts_newer_applies_unknown_drops_older() {
        let mut deck = Deck::new();
        deck.upsert(EN, card_at("run", 100));

        // Older incoming loses.
        let mut older = card_at("run", 50);
        older.encountered_form = "OLD".to_owned();
        assert!(!deck.apply_card_lww(EN, older));
        assert_eq!(deck.get(EN, "run").unwrap().encountered_form, "run");

        // Newer incoming wins.
        let mut newer = card_at("run", 200);
        newer.encountered_form = "NEW".to_owned();
        assert!(deck.apply_card_lww(EN, newer));
        assert_eq!(deck.get(EN, "run").unwrap().encountered_form, "NEW");

        // A card the deck never had is applied.
        assert!(deck.apply_card_lww(EN, card_at("city", 10)));
        assert!(deck.get(EN, "city").is_some());
    }

    #[test]
    fn apply_card_lww_preserves_the_local_captured_at() {
        // captured_at is the immutable encounter time; it has no wire field, so an
        // incoming op carries only the op timestamp. Applying must not overwrite it.
        let mut original = card("run"); // captured_at 0
        original.provenance.captured_at = 1_000;
        original.updated_at = 1_000;
        let mut deck = Deck::new();
        deck.upsert(EN, original);

        // A later grade elsewhere arrives as an op stamped 5_000 with captured_at=5_000.
        let mut graded = card("run");
        graded.provenance.captured_at = 5_000; // what the wire would fabricate
        graded.updated_at = 5_000;
        graded.encountered_form = "ran".to_owned();
        deck.apply_card_lww(EN, graded);

        let stored = deck.get(EN, "run").unwrap();
        assert_eq!(stored.encountered_form, "ran"); // the update landed
        assert_eq!(stored.provenance.captured_at, 1_000); // encounter time preserved
        assert_eq!(stored.updated_at, 5_000);
    }

    #[test]
    fn grading_and_mark_known_bump_the_card_updated_at() {
        let mut deck = deck_of(&["run", "seldom"]); // captured_at 0 → updated_at 0
        let params = FsrsParams::default();
        let mut knowledge = KnowledgeState::new();
        let mut session = ReviewSession::start(&deck, 0);
        session.grade(&mut deck, &params, Rating::Good, 5 * DAY);
        assert_eq!(deck.get(EN, "run").unwrap().updated_at, 5 * DAY);
        session.mark_known(&mut deck, &mut knowledge, 7 * DAY);
        assert_eq!(deck.get(EN, "seldom").unwrap().updated_at, 7 * DAY);
        // mark-known also stamps the status (in millis) so it syncs.
        assert_eq!(knowledge.status_updated_at(EN, "seldom"), 7 * DAY * 1000);
    }
}
