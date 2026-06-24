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
//! Split out of [`super::musicxml`] so it can be unit-tested (and counted by
//! `cargo llvm-cov`) on CI hosts. The parser is a streaming (SAX-style) state
//! machine over `quick-xml` events: a large score is consumed as a stream of
//! `Event`s with bounded memory, never as a full DOM.

use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use flutter_rust_bridge::frb;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

use super::musicxml::{
    Attributes, BeamState, Clef, Direction, DirectionKind, Lyric, NotationMeasure, NoteEvent,
    Pitch, ScoreDocument, ScoreMeta, StemDir, System, TimeSignature, Tuplet,
};

// --- Geometry constants --------------------------------------------------

/// Base width (px) allotted to a quarter-note time column.
pub(crate) const UNIT: f64 = 30.0;
/// Spacing exponent (< 1 ⇒ sub-linear: long notes grow slower than duration).
pub(crate) const K: f64 = 0.6;
/// Minimum width (px) a measure may ever have (e.g. a whole-measure rest).
pub(crate) const FLOOR: f64 = 60.0;
/// Fixed left padding (px) for clef/key/time at the head of a measure.
pub(crate) const LEFT_PAD: f64 = 20.0;

/// Non-linear spacing for a single time column of `duration` divisions.
///
/// `space(d) = UNIT * (d / divisions) ^ K`. With `K < 1` the growth is
/// sub-linear: doubling a note's duration grows its space by `2^K < 2`.
pub(crate) fn space(duration: u32, divisions: u32) -> f64 {
    let div = divisions.max(1) as f64;
    let d = duration.max(1) as f64;
    UNIT * (d / div).powf(K)
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

/// Greedily packs measures into systems for `available_width`, wrapping on
/// overflow. A measure wider than the line gets its own system. Order preserved.
pub(crate) fn layout_systems(doc: &ScoreDocument, available_width: f64) -> Vec<System> {
    let staves = doc.staves;
    let mut systems: Vec<System> = Vec::new();
    let mut current: Vec<u32> = Vec::new();
    let mut current_w = 0.0_f64;

    for m in &doc.measures {
        let w = m.min_width;
        if current.is_empty() {
            current.push(m.index);
            current_w = w;
            // Oversized single measure occupies its own system.
            if current_w > available_width {
                systems.push(System {
                    measures: std::mem::take(&mut current),
                    staves,
                });
                current_w = 0.0;
            }
        } else if current_w + w <= available_width {
            current.push(m.index);
            current_w += w;
        } else {
            systems.push(System {
                measures: std::mem::take(&mut current),
                staves,
            });
            current.push(m.index);
            current_w = w;
            if current_w > available_width {
                systems.push(System {
                    measures: std::mem::take(&mut current),
                    staves,
                });
                current_w = 0.0;
            }
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
pub(crate) fn parse(input: &[u8]) -> Result<ScoreDocument> {
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

/// Mutable parsing state threaded through the event loop.
///
/// `#[frb(ignore)]` keeps this internal state machine out of the generated
/// bridge (it is an implementation detail, not part of the FFI surface).
#[frb(ignore)]
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
    // parts / measures
    current_part: u32,
    measure_index: u32,
    cursor: u32,
    last_onset: u32,
    notes: Vec<NoteEvent>,
    directions: Vec<Direction>,
    measure_clefs: Vec<Clef>,
    measures: Vec<NotationMeasure>,
    // per-element builders
    note: Option<NoteEvent>,
    pitch: Option<Pitch>,
    clef_number: u32,
    clef_sign: char,
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
            note: None,
            pitch: None,
            clef_number: 1,
            clef_sign: 'G',
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
                });
            }
            b"chord" => {
                if let Some(n) = self.note.as_mut() {
                    n.is_chord = true;
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
            b"clef" => {
                self.clef_number = attr(e, b"number").and_then(|s| s.parse().ok()).unwrap_or(1);
                self.clef_sign = 'G';
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
                self.clef_sign = text.chars().next().unwrap_or('G');
            }
            b"line" => {
                if let Ok(v) = text.parse() {
                    self.clef_line = v;
                }
            }
            b"clef" => {
                let clef = Clef {
                    staff: self.clef_number,
                    sign: self.clef_sign,
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
                }
            }
            b"pitch" => {
                if let Some(n) = self.note.as_mut() {
                    n.pitch = self.pitch.take();
                }
            }
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
        if n.is_chord {
            n.position_divisions = self.last_onset;
            self.notes.push(n);
        } else {
            n.position_divisions = self.cursor;
            self.last_onset = self.cursor;
            self.cursor = self.cursor.saturating_add(n.duration_divisions);
            self.notes.push(n);
        }
    }

    /// Finalizes the current measure (first part only), computing its geometry
    /// and resetting the per-measure cursor.
    fn finish_measure(&mut self) {
        let notes = std::mem::take(&mut self.notes);
        let directions = std::mem::take(&mut self.directions);
        let clefs = std::mem::take(&mut self.measure_clefs);
        // Only the first part contributes to the (single-part) document.
        if self.current_part <= 1 {
            let width = min_width(&notes, self.divisions);
            self.measures.push(NotationMeasure {
                index: self.measure_index,
                notes,
                directions,
                clefs,
                min_width: width,
            });
            self.measure_index += 1;
        }
        self.cursor = 0;
        self.last_onset = 0;
    }

    fn into_document(self) -> ScoreDocument {
        ScoreDocument {
            meta: ScoreMeta {
                title: self.title,
                composer: self.composer,
            },
            staves: self.staves.unwrap_or(1).max(1),
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
        assert_eq!((treble.sign, treble.line), ('G', 2));
        assert_eq!((bass.sign, bass.line), ('F', 4));
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
        assert_eq!((init2.sign, init2.line), ('G', 2));
        // Measure 1 declares both clefs; measure 2 changes staff 2 to bass.
        assert!(
            doc.measures[0]
                .clefs
                .iter()
                .any(|c| c.staff == 2 && c.sign == 'G')
        );
        let m2 = doc.measures[1].clefs.iter().find(|c| c.staff == 2).unwrap();
        assert_eq!((m2.sign, m2.line), ('F', 4));
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
    fn shared_columns_are_not_double_counted() {
        // Two notes at the same position (e.g. a chord) form one column.
        let note = |pos: u32| NoteEvent {
            staff: 1,
            voice: 1,
            position_divisions: pos,
            pitch: None,
            is_rest: false,
            is_chord: false,
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
                    min_width: w,
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
    fn empty_document_lays_out_to_no_systems() {
        let doc = doc_with_widths(&[]);
        assert!(layout_systems(&doc, 500.0).is_empty());
    }
}
