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

//! The shared crawl run loop, used by both the CLI and the TUI.
//!
//! Prepares and runs each adapter through the license-first orchestrator
//! (deduplicating across sources), optionally emitting [`ProgressEvent`]s so a
//! front-end can show live progress, and returns the merged outcome.

use tokio::sync::mpsc::UnboundedSender;

use crate::crawl::{CrawlOutcome, CrawlStats, Orchestrator};
use crate::sources::SourceAdapter;

/// Progress emitted as the run advances (consumed by the TUI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressEvent {
    Started(String),
    Finished(String, CrawlStats),
    AllDone,
}

/// Runs every adapter, folding results into one [`CrawlOutcome`]. Per-source
/// prepare/discover failures are logged and skipped, never fatal.
pub async fn run_all(
    adapters: &[Box<dyn SourceAdapter>],
    limit: Option<usize>,
    tx: Option<UnboundedSender<ProgressEvent>>,
) -> CrawlOutcome {
    let mut orchestrator = Orchestrator::new();
    let mut merged = CrawlOutcome::default();

    for adapter in adapters {
        emit(&tx, ProgressEvent::Started(adapter.name().to_string()));
        if let Err(e) = adapter.prepare().await {
            tracing::warn!(source = adapter.name(), error = %e, "prepare failed; skipping");
            emit(
                &tx,
                ProgressEvent::Finished(adapter.name().to_string(), CrawlStats::default()),
            );
            continue;
        }
        let out = orchestrator.run(adapter.as_ref(), limit).await;
        emit(
            &tx,
            ProgressEvent::Finished(adapter.name().to_string(), out.stats.clone()),
        );
        merge(&mut merged, out);
    }
    emit(&tx, ProgressEvent::AllDone);
    merged
}

fn emit(tx: &Option<UnboundedSender<ProgressEvent>>, event: ProgressEvent) {
    if let Some(tx) = tx {
        let _ = tx.send(event);
    }
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

    // A distinct score per id (unique content ⇒ distinct hash, so the two fake
    // sources are not deduplicated against each other).
    fn score_for(id: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<score-partwise version="4.0"><work><work-title>{id}</work-title></work>
<part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<key><fifths>0</fifths></key><time><beats>4</beats><beat-type>4</beat-type></time>
<clef><sign>G</sign><line>2</line></clef></attributes>
<note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note></measure></part></score-partwise>"#
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

    #[tokio::test]
    async fn runs_all_adapters_and_emits_progress() {
        let adapters: Vec<Box<dyn SourceAdapter>> =
            vec![Box::new(fake("a", "1")), Box::new(fake("b", "2"))];
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let out = run_all(&adapters, None, Some(tx)).await;

        assert_eq!(out.stats.accepted, 2);

        let mut events = Vec::new();
        while let Ok(e) = rx.try_recv() {
            events.push(e);
        }
        assert_eq!(events.first(), Some(&ProgressEvent::Started("a".into())));
        assert_eq!(events.last(), Some(&ProgressEvent::AllDone));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, ProgressEvent::Finished(n, s) if n == "b" && s.accepted == 1))
        );
    }
}
