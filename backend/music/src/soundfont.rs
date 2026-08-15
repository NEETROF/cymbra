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
    /// Reward-shop cost in curation points (change: add-curation-rewards). `0` means
    /// **free** — served to any authenticated caller; `> 0` gates the raw bytes behind
    /// entitlement (change: add-soundfont-entitlement-previews).
    pub point_cost: i64,
    /// Whether the font is currently offered for redemption in the shop. A catalog
    /// *display* flag only — the entitlement gate keys on `point_cost`, not this, so a
    /// non-redeemable costed font is still gated.
    pub redeemable: bool,
    /// Moderator's rejection motive (change: add-soundfont-uploader-attribution);
    /// `None` while pending / when accepted. Surfaced to the uploader via the
    /// private-library proposal status.
    pub review_reason: Option<String>,
    /// Uploader's justification when re-proposing a previously `rejected` font
    /// (which reopens that row); `None` until a resubmission. Surfaced to the
    /// moderator on the privileged read.
    pub resubmission_note: Option<String>,
}

impl FontEntry {
    /// Whether this font is publicly visible (validated).
    pub fn is_accepted(&self) -> bool {
        self.moderation_status == "accepted"
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
    /// Whether `user_id` holds a redemption grant for the font `soundfont_id` — a
    /// `music.curation_grants` row (change: add-soundfont-entitlement-previews). Backs
    /// the entitlement gate on the raw `.sf2` bytes.
    async fn has_grant(&self, user_id: &str, soundfont_id: &str) -> Result<bool>;
    /// First non-`rejected` catalog font whose content digest matches `sha256`, used to
    /// refuse a byte-identical duplicate before storing (change: add-soundfont-moderation).
    async fn find_by_content(&self, sha256: &str) -> Result<Option<FontEntry>>;
    /// First `rejected` catalog font whose content digest matches `sha256` — the
    /// re-proposal target (change: add-soundfont-uploader-attribution): matching bytes
    /// reopen that row instead of piling up a duplicate.
    async fn find_rejected_by_content(&self, sha256: &str) -> Result<Option<FontEntry>>;
    /// Set a font's moderation status and stamp `reviewed_by` + `reviewed_at`. Returns
    /// whether a row matched. `reviewer_id` is the moderator's users.id (UUID string).
    /// `reason` is the rejection motive: stored as `review_reason` when provided (the
    /// caller passes it only on `rejected`); any transition overwrites the stored reason
    /// (so accepting / re-queuing clears a stale one).
    async fn set_moderation_status(
        &self,
        id: &str,
        status: &str,
        reviewer_id: &str,
        reason: Option<&str>,
    ) -> Result<bool>;
    /// Reopen a `rejected` font for re-review (change:
    /// add-soundfont-uploader-attribution): status → `pending`, re-attributed to
    /// `uploader_id`, prior `review_reason` and reviewer stamp cleared,
    /// `resubmission_note = note`. Returns whether a (rejected) row matched.
    async fn reopen_rejected(&self, id: &str, uploader_id: &str, note: &str) -> Result<bool>;
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
    /// Set a font's reward pricing (change: add-soundfont-reward-pricing):
    /// `point_cost` in curation points (`0` = free) and `redeemable` (whether the shop
    /// currently offers it). Returns whether a row matched. Kept apart from
    /// [`update_meta`](SoundFontRepo::update_meta) so metadata and pricing stay
    /// independently authorized — pricing is admin-only, metadata is moderator-or-admin.
    async fn set_pricing(&self, id: &str, point_cost: i64, redeemable: bool) -> Result<bool>;
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
        review_reason: row.get::<Option<String>, _>("review_reason"),
        resubmission_note: row.get::<Option<String>, _>("resubmission_note"),
    }
}

const SELECT_COLS: &str = "id, label, object_key, instrument, license, attribution, \
     size_bytes, moderation_status, reviewed_by, reviewed_at, uploaded_by, content_sha256, \
     point_cost, redeemable, review_reason, resubmission_note";

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

    async fn has_grant(&self, user_id: &str, soundfont_id: &str) -> Result<bool> {
        // A non-UUID user id can never hold a grant (grants key on users.id UUIDs);
        // treat it as no grant rather than failing the query.
        let Ok(user) = uuid::Uuid::parse_str(user_id) else {
            return Ok(false);
        };
        let found: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM music.curation_grants WHERE user_id = $1 AND key = $2",
        )
        .bind(user)
        .bind(soundfont_id)
        .fetch_optional(&self.pool)
        .await
        .context("soundfont grant lookup")?;
        Ok(found.is_some())
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

    async fn find_rejected_by_content(&self, sha256: &str) -> Result<Option<FontEntry>> {
        let row = sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM music.soundfonts \
             WHERE content_sha256 = $1 AND moderation_status = 'rejected' LIMIT 1"
        ))
        .bind(sha256)
        .fetch_optional(&self.pool)
        .await
        .context("find rejected soundfont by content")?;
        Ok(row.as_ref().map(row_to_entry))
    }

    async fn set_moderation_status(
        &self,
        id: &str,
        status: &str,
        reviewer_id: &str,
        reason: Option<&str>,
    ) -> Result<bool> {
        // A reviewer id that isn't a UUID is a caller/programming error; treat it as a
        // no-op not-found rather than failing the query (mirrors the score path).
        let Ok(reviewer) = uuid::Uuid::parse_str(reviewer_id) else {
            return Ok(false);
        };
        let r = sqlx::query(
            "UPDATE music.soundfonts \
             SET moderation_status = $2, reviewed_by = $3, reviewed_at = now(), \
                 review_reason = $4 \
             WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .bind(reviewer)
        .bind(reason)
        .execute(&self.pool)
        .await
        .context("set soundfont moderation status")?;
        Ok(r.rows_affected() > 0)
    }

    async fn reopen_rejected(&self, id: &str, uploader_id: &str, note: &str) -> Result<bool> {
        // Only a `rejected` row reopens; the WHERE guard makes a concurrent decision a
        // no-op not-found rather than clobbering a pending/accepted row.
        let Ok(uploader) = uuid::Uuid::parse_str(uploader_id) else {
            return Ok(false);
        };
        let r = sqlx::query(
            "UPDATE music.soundfonts \
             SET moderation_status = 'pending', uploaded_by = $2, review_reason = NULL, \
                 resubmission_note = $3, reviewed_by = NULL, reviewed_at = NULL \
             WHERE id = $1 AND moderation_status = 'rejected'",
        )
        .bind(id)
        .bind(uploader)
        .bind(note)
        .execute(&self.pool)
        .await
        .context("reopen rejected soundfont")?;
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

    async fn set_pricing(&self, id: &str, point_cost: i64, redeemable: bool) -> Result<bool> {
        // `point_cost` is an INT column; the caller (the RPC) already validates the range,
        // so an out-of-range value here is a programming error — refuse it rather than
        // silently truncating a price.
        let cost = i32::try_from(point_cost)
            .with_context(|| format!("soundfont point_cost {point_cost} out of range"))?;
        let r = sqlx::query(
            "UPDATE music.soundfonts SET point_cost = $2, redeemable = $3 WHERE id = $1",
        )
        .bind(id)
        .bind(cost)
        .bind(redeemable)
        .execute(&self.pool)
        .await
        .context("set soundfont pricing")?;
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
    /// Seeded redemption grants as `(user_id, soundfont_id)` pairs, backing
    /// [`has_grant`](SoundFontRepo::has_grant).
    grants: Mutex<Vec<(String, String)>>,
}

impl FakeSoundFontRepo {
    /// A repo seeded with the given entries.
    pub fn with(entries: Vec<FontEntry>) -> Self {
        Self {
            entries: Mutex::new(entries),
            grants: Mutex::new(Vec::new()),
        }
    }

    /// Seed a redemption grant so `user_id` owns `soundfont_id` (test helper).
    pub fn grant(&self, user_id: &str, soundfont_id: &str) {
        self.grants
            .lock()
            .unwrap()
            .push((user_id.to_string(), soundfont_id.to_string()));
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

    async fn has_grant(&self, user_id: &str, soundfont_id: &str) -> Result<bool> {
        Ok(self
            .grants
            .lock()
            .unwrap()
            .iter()
            .any(|(u, s)| u == user_id && s == soundfont_id))
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

    async fn find_rejected_by_content(&self, sha256: &str) -> Result<Option<FontEntry>> {
        Ok(self
            .entries
            .lock()
            .unwrap()
            .iter()
            .find(|e| {
                e.moderation_status == "rejected" && e.content_sha256.as_deref() == Some(sha256)
            })
            .cloned())
    }

    async fn set_moderation_status(
        &self,
        id: &str,
        status: &str,
        reviewer_id: &str,
        reason: Option<&str>,
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
                x.review_reason = reason.map(str::to_string);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn reopen_rejected(&self, id: &str, uploader_id: &str, note: &str) -> Result<bool> {
        // Lenient on the uploader id (like `insert`), so handler tests can use plain
        // string identities; Pg binds a UUID column and is strict.
        let mut e = self.entries.lock().unwrap();
        match e
            .iter_mut()
            .find(|x| x.id == id && x.moderation_status == "rejected")
        {
            Some(x) => {
                x.moderation_status = "pending".to_string();
                x.uploaded_by = Some(uploader_id.to_string());
                x.review_reason = None;
                x.resubmission_note = Some(note.to_string());
                x.reviewed_by = None;
                x.reviewed_at = None;
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

    async fn set_pricing(&self, id: &str, point_cost: i64, redeemable: bool) -> Result<bool> {
        let mut e = self.entries.lock().unwrap();
        match e.iter_mut().find(|x| x.id == id) {
            Some(x) => {
                x.point_cost = point_cost;
                x.redeemable = redeemable;
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
            review_reason: None,
            resubmission_note: None,
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
            review_reason: None,
            resubmission_note: None,
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
    async fn fake_repo_has_grant_reflects_seeded_grants() {
        let repo = FakeSoundFontRepo::with(vec![upright()]);
        // No grant seeded.
        assert!(!repo.has_grant("u", "upright-piano-kw").await.unwrap());
        repo.grant("u", "upright-piano-kw");
        assert!(repo.has_grant("u", "upright-piano-kw").await.unwrap());
        // Scoped to (user, font): another user / another font is not granted.
        assert!(!repo.has_grant("other", "upright-piano-kw").await.unwrap());
        assert!(!repo.has_grant("u", "another-font").await.unwrap());
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

    /// Pricing (change: add-soundfont-reward-pricing) writes only the two economy
    /// fields: a costed font keeps its label/licence/attribution/moderation state, and an
    /// unknown id matches nothing.
    #[tokio::test]
    async fn fake_repo_set_pricing_writes_only_the_economy_fields() {
        let repo = FakeSoundFontRepo::with(vec![upright()]);
        assert!(
            repo.set_pricing("upright-piano-kw", 250, false)
                .await
                .unwrap()
        );
        let e = repo.lookup("upright-piano-kw").await.unwrap().unwrap();
        assert_eq!(e.point_cost, 250);
        assert!(!e.redeemable);
        // Everything else is exactly the seeded font.
        assert_eq!(
            FontEntry {
                point_cost: 0,
                redeemable: true,
                ..e.clone()
            },
            upright()
        );

        // Back to free, and redeemable again.
        assert!(repo.set_pricing("upright-piano-kw", 0, true).await.unwrap());
        assert_eq!(
            repo.lookup("upright-piano-kw").await.unwrap(),
            Some(upright())
        );

        // An unknown id matches nothing.
        assert!(!repo.set_pricing("nope", 10, true).await.unwrap());
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
        // A rejected font no longer blocks its content…
        repo.set_moderation_status("ydp-grand", "rejected", REVIEWER, None)
            .await
            .unwrap();
        assert!(repo.find_by_content("cafe").await.unwrap().is_none());
        // …but is found by the re-proposal lookup.
        assert_eq!(
            repo.find_rejected_by_content("cafe")
                .await
                .unwrap()
                .map(|e| e.id),
            Some("ydp-grand".to_string())
        );
        assert!(
            repo.find_rejected_by_content("beef")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn set_moderation_status_stamps_reviewer() {
        let repo = FakeSoundFontRepo::with(vec![pending("ydp-grand", "cafe")]);
        assert!(
            repo.set_moderation_status("ydp-grand", "accepted", REVIEWER, None)
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
                .set_moderation_status("nope", "accepted", REVIEWER, None)
                .await
                .unwrap()
        );
        // A non-UUID reviewer is a no-op not-found.
        assert!(
            !repo
                .set_moderation_status("ydp-grand", "pending", "not-a-uuid", None)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn reject_stores_reason_and_other_statuses_clear_it() {
        let repo = FakeSoundFontRepo::with(vec![pending("ydp-grand", "cafe")]);
        // Rejecting with a reason stores it.
        assert!(
            repo.set_moderation_status("ydp-grand", "rejected", REVIEWER, Some("bad licence"))
                .await
                .unwrap()
        );
        let e = repo.lookup("ydp-grand").await.unwrap().unwrap();
        assert_eq!(e.review_reason.as_deref(), Some("bad licence"));
        // Any other decision clears the stale reason.
        repo.set_moderation_status("ydp-grand", "accepted", REVIEWER, None)
            .await
            .unwrap();
        let e = repo.lookup("ydp-grand").await.unwrap().unwrap();
        assert!(e.review_reason.is_none());
    }

    #[tokio::test]
    async fn reopen_rejected_requeues_reattributes_and_stores_the_note() {
        let repo = FakeSoundFontRepo::with(vec![pending("ydp-grand", "cafe")]);
        repo.set_moderation_status("ydp-grand", "rejected", REVIEWER, Some("nope"))
            .await
            .unwrap();
        // A non-rejected row does not reopen.
        assert!(
            !repo
                .reopen_rejected("unknown", "u2", "please")
                .await
                .unwrap()
        );
        // The rejected row reopens: pending, re-attributed, reason cleared, note stored.
        assert!(
            repo.reopen_rejected("ydp-grand", "u2", "please")
                .await
                .unwrap()
        );
        let e = repo.lookup("ydp-grand").await.unwrap().unwrap();
        assert_eq!(e.moderation_status, "pending");
        assert_eq!(e.uploaded_by.as_deref(), Some("u2"));
        assert!(e.review_reason.is_none());
        assert_eq!(e.resubmission_note.as_deref(), Some("please"));
        assert!(e.reviewed_by.is_none());
        assert!(e.reviewed_at.is_none());
        // Reopening again is a no-op (no longer rejected).
        assert!(
            !repo
                .reopen_rejected("ydp-grand", "u3", "again")
                .await
                .unwrap()
        );
    }
}
