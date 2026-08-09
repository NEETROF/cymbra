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

//! The persisted interactive-course catalog (change: add-notation-courses).
//!
//! A course is a self-describing, versioned JSON **manifest** (a block DSL) stored
//! as an opaque `JSONB` blob in `music.courses`: the backend serves it without
//! understanding its blocks — the client owns the format and its forward-
//! compatibility. `ListCourses` returns lightweight [`CourseSummary`]s (no
//! content) grouped by track/level; `GetCourse` returns the full [`Course`].
//!
//! Behind a trait so the RPCs and the seed path are testable with an in-memory
//! [`FakeCourseRepo`]. `title`/`content` are carried as raw JSON **text** (read
//! with a `::text` cast, written with a `::jsonb` cast), so this crate does not
//! depend on a JSON column feature; both surfaces treat them as opaque strings.

use std::sync::Mutex;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use sqlx::{PgPool, Row, postgres::PgRow};

/// Listing metadata for a course — everything the home screen needs to draw a
/// tile and group it, without the (potentially large) manifest body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseSummary {
    pub id: String,
    /// Instrument the course targets (e.g. `piano`); interactive blocks are
    /// instrument-scoped, so a later drums track reuses the format.
    pub instrument: String,
    /// Grouping dimensions for the home section (e.g. solfège / app-usage /
    /// technique × beginner / intermediate / advanced).
    pub track: String,
    pub level: String,
    /// Display order within its track/level.
    pub sort_order: i32,
    /// Manifest schema version; the client declines a version it cannot handle.
    pub schema_version: i32,
    /// Inline-localized title object (`{en, fr, es, it}`) as raw JSON text.
    pub title: String,
    /// Unit slug the course belongs to *within* its track/level section — a
    /// display grouping only, never an ordering key (empty = ungrouped).
    pub unit: String,
    /// Inline-localized unit heading (`{en, fr, es, it}`) as raw JSON text.
    pub unit_title: String,
}

/// A full course: its [`CourseSummary`] plus the manifest [`content`] (the block
/// DSL) as raw JSON text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Course {
    pub summary: CourseSummary,
    pub content: String,
}

/// Read + seed access to the persisted course catalog. Behind a trait so the
/// `ListCourses`/`GetCourse` RPCs and the seed path are testable with an
/// in-memory [`FakeCourseRepo`].
#[async_trait]
pub trait CourseRepo: Send + Sync {
    /// Published courses, ordered by track, level, then display order — the home
    /// listing (metadata only, no manifest body).
    async fn list_published(&self) -> Result<Vec<CourseSummary>>;
    /// A published course by id, with its manifest, or `None` when unknown/unpublished.
    async fn get(&self, id: &str) -> Result<Option<Course>>;
    /// Insert (or replace) a course — the seed path. Overwrites an existing id so
    /// re-seeding an updated manifest is idempotent.
    async fn upsert(&self, course: &Course, status: &str) -> Result<()>;
}

const SUMMARY_COLS: &str = "id, instrument, track, level, sort_order, schema_version, \
     title::text AS title, unit, unit_title::text AS unit_title";

fn row_to_summary(row: &PgRow) -> CourseSummary {
    CourseSummary {
        id: row.get::<String, _>("id"),
        instrument: row.get::<String, _>("instrument"),
        track: row.get::<String, _>("track"),
        level: row.get::<String, _>("level"),
        sort_order: row.get::<i32, _>("sort_order"),
        schema_version: row.get::<i32, _>("schema_version"),
        title: row.get::<String, _>("title"),
        unit: row.get::<String, _>("unit"),
        unit_title: row.get::<String, _>("unit_title"),
    }
}

/// Postgres-backed [`CourseRepo`] over the `music.courses` table.
pub struct PgCourseRepo {
    pool: PgPool,
}

impl PgCourseRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CourseRepo for PgCourseRepo {
    async fn list_published(&self) -> Result<Vec<CourseSummary>> {
        let rows = sqlx::query(&format!(
            "SELECT {SUMMARY_COLS} FROM music.courses \
             WHERE status = 'published' ORDER BY track, level, sort_order, id"
        ))
        .fetch_all(&self.pool)
        .await
        .context("list published courses")?;
        Ok(rows.iter().map(row_to_summary).collect())
    }

    async fn get(&self, id: &str) -> Result<Option<Course>> {
        let row = sqlx::query(&format!(
            "SELECT {SUMMARY_COLS}, content::text AS content FROM music.courses \
             WHERE id = $1 AND status = 'published'"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("get course")?;
        Ok(row.as_ref().map(|r| Course {
            summary: row_to_summary(r),
            content: r.get::<String, _>("content"),
        }))
    }

    async fn upsert(&self, course: &Course, status: &str) -> Result<()> {
        let s = &course.summary;
        sqlx::query(
            "INSERT INTO music.courses \
             (id, status, instrument, track, level, sort_order, schema_version, title, content, \
              unit, unit_title) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8::jsonb, $9::jsonb, $10, $11::jsonb) \
             ON CONFLICT (id) DO UPDATE SET \
               status = EXCLUDED.status, instrument = EXCLUDED.instrument, \
               track = EXCLUDED.track, level = EXCLUDED.level, \
               sort_order = EXCLUDED.sort_order, schema_version = EXCLUDED.schema_version, \
               title = EXCLUDED.title, content = EXCLUDED.content, \
               unit = EXCLUDED.unit, unit_title = EXCLUDED.unit_title, updated_at = now()",
        )
        .bind(&s.id)
        .bind(status)
        .bind(&s.instrument)
        .bind(&s.track)
        .bind(&s.level)
        .bind(s.sort_order)
        .bind(s.schema_version)
        .bind(&s.title)
        .bind(&course.content)
        .bind(&s.unit)
        .bind(&s.unit_title)
        .execute(&self.pool)
        .await
        .context("upsert course")?;
        Ok(())
    }
}

/// In-memory [`CourseRepo`] for tests (no database). Interior-mutable so `upsert`
/// works through the shared `&self` trait. Each entry carries its own status so
/// the published filter is exercised.
#[derive(Default)]
pub struct FakeCourseRepo {
    entries: Mutex<Vec<(String, Course)>>,
}

impl FakeCourseRepo {
    /// A repo seeded with published courses.
    pub fn with(courses: Vec<Course>) -> Self {
        Self {
            entries: Mutex::new(
                courses
                    .into_iter()
                    .map(|c| ("published".to_string(), c))
                    .collect(),
            ),
        }
    }
}

#[async_trait]
impl CourseRepo for FakeCourseRepo {
    async fn list_published(&self) -> Result<Vec<CourseSummary>> {
        let mut out: Vec<CourseSummary> = self
            .entries
            .lock()
            .unwrap()
            .iter()
            .filter(|(status, _)| status == "published")
            .map(|(_, c)| c.summary.clone())
            .collect();
        out.sort_by(|a, b| {
            (
                a.track.as_str(),
                a.level.as_str(),
                a.sort_order,
                a.id.as_str(),
            )
                .cmp(&(
                    b.track.as_str(),
                    b.level.as_str(),
                    b.sort_order,
                    b.id.as_str(),
                ))
        });
        Ok(out)
    }

    async fn get(&self, id: &str) -> Result<Option<Course>> {
        Ok(self
            .entries
            .lock()
            .unwrap()
            .iter()
            .find(|(status, c)| status == "published" && c.summary.id == id)
            .map(|(_, c)| c.clone()))
    }

    async fn upsert(&self, course: &Course, status: &str) -> Result<()> {
        if status.is_empty() {
            bail!("course status must not be empty");
        }
        let mut e = self.entries.lock().unwrap();
        match e
            .iter_mut()
            .find(|(_, c)| c.summary.id == course.summary.id)
        {
            Some(slot) => *slot = (status.to_string(), course.clone()),
            None => e.push((status.to_string(), course.clone())),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn course(id: &str, track: &str, level: &str, order: i32) -> Course {
        Course {
            summary: CourseSummary {
                id: id.into(),
                instrument: "piano".into(),
                track: track.into(),
                level: level.into(),
                sort_order: order,
                schema_version: 1,
                title: format!(r#"{{"en":"{id}","fr":"{id}"}}"#),
                unit: format!("unit-{id}"),
                unit_title: format!(r#"{{"en":"Unit {id}","fr":"Unité {id}"}}"#),
            },
            content: r#"{"schemaVersion":1,"blocks":[]}"#.into(),
        }
    }

    #[tokio::test]
    async fn lists_published_grouped_and_ordered() {
        let repo = FakeCourseRepo::with(vec![
            // Same track/level, out of display order → must come back by sort_order.
            course("s2", "solfege", "beginner", 2),
            course("s1", "solfege", "beginner", 1),
            // A different track groups separately (track name orders the groups).
            course("a1", "app-usage", "beginner", 1),
        ]);
        let ids: Vec<String> = repo
            .list_published()
            .await
            .unwrap()
            .into_iter()
            .map(|s| s.id)
            .collect();
        // Grouped by (track, level) then ordered by sort_order within a group;
        // "app-usage" sorts before "solfege" by track name.
        assert_eq!(ids, vec!["a1", "s1", "s2"]);
    }

    #[tokio::test]
    async fn unit_fields_round_trip_through_upsert_and_listing() {
        let repo = FakeCourseRepo::default();
        repo.upsert(&course("u1", "solfege", "beginner", 1), "published")
            .await
            .unwrap();
        // The unit is a display grouping carried verbatim by both read surfaces —
        // the listing summary and the full course.
        let listed = repo.list_published().await.unwrap();
        assert_eq!(listed[0].unit, "unit-u1");
        assert_eq!(listed[0].unit_title, r#"{"en":"Unit u1","fr":"Unité u1"}"#);
        let fetched = repo.get("u1").await.unwrap().unwrap();
        assert_eq!(fetched.summary.unit, "unit-u1");
        assert_eq!(fetched.summary.unit_title, listed[0].unit_title);
    }

    #[tokio::test]
    async fn get_returns_full_manifest_or_none() {
        let repo = FakeCourseRepo::with(vec![course("a1", "solfege", "beginner", 1)]);
        let c = repo.get("a1").await.unwrap().unwrap();
        assert_eq!(c.summary.schema_version, 1);
        assert!(c.content.contains("schemaVersion"));
        assert!(repo.get("nope").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn upsert_replaces_and_unpublished_is_hidden() {
        let repo = FakeCourseRepo::default();
        repo.upsert(&course("a1", "solfege", "beginner", 1), "published")
            .await
            .unwrap();
        assert_eq!(repo.list_published().await.unwrap().len(), 1);

        // Re-upsert the same id updates in place (idempotent re-seed), not a duplicate.
        repo.upsert(&course("a1", "solfege", "beginner", 5), "published")
            .await
            .unwrap();
        let listed = repo.list_published().await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].sort_order, 5);

        // A draft course is not listed nor fetched.
        repo.upsert(&course("d1", "solfege", "beginner", 1), "draft")
            .await
            .unwrap();
        assert_eq!(repo.list_published().await.unwrap().len(), 1);
        assert!(repo.get("d1").await.unwrap().is_none());
    }
}
