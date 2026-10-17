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

/// A user's progress on one course. `passed_at` / `best_sync_pct` are the
/// piece-step outcome (change: add-music-piece-lessons): first pass, best run.
#[derive(Debug, Clone, PartialEq)]
pub struct CourseProgress {
    pub course_id: String,
    pub completed: bool,
    pub play_count: i64,
    pub passed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub best_sync_pct: Option<f32>,
}

/// The result of recording a piece-step run: whether this was the FIRST pass of
/// the course, and the best overall synchronisation after this run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PieceOutcome {
    pub newly_passed: bool,
    pub best_sync_pct: f32,
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

    /// Records one piece-step run of `course_id` by `user_id` (change:
    /// add-music-piece-lessons): `passed_at` is set on the first pass and kept,
    /// `best_sync_pct` only rises. Never marks the course completed.
    async fn record_piece_result(
        &self,
        user_id: &str,
        course_id: &str,
        passed: bool,
        sync_pct: f32,
    ) -> Result<PieceOutcome>;
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
            "SELECT course_id, completed_at IS NOT NULL AS completed, play_count, \
                    passed_at, best_sync_pct \
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
                passed_at: r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("passed_at"),
                best_sync_pct: r.get::<Option<f32>, _>("best_sync_pct"),
            })
            .collect())
    }

    async fn record_piece_result(
        &self,
        user_id: &str,
        course_id: &str,
        passed: bool,
        sync_pct: f32,
    ) -> Result<PieceOutcome> {
        let user = uuid::Uuid::parse_str(user_id).context("record piece result: bad user id")?;
        // The CTE reads the row BEFORE the upsert so "first pass" is exact; the
        // upsert itself never touches completed_at / play_count (a piece run is
        // not a lesson completion).
        let row = sqlx::query(
            "WITH old AS ( \
                 SELECT passed_at FROM music.course_progress \
                 WHERE user_id = $1 AND course_id = $2 \
             ), up AS ( \
                 INSERT INTO music.course_progress \
                     (user_id, course_id, play_count, passed_at, best_sync_pct) \
                 VALUES ($1, $2, 0, CASE WHEN $3 THEN now() END, $4) \
                 ON CONFLICT (user_id, course_id) DO UPDATE SET \
                     passed_at = COALESCE(music.course_progress.passed_at, \
                                          CASE WHEN $3 THEN now() END), \
                     best_sync_pct = GREATEST( \
                         COALESCE(music.course_progress.best_sync_pct, 0), $4), \
                     updated_at = now() \
                 RETURNING passed_at, best_sync_pct \
             ) \
             SELECT up.passed_at, up.best_sync_pct, \
                    (SELECT passed_at FROM old) AS old_passed_at \
             FROM up",
        )
        .bind(user)
        .bind(course_id)
        .bind(passed)
        .bind(sync_pct)
        .fetch_one(&self.pool)
        .await
        .context("record course piece result")?;
        let now_passed = row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("passed_at")
            .is_some();
        let was_passed = row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("old_passed_at")
            .is_some();
        Ok(PieceOutcome {
            newly_passed: now_passed && !was_passed,
            best_sync_pct: row.get::<Option<f32>, _>("best_sync_pct").unwrap_or(0.0),
        })
    }
}

/// One in-memory row of the fake store.
#[derive(Debug, Clone)]
struct FakeRow {
    user_id: String,
    course_id: String,
    /// `None` until the first completion (a piece run alone creates the row).
    play_count: Option<i64>,
    passed_at: Option<chrono::DateTime<chrono::Utc>>,
    best_sync_pct: Option<f32>,
}

/// In-memory [`CourseProgressStore`] for tests. Interior-mutable so the write
/// path works through the shared `&self` trait. Encodes the same idempotency as
/// the SQL upserts: first completion is `newly_completed`, replays only count;
/// first pass is `newly_passed`, the best only rises.
#[derive(Default)]
pub struct FakeCourseProgressStore {
    rows: Mutex<Vec<FakeRow>>,
}

impl FakeCourseProgressStore {
    fn upsert<'a>(rows: &'a mut Vec<FakeRow>, user_id: &str, course_id: &str) -> &'a mut FakeRow {
        if let Some(i) = rows
            .iter()
            .position(|r| r.user_id == user_id && r.course_id == course_id)
        {
            return &mut rows[i];
        }
        rows.push(FakeRow {
            user_id: user_id.to_string(),
            course_id: course_id.to_string(),
            play_count: None,
            passed_at: None,
            best_sync_pct: None,
        });
        rows.last_mut().unwrap()
    }
}

#[async_trait]
impl CourseProgressStore for FakeCourseProgressStore {
    async fn record_completion(&self, user_id: &str, course_id: &str) -> Result<CompletionOutcome> {
        let mut rows = self.rows.lock().unwrap();
        let row = Self::upsert(&mut rows, user_id, course_id);
        let newly = row.play_count.is_none();
        let count = row.play_count.unwrap_or(0) + 1;
        row.play_count = Some(count);
        Ok(CompletionOutcome {
            newly_completed: newly,
            play_count: count,
        })
    }

    async fn list(&self, user_id: &str) -> Result<Vec<CourseProgress>> {
        let mut out: Vec<CourseProgress> = self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.user_id == user_id)
            .map(|r| CourseProgress {
                course_id: r.course_id.clone(),
                completed: r.play_count.is_some(),
                play_count: r.play_count.unwrap_or(0),
                passed_at: r.passed_at,
                best_sync_pct: r.best_sync_pct,
            })
            .collect();
        out.sort_by(|a, b| a.course_id.cmp(&b.course_id));
        Ok(out)
    }

    async fn record_piece_result(
        &self,
        user_id: &str,
        course_id: &str,
        passed: bool,
        sync_pct: f32,
    ) -> Result<PieceOutcome> {
        let mut rows = self.rows.lock().unwrap();
        let row = Self::upsert(&mut rows, user_id, course_id);
        let newly_passed = passed && row.passed_at.is_none();
        if newly_passed {
            row.passed_at = Some(chrono::Utc::now());
        }
        let best = row.best_sync_pct.map_or(sync_pct, |b| b.max(sync_pct));
        row.best_sync_pct = Some(best);
        Ok(PieceOutcome {
            newly_passed,
            best_sync_pct: best,
        })
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
    async fn piece_results_pass_once_and_only_raise_the_best() {
        let store = FakeCourseProgressStore::default();
        // A run below the bar: a row appears, not completed, not passed.
        let a = store
            .record_piece_result(U, "c1", false, 71.0)
            .await
            .unwrap();
        assert_eq!(
            a,
            PieceOutcome {
                newly_passed: false,
                best_sync_pct: 71.0
            }
        );
        let rows = store.list(U).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].completed);
        assert!(rows[0].passed_at.is_none());
        assert_eq!(rows[0].best_sync_pct, Some(71.0));
        // The first pass is the signal; a later lower run changes nothing.
        let b = store
            .record_piece_result(U, "c1", true, 84.0)
            .await
            .unwrap();
        assert!(b.newly_passed);
        assert_eq!(b.best_sync_pct, 84.0);
        let c = store
            .record_piece_result(U, "c1", true, 80.0)
            .await
            .unwrap();
        assert!(!c.newly_passed);
        assert_eq!(c.best_sync_pct, 84.0);
        let d = store
            .record_piece_result(U, "c1", false, 60.0)
            .await
            .unwrap();
        assert!(!d.newly_passed);
        assert_eq!(d.best_sync_pct, 84.0);
        assert!(store.list(U).await.unwrap()[0].passed_at.is_some());
        // Completing afterwards is a first completion, untouched by the runs.
        let e = store.record_completion(U, "c1").await.unwrap();
        assert!(e.newly_completed);
        assert_eq!(e.play_count, 1);
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
