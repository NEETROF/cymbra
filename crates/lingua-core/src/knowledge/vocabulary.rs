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

//! The reader's estimated vocabulary size — the figure comparable with the usual
//! "about 16,000 words at C2" estimates, which the CEFR ladder (a teaching list, not a
//! vocabulary) is not.
//!
//! Vocabulary-size tests sample the commonest words in bands of a thousand and apply the
//! share a learner knows in each band to the whole band. The estimate does the same with
//! what the engine already knows about the reader instead of a test.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::analysis::language::StudiedLanguage;

use super::level::CefrLevels;
use super::state::{FrequencyRanks, KnowledgeState};
use super::status::Status;

/// Width of a frequency band, in ranks.
pub const BAND: u32 = 1_000;

/// How many words of a band must say something about the reader before the band's known
/// share is applied in full to the rest of the band; below that, it is applied in
/// proportion (see [`KnowledgeState::vocabulary_estimate`]).
pub const MIN_EVIDENCE: usize = 25;

/// What an estimate rests on, so a surface can say so truthfully.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VocabularyBasis {
    /// A declared CEFR level (plus the words the reader marked), extrapolated.
    Level,
    /// The frequency calibration (plus the words the reader marked), extrapolated.
    Frequency,
    /// Only the words the reader marked: no level, no calibration, nothing extrapolated —
    /// the estimate is then the exact count of words marked known.
    #[default]
    Marked,
}

/// An estimated vocabulary size over a pack's dictionary words.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct VocabularyEstimate {
    /// Estimated number of known words among `universe`; never below `confirmed`.
    pub estimated: usize,
    /// Words the reader explicitly knows: marked known, validated in review or
    /// confirmed by reading.
    pub confirmed: usize,
    /// The dictionary words the estimate is taken over (ignored words left out).
    pub universe: usize,
    /// What the estimate rests on.
    pub basis: VocabularyBasis,
}

#[derive(Default)]
struct Band {
    words: usize,
    evidence: usize,
    known: usize,
    confirmed: usize,
}

impl Band {
    /// The band's estimated known words: those known (for sure or presumed) plus the rest
    /// of the band at the known share, where the share is taken over at least
    /// [`MIN_EVIDENCE`] words. A thin band's share is thus shrunk rather than switched
    /// off, so the estimate has no step: one more word of evidence moves it by less than
    /// a word's worth of the band, and a word marked not known never raises it.
    fn estimated(&self) -> usize {
        let rest = self.words - self.evidence;
        let over = self.evidence.max(MIN_EVIDENCE);
        // known + round(rest × known / over), in integers.
        self.known + (2 * rest * self.known + over) / (2 * over)
    }
}

impl KnowledgeState {
    /// Estimates how many of `words` — dictionary words with their frequency rank — the
    /// reader knows.
    ///
    /// Words are grouped in frequency bands of [`BAND`] ranks. In a band, the words the
    /// engine holds an opinion on are the evidence: an explicit status, and — with a
    /// declared CEFR level — every word that has a CEFR level (presumed known below the
    /// level, not at or above it), or — without one — every word (known up to the
    /// frequency calibration). The band's known words (for sure or presumed) count, and
    /// its other words count at the known share of the evidence — in full once the band
    /// holds [`MIN_EVIDENCE`] such words, in proportion before. Ignored words are left
    /// out, as a vocabulary size counts neither names nor noise. Integer-only, so the
    /// figure is identical on every target.
    pub fn vocabulary_estimate<'a>(
        &self,
        lang: StudiedLanguage,
        words: impl IntoIterator<Item = (&'a str, u32)>,
        lexis: &(impl FrequencyRanks + CefrLevels),
    ) -> VocabularyEstimate {
        let declared = self.declared_level(lang).is_some();
        let basis = if declared {
            VocabularyBasis::Level
        } else if self.calibration(lang) > 0 {
            VocabularyBasis::Frequency
        } else {
            VocabularyBasis::Marked
        };
        let mut bands: BTreeMap<u32, Band> = BTreeMap::new();
        for (lemma, rank) in words {
            let explicit = self.explicit_status(lang, lemma);
            if explicit == Some(Status::Ignored) {
                continue;
            }
            let band = bands.entry(rank.saturating_sub(1) / BAND).or_default();
            band.words += 1;
            let known = match explicit {
                Some(Status::Known(_)) => {
                    band.confirmed += 1;
                    Some(true)
                }
                Some(_) => Some(false),
                // A declared level says nothing about a word the CEFR lists do not hold.
                None if declared && lexis.level(lemma).is_none() => None,
                None => Some(matches!(
                    self.resolve_lemma(lang, lemma, lexis),
                    Some(status) if status.counts_as_known()
                )),
            };
            if let Some(known) = known {
                band.evidence += 1;
                band.known += usize::from(known);
            }
        }
        bands.values().fold(
            VocabularyEstimate {
                basis,
                ..VocabularyEstimate::default()
            },
            |mut total, band| {
                total.universe += band.words;
                total.confirmed += band.confirmed;
                total.estimated += band.estimated();
                total
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::level::CefrLevel;
    use crate::knowledge::state::MapFrequencyRanks;
    use crate::knowledge::status::KnownSource;

    const EN: StudiedLanguage = StudiedLanguage::English;

    fn word(rank: u32) -> &'static str {
        Box::leak(format!("w{rank}").into_boxed_str())
    }

    /// `n` words ranked 1..=n; the first `leveled` carry a CEFR level, A1 for the first
    /// half of them and B2 for the rest.
    fn words(n: u32, leveled: u32) -> (Vec<(&'static str, u32)>, MapFrequencyRanks) {
        let words: Vec<(&'static str, u32)> = (1..=n).map(|r| (word(r), r)).collect();
        let lexis = MapFrequencyRanks::from_pairs(words.clone()).with_levels(
            words.iter().take(leveled as usize).map(|&(w, r)| {
                (
                    w,
                    if r <= leveled / 2 {
                        CefrLevel::A1
                    } else {
                        CefrLevel::B2
                    },
                )
            }),
        );
        (words, lexis)
    }

    fn at_b1() -> KnowledgeState {
        let mut state = KnowledgeState::new();
        state.set_declared_level(EN, CefrLevel::B1);
        state
    }

    #[test]
    fn a_bands_known_share_is_extrapolated_to_all_its_words() {
        let (words, lexis) = words(100, 40);
        // 40 leveled words: the 20 A1 presumed known, the 20 B2 not — half of 100 words.
        let estimate = at_b1().vocabulary_estimate(EN, words, &lexis);
        assert_eq!(
            estimate,
            VocabularyEstimate {
                estimated: 50,
                confirmed: 0,
                universe: 100,
                basis: VocabularyBasis::Level,
            }
        );
    }

    #[test]
    fn a_thin_bands_share_is_shrunk_not_switched_off() {
        let (words, lexis) = words(100, 10);
        // 10 words of evidence, 5 known: the 90 others count at 5 / 25, not 5 / 10.
        assert_eq!(
            at_b1().vocabulary_estimate(EN, words, &lexis).estimated,
            5 + 18
        );
    }

    #[test]
    fn a_word_marked_not_known_never_raises_the_estimate() {
        let (words, lexis) = words(100, 10);
        let mut state = at_b1();
        let mut last = state
            .vocabulary_estimate(EN, words.clone(), &lexis)
            .estimated;
        // Carding unleveled words one by one walks the evidence across MIN_EVIDENCE.
        for r in 11..=60 {
            state.set_status(EN, word(r), Status::Learning);
            let now = state
                .vocabulary_estimate(EN, words.clone(), &lexis)
                .estimated;
            assert!(now <= last, "rank {r}: {now} > {last}");
            last = now;
        }
    }

    #[test]
    fn explicit_statuses_are_evidence_and_ignored_words_are_left_out() {
        let (words, lexis) = words(100, 0);
        let mut state = at_b1();
        for r in 1..=20 {
            state.set_status(EN, word(r), Status::Known(KnownSource::Manual));
        }
        for r in 21..=30 {
            state.set_status(EN, word(r), Status::Learning);
        }
        state.set_status(EN, word(31), Status::Ignored);
        let estimate = state.vocabulary_estimate(EN, words, &lexis);
        // 30 words of evidence, 20 known, over the 99 words left: 20 + round(69 × 20 / 30).
        assert_eq!(
            estimate,
            VocabularyEstimate {
                estimated: 66,
                confirmed: 20,
                universe: 99,
                basis: VocabularyBasis::Level,
            }
        );
    }

    #[test]
    fn without_a_declared_level_the_calibration_decides_band_by_band() {
        let (words, lexis) = words(2_000, 0);
        let mut state = KnowledgeState::new();
        state.set_calibration(EN, 1_500);
        let estimate = state.vocabulary_estimate(EN, words, &lexis);
        assert_eq!(estimate.estimated, 1_500);
        assert_eq!(estimate.universe, 2_000);
        assert_eq!(estimate.basis, VocabularyBasis::Frequency);
    }

    #[test]
    fn with_neither_level_nor_calibration_the_estimate_is_the_words_marked_known() {
        let (words, lexis) = words(100, 40);
        let mut state = KnowledgeState::new();
        state.set_calibration(EN, 0);
        for r in [3, 50, 99] {
            state.set_status(EN, word(r), Status::Known(KnownSource::Manual));
        }
        let estimate = state.vocabulary_estimate(EN, words.clone(), &lexis);
        assert_eq!((estimate.estimated, estimate.confirmed), (3, 3));
        assert_eq!(estimate.basis, VocabularyBasis::Marked);
        assert_eq!(
            KnowledgeState::new().vocabulary_estimate(EN, Vec::new(), &lexis),
            VocabularyEstimate::default()
        );
    }
}
