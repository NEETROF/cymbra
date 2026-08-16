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

//! Local filesystem output — the dev object-store backend + manifest export.
//!
//! Writes each retained `.mxl` under `<root>/<confidence-prefix>/<shard>/
//! <uuid>.mxl` (keyed by the score's stable UUID v7, not by mutable metadata),
//! keeping the safe and low-confidence corpora strictly separate, and emits
//! `manifest.csv` + `manifest.json` (consistent) and
//! `rejected.log`. This is the `LocalFs` half of ingestion; the Postgres
//! `catalog_scores` row is added with the backend score module (the manifest
//! is the same record either way).

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::crawl::CrawlOutcome;
use crate::license::Confidence;
use crate::manifest::{self, ManifestEntry};

/// Writes a crawl's output to a local corpus root.
pub struct OutputWriter {
    root: PathBuf,
    /// Where the run's own artefacts go — never the corpus root, which holds
    /// servable objects only (change: fix-crawler-corpus-isolation).
    work_dir: PathBuf,
    safe_prefix: String,
    low_confidence_prefix: String,
}

/// Counts written, for reporting.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WriteSummary {
    pub safe: usize,
    pub low_confidence: usize,
    pub rejected: usize,
}

impl OutputWriter {
    pub fn new(
        root: impl Into<PathBuf>,
        work_dir: impl Into<PathBuf>,
        safe_prefix: impl Into<String>,
        low_confidence_prefix: impl Into<String>,
    ) -> Self {
        Self {
            root: root.into(),
            work_dir: work_dir.into(),
            safe_prefix: safe_prefix.into(),
            low_confidence_prefix: low_confidence_prefix.into(),
        }
    }

    /// Writes every prepared score + the manifests + `rejected.log`. Sets each
    /// entry's `object_key` to its relative path before exporting the manifest,
    /// and returns those entries so they can also be ingested into the catalog.
    pub fn write(&self, outcome: &CrawlOutcome) -> Result<(WriteSummary, Vec<ManifestEntry>)> {
        let mut summary = WriteSummary {
            rejected: outcome.rejected.len(),
            ..Default::default()
        };
        let mut entries: Vec<ManifestEntry> = Vec::with_capacity(outcome.prepared.len());

        for prep in &outcome.prepared {
            let key = self.object_key(&prep.entry);
            let path = self.root.join(&key);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            std::fs::write(&path, &prep.mxl)
                .with_context(|| format!("writing {}", path.display()))?;
            match prep.entry.confidence {
                Confidence::Verified => summary.safe += 1,
                Confidence::Unverified => summary.low_confidence += 1,
            }
            let mut entry = prep.entry.clone();
            entry.object_key = Some(key);
            entries.push(entry);
        }

        // Run artefacts are audit output, not servable objects: they go to the
        // work dir so the corpus root keeps only `<prefix>/<shard>/<name>.mxl`
        // and the S3 mirror never carries them.
        std::fs::create_dir_all(&self.work_dir)
            .with_context(|| format!("creating {}", self.work_dir.display()))?;
        std::fs::write(
            self.work_dir.join("manifest.json"),
            manifest::to_json(&entries)?,
        )
        .context("writing manifest.json")?;
        std::fs::write(
            self.work_dir.join("manifest.csv"),
            manifest::to_csv(&entries)?,
        )
        .context("writing manifest.csv")?;
        std::fs::write(
            self.work_dir.join("rejected.log"),
            manifest::rejected_log(&outcome.rejected),
        )
        .context("writing rejected.log")?;

        Ok((summary, entries))
    }

    /// The relative object key `<prefix>/<shard>/<sha256>.mxl`.
    ///
    /// Keyed by the score's **content hash**, not by its row id: the id is a
    /// fresh UUID v7 per run, so keying on it made every re-crawl of unchanged
    /// content write a *second* object — half the production corpus ended up
    /// unreferenced that way (change: fix-crawler-corpus-isolation). A
    /// content-derived key makes the write idempotent with no catalog lookup:
    /// the same bytes always resolve to the same key and simply overwrite.
    ///
    /// Still immutable and metadata-free, so an object never needs re-keying when
    /// title/author change and the key leaks nothing. `<shard>` = the hash's last
    /// two hex chars (uniformly distributed), keeping directories small for a
    /// 100k+ corpus. Readers resolve bytes through the row's stored `object_key`,
    /// never by rebuilding it, so this stays decoupled from the catalog PK.
    fn object_key(&self, entry: &ManifestEntry) -> String {
        let prefix = match entry.confidence {
            Confidence::Verified => &self.safe_prefix,
            Confidence::Unverified => &self.low_confidence_prefix,
        };
        let name = &entry.sha256;
        let shard = &name[name.len().saturating_sub(2)..];
        format!("{prefix}/{shard}/{name}.mxl")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::{ConversionStatus, OriginFormat};
    use crate::crawl::{CrawlStats, PreparedScore};
    use crate::difficulty::{Level, LevelSource};
    use crate::manifest::RejectionRecord;

    fn entry(source: &str, confidence: Confidence, sha: &str) -> ManifestEntry {
        ManifestEntry {
            id: uuid::Uuid::now_v7().to_string(),
            title: Some("Clair de Lune".into()),
            composer: Some("Debussy".into()),
            arranger: None,
            source: source.into(),
            source_url: "https://ex/x".into(),
            source_item_id: "x".into(),
            license: "CC0-1.0".into(),
            license_url: None,
            confidence,
            sha256: sha.into(),
            content_fingerprint: format!("fp-{sha}"),
            origin_format: OriginFormat::MusicXml,
            conversion_status: ConversionStatus::Converted,
            object_key: None,
            size_bytes: 3,
            work_key: "debussy::clair de lune".into(),
            title_norm: Some("clair de lune".into()),
            is_piano: true,
            key_fifths: 0,
            time_sig: "4/4".into(),
            measure_count: 1,
            facets: cymbra_musicxml_core::ScoreFacets::default(),
            language: None,
            voicing: None,
            level: Some(Level::Advanced),
            level_source: Some(LevelSource::Heuristic),
        }
    }

    fn outcome() -> CrawlOutcome {
        CrawlOutcome {
            prepared: vec![
                PreparedScore {
                    entry: entry("pdmx", Confidence::Verified, "aaaaaaaa1111"),
                    mxl: b"MXL-A".to_vec(),
                },
                PreparedScore {
                    entry: entry("musetrainer", Confidence::Unverified, "bbbbbbbb2222"),
                    mxl: b"MXL-B".to_vec(),
                },
            ],
            rejected: vec![RejectionRecord {
                source: "pdmx".into(),
                url: "https://pdmx/x".into(),
                raw_signal: "All Rights Reserved".into(),
                reason: "not redistributable".into(),
            }],
            stats: CrawlStats::default(),
        }
    }

    fn tmp(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("score_crawler_out_{tag}"))
    }

    #[test]
    fn writes_confidence_separated_corpus_and_manifests() {
        let root = tmp("sep_unique_xyz");
        let work = tmp("sep_unique_xyz_work");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&work);
        let w = OutputWriter::new(&root, &work, "safe", "low_confidence");
        let (summary, entries) = w.write(&outcome()).unwrap();

        assert_eq!(summary.safe, 1);
        assert_eq!(summary.low_confidence, 1);
        assert_eq!(summary.rejected, 1);
        // Returned entries carry their object_key for catalog ingestion.
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.object_key.is_some()));

        // Key = `<prefix>/<shard>/<sha256>.mxl`, resolved straight from object_key.
        // Verified under safe/, unverified under low_confidence/ — never mixed.
        let safe = &entries[0];
        let low = &entries[1];
        let safe_key = safe.object_key.as_ref().unwrap();
        let low_key = low.object_key.as_ref().unwrap();
        assert_eq!(safe_key, "safe/11/aaaaaaaa1111.mxl");
        assert_eq!(low_key, "low_confidence/22/bbbbbbbb2222.mxl");
        assert_eq!(std::fs::read(root.join(safe_key)).unwrap(), b"MXL-A");
        assert_eq!(std::fs::read(root.join(low_key)).unwrap(), b"MXL-B");
        assert!(!root.join("safe/musetrainer").exists());

        // Manifests + rejected log written and consistent — in the WORK dir.
        let json = std::fs::read_to_string(work.join("manifest.json")).unwrap();
        let csv = std::fs::read_to_string(work.join("manifest.csv")).unwrap();
        let log = std::fs::read_to_string(work.join("rejected.log")).unwrap();
        let parsed: Vec<ManifestEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parsed.iter().all(|e| e.object_key.is_some()));
        assert_eq!(csv.lines().count(), 3); // header + 2
        assert!(log.contains("pdmx") && log.contains("not redistributable"));

        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&work);
    }

    /// Task 2.2 — the corpus root holds servable objects and nothing else; the
    /// run artefacts are found at the work location.
    #[test]
    fn corpus_root_holds_only_servable_objects() {
        let root = tmp("only_objects_root");
        let work = tmp("only_objects_work");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&work);
        OutputWriter::new(&root, &work, "safe", "low_confidence")
            .write(&outcome())
            .unwrap();

        // Top level of the corpus: only the two confidence prefixes.
        let mut top: Vec<String> = std::fs::read_dir(&root)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        top.sort();
        assert_eq!(top, vec!["low_confidence", "safe"]);

        // None of the artefacts leaked into the corpus.
        for stray in ["manifest.json", "manifest.csv", "rejected.log"] {
            assert!(
                !root.join(stray).exists(),
                "{stray} must not be written to the corpus root"
            );
            assert!(work.join(stray).exists(), "{stray} must be in the work dir");
        }

        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&work);
    }

    /// Task 3.3 — the same content written twice yields ONE object under a stable
    /// key. This is the regression that let production accumulate ~145k
    /// unreferenced objects: the key used to carry a per-run UUID.
    #[test]
    fn rewriting_identical_content_does_not_add_an_object() {
        let root = tmp("idempotent_root");
        let work = tmp("idempotent_work");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&work);
        let w = OutputWriter::new(&root, &work, "safe", "low_confidence");

        let (_, first) = w.write(&outcome()).unwrap();
        let count_after_first = mxl_count(&root);
        // A second run over the same content — fresh entries, so fresh row ids.
        let (_, second) = w.write(&outcome()).unwrap();

        assert_eq!(
            mxl_count(&root),
            count_after_first,
            "a re-crawl of unchanged content must not add corpus objects"
        );
        assert_eq!(count_after_first, 2);
        // Same key both times, despite different row ids.
        assert_eq!(first[0].object_key, second[0].object_key);
        assert_ne!(first[0].id, second[0].id);

        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&work);
    }

    /// Task 3.3 — different content gets a different key, and the key does not
    /// carry the catalog row id (the two are deliberately decoupled).
    #[test]
    fn key_is_content_derived_and_free_of_the_row_id() {
        let root = tmp("key_shape_root");
        let work = tmp("key_shape_work");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&work);
        let (_, entries) = OutputWriter::new(&root, &work, "safe", "low_confidence")
            .write(&outcome())
            .unwrap();

        let a = entries[0].object_key.as_ref().unwrap();
        let b = entries[1].object_key.as_ref().unwrap();
        assert_ne!(a, b, "different content must not share a key");
        for e in &entries {
            let key = e.object_key.as_ref().unwrap();
            assert!(key.contains(&e.sha256), "key must carry the content hash");
            assert!(!key.contains(&e.id), "key must not carry the row id");
        }

        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&work);
    }

    /// Counts `.mxl` files under a corpus root, recursively.
    fn mxl_count(root: &std::path::Path) -> usize {
        fn walk(dir: &std::path::Path, n: &mut usize) {
            let Ok(rd) = std::fs::read_dir(dir) else {
                return;
            };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, n);
                } else if p.extension().is_some_and(|x| x == "mxl") {
                    *n += 1;
                }
            }
        }
        let mut n = 0;
        walk(root, &mut n);
        n
    }
}
