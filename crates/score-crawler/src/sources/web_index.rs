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

//! The generic "index → work page → score" web adapter.
//!
//! Many libraries share the same shape: a listing page links to per-work pages;
//! each work page carries a licence marker and one or more score download links.
//! [`WebIndexSource`] captures that shape and reads the licence from the
//! (lightweight) work page before the (heavy) score is fetched, so the pipeline
//! stays license-first. CPDL, IMSLP, Project Gutenberg sheet music, and Hymnary
//! are constructors over this one type; parsing lives in [`super::web`] and is
//! fixture-tested. Sites with a bespoke shape get their own module.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;

use super::git::origin_from_ext;
use super::web;
use super::{Item, RawScore, SourceAdapter};
use crate::convert::OriginFormat;
use crate::http::Fetcher;
use crate::license::RawLicense;

/// A source that crawls a listing page for per-work pages, then the score link
/// and licence on each work page.
pub struct WebIndexSource {
    name: &'static str,
    fetcher: Arc<dyn Fetcher>,
    listing_url: String,
    /// Substring identifying a work-page link among the listing's links.
    work_marker: String,
    /// Score file extensions to accept on a work page (no leading dot).
    score_exts: Vec<String>,
}

impl WebIndexSource {
    pub fn new(
        name: &'static str,
        fetcher: Arc<dyn Fetcher>,
        listing_url: impl Into<String>,
        work_marker: impl Into<String>,
        score_exts: &[&str],
    ) -> Self {
        Self {
            name,
            fetcher,
            listing_url: listing_url.into(),
            work_marker: work_marker.into(),
            score_exts: score_exts.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// CPDL — MediaWiki work pages, per-page licence.
    pub fn cpdl(fetcher: Arc<dyn Fetcher>, listing_url: impl Into<String>) -> Self {
        Self::new("cpdl", fetcher, listing_url, "/wiki/", &["musicxml", "mxl"])
    }

    /// IMSLP — MediaWiki work pages, per-file legal status. Honour robots.txt
    /// (enforced by the HTTP fetcher).
    pub fn imslp(fetcher: Arc<dyn Fetcher>, listing_url: impl Into<String>) -> Self {
        Self::new(
            "imslp",
            fetcher,
            listing_url,
            "/wiki/",
            &["musicxml", "xml", "mxl"],
        )
    }

    /// Project Gutenberg sheet music — Gutenberg public-domain licence.
    pub fn gutenberg(fetcher: Arc<dyn Fetcher>, listing_url: impl Into<String>) -> Self {
        Self::new(
            "gutenberg",
            fetcher,
            listing_url,
            "/ebooks/",
            &["musicxml", "xml", "mxl"],
        )
    }

    /// Hymnary.org — per-item pages with MusicXML.
    pub fn hymnary(fetcher: Arc<dyn Fetcher>, listing_url: impl Into<String>) -> Self {
        Self::new(
            "hymnary",
            fetcher,
            listing_url,
            "/hymn/",
            &["musicxml", "xml"],
        )
    }

    fn exts(&self) -> Vec<&str> {
        self.score_exts.iter().map(|s| s.as_str()).collect()
    }

    async fn score_link(&self, work_url: &str) -> Result<String> {
        let html = self.fetcher.get_text(work_url).await?;
        web::links_with_ext(&html, work_url, &self.exts())?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("no score link on {work_url}"))
    }
}

#[async_trait]
impl SourceAdapter for WebIndexSource {
    fn name(&self) -> &str {
        self.name
    }

    async fn discover(&self) -> Result<Vec<Item>> {
        let html = self
            .fetcher
            .get_text(&self.listing_url)
            .await
            .with_context(|| format!("fetching {} listing", self.name))?;
        let mut seen = HashSet::new();
        let mut items = Vec::new();
        for url in web::links(&html, &self.listing_url)? {
            if url == self.listing_url
                || !url.contains(&self.work_marker)
                || origin_from_ext(Path::new(&url)).is_some()
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

    const SCORE: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<key><fifths>0</fifths></key><time><beats>4</beats><beat-type>4</beat-type></time>
<clef><sign>G</sign><line>2</line></clef></attributes>
<note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note></measure></part></score-partwise>"#;

    // --- CPDL: CC accepted, all-rights-reserved rejected ---

    const CPDL_LISTING: &str = r#"<html><body>
      <a href="https://cpdl.org/wiki/Ave_Verum">Ave Verum</a>
      <a href="https://cpdl.org/wiki/Copyrighted">Copyrighted</a>
      <a href="https://external.example/about">About</a>
    </body></html>"#;
    const CPDL_FREE: &str = r#"<html><body>
      <a href="https://creativecommons.org/licenses/by-sa/4.0/">CC BY-SA 4.0</a>
      <a href="https://cpdl.org/files/ave.musicxml">MusicXML</a></body></html>"#;
    const CPDL_COPYRIGHT: &str = r#"<html><body><p>All Rights Reserved.</p>
      <a href="https://cpdl.org/files/copy.musicxml">MusicXML</a></body></html>"#;

    fn cpdl() -> WebIndexSource {
        let f = MapFetcher::default()
            .with_page("https://cpdl.org/list", CPDL_LISTING)
            .with_page("https://cpdl.org/wiki/Ave_Verum", CPDL_FREE)
            .with_page("https://cpdl.org/wiki/Copyrighted", CPDL_COPYRIGHT)
            .with_blob("https://cpdl.org/files/ave.musicxml", SCORE.as_bytes());
        WebIndexSource::cpdl(Arc::new(f), "https://cpdl.org/list")
    }

    #[tokio::test]
    async fn cpdl_discovers_work_pages_only() {
        let items = cpdl().discover().await.unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.url.contains("/wiki/")));
    }

    #[tokio::test]
    async fn cpdl_accepts_cc_and_rejects_copyright() {
        let out = Orchestrator::new().run(&cpdl(), None).await;
        assert_eq!(out.stats.accepted, 1);
        assert_eq!(out.stats.rejected, 1);
        assert_eq!(out.prepared[0].entry.license, "CC-BY-SA-4.0");
        assert_eq!(out.prepared[0].entry.source, "cpdl");
    }

    // --- Gutenberg: public-domain text accepted ---

    const GUT_LISTING: &str =
        r#"<html><body><a href="https://gutenberg.org/ebooks/123">Sonata</a></body></html>"#;
    const GUT_WORK: &str = r#"<html><body>
      <p>This eBook is in the Public Domain in the United States.</p>
      <a href="https://gutenberg.org/files/123/score.musicxml">MusicXML</a></body></html>"#;

    fn gutenberg() -> WebIndexSource {
        let f = MapFetcher::default()
            .with_page("https://gutenberg.org/list", GUT_LISTING)
            .with_page("https://gutenberg.org/ebooks/123", GUT_WORK)
            .with_blob(
                "https://gutenberg.org/files/123/score.musicxml",
                SCORE.as_bytes(),
            );
        WebIndexSource::gutenberg(Arc::new(f), "https://gutenberg.org/list")
    }

    #[tokio::test]
    async fn gutenberg_accepts_public_domain() {
        let out = Orchestrator::new().run(&gutenberg(), None).await;
        assert_eq!(out.stats.accepted, 1);
        assert_eq!(out.prepared[0].entry.license, "PublicDomain");
        assert_eq!(out.prepared[0].entry.source, "gutenberg");
    }
}
