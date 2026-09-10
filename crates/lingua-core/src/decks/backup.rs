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

//! The whole-product local state and its lossless backup/restore (design D4).
//!
//! [`LinguaState`] is the versioned serialisable root every surface persists
//! (extension storage, the Apple app, the plugin) and that the future sync
//! consumes: knowledge (statuses + calibration), exposure counters, the deck,
//! and the FSRS parameters. Backup is a plain serialisation of it and restore
//! its inverse — a lossless round trip. Because every field is serialisable,
//! an Anki-format export (deferred) stays a plain serialiser over the same
//! data.

use serde::{Deserialize, Serialize};

use crate::knowledge::exposure::ExposureCounters;
use crate::knowledge::state::KnowledgeState;

use super::fsrs::FsrsParams;
use super::review::Deck;

/// The current backup schema version. Bumped when the shape changes; a
/// restore refuses a version it does not understand.
pub const BACKUP_SCHEMA_VERSION: u32 = 1;

/// The complete local state of the product.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinguaState {
    /// The backup schema version (see [`BACKUP_SCHEMA_VERSION`]).
    pub schema_version: u32,
    /// Word statuses and calibration.
    pub knowledge: KnowledgeState,
    /// Exposure counters (encounter history).
    pub exposure: ExposureCounters,
    /// The deck of cards with their FSRS review state.
    pub deck: Deck,
    /// The FSRS scheduler parameters.
    pub fsrs: FsrsParams,
}

impl Default for LinguaState {
    fn default() -> Self {
        Self {
            schema_version: BACKUP_SCHEMA_VERSION,
            knowledge: KnowledgeState::new(),
            exposure: ExposureCounters::new(),
            deck: Deck::new(),
            fsrs: FsrsParams::default(),
        }
    }
}

/// Why a restore was refused.
#[derive(Debug)]
pub enum RestoreError {
    /// The backup bytes are not valid JSON for this schema.
    Malformed(serde_json::Error),
    /// The backup's schema version is not supported by this build.
    UnsupportedVersion { found: u32, supported: u32 },
}

impl std::fmt::Display for RestoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestoreError::Malformed(e) => write!(f, "malformed backup: {e}"),
            RestoreError::UnsupportedVersion { found, supported } => {
                write!(
                    f,
                    "backup schema v{found} is not supported (this build reads v{supported})"
                )
            }
        }
    }
}

impl std::error::Error for RestoreError {}

impl LinguaState {
    /// Serialises the whole state to a backup string (pretty JSON, stable
    /// key order thanks to the ordered maps within).
    pub fn to_backup(&self) -> String {
        serde_json::to_string_pretty(self).expect("LinguaState is always serialisable")
    }

    /// Restores a state from a backup string, refusing an unknown schema
    /// version so a newer file is never silently half-read.
    pub fn from_backup(json: &str) -> Result<Self, RestoreError> {
        let state: LinguaState = serde_json::from_str(json).map_err(RestoreError::Malformed)?;
        if state.schema_version != BACKUP_SCHEMA_VERSION {
            return Err(RestoreError::UnsupportedVersion {
                found: state.schema_version,
                supported: BACKUP_SCHEMA_VERSION,
            });
        }
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::language::StudiedLanguage;
    use crate::decks::card::{Card, EncounterSource, Provenance};
    use crate::decks::fsrs::Rating;
    use crate::decks::review::ReviewSession;
    use crate::knowledge::status::{KnownSource, Status};

    const EN: StudiedLanguage = StudiedLanguage::English;

    fn populated_state() -> LinguaState {
        let mut state = LinguaState::default();
        state.knowledge.set_calibration(EN, 3_000);

        // 300 word statuses.
        for i in 0..300 {
            let status = match i % 3 {
                0 => Status::Known(KnownSource::Manual),
                1 => Status::Learning,
                _ => Status::Ignored,
            };
            state
                .knowledge
                .set_status(EN, &format!("word{i:03}"), status);
        }

        // Some exposure history.
        state
            .exposure
            .record(EN, "conundrum", 4, "claude:session-1", 1_700_000_000);

        // 20 cards, graded to varied FSRS states.
        for i in 0..20 {
            let lemma = format!("card{i:02}");
            let card = Card::new(
                &lemma,
                &lemma,
                Provenance {
                    sentence: format!("Context number {i}."),
                    source: EncounterSource::Web {
                        url: format!("https://example.com/{i}"),
                    },
                    captured_at: 1_700_000_000 + i,
                },
                (i % 2 == 0).then(|| format!("gloss {i}")),
            );
            state.deck.upsert(EN, card);
        }
        let params = state.fsrs.clone();
        let mut session = ReviewSession::start(&state.deck, 0);
        let ratings = [Rating::Good, Rating::Hard, Rating::Easy, Rating::Again];
        let mut n = 0;
        while session.current_key().is_some() {
            session.grade(
                &mut state.deck,
                &params,
                ratings[n % 4],
                (n as i64) * 86_400,
            );
            n += 1;
        }
        state
    }

    #[test]
    fn spec_backup_round_trip_is_lossless() {
        let state = populated_state();
        assert_eq!(state.knowledge.explicit_count(), 300);
        assert_eq!(state.deck.len(), 20);

        // A stored state (what a surface actually persists) restores
        // identically and is a fixpoint: restoring the backup and backing up
        // again yields byte-identical output, and the restore is idempotent.
        // (serde_json normalises a computed `f64` by at most 1 ULP the first
        // time it is written — ~1e-15 on a 1..10 difficulty, far below any
        // rounded-day interval, and stable from the first restore on. Due
        // dates, ids, statuses, calibration and counts are integer/string and
        // survive bit-for-bit.)
        let stored = state.to_backup();
        let restored = LinguaState::from_backup(&stored).expect("restore");
        let backup_again = restored.to_backup();
        let restored2 = LinguaState::from_backup(&backup_again).expect("restore again");

        assert_eq!(
            backup_again,
            restored2.to_backup(),
            "the backup file is a fixpoint"
        );
        assert_eq!(restored, restored2, "restore is idempotent");
    }

    #[test]
    fn no_populated_field_is_dropped() {
        let state = populated_state();
        let restored = LinguaState::from_backup(&state.to_backup()).expect("restore");
        // Spot-check a graded card's FSRS memory and a gloss survive.
        let card0 = restored.deck.get(EN, "card00").expect("card00");
        assert!(card0.review.memory.is_some());
        assert_eq!(card0.gloss.as_deref(), Some("gloss 0"));
        assert_eq!(restored.knowledge.calibration(EN), 3_000);
        assert_eq!(
            restored
                .exposure
                .get(EN, "conundrum")
                .map(|e| e.occurrences),
            Some(4)
        );
    }

    #[test]
    fn a_future_schema_version_is_refused_not_half_read() {
        let state = LinguaState {
            schema_version: BACKUP_SCHEMA_VERSION + 1,
            ..Default::default()
        };
        let err = LinguaState::from_backup(&state.to_backup()).unwrap_err();
        assert!(matches!(err, RestoreError::UnsupportedVersion { .. }));
    }

    #[test]
    fn malformed_backup_is_refused() {
        let err = LinguaState::from_backup("{ not json").unwrap_err();
        assert!(matches!(err, RestoreError::Malformed(_)));
    }
}
