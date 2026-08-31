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

//! In-app content reports (change: add-content-reporting).
//!
//! Google Play's UGC policy requires an **in-app** way to report objectionable
//! content and users. This file holds the wire vocabulary ([`ReportTarget`],
//! [`ReportReason`]), the pure validation of a submission, and the data-access
//! port with its in-memory fake — so the whole intake is host-testable without a
//! database. The Postgres adapter (coverage-excluded) mirrors the same rules in
//! SQL, including the one-open-report-per-reporter-per-target index.

use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};

/// The longest free-text note accepted, mirroring the column's CHECK. Anything
/// longer is rejected before the write, not truncated: silently dropping the end
/// of a report loses exactly the part a reporter added because it mattered.
pub const MAX_NOTE_LEN: usize = 2000;

/// What is being reported. A new reportable surface is a variant here plus the
/// migration's CHECK — not a new table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportTarget {
    /// A score in the public catalog (`catalog_scores.id`).
    CatalogScore,
    /// A sound font in the public catalog (`soundfonts.id`).
    SoundFont,
    /// A public profile, addressed by its handle.
    Profile,
}

impl ReportTarget {
    /// Parse the wire string; an unknown value is an `InvalidArgument` (rejected
    /// before any write).
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "catalog_score" => Ok(Self::CatalogScore),
            "soundfont" => Ok(Self::SoundFont),
            "profile" => Ok(Self::Profile),
            other => Err(AppError::InvalidArgument(format!(
                "unknown report target {other:?}"
            ))),
        }
    }

    /// The wire string (also the persisted `target_kind` value).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CatalogScore => "catalog_score",
            Self::SoundFont => "soundfont",
            Self::Profile => "profile",
        }
    }
}

/// Why the reporter is reporting. Kept short and mutually exclusive on purpose:
/// a long list makes a reporter hesitate, and the free-text note carries the
/// detail anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportReason {
    /// Infringes someone's rights (the notice half of notice-and-takedown).
    Copyright,
    /// Offensive, hateful, or otherwise unsuitable.
    Inappropriate,
    /// Mislabelled, broken, or not what it claims to be.
    WrongContent,
    /// Anything else — the note carries it.
    Other,
}

impl ReportReason {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "copyright" => Ok(Self::Copyright),
            "inappropriate" => Ok(Self::Inappropriate),
            "wrong_content" => Ok(Self::WrongContent),
            "other" => Ok(Self::Other),
            other => Err(AppError::InvalidArgument(format!(
                "unknown report reason {other:?}"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Copyright => "copyright",
            Self::Inappropriate => "inappropriate",
            Self::WrongContent => "wrong_content",
            Self::Other => "other",
        }
    }
}

/// A validated submission, ready to persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewReport {
    pub target: ReportTarget,
    pub target_id: String,
    pub reason: ReportReason,
    pub note: Option<String>,
}

/// Validate a raw submission: the vocabulary must be known, the target id must
/// not be blank, and the note is trimmed — an all-whitespace note becomes `None`
/// rather than an empty row that reads as "the reporter wrote something".
pub fn validate(
    target_kind: &str,
    target_id: &str,
    reason: &str,
    note: Option<&str>,
) -> Result<NewReport> {
    let target = ReportTarget::parse(target_kind)?;
    let reason = ReportReason::parse(reason)?;
    let id = target_id.trim();
    if id.is_empty() {
        return Err(AppError::InvalidArgument("target_id is required".into()));
    }
    let note = match note.map(str::trim).filter(|n| !n.is_empty()) {
        Some(n) if n.chars().count() > MAX_NOTE_LEN => {
            return Err(AppError::InvalidArgument(format!(
                "note is longer than {MAX_NOTE_LEN} characters"
            )));
        }
        Some(n) => Some(n.to_string()),
        None => None,
    };
    Ok(NewReport {
        target,
        target_id: id.to_string(),
        reason,
        note,
    })
}

/// One report as a moderator reads it back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportRow {
    pub id: String,
    pub target: ReportTarget,
    pub target_id: String,
    /// `None` once the reporter has erased their account.
    pub reporter_id: Option<String>,
    pub reason: ReportReason,
    pub note: Option<String>,
    pub created_at: i64,
}

/// Storage for content reports.
#[async_trait]
pub trait ContentReportRepo: Send + Sync {
    /// Record a report by `reporter_id`, returning its id. Re-reporting the same
    /// target while an earlier report is still open is a **no-op that returns the
    /// existing id**, not an error: the reporter has done their part, and telling
    /// them off for pressing twice teaches them not to report.
    async fn insert(&self, reporter_id: &str, report: &NewReport) -> Result<String>;

    /// The open queue, oldest first, capped at `limit`.
    async fn list_open(&self, limit: i64) -> Result<Vec<ReportRow>>;

    /// How many open reports stand against one target — what a moderation screen
    /// shows next to an item.
    async fn count_open_for(&self, target: ReportTarget, target_id: &str) -> Result<i64>;
}

/// In-memory [`ContentReportRepo`] for unit tests. Enforces the same
/// one-open-report-per-(reporter, target) rule as the partial unique index, so
/// module tests exercise the real de-duplication without a database.
#[derive(Default)]
pub struct FakeContentReportRepo {
    rows: Mutex<Vec<(String, ReportRow)>>,
}

impl FakeContentReportRepo {
    /// Every stored report, in insertion order — for assertions.
    pub fn all(&self) -> Vec<ReportRow> {
        self.rows
            .lock()
            .unwrap()
            .iter()
            .map(|(_, r)| r.clone())
            .collect()
    }
}

#[async_trait]
impl ContentReportRepo for FakeContentReportRepo {
    async fn insert(&self, reporter_id: &str, report: &NewReport) -> Result<String> {
        let mut rows = self.rows.lock().unwrap();
        if let Some((_, existing)) = rows.iter().find(|(rid, r)| {
            rid == reporter_id && r.target == report.target && r.target_id == report.target_id
        }) {
            return Ok(existing.id.clone());
        }
        let seq = rows.len();
        let id = format!("report-{}", seq + 1);
        rows.push((
            reporter_id.to_string(),
            ReportRow {
                id: id.clone(),
                target: report.target,
                target_id: report.target_id.clone(),
                reporter_id: Some(reporter_id.to_string()),
                reason: report.reason,
                note: report.note.clone(),
                created_at: seq as i64,
            },
        ));
        Ok(id)
    }

    async fn list_open(&self, limit: i64) -> Result<Vec<ReportRow>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .map(|(_, r)| r.clone())
            .take(limit.max(0) as usize)
            .collect())
    }

    async fn count_open_for(&self, target: ReportTarget, target_id: &str) -> Result<i64> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, r)| r.target == target && r.target_id == target_id)
            .count() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_round_trips_the_wire_vocabulary() {
        for s in ["catalog_score", "soundfont", "profile"] {
            assert_eq!(ReportTarget::parse(s).unwrap().as_str(), s);
        }
        for s in ["copyright", "inappropriate", "wrong_content", "other"] {
            assert_eq!(ReportReason::parse(s).unwrap().as_str(), s);
        }
    }

    #[test]
    fn rejects_unknown_vocabulary_before_any_write() {
        assert!(matches!(
            ReportTarget::parse("user_score"),
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            ReportReason::parse("spam"),
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[test]
    fn trims_the_note_and_drops_a_blank_one() {
        let r = validate("soundfont", " sf-1 ", "other", Some("   ")).unwrap();
        assert_eq!(r.target_id, "sf-1");
        assert_eq!(r.note, None);

        let r = validate("soundfont", "sf-1", "other", Some("  broken loop  ")).unwrap();
        assert_eq!(r.note.as_deref(), Some("broken loop"));
    }

    #[test]
    fn rejects_a_blank_target_and_an_overlong_note() {
        assert!(matches!(
            validate("catalog_score", "   ", "copyright", None),
            Err(AppError::InvalidArgument(_))
        ));
        let long = "é".repeat(MAX_NOTE_LEN + 1);
        assert!(matches!(
            validate("catalog_score", "s-1", "copyright", Some(&long)),
            Err(AppError::InvalidArgument(_))
        ));
        // The boundary itself is accepted, and counted in CHARACTERS not bytes —
        // an accented note must not be refused for being multi-byte.
        let at_limit = "é".repeat(MAX_NOTE_LEN);
        assert!(validate("catalog_score", "s-1", "copyright", Some(&at_limit)).is_ok());
    }

    #[tokio::test]
    async fn re_reporting_the_same_target_returns_the_first_id() {
        let repo = FakeContentReportRepo::default();
        let r = validate("catalog_score", "s-1", "copyright", None).unwrap();
        let first = repo.insert("u1", &r).await.unwrap();
        let again = repo.insert("u1", &r).await.unwrap();
        assert_eq!(first, again);
        assert_eq!(repo.all().len(), 1);

        // A different reporter is a genuinely new report.
        repo.insert("u2", &r).await.unwrap();
        assert_eq!(
            repo.count_open_for(ReportTarget::CatalogScore, "s-1")
                .await
                .unwrap(),
            2
        );
    }
}
