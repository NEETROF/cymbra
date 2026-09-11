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

//! The plugin's local store (`~/.lingua/`, SQLite, versioned schema; design D2).
//!
//! It holds lemmas, exposure counters, statuses, calibration, deck cards and the
//! per-transcript ingest offsets — **never any transcript content**, only the
//! sentences the user explicitly captures onto a card. The schema mirrors
//! `lingua-core`'s types (cards are its `Card` serialised) so a future merge with the
//! other surfaces is mechanical. English-only for the MVP, so no language column.

use std::path::Path;

use lingua_core::decks::card::Card;
use lingua_core::knowledge::state::KnowledgeState;
use lingua_core::knowledge::status::{KnownSource, Status};
use rusqlite::{Connection, OptionalExtension, params};

/// Current schema version; bump with a migration when the shape changes.
pub const SCHEMA_VERSION: i64 = 1;

/// Default calibration ("I know the 3000 most common words").
pub const DEFAULT_CALIBRATION: u32 = 3000;

/// The local SQLite store.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Opens (creating if needed) the store at `path` and migrates the schema.
    pub fn open(path: &Path) -> rusqlite::Result<Store> {
        let store = Store {
            conn: Connection::open(path)?,
        };
        store.migrate()?;
        Ok(store)
    }

    /// An in-memory store (tests).
    pub fn open_in_memory() -> rusqlite::Result<Store> {
        let store = Store {
            conn: Connection::open_in_memory()?,
        };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (k TEXT PRIMARY KEY, v TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS exposures (
               lemma TEXT PRIMARY KEY, occurrences INTEGER NOT NULL,
               last_source TEXT NOT NULL, last_seen INTEGER NOT NULL);
             CREATE TABLE IF NOT EXISTS statuses (lemma TEXT PRIMARY KEY, status TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS cards (lemma TEXT PRIMARY KEY, json TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS ingest (transcript TEXT PRIMARY KEY, offset INTEGER NOT NULL);",
        )?;
        self.conn.execute(
            "INSERT OR IGNORE INTO meta (k, v) VALUES ('schema_version', ?1)",
            params![SCHEMA_VERSION.to_string()],
        )?;
        Ok(())
    }

    /// The stored schema version.
    pub fn schema_version(&self) -> rusqlite::Result<i64> {
        self.meta("schema_version")
            .map(|v| v.and_then(|s| s.parse().ok()).unwrap_or(SCHEMA_VERSION))
    }

    fn meta(&self, key: &str) -> rusqlite::Result<Option<String>> {
        self.conn
            .query_row("SELECT v FROM meta WHERE k = ?1", params![key], |r| {
                r.get::<_, String>(0)
            })
            .optional()
    }

    fn set_meta(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (k, v) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    // --- Exposure counters ---

    /// Increments a lemma's exposure by `count`, stamping the source and time.
    pub fn record_exposure(
        &self,
        lemma: &str,
        count: u32,
        source: &str,
        timestamp: i64,
    ) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO exposures (lemma, occurrences, last_source, last_seen) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(lemma) DO UPDATE SET occurrences = occurrences + excluded.occurrences,
               last_source = excluded.last_source, last_seen = excluded.last_seen",
            params![lemma, count, source, timestamp],
        )?;
        Ok(())
    }

    /// A lemma's total occurrences (0 if never seen).
    pub fn exposure(&self, lemma: &str) -> rusqlite::Result<u32> {
        self.conn
            .query_row(
                "SELECT occurrences FROM exposures WHERE lemma = ?1",
                params![lemma],
                |r| r.get(0),
            )
            .optional()
            .map(|o| o.unwrap_or(0))
    }

    /// How many distinct lemmas have ever been seen.
    pub fn tracked_lemmas(&self) -> rusqlite::Result<usize> {
        self.conn
            .query_row("SELECT COUNT(*) FROM exposures", [], |r| r.get::<_, i64>(0))
            .map(|n| n as usize)
    }

    // --- Statuses + calibration ---

    pub fn set_status(&self, lemma: &str, status: Status) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO statuses (lemma, status) VALUES (?1, ?2)",
            params![lemma, status_str(status)],
        )?;
        Ok(())
    }

    pub fn calibration(&self) -> rusqlite::Result<u32> {
        Ok(self
            .meta("calibration")?
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_CALIBRATION))
    }

    pub fn set_calibration(&self, threshold: u32) -> rusqlite::Result<()> {
        self.set_meta("calibration", &threshold.to_string())
    }

    /// Builds a `lingua-core` knowledge state (calibration + statuses) for analysis.
    pub fn knowledge_state(&self) -> rusqlite::Result<KnowledgeState> {
        let mut state = KnowledgeState::new();
        state.set_calibration(
            lingua_core::analysis::language::StudiedLanguage::English,
            self.calibration()?,
        );
        let mut stmt = self.conn.prepare("SELECT lemma, status FROM statuses")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        for row in rows {
            let (lemma, status) = row?;
            if let Some(s) = status_from_str(&status) {
                state.set_status(
                    lingua_core::analysis::language::StudiedLanguage::English,
                    &lemma,
                    s,
                );
            }
        }
        Ok(state)
    }

    // --- Deck cards (lingua-core Card serialised) ---

    pub fn upsert_card(&self, card: &Card) -> rusqlite::Result<()> {
        let json = serde_json::to_string(card).expect("Card serialises");
        self.conn.execute(
            "INSERT OR REPLACE INTO cards (lemma, json) VALUES (?1, ?2)",
            params![card.lemma, json],
        )?;
        Ok(())
    }

    pub fn card(&self, lemma: &str) -> rusqlite::Result<Option<Card>> {
        let json: Option<String> = self
            .conn
            .query_row(
                "SELECT json FROM cards WHERE lemma = ?1",
                params![lemma],
                |r| r.get(0),
            )
            .optional()?;
        Ok(json.and_then(|j| serde_json::from_str(&j).ok()))
    }

    pub fn all_cards(&self) -> rusqlite::Result<Vec<Card>> {
        let mut stmt = self.conn.prepare("SELECT json FROM cards ORDER BY lemma")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut cards = Vec::new();
        for row in rows {
            if let Ok(card) = serde_json::from_str(&row?) {
                cards.push(card);
            }
        }
        Ok(cards)
    }

    pub fn deck_len(&self) -> rusqlite::Result<usize> {
        self.conn
            .query_row("SELECT COUNT(*) FROM cards", [], |r| r.get::<_, i64>(0))
            .map(|n| n as usize)
    }

    /// Cards due at `now` (Unix-epoch seconds), in deterministic lemma order.
    pub fn due_cards(&self, now: i64) -> rusqlite::Result<Vec<Card>> {
        Ok(self
            .all_cards()?
            .into_iter()
            .filter(|c| c.review.is_due(now))
            .collect())
    }

    // --- Ingest offsets (idempotence) ---

    pub fn ingest_offset(&self, transcript: &str) -> rusqlite::Result<u64> {
        self.conn
            .query_row(
                "SELECT offset FROM ingest WHERE transcript = ?1",
                params![transcript],
                |r| r.get::<_, i64>(0),
            )
            .optional()
            .map(|o| o.unwrap_or(0) as u64)
    }

    pub fn set_ingest_offset(&self, transcript: &str, offset: u64) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO ingest (transcript, offset) VALUES (?1, ?2)",
            params![transcript, offset as i64],
        )?;
        Ok(())
    }
}

fn status_str(status: Status) -> &'static str {
    match status {
        Status::Known(_) => "known",
        Status::Learning => "learning",
        Status::Ignored => "ignored",
    }
}

fn status_from_str(s: &str) -> Option<Status> {
    match s {
        "known" => Some(Status::Known(KnownSource::Manual)),
        "learning" => Some(Status::Learning),
        "ignored" => Some(Status::Ignored),
        _ => None,
    }
}
