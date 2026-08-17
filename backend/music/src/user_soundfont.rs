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

//! The private, per-user SoundFont library (change: add-soundfont-moderation).
//!
//! Distinct from the public catalog ([`crate::SoundFontRepo`]): fonts here belong to a
//! single user, are **not** moderated, and are visible/deliverable only to their owner.
//! Server-backed so a user's library follows them across devices. Every method is
//! **owner-scoped** — there is no cross-owner read path at the data layer. Uses
//! `anyhow`-based results like the catalog repo, since the HTTP delivery/import routes
//! resolve through it.

use std::sync::Mutex;

use anyhow::{Context, Result};
use async_trait::async_trait;

/// Today's private-library cap for a plain user (the pre-plan constant, kept as the
/// `free` default; change: add-premium-subscription).
pub const DEFAULT_LIBRARY_MAX_FONTS: i64 = 5;

/// Per-plan private-library quota, resolved **per request** (runtime config):
/// `extended` is whether the caller's effective plan grants the
/// `soundfont_library.extended` unlock. Moderators/admins are exempt upstream.
#[cfg_attr(test, mockall::automock)]
pub trait LibraryQuotaSource: Send + Sync {
    fn max_fonts(&self, extended: bool) -> i64;
}

/// Fixed quotas (tests, or a deployment with no flag store).
#[derive(Debug, Clone, Copy)]
pub struct FixedLibraryQuota {
    pub free: i64,
    pub premium: i64,
}

impl Default for FixedLibraryQuota {
    fn default() -> Self {
        Self {
            free: DEFAULT_LIBRARY_MAX_FONTS,
            premium: 50,
        }
    }
}

impl LibraryQuotaSource for FixedLibraryQuota {
    fn max_fonts(&self, extended: bool) -> i64 {
        if extended { self.premium } else { self.free }
    }
}
use sqlx::{PgPool, Row, postgres::PgRow};

/// One private-library font row. `id` and `user_id` are UUID strings; `object_key`
/// is the per-user key in the private bucket; `content_sha256` is the exact-byte digest
/// backing the idempotent re-import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserFontEntry {
    pub id: String,
    pub user_id: String,
    pub label: String,
    pub object_key: String,
    pub content_sha256: String,
    pub size_bytes: i64,
}

/// Owner-scoped storage surface for the private SoundFont library.
#[async_trait]
pub trait UserSoundFontRepo: Send + Sync {
    /// The caller's private fonts, newest first.
    async fn list(&self, user_id: &str) -> Result<Vec<UserFontEntry>>;
    /// How many fonts the caller currently holds (for the per-user quota).
    async fn count(&self, user_id: &str) -> Result<i64>;
    /// The caller's font whose content digest matches (backs the idempotent re-import);
    /// `None` if the caller holds no such content.
    async fn find_by_content(
        &self,
        user_id: &str,
        content_sha256: &str,
    ) -> Result<Option<UserFontEntry>>;
    /// One font the caller owns by id; `None` if absent or owned by someone else.
    async fn lookup(&self, user_id: &str, id: &str) -> Result<Option<UserFontEntry>>;
    /// Insert a new private font row.
    async fn insert(&self, entry: &UserFontEntry) -> Result<()>;
    /// Delete a font the caller owns, returning the removed row (its `object_key`
    /// drives object cleanup); `None` if absent or not theirs.
    async fn delete(&self, user_id: &str, id: &str) -> Result<Option<UserFontEntry>>;
}

fn row_to_entry(row: &PgRow) -> UserFontEntry {
    UserFontEntry {
        id: row.get::<uuid::Uuid, _>("id").to_string(),
        user_id: row.get::<uuid::Uuid, _>("user_id").to_string(),
        label: row.get::<String, _>("label"),
        object_key: row.get::<String, _>("object_key"),
        content_sha256: row.get::<String, _>("content_sha256"),
        size_bytes: row.get::<i64, _>("size_bytes"),
    }
}

const COLS: &str = "id, user_id, label, object_key, content_sha256, size_bytes";

/// Postgres-backed [`UserSoundFontRepo`] over `music.user_soundfonts`.
pub struct PgUserSoundFontRepo {
    pool: PgPool,
}

impl PgUserSoundFontRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserSoundFontRepo for PgUserSoundFontRepo {
    async fn list(&self, user_id: &str) -> Result<Vec<UserFontEntry>> {
        let Ok(uid) = uuid::Uuid::parse_str(user_id) else {
            return Ok(vec![]); // malformed caller id owns nothing
        };
        let rows = sqlx::query(&format!(
            "SELECT {COLS} FROM music.user_soundfonts \
             WHERE user_id = $1 ORDER BY created_at DESC"
        ))
        .bind(uid)
        .fetch_all(&self.pool)
        .await
        .context("list user soundfonts")?;
        Ok(rows.iter().map(row_to_entry).collect())
    }

    async fn count(&self, user_id: &str) -> Result<i64> {
        let Ok(uid) = uuid::Uuid::parse_str(user_id) else {
            return Ok(0);
        };
        let row = sqlx::query("SELECT count(*) AS n FROM music.user_soundfonts WHERE user_id = $1")
            .bind(uid)
            .fetch_one(&self.pool)
            .await
            .context("count user soundfonts")?;
        Ok(row.get::<i64, _>("n"))
    }

    async fn find_by_content(
        &self,
        user_id: &str,
        content_sha256: &str,
    ) -> Result<Option<UserFontEntry>> {
        let Ok(uid) = uuid::Uuid::parse_str(user_id) else {
            return Ok(None);
        };
        let row = sqlx::query(&format!(
            "SELECT {COLS} FROM music.user_soundfonts \
             WHERE user_id = $1 AND content_sha256 = $2 LIMIT 1"
        ))
        .bind(uid)
        .bind(content_sha256)
        .fetch_optional(&self.pool)
        .await
        .context("find user soundfont by content")?;
        Ok(row.as_ref().map(row_to_entry))
    }

    async fn lookup(&self, user_id: &str, id: &str) -> Result<Option<UserFontEntry>> {
        let (Ok(uid), Ok(fid)) = (uuid::Uuid::parse_str(user_id), uuid::Uuid::parse_str(id)) else {
            return Ok(None);
        };
        let row = sqlx::query(&format!(
            "SELECT {COLS} FROM music.user_soundfonts WHERE user_id = $1 AND id = $2"
        ))
        .bind(uid)
        .bind(fid)
        .fetch_optional(&self.pool)
        .await
        .context("lookup user soundfont")?;
        Ok(row.as_ref().map(row_to_entry))
    }

    async fn insert(&self, entry: &UserFontEntry) -> Result<()> {
        let uid = uuid::Uuid::parse_str(&entry.user_id).context("user_id is not a uuid")?;
        let fid = uuid::Uuid::parse_str(&entry.id).context("id is not a uuid")?;
        sqlx::query(
            "INSERT INTO music.user_soundfonts \
             (id, user_id, label, object_key, content_sha256, size_bytes) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(fid)
        .bind(uid)
        .bind(&entry.label)
        .bind(&entry.object_key)
        .bind(&entry.content_sha256)
        .bind(entry.size_bytes)
        .execute(&self.pool)
        .await
        .context("insert user soundfont")?;
        Ok(())
    }

    async fn delete(&self, user_id: &str, id: &str) -> Result<Option<UserFontEntry>> {
        let (Ok(uid), Ok(fid)) = (uuid::Uuid::parse_str(user_id), uuid::Uuid::parse_str(id)) else {
            return Ok(None);
        };
        // Owner-scoped DELETE … RETURNING so the caller can clean up the object.
        let row = sqlx::query(&format!(
            "DELETE FROM music.user_soundfonts \
             WHERE user_id = $1 AND id = $2 RETURNING {COLS}"
        ))
        .bind(uid)
        .bind(fid)
        .fetch_optional(&self.pool)
        .await
        .context("delete user soundfont")?;
        Ok(row.as_ref().map(row_to_entry))
    }
}

/// In-memory [`UserSoundFontRepo`] for tests (no database).
#[derive(Default)]
pub struct FakeUserSoundFontRepo {
    rows: Mutex<Vec<UserFontEntry>>,
}

impl FakeUserSoundFontRepo {
    pub fn with(rows: Vec<UserFontEntry>) -> Self {
        Self {
            rows: Mutex::new(rows),
        }
    }
}

#[async_trait]
impl UserSoundFontRepo for FakeUserSoundFontRepo {
    async fn list(&self, user_id: &str) -> Result<Vec<UserFontEntry>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn count(&self, user_id: &str) -> Result<i64> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.user_id == user_id)
            .count() as i64)
    }

    async fn find_by_content(
        &self,
        user_id: &str,
        content_sha256: &str,
    ) -> Result<Option<UserFontEntry>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.user_id == user_id && e.content_sha256 == content_sha256)
            .cloned())
    }

    async fn lookup(&self, user_id: &str, id: &str) -> Result<Option<UserFontEntry>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.user_id == user_id && e.id == id)
            .cloned())
    }

    async fn insert(&self, entry: &UserFontEntry) -> Result<()> {
        self.rows.lock().unwrap().push(entry.clone());
        Ok(())
    }

    async fn delete(&self, user_id: &str, id: &str) -> Result<Option<UserFontEntry>> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(i) = rows.iter().position(|e| e.user_id == user_id && e.id == id) {
            Ok(Some(rows.remove(i)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, user: &str, sha: &str) -> UserFontEntry {
        UserFontEntry {
            id: id.into(),
            user_id: user.into(),
            label: format!("font {id}"),
            object_key: format!("user/{user}/{id}.sf2"),
            content_sha256: sha.into(),
            size_bytes: 100,
        }
    }

    #[tokio::test]
    async fn fake_is_owner_scoped_and_counts() {
        let repo = FakeUserSoundFontRepo::default();
        repo.insert(&entry("a", "u1", "sha-a")).await.unwrap();
        repo.insert(&entry("b", "u1", "sha-b")).await.unwrap();
        repo.insert(&entry("c", "u2", "sha-a")).await.unwrap();

        // Listing and counting are owner-scoped.
        assert_eq!(repo.list("u1").await.unwrap().len(), 2);
        assert_eq!(repo.count("u1").await.unwrap(), 2);
        assert_eq!(repo.count("u2").await.unwrap(), 1);

        // Content lookup is per-user: u1's "sha-a" matches a, u2's matches c.
        assert_eq!(
            repo.find_by_content("u1", "sha-a")
                .await
                .unwrap()
                .map(|e| e.id),
            Some("a".into())
        );
        assert_eq!(
            repo.find_by_content("u2", "sha-a")
                .await
                .unwrap()
                .map(|e| e.id),
            Some("c".into())
        );
        // A user can't see another's content by digest.
        assert!(
            repo.find_by_content("u1", "sha-nope")
                .await
                .unwrap()
                .is_none()
        );

        // Lookup is owner-scoped: u2 can't fetch u1's font.
        assert!(repo.lookup("u1", "a").await.unwrap().is_some());
        assert!(repo.lookup("u2", "a").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn delete_is_owner_scoped() {
        let repo = FakeUserSoundFontRepo::default();
        repo.insert(&entry("a", "u1", "sha-a")).await.unwrap();
        // Another user can't delete it.
        assert!(repo.delete("u2", "a").await.unwrap().is_none());
        assert_eq!(repo.count("u1").await.unwrap(), 1);
        // The owner can, and gets the removed row back (for object cleanup).
        let removed = repo.delete("u1", "a").await.unwrap();
        assert_eq!(removed.map(|e| e.object_key), Some("user/u1/a.sf2".into()));
        assert_eq!(repo.count("u1").await.unwrap(), 0);
        // Deleting again is a no-op not-found.
        assert!(repo.delete("u1", "a").await.unwrap().is_none());
    }
}
