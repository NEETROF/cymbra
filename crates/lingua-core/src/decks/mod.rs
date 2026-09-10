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

//! Decks, cards, FSRS review and whole-product backup/restore (spec
//! `lingua-decks-review`).
//!
//! A [`card::Card`] carries provenance and an FSRS review state; a
//! [`review::Deck`] holds one card per `(studied language, lemma)`, and a
//! [`review::ReviewSession`] walks the due cards, grading them
//! ([`fsrs`]) or marking them known (writing `Known(Srs)` into the knowledge
//! model). [`backup::LinguaState`] is the versioned root the surfaces persist
//! and back up losslessly. Pure, clock-injected, WASM-identical.

pub mod backup;
pub mod card;
pub mod fsrs;
pub mod review;

pub use backup::{BACKUP_SCHEMA_VERSION, LinguaState, RestoreError};
pub use card::{Card, EncounterSource, Media, MediaSource, Provenance, SyncPolicy};
pub use fsrs::{FsrsParams, Memory, Rating, ReviewState};
pub use review::{Deck, ReviewSession};
