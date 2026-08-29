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

//! Collections over the private score library (change: add-private-score-catalog).
//!
//! A collection is a **named, owner-scoped grouping** of one's own uploads —
//! tag-like, not foldering: a score may sit in several collections at once, and a
//! collection holds no bytes. Deleting a collection never touches a score, and
//! deleting a score only drops its memberships (the DB cascades both ways).
//!
//! Every method is owner-scoped: the owner is a parameter, never inferred, and a
//! mismatch reads as absent rather than forbidden — the same "no cross-owner path
//! at the data layer" stance as [`crate::user_scores`]. Name uniqueness is
//! **case-insensitive per owner**, so one cannot hold both "Chopin" and "chopin";
//! the conflict surfaces as [`AppError::AlreadyExists`] so the app can localise it
//! instead of showing a constraint violation.

use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};

/// One collection row. `created_at` is unix seconds (newest-first ordering).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub created_at: i64,
}

/// Owner-scoped storage for private-library collections and their membership.
#[async_trait]
pub trait UserCollectionRepo: Send + Sync {
    /// Insert a collection. A case-insensitive name already used by this owner is
    /// an [`AppError::AlreadyExists`].
    async fn create(&self, c: &Collection) -> Result<()>;

    /// Rename a collection the caller owns. [`AppError::NotFound`] if absent or not
    /// theirs; [`AppError::AlreadyExists`] on a case-insensitive name collision with
    /// another of their collections (renaming to its own current name is a success).
    async fn rename(&self, id: &str, owner_id: &str, name: &str) -> Result<()>;

    /// Delete a collection the caller owns; its memberships cascade. Never deletes a
    /// score. [`AppError::NotFound`] if absent or not theirs.
    async fn delete(&self, id: &str, owner_id: &str) -> Result<()>;

    /// The caller's collections, newest first.
    async fn list(&self, owner_id: &str) -> Result<Vec<Collection>>;

    /// Add one of the caller's scores to one of their collections. Idempotent.
    /// [`AppError::NotFound`] unless BOTH the collection and the score exist and
    /// belong to `owner_id`.
    async fn add_item(&self, collection_id: &str, score_id: &str, owner_id: &str) -> Result<()>;

    /// Remove a score from a collection. Idempotent — removing an absent membership
    /// is a success. [`AppError::NotFound`] if the collection is absent or not theirs.
    async fn remove_item(&self, collection_id: &str, score_id: &str, owner_id: &str) -> Result<()>;

    /// The score ids in one of the caller's collections, newest-added first.
    /// [`AppError::NotFound`] if the collection is absent or not theirs.
    async fn list_score_ids(&self, collection_id: &str, owner_id: &str) -> Result<Vec<String>>;
}

/// In-memory [`UserCollectionRepo`] for unit tests. A monotonic counter stands in
/// for the clock so newest-first ordering is deterministic.
#[derive(Default)]
pub struct FakeUserCollectionRepo {
    rows: Mutex<Vec<Collection>>,
    items: Mutex<Vec<(String, String, u64)>>, // (collection_id, score_id, seq)
    /// Score ids per owner, seeded by tests so ownership validation has something
    /// to check without dragging the whole user-scores repo in.
    owned_scores: Mutex<Vec<(String, String)>>, // (owner_id, score_id)
    seq: Mutex<u64>,
}

impl FakeUserCollectionRepo {
    /// Declare that `owner_id` owns `score_id` (test setup for `add_item`).
    pub fn own_score(&self, owner_id: &str, score_id: &str) {
        self.owned_scores
            .lock()
            .expect("collections fake lock")
            .push((owner_id.into(), score_id.into()));
    }

    /// Snapshot of the membership rows (test assertions).
    pub fn items(&self) -> Vec<(String, String)> {
        self.items
            .lock()
            .expect("collections fake lock")
            .iter()
            .map(|(c, s, _)| (c.clone(), s.clone()))
            .collect()
    }

    fn next_seq(&self) -> u64 {
        let mut s = self.seq.lock().expect("collections fake lock");
        *s += 1;
        *s
    }

    fn owns(&self, id: &str, owner_id: &str) -> bool {
        self.rows
            .lock()
            .expect("collections fake lock")
            .iter()
            .any(|c| c.id == id && c.owner_id == owner_id)
    }
}

/// Case-insensitive comparison used for the per-owner name uniqueness, mirroring
/// the `lower(name)` functional unique index.
fn same_name(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

#[async_trait]
impl UserCollectionRepo for FakeUserCollectionRepo {
    async fn create(&self, c: &Collection) -> Result<()> {
        let mut rows = self.rows.lock().expect("collections fake lock");
        if rows
            .iter()
            .any(|r| r.owner_id == c.owner_id && same_name(&r.name, &c.name))
        {
            return Err(AppError::AlreadyExists("collection name taken".into()));
        }
        rows.push(c.clone());
        Ok(())
    }

    async fn rename(&self, id: &str, owner_id: &str, name: &str) -> Result<()> {
        let mut rows = self.rows.lock().expect("collections fake lock");
        if !rows.iter().any(|r| r.id == id && r.owner_id == owner_id) {
            return Err(AppError::NotFound("collection not found".into()));
        }
        if rows
            .iter()
            .any(|r| r.owner_id == owner_id && r.id != id && same_name(&r.name, name))
        {
            return Err(AppError::AlreadyExists("collection name taken".into()));
        }
        for r in rows.iter_mut().filter(|r| r.id == id) {
            r.name = name.to_string();
        }
        Ok(())
    }

    async fn delete(&self, id: &str, owner_id: &str) -> Result<()> {
        let mut rows = self.rows.lock().expect("collections fake lock");
        let before = rows.len();
        rows.retain(|r| !(r.id == id && r.owner_id == owner_id));
        if rows.len() == before {
            return Err(AppError::NotFound("collection not found".into()));
        }
        // Memberships cascade with the collection (the DB does this with an FK).
        self.items
            .lock()
            .expect("collections fake lock")
            .retain(|(c, _, _)| c != id);
        Ok(())
    }

    async fn list(&self, owner_id: &str) -> Result<Vec<Collection>> {
        let rows = self.rows.lock().expect("collections fake lock");
        let mut mine: Vec<Collection> = rows
            .iter()
            .filter(|r| r.owner_id == owner_id)
            .cloned()
            .collect();
        mine.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(mine)
    }

    async fn add_item(&self, collection_id: &str, score_id: &str, owner_id: &str) -> Result<()> {
        if !self.owns(collection_id, owner_id) {
            return Err(AppError::NotFound("collection not found".into()));
        }
        let owns_score = self
            .owned_scores
            .lock()
            .expect("collections fake lock")
            .iter()
            .any(|(o, s)| o == owner_id && s == score_id);
        if !owns_score {
            return Err(AppError::NotFound("score not found".into()));
        }
        let seq = self.next_seq();
        let mut items = self.items.lock().expect("collections fake lock");
        if !items
            .iter()
            .any(|(c, s, _)| c == collection_id && s == score_id)
        {
            items.push((collection_id.into(), score_id.into(), seq));
        }
        Ok(())
    }

    async fn remove_item(&self, collection_id: &str, score_id: &str, owner_id: &str) -> Result<()> {
        if !self.owns(collection_id, owner_id) {
            return Err(AppError::NotFound("collection not found".into()));
        }
        self.items
            .lock()
            .expect("collections fake lock")
            .retain(|(c, s, _)| !(c == collection_id && s == score_id));
        Ok(())
    }

    async fn list_score_ids(&self, collection_id: &str, owner_id: &str) -> Result<Vec<String>> {
        if !self.owns(collection_id, owner_id) {
            return Err(AppError::NotFound("collection not found".into()));
        }
        let items = self.items.lock().expect("collections fake lock");
        let mut mine: Vec<&(String, String, u64)> = items
            .iter()
            .filter(|(c, _, _)| c == collection_id)
            .collect();
        mine.sort_by(|a, b| b.2.cmp(&a.2));
        Ok(mine.iter().map(|(_, s, _)| s.clone()).collect())
    }
}
