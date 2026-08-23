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

//! Pure, host-testable MusicXML parsing and engraving geometry — no FFI, no IO.
//!
//! The reusable engine crate: the notation data model ([`model`]), a streaming
//! (SAX-style) parser and engraving geometry (this module), compressed `.mxl`
//! decoding ([`mxl`]), and a client/server-shared [`validate`] gate. Consumed by
//! the app FFI engine (`apps/music/rust`), the backend score module, and the
//! score crawler. The parser consumes a large score as a stream of `quick-xml`
//! `Event`s with bounded memory, never as a full DOM.

pub mod meta;
pub mod model;
pub mod mxl;
mod pitch_alter;
pub mod playback;
pub mod repeats;
pub mod validate;

pub use meta::{InstrumentKind, ScoreFacets, ScoreSummary, instrument_of, normalize_text};
pub use model::*;
pub use playback::{
    DEFAULT_VELOCITY, DRUM_CHANNEL, MELODIC_CHANNEL, PlaybackSchedule, TimedNote, grace_ms_of,
    midi_of_pitch, schedule,
};
pub use repeats::play_order;
pub use validate::{RejectReason, decode_and_parse, validate};

use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

/// Percussion fixtures shared by the parser, meta, playback and validate
/// tests, so every module exercises the same notation. `<midi-unpitched>`
/// values are deliberately **1-based** (General MIDI number plus one), exactly
/// as conforming exporters write them: MuseScore writes 39 for the GM-38
/// snare, 43 for the GM-42 closed hi-hat, 37 for the GM-36 kick.
#[cfg(test)]
pub(crate) mod fixtures {
    /// Fixture 1.1: a two-measure rock groove in two voices — closed hi-hat
    /// eighths with snare chords on beats 2 and 4 (voice 1, stems up), kick on
    /// beats 1 and 3 (voice 2, stems down) — under a percussion clef, at
    /// 120 bpm with `divisions` 2 (an eighth = 1 division = 250 ms).
    pub(crate) const ROCK_GROOVE: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <work><work-title>Groove</work-title></work>
  <part-list>
    <score-part id="P1">
      <part-name>Drumset</part-name>
      <score-instrument id="P1-I36"><instrument-name>Bass Drum 1</instrument-name></score-instrument>
      <score-instrument id="P1-I38"><instrument-name>Snare Drum</instrument-name></score-instrument>
      <score-instrument id="P1-I42"><instrument-name>Closed Hi-hat</instrument-name></score-instrument>
      <midi-instrument id="P1-I36"><midi-channel>10</midi-channel><midi-unpitched>37</midi-unpitched></midi-instrument>
      <midi-instrument id="P1-I38"><midi-channel>10</midi-channel><midi-unpitched>39</midi-unpitched></midi-instrument>
      <midi-instrument id="P1-I42"><midi-channel>10</midi-channel><midi-unpitched>43</midi-unpitched></midi-instrument>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>2</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>percussion</sign><line>2</line></clef>
      </attributes>
      <direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>120</per-minute></metronome></direction-type></direction>
      <note><unpitched><display-step>G</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I42"/><voice>1</voice><type>eighth</type><stem>up</stem></note>
      <note><unpitched><display-step>G</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I42"/><voice>1</voice><type>eighth</type><stem>up</stem></note>
      <note><unpitched><display-step>G</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I42"/><voice>1</voice><type>eighth</type><stem>up</stem></note>
      <note><chord/><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I38"/><voice>1</voice><type>eighth</type><stem>up</stem></note>
      <note><unpitched><display-step>G</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I42"/><voice>1</voice><type>eighth</type><stem>up</stem></note>
      <note><unpitched><display-step>G</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I42"/><voice>1</voice><type>eighth</type><stem>up</stem></note>
      <note><unpitched><display-step>G</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I42"/><voice>1</voice><type>eighth</type><stem>up</stem></note>
      <note><unpitched><display-step>G</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I42"/><voice>1</voice><type>eighth</type><stem>up</stem></note>
      <note><chord/><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I38"/><voice>1</voice><type>eighth</type><stem>up</stem></note>
      <note><unpitched><display-step>G</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I42"/><voice>1</voice><type>eighth</type><stem>up</stem></note>
      <backup><duration>8</duration></backup>
      <note><unpitched><display-step>F</display-step><display-octave>4</display-octave></unpitched><duration>2</duration><instrument id="P1-I36"/><voice>2</voice><type>quarter</type><stem>down</stem></note>
      <note><rest/><duration>2</duration><voice>2</voice><type>quarter</type></note>
      <note><unpitched><display-step>F</display-step><display-octave>4</display-octave></unpitched><duration>2</duration><instrument id="P1-I36"/><voice>2</voice><type>quarter</type><stem>down</stem></note>
      <note><rest/><duration>2</duration><voice>2</voice><type>quarter</type></note>
    </measure>
    <measure number="2">
      <note><unpitched><display-step>F</display-step><display-octave>4</display-octave></unpitched><duration>2</duration><instrument id="P1-I36"/><voice>2</voice><type>quarter</type><stem>down</stem></note>
      <note><rest/><duration>6</duration><voice>2</voice><type>half</type><dot/></note>
    </measure>
  </part>
</score-partwise>"#;

    /// Fixture 1.2, the degraded cases: an unpitched note whose
    /// `<instrument id>` is absent from the part list, next to a resolvable
    /// one — under a clef sign the parser does not recognise.
    pub(crate) const DEGRADED: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list>
    <score-part id="P1">
      <score-instrument id="P1-I38"><instrument-name>Snare Drum</instrument-name></score-instrument>
      <score-instrument id="P1-I42"><instrument-name>Closed Hi-hat</instrument-name></score-instrument>
      <midi-instrument id="P1-I38"><midi-unpitched>39</midi-unpitched></midi-instrument>
      <midi-instrument id="P1-I42"><midi-unpitched>43</midi-unpitched></midi-instrument>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>2</divisions><time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>TAB</sign><line>5</line></clef>
      </attributes>
      <direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>120</per-minute></metronome></direction-type></direction>
      <note><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched><duration>2</duration><instrument id="P1-I38"/><voice>1</voice><type>quarter</type></note>
      <note><unpitched><display-step>E</display-step><display-octave>5</display-octave></unpitched><duration>2</duration><instrument id="P9-MISSING"/><voice>1</voice><type>quarter</type></note>
      <note><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched><duration>2</duration><instrument id="P1-I38"/><voice>1</voice><type>quarter</type></note>
      <note><rest/><duration>2</duration><voice>1</voice><type>quarter</type></note>
    </measure>
  </part>
</score-partwise>"#;

    /// A single-instrument part (tambourine) whose notes omit the
    /// `<instrument>` reference — the MusicXML default-instrument convention.
    /// One note is written as an empty `<unpitched/>` (middle-line placement).
    pub(crate) const SOLE_INSTRUMENT: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list>
    <score-part id="P1">
      <score-instrument id="P1-I54"><instrument-name>Tambourine</instrument-name></score-instrument>
      <midi-instrument id="P1-I54"><midi-unpitched>55</midi-unpitched></midi-instrument>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>1</divisions><time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>percussion</sign><line>2</line></clef>
      </attributes>
      <note><unpitched><display-step>D</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><voice>1</voice><type>quarter</type></note>
      <note><unpitched/><duration>1</duration><voice>1</voice><type>quarter</type></note>
    </measure>
  </part>
</score-partwise>"#;

    /// A crash cymbal whole note tied across the barline, then a fresh attack:
    /// the chain must merge into one prolonged note, the fresh attack must not.
    pub(crate) const TIED_CYMBAL: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list>
    <score-part id="P1">
      <score-instrument id="P1-I49"><instrument-name>Crash Cymbal 1</instrument-name></score-instrument>
      <midi-instrument id="P1-I49"><midi-unpitched>50</midi-unpitched></midi-instrument>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>2</divisions><time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>percussion</sign><line>2</line></clef>
      </attributes>
      <direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>120</per-minute></metronome></direction-type></direction>
      <note><unpitched><display-step>A</display-step><display-octave>5</display-octave></unpitched><duration>8</duration><tie type="start"/><instrument id="P1-I49"/><voice>1</voice><type>whole</type></note>
    </measure>
    <measure number="2">
      <note><unpitched><display-step>A</display-step><display-octave>5</display-octave></unpitched><duration>8</duration><tie type="stop"/><instrument id="P1-I49"/><voice>1</voice><type>whole</type></note>
    </measure>
    <measure number="3">
      <note><unpitched><display-step>A</display-step><display-octave>5</display-octave></unpitched><duration>8</duration><instrument id="P1-I49"/><voice>1</voice><type>whole</type></note>
    </measure>
  </part>
</score-partwise>"#;

    /// A *mixed* score — pitched notes plus a stray unpitched one — which is
    /// admissible through today's validation gate: its unpitched note must
    /// keep being skipped by the schedules, and its classification is unknown.
    pub(crate) const MIXED: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list>
    <score-part id="P1">
      <score-instrument id="P1-I38"/>
      <midi-instrument id="P1-I38"><midi-unpitched>39</midi-unpitched></midi-instrument>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>1</divisions><time><beats>4</beats><beat-type>4</beat-type></time></attributes>
      <direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>120</per-minute></metronome></direction-type></direction>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><voice>1</voice><type>quarter</type></note>
      <note><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id="P1-I38"/><voice>1</voice><type>quarter</type></note>
      <note><pitch><step>E</step><octave>4</octave></pitch><duration>1</duration><voice>1</voice><type>quarter</type></note>
    </measure>
  </part>
</score-partwise>"#;
}

// --- Geometry constants --------------------------------------------------

/// Base width (px) allotted to a quarter-note time column.
pub(crate) const UNIT: f64 = 30.0;
/// Spacing exponent (< 1 ⇒ sub-linear: long notes grow slower than duration).
pub(crate) const K: f64 = 0.6;
/// Minimum width (px) a measure may ever have (e.g. a whole-measure rest).
pub(crate) const FLOOR: f64 = 60.0;
/// Fixed left padding (px) for clef/key/time at the head of a measure.
pub(crate) const LEFT_PAD: f64 = 20.0;
/// Minimum advance (px) of any time column, however short its duration — keeps
/// runs of 16ths/32nds readable instead of collapsing toward glyph width.
pub(crate) const MIN_COL: f64 = 22.0;

/// Non-linear spacing for a single time column of `duration` divisions.
///
/// `space(d) = UNIT * (d / divisions) ^ K`. With `K < 1` the growth is
/// sub-linear: doubling a note's duration grows its space by `2^K < 2`. The
/// result never drops below [`MIN_COL`], so very short columns keep a legible
/// advance (the floor only binds below a quarter note).
pub(crate) fn space(duration: u32, divisions: u32) -> f64 {
    let div = divisions.max(1) as f64;
    let d = duration.max(1) as f64;
    (UNIT * (d / div).powf(K)).max(MIN_COL)
}

/// Minimum engraving width of a measure: the left pad plus the summed spacing
/// over the *union* of distinct time positions across all staves and voices
/// (chord members and simultaneous notes share one column), clamped to [`FLOOR`].
pub(crate) fn min_width(notes: &[NoteEvent], divisions: u32) -> f64 {
    // position → shortest duration starting at that position (drives the column).
    let mut columns: BTreeMap<u32, u32> = BTreeMap::new();
    for n in notes {
        let d = n.duration_divisions.max(1);
        columns
            .entry(n.position_divisions)
            .and_modify(|e| {
                if d < *e {
                    *e = d;
                }
            })
            .or_insert(d);
    }
    let sum: f64 = columns.values().map(|&d| space(d, divisions)).sum();
    (LEFT_PAD + sum).max(FLOOR)
}

/// Upper bound on measures per system, so dense scores stay legible even on a
/// wide viewport (a real engraving of e.g. the Arabesque uses ~3 per line).
pub(crate) const MAX_MEASURES_PER_SYSTEM: usize = 3;

/// Greedily packs measures into systems for `available_width`, wrapping when the
/// next measure overflows the line *or* the per-system cap is reached. A measure
/// wider than the line gets its own system. Order preserved.
pub fn layout_systems(doc: &ScoreDocument, available_width: f64) -> Vec<System> {
    let staves = doc.staves;
    let mut systems: Vec<System> = Vec::new();
    let mut current: Vec<u32> = Vec::new();
    let mut current_w = 0.0_f64;

    for m in &doc.measures {
        let w = m.min_width;
        let overflow = !current.is_empty() && current_w + w > available_width;
        let at_cap = current.len() >= MAX_MEASURES_PER_SYSTEM;
        if overflow || at_cap {
            systems.push(System {
                measures: std::mem::take(&mut current),
                staves,
            });
            current_w = 0.0;
        }
        current.push(m.index);
        current_w += w;
        // An oversized lone measure occupies its own system.
        if current.len() == 1 && current_w > available_width {
            systems.push(System {
                measures: std::mem::take(&mut current),
                staves,
            });
            current_w = 0.0;
        }
    }
    if !current.is_empty() {
        systems.push(System {
            measures: current,
            staves,
        });
    }
    systems
}

// --- Streaming parser ----------------------------------------------------

/// Parses an uncompressed MusicXML document into a [`ScoreDocument`], filling
/// each measure's `min_width`. Malformed XML is a recoverable [`Err`], never a
/// panic; well-formed non-MusicXML yields an empty document.
pub fn parse(input: &[u8]) -> Result<ScoreDocument> {
    let mut reader = Reader::from_reader(input);
    reader.config_mut().trim_text(true);

    let mut p = Parser::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name().as_ref().to_vec();
                p.on_open(&name, &e);
                p.text.clear();
            }
            Ok(Event::Empty(e)) => {
                let name = e.name().as_ref().to_vec();
                p.on_open(&name, &e);
            }
            Ok(Event::Text(e)) => {
                let t = e.unescape().map(|c| c.into_owned()).unwrap_or_default();
                p.text.push_str(&t);
            }
            Ok(Event::End(e)) => {
                let name = e.name().as_ref().to_vec();
                let text = std::mem::take(&mut p.text);
                p.on_close(&name, text.trim());
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(anyhow!("malformed MusicXML: {e}")),
        }
        buf.clear();
    }

    Ok(p.into_document())
}

/// Mutable parsing state threaded through the event loop — an internal
/// implementation detail, never part of any public surface.
struct Parser {
    text: String,
    // metadata
    title: Option<String>,
    composer: Option<String>,
    creator_is_composer: bool,
    // attributes (most-recent wins)
    staves: Option<u32>,
    divisions: u32,
    clefs: Vec<Clef>,
    key_fifths: i32,
    beats: u32,
    beat_type: u32,
    // part-list instrument declarations (id → General MIDI number)
    instruments: Vec<InstrumentDecl>,
    /// `id` of the `<midi-instrument>` currently open, so its
    /// `<midi-unpitched>` lands on the right declaration.
    midi_instrument_id: Option<String>,
    // parts / measures
    current_part: u32,
    measure_index: u32,
    cursor: u32,
    last_onset: u32,
    notes: Vec<NoteEvent>,
    directions: Vec<Direction>,
    measure_clefs: Vec<Clef>,
    measures: Vec<NotationMeasure>,
    /// Whether any note carried an explicit `<alter>` or `<accidental>`. When this
    /// stays false for the whole document, the score has no alteration data and the
    /// key signature is inferred onto every note (see `into_document`).
    saw_alteration: bool,
    // per-element builders
    note: Option<NoteEvent>,
    pitch: Option<Pitch>,
    unpitched: Option<Unpitched>,
    clef_number: u32,
    /// `None` when the declared sign is one the parser does not act on
    /// (`TAB`, `jianpu`, `none`, or garbage): the clef is then not recorded and
    /// the staff keeps its default, rather than failing the parse.
    clef_sign: Option<ClefSign>,
    clef_line: i32,
    tuplet_actual: u32,
    tuplet_normal: u32,
    direction: Option<Direction>,
    direction_has_kind: bool,
    metro_unit: String,
    metro_pm: u32,
    in_metro: bool,
    lyric_syllabic: Option<String>,
    lyric_text: String,
    in_lyric: bool,
    in_dynamics: bool,
    in_backup: bool,
    in_forward: bool,
    /// Repeat notation collected for the current measure (taken per measure).
    repeat_marks: RepeatMarks,
    /// Active `<measure-repeat>` run: (group length in measures, slashes).
    /// Persists across measures until the `stop` attribute clears it.
    measure_repeat: Option<(u32, u32)>,
    /// A `<measure-repeat type="start">` is open — its text is the length.
    in_measure_repeat_start: bool,
}

impl Parser {
    fn new() -> Self {
        Parser {
            text: String::new(),
            title: None,
            composer: None,
            creator_is_composer: false,
            staves: None,
            divisions: 1,
            clefs: Vec::new(),
            key_fifths: 0,
            beats: 4,
            beat_type: 4,
            current_part: 0,
            measure_index: 0,
            cursor: 0,
            last_onset: 0,
            notes: Vec::new(),
            directions: Vec::new(),
            measure_clefs: Vec::new(),
            measures: Vec::new(),
            saw_alteration: false,
            instruments: Vec::new(),
            midi_instrument_id: None,
            note: None,
            pitch: None,
            unpitched: None,
            clef_number: 1,
            clef_sign: Some(ClefSign::G),
            clef_line: 2,
            tuplet_actual: 0,
            tuplet_normal: 0,
            direction: None,
            direction_has_kind: false,
            metro_unit: String::new(),
            metro_pm: 0,
            in_metro: false,
            lyric_syllabic: None,
            lyric_text: String::new(),
            in_lyric: false,
            in_dynamics: false,
            repeat_marks: RepeatMarks::default(),
            measure_repeat: None,
            in_measure_repeat_start: false,
            in_backup: false,
            in_forward: false,
        }
    }
}

/// First attribute value matching `key`, as an owned `String`.
fn attr(e: &BytesStart, key: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

impl Parser {
    fn on_open(&mut self, name: &[u8], e: &BytesStart) {
        match name {
            b"part" => self.current_part += 1,
            b"note" => {
                self.note = Some(NoteEvent {
                    staff: 1,
                    voice: 1,
                    position_divisions: 0,
                    pitch: None,
                    is_rest: false,
                    is_chord: false,
                    is_grace: false,
                    duration_divisions: 0,
                    note_type: None,
                    dots: 0,
                    accidental: None,
                    tie_start: false,
                    tie_stop: false,
                    slur_start: false,
                    slur_stop: false,
                    tuplet: None,
                    stem: None,
                    beams: Vec::new(),
                    lyric: None,
                    unpitched: None,
                    instrument_id: None,
                });
            }
            b"chord" => {
                if let Some(n) = self.note.as_mut() {
                    n.is_chord = true;
                }
            }
            b"grace" => {
                if let Some(n) = self.note.as_mut() {
                    n.is_grace = true;
                }
            }
            b"rest" => {
                if let Some(n) = self.note.as_mut() {
                    n.is_rest = true;
                }
            }
            b"dot" => {
                if let Some(n) = self.note.as_mut() {
                    n.dots += 1;
                }
            }
            b"tie" => {
                if let Some(n) = self.note.as_mut() {
                    match attr(e, b"type").as_deref() {
                        Some("start") => n.tie_start = true,
                        Some("stop") => n.tie_stop = true,
                        _ => {}
                    }
                }
            }
            // Phrasing slur (in <notations>), distinct from a tie.
            b"slur" => {
                if let Some(n) = self.note.as_mut() {
                    match attr(e, b"type").as_deref() {
                        Some("start") => n.slur_start = true,
                        Some("stop") => n.slur_stop = true,
                        _ => {}
                    }
                }
            }
            b"pitch" => {
                self.pitch = Some(Pitch {
                    step: 'C',
                    octave: 4,
                    alter: 0,
                });
            }
            // A percussion note's written position. `display-step`/`display-octave`
            // are optional: an empty `<unpitched/>` denotes the middle staff line
            // (B4 in treble-equivalent numbering), per the MusicXML default.
            b"unpitched" => {
                self.unpitched = Some(Unpitched {
                    display_step: 'B',
                    display_octave: 4,
                    gm_number: None,
                    head_class: HeadClass::Oval,
                });
            }
            // A note's `<instrument id>` reference (an empty element).
            b"instrument" => {
                if let Some(n) = self.note.as_mut() {
                    n.instrument_id = attr(e, b"id");
                }
            }
            b"score-instrument" => {
                if let Some(id) = attr(e, b"id") {
                    self.instruments.push(InstrumentDecl {
                        id,
                        name: None,
                        gm_number: None,
                    });
                }
            }
            b"midi-instrument" => {
                self.midi_instrument_id = attr(e, b"id");
            }
            b"clef" => {
                self.clef_number = attr(e, b"number").and_then(|s| s.parse().ok()).unwrap_or(1);
                self.clef_sign = Some(ClefSign::G);
                self.clef_line = 2;
            }
            b"time-modification" => {
                self.tuplet_actual = 0;
                self.tuplet_normal = 0;
            }
            b"direction" => {
                self.direction = Some(Direction {
                    staff: 1,
                    position_divisions: self.cursor,
                    kind: DirectionKind::Words(String::new()),
                });
                self.direction_has_kind = false;
            }
            b"dynamics" => self.in_dynamics = true,
            b"wedge" => {
                if let Some(d) = self.direction.as_mut() {
                    match attr(e, b"type").as_deref() {
                        Some("crescendo") => {
                            d.kind = DirectionKind::Wedge {
                                crescendo: true,
                                stop: false,
                            };
                            self.direction_has_kind = true;
                        }
                        Some("diminuendo") => {
                            d.kind = DirectionKind::Wedge {
                                crescendo: false,
                                stop: false,
                            };
                            self.direction_has_kind = true;
                        }
                        Some("stop") => {
                            d.kind = DirectionKind::Wedge {
                                crescendo: false,
                                stop: true,
                            };
                            self.direction_has_kind = true;
                        }
                        _ => {}
                    }
                }
            }
            b"metronome" => {
                self.in_metro = true;
                self.metro_unit = String::new();
                self.metro_pm = 0;
            }
            b"lyric" => {
                self.in_lyric = true;
                self.lyric_syllabic = None;
                self.lyric_text = String::new();
            }
            b"backup" => self.in_backup = true,
            b"forward" => self.in_forward = true,
            b"repeat" => match attr(e, b"direction").as_deref() {
                Some("forward") => self.repeat_marks.forward = true,
                Some("backward") => {
                    self.repeat_marks.backward_times = attr(e, b"times")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(2)
                        .max(1);
                }
                _ => {}
            },
            b"ending" => {
                let numbers: Vec<u32> = attr(e, b"number")
                    .map(|s| {
                        s.split(',')
                            .filter_map(|part| part.trim().parse().ok())
                            .collect()
                    })
                    .unwrap_or_default();
                match attr(e, b"type").as_deref() {
                    Some("start") if !numbers.is_empty() => {
                        self.repeat_marks.ending_start = numbers;
                    }
                    Some("stop") => self.repeat_marks.ending_stop = true,
                    Some("discontinue") => self.repeat_marks.ending_discontinue = true,
                    _ => {}
                }
            }
            b"measure-repeat" if self.current_part <= 1 => {
                match attr(e, b"type").as_deref() {
                    Some("start") => {
                        self.in_measure_repeat_start = true;
                        let slashes = attr(e, b"slashes")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1);
                        // Length (element text, 1 or 2 measures) lands on close;
                        // an empty element keeps the default of 1.
                        self.measure_repeat = Some((1, slashes));
                    }
                    Some("stop") => self.measure_repeat = None,
                    _ => {}
                }
            }
            b"segno" => self.repeat_marks.segno = true,
            b"coda" => self.repeat_marks.coda = true,
            b"sound" => {
                if attr(e, b"dacapo").as_deref() == Some("yes") {
                    self.repeat_marks.sound_dacapo = true;
                }
                if attr(e, b"dalsegno").is_some() {
                    self.repeat_marks.sound_dalsegno = true;
                }
                if attr(e, b"tocoda").is_some() {
                    self.repeat_marks.sound_tocoda = true;
                }
                if attr(e, b"fine").is_some() {
                    self.repeat_marks.sound_fine = true;
                }
                if attr(e, b"forward-repeat").as_deref() == Some("yes") {
                    self.repeat_marks.sound_forward_repeat = true;
                }
            }
            b"creator" => {
                self.creator_is_composer = attr(e, b"type").as_deref() == Some("composer");
            }
            // A dynamics value is an empty child element of <dynamics> (e.g. <pp/>).
            other if self.in_dynamics => {
                if let Some(d) = self.direction.as_mut() {
                    d.kind = DirectionKind::Dynamics(String::from_utf8_lossy(other).into_owned());
                    self.direction_has_kind = true;
                }
            }
            _ => {}
        }
    }

    fn on_close(&mut self, name: &[u8], text: &str) {
        match name {
            b"work-title" => {
                if !text.is_empty() {
                    self.title = Some(text.to_string());
                }
            }
            b"creator" => {
                if self.creator_is_composer && !text.is_empty() {
                    self.composer = Some(text.to_string());
                }
                self.creator_is_composer = false;
            }
            b"divisions" => {
                if let Ok(v) = text.parse() {
                    self.divisions = v;
                }
            }
            b"staves" => self.staves = text.parse().ok(),
            b"fifths" => {
                if let Ok(v) = text.parse() {
                    self.key_fifths = v;
                }
            }
            b"beats" => {
                if let Ok(v) = text.parse() {
                    self.beats = v;
                }
            }
            b"beat-type" => {
                if let Ok(v) = text.parse() {
                    self.beat_type = v;
                }
            }
            b"sign" => {
                // The full token, not its first character: `percussion` is a
                // multi-character sign. An unrecognised sign (TAB, jianpu,
                // none, garbage) yields `None` — the clef is then simply not
                // recorded, leaving the staff at its default.
                self.clef_sign = match text {
                    "G" => Some(ClefSign::G),
                    "F" => Some(ClefSign::F),
                    "C" => Some(ClefSign::C),
                    "percussion" => Some(ClefSign::Percussion),
                    _ => None,
                };
            }
            b"line" => {
                if let Ok(v) = text.parse() {
                    self.clef_line = v;
                }
            }
            b"clef" => {
                let Some(sign) = self.clef_sign else {
                    return;
                };
                let clef = Clef {
                    staff: self.clef_number,
                    sign,
                    line: self.clef_line,
                };
                // Record this measure's clef change (replace by staff).
                if let Some(existing) = self
                    .measure_clefs
                    .iter_mut()
                    .find(|c| c.staff == clef.staff)
                {
                    *existing = clef.clone();
                } else {
                    self.measure_clefs.push(clef.clone());
                }
                // The document's `attributes.clefs` keeps the *initial* clef per
                // staff (first occurrence); later changes live on the measure.
                if !self.clefs.iter().any(|c| c.staff == clef.staff) {
                    self.clefs.push(clef);
                }
            }
            b"step" => {
                if let Some(pt) = self.pitch.as_mut() {
                    pt.step = text.chars().next().unwrap_or('C');
                }
            }
            b"octave" => {
                if let (Some(pt), Ok(v)) = (self.pitch.as_mut(), text.parse()) {
                    pt.octave = v;
                }
            }
            b"alter" => {
                if let (Some(pt), Ok(v)) = (self.pitch.as_mut(), text.parse()) {
                    pt.alter = v;
                    self.saw_alteration = true;
                }
            }
            b"pitch" => {
                if let Some(n) = self.note.as_mut() {
                    n.pitch = self.pitch.take();
                }
            }
            b"display-step" => {
                if let (Some(u), Some(c)) = (self.unpitched.as_mut(), text.chars().next()) {
                    u.display_step = c;
                }
            }
            b"display-octave" => {
                if let (Some(u), Ok(v)) = (self.unpitched.as_mut(), text.parse()) {
                    u.display_octave = v;
                }
            }
            b"unpitched" => {
                if let Some(n) = self.note.as_mut() {
                    n.unpitched = self.unpitched.take();
                }
            }
            b"instrument-name" => {
                if let (Some(decl), false) = (self.instruments.last_mut(), text.is_empty()) {
                    decl.name = Some(text.to_string());
                }
            }
            // MusicXML's `<midi-unpitched>` is **1-based** (1–128): conforming
            // exporters write the General MIDI number plus one (MuseScore writes
            // 39 for the GM-38 snare), so the stored number is the element value
            // minus one. An out-of-range value is ignored rather than guessed.
            b"midi-unpitched" => {
                if let Ok(v) = text.parse::<u32>()
                    && (1..=128).contains(&v)
                {
                    let id = self.midi_instrument_id.as_deref();
                    if let Some(decl) = match id {
                        Some(id) => self.instruments.iter_mut().find(|d| d.id == id),
                        None => self.instruments.last_mut(),
                    } {
                        decl.gm_number = Some(v - 1);
                    }
                }
            }
            b"midi-instrument" => self.midi_instrument_id = None,
            b"duration" => {
                if let Ok(v) = text.parse::<u32>() {
                    if self.in_backup {
                        self.cursor = self.cursor.saturating_sub(v);
                    } else if self.in_forward {
                        self.cursor = self.cursor.saturating_add(v);
                    } else if let Some(n) = self.note.as_mut() {
                        n.duration_divisions = v;
                    }
                }
            }
            b"type" => {
                if let Some(n) = self.note.as_mut() {
                    n.note_type = Some(text.to_string());
                }
            }
            b"accidental" => {
                if let Some(n) = self.note.as_mut() {
                    n.accidental = Some(text.to_string());
                    self.saw_alteration = true;
                }
            }
            b"voice" => {
                if let (Some(n), Ok(v)) = (self.note.as_mut(), text.parse()) {
                    n.voice = v;
                }
            }
            b"staff" => {
                if let Ok(v) = text.parse() {
                    if let Some(n) = self.note.as_mut() {
                        n.staff = v;
                    } else if let Some(d) = self.direction.as_mut() {
                        d.staff = v;
                    }
                }
            }
            b"stem" => {
                if let Some(n) = self.note.as_mut() {
                    n.stem = match text {
                        "up" => Some(StemDir::Up),
                        "down" => Some(StemDir::Down),
                        _ => None,
                    };
                }
            }
            b"beam" => {
                if let Some(n) = self.note.as_mut() {
                    let state = match text {
                        "begin" => Some(BeamState::Begin),
                        "continue" => Some(BeamState::Continue),
                        "end" => Some(BeamState::End),
                        _ => None,
                    };
                    if let Some(s) = state {
                        n.beams.push(s);
                    }
                }
            }
            b"actual-notes" => {
                if let Ok(v) = text.parse() {
                    self.tuplet_actual = v;
                }
            }
            b"normal-notes" => {
                if let Ok(v) = text.parse() {
                    self.tuplet_normal = v;
                }
            }
            b"time-modification" => {
                if let Some(n) = self.note.as_mut() {
                    n.tuplet = Some(Tuplet {
                        actual: self.tuplet_actual,
                        normal: self.tuplet_normal,
                    });
                }
            }
            b"syllabic" => {
                if self.in_lyric && !text.is_empty() {
                    self.lyric_syllabic = Some(text.to_string());
                }
            }
            b"text" => {
                if self.in_lyric {
                    self.lyric_text.push_str(text);
                }
            }
            b"lyric" => {
                self.in_lyric = false;
                if let Some(n) = self.note.as_mut() {
                    n.lyric = Some(Lyric {
                        syllabic: self.lyric_syllabic.take(),
                        text: std::mem::take(&mut self.lyric_text),
                    });
                }
            }
            b"words" => {
                if let Some(d) = self.direction.as_mut() {
                    d.kind = DirectionKind::Words(text.to_string());
                    self.direction_has_kind = true;
                }
            }
            b"beat-unit" => {
                if self.in_metro {
                    self.metro_unit = text.to_string();
                }
            }
            b"per-minute" => {
                if let Ok(v) = text.parse() {
                    self.metro_pm = v;
                }
            }
            b"metronome" => {
                self.in_metro = false;
                if let Some(d) = self.direction.as_mut() {
                    d.kind = DirectionKind::Metronome {
                        beat_unit: std::mem::take(&mut self.metro_unit),
                        per_minute: self.metro_pm,
                    };
                    self.direction_has_kind = true;
                }
            }
            b"dynamics" => self.in_dynamics = false,
            b"backup" => self.in_backup = false,
            b"forward" => self.in_forward = false,
            b"note" => self.finish_note(),
            b"direction" => {
                // `take()` always runs (clearing the builder); push only the
                // directions we actually recognized a kind for.
                if let Some(d) = self.direction.take()
                    && self.direction_has_kind
                {
                    self.directions.push(d);
                }
                self.direction_has_kind = false;
            }
            b"measure-repeat" => {
                if self.in_measure_repeat_start {
                    self.in_measure_repeat_start = false;
                    if let (Some((_, slashes)), Ok(len)) =
                        (self.measure_repeat, text.parse::<u32>())
                    {
                        // Each active measure replays the one `len` back.
                        self.measure_repeat = Some((len.clamp(1, 8), slashes));
                    }
                }
            }
            b"measure" => self.finish_measure(),
            _ => {}
        }
    }

    /// Finalizes the current note: stamps its time position and advances the
    /// measure cursor (chord members share the previous onset and add no time).
    fn finish_note(&mut self) {
        let Some(mut n) = self.note.take() else {
            return;
        };
        // An empty `<unpitched/>` element produces no End event, so its builder
        // is still pending here: attach it with its default (middle-line)
        // position.
        if n.unpitched.is_none() {
            n.unpitched = self.unpitched.take();
        }
        self.unpitched = None;
        // A degenerate note carrying none of pitch/unpitched/rest is skipped —
        // it has nothing any consumer could draw or sound — but its duration
        // still advances the cursor so the surrounding notes keep their
        // positions.
        let degenerate = n.pitch.is_none() && n.unpitched.is_none() && !n.is_rest;
        if n.is_chord {
            n.position_divisions = self.last_onset;
            if !degenerate {
                self.notes.push(n);
            }
        } else {
            n.position_divisions = self.cursor;
            self.last_onset = self.cursor;
            self.cursor = self.cursor.saturating_add(n.duration_divisions);
            if !degenerate {
                self.notes.push(n);
            }
        }
    }

    /// Finalizes the current measure (first part only), computing its geometry
    /// and resetting the per-measure cursor.
    fn finish_measure(&mut self) {
        let notes = std::mem::take(&mut self.notes);
        let directions = std::mem::take(&mut self.directions);
        let clefs = std::mem::take(&mut self.measure_clefs);
        let mut repeats = std::mem::take(&mut self.repeat_marks);
        if let Some((len, slashes)) = self.measure_repeat
            && self.current_part <= 1
        {
            // A measure inside an active measure-repeat run replays the
            // measure `len` back; the reference resolves transitively (the
            // referenced measure is already resolved by induction), so every
            // `%` points at real content.
            repeats.measure_repeat_of = self.measure_index.checked_sub(len).map(|i| {
                self.measures
                    .get(i as usize)
                    .and_then(|m| m.repeats.measure_repeat_of)
                    .unwrap_or(i)
            });
            repeats.measure_repeat_slashes = slashes;
        }
        // Only the first part contributes to the (single-part) document.
        if self.current_part <= 1 {
            let width = min_width(&notes, self.divisions);
            self.measures.push(NotationMeasure {
                index: self.measure_index,
                notes,
                directions,
                clefs,
                key_fifths: self.key_fifths,
                min_width: width,
                repeats,
            });
            self.measure_index += 1;
        }
        self.cursor = 0;
        self.last_onset = 0;
    }

    fn into_document(mut self) -> ScoreDocument {
        // A score with no alteration data anywhere (no `<alter>`, no `<accidental>`)
        // drew an armure but never marked its notes: infer the key signature onto
        // every pitch, honoring mid-piece key changes via each measure's own key.
        // Conforming exporters always emit some alteration (MuseScore writes an
        // explicit `<alter>` on every sounding-altered note), so this never runs
        // for them and their pitches — including notes that are natural only
        // because a same-pitch note in another voice carries the accidental — are
        // left exactly as authored.
        if !self.saw_alteration {
            for measure in &mut self.measures {
                let fifths = measure.key_fifths;
                if fifths == 0 {
                    continue;
                }
                for note in &mut measure.notes {
                    if let Some(pitch) = note.pitch.as_mut() {
                        pitch.alter = pitch_alter::key_signature_alter(fifths, pitch.step);
                    }
                }
            }
        }
        // Resolve each unpitched note's General MIDI number against the
        // part-list instrument table — the ONE resolution site all four
        // consumers share. A note with no `<instrument>` reference in a part
        // declaring exactly one sounding instrument resolves to that
        // instrument (the MusicXML default-instrument convention, routine in
        // single-line percussion exports); anything else unresolvable stays
        // `None`, never inferred from the written position.
        let sole_gm = {
            let mut gms = self.instruments.iter().filter_map(|d| d.gm_number);
            match (gms.next(), gms.next()) {
                (Some(gm), None) => Some(gm),
                _ => None,
            }
        };
        for measure in &mut self.measures {
            for note in &mut measure.notes {
                let Some(u) = note.unpitched.as_mut() else {
                    continue;
                };
                u.gm_number = match note.instrument_id.as_deref() {
                    Some(id) => self
                        .instruments
                        .iter()
                        .find(|d| d.id == id)
                        .and_then(|d| d.gm_number),
                    None => sole_gm,
                };
                // The engraved head class rides beside the resolved number so
                // the painters never own GM ranges (add-drum-notation-render).
                u.head_class = HeadClass::of(u.gm_number);
            }
        }
        let play_order = repeats::play_order(&self.measures);
        ScoreDocument {
            meta: ScoreMeta {
                title: self.title,
                composer: self.composer,
            },
            staves: self.staves.unwrap_or(1).max(1),
            instruments: self.instruments,
            attributes: Attributes {
                divisions: self.divisions,
                clefs: self.clefs,
                key_fifths: self.key_fifths,
                time: TimeSignature {
                    beats: self.beats,
                    beat_type: self.beat_type,
                },
            },
            measures: self.measures,
            play_order,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Fixtures --------------------------------------------------------

    /// Minimal single-staff score: title, composer, one 4/4 measure of two
    /// quarter notes (C5 then D5), divisions = 4.
    const MINIMAL: &str = r#"<?xml version="1.0"?>
<score-partwise version="3.1">
  <work><work-title>Little Tune</work-title></work>
  <identification><creator type="composer">A. Composer</creator></identification>
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>4</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>5</octave></pitch>
        <duration>4</duration><voice>1</voice><type>quarter</type><staff>1</staff>
      </note>
      <note>
        <pitch><step>D</step><octave>5</octave></pitch>
        <duration>4</duration><voice>1</voice><type>quarter</type><staff>1</staff>
      </note>
    </measure>
  </part>
</score-partwise>"#;

    /// Two-staff grand staff: treble (G/2) on staff 1, bass (F/4) on staff 2,
    /// key fifths -3, 3/4. Treble has three quarter notes; a full-measure
    /// `backup` (12 divisions) rewinds, then the bass writes three quarters.
    /// Exercises ties, a triplet, accidental/alter, dots, stem, beams, lyric,
    /// chord, rest, and word/dynamics/wedge directions.
    const GRAND: &str = r#"<?xml version="1.0"?>
<score-partwise version="3.1">
  <part-list><score-part id="P1"/></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>4</divisions>
        <key><fifths>-3</fifths><mode>minor</mode></key>
        <time><beats>3</beats><beat-type>4</beat-type></time>
        <staves>2</staves>
        <clef number="1"><sign>G</sign><line>2</line></clef>
        <clef number="2"><sign>F</sign><line>4</line></clef>
      </attributes>
      <direction placement="above">
        <direction-type><words>Andantino</words></direction-type>
        <staff>1</staff>
      </direction>
      <direction>
        <direction-type><dynamics><pp/></dynamics></direction-type>
        <staff>1</staff>
      </direction>
      <direction>
        <direction-type><wedge type="crescendo"/></direction-type>
        <staff>1</staff>
      </direction>
      <note>
        <pitch><step>E</step><alter>-1</alter><octave>5</octave></pitch>
        <duration>4</duration><tie type="start"/><voice>1</voice><type>quarter</type>
        <accidental>flat</accidental><stem>up</stem><staff>1</staff>
        <lyric><syllabic>begin</syllabic><text>Dans</text></lyric>
      </note>
      <note>
        <chord/>
        <pitch><step>G</step><octave>5</octave></pitch>
        <duration>4</duration><voice>1</voice><type>quarter</type><staff>1</staff>
      </note>
      <note>
        <pitch><step>E</step><alter>-1</alter><octave>5</octave></pitch>
        <duration>4</duration><tie type="stop"/><voice>1</voice><type>quarter</type>
        <staff>1</staff>
      </note>
      <note>
        <pitch><step>C</step><octave>5</octave></pitch>
        <duration>4</duration><voice>1</voice><type>quarter</type><staff>1</staff>
      </note>
      <backup><duration>12</duration></backup>
      <note>
        <pitch><step>C</step><octave>3</octave></pitch>
        <duration>2</duration><voice>5</voice><type>eighth</type><stem>down</stem>
        <staff>2</staff><beam number="1">begin</beam>
        <time-modification><actual-notes>3</actual-notes><normal-notes>2</normal-notes></time-modification>
      </note>
      <note>
        <pitch><step>E</step><octave>3</octave></pitch>
        <duration>2</duration><voice>5</voice><type>eighth</type><staff>2</staff>
        <beam number="1">continue</beam>
      </note>
      <note>
        <pitch><step>G</step><octave>3</octave></pitch>
        <duration>2</duration><voice>5</voice><type>eighth</type><staff>2</staff>
        <beam number="1">end</beam>
      </note>
      <note>
        <rest/>
        <duration>6</duration><voice>5</voice><type>quarter</type><dot/><staff>2</staff>
      </note>
    </measure>
  </part>
</score-partwise>"#;

    /// A short two-measure single-staff score for layout tests: measure 0 has
    /// eight eighth notes (dense), measure 1 has two half notes (sparse).
    const TWO_MEASURES: &str = r#"<?xml version="1.0"?>
<score-partwise>
  <part-list><score-part id="P1"/></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>4</divisions></attributes>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>2</duration></note>
      <note><pitch><step>D</step><octave>4</octave></pitch><duration>2</duration></note>
      <note><pitch><step>E</step><octave>4</octave></pitch><duration>2</duration></note>
      <note><pitch><step>F</step><octave>4</octave></pitch><duration>2</duration></note>
      <note><pitch><step>G</step><octave>4</octave></pitch><duration>2</duration></note>
      <note><pitch><step>A</step><octave>4</octave></pitch><duration>2</duration></note>
      <note><pitch><step>B</step><octave>4</octave></pitch><duration>2</duration></note>
      <note><pitch><step>C</step><octave>5</octave></pitch><duration>2</duration></note>
    </measure>
    <measure number="2">
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>8</duration></note>
      <note><pitch><step>G</step><octave>4</octave></pitch><duration>8</duration></note>
    </measure>
  </part>
</score-partwise>"#;

    const NON_MUSIC: &str = r#"<html><body><p>not a score</p></body></html>"#;
    const MALFORMED: &str = r#"<score-partwise><measure></score-partwise>"#;

    fn parse_ok(s: &str) -> ScoreDocument {
        parse(s.as_bytes()).expect("parse")
    }

    // --- Metadata & structure -------------------------------------------

    #[test]
    fn extracts_title_and_composer() {
        let doc = parse_ok(MINIMAL);
        assert_eq!(doc.meta.title.as_deref(), Some("Little Tune"));
        assert_eq!(doc.meta.composer.as_deref(), Some("A. Composer"));
    }

    #[test]
    fn missing_metadata_is_none_not_error() {
        let doc = parse_ok(TWO_MEASURES);
        assert_eq!(doc.meta.title, None);
        assert_eq!(doc.meta.composer, None);
    }

    #[test]
    fn single_staff_defaults_to_one() {
        assert_eq!(parse_ok(MINIMAL).staves, 1);
    }

    #[test]
    fn grand_staff_reports_two_staves() {
        assert_eq!(parse_ok(GRAND).staves, 2);
    }

    #[test]
    fn per_staff_clefs_on_grand_staff() {
        let doc = parse_ok(GRAND);
        let treble = doc.attributes.clefs.iter().find(|c| c.staff == 1).unwrap();
        let bass = doc.attributes.clefs.iter().find(|c| c.staff == 2).unwrap();
        assert_eq!((treble.sign, treble.line), (ClefSign::G, 2));
        assert_eq!((bass.sign, bass.line), (ClefSign::F, 4));
    }

    #[test]
    fn clef_change_keeps_initial_and_records_per_measure() {
        // Staff 2 starts in treble (G) then switches to bass (F) in measure 2,
        // as in Debussy's Arabesque left hand.
        let xml = r#"<score-partwise><part-list><score-part id="P1"/></part-list>
        <part id="P1">
          <measure number="1">
            <attributes><divisions>4</divisions><staves>2</staves>
              <clef number="1"><sign>G</sign><line>2</line></clef>
              <clef number="2"><sign>G</sign><line>2</line></clef>
            </attributes>
            <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><staff>2</staff></note>
          </measure>
          <measure number="2">
            <attributes>
              <clef number="2"><sign>F</sign><line>4</line></clef>
            </attributes>
            <note><pitch><step>C</step><octave>3</octave></pitch><duration>4</duration><staff>2</staff></note>
          </measure>
        </part></score-partwise>"#;
        let doc = parse_ok(xml);
        // Initial clef for staff 2 is treble.
        let init2 = doc.attributes.clefs.iter().find(|c| c.staff == 2).unwrap();
        assert_eq!((init2.sign, init2.line), (ClefSign::G, 2));
        // Measure 1 declares both clefs; measure 2 changes staff 2 to bass.
        assert!(
            doc.measures[0]
                .clefs
                .iter()
                .any(|c| c.staff == 2 && c.sign == ClefSign::G)
        );
        let m2 = doc.measures[1].clefs.iter().find(|c| c.staff == 2).unwrap();
        assert_eq!((m2.sign, m2.line), (ClefSign::F, 4));
        assert!(doc.measures[1].clefs.iter().all(|c| c.staff != 1));
    }

    #[test]
    fn key_and_time_signature() {
        let doc = parse_ok(GRAND);
        assert_eq!(doc.attributes.key_fifths, -3);
        assert_eq!(doc.attributes.time.beats, 3);
        assert_eq!(doc.attributes.time.beat_type, 4);
        assert_eq!(doc.attributes.divisions, 4);
    }

    #[test]
    fn notes_routed_to_their_staff() {
        let doc = parse_ok(GRAND);
        let m = &doc.measures[0];
        assert!(m.notes.iter().any(|n| n.staff == 1));
        assert!(m.notes.iter().any(|n| n.staff == 2));
    }

    #[test]
    fn note_defaults_to_staff_one_when_absent() {
        let doc = parse_ok(MINIMAL);
        assert!(doc.measures[0].notes.iter().all(|n| n.staff == 1));
    }

    // --- NotationMeasure time navigation ----------------------------------------

    #[test]
    fn backup_rewinds_so_bass_aligns_with_treble() {
        let doc = parse_ok(GRAND);
        let m = &doc.measures[0];
        // Treble's first non-chord note is at position 0; after a full backup
        // the bass's first note is also at position 0.
        let treble0 = m.notes.iter().find(|n| n.staff == 1).unwrap();
        let bass0 = m.notes.iter().find(|n| n.staff == 2).unwrap();
        assert_eq!(treble0.position_divisions, 0);
        assert_eq!(bass0.position_divisions, 0);
    }

    #[test]
    fn chord_member_shares_onset_and_does_not_advance() {
        let doc = parse_ok(GRAND);
        let m = &doc.measures[0];
        let chord = m.notes.iter().find(|n| n.is_chord).unwrap();
        assert_eq!(chord.position_divisions, 0);
        // The note after the chord (still staff 1) advanced only once → pos 4.
        let third = m
            .notes
            .iter()
            .filter(|n| n.staff == 1 && !n.is_chord)
            .nth(1)
            .unwrap();
        assert_eq!(third.position_divisions, 4);
    }

    #[test]
    fn forward_advances_position() {
        let xml = r#"<score-partwise><part-list><score-part id="P1"/></part-list>
        <part id="P1"><measure number="1">
          <attributes><divisions>4</divisions></attributes>
          <forward><duration>4</duration></forward>
          <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration></note>
        </measure></part></score-partwise>"#;
        let doc = parse_ok(xml);
        assert_eq!(doc.measures[0].notes[0].position_divisions, 4);
    }

    // --- Note detail -----------------------------------------------------

    #[test]
    fn pitched_note_extracted() {
        let doc = parse_ok(MINIMAL);
        let n = &doc.measures[0].notes[0];
        let p = n.pitch.as_ref().unwrap();
        assert_eq!((p.step, p.octave), ('C', 5));
        assert_eq!(n.duration_divisions, 4);
        assert_eq!(n.voice, 1);
        assert!(!n.is_chord);
        assert_eq!(n.note_type.as_deref(), Some("quarter"));
    }

    #[test]
    fn altered_pitch_and_accidental() {
        let doc = parse_ok(GRAND);
        let n = doc.measures[0].notes.iter().find(|n| n.tie_start).unwrap();
        assert_eq!(n.pitch.as_ref().unwrap().alter, -1);
        assert_eq!(n.accidental.as_deref(), Some("flat"));
    }

    // --- Key-signature inference for unmarked scores ---------------------

    /// Alteration of the `idx`-th note in `measure`.
    fn alter_at(doc: &ScoreDocument, measure: usize, idx: usize) -> i32 {
        doc.measures[measure].notes[idx]
            .pitch
            .as_ref()
            .expect("pitched note")
            .alter
    }

    #[test]
    fn key_signature_inferred_when_no_alteration_data() {
        // No <alter>/<accidental> anywhere + three flats → B/E/A flattened, C not.
        let doc = parse_ok(
            r#"<score-partwise><part-list><score-part id="P1"/></part-list>
            <part id="P1"><measure number="1">
              <attributes><divisions>1</divisions><key><fifths>-3</fifths></key></attributes>
              <note><pitch><step>B</step><octave>4</octave></pitch><duration>1</duration></note>
              <note><pitch><step>E</step><octave>4</octave></pitch><duration>1</duration></note>
              <note><pitch><step>A</step><octave>4</octave></pitch><duration>1</duration></note>
              <note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration></note>
            </measure></part></score-partwise>"#,
        );
        assert_eq!(alter_at(&doc, 0, 0), -1, "B♭");
        assert_eq!(alter_at(&doc, 0, 1), -1, "E♭");
        assert_eq!(alter_at(&doc, 0, 2), -1, "A♭");
        assert_eq!(alter_at(&doc, 0, 3), 0, "C is unaffected");
    }

    #[test]
    fn a_single_alteration_disables_inference_document_wide() {
        // One explicit <alter> anywhere ⇒ inference off; other bare notes stay
        // natural. This matches conforming exporters: MuseScore writes <alter> on
        // every sounding-altered note, so a bare note is deliberately natural
        // (e.g. a repeat left natural because another voice carries the accidental)
        // and must never be re-derived from the key signature.
        let doc = parse_ok(
            r#"<score-partwise><part-list><score-part id="P1"/></part-list>
            <part id="P1"><measure number="1">
              <attributes><divisions>1</divisions><key><fifths>-3</fifths></key></attributes>
              <note><pitch><step>G</step><alter>1</alter><octave>4</octave></pitch><duration>1</duration></note>
              <note><pitch><step>B</step><octave>4</octave></pitch><duration>1</duration></note>
            </measure></part></score-partwise>"#,
        );
        assert_eq!(alter_at(&doc, 0, 0), 1, "explicit G♯ kept");
        assert_eq!(alter_at(&doc, 0, 1), 0, "B stays natural — no inference");
    }

    #[test]
    fn an_accidental_also_disables_inference() {
        // An <accidental> (even without <alter>) counts as alteration data.
        let doc = parse_ok(
            r#"<score-partwise><part-list><score-part id="P1"/></part-list>
            <part id="P1"><measure number="1">
              <attributes><divisions>1</divisions><key><fifths>-3</fifths></key></attributes>
              <note><pitch><step>B</step><octave>4</octave></pitch><accidental>natural</accidental><duration>1</duration></note>
              <note><pitch><step>E</step><octave>4</octave></pitch><duration>1</duration></note>
            </measure></part></score-partwise>"#,
        );
        assert_eq!(alter_at(&doc, 0, 0), 0, "explicit natural B");
        assert_eq!(alter_at(&doc, 0, 1), 0, "E stays natural — no inference");
    }

    #[test]
    fn inference_honors_mid_piece_key_change() {
        // Unmarked score: B is B♭ under -3 fifths, B natural once the key changes to 0.
        let doc = parse_ok(
            r#"<score-partwise><part-list><score-part id="P1"/></part-list>
            <part id="P1">
              <measure number="1">
                <attributes><divisions>1</divisions><key><fifths>-3</fifths></key></attributes>
                <note><pitch><step>B</step><octave>4</octave></pitch><duration>1</duration></note>
              </measure>
              <measure number="2">
                <attributes><key><fifths>0</fifths></key></attributes>
                <note><pitch><step>B</step><octave>4</octave></pitch><duration>1</duration></note>
              </measure>
            </part></score-partwise>"#,
        );
        assert_eq!(alter_at(&doc, 0, 0), -1, "B♭ in the flat measure");
        assert_eq!(alter_at(&doc, 1, 0), 0, "B natural after the key change");
    }

    #[test]
    fn each_measure_records_the_key_in_force() {
        // A modulating piece (like Haydn's canzonet): starts in 4 flats, then
        // changes to 1 flat. Every measure must carry the key in force during it,
        // carried forward across measures with no `<key>` — so the renderer can
        // draw the correct armure per system instead of one document-wide value.
        let doc = parse_ok(
            r#"<score-partwise><part-list><score-part id="P1"/></part-list>
            <part id="P1">
              <measure number="1">
                <attributes><divisions>1</divisions><key><fifths>-4</fifths></key></attributes>
                <note><pitch><step>C</step><octave>5</octave></pitch><duration>1</duration></note>
              </measure>
              <measure number="2">
                <note><pitch><step>C</step><octave>5</octave></pitch><duration>1</duration></note>
              </measure>
              <measure number="3">
                <attributes><key><fifths>-1</fifths></key></attributes>
                <note><pitch><step>C</step><octave>5</octave></pitch><duration>1</duration></note>
              </measure>
            </part></score-partwise>"#,
        );
        assert_eq!(doc.measures[0].key_fifths, -4, "opening key");
        assert_eq!(
            doc.measures[1].key_fifths, -4,
            "carried across an un-marked measure"
        );
        assert_eq!(doc.measures[2].key_fifths, -1, "modulation takes effect");
    }

    #[test]
    fn dotted_rest_extracted() {
        let doc = parse_ok(GRAND);
        let rest = doc.measures[0].notes.iter().find(|n| n.is_rest).unwrap();
        assert!(rest.is_rest);
        assert_eq!(rest.dots, 1);
        assert_eq!(rest.duration_divisions, 6);
    }

    #[test]
    fn tie_start_and_stop() {
        let doc = parse_ok(GRAND);
        let m = &doc.measures[0];
        assert!(m.notes.iter().any(|n| n.tie_start && !n.tie_stop));
        assert!(m.notes.iter().any(|n| n.tie_stop && !n.tie_start));
    }

    #[test]
    fn slur_start_and_stop() {
        let xml = r#"<score-partwise><part-list><score-part id="P1"/></part-list>
        <part id="P1"><measure number="1">
          <note><pitch><step>C</step><octave>4</octave></pitch><duration>2</duration>
            <notations><slur type="start" number="1"/></notations></note>
          <note><pitch><step>D</step><octave>4</octave></pitch><duration>2</duration></note>
          <note><pitch><step>E</step><octave>4</octave></pitch><duration>2</duration>
            <notations><slur type="stop" number="1"/></notations></note>
        </measure></part></score-partwise>"#;
        let doc = parse_ok(xml);
        let m = &doc.measures[0];
        assert!(m.notes[0].slur_start && !m.notes[0].slur_stop);
        assert!(!m.notes[1].slur_start && !m.notes[1].slur_stop);
        assert!(m.notes[2].slur_stop && !m.notes[2].slur_start);
    }

    #[test]
    fn triplet_ratio_captured() {
        let doc = parse_ok(GRAND);
        let t = doc.measures[0]
            .notes
            .iter()
            .find_map(|n| n.tuplet.clone())
            .unwrap();
        assert_eq!((t.actual, t.normal), (3, 2));
    }

    #[test]
    fn stem_directions_captured() {
        let doc = parse_ok(GRAND);
        let m = &doc.measures[0];
        assert!(m.notes.iter().any(|n| n.stem == Some(StemDir::Up)));
        assert!(m.notes.iter().any(|n| n.stem == Some(StemDir::Down)));
    }

    #[test]
    fn beam_group_captured() {
        let doc = parse_ok(GRAND);
        let beams: Vec<BeamState> = doc.measures[0]
            .notes
            .iter()
            .filter_map(|n| n.beams.first().cloned())
            .collect();
        assert_eq!(
            beams,
            vec![BeamState::Begin, BeamState::Continue, BeamState::End]
        );
    }

    #[test]
    fn lyric_syllable_attached() {
        let doc = parse_ok(GRAND);
        let lyric = doc.measures[0]
            .notes
            .iter()
            .find_map(|n| n.lyric.clone())
            .unwrap();
        assert_eq!(lyric.syllabic.as_deref(), Some("begin"));
        assert_eq!(lyric.text, "Dans");
    }

    // --- Directions ------------------------------------------------------

    #[test]
    fn words_dynamics_wedge_directions() {
        let doc = parse_ok(GRAND);
        let dirs = &doc.measures[0].directions;
        assert!(
            dirs.iter()
                .any(|d| matches!(&d.kind, DirectionKind::Words(w) if w == "Andantino"))
        );
        assert!(
            dirs.iter()
                .any(|d| matches!(&d.kind, DirectionKind::Dynamics(d) if d == "pp"))
        );
        assert!(dirs.iter().any(|d| matches!(
            d.kind,
            DirectionKind::Wedge {
                crescendo: true,
                stop: false
            }
        )));
    }

    #[test]
    fn metronome_direction() {
        let xml = r#"<score-partwise><part-list><score-part id="P1"/></part-list>
        <part id="P1"><measure number="1">
          <direction><direction-type><metronome>
            <beat-unit>quarter</beat-unit><per-minute>120</per-minute>
          </metronome></direction-type></direction>
          <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration></note>
        </measure></part></score-partwise>"#;
        let doc = parse_ok(xml);
        assert!(doc.measures[0].directions.iter().any(|d| matches!(
            &d.kind,
            DirectionKind::Metronome { beat_unit, per_minute }
                if beat_unit == "quarter" && *per_minute == 120
        )));
    }

    #[test]
    fn unknown_direction_ignored() {
        let xml = r#"<score-partwise><part-list><score-part id="P1"/></part-list>
        <part id="P1"><measure number="1">
          <direction><direction-type><bracket type="start"/></direction-type></direction>
          <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration></note>
        </measure></part></score-partwise>"#;
        let doc = parse_ok(xml);
        assert!(doc.measures[0].directions.is_empty());
    }

    // --- Robustness ------------------------------------------------------

    #[test]
    fn malformed_xml_is_recoverable_error() {
        assert!(parse(MALFORMED.as_bytes()).is_err());
    }

    #[test]
    fn non_musicxml_yields_empty_document() {
        let doc = parse_ok(NON_MUSIC);
        assert!(doc.measures.is_empty());
        assert_eq!(doc.staves, 1);
    }

    #[test]
    fn bytes_and_string_inputs_are_equivalent() {
        let s = MINIMAL.to_string();
        let from_bytes = parse(MINIMAL.as_bytes()).unwrap();
        let from_string = parse(s.as_bytes()).unwrap();
        assert_eq!(from_bytes, from_string);
    }

    #[test]
    fn second_part_is_ignored() {
        let xml = r#"<score-partwise>
          <part-list><score-part id="P1"/><score-part id="P2"/></part-list>
          <part id="P1"><measure number="1">
            <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration></note>
          </measure></part>
          <part id="P2"><measure number="1">
            <note><pitch><step>G</step><octave>4</octave></pitch><duration>4</duration></note>
          </measure></part>
        </score-partwise>"#;
        let doc = parse_ok(xml);
        assert_eq!(doc.measures.len(), 1);
    }

    // --- Geometry --------------------------------------------------------

    #[test]
    fn denser_measure_is_wider() {
        let doc = parse_ok(TWO_MEASURES);
        // NotationMeasure 0: eight eighth notes; measure 1: two half notes (equal total).
        assert!(doc.measures[0].min_width > doc.measures[1].min_width);
    }

    #[test]
    fn spacing_is_sub_linear_in_duration() {
        // Quarter (4) vs half (8) at divisions = 4.
        let q = space(4, 4);
        let h = space(8, 4);
        assert!(h > q, "longer note gets more space");
        assert!(h < 2.0 * q, "but less than twice (sub-linear)");
    }

    #[test]
    fn column_floor_binds_on_short_durations_only() {
        // 16th (1 div) and 32nd-ish (below) clamp to the minimum column
        // advance; a quarter (4 divs) keeps its non-linear value above it.
        assert_eq!(space(1, 4), MIN_COL);
        assert_eq!(space(2, 4), MIN_COL); // eighth: 30·0.5^0.6 ≈ 19.8 → floored
        assert!(space(4, 4) > MIN_COL);
        assert_eq!(space(4, 4), UNIT);
    }

    #[test]
    fn sixteenth_runs_grow_linearly_with_the_floor() {
        let note = |pos: u32| NoteEvent {
            staff: 1,
            voice: 1,
            position_divisions: pos,
            pitch: None,
            is_rest: false,
            is_chord: false,
            is_grace: false,
            duration_divisions: 1,
            note_type: None,
            dots: 0,
            accidental: None,
            tie_start: false,
            tie_stop: false,
            slur_start: false,
            slur_stop: false,
            tuplet: None,
            stem: None,
            beams: Vec::new(),
            lyric: None,
            unpitched: None,
            instrument_id: None,
        };
        let run: Vec<NoteEvent> = (0..8).map(note).collect();
        // Eight 16th columns each get the full floor, not a compressed share.
        assert!((min_width(&run, 4) - (LEFT_PAD + 8.0 * MIN_COL)).abs() < 1e-9);
    }

    #[test]
    fn shared_columns_are_not_double_counted() {
        // Two notes at the same position (e.g. a chord) form one column.
        let note = |pos: u32| NoteEvent {
            staff: 1,
            voice: 1,
            position_divisions: pos,
            pitch: None,
            is_rest: false,
            is_chord: false,
            is_grace: false,
            duration_divisions: 4,
            note_type: None,
            dots: 0,
            accidental: None,
            tie_start: false,
            tie_stop: false,
            slur_start: false,
            slur_stop: false,
            tuplet: None,
            stem: None,
            beams: Vec::new(),
            lyric: None,
            unpitched: None,
            instrument_id: None,
        };
        let one = vec![note(0)];
        let two_same = vec![note(0), note(0)];
        assert_eq!(min_width(&one, 4), min_width(&two_same, 4));
        let two_distinct = vec![note(0), note(4)];
        assert!(min_width(&two_distinct, 4) > min_width(&one, 4));
    }

    #[test]
    fn floor_is_respected_for_sparse_measure() {
        let rest = vec![NoteEvent {
            staff: 1,
            voice: 1,
            position_divisions: 0,
            pitch: None,
            is_rest: true,
            is_chord: false,
            is_grace: false,
            duration_divisions: 1,
            note_type: None,
            dots: 0,
            accidental: None,
            tie_start: false,
            tie_stop: false,
            slur_start: false,
            slur_stop: false,
            tuplet: None,
            stem: None,
            beams: Vec::new(),
            lyric: None,
            unpitched: None,
            instrument_id: None,
        }];
        assert!(min_width(&rest, 4) >= FLOOR);
    }

    // --- System layout ---------------------------------------------------

    fn doc_with_widths(widths: &[f64]) -> ScoreDocument {
        ScoreDocument {
            meta: ScoreMeta {
                title: None,
                composer: None,
            },
            staves: 2,
            instruments: Vec::new(),
            attributes: Attributes {
                divisions: 4,
                clefs: Vec::new(),
                key_fifths: 0,
                time: TimeSignature {
                    beats: 4,
                    beat_type: 4,
                },
            },
            measures: widths
                .iter()
                .enumerate()
                .map(|(i, &w)| NotationMeasure {
                    index: i as u32,
                    notes: Vec::new(),
                    directions: Vec::new(),
                    clefs: Vec::new(),
                    key_fifths: 0,
                    min_width: w,
                    repeats: RepeatMarks::default(),
                })
                .collect(),
            play_order: (0..widths.len())
                .map(|i| PlayedMeasure {
                    written_index: i as u32,
                    pass: 1,
                })
                .collect(),
        }
    }

    #[test]
    fn all_measures_fit_on_one_system() {
        let doc = doc_with_widths(&[100.0, 100.0, 100.0]);
        let systems = layout_systems(&doc, 1000.0);
        assert_eq!(systems.len(), 1);
        assert_eq!(systems[0].measures, vec![0, 1, 2]);
        assert_eq!(systems[0].staves, 2);
    }

    #[test]
    fn measures_wrap_into_multiple_systems_in_order() {
        let doc = doc_with_widths(&[100.0, 100.0, 100.0, 100.0]);
        let systems = layout_systems(&doc, 250.0);
        assert_eq!(systems.len(), 2);
        assert_eq!(systems[0].measures, vec![0, 1]);
        assert_eq!(systems[1].measures, vec![2, 3]);
    }

    #[test]
    fn oversized_measure_gets_its_own_system() {
        let doc = doc_with_widths(&[100.0, 500.0, 100.0]);
        let systems = layout_systems(&doc, 250.0);
        // m0 alone, m1 oversized alone, m2 alone.
        assert_eq!(systems.len(), 3);
        assert_eq!(systems[1].measures, vec![1]);
    }

    #[test]
    fn caps_measures_per_system_even_on_a_wide_line() {
        // Six narrow measures that would all fit width-wise still wrap at the cap.
        let doc = doc_with_widths(&[10.0, 10.0, 10.0, 10.0, 10.0, 10.0]);
        let systems = layout_systems(&doc, 100_000.0);
        assert_eq!(systems.len(), 2);
        assert_eq!(systems[0].measures.len(), MAX_MEASURES_PER_SYSTEM);
        assert_eq!(systems[1].measures, vec![3, 4, 5]);
    }

    #[test]
    fn empty_document_lays_out_to_no_systems() {
        let doc = doc_with_widths(&[]);
        assert!(layout_systems(&doc, 500.0).is_empty());
    }

    // --- Percussion notation (change: add-unpitched-notation) -------------

    #[test]
    fn head_classes_split_cymbals_from_drums_and_mark_the_open_hat() {
        // The cymbal set takes x heads; 46 additionally the open mark.
        for gm in [42, 44, 49, 51, 52, 53, 55, 57, 59] {
            assert_eq!(HeadClass::of(Some(gm)), HeadClass::X, "GM {gm}");
        }
        assert_eq!(HeadClass::of(Some(46)), HeadClass::XOpen);
        // The drums take the ordinary oval — kick, snares, toms, percussion
        // accessories alike — and so does an unresolved note: the written
        // position is authoritative even when the sound is not.
        for gm in [35, 36, 37, 38, 40, 41, 43, 45, 47, 48, 50, 54, 56] {
            assert_eq!(HeadClass::of(Some(gm)), HeadClass::Oval, "GM {gm}");
        }
        assert_eq!(HeadClass::of(None), HeadClass::Oval);
    }

    #[test]
    fn parse_derives_the_head_class_beside_the_resolved_number() {
        let doc = parse(crate::fixtures::ROCK_GROOVE.as_bytes()).unwrap();
        let class_of = |gm: u32| {
            doc.measures
                .iter()
                .flat_map(|m| &m.notes)
                .find_map(|n| {
                    n.unpitched
                        .as_ref()
                        .filter(|u| u.gm_number == Some(gm))
                        .map(|u| u.head_class)
                })
                .unwrap()
        };
        assert_eq!(class_of(42), HeadClass::X); // closed hi-hat
        assert_eq!(class_of(38), HeadClass::Oval); // snare
        assert_eq!(class_of(36), HeadClass::Oval); // kick

        // An unresolvable note keeps the oval (engraved, never dropped).
        let degraded = parse(crate::fixtures::DEGRADED.as_bytes()).unwrap();
        let unresolved = degraded
            .measures
            .iter()
            .flat_map(|m| &m.notes)
            .filter_map(|n| n.unpitched.as_ref())
            .find(|u| u.gm_number.is_none())
            .expect("the degraded fixture carries an unresolved note");
        assert_eq!(unresolved.head_class, HeadClass::Oval);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn head_class_rides_the_wasm_json_contract() {
        // The console painter consumes the field from the serde JSON — the
        // variant names are the wire contract.
        let u = Unpitched {
            display_step: 'G',
            display_octave: 5,
            gm_number: Some(46),
            head_class: HeadClass::of(Some(46)),
        };
        let json = serde_json::to_string(&u).unwrap();
        assert!(json.contains("\"head_class\":\"XOpen\""), "got {json}");
        let back: Unpitched = serde_json::from_str(&json).unwrap();
        assert_eq!(back, u);
    }

    #[test]
    fn percussion_fixture_parses_positions_and_resolves_gm_numbers() {
        let doc = parse(crate::fixtures::ROCK_GROOVE.as_bytes()).unwrap();

        // The part-list instrument table, with 1-based element values
        // converted: 37 → GM 36 (kick), 39 → GM 38 (snare), 43 → GM 42 (hat).
        let gm_of = |id: &str| {
            doc.instruments
                .iter()
                .find(|d| d.id == id)
                .and_then(|d| d.gm_number)
        };
        assert_eq!(gm_of("P1-I36"), Some(36));
        assert_eq!(gm_of("P1-I38"), Some(38));
        assert_eq!(gm_of("P1-I42"), Some(42));
        let snare = doc.instruments.iter().find(|d| d.id == "P1-I38").unwrap();
        assert_eq!(snare.name.as_deref(), Some("Snare Drum"));

        // Every sounding note is unpitched, carries its written position, its
        // voice, and the resolved GM number; none is exposed as a pitch.
        let m1 = &doc.measures[0];
        let hat = &m1.notes[0];
        assert!(hat.pitch.is_none() && !hat.is_rest);
        let u = hat.unpitched.as_ref().unwrap();
        assert_eq!((u.display_step, u.display_octave), ('G', 5));
        assert_eq!(u.gm_number, Some(42));
        assert_eq!(hat.voice, 1);
        // The chorded snare shares the hat's onset.
        let snare_note = m1
            .notes
            .iter()
            .find(|n| {
                n.unpitched
                    .as_ref()
                    .is_some_and(|u| u.gm_number == Some(38))
            })
            .unwrap();
        assert!(snare_note.is_chord);
        // The kick is in voice 2, stems down.
        let kick = m1
            .notes
            .iter()
            .find(|n| {
                n.unpitched
                    .as_ref()
                    .is_some_and(|u| u.gm_number == Some(36))
            })
            .unwrap();
        assert_eq!(kick.voice, 2);
        assert_eq!(kick.stem, Some(StemDir::Down));

        // The percussion clef is reported as such.
        assert_eq!(doc.attributes.clefs[0].sign, ClefSign::Percussion);
    }

    #[test]
    fn unresolvable_instrument_id_is_left_unknown_and_unknown_clef_degrades() {
        let doc = parse(crate::fixtures::DEGRADED.as_bytes()).unwrap();
        let notes = &doc.measures[0].notes;
        assert_eq!(notes[0].unpitched.as_ref().unwrap().gm_number, Some(38));
        // The id absent from the part list resolves to unknown, never guessed
        // from the written position.
        assert_eq!(notes[1].unpitched.as_ref().unwrap().gm_number, None);
        assert_eq!(notes[1].instrument_id.as_deref(), Some("P9-MISSING"));
        // A `TAB` clef sign is not recognised: the staff keeps its default —
        // no clef is recorded at all.
        assert!(doc.attributes.clefs.is_empty());
    }

    #[test]
    fn score_without_instrument_table_still_parses() {
        // MINIMAL declares no <score-instrument>: the table is empty.
        let doc = parse(MINIMAL.as_bytes()).unwrap();
        assert!(doc.instruments.is_empty());
    }

    #[test]
    fn sole_instrument_fallback_and_empty_unpitched_default() {
        let doc = parse(crate::fixtures::SOLE_INSTRUMENT.as_bytes()).unwrap();
        let notes = &doc.measures[0].notes;
        // No <instrument> reference, one declared instrument: both notes
        // resolve to the tambourine (element 55 → GM 54).
        assert_eq!(notes[0].unpitched.as_ref().unwrap().gm_number, Some(54));
        // The empty <unpitched/> is a valid note at the middle-line default.
        let u = notes[1].unpitched.as_ref().unwrap();
        assert_eq!((u.display_step, u.display_octave), ('B', 4));
        assert_eq!(u.gm_number, Some(54));
    }

    #[test]
    fn several_declared_instruments_leave_a_referenceless_note_unknown() {
        // DEGRADED declares two instruments: a note with no <instrument>
        // element cannot use the sole-instrument fallback.
        let xml = crate::fixtures::DEGRADED.replace(r#"<instrument id="P1-I38"/>"#, "");
        let doc = parse(xml.as_bytes()).unwrap();
        let notes = &doc.measures[0].notes;
        assert_eq!(notes[0].unpitched.as_ref().unwrap().gm_number, None);
    }

    #[test]
    fn degenerate_note_is_skipped_but_advances_the_cursor() {
        let xml = r#"<score-partwise><part-list><score-part id="P1"/></part-list>
        <part id="P1"><measure number="1">
          <attributes><divisions>1</divisions></attributes>
          <note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration></note>
          <note><duration>1</duration></note>
          <note><pitch><step>D</step><octave>4</octave></pitch><duration>1</duration></note>
        </measure></part></score-partwise>"#;
        let doc = parse(xml.as_bytes()).unwrap();
        let notes = &doc.measures[0].notes;
        // The degenerate middle note is dropped…
        assert_eq!(notes.len(), 2);
        // …but its duration still advanced the cursor: D4 sits at position 2.
        assert_eq!(notes[1].position_divisions, 2);
    }

    #[test]
    fn mixed_score_keeps_both_channels() {
        let doc = parse(crate::fixtures::MIXED.as_bytes()).unwrap();
        let notes = &doc.measures[0].notes;
        assert!(notes[0].pitch.is_some() && notes[0].unpitched.is_none());
        assert!(notes[1].pitch.is_none() && notes[1].unpitched.is_some());
        assert_eq!(notes[1].unpitched.as_ref().unwrap().gm_number, Some(38));
    }
}
