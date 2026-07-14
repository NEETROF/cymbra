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

//! Thin flutter_rust_bridge seam for MusicXML notation.
//!
//! The notation data model and the pure parser/geometry now live in the shared,
//! FFI-free [`cymbra_musicxml_core`] crate (reused by the backend score module
//! and the score crawler). frb does not follow re-exports of external types, so
//! this file declares `#[frb(mirror(...))]` shells that make the codegen emit
//! the same Dart classes while the real types — and all logic — stay in the
//! crate. The bridge functions take/return the crate's own types directly (no
//! conversions), and the generated Dart API is unchanged. The mirrors carry the
//! same doc comments so the generated Dart documentation is unchanged too.
//!
//! Excluded from the coverage gate: the testable logic is counted in
//! `cymbra-musicxml-core`. All numeric fields use `u32`/`i32`/`f64` (never
//! `u64`) so the generated Dart avoids `BigInt`.

use anyhow::Result;
use flutter_rust_bridge::frb;

// The real types live in the shared crate; re-export so downstream Rust (and the
// bridge functions below) refer to them by these names.
pub use cymbra_musicxml_core::{
    Attributes, BeamState, Clef, Direction, DirectionKind, Lyric, NotationMeasure, NoteEvent,
    Pitch, ScoreDocument, ScoreMeta, ScoreSummary, StemDir, System, TimeSignature, Tuplet,
};

// --- frb mirrors of the shared model -------------------------------------
//
// Each mirror re-declares a crate type's public shape (and its doc comments) so
// the codegen generates its Dart class unchanged. They carry no logic and are
// never constructed; `mirror(T)` binds them to the real
// `cymbra_musicxml_core::T`.

/// A parsed MusicXML document: metadata, staff count, starting attributes, and
/// the ordered measures with their notes, directions, and computed geometry.
#[frb(mirror(ScoreDocument))]
pub struct _ScoreDocument {
    pub meta: ScoreMeta,
    /// Number of staves in the (single) part — e.g. 2 for a piano grand staff.
    pub staves: u32,
    pub attributes: Attributes,
    pub measures: Vec<NotationMeasure>,
}

/// Score metadata; fields are absent (`None`) rather than failing when missing.
#[frb(mirror(ScoreMeta))]
pub struct _ScoreMeta {
    pub title: Option<String>,
    pub composer: Option<String>,
}

/// Starting musical attributes of the part (most-recent values win).
#[frb(mirror(Attributes))]
pub struct _Attributes {
    /// Divisions (ticks) per quarter note — the unit for every `duration`.
    pub divisions: u32,
    /// One clef per staff, identified by clef `number`.
    pub clefs: Vec<Clef>,
    /// Key signature, in fifths on the circle (negative = flats).
    pub key_fifths: i32,
    pub time: TimeSignature,
}

/// A clef on one staff: e.g. treble = `G`/2 on staff 1, bass = `F`/4 on staff 2.
#[frb(mirror(Clef))]
pub struct _Clef {
    pub staff: u32,
    pub sign: char,
    pub line: i32,
}

/// A time signature, e.g. 3/4 → `beats = 3`, `beat_type = 4`.
#[frb(mirror(TimeSignature))]
pub struct _TimeSignature {
    pub beats: u32,
    pub beat_type: u32,
}

/// A measure: its notes and directions in document order, plus the engraving
/// minimum width computed from note density.
#[frb(mirror(NotationMeasure))]
pub struct _NotationMeasure {
    /// 0-based position in the part.
    pub index: u32,
    pub notes: Vec<NoteEvent>,
    pub directions: Vec<Direction>,
    /// Clef changes declared in this measure (empty when unchanged), so the
    /// renderer can switch clefs mid-piece (e.g. a left hand that starts in
    /// treble and moves to bass).
    pub clefs: Vec<Clef>,
    /// Minimum engraving width (pixels) from the non-linear spacing function.
    pub min_width: f64,
}

/// A single note (or rest) event.
#[frb(mirror(NoteEvent))]
pub struct _NoteEvent {
    pub staff: u32,
    pub voice: u32,
    /// Running time position within the measure (set via backup/forward), so
    /// notes on different staves/voices at the same beat share a column.
    pub position_divisions: u32,
    /// Pitch, or `None` for a rest.
    pub pitch: Option<Pitch>,
    pub is_rest: bool,
    /// True when this note carries `<chord/>` (sounds with the preceding note).
    pub is_chord: bool,
    pub duration_divisions: u32,
    /// Note-type token when present (e.g. "quarter", "eighth").
    pub note_type: Option<String>,
    pub dots: u32,
    /// Accidental token when present (e.g. "flat", "sharp", "natural").
    pub accidental: Option<String>,
    pub tie_start: bool,
    pub tie_stop: bool,
    /// Phrasing slur start/stop (legato arc spanning several notes).
    pub slur_start: bool,
    pub slur_stop: bool,
    pub tuplet: Option<Tuplet>,
    pub stem: Option<StemDir>,
    pub beams: Vec<BeamState>,
    pub lyric: Option<Lyric>,
}

/// A pitch: diatonic step, octave, and chromatic alteration (semitones).
#[frb(mirror(Pitch))]
pub struct _Pitch {
    pub step: char,
    pub octave: i32,
    pub alter: i32,
}

/// Tuplet ratio from `time-modification` — e.g. a triplet is `3:2`.
#[frb(mirror(Tuplet))]
pub struct _Tuplet {
    pub actual: u32,
    pub normal: u32,
}

/// Stem direction.
#[frb(mirror(StemDir))]
pub enum _StemDir {
    Up,
    Down,
}

/// Beam state at a note within a beam group.
#[frb(mirror(BeamState))]
pub enum _BeamState {
    Begin,
    Continue,
    End,
}

/// A lyric syllable attached to a note.
#[frb(mirror(Lyric))]
pub struct _Lyric {
    pub syllabic: Option<String>,
    pub text: String,
}

/// A measure direction (expression/tempo) anchored at a staff and time position.
#[frb(mirror(Direction))]
pub struct _Direction {
    pub staff: u32,
    pub position_divisions: u32,
    pub kind: DirectionKind,
}

/// The supported direction kinds; unknown ones are dropped at parse time.
#[frb(mirror(DirectionKind))]
pub enum _DirectionKind {
    /// Free expression/tempo text (e.g. "Andantino", "dolce").
    Words(String),
    /// A dynamics marking (e.g. "pp", "f").
    Dynamics(String),
    /// A hairpin: `crescendo` true = opening (<), false = diminuendo (>).
    /// `stop` marks the end of a previously opened hairpin.
    Wedge { crescendo: bool, stop: bool },
    /// A metronome mark, e.g. quarter = 120.
    Metronome { beat_unit: String, per_minute: u32 },
}

/// One staff line of music: the measure indices it contains, in order, plus the
/// staff count so a grand staff lays out together.
#[frb(mirror(System))]
pub struct _System {
    pub measures: Vec<u32>,
    pub staves: u32,
}

/// A parsed score's derived summary — the fields worth surfacing to a caller
/// (the contribution preview header) without re-parsing. Server-derived on
/// upload; this mirror lets the app show the same values it will store.
#[frb(mirror(ScoreSummary))]
pub struct _ScoreSummary {
    pub title: Option<String>,
    pub composer: Option<String>,
    /// Accent-/case-folded title for typo-tolerant search.
    pub title_norm: Option<String>,
    /// Normalised `composer::title` key for dedup / grouping.
    pub work_key: String,
    /// Grand-staff heuristic (`staves >= 2`) — a keyboard/piano proxy.
    pub is_piano: bool,
    /// Number of staves (2 for a piano grand staff).
    pub staves: u32,
    pub key_fifths: i32,
    /// `beats/beat_type`, e.g. `4/4`.
    pub time_sig: String,
    pub measure_count: u32,
    /// Count of pitched (non-rest) note events — the "playable notes" check.
    pub note_count: u32,
}

/// Client-side validation outcome: on success the parsed [`ScoreSummary`]; on
/// failure a stable reject `code` (`too_large` / `undecodable` / `unparseable` /
/// `no_notes`). Exactly one field is set.
pub struct ValidationOutcome {
    pub summary: Option<ScoreSummary>,
    pub reject_code: Option<String>,
}

// --- FFI wrappers (delegate to the shared core) --------------------------

/// Parses an uncompressed MusicXML document (bytes) into a [`ScoreDocument`],
/// with each measure's `min_width` already computed. Returns an error on
/// malformed input rather than panicking.
pub fn parse_musicxml(bytes: Vec<u8>) -> Result<ScoreDocument> {
    cymbra_musicxml_core::parse(&bytes)
}

/// Validates raw bytes (plain MusicXML or a zipped `.mxl`) client-side, using the
/// **same** shared gate as the backend upload path — so a file that previews here
/// is accepted server-side (and vice-versa). Never panics.
pub fn validate_musicxml(bytes: Vec<u8>) -> ValidationOutcome {
    match cymbra_musicxml_core::validate(&bytes) {
        Ok(summary) => ValidationOutcome {
            summary: Some(summary),
            reject_code: None,
        },
        Err(reason) => ValidationOutcome {
            summary: None,
            reject_code: Some(reason.code().to_string()),
        },
    }
}

/// Lays the document's measures out into [`System`]s for the given available
/// width (pixels), keeping measure order and the grand staff together.
#[frb(sync)]
pub fn layout_systems(doc: &ScoreDocument, available_width: f64) -> Vec<System> {
    cymbra_musicxml_core::layout_systems(doc, available_width)
}
