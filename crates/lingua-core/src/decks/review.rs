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

    /// Every card with its language, borrowed, in (language, lemma) order.
    pub fn iter(&self) -> impl Iterator<Item = (StudiedLanguage, &Card)> {
        self.cards
            .iter()
            .flat_map(|(&lang, per_lang)| per_lang.values().map(move |card| (lang, card)))
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
    /// default) — for level-targeted feeding (`add-lingua-cefr-levels`). The
    /// glosses are the pack's, written in `gloss_language`; each card is stamped
    /// with the reserved `Import` source.
    ///
    /// A lemma is skipped, with no card and no status written for it, for three
    /// reasons: it already has a card, it holds an explicit status in
    /// `knowledge` (so seeding is idempotent), or the pack gives it no gloss —
    /// a card without one would have nothing behind its answer
    /// (seed-lingua-decks-with-glossed-lemmas D1). A skipped lemma takes no
    /// place under `cap`: the next lemma, in the caller's order, takes it (D2).
    /// Stops after `cap` new cards. Returns the number actually added.
    pub fn seed_lemmas<'a>(
        &mut self,
        lang: StudiedLanguage,
        lemmas: impl IntoIterator<Item = (&'a str, Option<&'a str>)>,
        gloss_language: &str,
        knowledge: &KnowledgeState,
        cap: usize,
        at: i64,
    ) -> usize {
        let mut added = 0;
        for (lemma, gloss) in lemmas {
            if added >= cap {
                break;
            }
            let Some(gloss) = gloss else {
                continue;
            };
            if self.get(lang, lemma).is_some() || knowledge.explicit_status(lang, lemma).is_some() {
                continue;
            }
            self.upsert(
                lang,
                Card::seeded(lemma, gloss.to_owned(), gloss_language, at),
            );
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

    /// Keeps `gloss` as the gloss the reader chose for the card of `lemma`
    /// (add-lingua-review-translation-fallback D3, D5): written in `gloss_language`, marked
    /// [`Card::gloss_by_reader`], the card stamped for sync without moving its clock back.
    /// Returns whether a card was there; the caller checks the text.
    pub fn keep_gloss(
        &mut self,
        lang: StudiedLanguage,
        lemma: &str,
        gloss: &str,
        gloss_language: &str,
        now: i64,
    ) -> bool {
        let Some(card) = self.cards.get_mut(&lang).and_then(|m| m.get_mut(lemma)) else {
            return false;
        };
        card.gloss = Some(gloss.to_owned());
        card.gloss_language = gloss_language.to_owned();
        card.gloss_by_reader = true;
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

    /// The number of cards a review started at `now` would hold in `languages` — every language
    /// when it is empty — with no cap (add-lingua-comeback-hints D1): the reviewed cards due,
    /// and in each language the never-reviewed cards due within what is left of that language's
    /// `allowance` today, every one of them without an allowance. Exactly the length of the
    /// queue [`ReviewSession::start_with`] builds with no `limit`: a reviewed card is one with a
    /// memory, as the session sorts it.
    pub fn reviewable_count_for(
        &self,
        now: i64,
        languages: &[StudiedLanguage],
        allowance: Option<NewCardAllowance>,
    ) -> usize {
        self.cards
            .iter()
            .filter(|(language, _)| languages.is_empty() || languages.contains(language))
            .map(|(&language, per_language)| {
                let (reviewed, fresh) = per_language
                    .values()
                    .filter(|card| card.review.is_due(now))
                    .fold((0, 0), |(reviewed, fresh), card| {
                        if card.review.memory.is_some() {
                            (reviewed + 1, fresh)
                        } else {
                            (reviewed, fresh + 1)
                        }
                    });
                let admitted = allowance.map_or(fresh, |allowance| {
                    fresh.min(
                        allowance
                            .per_day
                            .saturating_sub(self.introduced_since(language, allowance.day_start)),
                    )
                });
                reviewed + admitted
            })
            .sum()
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
            // The echo of a gloss the reader chose (add-lingua-review-translation-fallback D6):
            // a server or a client that predates the mark sends the same text back without
            // it. The text is the reader's, so the mark stays.
            if existing.gloss_by_reader && !card.gloss_by_reader && card.gloss == existing.gloss {
                card.gloss_by_reader = true;
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
    /// A practice session (add-lingua-review-practice D1): the reviewed, non-retired cards,
    /// due or not, whose answers change no card. Ignores `new_cards`.
    pub practice: bool,
}

/// What a session did, for its end (refine-lingua-review-session D8).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Whether its answers leave every card as it was (add-lingua-review-practice D2).
    practice: bool,
}

/// A session written out: what an engine that holds it keeps across its own restart. The asked
/// cards are a list, since JSON keys a map by strings only.
#[derive(Serialize, Deserialize)]
struct SessionSnapshot {
    queue: Vec<(StudiedLanguage, String)>,
    position: usize,
    revealed: bool,
    asked: Vec<(StudiedLanguage, String, u8, bool)>,
    summary: SessionSummary,
    /// Absent from a snapshot an older engine wrote: that session was a review.
    #[serde(default)]
    practice: bool,
}

impl ReviewSession {
    /// The session as text, to resume it with [`Self::from_snapshot`] — on Safari the engine that
    /// holds it lives in a background page the browser stops between two answers
    /// (refine-lingua-review-session D3).
    pub fn to_snapshot(&self) -> String {
        let snapshot = SessionSnapshot {
            queue: self.queue.clone(),
            position: self.position,
            revealed: self.revealed,
            asked: self
                .asked
                .iter()
                .map(|((language, lemma), asked)| {
                    (*language, lemma.clone(), asked.times, asked.missed)
                })
                .collect(),
            summary: self.summary,
            practice: self.practice,
        };
        serde_json::to_string(&snapshot).expect("a session is always serialisable")
    }

    /// Whether every card of the session is still in `deck`: a state restored under a session —
    /// most often the very state the session just saved, echoed back by another surface of the
    /// same engine — keeps it then, and a deck that lost one of its cards ends it (D12).
    pub fn fits(&self, deck: &Deck) -> bool {
        self.queue
            .iter()
            .all(|(language, lemma)| deck.get(*language, lemma).is_some())
    }

    /// The session [`Self::to_snapshot`] wrote, or `None` for text that is not one.
    pub fn from_snapshot(json: &str) -> Option<Self> {
        let s: SessionSnapshot = serde_json::from_str(json).ok()?;
        if s.position > s.queue.len() {
            return None;
        }
        Some(Self {
            queue: s.queue,
            position: s.position,
            revealed: s.revealed,
            asked: s
                .asked
                .into_iter()
                .map(|(language, lemma, times, missed)| {
                    ((language, lemma), Asked { times, missed })
                })
                .collect(),
            summary: s.summary,
            practice: s.practice,
        })
    }

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
        if options.practice {
            return Self::start_practice(deck, params, now, languages, options.limit);
        }
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
            practice: false,
        }
    }

    /// Starts a practice session (add-lingua-review-practice D1): the reviewed cards in
    /// `languages` (every language when it is empty) that are not retired, due or not, the
    /// lowest predicted recall first, then by due date, language and lemma, up to `limit`.
    /// No never-reviewed card: practice is for words already met in review.
    fn start_practice(
        deck: &Deck,
        params: &FsrsParams,
        now: i64,
        languages: &[StudiedLanguage],
        limit: Option<usize>,
    ) -> Self {
        let mut cards: Vec<(f64, i64, (StudiedLanguage, String))> = deck
            .cards
            .iter()
            .filter(|(language, _)| languages.is_empty() || languages.contains(language))
            .flat_map(|(&language, per_language)| {
                per_language.iter().filter_map(move |(lemma, card)| {
                    let due = card.review.due?;
                    if due == i64::MAX {
                        return None; // retired: known or hidden
                    }
                    let recall = card.review.retrievability_at(params, now)?;
                    Some((recall, due, (language, lemma.clone())))
                })
            })
            .collect();
        cards.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let queue = cards
            .into_iter()
            .map(|(_, _, key)| key)
            .take(limit.unwrap_or(usize::MAX))
            .collect();
        Self {
            queue,
            position: 0,
            revealed: false,
            asked: BTreeMap::new(),
            summary: SessionSummary::default(),
            practice: true,
        }
    }

    /// Whether this is a practice session, whose answers change no card.
    pub fn is_practice(&self) -> bool {
        self.practice
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

    /// Answers the current card and advances; returns whether the answer counts as a review —
    /// a card's first answer, which updates its FSRS state, except in a practice session, where
    /// it changes no card (add-lingua-review-practice D2). Ignored, returning `false`, when the
    /// session is finished.
    ///
    /// Only a card's first answer in the session is graded (refine-lingua-review-session D3):
    /// a missed card (`Again`) comes back after three other cards, or at the end when fewer
    /// remain — never when none does, which would show it again at once — until it is recalled
    /// or has been asked three times, and those later answers
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
        if first && self.practice {
            // Practice (D2): counted as a review, no card changed — no schedule, no sync stamp.
            graded = true;
            self.summary.reviewed += 1;
        } else if first
            && let Some(card) = deck.cards.get_mut(&key.0).and_then(|m| m.get_mut(&key.1))
        {
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
        // A missed card comes back only after another card: the last one of a session would
        // otherwise be shown again at once, up to three times in a row (add-lingua-review-repeat-hint D4).
        let others_left = self.position + 1 < self.queue.len();
        if missed && times < MAX_ASKS && others_left {
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
            "fr",
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
            practice: false,
            limit: Some(SESSION_CARDS),
            new_cards: Some(NewCardAllowance { per_day, day_start }),
        }
    }

    fn practice(limit: Option<usize>) -> SessionOptions {
        SessionOptions {
            practice: true,
            limit,
            new_cards: Some(NewCardAllowance {
                per_day: 0,
                day_start: 0,
            }),
        }
    }

    #[test]
    fn spec_the_last_card_missed_is_not_shown_again_at_once() {
        let mut deck = deck_of(&["alpha", "bravo"]);
        let params = FsrsParams::default();
        let mut session = ReviewSession::start(&deck, 0);
        // `alpha` missed with `bravo` still to come: it comes back after it.
        assert!(session.grade(&mut deck, &params, Rating::Again, 0));
        assert_eq!(order(&session)[session.position..], ["bravo", "alpha"]);
        assert!(session.grade(&mut deck, &params, Rating::Good, 0));
        // `alpha` missed again, nothing left after it: the session ends instead of repeating it.
        assert!(!session.grade(&mut deck, &params, Rating::Again, 0));
        assert!(session.current_key().is_none());
        let summary = session.summary();
        assert_eq!((summary.reviewed, summary.recovered), (2, 0));

        // A session of one card missed: graded once, not shown again.
        let mut deck = deck_of(&["alone"]);
        let mut session = ReviewSession::start(&deck, 0);
        assert!(session.grade(&mut deck, &params, Rating::Again, 0));
        assert!(session.current_key().is_none());
    }

    #[test]
    fn spec_a_practice_session_holds_the_reviewed_cards_not_retired_due_or_not() {
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        let mut deck = Deck::new();
        // At 1 day: `zeal` (stability 2) is the most fragile, then `faro`, then `anchor`.
        deck.upsert(EN, reviewed("anchor", 30.0, 0, 30 * DAY));
        deck.upsert(EN, reviewed("zeal", 2.0, 0, 2 * DAY));
        deck.upsert(ES, reviewed("faro", 10.0, 0, 10 * DAY));
        deck.upsert(EN, captured("fresh", 0)); // never reviewed: not practice
        let mut known = reviewed("known", 1.0, 0, DAY);
        known.review.retire(0); // retired: not practice
        deck.upsert(EN, known);
        let params = FsrsParams::default();
        let now = DAY;
        assert_eq!(
            deck.due_count(now),
            1,
            "only the never-reviewed card is due"
        );

        let all = ReviewSession::start_with(&deck, &params, now, &[], practice(None));
        assert!(all.is_practice());
        assert_eq!(order(&all), ["zeal", "faro", "anchor"]);
        let english = ReviewSession::start_with(&deck, &params, now, &[EN], practice(None));
        assert_eq!(order(&english), ["zeal", "anchor"]);
        let one = ReviewSession::start_with(&deck, &params, now, &[], practice(Some(1)));
        assert_eq!(order(&one), ["zeal"]);
        assert!(!ReviewSession::start(&deck, now).is_practice());
    }

    #[test]
    fn spec_a_practice_answer_changes_no_card_and_counts_a_review() {
        let mut deck = Deck::new();
        deck.upsert(EN, reviewed("zeal", 2.0, 0, 2 * DAY));
        deck.upsert(EN, reviewed("anchor", 30.0, 0, 30 * DAY));
        let before = deck.clone();
        let params = FsrsParams::default();
        let now = DAY;
        let mut session = ReviewSession::start_with(&deck, &params, now, &[], practice(None));

        // Missed: it comes back (fewer than three others, so at the end), counted once.
        assert!(session.grade(&mut deck, &params, Rating::Again, now));
        assert_eq!(order(&session)[session.position..], ["anchor", "zeal"]);
        assert!(session.grade(&mut deck, &params, Rating::Good, now));
        assert!(
            !session.grade(&mut deck, &params, Rating::Good, now),
            "a second answer is no review"
        );
        assert!(session.current_key().is_none());

        for lemma in ["zeal", "anchor"] {
            let (was, is) = (before.get(EN, lemma).unwrap(), deck.get(EN, lemma).unwrap());
            assert_eq!(
                is.review, was.review,
                "{lemma}: schedule, memory and history unchanged"
            );
            assert_eq!(is.updated_at, was.updated_at, "{lemma}: nothing to sync");
        }
        let summary = session.summary();
        assert_eq!(
            (summary.reviewed, summary.recovered, summary.holding),
            (2, 1, 0)
        );
    }

    #[test]
    fn spec_a_practice_session_survives_its_snapshot_and_an_old_snapshot_is_a_review() {
        let mut deck = Deck::new();
        deck.upsert(EN, reviewed("zeal", 2.0, 0, 2 * DAY));
        let params = FsrsParams::default();
        let session = ReviewSession::start_with(&deck, &params, DAY, &[], practice(None));
        let resumed = ReviewSession::from_snapshot(&session.to_snapshot()).unwrap();
        assert!(resumed.is_practice());
        assert_eq!(order(&resumed), ["zeal"]);

        // What an engine before practice wrote: the same snapshot without the field.
        let old = session.to_snapshot().replace(r#","practice":true"#, "");
        assert!(!old.contains("practice"));
        assert!(!ReviewSession::from_snapshot(&old).unwrap().is_practice());
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
    fn a_session_written_out_resumes_where_it_was() {
        let params = FsrsParams::default();
        let mut deck = Deck::new();
        for (i, w) in ["seldom", "dwell", "grin", "thrive"].iter().enumerate() {
            deck.upsert(EN, captured(w, i as i64));
        }
        let mut session = ReviewSession::start(&deck, DAY);
        session.grade(&mut deck, &params, Rating::Again, DAY);
        session.reveal();

        let resumed =
            ReviewSession::from_snapshot(&session.to_snapshot()).expect("a snapshot reads back");
        assert_eq!(resumed.queue, session.queue);
        assert_eq!(resumed.current_key(), session.current_key());
        assert!(resumed.is_revealed());
        assert_eq!(resumed.summary(), session.summary());
        assert_eq!(resumed.to_snapshot(), session.to_snapshot());
        assert!(ReviewSession::from_snapshot("not a session").is_none());
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
            practice: false,
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
            practice: false,
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

    /// English and Spanish cards of every kind: reviewed and due, reviewed and due later,
    /// never reviewed, retired after review, retired before any.
    fn mixed_deck(day_start: i64) -> Deck {
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        let mut deck = Deck::new();
        for (language, prefix) in [(EN, "e"), (ES, "s")] {
            for i in 0..4 {
                let due = day_start - DAY + i;
                deck.upsert(
                    language,
                    reviewed(&format!("{prefix}due{i}"), 3.0, due - 3 * DAY, due),
                );
            }
            for i in 0..3 {
                let due = day_start + (i + 2) * 3_600;
                deck.upsert(
                    language,
                    reviewed(&format!("{prefix}later{i}"), 3.0, due - 3 * DAY, due),
                );
            }
            for i in 0..14 {
                deck.upsert(
                    language,
                    captured(&format!("{prefix}new{i:02}"), day_start - DAY + i),
                );
            }
            let mut known = reviewed(&format!("{prefix}known"), 9.0, day_start - DAY, day_start);
            known.review.retire(day_start);
            deck.upsert(language, known);
            let mut hidden = captured(&format!("{prefix}hidden"), day_start - DAY);
            hidden.review.retire(day_start);
            deck.upsert(language, hidden);
        }
        deck
    }

    /// `reviewable_count_for` against the queue a session would build, for every option.
    fn assert_reviewable_counts_match_sessions(deck: &Deck, now: i64, day_start: i64) {
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        let params = FsrsParams::default();
        for languages in [&[][..], &[EN], &[ES], &[EN, ES]] {
            for allowance in [
                None,
                Some(NewCardAllowance {
                    per_day: 10,
                    day_start,
                }),
                Some(NewCardAllowance {
                    per_day: 5,
                    day_start,
                }),
                Some(NewCardAllowance {
                    per_day: 0,
                    day_start,
                }),
            ] {
                let options = SessionOptions {
                    practice: false,
                    limit: None,
                    new_cards: allowance,
                };
                let session = ReviewSession::start_with(deck, &params, now, languages, options);
                assert_eq!(
                    deck.reviewable_count_for(now, languages, allowance),
                    session.remaining(),
                    "at {now}, in {languages:?}, with {allowance:?}"
                );
            }
        }
    }

    #[test]
    fn spec_the_count_is_what_a_review_would_hold_today() {
        let day_start = 100 * DAY;
        let mut deck = mixed_deck(day_start);
        let morning = day_start + 3_600;
        // 4 reviewed cards due, 14 new words of which the allowance lets 10 in: 14 a language.
        let allowance = Some(NewCardAllowance {
            per_day: 10,
            day_start,
        });
        assert_eq!(deck.reviewable_count_for(morning, &[EN], allowance), 14);
        assert_eq!(deck.reviewable_count_for(morning, &[], allowance), 28);
        assert_eq!(deck.reviewable_count_for(morning, &[EN], None), 18);
        assert_eq!(deck.due_count_for(morning, &[EN]), 18, "the old figure");
        for now in [morning, day_start + 5 * 3_600, day_start + DAY - 1] {
            assert_reviewable_counts_match_sessions(&deck, now, day_start);
        }

        // A session of six English cards this morning: the four reviewed ones due and two new
        // words (the 4th and the 6th places), all recalled.
        let params = FsrsParams::default();
        let six = SessionOptions {
            practice: false,
            limit: Some(6),
            new_cards: allowance,
        };
        let mut session = ReviewSession::start_with(&deck, &params, morning, &[EN], six);
        while session.current_key().is_some() {
            session.grade(&mut deck, &params, Rating::Good, morning);
        }
        assert_eq!(deck.introduced_since(EN, day_start), 2);
        // Nothing reviewed is due a minute later, and 8 of English's allowance are left.
        let after = morning + 60;
        assert_eq!(deck.reviewable_count_for(after, &[EN], allowance), 8);
        for now in [after, day_start + 5 * 3_600, day_start + DAY + 3_600] {
            assert_reviewable_counts_match_sessions(&deck, now, day_start);
            assert_reviewable_counts_match_sessions(&deck, now, day_start + DAY);
        }
    }

    #[test]
    fn a_larger_allowance_lets_more_new_cards_in_and_no_options_take_every_due_card() {
        let mut deck = Deck::new();
        for i in 0..25 {
            deck.upsert(EN, captured(&format!("c{i:02}"), i));
        }
        let params = FsrsParams::default();
        let twenty = SessionOptions {
            practice: false,
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
            ("run", Some("courir")),          // already carded → skip
            ("city", Some("ville")),          // explicit status → skip
            ("nuance", Some("nuance")),       // new → add
            ("quixotic", Some("chimérique")), // new → add
            ("arcane", Some("ésotérique")),   // would add, but the cap stops us first
        ];
        let added = deck.seed_lemmas(EN, candidates, "fr", &knowledge, 2, 5 * DAY);
        assert_eq!(added, 2);
        assert!(deck.get(EN, "nuance").is_some());
        assert_eq!(
            deck.get(EN, "quixotic").unwrap().gloss.as_deref(),
            Some("chimérique")
        );
        assert!(deck.get(EN, "arcane").is_none()); // capped
        assert_eq!(
            deck.get(EN, "nuance").unwrap().provenance.source,
            EncounterSource::Import
        );
        assert_eq!(deck.get(EN, "nuance").unwrap().updated_at, 5 * DAY);
    }

    #[test]
    fn seed_lemmas_skips_a_lemma_without_a_gloss_and_takes_the_next() {
        // *The rarest words of a level have no gloss* (seed-lingua-decks-with-glossed-lemmas
        // D1, D2): a lemma the pack does not gloss gets no card and no status, and takes no
        // place under the cap — the next glossed lemma, in the caller's order, takes it.
        let mut deck = Deck::new();
        let knowledge = KnowledgeState::new();
        let candidates = [
            ("quixotic", None),         // no gloss → skip
            ("nuance", Some("nuance")), // add
            ("arcane", None),           // no gloss → skip
            ("lucid", Some("lucide")),  // add
            ("zeal", Some("zèle")),     // would add, but the cap stops us first
        ];
        let added = deck.seed_lemmas(EN, candidates, "fr", &knowledge, 2, 5 * DAY);
        assert_eq!(added, 2);
        let carded: Vec<&str> = deck.iter().map(|(_, card)| card.lemma.as_str()).collect();
        assert_eq!(carded, ["lucid", "nuance"]);
        assert_eq!(
            deck.get(EN, "lucid").unwrap().gloss.as_deref(),
            Some("lucide")
        );
        assert!(deck.get(EN, "zeal").is_none()); // capped
        for lemma in ["quixotic", "arcane"] {
            assert!(deck.get(EN, lemma).is_none(), "{lemma}");
            assert_eq!(knowledge.explicit_status(EN, lemma), None, "{lemma}");
        }
    }

    #[test]
    fn a_level_whose_glossed_lemmas_are_tracked_seeds_nothing() {
        // *Only lemmas without a gloss are left*: every glossed lemma already has a card or a
        // status, so seeding adds nothing and writes nothing.
        let mut deck = deck_of(&["run"]);
        let mut knowledge = KnowledgeState::new();
        knowledge.set_status(EN, "city", Status::Learning);
        let (deck_before, knowledge_before) = (deck.clone(), knowledge.clone());
        let candidates = [
            ("run", Some("courir")),
            ("quixotic", None),
            ("city", Some("ville")),
            ("arcane", None),
        ];
        assert_eq!(
            deck.seed_lemmas(EN, candidates, "fr", &knowledge, 20, 5 * DAY),
            0
        );
        assert_eq!(deck, deck_before);
        assert_eq!(knowledge, knowledge_before);
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
    fn keeping_a_gloss_marks_the_card_and_never_dates_it_backwards() {
        let mut deck = Deck::new();
        deck.upsert(EN, card_at("slacken", 9_000));
        assert!(!deck.keep_gloss(EN, "absent", "x", "fr", 10_000));
        for (now, stamped) in [(2_000, 9_000), (10_000, 10_000)] {
            assert!(deck.keep_gloss(EN, "slacken", "se relâcher", "en", now));
            let kept = deck.get(EN, "slacken").unwrap();
            assert_eq!(kept.gloss.as_deref(), Some("se relâcher"));
            assert_eq!(kept.gloss_language, "en");
            assert!(kept.gloss_by_reader);
            assert_eq!(kept.updated_at, stamped, "kept at {now}");
        }
    }

    #[test]
    fn apply_card_lww_keeps_the_readers_mark_on_an_echo_only() {
        // (local: marked, gloss) (incoming: marked, gloss, ts) → stored (marked, gloss)
        // add-lingua-review-translation-fallback D6, scenario « An echo without the mark ».
        type Side<'a> = (bool, Option<&'a str>);
        let cases: [(Side, Side, i64, Side); 6] = [
            // The echo: same text, no mark — the newer card lands and the mark stays.
            (
                (true, Some("se relâcher")),
                (false, Some("se relâcher")),
                200,
                (true, Some("se relâcher")),
            ),
            // Another text without the mark: the newer card wins, mark and all.
            (
                (true, Some("se relâcher")),
                (false, Some("détendre")),
                200,
                (false, Some("détendre")),
            ),
            // No gloss coming back is not an echo.
            (
                (true, Some("se relâcher")),
                (false, None),
                200,
                (false, None),
            ),
            // A marked card from elsewhere carries its mark in.
            (
                (false, Some("détendre")),
                (true, Some("se relâcher")),
                200,
                (true, Some("se relâcher")),
            ),
            // An unmarked local card stays unmarked on an unmarked echo.
            (
                (false, Some("détendre")),
                (false, Some("détendre")),
                200,
                (false, Some("détendre")),
            ),
            // An older echo loses as any older card does.
            (
                (true, Some("se relâcher")),
                (false, Some("détendre")),
                50,
                (true, Some("se relâcher")),
            ),
        ];
        for (i, ((local_mark, local_gloss), (in_mark, in_gloss), ts, (mark, gloss))) in
            cases.into_iter().enumerate()
        {
            let mut local = card_at("slacken", 100);
            local.gloss = local_gloss.map(str::to_owned);
            local.gloss_by_reader = local_mark;
            let mut deck = Deck::new();
            deck.upsert(EN, local);
            let mut incoming = card_at("slacken", ts);
            incoming.gloss = in_gloss.map(str::to_owned);
            incoming.gloss_by_reader = in_mark;
            deck.apply_card_lww(EN, incoming);
            let stored = deck.get(EN, "slacken").unwrap();
            assert_eq!(
                (stored.gloss_by_reader, stored.gloss.as_deref()),
                (mark, gloss),
                "case {i}"
            );
        }
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
