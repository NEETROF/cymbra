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
//! Writes each retained `.mxl` under `<root>/<confidence-prefix>/<source>/
//! <author>/<title>.mxl`, keeping the safe and low-confidence corpora strictly
//! separate, and emits `manifest.csv` + `manifest.json` (consistent) and
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
        safe_prefix: impl Into<String>,
        low_confidence_prefix: impl Into<String>,
    ) -> Self {
        Self {
            root: root.into(),
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

        std::fs::create_dir_all(&self.root)
            .with_context(|| format!("creating {}", self.root.display()))?;
        std::fs::write(
            self.root.join("manifest.json"),
            manifest::to_json(&entries)?,
        )
        .context("writing manifest.json")?;
        std::fs::write(self.root.join("manifest.csv"), manifest::to_csv(&entries)?)
            .context("writing manifest.csv")?;
        std::fs::write(
            self.root.join("rejected.log"),
            manifest::rejected_log(&outcome.rejected),
        )
        .context("writing rejected.log")?;

        Ok((summary, entries))
    }

    /// The relative object key `<prefix>/<source>/<author>/<title>-<sha8>.mxl`.
    /// A short content-hash suffix keeps same-titled works from colliding.
    fn object_key(&self, entry: &ManifestEntry) -> String {
        let prefix = match entry.confidence {
            Confidence::Verified => &self.safe_prefix,
            Confidence::Unverified => &self.low_confidence_prefix,
        };
        let author = sanitize(entry.composer.as_deref().unwrap_or("unknown"));
        let title = sanitize(entry.title.as_deref().unwrap_or(&entry.source_item_id));
        let sha8 = &entry.sha256[..entry.sha256.len().min(8)];
        format!("{prefix}/{}/{author}/{title}-{sha8}.mxl", entry.source)
    }
}

/// Makes a filesystem-safe path segment: keeps alphanumerics, folds everything
/// else to `_`, collapses runs, and bounds the length.
fn sanitize(s: &str) -> String {
    let mut out = String::new();
    let mut prev_us = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_us = false;
        } else if !prev_us {
            out.push('_');
            prev_us = true;
        }
    }
    let trimmed = out.trim_matches('_');
    let bounded: String = trimmed.chars().take(60).collect();
    if bounded.is_empty() {
        "untitled".to_string()
    } else {
        bounded
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
            id: format!("{source}:x"),
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
                source: "imslp".into(),
                url: "https://imslp/x".into(),
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
        let _ = std::fs::remove_dir_all(&root);
        let w = OutputWriter::new(&root, "safe", "low_confidence");
        let (summary, entries) = w.write(&outcome()).unwrap();

        assert_eq!(summary.safe, 1);
        assert_eq!(summary.low_confidence, 1);
        assert_eq!(summary.rejected, 1);
        // Returned entries carry their object_key for catalog ingestion.
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.object_key.is_some()));

        // Verified under safe/, unverified under low_confidence/ — never mixed.
        let safe = root.join("safe/pdmx/debussy/clair_de_lune-aaaaaaaa.mxl");
        let low = root.join("low_confidence/musetrainer/debussy/clair_de_lune-bbbbbbbb.mxl");
        assert_eq!(std::fs::read(&safe).unwrap(), b"MXL-A");
        assert_eq!(std::fs::read(&low).unwrap(), b"MXL-B");
        assert!(!root.join("safe/musetrainer").exists());

        // Manifests + rejected log written and consistent.
        let json = std::fs::read_to_string(root.join("manifest.json")).unwrap();
        let csv = std::fs::read_to_string(root.join("manifest.csv")).unwrap();
        let log = std::fs::read_to_string(root.join("rejected.log")).unwrap();
        let parsed: Vec<ManifestEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parsed.iter().all(|e| e.object_key.is_some()));
        assert_eq!(csv.lines().count(), 3); // header + 2
        assert!(log.contains("imslp") && log.contains("not redistributable"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn sanitize_folds_unsafe_characters() {
        assert_eq!(sanitize("Clair de Lune!"), "clair_de_lune");
        assert_eq!(sanitize("  Étude/Op.10  "), "tude_op_10");
        assert_eq!(sanitize("***"), "untitled");
    }
}
