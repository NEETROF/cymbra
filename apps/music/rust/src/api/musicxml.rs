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
    Attributes, BeamState, Clef, ClefSign, Direction, DirectionKind, InstrumentDecl,
    InstrumentKind, Lyric, NotationMeasure, NoteEvent, Pitch, PlayedMeasure, RepeatMarks,
    ScoreDocument, ScoreMeta, ScoreSummary, StemDir, System, TimeSignature, Tuplet, Unpitched,
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
    /// The part-list instrument table: one entry per declared
    /// `<score-instrument>`, carrying the General MIDI percussion number its
    /// `<midi-instrument>/<midi-unpitched>` denotes. Empty for a score
    /// declaring no instruments (the overwhelmingly common keyboard case).
    pub instruments: Vec<InstrumentDecl>,
    pub measures: Vec<NotationMeasure>,
    /// The **playback order**: the sequence of written-measure passes a
    /// performer would play, resolved from the repeat structure (repeat
    /// barlines, voltas, D.C./D.S. jumps) with safety caps — the written order
    /// one-to-one when the piece has no repeats or the structure is malformed.
    /// Computed once at parse time; every derivation (app timeline, browser
    /// preview, server audio render) consumes it instead of re-resolving.
    pub play_order: Vec<PlayedMeasure>,
}

/// One slot of the playback order: which written measure plays, and which pass
/// through it this is (1-based — a repeated measure has one slot per pass).
#[frb(mirror(PlayedMeasure))]
pub struct _PlayedMeasure {
    pub written_index: u32,
    pub pass: u32,
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
    pub sign: ClefSign,
    pub line: i32,
}

/// A clef sign the renderers act on. MusicXML defines seven signs; `TAB`,
/// `jianpu` and `none` are out of scope for a keyboard-and-drums product, and
/// an unrecognised sign leaves the staff at its default clef rather than
/// failing the parse.
#[frb(mirror(ClefSign))]
pub enum _ClefSign {
    G,
    F,
    C,
    Percussion,
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
    /// Key signature (fifths on the circle, negative = flats) in force during
    /// this measure, carried forward from the last `<key>` change. Lets the
    /// renderer draw the correct armure per system — and a key change where it
    /// differs from the previous measure — for a piece that modulates.
    pub key_fifths: i32,
    /// Minimum engraving width (pixels) from the non-linear spacing function.
    pub min_width: f64,
    /// Repeat notation engraved on this measure (barline repeats, voltas,
    /// measure-repeat `%`, segno/coda and jump semantics). All defaults when
    /// the measure carries none — the overwhelmingly common case.
    pub repeats: RepeatMarks,
}

/// The repeat structure engraved on one measure, in written order — everything
/// the unroll and the renderers need. Defaults mean "no repeat notation".
#[frb(mirror(RepeatMarks))]
pub struct _RepeatMarks {
    /// A forward repeat (`‖:`) opens this measure.
    pub forward: bool,
    /// A backward repeat (`:‖`) closes this measure: the `times` attribute
    /// (how many times the section is played in total; MusicXML default 2).
    /// 0 = no backward repeat.
    pub backward_times: u32,
    /// Volta numbers of an ending bracket **starting** at this measure
    /// (e.g. `[1]`, `[1, 2]`). Empty when none starts here.
    pub ending_start: Vec<u32>,
    /// An ending bracket **stops** (downward hook) at this measure.
    pub ending_stop: bool,
    /// An ending bracket ends open (discontinue) at this measure — the usual
    /// engraving of a final volta.
    pub ending_discontinue: bool,
    /// This is a measure-repeat (`%`) measure: the written measure whose
    /// content it replays (already resolved transitively for chained `%`
    /// runs). The measure's own note list stays empty — the sign is engraved,
    /// the referenced content is played.
    pub measure_repeat_of: Option<u32>,
    /// Slash count of the measure-repeat sign (1 = `%`).
    pub measure_repeat_slashes: u32,
    /// A segno sign is placed at this measure.
    pub segno: bool,
    /// A coda sign is placed at this measure.
    pub coda: bool,
    /// `<sound>` jump semantics attached to this measure's directions.
    pub sound_dacapo: bool,
    pub sound_dalsegno: bool,
    pub sound_tocoda: bool,
    pub sound_fine: bool,
    /// `<sound forward-repeat="yes">` — repeats are re-taken after the jump.
    pub sound_forward_repeat: bool,
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
    /// True when this note carries `<grace/>` — an ornamental small note that
    /// occupies no musical time (`duration_divisions` stays 0 and the cursor
    /// does not advance); playback gives it a short nominal duration stolen
    /// from just before its position.
    pub is_grace: bool,
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
    /// Written staff position of a percussion note (`<unpitched>`), with its
    /// resolved General MIDI number — a channel **distinct** from `pitch`: the
    /// written position is a staff placement, not a sounding pitch. A note is
    /// exactly one of pitched, unpitched, or a rest.
    pub unpitched: Option<Unpitched>,
    /// The note's `<instrument id>` reference when present.
    pub instrument_id: Option<String>,
}

/// A percussion note's written staff position and resolved sound.
#[frb(mirror(Unpitched))]
pub struct _Unpitched {
    /// `display-step`: the written staff placement's diatonic step (A–G).
    pub display_step: char,
    /// `display-octave`: the written staff placement's octave.
    pub display_octave: i32,
    /// The General MIDI percussion number (0-based, 35–81 for the standard
    /// kit) resolved from the part-list instrument table at parse time.
    /// `None` when the note's instrument could not be resolved; a consumer
    /// must omit such a note rather than fabricate a number.
    pub gm_number: Option<u32>,
}

/// One `<score-instrument>` declaration from the part list.
#[frb(mirror(InstrumentDecl))]
pub struct _InstrumentDecl {
    /// The declaration's `id`, referenced by a note's `<instrument id>`.
    pub id: String,
    /// `<instrument-name>` when declared (e.g. "Snare Drum").
    pub name: Option<String>,
    /// General MIDI percussion number (0-based) from `<midi-unpitched>`,
    /// already converted from the element's 1-based value.
    pub gm_number: Option<u32>,
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
    /// Which instrument family the score is written for, derived from the
    /// notation alone.
    pub instrument: InstrumentKind,
}

/// Which instrument family a score is written for, derived from its notation
/// alone — never from a filename, a part name, or any other external claim.
#[frb(mirror(InstrumentKind))]
pub enum _InstrumentKind {
    /// Every non-rest note is pitched.
    Keyboard,
    /// Every non-rest note is unpitched.
    Percussion,
    /// Mixed pitched and unpitched content, or no notes at all.
    Unknown,
}

/// Client-side validation outcome: on success the parsed [`ScoreSummary`]; on
/// failure a stable reject `code` (`too_large` / `undecodable` / `unparseable` /
/// `no_notes`). Exactly one field is set.
pub struct ValidationOutcome {
    pub summary: Option<ScoreSummary>,
    pub reject_code: Option<String>,
}

// --- FFI wrappers (delegate to the shared core) --------------------------

/// Parses a MusicXML document (bytes) into a [`ScoreDocument`], with each
/// measure's `min_width` already computed. A zipped `.mxl` container is decoded
/// first (same sniff as [`validate_musicxml`]), so the caller can pass the raw
/// picked file — plain `.musicxml`/`.xml` or `.mxl`. Errors on malformed input
/// rather than panicking.
pub fn parse_musicxml(bytes: Vec<u8>) -> Result<ScoreDocument> {
    let xml = if cymbra_musicxml_core::mxl::is_mxl(&bytes) {
        cymbra_musicxml_core::mxl::decode(&bytes)?
    } else {
        bytes
    };
    cymbra_musicxml_core::parse(&xml)
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
