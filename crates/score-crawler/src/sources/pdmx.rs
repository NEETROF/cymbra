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

//! PDMX (Zenodo dataset) adapter.
//!
//! PDMX ships a metadata index describing each MusicXML record with a per-record
//! licence and subset flag. Only the **`no_license_conflict`** subset is usable;
//! every other record is excluded up front (never discovered, never fetched).
//! The lightweight metadata is fetched once and memoised, so the licence is
//! known before any heavy score download (license-first).

use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::Mutex;

use super::{Item, RawScore, SourceAdapter};
use crate::convert::OriginFormat;
use crate::http::Fetcher;
use crate::license::RawLicense;

/// The usable subset flag; records outside it are rejected wholesale.
const USABLE_SUBSET: &str = "no_license_conflict";

/// One PDMX metadata record.
#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    composer: Option<String>,
    /// URL of the record's MusicXML.
    path: String,
    /// Per-record normalised-ish licence string (fed to the licence engine).
    license: String,
    /// PDMX subset; only `no_license_conflict` is used.
    subset: String,
}

/// The PDMX adapter over any [`Fetcher`].
pub struct PdmxSource {
    fetcher: Arc<dyn Fetcher>,
    metadata_url: String,
    cache: Mutex<Option<Vec<Record>>>,
}

impl PdmxSource {
    pub fn new(fetcher: Arc<dyn Fetcher>, metadata_url: impl Into<String>) -> Self {
        Self {
            fetcher,
            metadata_url: metadata_url.into(),
            cache: Mutex::new(None),
        }
    }

    /// Fetches + memoises the metadata index.
    async fn records(&self) -> Result<Vec<Record>> {
        let mut guard = self.cache.lock().await;
        if guard.is_none() {
            let text = self
                .fetcher
                .get_text(&self.metadata_url)
                .await
                .context("fetching PDMX metadata")?;
            let recs: Vec<Record> = serde_json::from_str(&text).context("parsing PDMX metadata")?;
            *guard = Some(recs);
        }
        Ok(guard.clone().unwrap_or_default())
    }

    /// The usable records (the `no_license_conflict` subset only).
    async fn usable(&self) -> Result<Vec<Record>> {
        Ok(self
            .records()
            .await?
            .into_iter()
            .filter(|r| r.subset == USABLE_SUBSET)
            .collect())
    }

    async fn record(&self, id: &str) -> Result<Record> {
        self.usable()
            .await?
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| anyhow!("PDMX record {id} not in usable subset"))
    }
}

#[async_trait]
impl SourceAdapter for PdmxSource {
    fn name(&self) -> &str {
        "pdmx"
    }

    async fn discover(&self) -> Result<Vec<Item>> {
        Ok(self
            .usable()
            .await?
            .into_iter()
            .map(|r| Item {
                source_item_id: r.id,
                url: r.path,
                title: r.title,
                composer: r.composer,
                arranger: None,
                source_grade: None,
            })
            .collect())
    }

    async fn extract_license(&self, item: &Item) -> Result<RawLicense> {
        // Licence is in the already-fetched metadata — no heavy download.
        let r = self.record(&item.source_item_id).await?;
        Ok(RawLicense::verified(r.license))
    }

    async fn fetch(&self, item: &Item) -> Result<RawScore> {
        let r = self.record(&item.source_item_id).await?;
        let bytes = self.fetcher.get_bytes(&r.path).await?;
        Ok(RawScore {
            origin: OriginFormat::MusicXml,
            bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawl::Orchestrator;
    use crate::http::test_fetcher::MapFetcher;

    const METADATA: &str = r#"[
      {"id":"good","title":"Sonatina","composer":"Clementi","path":"https://zenodo.example/good.musicxml","license":"CC-BY-4.0","subset":"no_license_conflict"},
      {"id":"bad","title":"Mystery","composer":"Unknown","path":"https://zenodo.example/bad.musicxml","license":"All Rights Reserved","subset":"license_conflict"}
    ]"#;

    const SCORE: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<key><fifths>0</fifths></key><time><beats>4</beats><beat-type>4</beat-type></time>
<clef><sign>G</sign><line>2</line></clef></attributes>
<note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note></measure></part></score-partwise>"#;

    fn source() -> PdmxSource {
        let fetcher = MapFetcher::default()
            .with_page("https://zenodo.example/index.json", METADATA)
            .with_blob("https://zenodo.example/good.musicxml", SCORE.as_bytes());
        PdmxSource::new(Arc::new(fetcher), "https://zenodo.example/index.json")
    }

    #[tokio::test]
    async fn discovers_only_the_usable_subset() {
        let items = source().discover().await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].source_item_id, "good");
    }

    #[tokio::test]
    async fn usable_record_flows_to_the_safe_corpus() {
        let out = Orchestrator::new().run(&source(), None).await;
        assert_eq!(out.stats.accepted, 1);
        assert_eq!(out.stats.rejected, 0);
        let e = &out.prepared[0].entry;
        assert_eq!(e.license, "CC-BY-4.0");
        assert_eq!(e.composer.as_deref(), Some("Clementi"));
        assert_eq!(e.source, "pdmx");
    }

    #[tokio::test]
    async fn conflicted_records_are_never_fetched() {
        // The `bad` record is not in the usable subset, so it never appears as an
        // item and its score is never requested (no blob fixture for it exists).
        let out = Orchestrator::new().run(&source(), None).await;
        assert_eq!(out.stats.discovered, 1);
    }
}
