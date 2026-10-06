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
//! [`ReviewSession`] walks cards due at its start, the most fragile first and at
//! most as many as its options allow, one at a time, answer hidden until
//! revealed; a card's first answer advances its FSRS state, a missed card comes
//! back later in the session, and "I know this" (`Known(Srs)`) or "Ne plus me le
//! montrer" (`Ignored`) retires the card without deleting it — the core
//! substrate the side panel / drawer surfaces drive (`add-lingua-extension-review`,
//! `refine-lingua-review-session`).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::analysis::language::StudiedLanguage;
use crate::knowledge::state::KnowledgeState;
use crate::knowledge::status::{KnownSource, Status};

use super::card::{Card, EncounterSource};
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

    /// Every language the deck holds a card for.
    pub fn languages(&self) -> BTreeSet<StudiedLanguage> {
        self.cards.keys().copied().collect()
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

    /// Retire the card for `lemma` if one exists (keep the card + its FSRS history; it
    /// simply stops coming due) and stamp it for sync. Returns whether a card was retired.
    /// Used when a word is reclassified known/ignored OUTSIDE a review — [`mark_known`] does
    /// the same for the in-review card — so a card never keeps surfacing for a word the
    /// reader no longer treats as to-learn. This is retirement, not deletion (the card
    /// stays, so it needs no deletion tombstone and syncs as a plain LWW update).
    ///
    /// [`mark_known`]: ReviewSession::mark_known
    pub fn retire(&mut self, lang: StudiedLanguage, lemma: &str, now: i64) -> bool {
        let Some(card) = self.cards.get_mut(&lang).and_then(|m| m.get_mut(lemma)) else {
            return false;
        };
        card.review.retire(now);
        // Sync: retiring is a change. Never move the card's clock backwards — a status
        // pulled from another device can be older than the last local edit, and a card
        // stamped back in time would lose last-write-wins and keep coming due elsewhere.
        card.updated_at = card.updated_at.max(now);
        true
    }

    /// Total number of cards across all languages.
    pub fn len(&self) -> usize {
        self.cards.values().map(BTreeMap::len).sum()
    }

    /// Whether the deck holds no cards.
    pub fn is_empty(&self) -> bool {
        self.cards.values().all(BTreeMap::is_empty)
    }

    /// The number of cards in `languages` — every language when it is empty
    /// (refine-lingua-review-language D4).
    pub fn count_for(&self, languages: &[StudiedLanguage]) -> usize {
        self.cards
            .iter()
            .filter(|(lang, _)| languages.is_empty() || languages.contains(lang))
            .map(|(_, per_lang)| per_lang.len())
            .sum()
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
        self.due_count_for(now, &[])
    }

    /// The number of cards due at `now` in `languages` — every language when it is empty
    /// (add-lingua-language-stats-review D1).
    pub fn due_count_for(&self, now: i64, languages: &[StudiedLanguage]) -> usize {
        self.cards
            .iter()
            .filter(|(lang, _)| languages.is_empty() || languages.contains(lang))
            .flat_map(|(_, per_lang)| per_lang.values())
            .filter(|card| card.review.is_due(now))
            .count()
    }

    /// The cards of `language` first answered at or after `day_start` (Unix-epoch seconds):
    /// those with one review, made today (refine-lingua-review-session D2). Each language has
    /// its own allowance, so each counts its own. A session grades a card once, so a card
    /// first answered today still has one review at the end of the day, and one first
    /// answered earlier has at least two by the time it is answered again.
    pub fn introduced_since(&self, language: StudiedLanguage, day_start: i64) -> usize {
        self.cards
            .get(&language)
            .into_iter()
            .flat_map(BTreeMap::values)
            .filter(|card| {
                card.review.reps == 1 && card.review.last_review.is_some_and(|t| t >= day_start)
            })
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
        if let Some(existing) = self.get(lang, &card.lemma) {
            card.provenance.captured_at = existing.provenance.captured_at;
            // The page a card was captured from never travels (add-lingua-privacy-controls):
            // a synced card arrives without one and keeps whatever this device recorded.
            if matches!(&card.provenance.source, EncounterSource::Web { url } if url.is_empty()) {
                card.provenance.source = existing.provenance.source.clone();
            }
        }
        let changed = self.get(lang, &card.lemma) != Some(&card);
        self.upsert(lang, card);
        changed
    }
}

/// How many distinct cards a session holds when the reader starts one
/// (refine-lingua-review-session D1).
pub const SESSION_CARDS: usize = 10;
/// While reviewed cards remain, a never-reviewed card takes every fourth place: the 4th, the
/// 8th… (D1).
const NEW_CARD_EVERY: usize = 4;
/// A missed card comes back after this many other cards (D3).
const RETURN_AFTER: usize = 3;
/// The most times one card is asked in a session (D3).
const MAX_ASKS: u8 = 3;
/// A first answer that schedules a card at least this far away (seconds) counts it as held
/// for more than a month (D8).
const HOLDING_SECONDS: i64 = 30 * 86_400;

/// The day's allowance of never-reviewed cards, in each studied language
/// (refine-lingua-review-session D2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewCardAllowance {
    /// At most this many never-reviewed cards of each language are first answered per day.
    pub per_day: usize,
    /// The reader's local midnight, Unix-epoch seconds: where "today" starts.
    pub day_start: i64,
}

/// What a session starts with. The default takes every due card with no allowance.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SessionOptions {
    /// At most this many distinct cards; `None` takes every due card.
    pub limit: Option<usize>,
    /// The day's allowance of never-reviewed cards; `None` lets every due one in.
    pub new_cards: Option<NewCardAllowance>,
}

/// What a session did, for its end (refine-lingua-review-session D8).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct SessionSummary {
    /// Cards whose first answer in the session updated their FSRS state.
    pub reviewed: usize,
    /// Cards missed, then recalled later in the same session.
    pub recovered: usize,
    /// Cards a first answer scheduled at least 30 days away.
    pub holding: usize,
    /// Cards marked known.
    pub known: usize,
    /// Cards hidden from review ("Ne plus me le montrer").
    pub hidden: usize,
}

/// How one card has fared in the current session.
#[derive(Debug, Clone, Copy, Default)]
struct Asked {
    /// Times it was answered in the session.
    times: u8,
    /// Whether its first answer was a miss.
    missed: bool,
}

/// The never-reviewed keys, in the order given, that the day's allowance lets into a session:
/// for each language, `per_day` less its cards already introduced today (D2). A session holds
/// one language, so an allowance shared by every language would let the first one reviewed in a
/// day spend it all. Without an allowance, every key.
fn within_allowance(
    deck: &Deck,
    fresh: impl Iterator<Item = (StudiedLanguage, String)>,
    allowance: Option<NewCardAllowance>,
) -> Vec<(StudiedLanguage, String)> {
    let Some(allowance) = allowance else {
        return fresh.collect();
    };
    let mut left: BTreeMap<StudiedLanguage, usize> = BTreeMap::new();
    fresh
        .filter(|(language, _)| {
            let left = left.entry(*language).or_insert_with(|| {
                allowance
                    .per_day
                    .saturating_sub(deck.introduced_since(*language, allowance.day_start))
            });
            let admitted = *left > 0;
            *left = left.saturating_sub(1);
            admitted
        })
        .collect()
}

/// A review session over the cards that were due when it started. Owns its
/// queue of keys, so it does not borrow the deck between steps; each action
/// takes the deck (and, for "I know this", the knowledge state) by reference.
#[derive(Debug, Clone)]
pub struct ReviewSession {
    queue: Vec<(StudiedLanguage, String)>,
    position: usize,
    revealed: bool,
    asked: BTreeMap<(StudiedLanguage, String), Asked>,
    summary: SessionSummary,
}

impl ReviewSession {
    /// Starts a session over everything due at `now`.
    pub fn start(deck: &Deck, now: i64) -> Self {
        Self::start_for(deck, now, &[])
    }

    /// Starts a session over everything due at `now` in `languages`, every language when it is
    /// empty (add-lingua-language-stats-review D1), in the order of [`Self::start_with`], with
    /// no cap and no allowance.
    pub fn start_for(deck: &Deck, now: i64, languages: &[StudiedLanguage]) -> Self {
        Self::start_with(
            deck,
            &FsrsParams::default(),
            now,
            languages,
            SessionOptions::default(),
        )
    }

    /// Starts a session over the cards due at `now` in `languages` (every language when it is
    /// empty), whatever their language, ordered for review (refine-lingua-review-session D1):
    /// reviewed cards by predicted recall, lowest first, then by due date, language and lemma;
    /// never-reviewed cards in capture order, oldest first, within their language's allowance
    /// for the day. While reviewed cards remain, a never-reviewed card takes every fourth
    /// place; when either kind runs out, the other fills the session, up to `options.limit`
    /// cards.
    pub fn start_with(
        deck: &Deck,
        params: &FsrsParams,
        now: i64,
        languages: &[StudiedLanguage],
        options: SessionOptions,
    ) -> Self {
        let mut reviewed: Vec<(f64, i64, (StudiedLanguage, String))> = Vec::new();
        let mut fresh: Vec<(i64, (StudiedLanguage, String))> = Vec::new();
        // `due_keys` comes in (language, lemma) order and both sorts are stable, so equal
        // cards keep that order.
        for key in deck.due_keys(now) {
            if !(languages.is_empty() || languages.contains(&key.0)) {
                continue;
            }
            let Some(card) = deck.get(key.0, &key.1) else {
                continue;
            };
            match card.review.retrievability_at(params, now) {
                Some(recall) => reviewed.push((recall, card.review.due.unwrap_or(i64::MIN), key)),
                None => fresh.push((card.provenance.captured_at, key)),
            }
        }
        reviewed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        fresh.sort_by_key(|(captured_at, _)| *captured_at);

        let fresh = within_allowance(
            deck,
            fresh.into_iter().map(|(_, key)| key),
            options.new_cards,
        );
        let limit = options.limit.unwrap_or(usize::MAX);
        let mut reviewed = reviewed.into_iter().map(|(_, _, key)| key);
        let mut fresh = fresh.into_iter();
        let mut queue = Vec::new();
        while queue.len() < limit {
            let new_card_turn = (queue.len() + 1) % NEW_CARD_EVERY == 0;
            let next = if new_card_turn {
                fresh.next().or_else(|| reviewed.next())
            } else {
                reviewed.next().or_else(|| fresh.next())
            };
            match next {
                Some(key) => queue.push(key),
                None => break,
            }
        }
        Self {
            queue,
            position: 0,
            revealed: false,
            asked: BTreeMap::new(),
            summary: SessionSummary::default(),
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

    /// Answers the current card and advances; returns whether the answer updated the card's
    /// FSRS state. Ignored, returning `false`, when the session is finished.
    ///
    /// Only a card's first answer in the session is graded (refine-lingua-review-session D3):
    /// a missed card (`Again`) comes back after three other cards, or at the end when fewer
    /// remain, until it is recalled or has been asked three times, and those later answers
    /// only keep it in the session or take it out. Grading them would apply the forgetting
    /// formula twice to a single lapse, since the core models no same-day review.
    pub fn grade(
        &mut self,
        deck: &mut Deck,
        params: &FsrsParams,
        rating: Rating,
        now: i64,
    ) -> bool {
        let Some(key) = self.current_key().cloned() else {
            return false;
        };
        let missed = rating == Rating::Again;
        let (first, times, recovered) = {
            let asked = self.asked.entry(key.clone()).or_default();
            asked.times = asked.times.saturating_add(1);
            let first = asked.times == 1;
            if first {
                asked.missed = missed;
            }
            (first, asked.times, !first && !missed && asked.missed)
        };
        if recovered {
            self.summary.recovered += 1;
        }
        let mut graded = false;
        if first && let Some(card) = deck.cards.get_mut(&key.0).and_then(|m| m.get_mut(&key.1)) {
            card.review.grade(params, rating, now);
            card.updated_at = now; // sync: a grade is a change
            graded = true;
            self.summary.reviewed += 1;
            if !missed
                && card
                    .review
                    .due
                    .is_some_and(|due| due - now >= HOLDING_SECONDS)
            {
                self.summary.holding += 1;
            }
        }
        if missed && times < MAX_ASKS {
            let at = (self.position + 1 + RETURN_AFTER).min(self.queue.len());
            self.queue.insert(at, key);
        }
        self.advance();
        graded
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
        self.summary.known += 1;
        self.advance();
    }

    /// Hides the current card's word from review ("Ne plus me le montrer",
    /// refine-lingua-review-session D5): the lemma becomes `ignored`, stamped for
    /// last-write-wins, and its card is retired as [`Deck::retire`] does — kept with its
    /// history, never due again. The word is put back to learn as any ignored word is.
    /// Advances.
    pub fn ignore(&mut self, deck: &mut Deck, knowledge: &mut KnowledgeState, now: i64) {
        let Some((lang, lemma)) = self.current_key().cloned() else {
            return;
        };
        knowledge.set_status_at(lang, &lemma, Status::Ignored, now * 1000);
        deck.retire(lang, &lemma, now);
        self.summary.hidden += 1;
        self.advance();
    }

    /// What the session has done so far (refine-lingua-review-session D8).
    pub fn summary(&self) -> SessionSummary {
        self.summary
    }

    fn advance(&mut self) {
        self.position += 1;
        self.revealed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decks::card::Provenance;
    use crate::decks::fsrs::Memory;

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

    /// A reviewed card: `stability` days, last reviewed at `last`, due at `due`.
    fn reviewed(lemma: &str, stability: f64, last: i64, due: i64) -> Card {
        let mut c = card(lemma);
        c.review.memory = Some(Memory {
            stability,
            difficulty: 5.0,
        });
        c.review.last_review = Some(last);
        c.review.due = Some(due);
        c.review.reps = 2;
        c
    }

    /// A never-reviewed card captured at `captured_at`.
    fn captured(lemma: &str, captured_at: i64) -> Card {
        let mut c = card(lemma);
        c.provenance.captured_at = captured_at;
        c
    }

    fn order(session: &ReviewSession) -> Vec<&str> {
        session.queue.iter().map(|(_, w)| w.as_str()).collect()
    }

    fn limited(per_day: usize, day_start: i64) -> SessionOptions {
        SessionOptions {
            limit: Some(SESSION_CARDS),
            new_cards: Some(NewCardAllowance { per_day, day_start }),
        }
    }

    #[test]
    fn spec_a_queue_starts_with_the_lowest_predicted_recall_in_one_language_or_several() {
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        let mut deck = Deck::new();
        // At 40 days: recall ≈ 0.87 for `anchor`, 0.72 for `faro`, 0.42 for `zeal`.
        deck.upsert(EN, reviewed("anchor", 30.0, 0, 10 * DAY));
        deck.upsert(EN, reviewed("zeal", 2.0, 0, 2 * DAY));
        deck.upsert(ES, reviewed("faro", 10.0, 0, 10 * DAY));
        deck.upsert(ES, captured("mar", 0));
        let now = 40 * DAY;

        let all = ReviewSession::start(&deck, now);
        let tagged: Vec<_> = all
            .queue
            .iter()
            .map(|(l, w)| (l.tag(), w.as_str()))
            .collect();
        assert_eq!(
            tagged,
            vec![
                ("en", "zeal"),
                ("es", "faro"),
                ("en", "anchor"),
                ("es", "mar")
            ]
        );

        let spanish = ReviewSession::start_for(&deck, now, &[ES]);
        assert_eq!(order(&spanish), vec!["faro", "mar"]);
        assert_eq!(deck.due_count_for(now, &[ES]), 2);
        assert_eq!(deck.due_count_for(now, &[]), 4);
        assert_eq!(deck.due_count(now), 4);
        // The deck's size by language (refine-lingua-review-language D4).
        assert_eq!(deck.count_for(&[ES]), 2);
        assert_eq!(deck.count_for(&[EN]), 2);
        assert_eq!(deck.count_for(&[]), deck.len());

        // One language follows the same order: no longer alphabetical.
        let english = ReviewSession::start_for(&deck, now, &[EN]);
        assert_eq!(order(&english), vec!["zeal", "anchor"]);
    }

    #[test]
    fn never_reviewed_cards_come_in_capture_order() {
        let mut deck = Deck::new();
        deck.upsert(EN, captured("apple", 9));
        deck.upsert(EN, captured("brook", 5));
        deck.upsert(EN, captured("cliff", 7));
        assert_eq!(
            order(&ReviewSession::start(&deck, DAY)),
            vec!["brook", "cliff", "apple"]
        );
    }

    #[test]
    fn spec_forty_cards_due_then_continuing() {
        let params = FsrsParams::default();
        let mut deck = Deck::new();
        for i in 0..40 {
            // A lower stability recalls worse: w00 is the most fragile.
            deck.upsert(EN, reviewed(&format!("w{i:02}"), f64::from(i + 1), 0, DAY));
        }
        let now = 50 * DAY;
        let options = SessionOptions {
            limit: Some(SESSION_CARDS),
            new_cards: None,
        };
        let mut session = ReviewSession::start_with(&deck, &params, now, &[], options);
        let expected: Vec<String> = (0..10).map(|i| format!("w{i:02}")).collect();
        assert_eq!(order(&session), expected);

        // « Encore 10 »: what was graded is no longer due, the next ten come.
        while session.current_key().is_some() {
            session.grade(&mut deck, &params, Rating::Good, now);
        }
        let next = ReviewSession::start_with(&deck, &params, now, &[], options);
        let expected: Vec<String> = (10..20).map(|i| format!("w{i:02}")).collect();
        assert_eq!(order(&next), expected);
    }

    #[test]
    fn spec_new_words_among_reviews() {
        let mut deck = Deck::new();
        for i in 0..12 {
            deck.upsert(EN, reviewed(&format!("r{i:02}"), f64::from(i + 1), 0, DAY));
        }
        for i in 0..5 {
            deck.upsert(EN, captured(&format!("n{i}"), i));
        }
        let session = ReviewSession::start_with(
            &deck,
            &FsrsParams::default(),
            20 * DAY,
            &[],
            limited(10, 20 * DAY),
        );
        let kinds: String = order(&session)
            .iter()
            .map(|w| w.chars().next().unwrap_or('?'))
            .collect();
        assert_eq!(kinds, "rrrnrrrnrr");
    }

    #[test]
    fn spec_a_heavy_reading_day_then_the_next_day() {
        let params = FsrsParams::default();
        let day_start = 100 * DAY;
        let mut deck = Deck::new();
        for i in 0..40 {
            deck.upsert(EN, captured(&format!("c{i:02}"), day_start + i));
        }
        let now = day_start + 3_600;
        let mut session =
            ReviewSession::start_with(&deck, &params, now, &[], limited(10, day_start));
        assert_eq!(session.remaining(), 10);
        while session.current_key().is_some() {
            session.grade(&mut deck, &params, Rating::Good, now);
        }
        assert_eq!(deck.introduced_since(EN, day_start), 10);

        // Later the same day: the allowance is spent, the other 30 wait.
        let again =
            ReviewSession::start_with(&deck, &params, now + 60, &[], limited(10, day_start));
        assert_eq!(again.remaining(), 0);
        assert_eq!(deck.due_count(now + 60), 30);

        // The next day: ten more of the waiting words.
        let tomorrow = day_start + DAY;
        let next =
            ReviewSession::start_with(&deck, &params, tomorrow + 60, &[], limited(10, tomorrow));
        let expected: Vec<String> = (10..20).map(|i| format!("c{i:02}")).collect();
        assert_eq!(order(&next), expected);
    }

    #[test]
    fn spec_two_languages_each_have_their_daily_allowance() {
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        let params = FsrsParams::default();
        let day_start = 100 * DAY;
        let mut deck = Deck::new();
        for i in 0..12 {
            deck.upsert(EN, captured(&format!("e{i:02}"), day_start + i));
            deck.upsert(ES, captured(&format!("s{i:02}"), day_start + i));
        }
        let now = day_start + 3_600;
        let mut morning =
            ReviewSession::start_with(&deck, &params, now, &[EN], limited(10, day_start));
        while morning.current_key().is_some() {
            morning.grade(&mut deck, &params, Rating::Good, now);
        }
        assert_eq!(deck.introduced_since(EN, day_start), 10);
        assert_eq!(deck.introduced_since(ES, day_start), 0);

        // The English words spent English's allowance, not Spanish's.
        let later = now + 60;
        let spanish =
            ReviewSession::start_with(&deck, &params, later, &[ES], limited(10, day_start));
        let expected: Vec<String> = (0..10).map(|i| format!("s{i:02}")).collect();
        assert_eq!(order(&spanish), expected);
        let english =
            ReviewSession::start_with(&deck, &params, later, &[EN], limited(10, day_start));
        assert_eq!(english.remaining(), 0);

        // A session over both languages takes what is left of each one's allowance.
        let eleven = SessionOptions {
            limit: None,
            new_cards: Some(NewCardAllowance {
                per_day: 11,
                day_start,
            }),
        };
        let both = ReviewSession::start_with(&deck, &params, later, &[], eleven);
        let in_language = |language| both.queue.iter().filter(|(l, _)| *l == language).count();
        assert_eq!((in_language(EN), in_language(ES)), (1, 11));
    }

    #[test]
    fn a_larger_allowance_lets_more_new_cards_in_and_no_options_take_every_due_card() {
        let mut deck = Deck::new();
        for i in 0..25 {
            deck.upsert(EN, captured(&format!("c{i:02}"), i));
        }
        let params = FsrsParams::default();
        let twenty = SessionOptions {
            limit: None,
            new_cards: Some(NewCardAllowance {
                per_day: 20,
                day_start: 0,
            }),
        };
        assert_eq!(
            ReviewSession::start_with(&deck, &params, DAY, &[], twenty).remaining(),
            20
        );
        assert_eq!(ReviewSession::start(&deck, DAY).remaining(), 25);
    }

    #[test]
    fn spec_a_word_recovered_is_graded_once() {
        let params = FsrsParams::default();
        let mut deck = Deck::new();
        deck.upsert(EN, captured("seldom", 0));
        for (i, w) in ["alpha", "bravo", "charlie"].iter().enumerate() {
            deck.upsert(EN, captured(w, 1 + i as i64));
        }
        let mut session = ReviewSession::start(&deck, DAY);
        assert!(session.grade(&mut deck, &params, Rating::Again, DAY));
        // It comes back after the three others.
        assert_eq!(
            order(&session),
            vec!["seldom", "alpha", "bravo", "charlie", "seldom"]
        );
        for _ in 0..3 {
            assert!(session.grade(&mut deck, &params, Rating::Good, DAY));
        }
        assert_eq!(
            session.current_key().map(|(_, w)| w.as_str()),
            Some("seldom")
        );
        // The return only takes it out of the session: one review for the card.
        assert!(!session.grade(&mut deck, &params, Rating::Good, DAY));
        assert_eq!(session.remaining(), 0);
        let seldom = deck.get(EN, "seldom").unwrap();
        assert_eq!(seldom.review.reps, 1);
        assert_eq!(seldom.review.last_review, Some(DAY));
        let summary = session.summary();
        assert_eq!((summary.reviewed, summary.recovered), (4, 1));
    }

    #[test]
    fn a_missed_card_returns_after_three_other_cards_not_at_the_end() {
        let params = FsrsParams::default();
        let mut deck = Deck::new();
        for (i, w) in ["x", "a", "b", "c", "d", "e"].iter().enumerate() {
            deck.upsert(EN, captured(w, i as i64));
        }
        let mut session = ReviewSession::start(&deck, DAY);
        session.grade(&mut deck, &params, Rating::Again, DAY);
        assert_eq!(order(&session), vec!["x", "a", "b", "c", "x", "d", "e"]);
    }

    #[test]
    fn spec_missed_three_times_leaves_the_session() {
        let params = FsrsParams::default();
        let mut deck = deck_of(&["stubborn"]);
        let mut session = ReviewSession::start(&deck, DAY);
        assert!(session.grade(&mut deck, &params, Rating::Again, DAY));
        assert!(!session.grade(&mut deck, &params, Rating::Again, DAY));
        assert!(!session.grade(&mut deck, &params, Rating::Again, DAY));
        assert_eq!(
            session.remaining(),
            0,
            "asked three times: it waits for its due date"
        );
        let card = deck.get(EN, "stubborn").unwrap();
        assert_eq!(card.review.reps, 1, "one FSRS update for the whole session");
        assert!(!card.review.is_due(DAY + 1), "due again tomorrow, not now");
        assert_eq!(session.summary().recovered, 0);
    }

    #[test]
    fn spec_ignore_hides_the_word_and_keeps_the_card() {
        let mut deck = deck_of(&["netherfield", "seldom"]);
        let mut knowledge = KnowledgeState::new();
        let mut session = ReviewSession::start(&deck, DAY);
        session.ignore(&mut deck, &mut knowledge, 2 * DAY);
        assert_eq!(
            knowledge.explicit_status(EN, "netherfield"),
            Some(Status::Ignored)
        );
        let card = deck.get(EN, "netherfield").expect("kept, not deleted");
        assert!(!card.review.is_due(i64::MAX - 1), "never due again");
        assert_eq!(card.updated_at, 2 * DAY, "stamped for sync");
        // The status travels as an op stamped in milliseconds.
        let op = knowledge
            .export_statuses()
            .into_iter()
            .find(|r| r.lemma == "netherfield")
            .expect("exported");
        assert_eq!(op.status, Some(Status::Ignored));
        assert_eq!(op.updated_at, 2 * DAY * 1000);
        assert_eq!(session.remaining(), 1, "the session moved on");
        assert_eq!(session.summary().hidden, 1);
    }

    #[test]
    fn spec_end_of_a_session_says_what_it_did() {
        let params = FsrsParams::default();
        let now = 40 * DAY;
        let mut deck = Deck::new();
        // Recall 0.9 at 40 days with a 40-day stability: a « Su » schedules it months away.
        deck.upsert(EN, reviewed("held", 40.0, 0, 40 * DAY));
        deck.upsert(EN, captured("fresh", 1));
        deck.upsert(EN, captured("missed", 2));
        deck.upsert(EN, captured("known", 3));
        deck.upsert(EN, captured("hidden", 4));
        let mut knowledge = KnowledgeState::new();
        let mut session = ReviewSession::start(&deck, now);
        assert_eq!(
            order(&session),
            vec!["held", "fresh", "missed", "known", "hidden"]
        );
        session.grade(&mut deck, &params, Rating::Good, now); // held
        session.grade(&mut deck, &params, Rating::Good, now); // fresh: three days
        session.grade(&mut deck, &params, Rating::Again, now); // missed
        session.mark_known(&mut deck, &mut knowledge, now);
        session.ignore(&mut deck, &mut knowledge, now);
        session.grade(&mut deck, &params, Rating::Good, now); // missed, back and recalled
        assert!(session.current_key().is_none());
        assert_eq!(
            session.summary(),
            SessionSummary {
                reviewed: 3,
                recovered: 1,
                holding: 1,
                known: 1,
                hidden: 1,
            }
        );
    }

    #[test]
    fn new_cards_are_all_due_and_counted() {
        let deck = deck_of(&["run", "ship", "code"]);
        assert_eq!(deck.len(), 3);
        assert_eq!(deck.due_count(0), 3);
        assert_eq!(deck.due_keys(0).len(), 3);
    }

    #[test]
    fn retire_stops_a_card_coming_due_but_keeps_it() {
        let mut deck = deck_of(&["run", "ship"]);
        assert_eq!(deck.due_count(0), 2);
        assert!(deck.retire(EN, "run", 1_000)); // reclassified known/ignored elsewhere
        assert_eq!(deck.due_count(i64::MAX - 1), 1); // `run` no longer due, `ship` still is
        assert!(deck.get(EN, "run").is_some()); // kept, not deleted
        assert_eq!(deck.get(EN, "run").unwrap().updated_at, 1_000); // stamped for sync
        assert!(!deck.retire(EN, "absent", 1_000)); // no-op when there is no card
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
    fn retiring_never_dates_a_card_backwards() {
        // A status pulled from another device can be older than the last local edit; the
        // retirement it triggers must not stamp the card back in time (it would then lose
        // last-write-wins and keep coming due on the other devices).
        let mut deck = Deck::new();
        let mut card = card("run");
        card.updated_at = 9_000;
        deck.upsert(EN, card);

        assert!(deck.retire(EN, "run", 2_000));

        let stored = deck.get(EN, "run").unwrap();
        assert_eq!(stored.updated_at, 9_000);
        assert!(
            !deck
                .due_keys(10_000_000_000)
                .iter()
                .any(|(_, l)| l == "run")
        ); // year 2286
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
    fn apply_card_lww_keeps_the_local_page_address() {
        // The page address never travels, so a synced edit arrives without one.
        let mut deck = deck_of(&["run"]); // captured on https://example.com
        let mut synced = card_at("run", 5_000);
        synced.provenance.source = EncounterSource::Web { url: String::new() };
        synced.encountered_form = "ran".to_owned();
        assert!(deck.apply_card_lww(EN, synced));

        let stored = deck.get(EN, "run").unwrap();
        assert_eq!(stored.encountered_form, "ran");
        assert_eq!(
            stored.provenance.source,
            EncounterSource::Web {
                url: "https://example.com".to_owned()
            }
        );
    }

    #[test]
    fn apply_card_lww_leaves_a_new_synced_card_without_an_address() {
        let mut deck = Deck::new();
        let mut synced = card_at("city", 10);
        synced.provenance.source = EncounterSource::Web { url: String::new() };
        assert!(deck.apply_card_lww(EN, synced));
        assert_eq!(
            deck.get(EN, "city").unwrap().provenance.source,
            EncounterSource::Web { url: String::new() }
        );
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
