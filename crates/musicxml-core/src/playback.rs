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

//! Visual/audio playback timing derived from a parsed [`ScoreDocument`].
//!
//! Turns the score into a flat, onset-sorted list of pitched notes with absolute
//! millisecond times, plus each measure's start time — the schedule the browser
//! preview uses to drive its synth and its animated playhead. This mirrors the
//! app's `notationToTimedNotes` (Dart) so the console times notes exactly as the
//! app does; it is pure (no IO/FFI), host- and wasm-testable.

use crate::model::{Pitch, ScoreDocument};

/// Default tempo when the score carries no `metronome` direction (matches the app).
pub const DEFAULT_BPM: u16 = 90;

/// Note velocity used for playback (the app plays every note at a constant velocity).
pub const DEFAULT_VELOCITY: u8 = 100;

const SEMITONE_OF_STEP: [(char, i32); 7] = [
    ('C', 0),
    ('D', 2),
    ('E', 4),
    ('F', 5),
    ('G', 7),
    ('A', 9),
    ('B', 11),
];

/// MIDI note number for a pitch (C4 = 60).
pub fn midi_of_pitch(pitch: &Pitch) -> i32 {
    let base = SEMITONE_OF_STEP
        .iter()
        .find(|(s, _)| *s == pitch.step)
        .map(|(_, v)| *v)
        .unwrap_or(0);
    (pitch.octave + 1) * 12 + base + pitch.alter
}

/// One pitched note with absolute playback times. `measure_index`/`note_index`
/// address the note in the document (`measures[measure_index].notes[note_index]`),
/// so the painter and the playhead can correlate a sounding note with its glyph.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimedNote {
    pub midi: i32,
    pub onset_ms: u32,
    pub duration_ms: u32,
    pub staff: u32,
    pub measure_index: u32,
    pub note_index: u32,
}

/// A playable schedule: onset-sorted pitched notes, each measure's start time, the
/// total length, and the tempo used.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlaybackSchedule {
    pub notes: Vec<TimedNote>,
    pub measure_start_ms: Vec<u32>,
    pub song_end_ms: u32,
    pub bpm: u16,
}

/// First `metronome` per-minute in the score (as quarter BPM), else [`DEFAULT_BPM`].
fn tempo_of(document: &ScoreDocument) -> u16 {
    for measure in &document.measures {
        for dir in &measure.directions {
            if let crate::model::DirectionKind::Metronome { per_minute, .. } = &dir.kind
                && *per_minute > 0
            {
                return (*per_minute).min(u32::from(u16::MAX)) as u16;
            }
        }
    }
    DEFAULT_BPM
}

/// Derive the playback schedule from a parsed score. Rests and pitchless notes are
/// omitted from `notes` (they make no sound); their measures still contribute to the
/// running time. Notes are returned sorted by onset.
pub fn schedule(document: &ScoreDocument) -> PlaybackSchedule {
    let divisions = document.attributes.divisions.max(1);
    let bpm = tempo_of(document);
    let ms_per_division = (60000.0 / f64::from(bpm)) / f64::from(divisions);

    let time = &document.attributes.time;
    let beat_type = if time.beat_type == 0 {
        4
    } else {
        time.beat_type
    };
    let divisions_per_measure = divisions * time.beats * 4 / beat_type;

    let mut notes: Vec<TimedNote> = Vec::new();
    let mut measure_start_ms: Vec<u32> = Vec::new();
    let mut song_end_ms = 0.0_f64;
    let mut measure_start_div: u32 = 0;

    for (mi, measure) in document.measures.iter().enumerate() {
        measure_start_ms.push((f64::from(measure_start_div) * ms_per_division).round() as u32);
        let mut measure_span = divisions_per_measure;
        for (ni, note) in measure.notes.iter().enumerate() {
            let end = note.position_divisions + note.duration_divisions;
            if end > measure_span {
                measure_span = end;
            }
            let start_ms = f64::from(measure_start_div + note.position_divisions) * ms_per_division;
            let duration_ms = f64::from(note.duration_divisions) * ms_per_division;

            let Some(pitch) = note.pitch.as_ref() else {
                continue;
            };
            if note.is_rest {
                continue;
            }
            notes.push(TimedNote {
                midi: midi_of_pitch(pitch),
                onset_ms: start_ms.round() as u32,
                duration_ms: duration_ms.round() as u32,
                staff: note.staff,
                measure_index: mi as u32,
                note_index: ni as u32,
            });
            if start_ms + duration_ms > song_end_ms {
                song_end_ms = start_ms + duration_ms;
            }
        }
        measure_start_div += measure_span;
    }

    notes.sort_by_key(|n| n.onset_ms);
    PlaybackSchedule {
        notes,
        measure_start_ms,
        song_end_ms: song_end_ms.round() as u32,
        bpm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    const TWO_MEASURES: &str = r#"<?xml version="1.0"?>
<score-partwise version="3.1">
  <part-list><score-part id="P1"/></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>4</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>120</per-minute></metronome></direction-type></direction>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>quarter</type><staff>1</staff></note>
      <note><rest/><duration>4</duration><type>quarter</type><staff>1</staff></note>
      <note><pitch><step>E</step><octave>4</octave></pitch><duration>8</duration><type>half</type><staff>1</staff></note>
    </measure>
    <measure number="2">
      <note><pitch><step>G</step><octave>4</octave></pitch><duration>16</duration><type>whole</type><staff>1</staff></note>
    </measure>
  </part>
</score-partwise>"#;

    #[test]
    fn midi_of_middle_c() {
        assert_eq!(
            midi_of_pitch(&Pitch {
                step: 'C',
                octave: 4,
                alter: 0
            }),
            60
        );
        assert_eq!(
            midi_of_pitch(&Pitch {
                step: 'A',
                octave: 4,
                alter: 0
            }),
            69
        );
        assert_eq!(
            midi_of_pitch(&Pitch {
                step: 'E',
                octave: 4,
                alter: -1
            }),
            63
        );
    }

    #[test]
    fn schedules_notes_with_metronome_tempo() {
        let doc = parse(TWO_MEASURES.as_bytes()).unwrap();
        let s = schedule(&doc);
        // Tempo from the metronome mark, not the default.
        assert_eq!(s.bpm, 120);
        // 120 bpm, divisions=4 → ms/division = (60000/120)/4 = 125ms.
        // Rest is skipped, so 3 pitched notes: C4@0, E4@250, G4@500(measure 2).
        assert_eq!(s.notes.len(), 3);
        assert_eq!(s.notes[0].midi, 60);
        assert_eq!(s.notes[0].onset_ms, 0);
        assert_eq!(s.notes[0].duration_ms, 500); // quarter = 4 div * 125
        // E4 starts after the quarter + quarter rest (8 divisions) = 1000ms.
        assert_eq!(s.notes[1].midi, 64);
        assert_eq!(s.notes[1].onset_ms, 1000);
        // Measure 2 starts at 16 divisions = 2000ms; G4 whole note there.
        assert_eq!(s.measure_start_ms, vec![0, 2000]);
        assert_eq!(s.notes[2].midi, 67);
        assert_eq!(s.notes[2].onset_ms, 2000);
        assert_eq!(s.song_end_ms, 4000); // 2000 + whole (16 div * 125)
        // Correlation ids: E4 is note_index 2 in measure 0 (after C4 and the rest).
        assert_eq!((s.notes[1].measure_index, s.notes[1].note_index), (0, 2));
    }

    #[test]
    fn defaults_tempo_without_metronome() {
        let xml = TWO_MEASURES.replace(
            "<direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>120</per-minute></metronome></direction-type></direction>",
            "",
        );
        let doc = parse(xml.as_bytes()).unwrap();
        assert_eq!(schedule(&doc).bpm, DEFAULT_BPM);
    }
}
