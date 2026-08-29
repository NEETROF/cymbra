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

//! The privileged, cross-owner surface over private scores — notice-and-takedown
//! (change: add-private-score-catalog).
//!
//! Deliberately a SEPARATE port from [`crate::user_scores`]: that one states, and
//! keeps, the property that no cross-owner read or delete exists at the data
//! layer. Takedown needs exactly such a path, so it lives here where it is
//! explicit, gated on a music-scope admin at the handler, and audited.
//!
//! The audit row is written **before** the score row and its object are removed,
//! and outlives them — it carries the identifying metadata (`sha256`, `title`)
//! precisely because the content it describes is gone.

use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_platform::Result;

use crate::user_scores::UserScore;

/// One takedown audit row. `created_at` is unix seconds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Takedown {
    pub id: String,
    pub user_score_id: String,
    pub owner_id: String,
    pub admin_id: String,
    pub sha256: String,
    pub title: Option<String>,
    pub reason: String,
    pub created_at: i64,
}

/// Cross-owner administration of private scores. Never serves bytes.
#[async_trait]
pub trait UserScoreAdminRepo: Send + Sync {
    /// Paged search by owner and/or case-insensitive title fragment, newest first.
    /// The "at least one criterion" rule is enforced above this: the port would
    /// happily answer an unfiltered scan, the module refuses to ask for one.
    async fn search(
        &self,
        owner_id: Option<&str>,
        title: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserScore>>;

    /// One score by id, whoever owns it; `None` if absent.
    async fn get_any(&self, id: &str) -> Result<Option<UserScore>>;

    /// Persist an audit row. Called BEFORE the deletion, so the record survives it.
    async fn record_takedown(&self, t: &Takedown) -> Result<()>;

    /// Delete a score row whoever owns it, returning the removed row (its
    /// `object_key` drives object cleanup); `None` if it was already gone.
    async fn delete_any(&self, id: &str) -> Result<Option<UserScore>>;
}

/// In-memory [`UserScoreAdminRepo`] for unit tests.
#[derive(Default)]
pub struct FakeUserScoreAdminRepo {
    scores: Mutex<Vec<UserScore>>,
    takedowns: Mutex<Vec<Takedown>>,
}

impl FakeUserScoreAdminRepo {
    /// Seed a score visible to the admin surface.
    pub fn seed(&self, s: UserScore) {
        self.scores.lock().expect("admin fake lock").push(s);
    }

    /// Snapshot of the audit rows (test assertions).
    pub fn takedowns(&self) -> Vec<Takedown> {
        self.takedowns.lock().expect("admin fake lock").clone()
    }

    /// Snapshot of the surviving score rows (test assertions).
    pub fn scores(&self) -> Vec<UserScore> {
        self.scores.lock().expect("admin fake lock").clone()
    }
}

#[async_trait]
impl UserScoreAdminRepo for FakeUserScoreAdminRepo {
    async fn search(
        &self,
        owner_id: Option<&str>,
        title: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserScore>> {
        let rows = self.scores.lock().expect("admin fake lock");
        let needle = title.map(str::to_lowercase);
        let mut hits: Vec<UserScore> = rows
            .iter()
            .filter(|s| owner_id.is_none_or(|o| s.owner_id == o))
            .filter(|s| {
                needle.as_ref().is_none_or(|n| {
                    s.meta
                        .title
                        .as_deref()
                        .is_some_and(|t| t.to_lowercase().contains(n))
                })
            })
            .cloned()
            .collect();
        hits.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(hits
            .into_iter()
            .skip(offset.max(0) as usize)
            .take(limit.max(0) as usize)
            .collect())
    }

    async fn get_any(&self, id: &str) -> Result<Option<UserScore>> {
        Ok(self
            .scores
            .lock()
            .expect("admin fake lock")
            .iter()
            .find(|s| s.id == id)
            .cloned())
    }

    async fn record_takedown(&self, t: &Takedown) -> Result<()> {
        self.takedowns
            .lock()
            .expect("admin fake lock")
            .push(t.clone());
        Ok(())
    }

    async fn delete_any(&self, id: &str) -> Result<Option<UserScore>> {
        let mut rows = self.scores.lock().expect("admin fake lock");
        match rows.iter().position(|s| s.id == id) {
            Some(i) => Ok(Some(rows.remove(i))),
            None => Ok(None),
        }
    }
}
