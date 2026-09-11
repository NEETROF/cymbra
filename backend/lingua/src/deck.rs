// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The Deck domain module: whole cards synced by stable client id, last-write-wins per
//! card (tombstones included). Same clamp + LWW rules as KnownWords
//! ([`known_words_core`]). Media contents never cross the wire (allow-list) — there is
//! no media field.

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_platform::Result;

use crate::known_words_core::clamp_ts;

/// A whole card as one device holds it (a tombstone when `deleted`). On a pull,
/// `sequence` is the server's change sequence; on a push it is ignored.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Card {
    pub client_id: String,
    pub lemma: String,
    pub surface_form: String,
    pub source_sentence: String,
    pub source: String,
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
    /// Cards with `sequence > cursor`, in sequence order (tombstones included).
    async fn changes_since(&self, user: &str, cursor: i64) -> Result<Vec<Card>>;
}

/// Orchestrates card sync over a [`DeckRepo`].
pub struct DeckModule {
    repo: Arc<dyn DeckRepo>,
}

impl DeckModule {
    pub fn new(repo: Arc<dyn DeckRepo>) -> Self {
        Self { repo }
    }

    pub async fn push_cards(&self, user: &str, cards: Vec<Card>, now: i64) -> Result<(u64, i64)> {
        let mut applied = 0u64;
        for mut card in cards {
            card.updated_at = clamp_ts(card.updated_at, now);
            if self.repo.apply_card(user, &card).await? {
                applied += 1;
            }
        }
        Ok((applied, self.repo.tip_cursor(user).await?))
    }

    pub async fn pull_cards(&self, user: &str, cursor: i64) -> Result<(Vec<Card>, i64)> {
        let cards = self.repo.changes_since(user, cursor).await?;
        let next = cards.iter().map(|c| c.sequence).max().unwrap_or(cursor);
        Ok((cards, next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::known_words_core::wins;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeDeckRepo {
        rows: Mutex<HashMap<String, HashMap<String, Card>>>, // user -> client_id -> card
        seq: Mutex<i64>,
    }

    #[async_trait]
    impl DeckRepo for FakeDeckRepo {
        async fn apply_card(&self, user: &str, card: &Card) -> Result<bool> {
            let mut rows = self.rows.lock().unwrap();
            let per_user = rows.entry(user.to_owned()).or_default();
            if let Some(existing) = per_user.get(&card.client_id)
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
            per_user.insert(card.client_id.clone(), stored);
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

        async fn changes_since(&self, user: &str, cursor: i64) -> Result<Vec<Card>> {
            let rows = self.rows.lock().unwrap();
            let mut out: Vec<Card> = rows
                .get(user)
                .into_iter()
                .flat_map(|m| m.values())
                .filter(|c| c.sequence > cursor)
                .cloned()
                .collect();
            out.sort_by_key(|c| c.sequence);
            Ok(out)
        }
    }

    fn card(id: &str, sentence: &str, ts: i64, device: &str) -> Card {
        Card {
            client_id: id.into(),
            lemma: "seldom".into(),
            surface_form: "seldom".into(),
            source_sentence: sentence.into(),
            source: "https://example.com/read".into(),
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
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()));
        module
            .push_cards(
                "u1",
                vec![card("c1", "They seldom ship.", 100, "iphone")],
                1_000,
            )
            .await
            .unwrap();
        let (cards, _) = module.pull_cards("u1", 0).await.unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].source_sentence, "They seldom ship.");
        assert!(!cards[0].deleted);
    }

    #[tokio::test]
    async fn deletion_propagates_by_lww() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()));
        module
            .push_cards("u1", vec![card("c1", "s", 100, "iphone")], 1_000)
            .await
            .unwrap();
        let mut tomb = card("c1", "s", 200, "mac");
        tomb.deleted = true;
        module.push_cards("u1", vec![tomb], 1_000).await.unwrap();
        let (cards, _) = module.pull_cards("u1", 0).await.unwrap();
        assert_eq!(cards.len(), 1);
        assert!(cards[0].deleted); // tombstone wins and is pulled so clients delete
    }

    #[tokio::test]
    async fn stale_edit_loses_to_the_newer_card() {
        let module = DeckModule::new(Arc::new(FakeDeckRepo::default()));
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
            module.repo.changes_since("u1", 0).await.unwrap()[0].source_sentence,
            "new"
        );
    }
}
