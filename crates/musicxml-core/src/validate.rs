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

//! The client/server-shared validation gate.
//!
//! One function decides whether a byte buffer is an acceptable playable score,
//! returning a structured [`ScoreSummary`] on success or a typed
//! [`RejectReason`] on failure — so the app preview and the backend upload gate
//! agree on exactly what is accepted (no client/server drift). Accepts plain
//! MusicXML or a compressed `.mxl`; decodes the latter first.

use std::fmt;

use crate::meta::ScoreSummary;
use crate::model::ScoreDocument;
use crate::mxl;
use crate::parse;

/// Upper bound (bytes) on the raw input handed to [`validate`]. Piano MusicXML
/// is small; anything larger is rejected before any parsing work.
pub const MAX_INPUT: usize = 16 * 1024 * 1024;

/// Why a buffer was rejected. Typed so callers map each case to a specific
/// user-facing message or gRPC status without string matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// Raw input exceeds [`MAX_INPUT`].
    TooLarge,
    /// Looked like an `.mxl` but the container could not be decoded.
    Undecodable,
    /// The MusicXML could not be parsed.
    Unparseable,
    /// Parsed, but contains no playable (pitched, non-rest) notes.
    NoNotes,
}

impl RejectReason {
    /// A stable, machine-readable code (e.g. for a gRPC error detail).
    pub fn code(self) -> &'static str {
        match self {
            RejectReason::TooLarge => "too_large",
            RejectReason::Undecodable => "undecodable",
            RejectReason::Unparseable => "unparseable",
            RejectReason::NoNotes => "no_notes",
        }
    }
}

impl fmt::Display for RejectReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            RejectReason::TooLarge => "file is too large",
            RejectReason::Undecodable => "compressed .mxl could not be decoded",
            RejectReason::Unparseable => "file is not valid MusicXML",
            RejectReason::NoNotes => "score contains no playable notes",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for RejectReason {}

/// Validates raw bytes (plain MusicXML or `.mxl`) as a playable score.
///
/// Decodes an `.mxl` container when detected, parses the MusicXML, and confirms
/// at least one pitched note is present. Never panics.
pub fn validate(bytes: &[u8]) -> Result<ScoreSummary, RejectReason> {
    if bytes.len() > MAX_INPUT {
        return Err(RejectReason::TooLarge);
    }

    let xml = if mxl::is_mxl(bytes) {
        mxl::decode(bytes).map_err(|_| RejectReason::Undecodable)?
    } else {
        bytes.to_vec()
    };

    let doc = parse(&xml).map_err(|_| RejectReason::Unparseable)?;

    // Derive the full summary once (title/composer/key/time/piano facets); the
    // "playable notes" gate reuses its note count so there is no second pass.
    let summary = ScoreSummary::from_document(&doc);
    if summary.note_count == 0 {
        return Err(RejectReason::NoNotes);
    }
    Ok(summary)
}

/// Gate, decode a `.mxl` container when present, and fully parse `bytes` into a
/// [`ScoreDocument`]. The shared front door for consumers that need the parsed
/// document (notation layout, playback schedule, audio render): non-MusicXML /
/// oversized / undecodable input is a typed [`RejectReason`], never partial output.
/// Never panics.
pub fn decode_and_parse(bytes: &[u8]) -> Result<ScoreDocument, RejectReason> {
    validate(bytes)?;
    let xml = if mxl::is_mxl(bytes) {
        mxl::decode(bytes).map_err(|_| RejectReason::Undecodable)?
    } else {
        bytes.to_vec()
    };
    parse(&xml).map_err(|_| RejectReason::Unparseable)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>1</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note>
    </measure>
  </part>
</score-partwise>"#;

    const RESTS_ONLY: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>1</divisions></attributes>
      <note><rest/><duration>4</duration><type>whole</type></note>
    </measure>
  </part>
</score-partwise>"#;

    #[test]
    fn accepts_plain_musicxml() {
        let s = validate(MINIMAL.as_bytes()).unwrap();
        assert_eq!(s.note_count, 1);
        assert!(s.measure_count >= 1);
    }

    #[test]
    fn rejects_rest_only_score() {
        assert_eq!(validate(RESTS_ONLY.as_bytes()), Err(RejectReason::NoNotes));
    }

    #[test]
    fn rejects_oversized_input() {
        let big = vec![b' '; MAX_INPUT + 1];
        assert_eq!(validate(&big), Err(RejectReason::TooLarge));
    }

    #[test]
    fn rejects_undecodable_mxl() {
        // ZIP magic but not a valid archive.
        let fake = b"PK\x03\x04garbage".to_vec();
        assert_eq!(validate(&fake), Err(RejectReason::Undecodable));
    }

    #[test]
    fn rejects_unparseable_xml() {
        assert_eq!(
            validate(b"<score-partwise><unclosed"),
            Err(RejectReason::Unparseable)
        );
    }

    #[test]
    fn reject_reason_codes_are_stable() {
        assert_eq!(RejectReason::NoNotes.code(), "no_notes");
        assert_eq!(RejectReason::TooLarge.code(), "too_large");
    }

    #[test]
    fn every_reject_reason_has_code_and_message() {
        for r in [
            RejectReason::TooLarge,
            RejectReason::Undecodable,
            RejectReason::Unparseable,
            RejectReason::NoNotes,
        ] {
            // Machine code and human message are both non-empty and distinct.
            assert!(!r.code().is_empty());
            let msg = r.to_string();
            assert!(!msg.is_empty());
            assert_ne!(r.code(), msg);
        }
        assert_eq!(RejectReason::Undecodable.code(), "undecodable");
        assert_eq!(RejectReason::Unparseable.code(), "unparseable");
        assert_eq!(
            RejectReason::NoNotes.to_string(),
            "score contains no playable notes"
        );
    }

    /// The deliberate boundary of `add-unpitched-notation`: a percussion score
    /// parses fully, but the admission gate still refuses it — opening the
    /// gate belongs to `add-drums-access`, together with the access controls.
    /// This test is the explicit tripwire a later change must consciously move.
    #[test]
    fn percussion_score_is_still_refused_by_the_gate() {
        assert_eq!(
            validate(crate::fixtures::ROCK_GROOVE.as_bytes()),
            Err(RejectReason::NoNotes)
        );
        // …while the parser, called directly, yields the full document.
        let doc = crate::parse(crate::fixtures::ROCK_GROOVE.as_bytes()).unwrap();
        assert!(!doc.instruments.is_empty());
        assert!(doc.measures[0].notes.iter().any(|n| n.unpitched.is_some()));
    }

    #[test]
    fn summary_carries_derived_facets() {
        // The gate returns the shared summary — musical facets included.
        let s = validate(MINIMAL.as_bytes()).unwrap();
        assert_eq!(s.time_sig, "4/4");
        assert_eq!(s.key_fifths, 0);
        assert!(!s.is_piano); // single staff
    }
}
