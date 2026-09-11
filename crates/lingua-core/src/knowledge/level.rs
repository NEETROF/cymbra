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

//! CEFR levels and the per-lemma level source (`add-lingua-cefr-levels`).
//!
//! A level is one of the six CEFR bands, ordered A1 < A2 < B1 < B2 < C1 < C2.
//! Levels come from the pack (built from CEFR-J A1–B2 + Octanove C1–C2 for
//! English); a language pair without licence-clean CEFR data has no level
//! source and the knowledge model falls back to frequency-rank calibration.
//! The source is a trait so the module stays host-testable with an in-memory
//! double, exactly like [`super::state::FrequencyRanks`].

use serde::{Deserialize, Serialize};

/// A CEFR level. `Ord` follows the framework's progression, so
/// `CefrLevel::A2 < CefrLevel::B1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CefrLevel {
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
}

impl CefrLevel {
    /// The six levels in ascending order — the ladder's rows.
    pub const ALL: [CefrLevel; 6] = [
        CefrLevel::A1,
        CefrLevel::A2,
        CefrLevel::B1,
        CefrLevel::B2,
        CefrLevel::C1,
        CefrLevel::C2,
    ];

    /// The canonical two-character label (`"A1"`, …) used on the wire and in UI.
    pub fn label(self) -> &'static str {
        match self {
            CefrLevel::A1 => "A1",
            CefrLevel::A2 => "A2",
            CefrLevel::B1 => "B1",
            CefrLevel::B2 => "B2",
            CefrLevel::C1 => "C1",
            CefrLevel::C2 => "C2",
        }
    }

    /// Parses a label back into a level; `None` for anything else.
    pub fn from_label(s: &str) -> Option<CefrLevel> {
        match s {
            "A1" => Some(CefrLevel::A1),
            "A2" => Some(CefrLevel::A2),
            "B1" => Some(CefrLevel::B1),
            "B2" => Some(CefrLevel::B2),
            "C1" => Some(CefrLevel::C1),
            "C2" => Some(CefrLevel::C2),
            _ => None,
        }
    }
}

/// A per-lemma CEFR level source, backed by the pack's optional level table.
/// A lemma absent from every CEFR list (rarer than C2 vocabulary, or a proper
/// noun) has no level — it is never presumed known when a level is declared.
pub trait CefrLevels {
    /// The CEFR level of a lemma, or `None` if the pack carries no level for it
    /// (or carries no level table at all).
    fn level(&self, lemma: &str) -> Option<CefrLevel>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_are_ordered_by_the_framework() {
        assert!(CefrLevel::A1 < CefrLevel::A2);
        assert!(CefrLevel::B2 < CefrLevel::C1);
        assert_eq!(CefrLevel::ALL.len(), 6);
        assert!(CefrLevel::ALL.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn label_round_trips() {
        for level in CefrLevel::ALL {
            assert_eq!(CefrLevel::from_label(level.label()), Some(level));
        }
        assert_eq!(CefrLevel::from_label("D1"), None);
    }
}
