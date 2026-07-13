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

//! The shared crawl run loop used by the CLI.
//!
//! Prepares and runs each adapter through the license-first orchestrator
//! (deduplicating across sources) and returns the merged outcome.

use crate::crawl::{CrawlOutcome, Orchestrator};
use crate::sources::SourceAdapter;

/// Runs every adapter, folding results into one [`CrawlOutcome`]. Per-source
/// prepare/discover failures are logged and skipped, never fatal.
pub async fn run_all(adapters: &[Box<dyn SourceAdapter>], limit: Option<usize>) -> CrawlOutcome {
    let mut orchestrator = Orchestrator::new();
    let mut merged = CrawlOutcome::default();

    for adapter in adapters {
        if let Err(e) = adapter.prepare().await {
            tracing::warn!(source = adapter.name(), error = %e, "prepare failed; skipping");
            continue;
        }
        let out = orchestrator.run(adapter.as_ref(), limit).await;
        merge(&mut merged, out);
    }
    merged
}

/// Folds one adapter's outcome into the merged totals.
pub fn merge(into: &mut CrawlOutcome, from: CrawlOutcome) {
    into.prepared.extend(from.prepared);
    into.rejected.extend(from.rejected);
    into.stats.discovered += from.stats.discovered;
    into.stats.accepted += from.stats.accepted;
    into.stats.low_confidence += from.stats.low_confidence;
    into.stats.rejected += from.stats.rejected;
    into.stats.failed += from.stats.failed;
    into.stats.deduped += from.stats.deduped;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::OriginFormat;
    use crate::license::RawLicense;
    use crate::sources::test_fake::FakeAdapter;
    use crate::sources::{Item, RawScore};
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    // A distinct score per id — distinct NOTES (not just metadata) so the two
    // fake sources are not merged by the musical-content fingerprint.
    fn score_for(id: &str) -> String {
        let bytes = id.as_bytes();
        let first = u32::from(bytes.first().copied().unwrap_or(b'x'));
        let last = u32::from(bytes.last().copied().unwrap_or(b'y'));
        // Octave from the first byte, step from the last ⇒ distinct ids differing
        // in either byte get distinct notes (no fingerprint collision).
        let step = ['C', 'D', 'E', 'F', 'G', 'A', 'B'][(last % 7) as usize];
        let octave = 2 + (first % 5);
        format!(
            r#"<?xml version="1.0"?>
<score-partwise version="4.0"><work><work-title>{id}</work-title></work>
<part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<key><fifths>0</fifths></key><time><beats>4</beats><beat-type>4</beat-type></time>
<clef><sign>G</sign><line>2</line></clef></attributes>
<note><pitch><step>{step}</step><octave>{octave}</octave></pitch><duration>4</duration><type>whole</type></note></measure></part></score-partwise>"#
        )
    }

    fn fake(name: &str, id: &str) -> FakeAdapter {
        let item = Item {
            source_item_id: id.into(),
            url: format!("https://ex/{id}"),
            title: Some(id.into()),
            composer: None,
            arranger: None,
            source_grade: None,
        };
        FakeAdapter {
            name: name.into(),
            items: vec![(
                item,
                RawLicense::verified("CC0"),
                RawScore {
                    origin: OriginFormat::MusicXml,
                    bytes: score_for(id).into_bytes(),
                },
            )],
            fetch_calls: Arc::new(AtomicUsize::new(0)),
            fail_fetch: vec![],
        }
    }

    /// A fake source exposing `n` distinct items.
    fn fake_n(name: &str, n: usize) -> FakeAdapter {
        let items = (0..n)
            .map(|i| {
                let id = format!("{name}-{i}");
                (
                    Item {
                        source_item_id: id.clone(),
                        url: format!("https://ex/{id}"),
                        title: Some(id.clone()),
                        composer: None,
                        arranger: None,
                        source_grade: None,
                    },
                    RawLicense::verified("CC0"),
                    RawScore {
                        origin: OriginFormat::MusicXml,
                        bytes: score_for(&id).into_bytes(),
                    },
                )
            })
            .collect();
        FakeAdapter {
            name: name.into(),
            items,
            fetch_calls: Arc::new(AtomicUsize::new(0)),
            fail_fetch: vec![],
        }
    }

    #[tokio::test]
    async fn limit_is_applied_per_source_not_globally() {
        // Two sources of 4 items each, --limit 2 ⇒ 2 processed PER source (4
        // total), not 2 across the whole run.
        let adapters: Vec<Box<dyn SourceAdapter>> =
            vec![Box::new(fake_n("a", 4)), Box::new(fake_n("b", 4))];
        let out = run_all(&adapters, Some(2)).await;

        assert_eq!(out.stats.discovered, 4, "2 per source × 2 sources");
        assert_eq!(out.stats.accepted, 4);
        assert_eq!(
            out.prepared
                .iter()
                .filter(|p| p.entry.source == "a")
                .count(),
            2
        );
        assert_eq!(
            out.prepared
                .iter()
                .filter(|p| p.entry.source == "b")
                .count(),
            2
        );
    }

    #[tokio::test]
    async fn runs_all_adapters_and_merges_outcomes() {
        let adapters: Vec<Box<dyn SourceAdapter>> =
            vec![Box::new(fake("a", "1")), Box::new(fake("b", "2"))];
        let out = run_all(&adapters, None).await;

        assert_eq!(out.stats.accepted, 2, "both sources' items are accepted");
        assert!(out.prepared.iter().any(|p| p.entry.source == "a"));
        assert!(out.prepared.iter().any(|p| p.entry.source == "b"));
    }
}
