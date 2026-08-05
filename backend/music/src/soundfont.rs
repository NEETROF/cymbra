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
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row, postgres::PgRow};

/// Exact-byte SHA-256 of `bytes` as a lowercase hex string — the content digest used
/// for identical-soundfont detection across uploads (change: add-soundfont-moderation).
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// A catalog entry: the client-facing id, the storage key inside the private
/// SoundFont bucket, the instrument family, and the licence/attribution (recorded
/// for CC-BY redistribution). Sourced from `music.soundfonts`.
///
/// Moderation fields (change: add-soundfont-moderation): `moderation_status` is one of
/// `pending`/`accepted`/`rejected`; `reviewed_by`/`reviewed_at` are set once a decision
/// is made; `uploaded_by` is who contributed the font; `content_sha256` is the exact-byte
/// digest used to detect identical content across uploads.
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
    pub moderation_status: String,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub uploaded_by: Option<String>,
    pub content_sha256: Option<String>,
    /// Reward-shop price in curation points (change: add-curation-rewards): `0` =
    /// free/default (available to everyone); `> 0` = a costed reward whose raw bytes
    /// are entitlement-gated (change: add-soundfont-entitlement-previews).
    pub point_cost: i64,
    /// Whether the costed font is offered in the shop now (`false` = "coming later").
    /// Display-only; the entitlement gate keys on `point_cost`, not this.
    pub redeemable: bool,
}

impl FontEntry {
    /// Whether this font is publicly visible (validated).
    pub fn is_accepted(&self) -> bool {
        self.moderation_status == "accepted"
    }

    /// Whether the raw bytes are free to any signed-in caller — a `point_cost` of 0.
    /// A costed font's bytes are entitlement-gated (owned / own-import / moderator).
    pub fn is_free(&self) -> bool {
        self.point_cost == 0
    }
}

/// Catalog-wide counts by moderation status (change: add-soundfont-moderation),
/// independent of any filter/page — backs the back-office KPI cards.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SoundFontStatusCounts {
    pub pending: i64,
    pub accepted: i64,
    pub rejected: i64,
    pub total: i64,
}

/// Read + admin-write access to the persisted SoundFont catalog. Behind a trait so
/// the delivery route, the listing/admin RPCs, and the upload route are testable
/// with an in-memory [`FakeSoundFontRepo`].
#[async_trait]
pub trait SoundFontRepo: Send + Sync {
    /// Every catalog font regardless of moderation status (the admin listing),
    /// ordered by label.
    async fn list(&self) -> Result<Vec<FontEntry>>;
    /// Only `accepted` fonts (the public listing / `ListSoundFonts`), ordered by label.
    async fn list_accepted(&self) -> Result<Vec<FontEntry>>;
    /// A page of the admin listing filtered by moderation status (`None` = all),
    /// ordered by label, together with the total count matching the filter (for
    /// pagination).
    async fn list_admin_page(
        &self,
        moderation_status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FontEntry>, i64)>;
    /// Catalog-wide counts by moderation status (all statuses, no filter/page).
    async fn status_counts(&self) -> Result<SoundFontStatusCounts>;
    /// Resolve a client-facing id to its entry (any status), or `None` if unknown.
    async fn lookup(&self, id: &str) -> Result<Option<FontEntry>>;
    /// Whether `user_id` owns the costed font `id` — a `reward` grant in
    /// `music.curation_grants` keyed by the font id (a redeemed reward). Backs the
    /// entitlement gate on the raw-bytes delivery (change: add-soundfont-entitlement-previews).
    async fn has_grant(&self, user_id: &str, id: &str) -> Result<bool>;
    /// First non-`rejected` catalog font whose content digest matches `sha256`, used to
    /// refuse a byte-identical duplicate before storing (change: add-soundfont-moderation).
    async fn find_by_content(&self, sha256: &str) -> Result<Option<FontEntry>>;
    /// Set a font's moderation status and stamp `reviewed_by` + `reviewed_at`. Returns
    /// whether a row matched. `reviewer_id` is the moderator's users.id (UUID string).
    async fn set_moderation_status(
        &self,
        id: &str,
        status: &str,
        reviewer_id: &str,
    ) -> Result<bool>;
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
        moderation_status: row.get::<String, _>("moderation_status"),
        reviewed_by: row
            .get::<Option<uuid::Uuid>, _>("reviewed_by")
            .map(|u| u.to_string()),
        reviewed_at: row.get::<Option<DateTime<Utc>>, _>("reviewed_at"),
        uploaded_by: row
            .get::<Option<uuid::Uuid>, _>("uploaded_by")
            .map(|u| u.to_string()),
        content_sha256: row.get::<Option<String>, _>("content_sha256"),
        point_cost: row.get::<i32, _>("point_cost") as i64,
        redeemable: row.get::<bool, _>("redeemable"),
    }
}

const SELECT_COLS: &str = "id, label, object_key, instrument, license, attribution, \
     size_bytes, moderation_status, reviewed_by, reviewed_at, uploaded_by, content_sha256, \
     point_cost, redeemable";

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

    async fn list_accepted(&self) -> Result<Vec<FontEntry>> {
        let rows = sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM music.soundfonts \
             WHERE moderation_status = 'accepted' ORDER BY label"
        ))
        .fetch_all(&self.pool)
        .await
        .context("list accepted soundfonts")?;
        Ok(rows.iter().map(row_to_entry).collect())
    }

    async fn list_admin_page(
        &self,
        moderation_status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FontEntry>, i64)> {
        let (total, rows) = match moderation_status {
            Some(s) => {
                let total: i64 = sqlx::query_scalar(
                    "SELECT count(*) FROM music.soundfonts WHERE moderation_status = $1",
                )
                .bind(s)
                .fetch_one(&self.pool)
                .await
                .context("count soundfonts by status")?;
                let rows = sqlx::query(&format!(
                    "SELECT {SELECT_COLS} FROM music.soundfonts \
                     WHERE moderation_status = $1 ORDER BY label LIMIT $2 OFFSET $3"
                ))
                .bind(s)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .context("list soundfonts page by status")?;
                (total, rows)
            }
            None => {
                let total: i64 = sqlx::query_scalar("SELECT count(*) FROM music.soundfonts")
                    .fetch_one(&self.pool)
                    .await
                    .context("count soundfonts")?;
                let rows = sqlx::query(&format!(
                    "SELECT {SELECT_COLS} FROM music.soundfonts \
                     ORDER BY label LIMIT $1 OFFSET $2"
                ))
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .context("list soundfonts page")?;
                (total, rows)
            }
        };
        Ok((rows.iter().map(row_to_entry).collect(), total))
    }

    async fn status_counts(&self) -> Result<SoundFontStatusCounts> {
        let row = sqlx::query(
            "SELECT \
               count(*) FILTER (WHERE moderation_status = 'pending')  AS pending, \
               count(*) FILTER (WHERE moderation_status = 'accepted') AS accepted, \
               count(*) FILTER (WHERE moderation_status = 'rejected') AS rejected, \
               count(*) AS total \
             FROM music.soundfonts",
        )
        .fetch_one(&self.pool)
        .await
        .context("soundfont status counts")?;
        Ok(SoundFontStatusCounts {
            pending: row.get::<i64, _>("pending"),
            accepted: row.get::<i64, _>("accepted"),
            rejected: row.get::<i64, _>("rejected"),
            total: row.get::<i64, _>("total"),
        })
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

    async fn has_grant(&self, user_id: &str, id: &str) -> Result<bool> {
        // A non-UUID user id can never have a grant (treat as not-owned rather than
        // erroring — mirrors the reviewer-id handling elsewhere).
        let Ok(user) = uuid::Uuid::parse_str(user_id) else {
            return Ok(false);
        };
        let owned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM music.curation_grants \
             WHERE user_id = $1 AND key = $2 AND grant_kind = 'reward')",
        )
        .bind(user)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("soundfont has_grant")?;
        Ok(owned)
    }

    async fn find_by_content(&self, sha256: &str) -> Result<Option<FontEntry>> {
        // Dedup against non-`rejected` rows only, so a rejected id never permanently
        // blocks a later corrected/relicensed submission of the same bytes.
        let row = sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM music.soundfonts \
             WHERE content_sha256 = $1 AND moderation_status <> 'rejected' LIMIT 1"
        ))
        .bind(sha256)
        .fetch_optional(&self.pool)
        .await
        .context("find soundfont by content")?;
        Ok(row.as_ref().map(row_to_entry))
    }

    async fn set_moderation_status(
        &self,
        id: &str,
        status: &str,
        reviewer_id: &str,
    ) -> Result<bool> {
        // A reviewer id that isn't a UUID is a caller/programming error; treat it as a
        // no-op not-found rather than failing the query (mirrors the score path).
        let Ok(reviewer) = uuid::Uuid::parse_str(reviewer_id) else {
            return Ok(false);
        };
        let r = sqlx::query(
            "UPDATE music.soundfonts \
             SET moderation_status = $2, reviewed_by = $3, reviewed_at = now() \
             WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .bind(reviewer)
        .execute(&self.pool)
        .await
        .context("set soundfont moderation status")?;
        Ok(r.rows_affected() > 0)
    }

    async fn insert(&self, entry: &FontEntry) -> Result<()> {
        // Plain INSERT (no ON CONFLICT) so a duplicate id errors — the id's primary
        // key is the backstop for the caller's pre-check. `uploaded_by` is bound as a
        // UUID (NULL when absent); `reviewed_by`/`reviewed_at` stay NULL until a decision.
        let uploaded_by = entry
            .uploaded_by
            .as_deref()
            .and_then(|s| uuid::Uuid::parse_str(s).ok());
        sqlx::query(
            "INSERT INTO music.soundfonts \
             (id, label, object_key, instrument, license, attribution, size_bytes, \
              moderation_status, uploaded_by, content_sha256) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(&entry.id)
        .bind(&entry.label)
        .bind(&entry.object_key)
        .bind(&entry.instrument)
        .bind(&entry.license)
        .bind(&entry.attribution)
        .bind(entry.size_bytes)
        .bind(&entry.moderation_status)
        .bind(uploaded_by)
        .bind(&entry.content_sha256)
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
    /// Redeemed reward grants as `(user_id, font_id)` pairs, backing [`Self::has_grant`].
    grants: Mutex<std::collections::HashSet<(String, String)>>,
}

impl FakeSoundFontRepo {
    /// A repo seeded with the given entries.
    pub fn with(entries: Vec<FontEntry>) -> Self {
        Self {
            entries: Mutex::new(entries),
            grants: Mutex::default(),
        }
    }

    /// Seed a redeemed grant so [`Self::has_grant`] returns true for `(user_id, id)`.
    pub fn grant(&self, user_id: &str, id: &str) {
        self.grants
            .lock()
            .unwrap()
            .insert((user_id.to_string(), id.to_string()));
    }
}

#[async_trait]
impl SoundFontRepo for FakeSoundFontRepo {
    async fn list(&self) -> Result<Vec<FontEntry>> {
        Ok(self.entries.lock().unwrap().clone())
    }

    async fn list_accepted(&self) -> Result<Vec<FontEntry>> {
        Ok(self
            .entries
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.is_accepted())
            .cloned()
            .collect())
    }

    async fn list_admin_page(
        &self,
        moderation_status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FontEntry>, i64)> {
        let mut all: Vec<FontEntry> = self
            .entries
            .lock()
            .unwrap()
            .iter()
            .filter(|e| moderation_status.is_none_or(|s| e.moderation_status == s))
            .cloned()
            .collect();
        all.sort_by(|a, b| a.label.cmp(&b.label));
        let total = all.len() as i64;
        let page = all
            .into_iter()
            .skip(offset.max(0) as usize)
            .take(limit.max(0) as usize)
            .collect();
        Ok((page, total))
    }

    async fn status_counts(&self) -> Result<SoundFontStatusCounts> {
        let e = self.entries.lock().unwrap();
        let count = |s: &str| e.iter().filter(|x| x.moderation_status == s).count() as i64;
        Ok(SoundFontStatusCounts {
            pending: count("pending"),
            accepted: count("accepted"),
            rejected: count("rejected"),
            total: e.len() as i64,
        })
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

    async fn has_grant(&self, user_id: &str, id: &str) -> Result<bool> {
        Ok(self
            .grants
            .lock()
            .unwrap()
            .contains(&(user_id.to_string(), id.to_string())))
    }

    async fn find_by_content(&self, sha256: &str) -> Result<Option<FontEntry>> {
        Ok(self
            .entries
            .lock()
            .unwrap()
            .iter()
            .find(|e| {
                e.moderation_status != "rejected" && e.content_sha256.as_deref() == Some(sha256)
            })
            .cloned())
    }

    async fn set_moderation_status(
        &self,
        id: &str,
        status: &str,
        reviewer_id: &str,
    ) -> Result<bool> {
        if uuid::Uuid::parse_str(reviewer_id).is_err() {
            return Ok(false);
        }
        let mut e = self.entries.lock().unwrap();
        match e.iter_mut().find(|x| x.id == id) {
            Some(x) => {
                x.moderation_status = status.to_string();
                x.reviewed_by = Some(reviewer_id.to_string());
                x.reviewed_at = Some(Utc::now());
                Ok(true)
            }
            None => Ok(false),
        }
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

    const REVIEWER: &str = "11111111-1111-1111-1111-111111111111";

    fn upright() -> FontEntry {
        FontEntry {
            id: "upright-piano-kw".into(),
            label: "Upright Piano KW".into(),
            object_key: "UprightPianoKW-20220221.sf2".into(),
            instrument: "piano".into(),
            license: "CC0-1.0".into(),
            attribution: None,
            size_bytes: None,
            moderation_status: "accepted".into(),
            reviewed_by: None,
            reviewed_at: None,
            uploaded_by: None,
            content_sha256: Some("deadbeef".into()),
            point_cost: 0,
            redeemable: true,
        }
    }

    /// A `pending` font with a distinct content digest.
    fn pending(id: &str, sha: &str) -> FontEntry {
        FontEntry {
            id: id.into(),
            label: id.into(),
            object_key: format!("{id}.sf2"),
            instrument: "piano".into(),
            license: "CC-BY 3.0".into(),
            attribution: Some("Someone".into()),
            size_bytes: Some(42),
            moderation_status: "pending".into(),
            reviewed_by: None,
            reviewed_at: None,
            uploaded_by: Some(REVIEWER.into()),
            content_sha256: Some(sha.into()),
            point_cost: 0,
            redeemable: true,
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
        let ydp = pending("ydp-grand", "cafe");
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

    #[tokio::test]
    async fn list_accepted_hides_unvalidated() {
        let repo = FakeSoundFontRepo::with(vec![upright(), pending("ydp-grand", "cafe")]);
        // The admin listing sees both; the public listing only the accepted one.
        assert_eq!(repo.list().await.unwrap().len(), 2);
        let accepted = repo.list_accepted().await.unwrap();
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].id, "upright-piano-kw");
    }

    #[tokio::test]
    async fn find_by_content_matches_non_rejected_only() {
        let repo = FakeSoundFontRepo::with(vec![pending("ydp-grand", "cafe")]);
        // A non-rejected byte-identical match is found (dedup guard).
        assert_eq!(
            repo.find_by_content("cafe").await.unwrap().map(|e| e.id),
            Some("ydp-grand".to_string())
        );
        // Unknown content → no match.
        assert!(repo.find_by_content("beef").await.unwrap().is_none());
        // A rejected font no longer blocks its content.
        repo.set_moderation_status("ydp-grand", "rejected", REVIEWER)
            .await
            .unwrap();
        assert!(repo.find_by_content("cafe").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn fake_repo_tracks_reward_grants() {
        let repo = FakeSoundFontRepo::with(vec![upright()]);
        // No grant → not owned.
        assert!(!repo.has_grant("user-a", "reward-grand").await.unwrap());
        // After seeding a grant, the (user, font) pair is owned; others are not.
        repo.grant("user-a", "reward-grand");
        assert!(repo.has_grant("user-a", "reward-grand").await.unwrap());
        assert!(!repo.has_grant("user-b", "reward-grand").await.unwrap());
        assert!(!repo.has_grant("user-a", "other").await.unwrap());
    }

    #[tokio::test]
    async fn set_moderation_status_stamps_reviewer() {
        let repo = FakeSoundFontRepo::with(vec![pending("ydp-grand", "cafe")]);
        assert!(
            repo.set_moderation_status("ydp-grand", "accepted", REVIEWER)
                .await
                .unwrap()
        );
        let e = repo.lookup("ydp-grand").await.unwrap().unwrap();
        assert_eq!(e.moderation_status, "accepted");
        assert_eq!(e.reviewed_by.as_deref(), Some(REVIEWER));
        assert!(e.reviewed_at.is_some());
        // Now publicly visible.
        assert_eq!(repo.list_accepted().await.unwrap().len(), 1);
        // Unknown id → no row matched.
        assert!(
            !repo
                .set_moderation_status("nope", "accepted", REVIEWER)
                .await
                .unwrap()
        );
        // A non-UUID reviewer is a no-op not-found.
        assert!(
            !repo
                .set_moderation_status("ydp-grand", "pending", "not-a-uuid")
                .await
                .unwrap()
        );
    }
}
