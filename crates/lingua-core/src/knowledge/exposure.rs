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
    /// UTC day number (whole days since the Unix epoch) of the most recent
    /// encounter. Kept so distinct reading days can be counted without storing
    /// every day. `#[serde(default)]` so an older backup restores with 0.
    #[serde(default)]
    pub last_day: i64,
    /// Number of distinct UTC days on which the lemma has been encountered. It
    /// is the promotion signal (`add-lingua-cefr-levels`): "read repeatedly over
    /// time", not "seen many times in one page". Advanced only when a record
    /// lands on a later day than `last_day`, so it is exact for the
    /// forward-in-time reading stream and bounded to two extra fields; an
    /// out-of-order earlier record does not inflate it. `#[serde(default)]` so
    /// an older backup restores with 0.
    #[serde(default)]
    pub distinct_days: u32,
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
                last_day: 0,
                distinct_days: 0,
            });
        entry.occurrences += count;
        entry.last_source = source.to_owned();
        entry.last_seen = timestamp;
        // Whole UTC days since the epoch. A first record (distinct_days == 0) or
        // one on a later day than the last advances the distinct-day count; a
        // same-day or out-of-order-earlier record does not.
        let day = timestamp.div_euclid(86_400);
        if entry.distinct_days == 0 || day > entry.last_day {
            entry.distinct_days = entry.distinct_days.saturating_add(1);
            entry.last_day = day;
        }
    }

    /// Keep only the lemmas `keep` accepts. Counters exist for one purpose — confirming a
    /// presumed word by repeated reading — so one that can no longer be promoted (the reader
    /// decided about the word, or the declared level moved) is dead weight in a store that
    /// every surface loads.
    pub fn retain(&mut self, lang: StudiedLanguage, mut keep: impl FnMut(&str, &Exposure) -> bool) {
        if let Some(per_lang) = self.counters.get_mut(&lang) {
            per_lang.retain(|lemma, exposure| keep(lemma, exposure));
            if per_lang.is_empty() {
                self.counters.remove(&lang);
            }
        }
    }

    /// Bound the counters to the `max` lemmas seen most recently, so a reader's store cannot
    /// grow with every new word they ever meet. Dropping the least recent loses the least:
    /// promotion needs reading spread over days, which a lemma not seen in a long time is
    /// not accumulating.
    pub fn cap_by_recency(&mut self, lang: StudiedLanguage, max: usize) {
        let Some(per_lang) = self.counters.get_mut(&lang) else {
            return;
        };
        if per_lang.len() <= max {
            return;
        }
        let mut by_recency: Vec<(i64, String)> = per_lang
            .iter()
            .map(|(lemma, exp)| (exp.last_seen, lemma.clone()))
            .collect();
        // Most recent first; the lemma breaks ties so the result never depends on map order.
        by_recency.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        for (_, lemma) in by_recency.into_iter().skip(max) {
            per_lang.remove(&lemma);
        }
    }

    /// The exposure record for a lemma, if it has ever been seen.
    pub fn get(&self, lang: StudiedLanguage, lemma: &str) -> Option<&Exposure> {
        self.counters
            .get(&lang)
            .and_then(|per_lang| per_lang.get(lemma))
    }

    /// Every recorded lemma for a language, with its exposure, in deterministic
    /// (lemma) order — the source the exposure-promotion pass folds over.
    pub fn lemmas(&self, lang: StudiedLanguage) -> impl Iterator<Item = (&str, &Exposure)> + '_ {
        self.counters
            .get(&lang)
            .into_iter()
            .flat_map(|per_lang| per_lang.iter().map(|(lemma, exp)| (lemma.as_str(), exp)))
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

    const DAY: i64 = 86_400;

    #[test]
    fn distinct_days_count_days_not_occurrences() {
        let mut counters = ExposureCounters::new();
        // Three encounters on day 0, one on day 1: two distinct days.
        counters.record(EN, "run", 1, "p1", 10);
        counters.record(EN, "run", 5, "p2", 20);
        counters.record(EN, "run", 1, "p3", DAY + 5);
        let exp = counters.get(EN, "run").expect("recorded");
        assert_eq!(exp.occurrences, 7);
        assert_eq!(exp.distinct_days, 2);
    }

    #[test]
    fn an_out_of_order_earlier_record_does_not_inflate_distinct_days() {
        let mut counters = ExposureCounters::new();
        counters.record(EN, "run", 1, "p", 3 * DAY);
        counters.record(EN, "run", 1, "p", DAY); // earlier day, arrives later
        let exp = counters.get(EN, "run").expect("recorded");
        assert_eq!(exp.distinct_days, 1);
    }

    #[test]
    fn lemmas_iterates_in_lemma_order() {
        let mut counters = ExposureCounters::new();
        counters.record(EN, "zebra", 1, "p", 0);
        counters.record(EN, "apple", 1, "p", 0);
        let seen: Vec<&str> = counters.lemmas(EN).map(|(l, _)| l).collect();
        assert_eq!(seen, ["apple", "zebra"]);
    }

    #[test]
    fn retain_drops_what_can_no_longer_be_promoted() {
        let mut counters = ExposureCounters::new();
        counters.record(StudiedLanguage::English, "cat", 1, "r", 1_000);
        counters.record(StudiedLanguage::English, "run", 1, "r", 2_000);

        counters.retain(StudiedLanguage::English, |lemma, _| lemma == "cat");

        assert!(counters.get(StudiedLanguage::English, "cat").is_some());
        assert!(counters.get(StudiedLanguage::English, "run").is_none());

        // Emptying a language drops the language itself, so the state serialises to {}.
        counters.retain(StudiedLanguage::English, |_, _| false);
        assert_eq!(counters.tracked_len(), 0);
    }

    #[test]
    fn cap_keeps_the_most_recently_read() {
        let mut counters = ExposureCounters::new();
        for (lemma, seen) in [("old", 1_000), ("newer", 3_000), ("newest", 5_000)] {
            counters.record(StudiedLanguage::English, lemma, 1, "r", seen);
        }

        counters.cap_by_recency(StudiedLanguage::English, 2);

        assert_eq!(counters.tracked_len(), 2);
        assert!(counters.get(StudiedLanguage::English, "old").is_none());
        assert!(counters.get(StudiedLanguage::English, "newest").is_some());
    }

    #[test]
    fn cap_above_the_count_changes_nothing() {
        let mut counters = ExposureCounters::new();
        counters.record(StudiedLanguage::English, "cat", 1, "r", 1_000);

        counters.cap_by_recency(StudiedLanguage::English, 10);

        assert_eq!(counters.tracked_len(), 1);
    }
}
