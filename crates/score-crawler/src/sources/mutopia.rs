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

//! Mutopia Project adapter.
//!
//! Mutopia ships LilyPond `.ly` sources; every piece is Public Domain or under a
//! Creative Commons licence, but that includes NC/ND variants we reject — so the
//! licence is read **per file** from the `.ly` header (`license`/`copyright`
//! field) and run through the same gate. Conversion is LilyPond → MusicXML via
//! `python-ly` (degrades to a per-item failure when the binary is absent). MIDI
//! and PDF renditions are ignored.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;

use super::git::{collect_scores, ensure_checkout, origin_from_ext};
use super::{Item, RawScore, SourceAdapter};
use crate::convert::OriginFormat;
use crate::license::RawLicense;

/// Canonical Mutopia source repository.
pub const MUTOPIA_REPO: &str = "https://github.com/MutopiaProject/MutopiaProject";

/// Mutopia adapter over a cloned checkout.
pub struct MutopiaSource {
    checkout: PathBuf,
}

impl MutopiaSource {
    pub fn new(checkout: impl Into<PathBuf>) -> Self {
        Self {
            checkout: checkout.into(),
        }
    }

    fn item_for(&self, path: &Path) -> Option<Item> {
        // LilyPond sources only (ignore any bundled MIDI/PDF).
        if origin_from_ext(path) != Some(OriginFormat::LilyPond) {
            return None;
        }
        let rel = path.strip_prefix(&self.checkout).ok()?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().replace(['_', '-'], " "));
        Some(Item {
            source_item_id: rel_str.clone(),
            url: format!("{}/{}", MUTOPIA_REPO, rel_str),
            title,
            composer: None,
            arranger: None,
            source_grade: None,
        })
    }
}

#[async_trait]
impl SourceAdapter for MutopiaSource {
    fn name(&self) -> &str {
        "mutopia"
    }

    async fn prepare(&self) -> Result<()> {
        let dest = self.checkout.clone();
        tokio::task::spawn_blocking(move || ensure_checkout(MUTOPIA_REPO, &dest))
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

    async fn extract_license(&self, item: &Item) -> Result<RawLicense> {
        // The licence lives in the (local, small) `.ly` header — a source-
        // authoritative field, so it is `verified`, gated per file.
        let path = self.checkout.join(&item.source_item_id);
        let ly = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let signal = parse_ly_license(&ly)
            .ok_or_else(|| anyhow!("no licence field in {}", item.source_item_id))?;
        Ok(RawLicense::verified(signal))
    }

    async fn fetch(&self, item: &Item) -> Result<RawScore> {
        let path = self.checkout.join(&item.source_item_id);
        let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        Ok(RawScore {
            origin: OriginFormat::LilyPond,
            bytes,
        })
    }
}

/// Extracts the licence signal from a LilyPond header: the value of the first
/// `license = "…"` (or `copyright = "…"`) field. Pure; testable offline.
pub fn parse_ly_license(ly: &str) -> Option<String> {
    for key in ["license", "copyright"] {
        for line in ly.lines() {
            let l = line.trim();
            if let Some(after) = l.strip_prefix(key)
                && after.trim_start().starts_with('=')
                && let Some(value) = between_quotes(after)
                && !value.is_empty()
            {
                return Some(value);
            }
        }
    }
    None
}

/// The text between the first pair of double quotes.
fn between_quotes(s: &str) -> Option<String> {
    let start = s.find('"')?;
    let rest = &s[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawl::Orchestrator;
    use crate::license::{Decision, evaluate};

    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutopia")
    }

    #[test]
    fn parses_license_and_copyright_fields() {
        assert_eq!(
            parse_ly_license("\\header {\n  license = \"Public Domain\"\n}").as_deref(),
            Some("Public Domain")
        );
        assert_eq!(
            parse_ly_license("  copyright = \"Creative Commons Attribution-ShareAlike 4.0\"")
                .as_deref(),
            Some("Creative Commons Attribution-ShareAlike 4.0")
        );
        assert_eq!(parse_ly_license("title = \"No licence here\""), None);
    }

    #[tokio::test]
    async fn per_file_gate_accepts_free_and_rejects_nc() {
        let src = MutopiaSource::new(fixture());
        let items = src.discover().await.unwrap();
        assert_eq!(items.len(), 3, "three .ly fixtures");

        // Each file's own header drives the decision.
        let mut by_id = std::collections::HashMap::new();
        for item in &items {
            let raw = src.extract_license(item).await.unwrap();
            by_id.insert(item.source_item_id.clone(), evaluate(&raw).1);
        }
        assert_eq!(by_id["free_sa.ly"], Decision::Accept);
        assert_eq!(by_id["pd.ly"], Decision::Accept);
        assert!(matches!(by_id["nonfree_nc.ly"], Decision::Reject { .. }));
    }

    #[tokio::test]
    async fn orchestrator_gates_nc_before_any_fetch() {
        // Through the orchestrator, the NC file is rejected at the gate; the two
        // free files pass the gate then fail conversion only if python-ly is
        // absent (a per-item failure, never a panic).
        let out = Orchestrator::new()
            .run(&MutopiaSource::new(fixture()), None)
            .await;
        assert!(
            out.stats.rejected >= 1,
            "the CC-BY-NC file is licence-rejected"
        );
        assert!(
            out.rejected
                .iter()
                .any(|r| r.url.contains("nonfree_nc") && r.reason.contains("not redistributable"))
        );
    }
}
