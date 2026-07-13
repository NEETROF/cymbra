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

//! The provenance manifest — the attribution that must travel with every
//! redistributed score.
//!
//! [`ManifestEntry`] is the flat, serde-friendly record (one per retained file)
//! that becomes a `catalog_scores` row once ingestion lands, and is exported to
//! `manifest.csv` / `manifest.json` (consistent between the two). Rejections are
//! journalled separately via [`RejectionRecord`] so exclusions stay auditable.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::convert::{ConversionStatus, OriginFormat};
use crate::difficulty::{Level, LevelSource};
use crate::license::Confidence;

/// One retained score's full provenance + search/musical metadata. Flat so it
/// serialises cleanly to both CSV and JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub id: String,
    pub title: Option<String>,
    pub composer: Option<String>,
    pub arranger: Option<String>,
    pub source: String,
    pub source_url: String,
    pub source_item_id: String,
    /// Normalised licence code, e.g. `CC-BY-SA-4.0`.
    pub license: String,
    pub license_url: Option<String>,
    pub confidence: Confidence,
    /// SHA-256 of the canonical MusicXML — the exact-content dedup key.
    pub sha256: String,
    /// Musical content fingerprint — dedup that survives re-encoding (same notes,
    /// different editor/`divisions`). See [`crate::fingerprint`].
    pub content_fingerprint: String,
    pub origin_format: OriginFormat,
    pub conversion_status: ConversionStatus,
    /// Object-store key; `None` until the item is ingested.
    pub object_key: Option<String>,
    pub size_bytes: u64,
    // --- search / musical metadata ---
    pub work_key: String,
    pub title_norm: Option<String>,
    pub is_piano: bool,
    pub key_fifths: i32,
    pub time_sig: String,
    pub measure_count: u32,
    pub language: Option<String>,
    pub voicing: Option<String>,
    pub level: Option<Level>,
    pub level_source: Option<LevelSource>,
}

/// One excluded item, journalled to `rejected.log`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectionRecord {
    pub source: String,
    pub url: String,
    /// The raw licence signal observed (or a conversion error).
    pub raw_signal: String,
    pub reason: String,
}

impl RejectionRecord {
    /// A single audit line for `rejected.log`.
    pub fn to_log_line(&self) -> String {
        format!(
            "[{}] {} — signal={:?} — {}",
            self.source, self.url, self.raw_signal, self.reason
        )
    }
}

/// Serialises entries to pretty JSON.
pub fn to_json(entries: &[ManifestEntry]) -> Result<String> {
    serde_json::to_string_pretty(entries).context("manifest: serialise json")
}

/// Serialises entries to CSV (header + one row per entry).
pub fn to_csv(entries: &[ManifestEntry]) -> Result<String> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    for e in entries {
        wtr.serialize(e).context("manifest: serialise csv row")?;
    }
    let bytes = wtr.into_inner().context("manifest: flush csv")?;
    String::from_utf8(bytes).context("manifest: csv utf8")
}

/// Joins rejection records into a `rejected.log` body.
pub fn rejected_log(records: &[RejectionRecord]) -> String {
    let mut out = String::new();
    for r in records {
        out.push_str(&r.to_log_line());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str) -> ManifestEntry {
        ManifestEntry {
            id: id.into(),
            title: Some("Clair de Lune".into()),
            composer: Some("Claude Debussy".into()),
            arranger: None,
            source: "openscore".into(),
            source_url: "https://example.org/item/1".into(),
            source_item_id: "1".into(),
            license: "CC0-1.0".into(),
            license_url: Some("https://creativecommons.org/publicdomain/zero/1.0/".into()),
            confidence: Confidence::Verified,
            sha256: "abc123".into(),
            content_fingerprint: "fp123".into(),
            origin_format: OriginFormat::MusicXml,
            conversion_status: ConversionStatus::Converted,
            object_key: None,
            size_bytes: 2048,
            work_key: "claude debussy::clair de lune".into(),
            title_norm: Some("clair de lune".into()),
            is_piano: true,
            key_fifths: -3,
            time_sig: "9/8".into(),
            measure_count: 72,
            language: None,
            voicing: None,
            level: Some(Level::Advanced),
            level_source: Some(LevelSource::Heuristic),
        }
    }

    #[test]
    fn json_and_csv_are_consistent() {
        let entries = vec![entry("a"), entry("b")];
        let json = to_json(&entries).unwrap();
        let csv = to_csv(&entries).unwrap();

        // JSON round-trips to the same entries.
        let back: Vec<ManifestEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, entries);

        // CSV has a header + two data rows, and carries the key fields.
        assert_eq!(csv.lines().count(), 3);
        assert!(csv.contains("CC0-1.0"));
        assert!(csv.contains("advanced"));
        assert!(csv.contains("heuristic"));
    }

    #[test]
    fn confidence_serialises_lowercase() {
        let json = to_json(&[entry("a")]).unwrap();
        assert!(json.contains("\"verified\""));
    }

    #[test]
    fn rejection_log_lines_carry_source_url_and_reason() {
        let r = RejectionRecord {
            source: "imslp".into(),
            url: "https://imslp.org/x".into(),
            raw_signal: "All Rights Reserved".into(),
            reason: "licence AllRightsReserved is not redistributable".into(),
        };
        let line = r.to_log_line();
        assert!(line.contains("imslp"));
        assert!(line.contains("https://imslp.org/x"));
        assert!(line.contains("not redistributable"));
        assert_eq!(rejected_log(&[r]).lines().count(), 1);
    }
}
