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

//! Ingestion into the shared `score` catalog.
//!
//! Maps a retained [`ManifestEntry`] (bytes already written to the object store,
//! `object_key` set) to a backend [`CatalogEntry`] and inserts it through the
//! [`CatalogRepo`] — idempotently, so re-ingesting existing content is a no-op.
//! The DB row is the source of truth; the manifest is the same record exported.

use anyhow::Result;
use cymbra_score::{CatalogEntry, CatalogRepo};
use serde::Serialize;

use crate::manifest::ManifestEntry;

/// Serialises a `#[serde(rename_all = "snake_case")]` enum to its string form,
/// matching the `catalog_scores` CHECK vocabulary.
fn variant<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|j| j.as_str().map(String::from))
        .unwrap_or_default()
}

/// Maps a manifest entry to a backend catalog row (fresh UUID v7 id).
pub fn to_catalog_entry(e: &ManifestEntry) -> CatalogEntry {
    CatalogEntry {
        id: uuid::Uuid::now_v7().to_string(),
        title: e.title.clone(),
        composer: e.composer.clone(),
        arranger: e.arranger.clone(),
        source: e.source.clone(),
        source_url: e.source_url.clone(),
        source_item_id: e.source_item_id.clone(),
        license: e.license.clone(),
        license_url: e.license_url.clone(),
        confidence: variant(&e.confidence),
        sha256: e.sha256.clone(),
        origin_format: variant(&e.origin_format),
        conversion_status: variant(&e.conversion_status),
        object_key: e.object_key.clone().unwrap_or_default(),
        size_bytes: e.size_bytes as i64,
        work_key: e.work_key.clone(),
        title_norm: e.title_norm.clone(),
        is_piano: e.is_piano,
        key_fifths: e.key_fifths,
        time_sig: e.time_sig.clone(),
        measure_count: e.measure_count as i32,
        language: e.language.clone(),
        voicing: e.voicing.clone(),
        level: e.level.as_ref().map(variant),
        level_source: e.level_source.as_ref().map(variant),
    }
}

/// Inserts every entry into the catalog, returning the count of new rows
/// (duplicates by `sha256` are skipped).
pub async fn ingest(repo: &dyn CatalogRepo, entries: &[ManifestEntry]) -> Result<usize> {
    let mut inserted = 0;
    for e in entries {
        if repo.insert(&to_catalog_entry(e)).await? {
            inserted += 1;
        }
    }
    Ok(inserted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::{ConversionStatus, OriginFormat};
    use crate::difficulty::{Level, LevelSource};
    use crate::license::Confidence;
    use cymbra_score::FakeCatalogRepo;

    fn entry(sha: &str) -> ManifestEntry {
        ManifestEntry {
            id: "cpdl:x".into(),
            title: Some("Ave Verum".into()),
            composer: Some("Mozart".into()),
            arranger: None,
            source: "cpdl".into(),
            source_url: "https://cpdl/x".into(),
            source_item_id: "x".into(),
            license: "CC-BY-SA-4.0".into(),
            license_url: Some("https://cc/by-sa/4.0".into()),
            confidence: Confidence::Verified,
            sha256: sha.into(),
            origin_format: OriginFormat::MusicXml,
            conversion_status: ConversionStatus::Converted,
            object_key: Some("safe/cpdl/mozart/ave_verum-abcd1234.mxl".into()),
            size_bytes: 1234,
            work_key: "mozart::ave verum".into(),
            title_norm: Some("ave verum".into()),
            is_piano: true,
            key_fifths: 1,
            time_sig: "4/4".into(),
            measure_count: 46,
            language: Some("la".into()),
            voicing: Some("SATB".into()),
            level: Some(Level::Intermediate),
            level_source: Some(LevelSource::Heuristic),
        }
    }

    #[test]
    fn maps_enum_fields_to_check_vocabulary() {
        let c = to_catalog_entry(&entry("aaa"));
        assert_eq!(c.confidence, "verified");
        assert_eq!(c.origin_format, "music_xml");
        assert_eq!(c.conversion_status, "converted");
        assert_eq!(c.level.as_deref(), Some("intermediate"));
        assert_eq!(c.level_source.as_deref(), Some("heuristic"));
        assert_eq!(c.object_key, "safe/cpdl/mozart/ave_verum-abcd1234.mxl");
        assert_eq!(c.size_bytes, 1234);
        // A fresh UUID v7 id was assigned.
        assert!(uuid::Uuid::parse_str(&c.id).is_ok());
    }

    #[tokio::test]
    async fn ingest_inserts_and_dedups() {
        let repo = FakeCatalogRepo::default();
        let entries = vec![entry("aaa"), entry("bbb"), entry("aaa")];
        let n = ingest(&repo, &entries).await.unwrap();
        assert_eq!(n, 2); // the third (dup sha) is skipped
        assert_eq!(repo.rows().len(), 2);
    }
}
