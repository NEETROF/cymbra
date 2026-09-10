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

//! The status a lemma can hold and where a "known" came from (design D1).

use serde::{Deserialize, Serialize};

/// Where a `known` classification originated. Recorded on every `known` so a
/// dubious bulk source (a LingQ import) stays identifiable and
/// bulk-correctable, and so the future SRS inference is distinguishable from
/// a manual decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum KnownSource {
    /// The user marked the word known by hand.
    Manual,
    /// Implicitly known because its frequency rank is at or below the
    /// calibration threshold. Never stored as an explicit entry — it is
    /// recomputed on every analysis — but reported as the provenance of an
    /// implicit "known".
    Calibration,
    /// Inferred from spaced-repetition performance. Written by
    /// `add-lingua-decks-review`; the variant is declared here so the schema
    /// is frozen from day 1.
    Srs,
    /// Brought in by a bulk import (e.g. a LingQ export). Declared to freeze
    /// the schema; the import path itself is deferred to a later change (no
    /// code writes this in the MVP), the same way `Srs` is declared here and
    /// wired by `add-lingua-decks-review`.
    Import,
}

/// The explicit status of a lemma. The absence of an entry means "new"; a
/// lemma below the calibration threshold is implicitly `Known(Calibration)`
/// without any stored entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Actively being learned (in a deck). Counts as NOT known.
    Learning,
    /// Known, with its provenance. Counts as known.
    Known(KnownSource),
    /// Deliberately ignored (proper nouns the user does not care to learn,
    /// noise). Counts as known so it stops being highlighted.
    Ignored,
}

impl Status {
    /// Whether this status counts as known in the page percentage
    /// (`Known` and `Ignored`; `Learning` does not).
    pub fn counts_as_known(self) -> bool {
        matches!(self, Status::Known(_) | Status::Ignored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_and_ignored_count_as_known_learning_does_not() {
        assert!(Status::Known(KnownSource::Manual).counts_as_known());
        assert!(Status::Known(KnownSource::Calibration).counts_as_known());
        assert!(Status::Ignored.counts_as_known());
        assert!(!Status::Learning.counts_as_known());
    }
}
