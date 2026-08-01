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
        // Filename fallback only — `enrich_from_header` replaces this with the
        // real `\header` title (e.g. `bwv-1001_1` → "BWV 1001 Adagio").
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

/// Overwrites an item's `title`/`composer` with the source-authoritative values
/// from the `.ly` `\header`, when present. Mutopia names files by an opaque id
/// (`bwv-1001_1`), so the filename-derived title is a poor last resort; the
/// header carries the real work title. The movement-distinct `mutopiatitle`
/// ("BWV 1001 Adagio") is preferred over the shared `title` ("Sonata I BWV
/// 1001") so sibling movements don't collapse to one name.
fn enrich_from_header(item: &mut Item, ly: &str) {
    let (title, composer) = header_title_composer(ly);
    if let Some(title) = title {
        item.title = Some(title);
    }
    if let Some(composer) = composer {
        item.composer = Some(composer);
    }
}

/// The source-authoritative `(title, composer)` from a `.ly` `\header`, each
/// `None` when its field is absent. Shared by discovery ([`enrich_from_header`])
/// and the catalog title backfill so both derive them identically. Prefers the
/// movement-distinct `mutopiatitle` over the shared `title`; `composer` is the
/// human name ("Johann Sebastian Bach (1685-1750)") — never the opaque
/// `mutopiacomposer` id ("BachJS").
pub fn header_title_composer(ly: &str) -> (Option<String>, Option<String>) {
    let title =
        parse_ly_header_field(ly, "mutopiatitle").or_else(|| parse_ly_header_field(ly, "title"));
    let composer = parse_ly_header_field(ly, "composer");
    (title, composer)
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
        // A Mutopia repo holds real piece files (a `\header` with `license = …`)
        // AND many `-lys/` includes + `contrib/templates` with no licence. Keep
        // only the real pieces; skip the rest silently (they are not failures).
        // Read each `.ly` once: the licence gates it in or out, and its `\header`
        // supplies the real title/composer (the filename is only a fallback).
        let items = files
            .iter()
            .filter_map(|p| {
                let mut item = self.item_for(p)?;
                let ly = std::fs::read_to_string(p).ok()?;
                parse_ly_license(&ly)?; // real piece only
                enrich_from_header(&mut item, &ly);
                Some(item)
            })
            .collect();
        Ok(items)
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
            // Require `key = "value"` (a quote right after `=`), so a
            // `copyright = \markup { … "font" … }` never yields a font name.
            if let Some(rest) = line.trim().strip_prefix(key)
                && let Some(rest) = rest.trim_start().strip_prefix('=')
                && rest.trim_start().starts_with('"')
                && let Some(value) = between_quotes(rest.trim_start())
                && !value.is_empty()
            {
                return Some(value);
            }
        }
    }
    None
}

/// Extracts the value of a quoted `\header` field (`key = "value"`). Same quote
/// discipline as [`parse_ly_license`]: a quote must sit right after `=`, so a
/// `\markup { … }` value never leaks through, and the `= ` guard means `title`
/// won't match `subtitle`/`mutopiatitle`. Pure; testable offline.
pub fn parse_ly_header_field(ly: &str, key: &str) -> Option<String> {
    for line in ly.lines() {
        if let Some(rest) = line.trim().strip_prefix(key)
            && let Some(rest) = rest.trim_start().strip_prefix('=')
            && rest.trim_start().starts_with('"')
            && let Some(value) = between_quotes(rest.trim_start())
            && !value.is_empty()
        {
            return Some(value);
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
        // A markup copyright (a real Mutopia pattern) must NOT yield a font name.
        assert_eq!(
            parse_ly_license("copyright = \\markup {\\override #'(font-name . \"DejaVu Sans\")}"),
            None
        );
        // ...but when a real `license` line is also present, it still wins.
        assert_eq!(
            parse_ly_license("license = \"Public Domain\"\ncopyright = \\markup {\"DejaVu\"}")
                .as_deref(),
            Some("Public Domain")
        );
    }

    // A realistic Mutopia `\header` (BachJS/BWV1001/bwv-1001_1): the file is named
    // by an opaque id, and the real metadata lives in the header.
    const BWV_1001_1: &str = r#"\header {
        title = "Sonata I BWV 1001"
        subtitle = "\"Sechs Sonaten für Violine\""
        piece = "1. Adagio"
        mutopiatitle = "BWV 1001 Adagio"
        composer = "Johann Sebastian Bach (1685-1750)"
        mutopiacomposer = "BachJS"
        opus = "BWV 1001"
        copyright = "Creative Commons Attribution-ShareAlike 3.0"
    }"#;

    #[test]
    fn header_field_respects_the_equals_guard() {
        // `title` must not be shadowed by `subtitle`/`mutopiatitle`.
        assert_eq!(
            parse_ly_header_field(BWV_1001_1, "title").as_deref(),
            Some("Sonata I BWV 1001")
        );
        assert_eq!(
            parse_ly_header_field(BWV_1001_1, "mutopiatitle").as_deref(),
            Some("BWV 1001 Adagio")
        );
        // `composer`, not the opaque `mutopiacomposer` id.
        assert_eq!(
            parse_ly_header_field(BWV_1001_1, "composer").as_deref(),
            Some("Johann Sebastian Bach (1685-1750)")
        );
        assert_eq!(parse_ly_header_field(BWV_1001_1, "arranger"), None);
    }

    #[test]
    fn enrich_prefers_mutopiatitle_and_real_composer_over_filename() {
        // Start from the filename-derived fallback the walker would produce.
        let mut item = Item {
            source_item_id: "ftp/BachJS/BWV1001/bwv-1001_1/bwv-1001_1.ly".into(),
            url: "url".into(),
            title: Some("bwv 1001 1".into()),
            composer: None,
            arranger: None,
            source_grade: None,
        };
        enrich_from_header(&mut item, BWV_1001_1);
        // The movement-distinct mutopiatitle wins over the shared `title`...
        assert_eq!(item.title.as_deref(), Some("BWV 1001 Adagio"));
        // ...and the human composer name is filled in (never "BachJS").
        assert_eq!(
            item.composer.as_deref(),
            Some("Johann Sebastian Bach (1685-1750)")
        );
    }

    #[test]
    fn enrich_falls_back_to_title_then_keeps_filename() {
        // No `mutopiatitle`: the plain `title` is used.
        let mut item = Item {
            source_item_id: "x.ly".into(),
            url: "url".into(),
            title: Some("x".into()),
            composer: None,
            arranger: None,
            source_grade: None,
        };
        enrich_from_header(&mut item, "\\header {\n  title = \"Prelude\"\n}");
        assert_eq!(item.title.as_deref(), Some("Prelude"));

        // No title field at all: the filename fallback survives untouched.
        let mut bare = Item {
            source_item_id: "y.ly".into(),
            url: "url".into(),
            title: Some("my file".into()),
            composer: None,
            arranger: None,
            source_grade: None,
        };
        enrich_from_header(&mut bare, "\\header {\n  license = \"Public Domain\"\n}");
        assert_eq!(bare.title.as_deref(), Some("my file"));
        assert_eq!(bare.composer, None);
    }

    #[tokio::test]
    async fn per_file_gate_accepts_free_and_rejects_nc() {
        let src = MutopiaSource::new(fixture());
        let items = src.discover().await.unwrap();
        assert_eq!(items.len(), 3, "three .ly fixtures");

        // discover() enriches from the `\header`: titles/composers come from the
        // header fields, not the filenames (`pd.ly` → "Old Piece", not "pd").
        let titled: std::collections::HashMap<_, _> = items
            .iter()
            .map(|i| (i.source_item_id.clone(), i))
            .collect();
        assert_eq!(titled["pd.ly"].title.as_deref(), Some("Old Piece"));
        assert_eq!(titled["pd.ly"].composer.as_deref(), Some("J. S. Bach"));
        assert_eq!(titled["free_sa.ly"].title.as_deref(), Some("Free Piece"));

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
