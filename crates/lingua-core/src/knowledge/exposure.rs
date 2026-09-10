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

//! Exposure counters (design D5).
//!
//! Per `(studied language, lemma)`: how often the lemma has been encountered,
//! where it was last seen and when. Exposure **never** changes a status in
//! v1 — LingQ's auto-known-on-page-turn is the category's most-hated
//! behaviour and is deliberately rejected. These counts are input data for
//! the future "known"-from-exposure inference, fed by agent ingestion and by
//! reading. The clock lives outside the core: callers pass the timestamp, so
//! the module stays deterministic and host-testable.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::analysis::language::StudiedLanguage;

/// The exposure record for one lemma.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exposure {
    /// Total occurrences encountered across all sources.
    pub occurrences: u32,
    /// A tag for the source of the most recent encounter (e.g. a URL, an
    /// agent session id). Opaque to the core.
    pub last_source: String,
    /// Unix-epoch seconds of the most recent encounter, supplied by the
    /// caller.
    pub last_seen: i64,
}

/// Exposure counters for the whole knowledge model, deterministic (ordered
/// maps) and serialisable. Nested by language then lemma so the state
/// serialises to plain JSON objects (see [`super::state::KnowledgeState`]).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposureCounters {
    counters: BTreeMap<StudiedLanguage, BTreeMap<String, Exposure>>,
}

impl ExposureCounters {
    /// Empty counters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records `count` new occurrences of `lemma`, updating the last source
    /// and timestamp. Returns nothing about status — recording exposure
    /// never changes a status (design D5).
    pub fn record(
        &mut self,
        lang: StudiedLanguage,
        lemma: &str,
        count: u32,
        source: &str,
        timestamp: i64,
    ) {
        let entry = self
            .counters
            .entry(lang)
            .or_default()
            .entry(lemma.to_owned())
            .or_insert(Exposure {
                occurrences: 0,
                last_source: String::new(),
                last_seen: 0,
            });
        entry.occurrences += count;
        entry.last_source = source.to_owned();
        entry.last_seen = timestamp;
    }

    /// The exposure record for a lemma, if it has ever been seen.
    pub fn get(&self, lang: StudiedLanguage, lemma: &str) -> Option<&Exposure> {
        self.counters
            .get(&lang)
            .and_then(|per_lang| per_lang.get(lemma))
    }

    /// Number of distinct lemmas with any exposure across all languages.
    pub fn tracked_len(&self) -> usize {
        self.counters.values().map(BTreeMap::len).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: StudiedLanguage = StudiedLanguage::English;

    #[test]
    fn spec_ingesting_an_agent_session_only_counts() {
        let mut counters = ExposureCounters::new();
        counters.record(EN, "conundrum", 2, "claude:session-42", 1_700_000_000);
        let exposure = counters.get(EN, "conundrum").expect("recorded");
        assert_eq!(exposure.occurrences, 2);
        assert_eq!(exposure.last_source, "claude:session-42");
        assert_eq!(exposure.last_seen, 1_700_000_000);
        // The record carries no status — exposure never sets one.
    }

    #[test]
    fn repeated_encounters_accumulate_and_refresh_source() {
        let mut counters = ExposureCounters::new();
        counters.record(EN, "run", 3, "https://a.example", 1_000);
        counters.record(EN, "run", 2, "https://b.example", 2_000);
        let exposure = counters.get(EN, "run").expect("recorded");
        assert_eq!(exposure.occurrences, 5);
        assert_eq!(exposure.last_source, "https://b.example");
        assert_eq!(exposure.last_seen, 2_000);
        assert_eq!(counters.tracked_len(), 1);
    }

    #[test]
    fn unseen_lemma_has_no_record() {
        let counters = ExposureCounters::new();
        assert_eq!(counters.get(EN, "never"), None);
    }
}
