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

//! The central orchestrator: the fixed, license-first pipeline.
//!
//! For each discovered item the licence is evaluated **before** any heavy
//! download; a non-whitelisted licence is journalled and skipped with no
//! `fetch`. Accepted items are fetched, converted to validated `.mxl`,
//! deduplicated by content hash, and enriched with search metadata + a
//! difficulty assessment into a [`PreparedScore`] ready for ingestion (the
//! object-store + `catalog_scores` write lands with the backend score module).
//! A failure on one item is recorded and skipped — it never aborts the crawl.

use std::collections::HashSet;

use sha2::{Digest, Sha256};
use tracing::warn;

use crate::difficulty::assess;
use crate::license::{Confidence, Decision, evaluate};
use crate::manifest::{ManifestEntry, RejectionRecord};
use crate::metadata;
use crate::sources::{Item, SourceAdapter};

/// Running tallies for a crawl.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CrawlStats {
    pub discovered: usize,
    pub accepted: usize,
    pub low_confidence: usize,
    pub rejected: usize,
    pub failed: usize,
    pub deduped: usize,
}

/// A retained score ready to ingest: its manifest entry and the `.mxl` bytes.
#[derive(Debug, Clone)]
pub struct PreparedScore {
    pub entry: ManifestEntry,
    pub mxl: Vec<u8>,
}

/// The result of crawling one (or more) adapters.
#[derive(Debug, Default)]
pub struct CrawlOutcome {
    pub prepared: Vec<PreparedScore>,
    /// Licence rejections and per-item failures, for `rejected.log`.
    pub rejected: Vec<RejectionRecord>,
    pub stats: CrawlStats,
}

/// Drives adapters through the license-first pipeline, deduplicating by content
/// hash across everything it has already seen this run (and, later, against
/// existing `catalog_scores` rows via [`Orchestrator::with_seen`]).
pub struct Orchestrator {
    /// SHA-256 of the canonical MusicXML — exact-content dedup.
    seen: HashSet<String>,
    /// Musical content fingerprints — dedup across re-encodings/sources.
    seen_fp: HashSet<String>,
    /// Whether to also dedup by musical fingerprint (not just exact bytes).
    dedup_fingerprints: bool,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
            seen_fp: HashSet::new(),
            dedup_fingerprints: true,
        }
    }

    /// Seed the exact-content dedup set with hashes already in the catalog so a
    /// resumed crawl does not re-ingest existing content.
    pub fn with_seen(seen: HashSet<String>) -> Self {
        Self {
            seen,
            ..Self::new()
        }
    }

    /// Seed the fingerprint dedup set (e.g. from existing `catalog_scores`).
    pub fn seed_fingerprints(mut self, fingerprints: HashSet<String>) -> Self {
        self.seen_fp = fingerprints;
        self
    }

    /// Disables musical-fingerprint dedup (keep only exact-content dedup).
    pub fn without_fingerprint_dedup(mut self) -> Self {
        self.dedup_fingerprints = false;
        self
    }

    /// Crawls one adapter, up to `limit` items. Never panics; per-item errors are
    /// recorded and skipped.
    pub async fn run(&mut self, adapter: &dyn SourceAdapter, limit: Option<usize>) -> CrawlOutcome {
        let mut out = CrawlOutcome::default();
        let items = match adapter.discover().await {
            Ok(items) => items,
            Err(e) => {
                warn!(source = adapter.name(), error = %e, "discover failed; skipping source");
                return out;
            }
        };
        for item in items.into_iter().take(limit.unwrap_or(usize::MAX)) {
            out.stats.discovered += 1;
            self.process(adapter, &item, &mut out).await;
        }
        out
    }

    async fn process(&mut self, adapter: &dyn SourceAdapter, item: &Item, out: &mut CrawlOutcome) {
        // --- License-first: decide before any heavy download. ---
        let raw = match adapter.extract_license(item).await {
            Ok(r) => r,
            Err(e) => {
                self.fail(
                    out,
                    adapter,
                    item,
                    "<none>",
                    format!("licence read failed: {e}"),
                );
                return;
            }
        };
        let (outcome, decision) = evaluate(&raw);
        let confidence = match decision {
            Decision::Reject { reason } => {
                out.stats.rejected += 1;
                out.rejected.push(RejectionRecord {
                    source: adapter.name().to_string(),
                    url: item.url.clone(),
                    raw_signal: raw.signal.clone(),
                    reason,
                });
                return; // gate: no fetch, no conversion for rejected licences.
            }
            Decision::Accept => Confidence::Verified,
            Decision::LowConfidence => Confidence::Unverified,
        };

        // --- Heavy work only for whitelisted licences. ---
        let raw_score = match adapter.fetch(item).await {
            Ok(s) => s,
            Err(e) => {
                self.fail(
                    out,
                    adapter,
                    item,
                    &raw.signal,
                    format!("fetch failed: {e}"),
                );
                return;
            }
        };
        let origin = raw_score.origin;
        let converted = match adapter.to_musicxml(raw_score).await {
            Ok(c) => c,
            Err(e) => {
                self.fail(
                    out,
                    adapter,
                    item,
                    &raw.signal,
                    format!("conversion failed: {e}"),
                );
                return;
            }
        };

        // --- Decode + parse once (the decoded MusicXML, not the zip container,
        // so framing differences never defeat dedup). ---
        let inner = match cymbra_musicxml_core::mxl::decode(&converted.mxl) {
            Ok(inner) => inner,
            Err(e) => {
                self.fail(
                    out,
                    adapter,
                    item,
                    &raw.signal,
                    format!("decode failed: {e}"),
                );
                return;
            }
        };
        let doc = match cymbra_musicxml_core::parse(&inner) {
            Ok(doc) => doc,
            Err(e) => {
                self.fail(
                    out,
                    adapter,
                    item,
                    &raw.signal,
                    format!("parse failed: {e}"),
                );
                return;
            }
        };

        // --- Deduplicate: exact content (SHA-256) always; musical fingerprint
        // (same notes, any encoding/source) when enabled. ---
        let sha = sha256_hex(&inner);
        let fingerprint = crate::fingerprint::content_fingerprint(&doc);
        let is_dup = self.seen.contains(&sha)
            || (self.dedup_fingerprints && self.seen_fp.contains(&fingerprint));
        if is_dup {
            out.stats.deduped += 1;
            return;
        }
        self.seen.insert(sha.clone());
        self.seen_fp.insert(fingerprint.clone());

        // --- Enrich from the parsed score. ---
        let meta = metadata::extract(&doc);
        let difficulty = assess(&doc, item.source_grade);

        let entry = ManifestEntry {
            id: format!("{}:{}", adapter.name(), item.source_item_id),
            title: item.title.clone().or(meta.title),
            composer: item.composer.clone().or(meta.composer),
            arranger: item.arranger.clone(),
            source: adapter.name().to_string(),
            source_url: item.url.clone(),
            source_item_id: item.source_item_id.clone(),
            license: outcome.code,
            license_url: outcome.url,
            confidence,
            sha256: sha,
            content_fingerprint: fingerprint,
            origin_format: origin,
            conversion_status: converted.status,
            object_key: None, // set at ingest
            size_bytes: converted.mxl.len() as u64,
            work_key: meta.work_key,
            title_norm: meta.title_norm,
            is_piano: meta.is_piano,
            key_fifths: meta.key_fifths,
            time_sig: meta.time_sig,
            measure_count: meta.measure_count,
            language: meta.language,
            voicing: meta.voicing,
            level: difficulty.level,
            level_source: difficulty.source,
        };

        match confidence {
            Confidence::Verified => out.stats.accepted += 1,
            Confidence::Unverified => out.stats.low_confidence += 1,
        }
        out.prepared.push(PreparedScore {
            entry,
            mxl: converted.mxl,
        });
    }

    /// Records a per-item failure (not a licence rejection) and continues.
    fn fail(
        &self,
        out: &mut CrawlOutcome,
        adapter: &dyn SourceAdapter,
        item: &Item,
        signal: &str,
        reason: String,
    ) {
        warn!(source = adapter.name(), item = item.source_item_id, %reason, "item skipped");
        out.stats.failed += 1;
        out.rejected.push(RejectionRecord {
            source: adapter.name().to_string(),
            url: item.url.clone(),
            raw_signal: signal.to_string(),
            reason,
        });
    }
}

/// Lowercase hex SHA-256 of some bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::convert::OriginFormat;
    use crate::license::RawLicense;
    use crate::sources::test_fake::FakeAdapter;
    use crate::sources::{Item, RawScore};

    const SCORE_A: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<key><fifths>0</fifths></key><time><beats>4</beats><beat-type>4</beat-type></time>
<clef><sign>G</sign><line>2</line></clef></attributes>
<note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note></measure></part></score-partwise>"#;

    const SCORE_B: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<key><fifths>2</fifths></key><time><beats>3</beats><beat-type>4</beat-type></time>
<clef><sign>G</sign><line>2</line></clef></attributes>
<note><pitch><step>G</step><octave>4</octave></pitch><duration>3</duration><type>half</type></note></measure></part></score-partwise>"#;

    // SCORE_A re-encoded: same note (C4 whole) but a different editor/divisions,
    // so different bytes (different sha256) yet identical music (same fingerprint).
    const SCORE_A_REENCODED: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Klavier</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>2</divisions>
<key><fifths>0</fifths></key><time><beats>4</beats><beat-type>4</beat-type></time>
<clef><sign>G</sign><line>2</line></clef></attributes>
<note><pitch><step>C</step><octave>4</octave></pitch><duration>8</duration><type>whole</type></note></measure></part></score-partwise>"#;

    fn item(id: &str) -> Item {
        Item {
            source_item_id: id.into(),
            url: format!("https://example.org/{id}"),
            title: Some(format!("Piece {id}")),
            composer: Some("Anon".into()),
            arranger: None,
            source_grade: None,
        }
    }

    fn score(xml: &str) -> RawScore {
        RawScore {
            origin: OriginFormat::MusicXml,
            bytes: xml.as_bytes().to_vec(),
        }
    }

    fn adapter(
        items: Vec<(Item, RawLicense, RawScore)>,
        fail: Vec<String>,
    ) -> (FakeAdapter, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        (
            FakeAdapter {
                name: "fake".into(),
                items,
                fetch_calls: calls.clone(),
                fail_fetch: fail,
            },
            calls,
        )
    }

    #[tokio::test]
    async fn accepts_whitelisted_and_rejects_others_without_fetching() {
        let (a, calls) = adapter(
            vec![
                (item("ok"), RawLicense::verified("CC0"), score(SCORE_A)),
                (
                    item("bad"),
                    RawLicense::verified("All Rights Reserved"),
                    score(SCORE_B),
                ),
            ],
            vec![],
        );
        let out = Orchestrator::new().run(&a, None).await;

        assert_eq!(out.stats.accepted, 1);
        assert_eq!(out.stats.rejected, 1);
        assert_eq!(out.prepared.len(), 1);
        assert_eq!(out.prepared[0].entry.license, "CC0-1.0");
        // The gate skipped fetch for the rejected item: only one download.
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        // The rejection is journalled with its reason.
        assert!(
            out.rejected
                .iter()
                .any(|r| r.reason.contains("not redistributable"))
        );
    }

    #[tokio::test]
    async fn self_declared_pd_is_low_confidence() {
        let (a, _) = adapter(
            vec![(
                item("x"),
                RawLicense::declared("Public Domain"),
                score(SCORE_A),
            )],
            vec![],
        );
        let out = Orchestrator::new().run(&a, None).await;
        assert_eq!(out.stats.low_confidence, 1);
        assert_eq!(out.prepared[0].entry.confidence, Confidence::Unverified);
    }

    #[tokio::test]
    async fn deduplicates_identical_content_across_items() {
        let (a, _) = adapter(
            vec![
                (
                    item("one"),
                    RawLicense::verified("CC-BY-4.0"),
                    score(SCORE_A),
                ),
                (
                    item("two"),
                    RawLicense::verified("CC-BY-4.0"),
                    score(SCORE_A),
                ),
            ],
            vec![],
        );
        let out = Orchestrator::new().run(&a, None).await;
        assert_eq!(out.prepared.len(), 1);
        assert_eq!(out.stats.deduped, 1);
    }

    #[tokio::test]
    async fn fingerprint_dedups_reencoded_music() {
        // Same piece, re-encoded (different bytes/sha) — deduped by fingerprint.
        let (a, _) = adapter(
            vec![
                (item("orig"), RawLicense::verified("CC0"), score(SCORE_A)),
                (
                    item("reenc"),
                    RawLicense::verified("CC0"),
                    score(SCORE_A_REENCODED),
                ),
            ],
            vec![],
        );
        let out = Orchestrator::new().run(&a, None).await;
        assert_eq!(
            out.prepared.len(),
            1,
            "the re-encoding is a musical duplicate"
        );
        assert_eq!(out.stats.deduped, 1);

        // With fingerprint dedup off, both are kept (exact-content dedup only).
        let (b, _) = adapter(
            vec![
                (item("orig"), RawLicense::verified("CC0"), score(SCORE_A)),
                (
                    item("reenc"),
                    RawLicense::verified("CC0"),
                    score(SCORE_A_REENCODED),
                ),
            ],
            vec![],
        );
        let out = Orchestrator::new()
            .without_fingerprint_dedup()
            .run(&b, None)
            .await;
        assert_eq!(out.prepared.len(), 2);
    }

    #[tokio::test]
    async fn a_single_item_failure_does_not_abort_the_crawl() {
        let (a, _) = adapter(
            vec![
                (item("boom"), RawLicense::verified("CC0"), score(SCORE_A)),
                (item("fine"), RawLicense::verified("CC0"), score(SCORE_B)),
            ],
            vec!["boom".into()],
        );
        let out = Orchestrator::new().run(&a, None).await;
        // The failing item is recorded; the next one still succeeds.
        assert_eq!(out.stats.failed, 1);
        assert_eq!(out.stats.accepted, 1);
        assert!(
            out.rejected
                .iter()
                .any(|r| r.reason.contains("fetch failed"))
        );
    }

    #[tokio::test]
    async fn limit_bounds_the_run() {
        let items: Vec<_> = (0..5)
            .map(|i| {
                (
                    item(&i.to_string()),
                    RawLicense::verified("CC0"),
                    score(SCORE_A),
                )
            })
            .collect();
        let (a, _) = adapter(items, vec![]);
        let out = Orchestrator::new().run(&a, Some(2)).await;
        assert_eq!(out.stats.discovered, 2);
    }
}
