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

//! Per-user course completion, cross-device (change: add-notation-courses).
//!
//! Idempotent by construction: [`record_completion`](CourseProgressStore::
//! record_completion) sets `completed_at` on the **first** completion (returning
//! `newly_completed = true`, the signal to award the badge exactly once) and on
//! any later replay only bumps `play_count`. Behind a trait so the RPCs are
//! testable with an in-memory [`FakeCourseProgressStore`].

use std::sync::Mutex;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Row};

/// A user's progress on one course.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseProgress {
    pub course_id: String,
    pub completed: bool,
    pub play_count: i64,
}

/// The result of recording a completion: whether this was the **first** time the
/// course was completed (so the badge is awarded once), and the running count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletionOutcome {
    pub newly_completed: bool,
    pub play_count: i64,
}

/// Read + record access to per-user course completion. Behind a trait so the
/// `RecordCourseCompletion`/`GetCourseProgress` RPCs are testable with an
/// in-memory [`FakeCourseProgressStore`].
#[async_trait]
pub trait CourseProgressStore: Send + Sync {
    /// Records a completion of `course_id` by `user_id`. First completion sets it
    /// done (`newly_completed = true`); a replay only increments the count.
    async fn record_completion(&self, user_id: &str, course_id: &str) -> Result<CompletionOutcome>;

    /// Every course the user has any progress on (completed courses included).
    async fn list(&self, user_id: &str) -> Result<Vec<CourseProgress>>;
}

/// Postgres-backed [`CourseProgressStore`] over `music.course_progress`.
pub struct PgCourseProgressStore {
    pool: PgPool,
}

impl PgCourseProgressStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CourseProgressStore for PgCourseProgressStore {
    async fn record_completion(&self, user_id: &str, course_id: &str) -> Result<CompletionOutcome> {
        let user = uuid::Uuid::parse_str(user_id).context("record completion: bad user id")?;
        // Upsert: the first completion inserts (xmax = 0), a replay updates and
        // keeps the original completed_at. `inserted` is exactly the badge signal.
        let row = sqlx::query(
            "INSERT INTO music.course_progress (user_id, course_id, completed_at, play_count) \
             VALUES ($1, $2, now(), 1) \
             ON CONFLICT (user_id, course_id) DO UPDATE SET \
               play_count = music.course_progress.play_count + 1, \
               completed_at = COALESCE(music.course_progress.completed_at, now()), \
               updated_at = now() \
             RETURNING (xmax = 0) AS inserted, play_count",
        )
        .bind(user)
        .bind(course_id)
        .fetch_one(&self.pool)
        .await
        .context("record course completion")?;
        Ok(CompletionOutcome {
            newly_completed: row.get::<bool, _>("inserted"),
            play_count: row.get::<i32, _>("play_count") as i64,
        })
    }

    async fn list(&self, user_id: &str) -> Result<Vec<CourseProgress>> {
        // A non-UUID user id can hold no progress; empty rather than an error.
        let Ok(user) = uuid::Uuid::parse_str(user_id) else {
            return Ok(Vec::new());
        };
        let rows = sqlx::query(
            "SELECT course_id, completed_at IS NOT NULL AS completed, play_count \
             FROM music.course_progress WHERE user_id = $1 ORDER BY course_id",
        )
        .bind(user)
        .fetch_all(&self.pool)
        .await
        .context("list course progress")?;
        Ok(rows
            .iter()
            .map(|r| CourseProgress {
                course_id: r.get::<String, _>("course_id"),
                completed: r.get::<bool, _>("completed"),
                play_count: r.get::<i32, _>("play_count") as i64,
            })
            .collect())
    }
}

/// In-memory [`CourseProgressStore`] for tests. Interior-mutable so the write
/// path works through the shared `&self` trait. Encodes the same idempotency as
/// the SQL upsert: first completion is `newly_completed`, replays only count.
#[derive(Default)]
pub struct FakeCourseProgressStore {
    // (user_id, course_id) -> play_count (presence ⇒ completed).
    rows: Mutex<Vec<(String, String, i64)>>,
}

#[async_trait]
impl CourseProgressStore for FakeCourseProgressStore {
    async fn record_completion(&self, user_id: &str, course_id: &str) -> Result<CompletionOutcome> {
        let mut rows = self.rows.lock().unwrap();
        match rows
            .iter_mut()
            .find(|(u, c, _)| u == user_id && c == course_id)
        {
            Some((_, _, count)) => {
                *count += 1;
                Ok(CompletionOutcome {
                    newly_completed: false,
                    play_count: *count,
                })
            }
            None => {
                rows.push((user_id.to_string(), course_id.to_string(), 1));
                Ok(CompletionOutcome {
                    newly_completed: true,
                    play_count: 1,
                })
            }
        }
    }

    async fn list(&self, user_id: &str) -> Result<Vec<CourseProgress>> {
        let mut out: Vec<CourseProgress> = self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|(u, _, _)| u == user_id)
            .map(|(_, c, count)| CourseProgress {
                course_id: c.clone(),
                completed: true,
                play_count: *count,
            })
            .collect();
        out.sort_by(|a, b| a.course_id.cmp(&b.course_id));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const U: &str = "u1";

    #[tokio::test]
    async fn first_completion_is_new_then_replays_only_count() {
        let store = FakeCourseProgressStore::default();
        let a = store.record_completion(U, "c1").await.unwrap();
        assert_eq!(
            a,
            CompletionOutcome {
                newly_completed: true,
                play_count: 1
            }
        );
        // Replaying the same course never re-awards; the count grows.
        let b = store.record_completion(U, "c1").await.unwrap();
        assert_eq!(
            b,
            CompletionOutcome {
                newly_completed: false,
                play_count: 2
            }
        );
    }

    #[tokio::test]
    async fn list_returns_completed_courses_per_user() {
        let store = FakeCourseProgressStore::default();
        store.record_completion(U, "c1").await.unwrap();
        store.record_completion(U, "c2").await.unwrap();
        store.record_completion("other", "c9").await.unwrap();

        let mine = store.list(U).await.unwrap();
        assert_eq!(mine.len(), 2);
        assert!(mine.iter().all(|p| p.completed));
        assert_eq!(mine[0].course_id, "c1");
        assert_eq!(mine[1].course_id, "c2");
        // Isolated per user.
        assert_eq!(store.list("other").await.unwrap().len(), 1);
        assert_eq!(store.list("nobody").await.unwrap().len(), 0);
    }
}
