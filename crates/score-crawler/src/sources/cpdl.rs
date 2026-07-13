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

//! CPDL (Choral Public Domain Library) web-crawl adapter.
//!
//! A listing page links to per-work pages; each work page carries a licence
//! marker and one or more MusicXML download links. The licence is read from the
//! (lightweight) work page before the (heavy) score is fetched, so the pipeline
//! stays license-first. HTML parsing lives in [`super::web`] and is fixture
//! tested; only the GETs go through the injected [`Fetcher`].

use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;

use super::web;
use super::{Item, RawScore, SourceAdapter};
use crate::convert::OriginFormat;
use crate::http::Fetcher;
use crate::license::RawLicense;

/// CPDL adapter over any [`Fetcher`].
pub struct CpdlSource {
    fetcher: Arc<dyn Fetcher>,
    listing_url: String,
    /// Substring identifying a work page among the listing's links.
    work_marker: String,
}

impl CpdlSource {
    pub fn new(fetcher: Arc<dyn Fetcher>, listing_url: impl Into<String>) -> Self {
        Self {
            fetcher,
            listing_url: listing_url.into(),
            work_marker: "/wiki/".to_string(),
        }
    }

    /// The first MusicXML/`.mxl` download link on a work page.
    async fn score_link(&self, work_url: &str) -> Result<String> {
        let html = self.fetcher.get_text(work_url).await?;
        web::links_with_ext(&html, work_url, &["musicxml", "mxl"])?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("no MusicXML link on {work_url}"))
    }
}

#[async_trait]
impl SourceAdapter for CpdlSource {
    fn name(&self) -> &str {
        "cpdl"
    }

    async fn discover(&self) -> Result<Vec<Item>> {
        let html = self
            .fetcher
            .get_text(&self.listing_url)
            .await
            .context("fetching CPDL listing")?;
        let mut seen = std::collections::HashSet::new();
        let mut items = Vec::new();
        for url in web::links(&html, &self.listing_url)? {
            if url == self.listing_url
                || !url.contains(&self.work_marker)
                || super::git::origin_from_ext(std::path::Path::new(&url)).is_some()
            {
                continue; // skip self, non-work links, and direct file links
            }
            if seen.insert(url.clone()) {
                let title = url.rsplit('/').next().map(|s| s.replace(['_', '-'], " "));
                items.push(Item {
                    source_item_id: url.clone(),
                    url,
                    title,
                    composer: None,
                    arranger: None,
                    source_grade: None,
                });
            }
        }
        Ok(items)
    }

    async fn extract_license(&self, item: &Item) -> Result<RawLicense> {
        // Read the licence off the lightweight work page (not the score).
        let html = self.fetcher.get_text(&item.url).await?;
        let signal = web::detect_license(&html)
            .ok_or_else(|| anyhow!("no licence marker on {}", item.url))?;
        Ok(RawLicense::verified(signal))
    }

    async fn fetch(&self, item: &Item) -> Result<RawScore> {
        let link = self.score_link(&item.url).await?;
        let bytes = self.fetcher.get_bytes(&link).await?;
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

    const LISTING: &str = r#"<html><body>
      <a href="https://cpdl.org/wiki/Ave_Verum">Ave Verum</a>
      <a href="https://cpdl.org/wiki/Copyrighted_Work">Copyrighted</a>
      <a href="https://external.example/about">About</a>
    </body></html>"#;

    const FREE_WORK: &str = r#"<html><body>
      <p>Edition licensed under
        <a href="https://creativecommons.org/licenses/by-sa/4.0/">CC BY-SA 4.0</a>.</p>
      <a href="https://cpdl.org/files/ave_verum.musicxml">Download MusicXML</a>
    </body></html>"#;

    const COPYRIGHTED_WORK: &str = r#"<html><body>
      <p>All Rights Reserved by the arranger.</p>
      <a href="https://cpdl.org/files/copyrighted.musicxml">Download MusicXML</a>
    </body></html>"#;

    const SCORE: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<key><fifths>0</fifths></key><time><beats>4</beats><beat-type>4</beat-type></time>
<clef><sign>G</sign><line>2</line></clef></attributes>
<note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note></measure></part></score-partwise>"#;

    fn source() -> CpdlSource {
        let fetcher = MapFetcher::default()
            .with_page("https://cpdl.org/list", LISTING)
            .with_page("https://cpdl.org/wiki/Ave_Verum", FREE_WORK)
            .with_page("https://cpdl.org/wiki/Copyrighted_Work", COPYRIGHTED_WORK)
            .with_blob(
                "https://cpdl.org/files/ave_verum.musicxml",
                SCORE.as_bytes(),
            );
        CpdlSource::new(Arc::new(fetcher), "https://cpdl.org/list")
    }

    #[tokio::test]
    async fn discovers_work_pages_only() {
        let items = source().discover().await.unwrap();
        // Two wiki work links; the external "About" link is excluded.
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.url.contains("/wiki/")));
    }

    #[tokio::test]
    async fn accepts_the_cc_work_and_rejects_the_copyrighted_one() {
        let out = Orchestrator::new().run(&source(), None).await;
        assert_eq!(out.stats.accepted, 1);
        assert_eq!(out.stats.rejected, 1);
        assert_eq!(out.prepared[0].entry.license, "CC-BY-SA-4.0");
        assert!(
            out.rejected
                .iter()
                .any(|r| r.reason.contains("not redistributable"))
        );
    }
}
