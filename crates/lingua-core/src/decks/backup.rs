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
//! the state (add-lingua-studied-language-profile). A backup is written in the
//! oldest version that holds it:
//! - version 1 is a state with the default profile and English records only,
//!   their glosses French, written exactly as builds wrote it before the profile
//!   was stored, so a build released before version 2 keeps reading an English
//!   reader's backup;
//! - version 3 is a state that names French as a studied language, in its
//!   profile or in any per-language record (add-lingua-french-baseline D7). A
//!   build released with version 2 reads the version first, so it refuses such a
//!   file as unsupported, by name, never as malformed;
//! - version 2 is any other state: a reader of Spanish, or a native language
//!   other than French. Their files do not move when version 3 exists.
//!
//! A French native language alone never raises the version: every installed
//! reader has it.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::analysis::language::StudiedLanguage;
use crate::knowledge::exposure::ExposureCounters;
use crate::knowledge::profile::Profile;
use crate::knowledge::state::KnowledgeState;

use super::card::FRENCH;
use super::fsrs::FsrsParams;
use super::review::Deck;

/// The newest backup schema version this build reads and writes. A restore
/// reads every version from 1 up to it, and refuses any other.
pub const BACKUP_SCHEMA_VERSION: u32 = 3;

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

    /// The schema version a backup of this state is written in: 3 when French
    /// is among the profile's studied languages or among the languages the
    /// knowledge, exposure or deck records hold; else 1 while the profile is the
    /// default, every record is English and every gloss is French; else 2. A
    /// build released before version 2 cannot read another language; a gloss in
    /// another language is one more record of it (add-lingua-card-gloss-language
    /// D1), so a version 1 file never carries a gloss language. A build released
    /// before version 3 cannot read French, and refuses the version it is written
    /// in before reading a record (add-lingua-french-baseline D7).
    pub fn backup_version(&self) -> u32 {
        let records = self.record_languages();
        let french = StudiedLanguage::French;
        if self.profile.studied_languages.contains(&french) || records.contains(&french) {
            return 3;
        }
        let english_only = self.profile.is_default()
            && records
                .into_iter()
                .all(|language| language == StudiedLanguage::English)
            && self
                .deck
                .iter()
                .all(|(_, card)| card.gloss_language == FRENCH);
        if english_only { 1 } else { 2 }
    }

    /// Every language the per-language records hold: statuses, calibrations and
    /// declared levels, exposure counters, cards.
    fn record_languages(&self) -> BTreeSet<StudiedLanguage> {
        let mut languages = self.knowledge.languages();
        languages.extend(self.exposure.languages());
        languages.extend(self.deck.languages());
        languages
    }

    /// Restores a state from a backup string. The schema version is read
    /// first, so a version this build does not know is refused as such, never
    /// half-read and never reported as malformed.
    pub fn from_backup(json: &str) -> Result<Self, RestoreError> {
        read_backup(json, BACKUP_SCHEMA_VERSION)
    }
}

/// A restore by a build that reads schema versions 1 to `newest`: the header
/// first, then the state. Every build released since 1.5.0 restores this way, so
/// a file of a later version is refused as unsupported, by name, whatever its
/// records hold.
fn read_backup(json: &str, newest: u32) -> Result<LinguaState, RestoreError> {
    let header: Header = serde_json::from_str(json).map_err(RestoreError::Malformed)?;
    if !(1..=newest).contains(&header.schema_version) {
        return Err(RestoreError::UnsupportedVersion {
            found: header.schema_version,
            supported: newest,
        });
    }
    serde_json::from_str(json).map_err(RestoreError::Malformed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::language::StudiedLanguage;
    use crate::decks::card::{Card, EncounterSource, Provenance};
    use crate::decks::fsrs::Rating;
    use crate::decks::review::ReviewSession;
    use crate::knowledge::level::CefrLevel;
    use crate::knowledge::profile::{NativeLanguage, Profile};
    use crate::knowledge::status::{KnownSource, Status};

    const EN: StudiedLanguage = StudiedLanguage::English;
    const ES: StudiedLanguage = StudiedLanguage::Spanish;
    const FR: StudiedLanguage = StudiedLanguage::French;

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
                "fr",
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
                found: 4,
                supported: 3
            }
        ));
        assert_eq!(
            err.to_string(),
            "backup schema v4 is not supported (this build reads v1 to v3)"
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
        let future = r#"{"schema_version": 4, "knowledge": {"statuses": {"Klingon": {}}}}"#;
        assert!(matches!(
            LinguaState::from_backup(future),
            Err(RestoreError::UnsupportedVersion {
                found: 4,
                supported: 3
            })
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
    fn the_studied_languages_are_written_where_the_extension_reads_them_without_an_engine() {
        // apps/lingua-extension/src/state/profile.ts reads `profile.studied_languages`, under the
        // enum's names, straight from the stored backup (generalise-lingua-translation-model-state
        // D2): renaming either breaks the extension's translation models without a type error.
        let mut state = LinguaState::default();
        state
            .profile
            .set_studied_languages(vec![ES, EN])
            .expect("two languages");
        let json: serde_json::Value = serde_json::from_str(&state.to_backup()).expect("json");
        assert_eq!(
            json["profile"]["studied_languages"],
            serde_json::json!(["Spanish", "English"])
        );
    }

    #[test]
    fn the_native_language_is_written_where_the_extension_reads_it_without_an_engine() {
        // apps/lingua-extension/src/state/profile.ts reads `profile.native_language`, under the
        // enum's names, straight from the stored backup (generalise-lingua-native-language D7):
        // renaming a variant would silently give every such reader French.
        let mut state = LinguaState::default();
        state
            .profile
            .set(NativeLanguage::English, vec![ES])
            .expect("Spanish for an English reader");
        let json: serde_json::Value = serde_json::from_str(&state.to_backup()).expect("json");
        assert_eq!(json["profile"]["native_language"], "English");
        assert_eq!(
            NativeLanguage::ALL.map(|native| serde_json::to_value(native).expect("a name")),
            ["French", "English", "Spanish"].map(serde_json::Value::from)
        );
    }

    #[test]
    fn another_native_language_is_written_as_version_2_and_restored() {
        let mut state = LinguaState::default();
        state.knowledge.set_status(ES, "faro", Status::Learning);
        state
            .profile
            .set(NativeLanguage::English, vec![ES])
            .expect("Spanish for an English reader");
        assert_eq!(state.backup_version(), 2);
        let backup = state.to_backup();
        assert!(backup.starts_with("{\n  \"schema_version\": 2,"));
        assert!(
            backup.contains("\"native_language\": \"English\""),
            "{backup}"
        );
        let restored = LinguaState::from_backup(&backup).expect("restore");
        assert_eq!(
            restored.profile,
            Profile::studying(NativeLanguage::English, ES)
        );
        assert_eq!(restored, state, "a version 2 round trip is lossless");
    }

    #[test]
    fn spec_scenario_knowledge_does_not_follow_the_native_language() {
        // lingua-knowledge-model: a reader's native language changes from French to English,
        // and their statuses, declared levels and cards in Spanish are kept as they were.
        let mut state = LinguaState::default();
        state
            .profile
            .set(NativeLanguage::French, vec![ES])
            .expect("Spanish for a French reader");
        state.knowledge.set_status(ES, "faro", Status::Learning);
        state.knowledge.set_declared_level(ES, CefrLevel::B1);
        state.deck.upsert(
            ES,
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
                Some("phare".to_owned()),
                "fr",
            ),
        );
        let (knowledge, deck) = (state.knowledge.clone(), state.deck.clone());

        state
            .profile
            .set(NativeLanguage::English, vec![ES])
            .expect("Spanish for an English reader");

        assert_eq!(state.knowledge, knowledge, "statuses and declared levels");
        assert_eq!(state.deck, deck, "cards");
        assert_eq!(
            state.knowledge.explicit_status(ES, "faro"),
            Some(Status::Learning)
        );
        assert_eq!(state.knowledge.declared_level(ES), Some(CefrLevel::B1));
        assert!(state.deck.get(ES, "faro").is_some());
        // And so after the backup that carries the new native language.
        let restored = LinguaState::from_backup(&state.to_backup()).expect("restore");
        assert_eq!(restored.profile.native_language, NativeLanguage::English);
        assert_eq!(restored.knowledge, knowledge);
        assert_eq!(restored.deck, deck);
    }

    #[test]
    fn a_version_2_backup_of_the_previous_build_restores_with_french_native() {
        // The previous build wrote every profile with French native, whatever it studied.
        let mut previous: serde_json::Value =
            serde_json::from_str(&LinguaState::default().to_backup()).expect("json");
        previous["schema_version"] = 2.into();
        previous["profile"] = serde_json::json!({
            "native_language": "French",
            "studied_languages": ["Spanish", "English"]
        });
        let restored = LinguaState::from_backup(&previous.to_string()).expect("restore");
        assert_eq!(restored.profile.native_language, NativeLanguage::French);
        assert_eq!(restored.profile.studied_languages, vec![ES, EN]);
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
                "fr",
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

    /// A Spanish card, its gloss written in `gloss_language`.
    fn faro(gloss: Option<&str>, gloss_language: &str) -> Card {
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
            gloss.map(str::to_owned),
            gloss_language,
        )
    }

    #[test]
    fn spec_scenario_every_card_today() {
        // lingua-decks-review: a reader of French with 20 French-glossed cards is backed up. The
        // backup is byte for byte what the previous build wrote — no card carries a label — and
        // every card restores with the gloss language `fr`; a version 1 backup (English cards)
        // and a version 2 one (a Spanish card, glossed in French) alike. `tests/backup_format.rs`
        // pins the version 1 bytes themselves.
        let state = populated_state();
        assert_eq!((state.deck.len(), state.backup_version()), (20, 1));
        let backup = state.to_backup();
        assert!(!backup.contains("gloss_language"), "{backup}");
        let restored = LinguaState::from_backup(&backup).expect("restore");
        for i in 0..20 {
            let card = restored
                .deck
                .get(EN, &format!("card{i:02}"))
                .expect("a card");
            assert_eq!(card.gloss_language, "fr");
        }
        // Written again, still without a label (the fixpoint itself is
        // spec_backup_round_trip_is_lossless: a graded card's first write may move an f64 by 1 ULP).
        assert!(!restored.to_backup().contains("gloss_language"));

        let mut spanish = LinguaState::default();
        spanish.deck.upsert(ES, faro(Some("phare"), "fr"));
        assert_eq!(spanish.backup_version(), 2);
        let backup = spanish.to_backup();
        assert!(backup.starts_with("{\n  \"schema_version\": 2,"));
        assert!(!backup.contains("gloss_language"), "{backup}");
        let restored = LinguaState::from_backup(&backup).expect("restore");
        assert_eq!(
            restored.deck.get(ES, "faro").expect("faro").gloss_language,
            "fr"
        );
        assert_eq!(restored, spanish);
    }

    #[test]
    fn spec_scenario_a_card_created_on_an_engine_glossed_in_english() {
        // lingua-decks-review: the card's gloss language is `en`, the backup carries it — at
        // version 2, under the schema version the previous build reads, to which the label is
        // one more field a card does not deny — and it reads back.
        let mut state = LinguaState::default();
        state.deck.upsert(ES, faro(Some("lighthouse"), "en"));
        assert_eq!(state.backup_version(), 2, "the label bumps no schema");
        let backup = state.to_backup();
        assert!(backup.contains("\"gloss_language\": \"en\""), "{backup}");
        let restored = LinguaState::from_backup(&backup).expect("restore");
        let faro = restored.deck.get(ES, "faro").expect("faro");
        assert_eq!(
            (faro.gloss_language.as_str(), faro.gloss.as_deref()),
            ("en", Some("lighthouse"))
        );
        assert_eq!(restored, state, "a version 2 round trip is lossless");
    }

    #[test]
    fn a_gloss_in_another_language_is_written_in_version_2() {
        // An English reader with the default profile, whose one non-default record is a card
        // glossed in English: another language in the records, so version 2 — a version 1
        // file never carries a gloss language. `tests/backup_format.rs` pins that the
        // version 1 English backup is unchanged.
        let mut state = populated_state();
        assert_eq!(state.backup_version(), 1);
        let mut harbour = Card::new(
            "harbour",
            "harbours",
            Provenance {
                sentence: "Ships rest in the harbours.".to_owned(),
                source: EncounterSource::Web {
                    url: "https://example.com/sea".to_owned(),
                },
                captured_at: 1_700_000_004,
            },
            Some("port".to_owned()),
            "fr",
        );
        state.deck.upsert(EN, harbour.clone());
        assert_eq!(
            state.backup_version(),
            1,
            "a French gloss on an English card"
        );
        harbour.gloss_language = "en".to_owned();
        state.deck.upsert(EN, harbour);
        assert!(state.profile.is_default());
        assert_eq!(state.backup_version(), 2);
        let backup = state.to_backup();
        assert!(backup.starts_with("{\n  \"schema_version\": 2,"));
        assert_eq!(backup.matches("gloss_language").count(), 1, "{backup}");
        let restored = LinguaState::from_backup(&backup).expect("restore");
        assert_eq!(
            restored
                .deck
                .get(EN, "harbour")
                .expect("harbour")
                .gloss_language,
            "en"
        );
        assert_eq!(restored.backup_version(), 2);
    }

    #[test]
    fn a_backup_carrying_a_blank_gloss_language_restores_french() {
        // Nothing writes a blank label; a file carrying one restores every card as `fr`, and
        // is written again as the previous build wrote it.
        let written = populated_state().to_backup();
        let mut file: serde_json::Value = serde_json::from_str(&written).expect("a backup");
        let cards = file["deck"]["cards"]["English"]
            .as_object_mut()
            .expect("the English cards");
        cards.get_mut("card00").expect("card00")["gloss_language"] = "".into();
        cards.get_mut("card01").expect("card01")["gloss_language"] = "  ".into();
        let restored = LinguaState::from_backup(&file.to_string()).expect("restore");
        for lemma in ["card00", "card01"] {
            assert_eq!(
                restored.deck.get(EN, lemma).expect(lemma).gloss_language,
                "fr"
            );
        }
        assert_eq!(restored.backup_version(), 1);
        assert!(!restored.to_backup().contains("gloss_language"));
        assert_eq!(
            restored,
            LinguaState::from_backup(&written).expect("restore"),
            "as the file without the blanks"
        );
    }

    /// A French card, its gloss written in English.
    fn homme() -> Card {
        Card::new(
            "homme",
            "l'homme",
            Provenance {
                sentence: "L'homme regardait la mer.".to_owned(),
                source: EncounterSource::Web {
                    url: "https://example.fr".to_owned(),
                },
                captured_at: 1,
            },
            Some("man".to_owned()),
            "en",
        )
    }

    /// A reader of Spanish, English native: the profile French records sit under.
    fn spanish_for_english() -> LinguaState {
        let mut state = LinguaState::default();
        state
            .profile
            .set(NativeLanguage::English, vec![ES])
            .expect("Spanish for an English reader");
        state
    }

    #[test]
    fn spec_scenario_a_french_reader_s_backup() {
        // lingua-decks-review: French studied with English native, backed up and restored.
        let mut state = LinguaState::default();
        state
            .profile
            .set(NativeLanguage::English, vec![FR])
            .expect("French for an English reader");
        assert_eq!(state.backup_version(), 3, "French in the profile alone");
        let backup = state.to_backup();
        assert!(
            backup.starts_with("{\n  \"schema_version\": 3,"),
            "{backup}"
        );
        assert!(
            backup.contains("\"studied_languages\": [\n      \"French\"\n    ]"),
            "{backup}"
        );
        let restored = LinguaState::from_backup(&backup).expect("restore");
        assert_eq!(
            restored.profile,
            Profile::studying(NativeLanguage::English, FR)
        );
        assert_eq!(restored, state, "a version 3 round trip is lossless");
    }

    #[test]
    fn spec_scenario_french_in_the_records_only() {
        // Under a Spanish profile, any French record raises the backup to version 3.
        let mut deck = spanish_for_english();
        deck.deck.upsert(FR, homme());
        let mut statuses = spanish_for_english();
        statuses.knowledge.set_status(FR, "homme", Status::Learning);
        let mut exposure = spanish_for_english();
        exposure.exposure.record(FR, "mer", 1, "web", 1);
        let mut calibration = spanish_for_english();
        calibration.knowledge.set_calibration(FR, 100);
        // And under the default profile, which every installed reader has.
        let mut default = LinguaState::default();
        default.knowledge.set_status(FR, "homme", Status::Learning);
        for state in [deck, statuses, exposure, calibration, default] {
            assert_eq!(state.backup_version(), 3);
            let backup = state.to_backup();
            assert!(backup.starts_with("{\n  \"schema_version\": 3,"));
            let restored = LinguaState::from_backup(&backup).expect("restore");
            assert_eq!(restored, state, "a version 3 round trip is lossless");
            assert_eq!(restored.backup_version(), 3);
        }
        assert_eq!(spanish_for_english().backup_version(), 2);
    }

    #[test]
    fn a_french_native_language_never_raises_the_version() {
        // Every installed reader is French native: the default stays version 1, a reader of
        // Spanish version 2.
        assert_eq!(LinguaState::default().backup_version(), 1);
        assert_eq!(populated_state().backup_version(), 1);
        let mut spanish = LinguaState::default();
        spanish
            .profile
            .set(NativeLanguage::French, vec![ES, EN])
            .expect("Spanish for a French reader");
        spanish.knowledge.set_status(ES, "faro", Status::Learning);
        assert_eq!(spanish.backup_version(), 2);
        assert!(
            spanish
                .to_backup()
                .starts_with("{\n  \"schema_version\": 2,")
        );
        // A Spanish-native reader of English is version 2 as well.
        let mut english = LinguaState::default();
        english
            .profile
            .set(NativeLanguage::Spanish, vec![EN])
            .expect("English for a Spanish reader");
        assert_eq!(english.backup_version(), 2);
    }

    #[test]
    fn spec_scenario_a_build_released_before_version_3() {
        // lingua-decks-review: a build that reads versions 1 and 2 reads the version first (it has
        // since 1.5.0), so a version 3 backup holding French records is refused as unsupported
        // version 3, never as malformed — although `"French"` is a name it could not read.
        let mut state = spanish_for_english();
        state.deck.upsert(FR, homme());
        state.knowledge.set_status(FR, "homme", Status::Learning);
        let backup = state.to_backup();
        let err = read_backup(&backup, 2).unwrap_err();
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
        // This build reads all three versions.
        for version in 1..=BACKUP_SCHEMA_VERSION {
            let file = with_version(&LinguaState::default().to_backup(), version);
            assert!(LinguaState::from_backup(&file).is_ok(), "version {version}");
        }
    }

    #[test]
    fn a_committed_version_3_file_is_refused_by_a_build_reading_version_2() {
        // The file this build writes for a reader with French records, pinned by
        // tests/backup_format.rs: a build that reads versions 1 and 2 refuses it by its version,
        // and this build restores it.
        let file = include_str!("../../tests/fixtures/backup-v3-french.json");
        assert!(file.contains("\"French\": {"), "French records");
        assert!(matches!(
            read_backup(file, 2),
            Err(RestoreError::UnsupportedVersion {
                found: 3,
                supported: 2
            })
        ));
        let restored = LinguaState::from_backup(file).expect("restore");
        assert_eq!(restored.backup_version(), 3);
        assert_eq!(restored.to_backup(), file, "written back byte for byte");
    }
}
