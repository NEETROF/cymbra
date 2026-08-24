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
//!
//! Under the optional `serde` feature the model derives `Serialize`/`Deserialize`
//! so a non-FFI consumer (the wasm notation renderer) can ship the geometry across
//! the WebAssembly boundary. The feature is off by default, so the app engine and
//! backend build unchanged.

/// A parsed MusicXML document: metadata, staff count, starting attributes, and
/// the ordered measures with their notes, directions, and computed geometry.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreDocument {
    pub meta: ScoreMeta,
    /// Number of staves in the (single) part — e.g. 2 for a piano grand staff.
    pub staves: u32,
    pub attributes: Attributes,
    /// The part-list instrument table: one entry per declared
    /// `<score-instrument>`, carrying the General MIDI percussion number its
    /// `<midi-instrument>/<midi-unpitched>` denotes. The only authoritative
    /// link between a written percussion note and the sound it denotes; empty
    /// for a score declaring no instruments (the overwhelmingly common
    /// keyboard case).
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayedMeasure {
    pub written_index: u32,
    pub pass: u32,
}

/// Score metadata; fields are absent (`None`) rather than failing when missing.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreMeta {
    pub title: Option<String>,
    pub composer: Option<String>,
}

/// Starting musical attributes of the part (most-recent values win).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Clef {
    pub staff: u32,
    pub sign: ClefSign,
    pub line: i32,
}

/// A clef sign the renderers act on. MusicXML defines seven signs; `TAB`,
/// `jianpu` and `none` are out of scope for a keyboard-and-drums product, and an
/// unrecognised sign leaves the staff at its default clef rather than failing
/// the parse — so this enum never needs a catch-all variant.
///
/// The single-letter variants serialize as `"G"`/`"F"`/`"C"`, exactly what the
/// former `char` field produced, so the wasm consumer's JSON is unchanged;
/// `Percussion` serializes as the MusicXML token `"percussion"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClefSign {
    G,
    F,
    C,
    #[cfg_attr(feature = "serde", serde(rename = "percussion"))]
    Percussion,
}

/// A time signature, e.g. 3/4 → `beats = 3`, `beat_type = 4`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeSignature {
    pub beats: u32,
    pub beat_type: u32,
}

/// A measure: its notes and directions in document order, plus the engraving
/// minimum width computed from note density.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NotationMeasure {
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
/// the unroll ([`crate::play_order`]) and the renderers need. `Default` means
/// "no repeat notation on this measure".
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RepeatMarks {
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
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// resolved General MIDI number. Held in a channel **distinct** from
    /// [`Self::pitch`]: the written position is a staff placement, not a
    /// sounding pitch, and exposing it as a `Pitch` would let a consumer
    /// compute a meaningless frequency from it. A note is exactly one of
    /// pitched, unpitched, or a rest.
    pub unpitched: Option<Unpitched>,
    /// The note's `<instrument id>` reference when present, kept so a consumer
    /// can reach the part-list declaration (e.g. for the instrument's name).
    pub instrument_id: Option<String>,
}

/// How an unpitched note's head is engraved (change: add-drum-notation-render).
///
/// Derived once here, beside the resolved General MIDI number, and carried to
/// both painters (serde → console wasm, frb → app) so neither re-derives head
/// classes from GM ranges of its own — two hand-maintained tables of the same
/// knowledge is exactly how independent painters drift. The app's kit-view
/// table stays the separate authority for the *gameplay* question (lanes and
/// pads); the cymbal overlap between the two is pinned by an app test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HeadClass {
    /// Ordinary oval head — the drums, and any unresolved note (the written
    /// position is authoritative even when the sound is not).
    Oval,
    /// X-form head — the cymbals, following the duration class like ordinary
    /// heads (filled x for quarter and shorter, open x forms above).
    X,
    /// X-form head carrying the conventional open mark (a small circle above
    /// the head) — the open hi-hat stroke, GM 46, so the open/closed
    /// distinction the file encodes as two GM numbers stays readable.
    XOpen,
}

/// The cymbal sounds of the standard kit (0-based General MIDI): hi-hats
/// (42/44/46 — 44, the pedal "chick", is a cymbal *sound* and engraves as x
/// by convention even though the kit view lanes it generically), crashes
/// (49/52/55/57), rides (51/53/59). Everything else — and every unresolved
/// note — takes the ordinary oval head.
const CYMBAL_GM: [u32; 10] = [42, 44, 46, 49, 51, 52, 53, 55, 57, 59];

/// GM 46, the open hi-hat — the one stroke that carries the open mark.
const OPEN_HI_HAT_GM: u32 = 46;

impl HeadClass {
    /// Classify a resolved General MIDI number (0-based) into its engraved
    /// head class; `None` (unresolved) takes the ordinary oval.
    pub fn of(gm_number: Option<u32>) -> Self {
        match gm_number {
            Some(OPEN_HI_HAT_GM) => HeadClass::XOpen,
            Some(gm) if CYMBAL_GM.contains(&gm) => HeadClass::X,
            _ => HeadClass::Oval,
        }
    }
}

/// A percussion note's written staff position and resolved sound.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Unpitched {
    /// `display-step`: the written staff placement's diatonic step (A–G).
    pub display_step: char,
    /// `display-octave`: the written staff placement's octave.
    pub display_octave: i32,
    /// The General MIDI percussion number (0-based, 35–81 for the standard
    /// kit) resolved from the part-list instrument table at parse time — the
    /// element value **minus one**, since MusicXML's `<midi-unpitched>` is
    /// 1-based. `None` when the note's instrument could not be resolved; a
    /// consumer must omit such a note rather than fabricate a number.
    pub gm_number: Option<u32>,
    /// The engraved head class ([`HeadClass::of`] the resolved number) — the
    /// painters consume this verbatim and never own GM ranges.
    pub head_class: HeadClass,
}

/// One `<score-instrument>` declaration from the part list.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InstrumentDecl {
    /// The declaration's `id`, referenced by a note's `<instrument id>`.
    pub id: String,
    /// `<instrument-name>` when declared (e.g. "Snare Drum").
    pub name: Option<String>,
    /// General MIDI percussion number (0-based) from `<midi-unpitched>`,
    /// already converted from the element's 1-based value. `None` when the
    /// declaration carries no `<midi-unpitched>`.
    pub gm_number: Option<u32>,
}

/// A pitch: diatonic step, octave, and chromatic alteration (semitones).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pitch {
    pub step: char,
    pub octave: i32,
    pub alter: i32,
}

/// Tuplet ratio from `time-modification` — e.g. a triplet is `3:2`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tuplet {
    pub actual: u32,
    pub normal: u32,
}

/// Stem direction.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StemDir {
    Up,
    Down,
}

/// Beam state at a note within a beam group.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BeamState {
    Begin,
    Continue,
    End,
}

/// A lyric syllable attached to a note.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lyric {
    pub syllabic: Option<String>,
    pub text: String,
}

/// A measure direction (expression/tempo) anchored at a staff and time position.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Direction {
    pub staff: u32,
    pub position_divisions: u32,
    pub kind: DirectionKind,
}

/// The supported direction kinds; unknown ones are dropped at parse time.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct System {
    pub measures: Vec<u32>,
    pub staves: u32,
}
