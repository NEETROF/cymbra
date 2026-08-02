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

//! The persisted SoundFont catalog (change: add-soundfont-catalog-db).
//!
//! One source of truth in `music.soundfonts` that both the ListSoundFonts RPC
//! (this crate's [`crate::grpc`]) and the HTTP delivery route (in the server
//! binary) resolve through, so adding a font is a data change (a row + an object)
//! rather than a code change. The catalog **types** live here (moved out of the
//! server binary) so both surfaces share one definition.
//!
//! Uses the runtime `sqlx::query(...).bind(...)` API with fully-qualified table
//! names, matching the other repos in this crate.

use std::sync::Mutex;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use sqlx::{PgPool, Row, postgres::PgRow};

/// A catalog entry: the client-facing id, the storage key inside the private
/// SoundFont bucket, the instrument family, and the licence/attribution (recorded
/// for CC-BY redistribution). Sourced from `music.soundfonts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontEntry {
    pub id: String,
    pub label: String,
    pub object_key: String,
    /// Instrument family the font is for (e.g. "piano"), correlating it to matching
    /// instrument scores.
    pub instrument: String,
    pub license: String,
    pub attribution: Option<String>,
    pub size_bytes: Option<i64>,
}

/// Read + admin-write access to the persisted SoundFont catalog. Behind a trait so
/// the delivery route, the listing/admin RPCs, and the upload route are testable
/// with an in-memory [`FakeSoundFontRepo`].
#[async_trait]
pub trait SoundFontRepo: Send + Sync {
    /// Every catalog font (for the listing/admin endpoints), ordered by label.
    async fn list(&self) -> Result<Vec<FontEntry>>;
    /// Resolve a client-facing id to its entry, or `None` if unknown.
    async fn lookup(&self, id: &str) -> Result<Option<FontEntry>>;
    /// Insert a new font row (change: add-soundfont-back-office-management). Errors if
    /// the id already exists — callers refuse a duplicate before storing the object.
    async fn insert(&self, entry: &FontEntry) -> Result<()>;
    /// Update a font's editable metadata (label, licence, attribution). The id,
    /// `object_key`, and `instrument` are immutable. Returns whether a row matched.
    async fn update_meta(
        &self,
        id: &str,
        label: &str,
        license: &str,
        attribution: Option<&str>,
    ) -> Result<bool>;
    /// Delete a font's row. Returns whether a row was removed. (The stored object is
    /// removed separately by the caller.)
    async fn delete(&self, id: &str) -> Result<bool>;
}

/// Maps a `music.soundfonts` row to a [`FontEntry`].
fn row_to_entry(row: &PgRow) -> FontEntry {
    FontEntry {
        id: row.get::<String, _>("id"),
        label: row.get::<String, _>("label"),
        object_key: row.get::<String, _>("object_key"),
        instrument: row.get::<String, _>("instrument"),
        license: row.get::<String, _>("license"),
        attribution: row.get::<Option<String>, _>("attribution"),
        size_bytes: row.get::<Option<i64>, _>("size_bytes"),
    }
}

const SELECT_COLS: &str = "id, label, object_key, instrument, license, attribution, size_bytes";

/// Postgres-backed [`SoundFontRepo`] over the `music.soundfonts` table.
pub struct PgSoundFontRepo {
    pool: PgPool,
}

impl PgSoundFontRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SoundFontRepo for PgSoundFontRepo {
    async fn list(&self) -> Result<Vec<FontEntry>> {
        let rows = sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM music.soundfonts ORDER BY label"
        ))
        .fetch_all(&self.pool)
        .await
        .context("list soundfonts")?;
        Ok(rows.iter().map(row_to_entry).collect())
    }

    async fn lookup(&self, id: &str) -> Result<Option<FontEntry>> {
        let row = sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM music.soundfonts WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("lookup soundfont")?;
        Ok(row.as_ref().map(row_to_entry))
    }

    async fn insert(&self, entry: &FontEntry) -> Result<()> {
        // Plain INSERT (no ON CONFLICT) so a duplicate id errors — the id's primary
        // key is the backstop for the caller's pre-check.
        sqlx::query(
            "INSERT INTO music.soundfonts \
             (id, label, object_key, instrument, license, attribution, size_bytes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&entry.id)
        .bind(&entry.label)
        .bind(&entry.object_key)
        .bind(&entry.instrument)
        .bind(&entry.license)
        .bind(&entry.attribution)
        .bind(entry.size_bytes)
        .execute(&self.pool)
        .await
        .context("insert soundfont")?;
        Ok(())
    }

    async fn update_meta(
        &self,
        id: &str,
        label: &str,
        license: &str,
        attribution: Option<&str>,
    ) -> Result<bool> {
        let r = sqlx::query(
            "UPDATE music.soundfonts \
             SET label = $2, license = $3, attribution = $4 WHERE id = $1",
        )
        .bind(id)
        .bind(label)
        .bind(license)
        .bind(attribution)
        .execute(&self.pool)
        .await
        .context("update soundfont")?;
        Ok(r.rows_affected() > 0)
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let r = sqlx::query("DELETE FROM music.soundfonts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("delete soundfont")?;
        Ok(r.rows_affected() > 0)
    }
}

/// In-memory [`SoundFontRepo`] for tests (no database). Interior-mutable so the
/// write methods work through the shared `&self` trait.
#[derive(Default)]
pub struct FakeSoundFontRepo {
    entries: Mutex<Vec<FontEntry>>,
}

impl FakeSoundFontRepo {
    /// A repo seeded with the given entries.
    pub fn with(entries: Vec<FontEntry>) -> Self {
        Self {
            entries: Mutex::new(entries),
        }
    }
}

#[async_trait]
impl SoundFontRepo for FakeSoundFontRepo {
    async fn list(&self) -> Result<Vec<FontEntry>> {
        Ok(self.entries.lock().unwrap().clone())
    }

    async fn lookup(&self, id: &str) -> Result<Option<FontEntry>> {
        Ok(self
            .entries
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.id == id)
            .cloned())
    }

    async fn insert(&self, entry: &FontEntry) -> Result<()> {
        let mut e = self.entries.lock().unwrap();
        if e.iter().any(|x| x.id == entry.id) {
            bail!("soundfont {} already exists", entry.id);
        }
        e.push(entry.clone());
        Ok(())
    }

    async fn update_meta(
        &self,
        id: &str,
        label: &str,
        license: &str,
        attribution: Option<&str>,
    ) -> Result<bool> {
        let mut e = self.entries.lock().unwrap();
        match e.iter_mut().find(|x| x.id == id) {
            Some(x) => {
                x.label = label.to_string();
                x.license = license.to_string();
                x.attribution = attribution.map(str::to_string);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let mut e = self.entries.lock().unwrap();
        let before = e.len();
        e.retain(|x| x.id != id);
        Ok(e.len() != before)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upright() -> FontEntry {
        FontEntry {
            id: "upright-piano-kw".into(),
            label: "Upright Piano KW".into(),
            object_key: "UprightPianoKW-20220221.sf2".into(),
            instrument: "piano".into(),
            license: "CC0-1.0".into(),
            attribution: None,
            size_bytes: None,
        }
    }

    #[tokio::test]
    async fn fake_repo_lists_and_looks_up() {
        let repo = FakeSoundFontRepo::with(vec![upright()]);
        assert_eq!(repo.list().await.unwrap(), vec![upright()]);
        assert_eq!(
            repo.lookup("upright-piano-kw").await.unwrap(),
            Some(upright())
        );
        assert_eq!(repo.lookup("nope").await.unwrap(), None);
    }

    #[tokio::test]
    async fn fake_repo_insert_rejects_duplicate_id() {
        let repo = FakeSoundFontRepo::with(vec![upright()]);
        // A different font inserts.
        let ydp = FontEntry {
            id: "ydp-grand".into(),
            label: "YDP".into(),
            object_key: "ydp-grand.sf2".into(),
            instrument: "piano".into(),
            license: "CC-BY 3.0".into(),
            attribution: Some("Roberto".into()),
            size_bytes: Some(42),
        };
        repo.insert(&ydp).await.unwrap();
        assert_eq!(repo.list().await.unwrap().len(), 2);
        // Re-inserting an existing id errors.
        assert!(repo.insert(&upright()).await.is_err());
    }

    #[tokio::test]
    async fn fake_repo_update_meta_and_delete() {
        let repo = FakeSoundFontRepo::with(vec![upright()]);
        assert!(
            repo.update_meta("upright-piano-kw", "Renamed", "CC0-1.0", Some("Someone"),)
                .await
                .unwrap()
        );
        let e = repo.lookup("upright-piano-kw").await.unwrap().unwrap();
        assert_eq!(e.label, "Renamed");
        assert_eq!(e.attribution.as_deref(), Some("Someone"));
        // The object key + instrument are immutable across a metadata update.
        assert_eq!(e.object_key, "UprightPianoKW-20220221.sf2");
        assert_eq!(e.instrument, "piano");

        // Updating an unknown id matches nothing.
        assert!(!repo.update_meta("nope", "x", "x", None).await.unwrap());

        assert!(repo.delete("upright-piano-kw").await.unwrap());
        assert!(repo.list().await.unwrap().is_empty());
        assert!(!repo.delete("upright-piano-kw").await.unwrap());
    }
}
