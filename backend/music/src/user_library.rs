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

//! The per-user saved-catalog library data-access port (change: score-hub-search).
//!
//! Owner-scoped, gRPC-facing (returns the platform [`Result`]). A save records an
//! `(owner_id, catalog_id)` row; the set is the backend source of truth, so it
//! syncs across the account's devices. `save`/`remove` are idempotent. `list_ids`
//! returns the caller's saved catalog ids newest-first; the module joins them to
//! the catalog (omitting any whose entry is gone) to build the saved list.

use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_platform::Result;

/// Owner-scoped storage for saved catalog scores.
#[async_trait]
pub trait UserLibraryRepo: Send + Sync {
    /// Record a save. Idempotent: saving an already-saved score is a success that
    /// creates no duplicate.
    async fn save(&self, owner_id: &str, catalog_id: &str) -> Result<()>;

    /// Remove a save. Idempotent: removing a not-saved score is a no-op success.
    async fn remove(&self, owner_id: &str, catalog_id: &str) -> Result<()>;

    /// The caller's saved catalog ids, newest-saved first.
    async fn list_ids(&self, owner_id: &str) -> Result<Vec<String>>;
}

/// In-memory [`UserLibraryRepo`] for unit tests. A monotonic counter stands in for
/// `created_at` so newest-first ordering is deterministic without a clock.
#[derive(Default)]
pub struct FakeUserLibraryRepo {
    rows: Mutex<Vec<Saved>>,
    seq: Mutex<u64>,
}

#[derive(Clone)]
struct Saved {
    owner_id: String,
    catalog_id: String,
    seq: u64,
}

impl FakeUserLibraryRepo {
    /// Number of saved rows for an owner (test assertions).
    pub fn count(&self, owner_id: &str) -> usize {
        self.rows
            .lock()
            .expect("user_library fake lock")
            .iter()
            .filter(|r| r.owner_id == owner_id)
            .count()
    }
}

#[async_trait]
impl UserLibraryRepo for FakeUserLibraryRepo {
    async fn save(&self, owner_id: &str, catalog_id: &str) -> Result<()> {
        let mut rows = self.rows.lock().expect("user_library fake lock");
        if rows
            .iter()
            .any(|r| r.owner_id == owner_id && r.catalog_id == catalog_id)
        {
            return Ok(()); // idempotent — no duplicate
        }
        let mut seq = self.seq.lock().expect("user_library fake seq");
        *seq += 1;
        rows.push(Saved {
            owner_id: owner_id.to_string(),
            catalog_id: catalog_id.to_string(),
            seq: *seq,
        });
        Ok(())
    }

    async fn remove(&self, owner_id: &str, catalog_id: &str) -> Result<()> {
        let mut rows = self.rows.lock().expect("user_library fake lock");
        rows.retain(|r| !(r.owner_id == owner_id && r.catalog_id == catalog_id));
        Ok(())
    }

    async fn list_ids(&self, owner_id: &str) -> Result<Vec<String>> {
        let rows = self.rows.lock().expect("user_library fake lock");
        let mut mine: Vec<&Saved> = rows.iter().filter(|r| r.owner_id == owner_id).collect();
        mine.sort_by_key(|r| std::cmp::Reverse(r.seq)); // newest-saved first
        Ok(mine.into_iter().map(|r| r.catalog_id.clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn save_is_idempotent_and_owner_scoped() {
        let repo = FakeUserLibraryRepo::default();
        repo.save("u1", "c1").await.unwrap();
        repo.save("u1", "c1").await.unwrap(); // duplicate → no-op
        repo.save("u2", "c1").await.unwrap(); // different owner, same score
        assert_eq!(repo.count("u1"), 1);
        assert_eq!(repo.count("u2"), 1);
    }

    #[tokio::test]
    async fn remove_is_idempotent_and_owner_scoped() {
        let repo = FakeUserLibraryRepo::default();
        repo.save("u1", "c1").await.unwrap();
        repo.save("u2", "c1").await.unwrap();
        // Removing a not-saved score is a no-op success.
        repo.remove("u1", "cX").await.unwrap();
        assert_eq!(repo.count("u1"), 1);
        // Removing is owner-scoped: u1's remove leaves u2's save intact.
        repo.remove("u1", "c1").await.unwrap();
        assert_eq!(repo.count("u1"), 0);
        assert_eq!(repo.count("u2"), 1);
    }

    #[tokio::test]
    async fn list_reflects_saves_newest_first_and_removals() {
        let repo = FakeUserLibraryRepo::default();
        repo.save("u1", "a").await.unwrap();
        repo.save("u1", "b").await.unwrap();
        repo.save("u1", "c").await.unwrap();
        assert_eq!(repo.list_ids("u1").await.unwrap(), ["c", "b", "a"]);
        // A removal is reflected in a subsequent list (cross-session sync source).
        repo.remove("u1", "b").await.unwrap();
        assert_eq!(repo.list_ids("u1").await.unwrap(), ["c", "a"]);
        // Isolation: another owner sees nothing of u1's.
        assert!(repo.list_ids("u2").await.unwrap().is_empty());
    }
}
