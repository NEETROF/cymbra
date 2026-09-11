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

use super::exposure::ExposureCounters;
use super::level::{CefrLevel, CefrLevels};
use super::status::{KnownSource, Status};

/// Frequency ranks for the studied language (rank 1 = most frequent). Backed
/// by the pack's frequency table; a lemma absent from the table has no rank
/// and is never implicitly known. The real table arrives with
/// `add-lingua-data-pack`; tests use [`MapFrequencyRanks`].
pub trait FrequencyRanks {
    /// The 1-based frequency rank of a lemma, or `None` if it is not ranked.
    fn rank(&self, lemma: &str) -> Option<u32>;
}

/// In-memory ranks (and, optionally, CEFR levels) for tests and small
/// fixtures. Implements both [`FrequencyRanks`] and [`CefrLevels`] so one
/// object satisfies the resolver's bound, exactly as the real [`Pack`] does.
///
/// [`Pack`]: crate::packs::Pack
#[derive(Debug, Default, Clone)]
pub struct MapFrequencyRanks {
    ranks: BTreeMap<String, u32>,
    levels: BTreeMap<String, CefrLevel>,
}

impl MapFrequencyRanks {
    /// Builds a rank table from `(lemma, rank)` pairs (no CEFR levels).
    pub fn from_pairs(pairs: impl IntoIterator<Item = (&'static str, u32)>) -> Self {
        Self {
            ranks: pairs.into_iter().map(|(l, r)| (l.to_owned(), r)).collect(),
            levels: BTreeMap::new(),
        }
    }

    /// Attaches CEFR levels to some lemmas, for the level-aware tests.
    pub fn with_levels(
        mut self,
        pairs: impl IntoIterator<Item = (&'static str, CefrLevel)>,
    ) -> Self {
        self.levels = pairs
            .into_iter()
            .map(|(l, lvl)| (l.to_owned(), lvl))
            .collect();
        self
    }
}

impl FrequencyRanks for MapFrequencyRanks {
    fn rank(&self, lemma: &str) -> Option<u32> {
        self.ranks.get(lemma).copied()
    }
}

impl CefrLevels for MapFrequencyRanks {
    fn level(&self, lemma: &str) -> Option<CefrLevel> {
        self.levels.get(lemma).copied()
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
    /// implicitly known). Used only when no CEFR level is declared for the
    /// language (the fallback for pairs without CEFR data).
    calibration: BTreeMap<StudiedLanguage, u32>,
    /// Per-language declared CEFR level (`add-lingua-cefr-levels`). When set,
    /// it replaces the frequency-rank calibration: a lemma with no explicit
    /// status is presumed known iff its CEFR level is strictly below this one;
    /// a lemma at or above the level, or with no CEFR level at all, is unknown.
    /// Absent = no declared level (fall back to `calibration`). `#[serde(default)]`
    /// so an older backup restores cleanly.
    #[serde(default)]
    declared_level: BTreeMap<StudiedLanguage, CefrLevel>,
    /// Last-change time (epoch millis) per lemma, for cross-device last-write-
    /// wins sync (`add-lingua-connected-clients`). Absent = 0 (a pre-sync entry,
    /// which loses to any real timestamp). `#[serde(default)]` so an older backup
    /// without this map restores cleanly. A cleared lemma keeps its timestamp as
    /// a tombstone so a stale re-add cannot win.
    #[serde(default)]
    updated: BTreeMap<StudiedLanguage, BTreeMap<String, i64>>,
}

/// One exported status for the sync outbox — a lemma's current status and the
/// time it last changed on this device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusRecord {
    pub language: StudiedLanguage,
    pub lemma: String,
    pub status: Status,
    pub updated_at: i64,
}

impl KnowledgeState {
    /// A fresh, empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a lemma's explicit status (no sync timestamp — internal/test use;
    /// the timestamp defaults to 0, which loses to any real one in LWW).
    pub fn set_status(&mut self, lang: StudiedLanguage, lemma: &str, status: Status) {
        self.statuses
            .entry(lang)
            .or_default()
            .insert(lemma.to_owned(), status);
    }

    /// Sets a lemma's explicit status and stamps its sync timestamp (epoch
    /// millis). The mutation path the surfaces use, so a change carries a time
    /// the sync outbox and cross-device LWW can order it by.
    pub fn set_status_at(
        &mut self,
        lang: StudiedLanguage,
        lemma: &str,
        status: Status,
        at_ms: i64,
    ) {
        self.set_status(lang, lemma, status);
        self.updated
            .entry(lang)
            .or_default()
            .insert(lemma.to_owned(), at_ms);
    }

    /// Removes any explicit status, returning the lemma to "new" (subject to
    /// calibration again). Its sync timestamp is left as a tombstone.
    pub fn clear_status(&mut self, lang: StudiedLanguage, lemma: &str) {
        if let Some(per_lang) = self.statuses.get_mut(&lang) {
            per_lang.remove(lemma);
            if per_lang.is_empty() {
                self.statuses.remove(&lang);
            }
        }
    }

    /// The last-change time (epoch millis) of a lemma's status; 0 if never
    /// stamped (a pre-sync entry).
    pub fn status_updated_at(&self, lang: StudiedLanguage, lemma: &str) -> i64 {
        self.updated
            .get(&lang)
            .and_then(|m| m.get(lemma))
            .copied()
            .unwrap_or(0)
    }

    /// Every explicit status with its sync timestamp, in deterministic order —
    /// the outbox source for a full push (e.g. first sign-in).
    pub fn export_statuses(&self) -> Vec<StatusRecord> {
        let mut out = Vec::new();
        for (&language, per_lang) in &self.statuses {
            for (lemma, &status) in per_lang {
                out.push(StatusRecord {
                    language,
                    lemma: lemma.clone(),
                    status,
                    updated_at: self.status_updated_at(language, lemma),
                });
            }
        }
        out
    }

    /// Apply an incoming status change under last-write-wins: a strictly-newer
    /// change wins; an equal timestamp also applies (the server already resolved
    /// ties, so its value is authoritative and re-applying is a no-op); an older
    /// one is dropped. `incoming = None` clears the lemma (the `"cleared"` wire
    /// value). Returns whether local state changed. The timestamp is always
    /// advanced to `at_ms` when the change is accepted, so a later stale op loses.
    pub fn apply_status_lww(
        &mut self,
        lang: StudiedLanguage,
        lemma: &str,
        incoming: Option<Status>,
        at_ms: i64,
    ) -> bool {
        if at_ms < self.status_updated_at(lang, lemma) {
            return false;
        }
        let before = self.explicit_status(lang, lemma);
        match incoming {
            Some(status) => self.set_status(lang, lemma, status),
            None => self.clear_status(lang, lemma),
        }
        self.updated
            .entry(lang)
            .or_default()
            .insert(lemma.to_owned(), at_ms);
        before != incoming
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

    /// Declares the user's CEFR level for a language ("I'm B2"). Once set, it
    /// governs presumed-known instead of the frequency calibration.
    pub fn set_declared_level(&mut self, lang: StudiedLanguage, level: CefrLevel) {
        self.declared_level.insert(lang, level);
    }

    /// The declared CEFR level for a language, if any.
    pub fn declared_level(&self, lang: StudiedLanguage) -> Option<CefrLevel> {
        self.declared_level.get(&lang).copied()
    }

    /// Clears the declared level, returning the language to frequency-rank
    /// calibration.
    pub fn clear_declared_level(&mut self, lang: StudiedLanguage) {
        self.declared_level.remove(&lang);
    }

    /// Number of explicit statuses across all languages (for stats / tests).
    pub fn explicit_count(&self) -> usize {
        self.statuses.values().map(BTreeMap::len).sum()
    }

    /// The effective status of a single lemma: the explicit one if present;
    /// otherwise, when a CEFR level is declared, implicit `Known(Calibration)`
    /// iff the lemma's level is strictly below it (and `None` — unknown — when
    /// at/above or unlevelled); otherwise the frequency-rank fallback (implicit
    /// `Known(Calibration)` when ranked at or below the calibration threshold);
    /// otherwise `None` ("new"/unknown).
    pub fn resolve_lemma(
        &self,
        lang: StudiedLanguage,
        lemma: &str,
        lexis: &(impl FrequencyRanks + CefrLevels),
    ) -> Option<Status> {
        if let Some(explicit) = self.explicit_status(lang, lemma) {
            return Some(explicit);
        }
        // A declared CEFR level gates presumed-known by level, not by rank:
        // below the level is presumed known; at/above it — or a lemma with no
        // CEFR level (rarer than C2, so a hard word) — is unknown.
        if let Some(declared) = self.declared_level(lang) {
            return match lexis.level(lemma) {
                Some(level) if level < declared => Some(Status::Known(KnownSource::Calibration)),
                _ => None,
            };
        }
        // No declared level: fall back to frequency-rank calibration.
        match lexis.rank(lemma) {
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
        lexis: &(impl FrequencyRanks + CefrLevels),
    ) -> TokenClass {
        let mut best = Verdict::Unknown;
        for candidate in candidates {
            let verdict = match self.resolve_lemma(lang, candidate, lexis) {
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

    /// Classifies a hyphenated compound the lexicon does not know as a unit,
    /// from its part lemmas (e.g. `repo-wide` → `["repo", "wide"]`).
    ///
    /// An explicit status the reader set on the whole compound wins — a card
    /// added for `repo-wide` is honoured as its own lemma. Otherwise the compound is
    /// only as known as its **weakest** part: you understand `repo-wide` only
    /// if you understand both `repo` and `wide` (unlike ambiguous single-word
    /// candidates in [`classify`](Self::classify), where knowing any one sense
    /// is enough). An empty part list is `Unknown`.
    pub fn classify_compound(
        &self,
        lang: StudiedLanguage,
        whole: &str,
        parts: &[String],
        lexis: &(impl FrequencyRanks + CefrLevels),
    ) -> TokenClass {
        if self.explicit_status(lang, whole).is_some() {
            return self.classify(lang, &[whole], lexis);
        }
        let mut weakest: Option<Verdict> = None;
        for part in parts {
            let verdict = match self.resolve_lemma(lang, part, lexis) {
                Some(Status::Known(_)) => Verdict::Known,
                Some(Status::Ignored) => Verdict::Ignored,
                Some(Status::Learning) => Verdict::Learning,
                None => Verdict::Unknown,
            };
            weakest = Some(weakest.map_or(verdict, |w| w.min(verdict)));
        }
        weakest.unwrap_or(Verdict::Unknown).into()
    }

    /// Folds a band of lemmas (typically all lemmas of one CEFR level) into a
    /// confirmed / presumed / to-learn breakdown for the progression ladder.
    /// `confirmed` = an explicit `known`/`ignored` (real, any provenance except
    /// the implicit `calibration`); `presumed` = implicit `Known(Calibration)`
    /// (below the declared level, unproven); `to_learn` = `learning` or new.
    /// The three always sum to the number of lemmas folded.
    pub fn band_stats<'a>(
        &self,
        lang: StudiedLanguage,
        lemmas: impl IntoIterator<Item = &'a str>,
        lexis: &(impl FrequencyRanks + CefrLevels),
    ) -> BandStats {
        let mut stats = BandStats::default();
        for lemma in lemmas {
            match self.explicit_status(lang, lemma) {
                Some(status) if status.counts_as_known() => stats.confirmed += 1,
                Some(_) => stats.to_learn += 1, // learning
                None => match self.resolve_lemma(lang, lemma, lexis) {
                    Some(Status::Known(KnownSource::Calibration)) => stats.presumed += 1,
                    _ => stats.to_learn += 1,
                },
            }
        }
        stats
    }

    /// Confirms presumed-known lemmas that reading has vouched for
    /// (`add-lingua-cefr-levels`). A **caller-driven** operation — recording
    /// exposure never calls it, so a caller that never promotes (the agent
    /// plugin) keeps the v1 "exposure never changes a status" behaviour.
    ///
    /// Promotes a lemma to `Known(Exposure)` iff ALL hold: a CEFR level is
    /// declared for the language and the lemma is strictly below it (presumed),
    /// it has no explicit status (so any user interaction blocks promotion), and
    /// it has been encountered on at least `threshold_days` distinct days.
    /// Returns the promoted lemmas (deterministic order). `Known(Exposure)`
    /// keeps promotions distinguishable and bulk-reversible.
    pub fn promote_by_exposure(
        &mut self,
        lang: StudiedLanguage,
        exposures: &ExposureCounters,
        lexis: &(impl FrequencyRanks + CefrLevels),
        threshold_days: u32,
        at_ms: i64,
    ) -> Vec<String> {
        let Some(declared) = self.declared_level(lang) else {
            return Vec::new();
        };
        let mut promoted = Vec::new();
        for (lemma, exposure) in exposures.lemmas(lang) {
            if exposure.distinct_days < threshold_days {
                continue;
            }
            if self.explicit_status(lang, lemma).is_some() {
                continue;
            }
            if matches!(lexis.level(lemma), Some(level) if level < declared) {
                promoted.push(lemma.to_owned());
            }
        }
        for lemma in &promoted {
            self.set_status_at(lang, lemma, Status::Known(KnownSource::Exposure), at_ms);
        }
        promoted
    }
}

/// Per-band knowledge breakdown for the CEFR ladder. The three counts sum to
/// the size of the band folded (see [`KnowledgeState::band_stats`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BandStats {
    /// Explicitly known/ignored — proven, any provenance except `calibration`.
    pub confirmed: usize,
    /// Presumed known below the declared level (implicit, unproven).
    pub presumed: usize,
    /// Learning or new — not yet known.
    pub to_learn: usize,
}

impl BandStats {
    /// The band size (all three tiers).
    pub fn total(self) -> usize {
        self.confirmed + self.presumed + self.to_learn
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

    fn parts(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn compound_is_known_only_when_every_part_is_known() {
        let mut state = KnowledgeState::new();
        state.set_calibration(EN, 3_000);
        // run (500) + code (2,500): both below the threshold → Known.
        assert_eq!(
            state.classify_compound(EN, "run-code", &parts(&["run", "code"]), &ranks()),
            TokenClass::Known
        );
        // run known, seldom (5,100) above the threshold → weakest link Unknown.
        assert_eq!(
            state.classify_compound(EN, "run-seldom", &parts(&["run", "seldom"]), &ranks()),
            TokenClass::Unknown
        );
    }

    #[test]
    fn compound_takes_its_weakest_part_not_its_strongest() {
        let mut state = KnowledgeState::new();
        state.set_calibration(EN, 3_000);
        state.set_status(EN, "run", Status::Learning); // below threshold, but in the deck
        // run Learning + code Known → weakest is Learning (contrast `classify`,
        // which would pick the strongest and return Known).
        assert_eq!(
            state.classify_compound(EN, "run-code", &parts(&["run", "code"]), &ranks()),
            TokenClass::Learning
        );
    }

    #[test]
    fn an_ignored_part_still_counts_as_known_for_the_compound() {
        let mut state = KnowledgeState::new();
        state.set_calibration(EN, 3_000);
        state.set_status(EN, "run", Status::Ignored);
        // Ignored counts as known (like a Known part), so run + code → Ignored,
        // which the coverage tally treats as known.
        assert_eq!(
            state.classify_compound(EN, "run-code", &parts(&["run", "code"]), &ranks()),
            TokenClass::Ignored
        );
    }

    #[test]
    fn an_explicit_status_on_the_whole_compound_wins() {
        let mut state = KnowledgeState::new();
        state.set_calibration(EN, 3_000);
        // Neither part is known, but the reader added the compound to their deck.
        state.set_status(EN, "repo-wide", Status::Learning);
        assert_eq!(
            state.classify_compound(EN, "repo-wide", &parts(&["repo", "wide"]), &ranks()),
            TokenClass::Learning
        );
    }

    #[test]
    fn a_compound_with_no_parts_is_unknown() {
        let state = KnowledgeState::new();
        assert_eq!(
            state.classify_compound(EN, "repo-wide", &[], &ranks()),
            TokenClass::Unknown
        );
    }

    #[test]
    fn set_status_at_stamps_and_exports_with_its_timestamp() {
        let mut state = KnowledgeState::new();
        state.set_status_at(EN, "seldom", Status::Learning, 1_000);
        state.set_status_at(EN, "run", Status::Known(KnownSource::Manual), 2_000);
        assert_eq!(state.status_updated_at(EN, "seldom"), 1_000);
        let records = state.export_statuses();
        assert_eq!(records.len(), 2);
        // Deterministic (lemma-ordered): run before seldom.
        assert_eq!(records[0].lemma, "run");
        assert_eq!(records[0].updated_at, 2_000);
        assert_eq!(records[1].lemma, "seldom");
        assert_eq!(records[1].status, Status::Learning);
    }

    #[test]
    fn plain_set_status_leaves_a_zero_timestamp() {
        let mut state = KnowledgeState::new();
        state.set_status(EN, "run", Status::Ignored);
        assert_eq!(state.status_updated_at(EN, "run"), 0);
    }

    #[test]
    fn lww_applies_newer_and_equal_but_drops_older() {
        let mut state = KnowledgeState::new();
        state.set_status_at(EN, "run", Status::Learning, 100);

        // Older op: dropped, no change.
        assert!(!state.apply_status_lww(EN, "run", Some(Status::Ignored), 50));
        assert_eq!(state.explicit_status(EN, "run"), Some(Status::Learning));

        // Newer op wins.
        assert!(state.apply_status_lww(EN, "run", Some(Status::Known(KnownSource::Srs)), 200));
        assert_eq!(
            state.explicit_status(EN, "run"),
            Some(Status::Known(KnownSource::Srs))
        );
        assert_eq!(state.status_updated_at(EN, "run"), 200);

        // Equal timestamp applies (server-authoritative) but reports no change when identical.
        assert!(!state.apply_status_lww(EN, "run", Some(Status::Known(KnownSource::Srs)), 200));
    }

    #[test]
    fn lww_clear_removes_the_status_but_keeps_a_tombstone() {
        let mut state = KnowledgeState::new();
        state.set_status_at(EN, "run", Status::Learning, 100);
        assert!(state.apply_status_lww(EN, "run", None, 300));
        assert_eq!(state.explicit_status(EN, "run"), None);
        // A stale re-add older than the clear loses.
        assert!(!state.apply_status_lww(EN, "run", Some(Status::Learning), 200));
        assert_eq!(state.explicit_status(EN, "run"), None);
    }

    #[test]
    fn a_backup_without_the_updated_map_restores_with_zero_timestamps() {
        // An older backup (add-lingua-decks-review era) has no `updated` field.
        // Strip it from a real serialisation rather than hand-writing the format.
        let mut original = KnowledgeState::new();
        original.set_status_at(EN, "run", Status::Known(KnownSource::Manual), 5_000);
        original.set_calibration(EN, 3_000);
        let mut value = serde_json::to_value(&original).expect("serialises");
        value.as_object_mut().expect("object").remove("updated");
        assert!(value.get("updated").is_none());

        let restored: KnowledgeState =
            serde_json::from_value(value).expect("restores without `updated`");
        assert_eq!(
            restored.explicit_status(EN, "run"),
            Some(Status::Known(KnownSource::Manual))
        );
        assert_eq!(restored.status_updated_at(EN, "run"), 0);
        assert_eq!(restored.export_statuses()[0].updated_at, 0);
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

    // Ranks plus CEFR levels for the level-aware tests. `cat` A1, `run` A2,
    // `city` B1, `nuance` B2, `quixotic` C1; `zyzzyva` is ranked but unlevelled.
    fn leveled() -> MapFrequencyRanks {
        MapFrequencyRanks::from_pairs([
            ("cat", 300),
            ("run", 500),
            ("city", 1_200),
            ("nuance", 4_000),
            ("quixotic", 9_000),
            ("zyzzyva", 40_000),
        ])
        .with_levels([
            ("cat", CefrLevel::A1),
            ("run", CefrLevel::A2),
            ("city", CefrLevel::B1),
            ("nuance", CefrLevel::B2),
            ("quixotic", CefrLevel::C1),
        ])
    }

    #[test]
    fn spec_presumed_known_below_the_declared_level() {
        let mut state = KnowledgeState::new();
        state.set_declared_level(EN, CefrLevel::B2);
        // Below B2 → presumed known (implicit calibration).
        assert_eq!(
            state.resolve_lemma(EN, "run", &leveled()),
            Some(Status::Known(KnownSource::Calibration))
        );
        assert_eq!(state.classify(EN, &["city"], &leveled()), TokenClass::Known);
        // At the level and above → unknown (highlighted).
        assert_eq!(
            state.classify(EN, &["nuance"], &leveled()),
            TokenClass::Unknown
        );
        assert_eq!(
            state.classify(EN, &["quixotic"], &leveled()),
            TokenClass::Unknown
        );
    }

    #[test]
    fn a_declared_level_leaves_unlevelled_words_unknown_not_rank_calibrated() {
        let mut state = KnowledgeState::new();
        state.set_declared_level(EN, CefrLevel::C2);
        // Even a very common rank: with a level declared, no CEFR level ⇒ unknown.
        state.set_calibration(EN, 100_000); // would mark everything under the rank path
        assert_eq!(
            state.classify(EN, &["zyzzyva"], &leveled()),
            TokenClass::Unknown
        );
    }

    #[test]
    fn an_explicit_status_still_wins_under_a_declared_level() {
        let mut state = KnowledgeState::new();
        state.set_declared_level(EN, CefrLevel::B2);
        state.set_status(EN, "nuance", Status::Learning); // at the level, but in the deck
        assert_eq!(
            state.classify(EN, &["nuance"], &leveled()),
            TokenClass::Learning
        );
    }

    #[test]
    fn band_stats_splits_confirmed_presumed_and_to_learn() {
        let mut state = KnowledgeState::new();
        state.set_declared_level(EN, CefrLevel::B2);
        // Among these B1/A-level lemmas: mark one confirmed, leave the rest presumed.
        state.set_status(EN, "run", Status::Known(KnownSource::Manual)); // confirmed
        state.set_status(EN, "cat", Status::Learning); // to-learn
        // city presumed (below B2, no explicit); quixotic (C1) is above → to-learn.
        let stats = state.band_stats(EN, ["run", "cat", "city", "quixotic"], &leveled());
        assert_eq!(stats.confirmed, 1);
        assert_eq!(stats.presumed, 1);
        assert_eq!(stats.to_learn, 2);
        assert_eq!(stats.total(), 4);
    }

    #[test]
    fn promote_by_exposure_confirms_below_level_after_threshold_days() {
        let mut state = KnowledgeState::new();
        state.set_declared_level(EN, CefrLevel::B2);
        let mut exp = ExposureCounters::new();
        // `run` (A2, below B2) read on 4 distinct days; `nuance` (B2) also read a lot.
        for d in 0..4 {
            exp.record(EN, "run", 1, "https://x", d * 86_400);
        }
        for d in 0..9 {
            exp.record(EN, "nuance", 1, "https://x", d * 86_400);
        }
        let promoted = state.promote_by_exposure(EN, &exp, &leveled(), 4, 1_000);
        assert_eq!(promoted, vec!["run".to_string()]); // nuance is at-level, not promoted
        assert_eq!(
            state.explicit_status(EN, "run"),
            Some(Status::Known(KnownSource::Exposure))
        );
        assert_eq!(state.status_updated_at(EN, "run"), 1_000);
    }

    #[test]
    fn promote_by_exposure_respects_the_day_threshold() {
        let mut state = KnowledgeState::new();
        state.set_declared_level(EN, CefrLevel::B2);
        let mut exp = ExposureCounters::new();
        // 20 occurrences but all on one day → 1 distinct day → not promoted at N=4.
        for _ in 0..20 {
            exp.record(EN, "run", 1, "https://x", 10);
        }
        assert!(
            state
                .promote_by_exposure(EN, &exp, &leveled(), 4, 1_000)
                .is_empty()
        );
        assert_eq!(state.explicit_status(EN, "run"), None);
    }

    #[test]
    fn an_interaction_blocks_exposure_promotion() {
        let mut state = KnowledgeState::new();
        state.set_declared_level(EN, CefrLevel::B2);
        state.set_status(EN, "run", Status::Learning); // the user added it to the deck
        let mut exp = ExposureCounters::new();
        for d in 0..6 {
            exp.record(EN, "run", 1, "https://x", d * 86_400);
        }
        assert!(
            state
                .promote_by_exposure(EN, &exp, &leveled(), 4, 1_000)
                .is_empty()
        );
        assert_eq!(state.explicit_status(EN, "run"), Some(Status::Learning));
    }

    #[test]
    fn promote_by_exposure_needs_a_declared_level() {
        let mut state = KnowledgeState::new();
        // No declared level → the presumed set is undefined → no promotion.
        let mut exp = ExposureCounters::new();
        for d in 0..9 {
            exp.record(EN, "run", 1, "https://x", d * 86_400);
        }
        assert!(
            state
                .promote_by_exposure(EN, &exp, &leveled(), 4, 1_000)
                .is_empty()
        );
    }

    #[test]
    fn recording_exposure_alone_never_changes_a_status() {
        // The D5 invariant: recording never promotes; only the explicit op does.
        let mut exp = ExposureCounters::new();
        for d in 0..10 {
            exp.record(EN, "run", 1, "https://x", d * 86_400);
        }
        let state = KnowledgeState::new();
        // The reader never called promote_by_exposure → `run` stays new.
        assert_eq!(state.explicit_status(EN, "run"), None);
    }
}
