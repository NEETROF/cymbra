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

use super::level::{CefrLevel, CefrLevels};
use super::state::{FrequencyRanks, KnowledgeState};
use super::status::Status;

/// Width of a frequency band, in ranks.
pub const BAND: u32 = 1_000;

/// How many words of a band must say something about the reader before the band's known
/// share is applied in full to the rest of the band; below that, it is applied in
/// proportion (see [`KnowledgeState::vocabulary_estimate`]).
pub const MIN_EVIDENCE: usize = 25;

/// The vocabulary typical of an English reader at each CEFR level, A1 to C2: what
/// [`level_vocabulary`] gives over English's CEFR lists. A pack whose levels are estimated
/// from frequency cannot say what a reader of a level knows, so its ladder shows these
/// figures, said to be English's (fix-lingua-spanish-ladder-estimates).
///
/// Frozen, so a Spanish ladder shows the same figures whatever packs the engine holds — a
/// reader whose native language is English never holds the en-fr pack — and whatever the
/// next English dictionary update does (generalise-lingua-native-language D6). Provenance:
/// `level_vocabulary` over the en-fr pack `2026.09.26+627146e`, frozen on 2026-10-07.
/// English's own ladder keeps computing its figures from its pack.
pub const ENGLISH_TYPICAL_VOCABULARY: [usize; 6] = [0, 1_292, 3_359, 7_988, 16_326, 20_556];

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

/// What the engine holds about one word of the estimate's universe.
enum Opinion {
    /// Left out of the universe, as a vocabulary size counts neither names nor noise.
    Ignored,
    /// In the universe, but no evidence either way.
    Silent,
    /// Known — `confirmed` when the reader said so rather than presumed.
    Known {
        confirmed: bool,
    },
    NotKnown,
}

/// Groups `words` in frequency bands and sums each band's estimate (see [`Band`]), the
/// `opinion` on each word deciding what it counts as.
fn estimate_over<'a>(
    words: impl IntoIterator<Item = (&'a str, u32)>,
    basis: VocabularyBasis,
    mut opinion: impl FnMut(&str) -> Opinion,
) -> VocabularyEstimate {
    let mut bands: BTreeMap<u32, Band> = BTreeMap::new();
    for (lemma, rank) in words {
        let opinion = opinion(lemma);
        if matches!(opinion, Opinion::Ignored) {
            continue;
        }
        let band = bands.entry(rank.saturating_sub(1) / BAND).or_default();
        band.words += 1;
        match opinion {
            Opinion::Ignored | Opinion::Silent => {}
            Opinion::Known { confirmed } => {
                band.evidence += 1;
                band.known += 1;
                band.confirmed += usize::from(confirmed);
            }
            Opinion::NotKnown => band.evidence += 1,
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

/// The vocabulary size typical of a reader at `level`, over `words` — dictionary words
/// with their frequency rank.
///
/// The CEFR lists are teaching lists: they stop at the words a course introduces, so their
/// running total stays far below the vocabulary a reader of the level actually has. This is
/// [`KnowledgeState::vocabulary_estimate`] for a reader who declared `level` and marked
/// nothing: every listed word below the level known, none at or above it, each band's
/// known share among its leveled words extrapolated to the band. It depends on the pack
/// only, not on the reader — and, the lowest level presuming nothing, it is 0 at A1.
pub fn level_vocabulary<'a>(
    level: CefrLevel,
    words: impl IntoIterator<Item = (&'a str, u32)>,
    lexis: &impl CefrLevels,
) -> usize {
    estimate_over(words, VocabularyBasis::Level, |lemma| {
        match lexis.level(lemma) {
            Some(l) if l < level => Opinion::Known { confirmed: false },
            Some(_) => Opinion::NotKnown,
            None => Opinion::Silent,
        }
    })
    .estimated
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
        estimate_over(words, basis, |lemma| {
            match self.explicit_status(lang, lemma) {
                Some(Status::Ignored) => Opinion::Ignored,
                Some(Status::Known(_)) => Opinion::Known { confirmed: true },
                Some(_) => Opinion::NotKnown,
                // A declared level says nothing about a word the CEFR lists do not hold.
                None if declared && lexis.level(lemma).is_none() => Opinion::Silent,
                None => match self.resolve_lemma(lang, lemma, lexis) {
                    Some(status) if status.counts_as_known() => Opinion::Known { confirmed: false },
                    _ => Opinion::NotKnown,
                },
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decks::Deck;
    use crate::knowledge::state::MapFrequencyRanks;
    use crate::knowledge::status::KnownSource;

    const EN: StudiedLanguage = StudiedLanguage::English;

    #[test]
    fn english_typical_vocabularies_are_frozen() {
        // Pinned on purpose, and never tied to the live en-fr figures: an English dictionary
        // update must not move another language's ladder (generalise-lingua-native-language D6).
        assert_eq!(
            ENGLISH_TYPICAL_VOCABULARY,
            [0, 1_292, 3_359, 7_988, 16_326, 20_556]
        );
        assert_eq!(ENGLISH_TYPICAL_VOCABULARY.len(), CefrLevel::ALL.len());
    }

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

    #[test]
    fn a_levels_vocabulary_extrapolates_the_listed_words_below_it() {
        let (words, lexis) = words(100, 40);
        assert_eq!(level_vocabulary(CefrLevel::A1, words.clone(), &lexis), 0);
        // B1: the 20 A1 words known, the 20 B2 not — half of 100 words.
        assert_eq!(level_vocabulary(CefrLevel::B1, words.clone(), &lexis), 50);
        assert_eq!(level_vocabulary(CefrLevel::B2, words.clone(), &lexis), 50);
        // C1: every listed word known — the whole band.
        assert_eq!(level_vocabulary(CefrLevel::C1, words, &lexis), 100);
    }

    #[test]
    fn a_lemma_left_unseeded_keeps_its_presumption_and_its_place_in_the_estimate() {
        // seed-lingua-decks-with-glossed-lemmas D3: a lemma the pack does not gloss is skipped
        // by a level's seeding, and stays a word of its language — no status, presumed known
        // below the declared level, still counted by the estimate, the estimate unmoved.
        let (words, lexis) = words(100, 40);
        let state = at_b1();
        let before = state.vocabulary_estimate(EN, words.clone(), &lexis);
        // w1 and w2 are A1 (below B1), w21 and w22 B2 (above): w1 and w21 have no gloss.
        let (a1_bare, a1, b2_bare, b2) = (word(1), word(2), word(21), word(22));
        let mut deck = Deck::new();
        let added = deck.seed_lemmas(
            EN,
            [
                (a1_bare, None),
                (a1, Some("un")),
                (b2_bare, None),
                (b2, Some("deux")),
            ],
            "fr",
            &state,
            4,
            0,
        );
        assert_eq!(added, 2);
        for lemma in [a1_bare, b2_bare] {
            assert!(deck.get(EN, lemma).is_none(), "{lemma}");
            assert_eq!(state.explicit_status(EN, lemma), None, "{lemma}");
        }
        assert_eq!(
            state.resolve_lemma(EN, a1_bare, &lexis),
            Some(Status::Known(KnownSource::Calibration)),
            "presumed known below the declared level"
        );
        assert_eq!(state.resolve_lemma(EN, b2_bare, &lexis), None, "to learn");
        let after = state.vocabulary_estimate(EN, words, &lexis);
        assert_eq!(after, before);
        assert_eq!(after.universe, 100, "every word counted, glossed or not");
    }

    #[test]
    fn a_levels_vocabulary_is_a_fresh_reader_declared_at_it() {
        let (words, lexis) = words(3_000, 1_200);
        for level in CefrLevel::ALL {
            let mut state = KnowledgeState::new();
            state.set_declared_level(EN, level);
            assert_eq!(
                level_vocabulary(level, words.clone(), &lexis),
                state
                    .vocabulary_estimate(EN, words.clone(), &lexis)
                    .estimated,
                "{level:?}"
            );
        }
    }
}
