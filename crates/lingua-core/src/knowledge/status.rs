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
    /// Confirmed by reading exposure: a below-level lemma the reader met on
    /// enough distinct days without ever acting on it, promoted by the explicit
    /// exposure-promotion operation (`add-lingua-cefr-levels`). Kept distinct
    /// from `Manual`/`Calibration` so an exposure-inferred known stays
    /// identifiable and bulk-reversible.
    Exposure,
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

    /// The sync-protocol status string (`add-lingua-connected-clients`,
    /// `KnownWordsService`): `"known" | "learning" | "ignored"`. (`"cleared"`
    /// is the wire value for an *absent* status; see [`Status::from_wire`].)
    pub fn wire_kind(self) -> &'static str {
        match self {
            Status::Learning => "learning",
            Status::Known(_) => "known",
            Status::Ignored => "ignored",
        }
    }

    /// The sync-protocol provenance string:
    /// `"manual" | "srs" | "exposure" | "import"`. `Known(Calibration)` is never
    /// an explicit entry, so it never reaches the wire; a learning/ignored
    /// status is a manual decision. This client round-trips every value it emits
    /// (see [`Status::from_wire`]); a client that predates a provenance it has
    /// never heard of degrades that value back to `manual` — a lossy but safe
    /// degrade (it stays a known).
    pub fn wire_provenance(self) -> &'static str {
        match self {
            Status::Known(KnownSource::Srs) => "srs",
            Status::Known(KnownSource::Exposure) => "exposure",
            Status::Known(KnownSource::Import) => "import",
            _ => "manual",
        }
    }

    /// Parse a `(kind, provenance)` wire pair into a status, or `None` for
    /// `"cleared"` / an unrecognised kind (which the caller treats as a clear).
    pub fn from_wire(kind: &str, provenance: &str) -> Option<Status> {
        match kind {
            "learning" => Some(Status::Learning),
            "ignored" => Some(Status::Ignored),
            "known" => Some(Status::Known(match provenance {
                "srs" => KnownSource::Srs,
                "exposure" => KnownSource::Exposure,
                "import" => KnownSource::Import,
                _ => KnownSource::Manual,
            })),
            _ => None,
        }
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

    #[test]
    fn wire_kind_and_provenance_round_trip_through_from_wire() {
        for status in [
            Status::Learning,
            Status::Ignored,
            Status::Known(KnownSource::Manual),
            Status::Known(KnownSource::Srs),
            Status::Known(KnownSource::Exposure),
            Status::Known(KnownSource::Import),
        ] {
            let back = Status::from_wire(status.wire_kind(), status.wire_provenance());
            assert_eq!(back, Some(status), "round trip {status:?}");
        }
    }

    #[test]
    fn an_unknown_provenance_degrades_to_a_manual_known() {
        // D6 back-compat: a provenance this client has never heard of (e.g. one a
        // future client invents) stays a known rather than being dropped.
        assert_eq!(
            Status::from_wire("known", "something-new"),
            Some(Status::Known(KnownSource::Manual)),
        );
    }

    #[test]
    fn calibration_known_serialises_as_a_manual_known_on_the_wire() {
        // Calibration is never an explicit entry, so it collapses to manual if it
        // ever reaches the wire — never the reverse of a stored SRS/import known.
        let s = Status::Known(KnownSource::Calibration);
        assert_eq!(s.wire_kind(), "known");
        assert_eq!(s.wire_provenance(), "manual");
    }

    #[test]
    fn cleared_and_unknown_kinds_parse_to_none() {
        assert_eq!(Status::from_wire("cleared", "manual"), None);
        assert_eq!(Status::from_wire("bogus", "manual"), None);
    }
}
