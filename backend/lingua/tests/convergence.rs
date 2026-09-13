// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! End-to-end convergence (task 3.6): two simulated devices push through the three
//! modules and converge on read — statuses, cards and stats — over in-memory repos that
//! apply the same LWW rule the Postgres adapters do.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cymbra_lingua::deck::{Card, DeckModule, DeckRepo};
use cymbra_lingua::known_words::{
    DeclaredLevelChange, DeclaredLevelOpInput, KnownWordsModule, KnownWordsRepo, StatusChange,
    StatusOpInput,
};
use cymbra_lingua::known_words_core::wins;
use cymbra_lingua::stats::{DailyStat, StatsModule, StatsRepo};
use cymbra_platform::Result;

// --- Fakes (server state) --------------------------------------------------------

#[derive(Default)]
struct StatusRow {
    status: String,
    updated_at: i64,
    device_id: String,
    sequence: i64,
}

#[derive(Default)]
struct LevelRow {
    level: String,
    updated_at: i64,
    device_id: String,
    sequence: i64,
}

#[derive(Default)]
struct FakeStatuses {
    rows: Mutex<HashMap<(String, String), StatusRow>>,
    levels: Mutex<HashMap<String, LevelRow>>, // language -> declared level
    seq: Mutex<i64>,
}

#[async_trait]
impl KnownWordsRepo for FakeStatuses {
    async fn apply_op(&self, _user: &str, op: &StatusOpInput) -> Result<bool> {
        let mut rows = self.rows.lock().unwrap();
        let key = (op.language.clone(), op.lemma.clone());
        if let Some(e) = rows.get(&key)
            && !wins(e.updated_at, &e.device_id, op.client_ts, &op.device_id)
        {
            return Ok(false);
        }
        let mut seq = self.seq.lock().unwrap();
        *seq += 1;
        rows.insert(
            key,
            StatusRow {
                status: op.status.clone(),
                updated_at: op.client_ts,
                device_id: op.device_id.clone(),
                sequence: *seq,
            },
        );
        Ok(true)
    }
    async fn tip_cursor(&self, _user: &str) -> Result<i64> {
        let status_max = self
            .rows
            .lock()
            .unwrap()
            .values()
            .map(|r| r.sequence)
            .max()
            .unwrap_or(0);
        let level_max = self
            .levels
            .lock()
            .unwrap()
            .values()
            .map(|r| r.sequence)
            .max()
            .unwrap_or(0);
        Ok(status_max.max(level_max))
    }
    async fn changes_since(&self, _user: &str, cursor: i64) -> Result<Vec<StatusChange>> {
        let rows = self.rows.lock().unwrap();
        let mut out: Vec<StatusChange> = rows
            .iter()
            .filter(|(_, r)| r.sequence > cursor)
            .map(|((lang, lemma), r)| StatusChange {
                language: lang.clone(),
                lemma: lemma.clone(),
                status: r.status.clone(),
                updated_at: r.updated_at,
                sequence: r.sequence,
            })
            .collect();
        out.sort_by_key(|c| c.sequence);
        Ok(out)
    }
    async fn snapshot(&self, user: &str) -> Result<Vec<StatusChange>> {
        self.changes_since(user, 0).await
    }
    async fn apply_level_op(&self, _user: &str, op: &DeclaredLevelOpInput) -> Result<bool> {
        let mut levels = self.levels.lock().unwrap();
        if let Some(e) = levels.get(&op.language)
            && !wins(e.updated_at, &e.device_id, op.client_ts, &op.device_id)
        {
            return Ok(false);
        }
        let mut seq = self.seq.lock().unwrap();
        *seq += 1;
        levels.insert(
            op.language.clone(),
            LevelRow {
                level: op.level.clone(),
                updated_at: op.client_ts,
                device_id: op.device_id.clone(),
                sequence: *seq,
            },
        );
        Ok(true)
    }
    async fn level_changes_since(
        &self,
        _user: &str,
        cursor: i64,
    ) -> Result<Vec<DeclaredLevelChange>> {
        let levels = self.levels.lock().unwrap();
        let mut out: Vec<DeclaredLevelChange> = levels
            .iter()
            .filter(|(_, r)| r.sequence > cursor)
            .map(|(lang, r)| DeclaredLevelChange {
                language: lang.clone(),
                level: r.level.clone(),
                updated_at: r.updated_at,
                sequence: r.sequence,
            })
            .collect();
        out.sort_by_key(|c| c.sequence);
        Ok(out)
    }
    async fn level_snapshot(&self, user: &str) -> Result<Vec<DeclaredLevelChange>> {
        self.level_changes_since(user, 0).await
    }
}

#[derive(Default)]
struct FakeDeck {
    rows: Mutex<HashMap<String, Card>>,
    seq: Mutex<i64>,
}

#[async_trait]
impl DeckRepo for FakeDeck {
    async fn apply_card(&self, _user: &str, card: &Card) -> Result<bool> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(e) = rows.get(&card.client_id)
            && !wins(e.updated_at, &e.device_id, card.updated_at, &card.device_id)
        {
            return Ok(false);
        }
        let mut seq = self.seq.lock().unwrap();
        *seq += 1;
        let mut stored = card.clone();
        stored.sequence = *seq;
        rows.insert(card.client_id.clone(), stored);
        Ok(true)
    }
    async fn tip_cursor(&self, _user: &str) -> Result<i64> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .values()
            .map(|c| c.sequence)
            .max()
            .unwrap_or(0))
    }
    async fn changes_since(&self, _user: &str, cursor: i64) -> Result<Vec<Card>> {
        let rows = self.rows.lock().unwrap();
        let mut out: Vec<Card> = rows
            .values()
            .filter(|c| c.sequence > cursor)
            .cloned()
            .collect();
        out.sort_by_key(|c| c.sequence);
        Ok(out)
    }
}

#[derive(Default)]
struct FakeStats {
    rows: Mutex<HashMap<(i32, String, String), DailyStat>>,
}

#[async_trait]
impl StatsRepo for FakeStats {
    async fn upsert(&self, _user: &str, s: &DailyStat) -> Result<()> {
        self.rows
            .lock()
            .unwrap()
            .insert((s.day, s.language.clone(), s.device_id.clone()), s.clone());
        Ok(())
    }
    async fn range(
        &self,
        _user: &str,
        from: i32,
        to: i32,
        language: Option<&str>,
    ) -> Result<Vec<DailyStat>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .values()
            .filter(|s| s.day >= from && s.day <= to && language.is_none_or(|l| s.language == l))
            .cloned()
            .collect())
    }
}

// --- The scenario ----------------------------------------------------------------

fn op(lemma: &str, status: &str, ts: i64, device: &str) -> StatusOpInput {
    StatusOpInput {
        language: "en".into(),
        lemma: lemma.into(),
        status: status.into(),
        provenance: "manual".into(),
        client_ts: ts,
        device_id: device.into(),
    }
}

#[tokio::test]
async fn two_devices_converge_across_statuses_cards_and_stats() {
    const USER: &str = "user-1";
    let words = KnownWordsModule::new(Arc::new(FakeStatuses::default()));
    let deck = DeckModule::new(Arc::new(FakeDeck::default()));
    let stats = StatsModule::new(Arc::new(FakeStats::default()));

    // Statuses: Mac marks `seldom` known @100; iPhone marks it learning @200 (later wins).
    words
        .push_ops(USER, vec![op("seldom", "known", 100, "mac")], 1_000)
        .await
        .unwrap();
    words
        .push_ops(USER, vec![op("seldom", "learning", 200, "iphone")], 1_000)
        .await
        .unwrap();
    let (mac_view, _) = words.pull_changes(USER, 0).await.unwrap();
    let (iphone_view, _) = words.pull_changes(USER, 0).await.unwrap();
    assert_eq!(mac_view, iphone_view); // both devices see the same resolved state
    assert_eq!(
        mac_view
            .iter()
            .find(|c| c.lemma == "seldom")
            .unwrap()
            .status,
        "learning"
    );

    // Declared level: Mac sets B1 @100; iPhone upgrades to B2 @200 (later wins). The
    // level rides the same service and cursor, so both devices converge on it too.
    words
        .push_level_ops(
            USER,
            vec![DeclaredLevelOpInput {
                language: "en".into(),
                level: "B1".into(),
                client_ts: 100,
                device_id: "mac".into(),
            }],
            1_000,
        )
        .await
        .unwrap();
    words
        .push_level_ops(
            USER,
            vec![DeclaredLevelOpInput {
                language: "en".into(),
                level: "B2".into(),
                client_ts: 200,
                device_id: "iphone".into(),
            }],
            1_000,
        )
        .await
        .unwrap();
    let levels = words.level_snapshot(USER).await.unwrap();
    assert_eq!(levels.len(), 1);
    assert_eq!(levels[0].level, "B2"); // the later decision won on both devices

    // Cards: iPhone creates a card; Mac pulls it, edits it later; iPhone pulls the edit.
    let mut card = Card {
        client_id: "c1".into(),
        source_sentence: "They seldom ship.".into(),
        updated_at: 100,
        device_id: "iphone".into(),
        ..Card::default()
    };
    card.lemma = "seldom".into();
    deck.push_cards(USER, vec![card], 1_000).await.unwrap();
    let (mac_cards, _) = deck.pull_cards(USER, 0).await.unwrap();
    assert_eq!(mac_cards[0].source_sentence, "They seldom ship.");
    let edited = Card {
        client_id: "c1".into(),
        source_sentence: "They rarely ship.".into(),
        updated_at: 300,
        device_id: "mac".into(),
        ..Card::default()
    };
    deck.push_cards(USER, vec![edited], 1_000).await.unwrap();
    let (iphone_cards, _) = deck.pull_cards(USER, 0).await.unwrap();
    assert_eq!(iphone_cards[0].source_sentence, "They rarely ship."); // the later edit won

    // Stats: both devices upsert the same day; the consolidated read sums them.
    stats
        .upsert_stats(
            USER,
            vec![
                DailyStat {
                    day: 20_000,
                    language: "en".into(),
                    device_id: "mac".into(),
                    exposures: 0,
                    words_learned: 1,
                    reviews_done: 20,
                },
                DailyStat {
                    day: 20_000,
                    language: "en".into(),
                    device_id: "iphone".into(),
                    exposures: 0,
                    words_learned: 2,
                    reviews_done: 10,
                },
            ],
        )
        .await
        .unwrap();
    let consolidated = stats
        .get_stats(USER, 20_000, 20_000, Some("en"))
        .await
        .unwrap();
    assert_eq!(consolidated.len(), 1);
    assert_eq!(consolidated[0].reviews_done, 30);
    assert_eq!(consolidated[0].words_learned, 3);
}
