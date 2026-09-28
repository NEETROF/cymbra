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

//! The announcement event model (change: add-discord-notifications, design D4).
//!
//! A producer enqueues an **event descriptor** — what happened and to which
//! subject — never a rendered message: the worker re-reads the subject and
//! re-evaluates the flags at publication time, so a message never describes a
//! state that stopped being public between the enqueue and the post.
//!
//! **The deny-list is the type.** Only publicly accepted catalog content and
//! anonymous season records have a variant: authentication, pending or rejected
//! moderation states, and anything naming a player cannot be expressed, so no
//! producer can enqueue them by mistake.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// The product whose section of the server a category belongs to. The server
/// groups channels by product, so routing is keyed by product first (design D1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Product {
    Music,
}

impl Product {
    pub fn as_str(self) -> &'static str {
        match self {
            Product::Music => "music",
        }
    }
}

/// A product-namespaced announcement category — the unit a channel, and a
/// back-office flag, is attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Category {
    /// A score or a SoundFont accepted into the public catalog.
    MusicCatalog,
    /// A season record beaten on a catalog piece (anonymous).
    MusicRecords,
}

impl Category {
    /// Every category, for exhaustive routing and flag tests.
    pub const ALL: [Category; 2] = [Category::MusicCatalog, Category::MusicRecords];

    pub fn product(self) -> Product {
        match self {
            Category::MusicCatalog | Category::MusicRecords => Product::Music,
        }
    }

    /// `<product>.<name>` — the namespaced key used in flags and logs.
    pub fn key(self) -> &'static str {
        match self {
            Category::MusicCatalog => "music.catalog",
            Category::MusicRecords => "music.records",
        }
    }
}

/// Which board a season record was set on. A copy of the music leaderboard's
/// mode, so this crate stays free of the music module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordMode {
    /// Free play: the synchronization sub-score.
    Tempo,
    /// Wait Mode: the reaction sub-score.
    Reaction,
}

impl RecordMode {
    pub fn as_str(self) -> &'static str {
        match self {
            RecordMode::Tempo => "tempo",
            RecordMode::Reaction => "reaction",
        }
    }
}

/// What happened. Serialized as the `discord_notify` job payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnnouncementEvent {
    /// A SoundFont moved to `accepted` in the public catalog.
    SoundFontAccepted { soundfont_id: String },
    /// A score proposal moved to `accepted` in the public catalog.
    ScoreAccepted { catalog_score_id: String },
    /// A player's result became the best of the season on a catalog piece and
    /// mode, beating a record another player held. Carries **no player
    /// identity**: naming needs a dedicated consent (a later tranche).
    SeasonRecord {
        season_id: String,
        catalog_score_id: String,
        mode: RecordMode,
        /// The new record, a percentage in `0..=100`.
        subscore: f32,
        /// The UTC day it was achieved — the throttle unit (see [`Self::dedup_key`]).
        achieved_on: NaiveDate,
    },
}

impl AnnouncementEvent {
    pub fn category(&self) -> Category {
        match self {
            AnnouncementEvent::SoundFontAccepted { .. }
            | AnnouncementEvent::ScoreAccepted { .. } => Category::MusicCatalog,
            AnnouncementEvent::SeasonRecord { .. } => Category::MusicRecords,
        }
    }

    /// The idempotency key (design D5): two events with the same key are one
    /// announcement, whatever the number of enqueues or job attempts.
    ///
    /// A catalog item is announced once in its life, even if it is rejected and
    /// accepted again. A piece's records are announced at most **once per mode per
    /// UTC day**: a popular piece whose record falls ten times in an evening
    /// yields one message, not ten — the channel stays readable.
    pub fn dedup_key(&self) -> String {
        match self {
            AnnouncementEvent::SoundFontAccepted { soundfont_id } => {
                format!("discord:music.soundfont_accepted:{soundfont_id}")
            }
            AnnouncementEvent::ScoreAccepted { catalog_score_id } => {
                format!("discord:music.score_accepted:{catalog_score_id}")
            }
            AnnouncementEvent::SeasonRecord {
                season_id,
                catalog_score_id,
                mode,
                achieved_on,
                ..
            } => format!(
                "discord:music.season_record:{season_id}:{catalog_score_id}:{}:{achieved_on}",
                mode.as_str()
            ),
        }
    }
}

/// Whether a new season best is an announceable **record**: it must beat a best
/// that **another** player holds, and the player must not already have held the
/// record. The first result on a piece is not a record (the start of every
/// season would otherwise flood the channel), and a holder improving their own
/// record is not "beaten".
///
/// `others_best` is the best of every other player on the same season, piece and
/// mode; `own_previous` the player's own best before this result.
pub fn beats_record(others_best: Option<f32>, own_previous: Option<f32>, new: f32) -> bool {
    let Some(others) = others_best else {
        return false;
    };
    let already_held = own_previous.is_some_and(|own| own > others);
    new > others && !already_held
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(day: u32, mode: RecordMode) -> AnnouncementEvent {
        AnnouncementEvent::SeasonRecord {
            season_id: "2026-09".into(),
            catalog_score_id: "p1".into(),
            mode,
            subscore: 97.5,
            achieved_on: NaiveDate::from_ymd_opt(2026, 9, day).unwrap(),
        }
    }

    #[test]
    fn categories_follow_the_event_kind() {
        let sf = AnnouncementEvent::SoundFontAccepted {
            soundfont_id: "s".into(),
        };
        let score = AnnouncementEvent::ScoreAccepted {
            catalog_score_id: "c".into(),
        };
        assert_eq!(sf.category(), Category::MusicCatalog);
        assert_eq!(score.category(), Category::MusicCatalog);
        assert_eq!(
            record(1, RecordMode::Tempo).category(),
            Category::MusicRecords
        );
        for c in Category::ALL {
            assert_eq!(c.product(), Product::Music);
            assert!(c.key().starts_with(c.product().as_str()));
        }
    }

    #[test]
    fn dedup_keys_are_distinct_per_subject_and_stable() {
        let a = AnnouncementEvent::ScoreAccepted {
            catalog_score_id: "c1".into(),
        };
        let b = AnnouncementEvent::SoundFontAccepted {
            soundfont_id: "c1".into(),
        };
        assert_eq!(a.dedup_key(), "discord:music.score_accepted:c1");
        assert_ne!(a.dedup_key(), b.dedup_key());
    }

    #[test]
    fn a_record_is_announced_once_per_piece_mode_and_day() {
        let mut later_same_day = record(3, RecordMode::Tempo);
        if let AnnouncementEvent::SeasonRecord { subscore, .. } = &mut later_same_day {
            *subscore = 99.0;
        }
        assert_eq!(
            record(3, RecordMode::Tempo).dedup_key(),
            later_same_day.dedup_key()
        );
        assert_ne!(
            record(3, RecordMode::Tempo).dedup_key(),
            record(4, RecordMode::Tempo).dedup_key()
        );
        assert_ne!(
            record(3, RecordMode::Tempo).dedup_key(),
            record(3, RecordMode::Reaction).dedup_key()
        );
    }

    #[test]
    fn the_payload_round_trips_as_tagged_json() {
        let e = record(3, RecordMode::Reaction);
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json["kind"], "season_record");
        assert_eq!(json["mode"], "reaction");
        assert_eq!(json["achieved_on"], "2026-09-03");
        let back: AnnouncementEvent = serde_json::from_value(json).unwrap();
        assert_eq!(back, e);
        assert_eq!(RecordMode::Tempo.as_str(), "tempo");
    }

    #[test]
    fn an_unknown_kind_is_rejected() {
        let bad = serde_json::json!({ "kind": "user_signed_in", "user_id": "u" });
        assert!(serde_json::from_value::<AnnouncementEvent>(bad).is_err());
    }

    #[test]
    fn the_first_result_on_a_piece_is_not_a_record() {
        assert!(!beats_record(None, None, 100.0));
    }

    #[test]
    fn beating_another_players_best_is_a_record() {
        assert!(beats_record(Some(90.0), None, 90.5));
        assert!(beats_record(Some(90.0), Some(80.0), 95.0));
        // Having tied the record before is not holding it alone.
        assert!(beats_record(Some(90.0), Some(90.0), 91.0));
    }

    #[test]
    fn a_tie_or_a_lower_result_is_not_a_record() {
        assert!(!beats_record(Some(90.0), None, 90.0));
        assert!(!beats_record(Some(90.0), Some(70.0), 89.0));
    }

    #[test]
    fn improving_ones_own_record_is_not_beating_it() {
        assert!(!beats_record(Some(90.0), Some(95.0), 98.0));
    }
}
