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

//! PDMX (Public Domain MusicXML) dataset adapter.
//!
//! PDMX is a bulk Zenodo dataset, not a per-item API: a `PDMX.csv` index (with a
//! `subset:no_license_conflict` flag) plus a `mxl.tar.gz` archive of the scores.
//! `prepare` downloads and extracts both; only the **`no_license_conflict`**
//! subset (≈222k of ~250k) is discovered — the rest is excluded up front. Each
//! score is already MusicXML, so no conversion is needed.
//!
//! NOTE: the exact CSV column names for the score path/title/composer are
//! configurable and default to reasonable guesses — confirm them against the
//! real `PDMX.csv` before a production run. The full 1.9 GB download/extract is
//! network/IO glue and is not unit-tested; the CSV subset filtering and the
//! extracted-file read are.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use super::{Item, RawScore, SourceAdapter};
use crate::convert::OriginFormat;
use crate::license::RawLicense;

/// Default Zenodo file base for PDMX record 15571083.
pub const PDMX_BASE: &str = "https://zenodo.org/records/15571083/files";
const CSV_FILE: &str = "PDMX.csv";
const MXL_ARCHIVE: &str = "mxl.tar.gz";
/// The subset flag column — the usable (`no_license_conflict`) rows.
const SUBSET_COL: &str = "subset:no_license_conflict";

/// PDMX adapter over a local cache directory.
pub struct PdmxDatasetSource {
    base_url: String,
    cache: PathBuf,
    /// CSV column giving the `.mxl` path within the extracted archive.
    path_col: String,
    title_col: String,
    composer_col: String,
}

/// One usable CSV row.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    path: String,
    title: Option<String>,
    composer: Option<String>,
}

impl PdmxDatasetSource {
    pub fn new(cache: impl Into<PathBuf>) -> Self {
        Self {
            base_url: PDMX_BASE.to_string(),
            cache: cache.into(),
            path_col: "mxl".to_string(),
            title_col: "title".to_string(),
            composer_col: "composer".to_string(),
        }
    }

    fn csv_path(&self) -> PathBuf {
        self.cache.join(CSV_FILE)
    }
    fn mxl_dir(&self) -> PathBuf {
        self.cache.join("mxl")
    }
}

#[async_trait]
impl SourceAdapter for PdmxDatasetSource {
    fn name(&self) -> &str {
        "pdmx"
    }

    async fn prepare(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.cache).await.ok();
        if !self.csv_path().exists() {
            download_to_file(&format!("{}/{CSV_FILE}", self.base_url), &self.csv_path()).await?;
        }
        if !self.mxl_dir().exists() {
            let archive = self.cache.join(MXL_ARCHIVE);
            if !archive.exists() {
                download_to_file(&format!("{}/{MXL_ARCHIVE}", self.base_url), &archive).await?;
            }
            let (a, d) = (archive.clone(), self.mxl_dir());
            tokio::task::spawn_blocking(move || extract_tar_gz(&a, &d))
                .await
                .context("joining extract task")??;
        }
        Ok(())
    }

    async fn discover(&self) -> Result<Vec<Item>> {
        let rows = parse_usable(
            &self.csv_path(),
            &self.path_col,
            &self.title_col,
            &self.composer_col,
        )?;
        Ok(rows
            .into_iter()
            .map(|r| Item {
                source_item_id: r.path.clone(),
                url: format!("{}/{MXL_ARCHIVE}#{}", self.base_url, r.path),
                title: r.title,
                composer: r.composer,
                arranger: None,
                source_grade: None,
            })
            .collect())
    }

    async fn extract_license(&self, _item: &Item) -> Result<RawLicense> {
        // The `no_license_conflict` subset is PDMX's public-domain guarantee.
        Ok(RawLicense::verified("Public Domain"))
    }

    async fn fetch(&self, item: &Item) -> Result<RawScore> {
        let path = self.mxl_dir().join(&item.source_item_id);
        let bytes = std::fs::read(&path)
            .with_context(|| format!("reading extracted score {}", path.display()))?;
        Ok(RawScore {
            origin: OriginFormat::Mxl,
            bytes,
        })
    }
}

/// Parses `PDMX.csv`, returning the rows in the `no_license_conflict` subset with
/// a non-empty score path. Pure; testable offline.
fn parse_usable(
    csv_path: &Path,
    path_col: &str,
    title_col: &str,
    composer_col: &str,
) -> Result<Vec<Row>> {
    let mut rdr = csv::Reader::from_path(csv_path)
        .with_context(|| format!("opening {}", csv_path.display()))?;
    let headers = rdr.headers().context("reading PDMX.csv headers")?.clone();
    let index = |name: &str| headers.iter().position(|h| h == name);

    let subset_i =
        index(SUBSET_COL).ok_or_else(|| anyhow!("PDMX.csv has no '{SUBSET_COL}' column"))?;
    let path_i = index(path_col).ok_or_else(|| anyhow!("PDMX.csv has no '{path_col}' column"))?;
    let title_i = index(title_col);
    let composer_i = index(composer_col);

    let mut out = Vec::new();
    for record in rdr.records() {
        let record = record.context("reading PDMX.csv row")?;
        if !matches!(record.get(subset_i), Some("True" | "true" | "1")) {
            continue; // not in the usable subset
        }
        let path = match record.get(path_i) {
            Some(p) if !p.is_empty() => p.to_string(),
            _ => continue,
        };
        let field = |i: Option<usize>| {
            i.and_then(|i| record.get(i))
                .filter(|s| !s.is_empty())
                .map(String::from)
        };
        out.push(Row {
            path,
            title: field(title_i),
            composer: field(composer_i),
        });
    }
    Ok(out)
}

/// Streams a URL to a file (`.part` then rename), so multi-GB archives never sit
/// fully in memory.
async fn download_to_file(url: &str, dest: &Path) -> Result<()> {
    let resp = reqwest::get(url)
        .await
        .with_context(|| format!("GET {url}"))?;
    if !resp.status().is_success() {
        return Err(anyhow!("GET {url}: HTTP {}", resp.status()));
    }
    let tmp = dest.with_extension("part");
    let mut file = tokio::fs::File::create(&tmp)
        .await
        .with_context(|| format!("creating {}", tmp.display()))?;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("download chunk")?;
        file.write_all(&chunk).await.context("writing download")?;
    }
    file.flush().await.ok();
    drop(file);
    tokio::fs::rename(&tmp, dest)
        .await
        .with_context(|| format!("finalising {}", dest.display()))
}

/// Extracts a `.tar.gz` into `dest`.
fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    let file =
        std::fs::File::open(archive).with_context(|| format!("opening {}", archive.display()))?;
    let gz = flate2::read::GzDecoder::new(file);
    tar::Archive::new(gz)
        .unpack(dest)
        .with_context(|| format!("extracting into {}", dest.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawl::Orchestrator;
    use crate::license::Confidence;

    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pdmx")
    }

    #[test]
    fn parses_only_the_no_license_conflict_subset() {
        let rows = parse_usable(&fixture().join(CSV_FILE), "mxl", "title", "composer").unwrap();
        assert_eq!(rows.len(), 1, "only the no_license_conflict row");
        assert_eq!(rows[0].path, "scores/ok.mxl");
        assert_eq!(rows[0].composer.as_deref(), Some("Clementi"));
    }

    #[tokio::test]
    async fn discovers_and_flows_to_the_safe_corpus() {
        // Cache pointed at the fixture (skips download/extract); the .mxl already
        // sits under fixtures/pdmx/mxl/.
        let src = PdmxDatasetSource::new(fixture());
        let items = src.discover().await.unwrap();
        assert_eq!(items.len(), 1);

        let out = Orchestrator::new().run(&src, None).await;
        assert_eq!(out.stats.accepted, 1, "the usable score is ingested");
        let e = &out.prepared[0].entry;
        assert_eq!(e.license, "PublicDomain");
        assert_eq!(e.confidence, Confidence::Verified);
        assert_eq!(e.source, "pdmx");
    }
}
