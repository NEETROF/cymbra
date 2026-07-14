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

//! The user-upload business logic (design 2b / 4 / 9).
//!
//! Every upload is: **check client inputs → quota → re-validate & re-derive the
//! bytes (never trusting the client) → store canonical bytes → persist an
//! owner-attributed record**. Descriptive metadata comes only from the server's
//! own parse; the client supplies bytes + level + the rights attestation. Reads,
//! deletes and the quota count are all owner-scoped.

use std::sync::Arc;

use cymbra_musicxml_core::{ScoreSummary, mxl, validate};
use cymbra_platform::{AppError, Result};
use cymbra_storage::ObjectStorage;
use sha2::{Digest, Sha256};

use crate::user_scores::{UserScore, UserScoreRepo};

const LEVELS: [&str; 3] = ["beginner", "intermediate", "advanced"];
const RIGHTS_BASES: [&str; 2] = ["own_work", "public_domain"];

/// The caller-supplied part of an upload (identity comes separately, from auth).
pub struct UploadInput {
    pub data: Vec<u8>,
    pub filename: String,
    pub level: String,
    pub rights_basis: String,
    pub rights_ack: bool,
}

/// User-upload logic over an owner-scoped repo and the object store.
pub struct ScoreModule {
    repo: Arc<dyn UserScoreRepo>,
    storage: Arc<dyn ObjectStorage>,
    quota_max: u32,
    quota_window_days: u32,
    max_bytes: usize,
}

impl ScoreModule {
    pub fn new(
        repo: Arc<dyn UserScoreRepo>,
        storage: Arc<dyn ObjectStorage>,
        quota_max: u32,
        quota_window_days: u32,
        max_bytes: usize,
    ) -> Self {
        Self {
            repo,
            storage,
            quota_max,
            quota_window_days,
            max_bytes,
        }
    }

    /// Validate, re-derive, enforce the quota, store, and persist. Returns the
    /// stored record. `owner_id` is the authenticated caller.
    pub async fn upload(&self, owner_id: &str, input: UploadInput) -> Result<UserScore> {
        // 1. Client inputs the server owns the truth of (design 2b).
        if !input.rights_ack {
            return Err(AppError::InvalidArgument(
                "rights acknowledgement is required".into(),
            ));
        }
        if !LEVELS.contains(&input.level.as_str()) {
            return Err(AppError::InvalidArgument(format!(
                "unknown level {:?}",
                input.level
            )));
        }
        if !RIGHTS_BASES.contains(&input.rights_basis.as_str()) {
            return Err(AppError::InvalidArgument(format!(
                "unknown rights basis {:?}",
                input.rights_basis
            )));
        }
        if input.data.len() > self.max_bytes {
            return Err(AppError::InvalidArgument("file is too large".into()));
        }

        // 2. Quota — before any validation/storage work (design 9).
        let recent = self
            .repo
            .count_recent(owner_id, self.quota_window_days)
            .await?;
        if recent >= self.quota_max as i64 {
            return Err(AppError::ResourceExhausted(format!(
                "upload quota reached ({} per {} days)",
                self.quota_max, self.quota_window_days
            )));
        }

        // 3. Re-validate the received bytes and re-derive metadata (never trust
        //    the client's parse). The summary IS the stored metadata.
        let summary: ScoreSummary = validate(&input.data).map_err(|r| {
            AppError::InvalidArgument(format!("invalid score ({}): {}", r.code(), r))
        })?;

        // 4. Canonical bytes = the decoded MusicXML (so a re-zip of the same piece
        //    dedups); the read path decodes `.mxl` transparently, so we store the
        //    plain XML. sha256 over the canonical form is the per-owner dedup key.
        let canonical = decode_canonical(&input.data)?;
        let sha = sha256_hex(&canonical);

        // 5. Store the object, then persist the row (design 4: object before row;
        //    an orphaned object is invisible/reclaimable, a dangling row is not).
        let id = uuid::Uuid::now_v7().to_string();
        let object_key = format!("user-scores/{owner_id}/{id}.musicxml");
        self.storage
            .put(&object_key, canonical)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("store score: {e}")))?;

        let record = UserScore {
            id,
            owner_id: owner_id.to_string(),
            level: input.level,
            rights_basis: input.rights_basis,
            rights_ack: true,
            title: summary.title,
            composer: summary.composer,
            title_norm: summary.title_norm,
            work_key: summary.work_key,
            key_fifths: summary.key_fifths,
            time_sig: summary.time_sig,
            measure_count: summary.measure_count as i32,
            is_piano: summary.is_piano,
            sha256: sha,
            size_bytes: input.data.len() as i64,
            object_key: object_key.clone(),
            created_at: now_unix(),
        };

        if let Err(e) = self.repo.insert(&record).await {
            // Row write lost the race (e.g. a duplicate): reclaim the object we
            // just wrote so it does not linger as an orphan.
            let _ = self.storage.delete(&object_key).await;
            return Err(e);
        }
        Ok(record)
    }

    /// The caller's own contributed scores, newest first.
    pub async fn list(&self, owner_id: &str) -> Result<Vec<UserScore>> {
        self.repo.list_by_owner(owner_id).await
    }

    /// Delete a score the caller owns: remove the row, then best-effort delete the
    /// object. The row is the source of truth; a failed object delete leaves a
    /// reclaimable orphan (an idempotent cleanup job is wired in task 4.1).
    pub async fn delete(&self, owner_id: &str, id: &str) -> Result<()> {
        let removed = self.repo.delete_owned(id, owner_id).await?;
        let Some(row) = removed else {
            return Err(AppError::NotFound("score not found".into()));
        };
        let _ = self.storage.delete(&row.object_key).await;
        Ok(())
    }

    /// Fetch the bytes of a score the caller owns (for the player).
    pub async fn get_bytes(&self, owner_id: &str, id: &str) -> Result<Vec<u8>> {
        let row = self
            .repo
            .get_owned(id, owner_id)
            .await?
            .ok_or_else(|| AppError::NotFound("score not found".into()))?;
        self.storage
            .get(&row.object_key)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("read score: {e}")))
    }
}

/// The canonical MusicXML bytes: the decoded payload for a `.mxl`, else the input.
fn decode_canonical(data: &[u8]) -> Result<Vec<u8>> {
    if mxl::is_mxl(data) {
        mxl::decode(data)
            .map_err(|e| AppError::InvalidArgument(format!("could not decode .mxl: {e}")))
    } else {
        Ok(data.to_vec())
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_storage::FakeStore;

    use crate::user_scores::FakeUserScoreRepo;

    const VALID: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <work><work-title>Test Piece</work-title></work>
  <identification><creator type="composer">A. Composer</creator></identification>
  <part-list><score-part id="P1"/></part-list>
  <part id="P1"><measure number="1">
    <attributes><divisions>1</divisions>
      <key><fifths>2</fifths></key>
      <time><beats>3</beats><beat-type>4</beat-type></time>
      <staves>2</staves>
    </attributes>
    <note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><staff>1</staff></note>
  </measure></part>
</score-partwise>"#;

    fn module(max: u32, window: u32) -> (ScoreModule, Arc<FakeUserScoreRepo>, Arc<FakeStore>) {
        let repo = Arc::new(FakeUserScoreRepo::default());
        let store = Arc::new(FakeStore::default());
        let m = ScoreModule::new(repo.clone(), store.clone(), max, window, 8 * 1024 * 1024);
        (m, repo, store)
    }

    fn input(data: &str, level: &str, basis: &str, ack: bool) -> UploadInput {
        UploadInput {
            data: data.as_bytes().to_vec(),
            filename: "x.musicxml".into(),
            level: level.into(),
            rights_basis: basis.into(),
            rights_ack: ack,
        }
    }

    #[tokio::test]
    async fn upload_validates_derives_stores_and_persists() {
        let (m, repo, store) = module(5, 7);
        let rec = m
            .upload("u1", input(VALID, "intermediate", "own_work", true))
            .await
            .unwrap();
        // Server-derived metadata (client sent none).
        assert_eq!(rec.title.as_deref(), Some("Test Piece"));
        assert_eq!(rec.composer.as_deref(), Some("A. Composer"));
        assert_eq!(rec.key_fifths, 2);
        assert_eq!(rec.time_sig, "3/4");
        assert!(rec.is_piano);
        assert_eq!(rec.level, "intermediate");
        assert_eq!(
            rec.object_key,
            format!("user-scores/u1/{}.musicxml", rec.id)
        );
        // Object stored + row persisted.
        assert!(store.contains(&rec.object_key));
        assert_eq!(repo.rows().len(), 1);
    }

    #[tokio::test]
    async fn upload_rejects_bad_inputs_without_storing() {
        let (m, repo, store) = module(5, 7);
        // Missing ack, bad level, bad basis, unparseable bytes.
        assert!(matches!(
            m.upload("u1", input(VALID, "intermediate", "own_work", false))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            m.upload("u1", input(VALID, "expert", "own_work", true))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            m.upload("u1", input(VALID, "beginner", "stolen", true))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            m.upload("u1", input("<not-a-score/>", "beginner", "own_work", true))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(store.is_empty());
        assert!(repo.rows().is_empty());
    }

    #[tokio::test]
    async fn upload_enforces_the_quota_before_storing() {
        let (m, _repo, store) = module(2, 7);
        for i in 0..2 {
            // Distinct content so per-owner sha dedup doesn't interfere.
            let xml = VALID.replace("Test Piece", &format!("Piece {i}"));
            m.upload("u1", input(&xml, "beginner", "own_work", true))
                .await
                .unwrap();
        }
        let third = VALID.replace("Test Piece", "Piece 3");
        assert!(matches!(
            m.upload("u1", input(&third, "beginner", "own_work", true))
                .await,
            Err(AppError::ResourceExhausted(_))
        ));
        assert_eq!(store.len(), 2); // the rejected one stored nothing
    }

    #[tokio::test]
    async fn oversized_upload_rejected_before_validation() {
        let (m, _repo, store) = module(5, 7);
        let mut m2 = m;
        m2.max_bytes = 10;
        assert!(matches!(
            m2.upload("u1", input(VALID, "beginner", "own_work", true))
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(store.is_empty());
    }

    #[tokio::test]
    async fn list_delete_and_get_bytes_are_owner_scoped() {
        let (m, _repo, store) = module(5, 7);
        let rec = m
            .upload("u1", input(VALID, "beginner", "own_work", true))
            .await
            .unwrap();
        assert_eq!(m.list("u1").await.unwrap().len(), 1);
        assert!(m.list("u2").await.unwrap().is_empty());
        // Non-owner cannot read or delete.
        assert!(matches!(
            m.get_bytes("u2", &rec.id).await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            m.delete("u2", &rec.id).await,
            Err(AppError::NotFound(_))
        ));
        // Owner reads the canonical bytes back, then deletes (row + object).
        let bytes = m.get_bytes("u1", &rec.id).await.unwrap();
        assert!(!bytes.is_empty());
        m.delete("u1", &rec.id).await.unwrap();
        assert!(m.list("u1").await.unwrap().is_empty());
        assert!(!store.contains(&rec.object_key));
    }

    #[tokio::test]
    async fn duplicate_upload_is_rejected_and_leaves_no_orphan() {
        let (m, _repo, store) = module(5, 7);
        m.upload("u1", input(VALID, "beginner", "own_work", true))
            .await
            .unwrap();
        // Same bytes again → per-owner sha conflict; the just-written object is
        // reclaimed, so exactly one object remains.
        assert!(matches!(
            m.upload("u1", input(VALID, "beginner", "own_work", true))
                .await,
            Err(AppError::AlreadyExists(_))
        ));
        assert_eq!(store.len(), 1);
    }
}
