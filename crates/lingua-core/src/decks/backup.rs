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
//!
//! The file carries a schema version, which belongs to the file rather than to
//! the state (add-lingua-studied-language-profile). Version 1 is a state with the
//! default profile and English records only, written exactly as builds wrote it
//! before the profile was stored; version 2 is any other state. A backup is
//! written in the oldest version that holds it, so a build released before
//! version 2 keeps reading an English reader's backup.

use serde::{Deserialize, Serialize};

use crate::analysis::language::StudiedLanguage;
use crate::knowledge::exposure::ExposureCounters;
use crate::knowledge::profile::Profile;
use crate::knowledge::state::KnowledgeState;

use super::fsrs::FsrsParams;
use super::review::Deck;

/// The newest backup schema version this build reads and writes. A restore
/// reads every version from 1 up to it, and refuses any other.
pub const BACKUP_SCHEMA_VERSION: u32 = 2;

/// The complete local state of the product.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinguaState {
    /// Word statuses and calibration.
    pub knowledge: KnowledgeState,
    /// Exposure counters (encounter history).
    pub exposure: ExposureCounters,
    /// The deck of cards with their FSRS review state.
    pub deck: Deck,
    /// The FSRS scheduler parameters.
    pub fsrs: FsrsParams,
    /// The reader's language profile: the studied languages, the primary first,
    /// and the native language. Kept in the backup only, never synced. Left out
    /// while it is the default, so an English backup does not move; a backup
    /// without it restores the default.
    #[serde(
        default = "Profile::english_for_french",
        skip_serializing_if = "Profile::is_default"
    )]
    pub profile: Profile,
}

impl Default for LinguaState {
    fn default() -> Self {
        Self {
            knowledge: KnowledgeState::new(),
            exposure: ExposureCounters::new(),
            deck: Deck::new(),
            fsrs: FsrsParams::default(),
            profile: Profile::english_for_french(),
        }
    }
}

/// The file [`LinguaState::to_backup`] writes: its schema version first, then
/// the state's own fields, in the order a version 1 file has always had.
#[derive(Serialize)]
struct Written<'a> {
    schema_version: u32,
    #[serde(flatten)]
    state: &'a LinguaState,
}

/// What [`LinguaState::from_backup`] reads before anything else.
#[derive(Deserialize)]
struct Header {
    schema_version: u32,
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
                    "backup schema v{found} is not supported (this build reads v1 to v{supported})"
                )
            }
        }
    }
}

impl std::error::Error for RestoreError {}

impl LinguaState {
    /// Serialises the whole state to a backup string (pretty JSON, stable
    /// key order thanks to the ordered maps within), in the oldest schema
    /// version that holds it.
    pub fn to_backup(&self) -> String {
        let written = Written {
            schema_version: self.backup_version(),
            state: self,
        };
        serde_json::to_string_pretty(&written).expect("LinguaState is always serialisable")
    }

    /// The schema version a backup of this state is written in: 1 while the
    /// profile is the default and every record is English, 2 otherwise. A
    /// build released before version 2 cannot read another language.
    pub fn backup_version(&self) -> u32 {
        let english_only = self.profile.is_default()
            && self
                .knowledge
                .languages()
                .into_iter()
                .chain(self.exposure.languages())
                .chain(self.deck.languages())
                .all(|language| language == StudiedLanguage::English);
        if english_only { 1 } else { 2 }
    }

    /// Restores a state from a backup string. The schema version is read
    /// first, so a version this build does not know is refused as such, never
    /// half-read and never reported as malformed.
    pub fn from_backup(json: &str) -> Result<Self, RestoreError> {
        let header: Header = serde_json::from_str(json).map_err(RestoreError::Malformed)?;
        if !(1..=BACKUP_SCHEMA_VERSION).contains(&header.schema_version) {
            return Err(RestoreError::UnsupportedVersion {
                found: header.schema_version,
                supported: BACKUP_SCHEMA_VERSION,
            });
        }
        serde_json::from_str(json).map_err(RestoreError::Malformed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::language::StudiedLanguage;
    use crate::decks::card::{Card, EncounterSource, Provenance};
    use crate::decks::fsrs::Rating;
    use crate::decks::review::ReviewSession;
    use crate::knowledge::profile::Profile;
    use crate::knowledge::status::{KnownSource, Status};

    const EN: StudiedLanguage = StudiedLanguage::English;
    const ES: StudiedLanguage = StudiedLanguage::Spanish;

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

    /// A backup as `to_backup` wrote it, with its version replaced.
    fn with_version(backup: &str, version: u32) -> String {
        let mut file: serde_json::Value = serde_json::from_str(backup).expect("a backup");
        file["schema_version"] = version.into();
        file.to_string()
    }

    #[test]
    fn a_future_schema_version_is_refused_not_half_read() {
        let future = with_version(
            &LinguaState::default().to_backup(),
            BACKUP_SCHEMA_VERSION + 1,
        );
        let err = LinguaState::from_backup(&future).unwrap_err();
        assert!(matches!(
            err,
            RestoreError::UnsupportedVersion {
                found: 3,
                supported: 2
            }
        ));
        assert_eq!(
            err.to_string(),
            "backup schema v3 is not supported (this build reads v1 to v2)"
        );
        let none = with_version(&LinguaState::default().to_backup(), 0);
        assert!(matches!(
            LinguaState::from_backup(&none),
            Err(RestoreError::UnsupportedVersion { found: 0, .. })
        ));
    }

    #[test]
    fn the_version_is_read_before_the_records() {
        // A later version is refused as such even when its records would not
        // parse here: it is never reported as malformed.
        let future = r#"{"schema_version": 3, "knowledge": {"statuses": {"Klingon": {}}}}"#;
        assert!(matches!(
            LinguaState::from_backup(future),
            Err(RestoreError::UnsupportedVersion { found: 3, .. })
        ));
    }

    #[test]
    fn malformed_backup_is_refused() {
        let err = LinguaState::from_backup("{ not json").unwrap_err();
        assert!(matches!(err, RestoreError::Malformed(_)));
        assert!(err.to_string().starts_with("malformed backup: "));
        // A file without a version is malformed, not of some version.
        let err = LinguaState::from_backup(r#"{"knowledge": {}}"#).unwrap_err();
        assert!(matches!(err, RestoreError::Malformed(_)));
    }

    #[test]
    fn an_english_state_is_written_as_version_1_without_a_profile() {
        let state = populated_state();
        assert_eq!(state.backup_version(), 1);
        let backup = state.to_backup();
        assert!(backup.starts_with("{\n  \"schema_version\": 1,\n  \"knowledge\""));
        assert!(!backup.contains("\"profile\""));
        assert_eq!(
            LinguaState::from_backup(&backup).expect("restore").profile,
            Profile::english_for_french()
        );
    }

    #[test]
    fn another_studied_language_is_written_as_version_2_and_restored() {
        // Ungraded, so no computed f64 is normalised on the first write.
        let mut state = LinguaState::default();
        state.knowledge.set_status(EN, "harbour", Status::Learning);
        state
            .profile
            .set_studied_languages(vec![ES, EN])
            .expect("two languages");
        assert_eq!(state.backup_version(), 2);
        let backup = state.to_backup();
        assert!(backup.starts_with("{\n  \"schema_version\": 2,"));
        let restored = LinguaState::from_backup(&backup).expect("restore");
        assert_eq!(restored.profile.studied_languages, vec![ES, EN]);
        assert_eq!(restored, state, "a version 2 round trip is lossless");
    }

    #[test]
    fn a_record_in_another_language_is_written_as_version_2() {
        let lighthouse = || {
            Card::new(
                "faro",
                "faros",
                Provenance {
                    sentence: "Los faros brillan.".to_owned(),
                    source: EncounterSource::Web {
                        url: "https://example.es".to_owned(),
                    },
                    captured_at: 1,
                },
                None,
            )
        };
        let mut deck = LinguaState::default();
        deck.deck.upsert(ES, lighthouse());
        let mut statuses = LinguaState::default();
        statuses.knowledge.set_status(ES, "faro", Status::Learning);
        let mut exposure = LinguaState::default();
        exposure.exposure.record(ES, "faro", 1, "web", 1);
        let mut calibration = LinguaState::default();
        calibration.knowledge.set_calibration(ES, 100);
        for state in [deck, statuses, exposure, calibration] {
            assert!(state.profile.is_default());
            assert_eq!(state.backup_version(), 2);
            let restored = LinguaState::from_backup(&state.to_backup()).expect("restore");
            assert_eq!(restored, state);
        }
        assert_eq!(LinguaState::default().backup_version(), 1);
    }

    #[test]
    fn a_version_1_file_carrying_a_profile_is_read() {
        // Nothing writes it, but a version 1 file is read with the version 2 shape.
        let mut state = LinguaState::default();
        state.profile.set_studied_languages(vec![ES]).expect("one");
        let as_v1 = with_version(&state.to_backup(), 1);
        assert_eq!(
            LinguaState::from_backup(&as_v1)
                .expect("restore")
                .profile
                .studied_languages,
            vec![ES]
        );
    }
}
