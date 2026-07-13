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

//! The pure MusicXML notation data model: a parsed score's metadata, staff
//! count, starting attributes, and ordered measures with notes/directions and
//! computed engraving geometry. No FFI, no IO — shared by the app engine, the
//! backend score module, and the crawler.
//!
//! All numeric fields use `u32`/`i32`/`f64` (never `u64`) so the app's
//! flutter_rust_bridge codegen avoids Dart `BigInt`.

/// A parsed MusicXML document: metadata, staff count, starting attributes, and
/// the ordered measures with their notes, directions, and computed geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreDocument {
    pub meta: ScoreMeta,
    /// Number of staves in the (single) part — e.g. 2 for a piano grand staff.
    pub staves: u32,
    pub attributes: Attributes,
    pub measures: Vec<NotationMeasure>,
}

/// Score metadata; fields are absent (`None`) rather than failing when missing.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreMeta {
    pub title: Option<String>,
    pub composer: Option<String>,
}

/// Starting musical attributes of the part (most-recent values win).
#[derive(Debug, Clone, PartialEq)]
pub struct Attributes {
    /// Divisions (ticks) per quarter note — the unit for every `duration`.
    pub divisions: u32,
    /// One clef per staff, identified by clef `number`.
    pub clefs: Vec<Clef>,
    /// Key signature, in fifths on the circle (negative = flats).
    pub key_fifths: i32,
    pub time: TimeSignature,
}

/// A clef on one staff: e.g. treble = `G`/2 on staff 1, bass = `F`/4 on staff 2.
#[derive(Debug, Clone, PartialEq)]
pub struct Clef {
    pub staff: u32,
    pub sign: char,
    pub line: i32,
}

/// A time signature, e.g. 3/4 → `beats = 3`, `beat_type = 4`.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeSignature {
    pub beats: u32,
    pub beat_type: u32,
}

/// A measure: its notes and directions in document order, plus the engraving
/// minimum width computed from note density.
#[derive(Debug, Clone, PartialEq)]
pub struct NotationMeasure {
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
#[derive(Debug, Clone, PartialEq)]
pub struct NoteEvent {
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
#[derive(Debug, Clone, PartialEq)]
pub struct Pitch {
    pub step: char,
    pub octave: i32,
    pub alter: i32,
}

/// Tuplet ratio from `time-modification` — e.g. a triplet is `3:2`.
#[derive(Debug, Clone, PartialEq)]
pub struct Tuplet {
    pub actual: u32,
    pub normal: u32,
}

/// Stem direction.
#[derive(Debug, Clone, PartialEq)]
pub enum StemDir {
    Up,
    Down,
}

/// Beam state at a note within a beam group.
#[derive(Debug, Clone, PartialEq)]
pub enum BeamState {
    Begin,
    Continue,
    End,
}

/// A lyric syllable attached to a note.
#[derive(Debug, Clone, PartialEq)]
pub struct Lyric {
    pub syllabic: Option<String>,
    pub text: String,
}

/// A measure direction (expression/tempo) anchored at a staff and time position.
#[derive(Debug, Clone, PartialEq)]
pub struct Direction {
    pub staff: u32,
    pub position_divisions: u32,
    pub kind: DirectionKind,
}

/// The supported direction kinds; unknown ones are dropped at parse time.
#[derive(Debug, Clone, PartialEq)]
pub enum DirectionKind {
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
#[derive(Debug, Clone, PartialEq)]
pub struct System {
    pub measures: Vec<u32>,
    pub staves: u32,
}
