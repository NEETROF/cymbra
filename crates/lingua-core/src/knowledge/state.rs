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

//! The knowledge state: explicit statuses per `(studied language, lemma)`,
//! frequency-rank calibration, and token classification (designs D1, D2).
//!
//! Classification precedence, per candidate lemma: an explicit status always
//! wins over calibration; a lemma with no entry is implicitly known when its
//! rank is at or below the calibration threshold, otherwise unknown. Across a
//! token's candidate lemmas the token takes the most-known verdict (known if
//! **any** candidate is known — the learner's favour).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::analysis::language::StudiedLanguage;
use crate::analysis::percent::TokenClass;

use super::status::{KnownSource, Status};

/// Frequency ranks for the studied language (rank 1 = most frequent). Backed
/// by the pack's frequency table; a lemma absent from the table has no rank
/// and is never implicitly known. The real table arrives with
/// `add-lingua-data-pack`; tests use [`MapFrequencyRanks`].
pub trait FrequencyRanks {
    /// The 1-based frequency rank of a lemma, or `None` if it is not ranked.
    fn rank(&self, lemma: &str) -> Option<u32>;
}

/// In-memory ranks for tests and small fixtures.
#[derive(Debug, Default, Clone)]
pub struct MapFrequencyRanks {
    ranks: BTreeMap<String, u32>,
}

impl MapFrequencyRanks {
    /// Builds a rank table from `(lemma, rank)` pairs.
    pub fn from_pairs(pairs: impl IntoIterator<Item = (&'static str, u32)>) -> Self {
        Self {
            ranks: pairs.into_iter().map(|(l, r)| (l.to_owned(), r)).collect(),
        }
    }
}

impl FrequencyRanks for MapFrequencyRanks {
    fn rank(&self, lemma: &str) -> Option<u32> {
        self.ranks.get(lemma).copied()
    }
}

/// The learner's knowledge, serialisable and deterministic (ordered maps).
///
/// Holds only *explicit* statuses; implicit "known" by calibration is
/// recomputed on every classification, so moving the calibration slider is
/// free and reversible and never overwrites an explicit decision.
/// Keyed by studied language, then by lemma. Two nested maps rather than a
/// `(language, lemma)` tuple key so the state serialises to plain JSON
/// objects (a JSON object key must be a string; a tuple is not one), while
/// staying deterministic (ordered maps).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeState {
    /// Explicit statuses per language, keyed by lemma.
    statuses: BTreeMap<StudiedLanguage, BTreeMap<String, Status>>,
    /// Per-language calibration threshold: a lemma with no explicit status
    /// and a rank ≤ this value is implicitly known. Absent = 0 (nothing
    /// implicitly known).
    calibration: BTreeMap<StudiedLanguage, u32>,
}

impl KnowledgeState {
    /// A fresh, empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a lemma's explicit status.
    pub fn set_status(&mut self, lang: StudiedLanguage, lemma: &str, status: Status) {
        self.statuses
            .entry(lang)
            .or_default()
            .insert(lemma.to_owned(), status);
    }

    /// Removes any explicit status, returning the lemma to "new" (subject to
    /// calibration again).
    pub fn clear_status(&mut self, lang: StudiedLanguage, lemma: &str) {
        if let Some(per_lang) = self.statuses.get_mut(&lang) {
            per_lang.remove(lemma);
            if per_lang.is_empty() {
                self.statuses.remove(&lang);
            }
        }
    }

    /// The explicit status of a lemma, if any.
    pub fn explicit_status(&self, lang: StudiedLanguage, lemma: &str) -> Option<Status> {
        self.statuses
            .get(&lang)
            .and_then(|per_lang| per_lang.get(lemma))
            .copied()
    }

    /// Sets the calibration threshold for a language ("I know the N most
    /// common words"). A threshold of 0 makes nothing implicitly known.
    pub fn set_calibration(&mut self, lang: StudiedLanguage, threshold: u32) {
        self.calibration.insert(lang, threshold);
    }

    /// The calibration threshold for a language (0 if unset).
    pub fn calibration(&self, lang: StudiedLanguage) -> u32 {
        self.calibration.get(&lang).copied().unwrap_or(0)
    }

    /// Number of explicit statuses across all languages (for stats / tests).
    pub fn explicit_count(&self) -> usize {
        self.statuses.values().map(BTreeMap::len).sum()
    }

    /// The effective status of a single lemma: the explicit one if present,
    /// otherwise implicit `Known(Calibration)` when ranked at or below the
    /// threshold, otherwise `None` ("new"/unknown).
    pub fn resolve_lemma(
        &self,
        lang: StudiedLanguage,
        lemma: &str,
        ranks: &impl FrequencyRanks,
    ) -> Option<Status> {
        if let Some(explicit) = self.explicit_status(lang, lemma) {
            return Some(explicit);
        }
        match ranks.rank(lemma) {
            Some(rank) if rank <= self.calibration(lang) => {
                Some(Status::Known(KnownSource::Calibration))
            }
            _ => None,
        }
    }

    /// Classifies a token from its candidate lemmas (design D1: known if any
    /// candidate is known). Never returns
    /// [`TokenClass::ProperNounOutOfLexicon`] — proper-noun exclusion is the
    /// analysis layer's job, applied before this.
    pub fn classify(
        &self,
        lang: StudiedLanguage,
        candidates: &[&str],
        ranks: &impl FrequencyRanks,
    ) -> TokenClass {
        let mut best = Verdict::Unknown;
        for candidate in candidates {
            let verdict = match self.resolve_lemma(lang, candidate, ranks) {
                Some(Status::Known(_)) => Verdict::Known,
                Some(Status::Ignored) => Verdict::Ignored,
                Some(Status::Learning) => Verdict::Learning,
                None => Verdict::Unknown,
            };
            best = best.max(verdict);
            if best == Verdict::Known {
                break;
            }
        }
        best.into()
    }
}

/// Cross-candidate precedence: a token is as known as its most-known
/// candidate. `Known` > `Ignored` (both count known, `Known` preferred for a
/// stable class) > `Learning` > `Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Verdict {
    Unknown,
    Learning,
    Ignored,
    Known,
}

impl From<Verdict> for TokenClass {
    fn from(v: Verdict) -> Self {
        match v {
            Verdict::Unknown => TokenClass::Unknown,
            Verdict::Learning => TokenClass::Learning,
            Verdict::Ignored => TokenClass::Ignored,
            Verdict::Known => TokenClass::Known,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: StudiedLanguage = StudiedLanguage::English;

    fn ranks() -> MapFrequencyRanks {
        MapFrequencyRanks::from_pairs([
            ("the", 1),
            ("run", 500),
            ("code", 2_500),
            ("seldom", 5_100),
        ])
    }

    #[test]
    fn spec_calibration_at_startup() {
        let mut state = KnowledgeState::new();
        state.set_calibration(EN, 3_000);
        // No word marked: everything at or below rank 3,000 is known.
        assert_eq!(state.classify(EN, &["run"], &ranks()), TokenClass::Known);
        assert_eq!(state.classify(EN, &["code"], &ranks()), TokenClass::Known);
        // Beyond the threshold, unknown.
        assert_eq!(
            state.classify(EN, &["seldom"], &ranks()),
            TokenClass::Unknown
        );
    }

    #[test]
    fn spec_explicit_status_wins_over_calibration() {
        let mut state = KnowledgeState::new();
        state.set_calibration(EN, 3_000);
        state.set_status(EN, "run", Status::Learning); // rank 500, below threshold
        assert_eq!(state.classify(EN, &["run"], &ranks()), TokenClass::Learning);
    }

    #[test]
    fn spec_ambiguity_resolved_in_the_learners_favour() {
        let mut state = KnowledgeState::new();
        state.set_status(EN, "can", Status::Known(KnownSource::Manual));
        // The token `cans` offers candidates [can, cans]; `can` is known.
        assert_eq!(
            state.classify(EN, &["can", "cans"], &ranks()),
            TokenClass::Known
        );
    }

    #[test]
    fn multi_candidate_precedence_prefers_the_most_known() {
        let mut state = KnowledgeState::new();
        state.set_status(EN, "lead_v", Status::Learning);
        state.set_status(EN, "lead_n", Status::Known(KnownSource::Manual));
        // Learning + Known → Known wins.
        assert_eq!(
            state.classify(EN, &["lead_v", "lead_n"], &ranks()),
            TokenClass::Known
        );
        // Learning alone stays Learning; an unranked unknown stays Unknown.
        assert_eq!(
            state.classify(EN, &["lead_v"], &ranks()),
            TokenClass::Learning
        );
        assert_eq!(
            state.classify(EN, &["mystery"], &ranks()),
            TokenClass::Unknown
        );
    }

    #[test]
    fn ignored_counts_known_and_clearing_returns_to_calibration() {
        let mut state = KnowledgeState::new();
        state.set_status(EN, "seldom", Status::Ignored);
        assert_eq!(
            state.classify(EN, &["seldom"], &ranks()),
            TokenClass::Ignored
        );
        state.clear_status(EN, "seldom");
        // Back to "new": rank 5,100 with no calibration → unknown.
        assert_eq!(
            state.classify(EN, &["seldom"], &ranks()),
            TokenClass::Unknown
        );
        assert_eq!(state.explicit_count(), 0);
    }

    #[test]
    fn implicit_known_reports_calibration_provenance() {
        let mut state = KnowledgeState::new();
        state.set_calibration(EN, 1_000);
        assert_eq!(
            state.resolve_lemma(EN, "run", &ranks()),
            Some(Status::Known(KnownSource::Calibration))
        );
        // Unranked lemma is never implicitly known.
        assert_eq!(state.resolve_lemma(EN, "mystery", &ranks()), None);
    }

    #[test]
    fn state_roundtrips_through_serde_deterministically() {
        let mut state = KnowledgeState::new();
        state.set_calibration(EN, 3_000);
        state.set_status(EN, "run", Status::Learning);
        state.set_status(EN, "code", Status::Known(KnownSource::Import));
        let json = serde_json::to_string(&state).expect("serialise");
        let back: KnowledgeState = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(state, back);
        // Ordered maps ⇒ a second serialise is byte-identical.
        assert_eq!(json, serde_json::to_string(&back).expect("serialise again"));
    }
}
