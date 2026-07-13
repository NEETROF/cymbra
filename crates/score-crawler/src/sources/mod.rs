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

//! The `SourceAdapter` contract every source implements.
//!
//! The orchestrator ([`crate::crawl`]) drives adapters through a fixed pipeline
//! — `discover → extract_license → [gate] → fetch → convert` — so licence is
//! always decided before any heavy download. Concrete adapters are git-clone
//! ([`git`], [`mutopia`]) and bulk-dataset ([`pdmx`]) families; this module
//! defines the trait, its data types, and a test fake.

use anyhow::Result;
use async_trait::async_trait;

use crate::convert::{Converted, OriginFormat, convert_any};
use crate::difficulty::Level;
use crate::license::RawLicense;

pub mod git;
pub mod mutopia;
pub mod pdmx;

/// Every source this crawler harvests; `--all` expands to this list. All are
/// implemented (see [`crate::registry`]).
///
/// Sources that were evaluated and **dropped** (their adapters removed) are
/// recorded in [`EXCLUDED_SOURCES`] with the reason, so scope decisions stay
/// discoverable in code rather than only in history.
pub const ALL_SOURCES: &[&str] = &[
    "openscore",
    "mutopia",
    "pdmx",
    "musetrainer",
    "eduardomourar",
];

/// Sources evaluated during the crawler's bring-up and **excluded** — their
/// adapters (and the generic web-crawl/HTTP-politeness layer that only they
/// used: `http`, `robots`, `politeness`, `sources::web*`) were removed rather
/// than shipped broken. Restore from git history if a source becomes viable.
///
/// | source | why excluded |
/// |--------|--------------|
/// | `cpdl` (ChoralWiki) | automated access returns HTTP 403; scores are mostly PDF/MuseScore, MusicXML is rare. |
/// | `imslp` | scores sit behind a paywall/wait and are PDF scans; almost no MusicXML. |
/// | `gutenberg` | the "sheet music" holdings are text ebooks (GUTINDEX); no MusicXML is served. |
/// | `hymnary` | scores are gated, paid "FlexScores"; no bulk/free MusicXML and no work index to crawl. |
/// | `neuma` (`neuma.huma-num.fr`) | the site no longer exists. |
/// | Josquin Research Project | the site could not be located. |
pub const EXCLUDED_SOURCES: &[(&str, &str)] = &[
    (
        "cpdl",
        "HTTP 403 on automated access; MusicXML rare (mostly PDF/MuseScore)",
    ),
    ("imslp", "paywalled PDF scans; almost no MusicXML"),
    ("gutenberg", "text ebooks only; no MusicXML served"),
    ("hymnary", "gated paid FlexScores; no free bulk MusicXML"),
    ("neuma", "site no longer exists"),
    ("josquin", "site could not be located"),
];

/// A discovered candidate, known before licence evaluation or heavy download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Stable id at the source (for resume + `source_item_id`).
    pub source_item_id: String,
    pub url: String,
    pub title: Option<String>,
    pub composer: Option<String>,
    pub arranger: Option<String>,
    /// A difficulty grade the source itself declares, if any (authoritative).
    pub source_grade: Option<Level>,
}

/// The raw payload fetched for an accepted item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawScore {
    pub origin: OriginFormat,
    pub bytes: Vec<u8>,
}

/// A per-source adapter. `extract_license` MUST be cheap (no heavy download) so
/// the orchestrator can gate before `fetch`.
#[async_trait]
pub trait SourceAdapter: Send + Sync {
    /// Stable source name (e.g. `openscore`), used in provenance + paths.
    fn name(&self) -> &str;

    /// One-time preparation before discovery (e.g. a git clone/pull). The
    /// default is a no-op; web adapters need nothing here.
    async fn prepare(&self) -> Result<()> {
        Ok(())
    }

    /// Enumerate candidate items.
    async fn discover(&self) -> Result<Vec<Item>>;

    /// Obtain the raw licence signal for an item without downloading the score.
    async fn extract_license(&self, item: &Item) -> Result<RawLicense>;

    /// Download the raw score payload (only called for whitelisted licences).
    async fn fetch(&self, item: &Item) -> Result<RawScore>;

    /// Convert a fetched payload to a validated `.mxl`. The default routes
    /// through the central [`convert_any`]; adapters override only for bespoke
    /// handling.
    async fn to_musicxml(&self, raw: RawScore) -> Result<Converted> {
        convert_any(raw.origin, &raw.bytes)
    }
}

#[cfg(test)]
pub(crate) mod test_fake {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// A scripted adapter for orchestrator tests. Each item carries the licence
    /// signal to return and the bytes to yield; `fetch_calls` counts downloads
    /// so tests can assert the licence-first gate skipped rejected items.
    pub struct FakeAdapter {
        pub name: String,
        pub items: Vec<(Item, RawLicense, RawScore)>,
        pub fetch_calls: Arc<AtomicUsize>,
        /// Item ids whose `fetch` should fail (error isolation test).
        pub fail_fetch: Vec<String>,
    }

    #[async_trait]
    impl SourceAdapter for FakeAdapter {
        fn name(&self) -> &str {
            &self.name
        }

        async fn discover(&self) -> Result<Vec<Item>> {
            Ok(self.items.iter().map(|(i, _, _)| i.clone()).collect())
        }

        async fn extract_license(&self, item: &Item) -> Result<RawLicense> {
            self.items
                .iter()
                .find(|(i, _, _)| i.source_item_id == item.source_item_id)
                .map(|(_, l, _)| l.clone())
                .ok_or_else(|| anyhow::anyhow!("unknown item"))
        }

        async fn fetch(&self, item: &Item) -> Result<RawScore> {
            self.fetch_calls.fetch_add(1, Ordering::SeqCst);
            if self.fail_fetch.contains(&item.source_item_id) {
                return Err(anyhow::anyhow!("simulated fetch failure"));
            }
            self.items
                .iter()
                .find(|(i, _, _)| i.source_item_id == item.source_item_id)
                .map(|(_, _, s)| s.clone())
                .ok_or_else(|| anyhow::anyhow!("unknown item"))
        }
    }
}
