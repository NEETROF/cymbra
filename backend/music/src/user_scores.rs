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

//! User-uploaded scores: the `user_scores` data-access port.
//!
//! Distinct from the crawler's [`crate::CatalogRepo`] (which is `anyhow`-based and
//! written to directly): this surface is gRPC-facing, so it returns the platform
//! [`Result`]/`AppError` the handler propagates. Every method is **owner-scoped**
//! — there is no cross-owner read/delete path at the data layer.

use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};

/// One contributed-score row. Descriptive fields (`title`..`is_piano`) are
/// **server-derived** from the parsed file (design 2b); `level`/`rights_*` are the
/// only caller-owned inputs. `created_at` is unix seconds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserScore {
    pub id: String,
    pub owner_id: String,
    pub level: String,
    pub rights_basis: String,
    pub rights_ack: bool,
    pub title: Option<String>,
    pub composer: Option<String>,
    pub title_norm: Option<String>,
    pub work_key: String,
    pub key_fifths: i32,
    pub time_sig: String,
    pub measure_count: i32,
    pub is_piano: bool,
    pub sha256: String,
    pub size_bytes: i64,
    pub object_key: String,
    pub created_at: i64,
}

/// Owner-scoped storage surface for user uploads.
#[async_trait]
pub trait UserScoreRepo: Send + Sync {
    /// Insert a new record. A per-owner `sha256` duplicate is an `AlreadyExists`.
    async fn insert(&self, s: &UserScore) -> Result<()>;
    /// The caller's scores, newest first.
    async fn list_by_owner(&self, owner_id: &str) -> Result<Vec<UserScore>>;
    /// One score the caller owns; `None` if absent or owned by someone else.
    async fn get_owned(&self, id: &str, owner_id: &str) -> Result<Option<UserScore>>;
    /// Delete a score the caller owns, returning the removed row (its `object_key`
    /// drives object cleanup); `None` if absent or not theirs.
    async fn delete_owned(&self, id: &str, owner_id: &str) -> Result<Option<UserScore>>;
    /// Count the caller's scores created within the last `window_days` (quota).
    async fn count_recent(&self, owner_id: &str, window_days: u32) -> Result<i64>;
}

/// In-memory [`UserScoreRepo`] for unit tests.
#[derive(Default)]
pub struct FakeUserScoreRepo {
    rows: Mutex<Vec<UserScore>>,
}

impl FakeUserScoreRepo {
    /// Snapshot of all stored rows (test assertions).
    pub fn rows(&self) -> Vec<UserScore> {
        self.rows.lock().expect("user_scores fake lock").clone()
    }
}

fn cutoff_unix(window_days: u32) -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    now - (window_days as i64) * 86_400
}

#[async_trait]
impl UserScoreRepo for FakeUserScoreRepo {
    async fn insert(&self, s: &UserScore) -> Result<()> {
        let mut rows = self.rows.lock().expect("user_scores fake lock");
        if rows
            .iter()
            .any(|r| r.owner_id == s.owner_id && r.sha256 == s.sha256)
        {
            return Err(AppError::AlreadyExists("score already uploaded".into()));
        }
        rows.push(s.clone());
        Ok(())
    }

    async fn list_by_owner(&self, owner_id: &str) -> Result<Vec<UserScore>> {
        let rows = self.rows.lock().expect("user_scores fake lock");
        let mut mine: Vec<UserScore> = rows
            .iter()
            .filter(|r| r.owner_id == owner_id)
            .cloned()
            .collect();
        mine.sort_by_key(|r| std::cmp::Reverse(r.created_at));
        Ok(mine)
    }

    async fn get_owned(&self, id: &str, owner_id: &str) -> Result<Option<UserScore>> {
        let rows = self.rows.lock().expect("user_scores fake lock");
        Ok(rows
            .iter()
            .find(|r| r.id == id && r.owner_id == owner_id)
            .cloned())
    }

    async fn delete_owned(&self, id: &str, owner_id: &str) -> Result<Option<UserScore>> {
        let mut rows = self.rows.lock().expect("user_scores fake lock");
        if let Some(pos) = rows
            .iter()
            .position(|r| r.id == id && r.owner_id == owner_id)
        {
            Ok(Some(rows.remove(pos)))
        } else {
            Ok(None)
        }
    }

    async fn count_recent(&self, owner_id: &str, window_days: u32) -> Result<i64> {
        let cutoff = cutoff_unix(window_days);
        let rows = self.rows.lock().expect("user_scores fake lock");
        Ok(rows
            .iter()
            .filter(|r| r.owner_id == owner_id && r.created_at >= cutoff)
            .count() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(id: &str, owner: &str, sha: &str, created_at: i64) -> UserScore {
        UserScore {
            id: id.into(),
            owner_id: owner.into(),
            level: "beginner".into(),
            rights_basis: "own_work".into(),
            rights_ack: true,
            title: Some("T".into()),
            composer: None,
            title_norm: Some("t".into()),
            work_key: "::t".into(),
            key_fifths: 0,
            time_sig: "4/4".into(),
            measure_count: 4,
            is_piano: true,
            sha256: sha.into(),
            size_bytes: 100,
            object_key: format!("user-scores/{owner}/{id}.mxl"),
            created_at,
        }
    }

    fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    #[tokio::test]
    async fn insert_rejects_per_owner_sha_duplicate() {
        let r = FakeUserScoreRepo::default();
        r.insert(&score("a", "u1", "sha1", now())).await.unwrap();
        // Same owner + same sha → conflict.
        assert!(matches!(
            r.insert(&score("b", "u1", "sha1", now())).await,
            Err(AppError::AlreadyExists(_))
        ));
        // A different owner may hold the same work.
        r.insert(&score("c", "u2", "sha1", now())).await.unwrap();
        assert_eq!(r.rows().len(), 2);
    }

    #[tokio::test]
    async fn list_and_get_are_owner_scoped_and_ordered() {
        let r = FakeUserScoreRepo::default();
        r.insert(&score("a", "u1", "s1", 100)).await.unwrap();
        r.insert(&score("b", "u1", "s2", 200)).await.unwrap();
        r.insert(&score("c", "u2", "s3", 300)).await.unwrap();
        let mine = r.list_by_owner("u1").await.unwrap();
        assert_eq!(
            mine.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
            vec!["b", "a"] // newest first
        );
        // u1 cannot see u2's score even by id.
        assert!(r.get_owned("c", "u1").await.unwrap().is_none());
        assert!(r.get_owned("c", "u2").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn delete_is_owner_scoped_and_returns_row() {
        let r = FakeUserScoreRepo::default();
        r.insert(&score("a", "u1", "s1", now())).await.unwrap();
        // Non-owner delete is a no-op returning None.
        assert!(r.delete_owned("a", "u2").await.unwrap().is_none());
        let removed = r.delete_owned("a", "u1").await.unwrap().unwrap();
        assert_eq!(removed.object_key, "user-scores/u1/a.mxl");
        assert!(r.rows().is_empty());
    }

    #[tokio::test]
    async fn count_recent_respects_the_rolling_window() {
        let r = FakeUserScoreRepo::default();
        let n = now();
        // Two recent, one 10 days old.
        r.insert(&score("a", "u1", "s1", n)).await.unwrap();
        r.insert(&score("b", "u1", "s2", n - 3600)).await.unwrap();
        r.insert(&score("old", "u1", "s3", n - 10 * 86_400))
            .await
            .unwrap();
        assert_eq!(r.count_recent("u1", 7).await.unwrap(), 2); // old one excluded
        assert_eq!(r.count_recent("u1", 30).await.unwrap(), 3); // all in a 30d window
        assert_eq!(r.count_recent("u2", 7).await.unwrap(), 0); // owner-scoped
    }
}
