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

//! The git-clone adapter family.
//!
//! A [`GitRepoSource`] harvests a cloned repository: it walks the checkout for
//! score files and yields them as items with a repo-wide licence. The clone/pull
//! itself is a thin `git` subprocess ([`ensure_checkout`], not unit-tested);
//! everything else — walking, licence, reading bytes — is pure filesystem work
//! tested against an on-disk fixture. Concrete sources (OpenScore, musetrainer,
//! eduardomourar) are constructors over this one type.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;

use super::{Item, RawScore, SourceAdapter};
use crate::convert::OriginFormat;
use crate::difficulty::Level;
use crate::license::RawLicense;

/// How a repository declares the licence covering its scores.
#[derive(Debug, Clone)]
pub enum RepoLicense {
    /// One authoritative licence for the whole repo (e.g. OpenScore is CC0).
    Verified(String),
    /// User-uploaded, self-declared public domain (e.g. musetrainer) →
    /// low-confidence.
    Declared(String),
}

impl RepoLicense {
    fn to_raw(&self) -> RawLicense {
        match self {
            RepoLicense::Verified(s) => RawLicense::verified(s.clone()),
            RepoLicense::Declared(s) => RawLicense::declared(s.clone()),
        }
    }
}

/// A source backed by a cloned git repository on disk.
pub struct GitRepoSource {
    name: String,
    /// Local checkout directory (populated by [`ensure_checkout`]).
    checkout: PathBuf,
    /// Canonical repo URL, used to build per-item source URLs.
    repo_url: String,
    license: RepoLicense,
    /// A source-declared difficulty grade, if the repo provides one.
    grade: Option<Level>,
}

impl GitRepoSource {
    pub fn new(
        name: impl Into<String>,
        checkout: impl Into<PathBuf>,
        repo_url: impl Into<String>,
        license: RepoLicense,
    ) -> Self {
        Self {
            name: name.into(),
            checkout: checkout.into(),
            repo_url: repo_url.into(),
            license,
            grade: None,
        }
    }

    /// OpenScore Lieder Corpus — MuseScore `.mscx`, CC0 repo-wide.
    pub fn openscore(checkout: impl Into<PathBuf>) -> Self {
        Self::new(
            "openscore",
            checkout,
            "https://github.com/OpenScore/Lieder",
            RepoLicense::Verified("CC0".into()),
        )
    }

    /// musetrainer/library — MusicXML, self-declared public domain →
    /// low-confidence.
    pub fn musetrainer(checkout: impl Into<PathBuf>) -> Self {
        Self::new(
            "musetrainer",
            checkout,
            "https://github.com/musetrainer/library",
            RepoLicense::Declared("Public Domain".into()),
        )
    }

    /// eduardomourar/music-scores-musicxml — MusicXML; treat as CC0 pending the
    /// README/LICENSE check.
    pub fn eduardomourar(checkout: impl Into<PathBuf>) -> Self {
        Self::new(
            "eduardomourar",
            checkout,
            "https://github.com/eduardomourar/music-scores-musicxml",
            RepoLicense::Verified("CC0".into()),
        )
    }

    /// Builds an [`Item`] for a score file under the checkout, or `None` if the
    /// extension is not a supported score format.
    fn item_for(&self, path: &Path) -> Option<Item> {
        origin_from_ext(path)?;
        let rel = path.strip_prefix(&self.checkout).ok()?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().replace(['_', '-'], " "));
        Some(Item {
            source_item_id: rel_str.clone(),
            url: format!("{}/{}", self.repo_url.trim_end_matches('/'), rel_str),
            title,
            composer: None,
            arranger: None,
            source_grade: self.grade,
        })
    }
}

#[async_trait]
impl SourceAdapter for GitRepoSource {
    fn name(&self) -> &str {
        &self.name
    }

    async fn prepare(&self) -> Result<()> {
        // Clone/pull the repo off the async runtime (blocking `git` subprocess).
        let url = self.repo_url.clone();
        let dest = self.checkout.clone();
        tokio::task::spawn_blocking(move || ensure_checkout(&url, &dest))
            .await
            .context("joining git task")?
    }

    async fn discover(&self) -> Result<Vec<Item>> {
        let mut files = Vec::new();
        collect_scores(&self.checkout, &mut files)
            .with_context(|| format!("walking checkout {}", self.checkout.display()))?;
        files.sort();
        Ok(files.iter().filter_map(|p| self.item_for(p)).collect())
    }

    async fn extract_license(&self, _item: &Item) -> Result<RawLicense> {
        // Repo-wide licence — the same verified/declared signal for every file.
        Ok(self.license.to_raw())
    }

    async fn fetch(&self, item: &Item) -> Result<RawScore> {
        let path = self.checkout.join(&item.source_item_id);
        let origin = origin_from_ext(&path)
            .ok_or_else(|| anyhow!("unsupported score format: {}", item.source_item_id))?;
        let bytes =
            std::fs::read(&path).with_context(|| format!("reading score {}", path.display()))?;
        Ok(RawScore { origin, bytes })
    }
}

/// The origin format implied by a file extension, or `None` if unsupported.
/// MIDI is deliberately excluded — never a score source.
pub fn origin_from_ext(path: &Path) -> Option<OriginFormat> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("musicxml") | Some("xml") => Some(OriginFormat::MusicXml),
        Some("mxl") => Some(OriginFormat::Mxl),
        Some("mscx") | Some("mscz") => Some(OriginFormat::MuseScore),
        Some("mei") => Some(OriginFormat::Mei),
        Some("ly") => Some(OriginFormat::LilyPond),
        _ => None,
    }
}

/// Recursively collects score files under `dir`, skipping `.git`.
fn collect_scores(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Err(anyhow!("checkout does not exist: {}", dir.display()));
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                continue;
            }
            collect_scores(&path, out)?;
        } else if origin_from_ext(&path).is_some() {
            out.push(path);
        }
    }
    Ok(())
}

/// Clones `repo_url` into `dest` (or `git pull`s an existing checkout). Thin
/// subprocess glue — not exercised by unit tests.
pub fn ensure_checkout(repo_url: &str, dest: &Path) -> Result<()> {
    let status = if dest.join(".git").exists() {
        Command::new("git")
            .arg("-C")
            .arg(dest)
            .arg("pull")
            .arg("--ff-only")
            .status()
    } else {
        Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(repo_url)
            .arg(dest)
            .status()
    }
    .with_context(|| format!("running git for {repo_url}"))?;
    if !status.success() {
        return Err(anyhow!("git failed for {repo_url} ({status})"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawl::Orchestrator;
    use crate::license::Confidence;

    fn fixture(repo: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(repo)
    }

    #[test]
    fn origin_by_extension_excludes_midi() {
        assert_eq!(
            origin_from_ext(Path::new("a.musicxml")),
            Some(OriginFormat::MusicXml)
        );
        assert_eq!(origin_from_ext(Path::new("a.MXL")), Some(OriginFormat::Mxl));
        assert_eq!(
            origin_from_ext(Path::new("a.mscx")),
            Some(OriginFormat::MuseScore)
        );
        assert_eq!(origin_from_ext(Path::new("a.mid")), None);
        assert_eq!(origin_from_ext(Path::new("readme.txt")), None);
    }

    #[tokio::test]
    async fn discovers_and_reads_scores_from_a_checkout() {
        let src = GitRepoSource::musetrainer(fixture("musetrainer"));
        let items = src.discover().await.unwrap();
        // Two score files in the fixture; the LICENSE/README are ignored.
        assert_eq!(items.len(), 2);
        let first = &items[0];
        let raw = src.extract_license(first).await.unwrap();
        assert!(raw.self_declared);
        let score = src.fetch(first).await.unwrap();
        assert_eq!(score.origin, OriginFormat::MusicXml);
        assert!(!score.bytes.is_empty());
    }

    #[tokio::test]
    async fn musetrainer_flows_to_low_confidence_through_the_orchestrator() {
        let src = GitRepoSource::musetrainer(fixture("musetrainer"));
        let out = Orchestrator::new().run(&src, None).await;
        // Self-declared PD → low-confidence corpus, never the safe corpus.
        assert_eq!(out.stats.low_confidence, 2);
        assert_eq!(out.stats.accepted, 0);
        assert!(
            out.prepared
                .iter()
                .all(|p| p.entry.confidence == Confidence::Unverified)
        );
        assert!(out.prepared.iter().all(|p| p.entry.source == "musetrainer"));
    }

    #[tokio::test]
    async fn missing_checkout_yields_no_items_without_panicking() {
        let src = GitRepoSource::openscore(fixture("does_not_exist"));
        assert!(src.discover().await.is_err());
    }
}
