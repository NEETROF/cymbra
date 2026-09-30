// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The Deck domain module: whole cards synced by stable client id within a studied
//! language, last-write-wins per card (tombstones included). A pull names the languages
//! the client accepts and receives nothing else (add-lingua-card-language), so a client
//! that predates card languages — it names none, so English only — never receives a card
//! it would mis-file. Same clamp + LWW rules as KnownWords
//! ([`known_words_core`]). Media contents never cross the wire (allow-list) — there is
//! no media field — and neither does the page a card was captured from
//! (add-lingua-privacy-controls): a card has no source here. Ops dated before the user's
//! erasure mark are dropped ([`data_core`](crate::data_core)).

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_platform::Result;

use crate::data::ErasureMarks;
use crate::data_core::predates_erasure;
use crate::known_words_core::clamp_ts;
use crate::language_core::accepted_languages;

/// A whole card as one device holds it (a tombstone when `deleted`). On a pull,
/// `sequence` is the server's change sequence; on a push it is ignored.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Card {
    /// The studied language; normalised by the adapter, `en` when the client sent none.
    pub language: String,
    pub client_id: String,
    pub lemma: String,
    pub surface_form: String,
    pub source_sentence: String,
    pub gloss: String,
    pub fsrs_state: String,
    pub deleted: bool,
    pub updated_at: i64,
    pub device_id: String,
    pub sequence: i64,
}

/// Storage port for cards.
#[async_trait]
pub trait DeckRepo: Send + Sync {
    /// Apply one card op (its `updated_at` is already clamped). Returns whether it won
    /// LWW and changed stored state; a winning op is assigned the next change sequence.
    async fn apply_card(&self, user: &str, card: &Card) -> Result<bool>;
    async fn tip_cursor(&self, user: &str) -> Result<i64>;
    /// Cards with `sequence > cursor` whose language is in `languages`, in sequence order
    /// (tombstones included).
    async fn changes_since(
        &self,
        user: &str,
        cursor: i64,
        languages: &[String],
    ) -> Result<Vec<Card>>;
}

/// Orchestrates card sync over a [`DeckRepo`].
pub struct DeckModule {
    repo: Arc<dyn DeckRepo>,
    marks: Arc<dyn ErasureMarks>,
}

impl DeckModule {
    pub fn new(repo: Arc<dyn DeckRepo>, marks: Arc<dyn ErasureMarks>) -> Self {
        Self { repo, marks }
    }

    /// Drain a client's cards: clamp, drop what predates the user's erasure, apply LWW.
    /// Returns how many changed state plus the new tip cursor; dropped cards still count
    /// as acknowledged (they are simply not stored).
    pub async fn push_cards(&self, user: &str, cards: Vec<Card>, now: i64) -> Result<(u64, i64)> {
        let erased_at = self.marks.erased_at(user).await?;
        let mut applied = 0u64;
        for mut card in cards {
            card.updated_at = clamp_ts(card.updated_at, now);
            if predates_erasure(card.updated_at, erased_at) {
                continue;
            }
            if self.repo.apply_card(user, &card).await? {
                applied += 1;
            }
        }
        Ok((applied, self.repo.tip_cursor(user).await?))
    }

    /// The cards after `cursor` in the languages the client accepts (none named = English
    /// only). The cursor returned is the highest sequence among the cards returned, or
    /// `cursor` when none was: cards in other languages are withheld, not consumed, so a
    /// client that later accepts more languages can pull again from an earlier cursor.
    pub async fn pull_cards(
        &self,
        user: &str,
        cursor: i64,
        languages: &[String],
    ) -> Result<(Vec<Card>, i64)> {
        let accepted = accepted_languages(languages);
        let cards = self.repo.changes_since(user, cursor, &accepted).await?;
        let next = cards.iter().map(|c| c.sequence).max().unwrap_or(cursor);
        Ok((cards, next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MockErasureMarks;
    use crate::known_words_core::wins;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeDeckRepo {
        // user -> (language, client_id) -> card
        #[allow(clippy::type_complexity)]
        rows: Mutex<HashMap<String, HashMap<(String, String), Card>>>,
        seq: Mutex<i64>,
    }

    #[async_trait]
    impl DeckRepo for FakeDeckRepo {
        async fn apply_card(&self, user: &str, card: &Card) -> Result<bool> {
            let mut rows = self.rows.lock().unwrap();
            let per_user = rows.entry(user.to_owned()).or_default();
            let key = (card.language.clone(), card.client_id.clone());
            if let Some(existing) = per_user.get(&key)
                && !wins(
                    existing.updated_at,
                    &existing.device_id,
                    card.updated_at,
                    &card.device_id,
                )
            {
                return Ok(false);
            }
            let mut seq = self.seq.lock().unwrap();
            *seq += 1;
            let mut stored = card.clone();
            stored.sequence = *seq;
            per_user.insert(key, stored);
            Ok(true)
        }

        async fn tip_cursor(&self, user: &str) -> Result<i64> {
            let rows = self.rows.lock().unwrap();
            Ok(rows
                .get(user)
                .into_iter()
                .flat_map(|m| m.values())
                .map(|c| c.sequence)
                .max()
                .unwrap_or(0))
        }

        async fn changes_since(
            &self,
            user: &str,
            cursor: i64,
            languages: &[String],
        ) -> Result<Vec<Card>> {
            let rows = self.rows.lock().unwrap();
            let mut out: Vec<Card> = rows
                .get(user)
                .into_iter()
                .flat_map(|m| m.values())
                .filter(|c| c.sequence > cursor && languages.contains(&c.language))
                .cloned()
                .collect();
            out.sort_by_key(|c| c.sequence);
            Ok(out)
        }
    }

    fn marks(erased_at: i64) -> Arc<dyn ErasureMarks> {
        let mut marks = MockErasureMarks::new();
        marks.expect_erased_at().returning(move |_| Ok(erased_at));
        Arc::new(marks)
    }

    fn never_erased() -> Arc<dyn ErasureMarks> {
        marks(0)
    }

    fn card(id: &str, sentence: &str, ts: i64, device: &str) -> Card {
        Card {
            language: "en".into(),
            client_id: id.into(),
            lemma: "seldom".into(),
            surface_form: "seldom".into(),
            source_sentence: sentence.into(),
            gloss: "rarement".into(),
            fsrs_state: "{}".into(),
            deleted: false,
            updated_at: ts,
            device_id: device.into(),
            sequence: 0,
        }
    }

    #[tokio::test]
    async fn card_syncs_with_its_sentence_and_review_state() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()), never_erased());
        module
            .push_cards(
                "u1",
                vec![card("c1", "They seldom ship.", 100, "iphone")],
                1_000,
            )
            .await
            .unwrap();
        let (cards, _) = module.pull_cards("u1", 0, &[]).await.unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].source_sentence, "They seldom ship.");
        assert!(!cards[0].deleted);
    }

    #[tokio::test]
    async fn deletion_propagates_by_lww() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()), never_erased());
        module
            .push_cards("u1", vec![card("c1", "s", 100, "iphone")], 1_000)
            .await
            .unwrap();
        let mut tomb = card("c1", "s", 200, "mac");
        tomb.deleted = true;
        module.push_cards("u1", vec![tomb], 1_000).await.unwrap();
        let (cards, _) = module.pull_cards("u1", 0, &[]).await.unwrap();
        assert_eq!(cards.len(), 1);
        assert!(cards[0].deleted); // tombstone wins and is pulled so clients delete
    }

    #[tokio::test]
    async fn stale_edit_loses_to_the_newer_card() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()), never_erased());
        module
            .push_cards("u1", vec![card("c1", "new", 200, "mac")], 1_000)
            .await
            .unwrap();
        let (applied, _) = module
            .push_cards("u1", vec![card("c1", "old", 100, "iphone")], 1_000)
            .await
            .unwrap();
        assert_eq!(applied, 0);
        assert_eq!(
            module
                .repo
                .changes_since("u1", 0, &["en".to_string()])
                .await
                .unwrap()[0]
                .source_sentence,
            "new"
        );
    }

    #[tokio::test]
    async fn cards_dated_before_the_erasure_are_not_stored() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()), marks(500));
        let (applied, _) = module
            .push_cards(
                "u1",
                vec![card("old", "s", 400, "mac"), card("edge", "s", 500, "mac")],
                1_000,
            )
            .await
            .unwrap();
        assert_eq!(applied, 0);
        assert!(module.pull_cards("u1", 0, &[]).await.unwrap().0.is_empty());
    }

    #[tokio::test]
    async fn a_card_made_after_the_erasure_syncs() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()), marks(500));
        let (applied, _) = module
            .push_cards("u1", vec![card("new", "s", 501, "mac")], 1_000)
            .await
            .unwrap();
        assert_eq!(applied, 1);
        assert_eq!(
            module.pull_cards("u1", 0, &[]).await.unwrap().0[0].client_id,
            "new"
        );
    }

    #[tokio::test]
    async fn a_future_dated_card_is_judged_at_its_receipt_time() {
        // Clamped to `now` (400) first, so a skewed clock cannot slip past the mark.
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()), marks(500));
        let (applied, _) = module
            .push_cards("u1", vec![card("skewed", "s", 9_999, "mac")], 400)
            .await
            .unwrap();
        assert_eq!(applied, 0);
    }

    fn spanish(id: &str, ts: i64, device: &str) -> Card {
        Card {
            language: "es".into(),
            lemma: id.into(),
            surface_form: id.into(),
            gloss: "fils".into(),
            ..card(id, "Tiene dos hijos y un hijo.", ts, device)
        }
    }

    #[tokio::test]
    async fn the_same_client_id_in_two_languages_is_two_cards() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()), never_erased());
        let (applied, _) = module
            .push_cards(
                "u1",
                vec![
                    card("son", "Their son left.", 100, "mac"),
                    spanish("son", 100, "mac"),
                ],
                1_000,
            )
            .await
            .unwrap();
        assert_eq!(applied, 2);
        // Editing the Spanish one leaves the English one untouched.
        let mut edited = spanish("son", 200, "mac");
        edited.gloss = "ils sont".into();
        module.push_cards("u1", vec![edited], 1_000).await.unwrap();
        let both = ["en".to_string(), "es".to_string()];
        let (cards, _) = module.pull_cards("u1", 0, &both).await.unwrap();
        assert_eq!(cards.len(), 2);
        let english = cards.iter().find(|c| c.language == "en").unwrap();
        let spanish_card = cards.iter().find(|c| c.language == "es").unwrap();
        assert_eq!(english.source_sentence, "Their son left.");
        assert_eq!(spanish_card.gloss, "ils sont");
    }

    #[tokio::test]
    async fn a_pull_naming_no_language_receives_english_only_and_stops_its_cursor_there() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()), never_erased());
        module
            .push_cards("u1", vec![card("c1", "s", 100, "mac")], 1_000)
            .await
            .unwrap();
        module
            .push_cards("u1", vec![spanish("hijo", 100, "mac")], 1_000)
            .await
            .unwrap();
        // The installed client: no language named.
        let (cards, cursor) = module.pull_cards("u1", 0, &[]).await.unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].language, "en");
        assert_eq!(cursor, 1, "the withheld Spanish card is not consumed");
        // Nothing new for it afterwards, and the cursor does not move.
        let (again, cursor2) = module.pull_cards("u1", cursor, &[]).await.unwrap();
        assert!(again.is_empty());
        assert_eq!(cursor2, cursor);
    }

    #[tokio::test]
    async fn widening_the_accepted_set_delivers_what_was_withheld() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()), never_erased());
        module
            .push_cards(
                "u1",
                vec![card("c1", "s", 100, "mac"), spanish("hijo", 100, "mac")],
                1_000,
            )
            .await
            .unwrap();
        let (_, cursor) = module.pull_cards("u1", 0, &[]).await.unwrap();
        let both = ["en".to_string(), "es-ES".to_string()];
        let (cards, next) = module.pull_cards("u1", cursor, &both).await.unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].language, "es");
        assert_eq!(next, 2);
        // Re-pulling from the start with both is a no-op for what it already holds.
        let (all, _) = module.pull_cards("u1", 0, &both).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn a_language_never_seen_reaches_only_the_clients_that_accept_it() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()), never_erased());
        let mut pt = spanish("filho", 100, "mac");
        pt.language = "pt".into();
        module.push_cards("u1", vec![pt], 1_000).await.unwrap();
        assert!(module.pull_cards("u1", 0, &[]).await.unwrap().0.is_empty());
        let es = ["es".to_string()];
        assert!(module.pull_cards("u1", 0, &es).await.unwrap().0.is_empty());
        let pt = ["pt".to_string()];
        assert_eq!(module.pull_cards("u1", 0, &pt).await.unwrap().0.len(), 1);
    }
}
