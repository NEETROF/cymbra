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

use crate::mxl;
use crate::parse;

/// Upper bound (bytes) on the raw input handed to [`validate`]. Piano MusicXML
/// is small; anything larger is rejected before any parsing work.
pub const MAX_INPUT: usize = 16 * 1024 * 1024;

/// A successful validation's extracted summary — the fields worth surfacing to a
/// caller (preview header, upload record) without re-parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreSummary {
    pub title: Option<String>,
    pub composer: Option<String>,
    /// Number of staves (2 for a piano grand staff).
    pub staves: u32,
    pub measure_count: u32,
    /// Count of pitched (non-rest) note events — the "playable notes" check.
    pub note_count: u32,
}

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

    let note_count: u32 = doc
        .measures
        .iter()
        .flat_map(|m| &m.notes)
        .filter(|n| n.pitch.is_some() && !n.is_rest)
        .count() as u32;
    if note_count == 0 {
        return Err(RejectReason::NoNotes);
    }

    Ok(ScoreSummary {
        title: doc.meta.title,
        composer: doc.meta.composer,
        staves: doc.staves,
        measure_count: doc.measures.len() as u32,
        note_count,
    })
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
}
