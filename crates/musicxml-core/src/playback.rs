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
    /// Start time of each **played slot** of the playback order (the written
    /// measure table one-to-one when the piece has no repeats).
    pub measure_start_ms: Vec<u32>,
    /// The written measure each played slot performs, aligned with
    /// [`Self::measure_start_ms`] — a repeated written measure appears once
    /// per pass.
    pub written_measure: Vec<u32>,
    pub song_end_ms: u32,
    pub bpm: u16,
}

/// Nominal audible duration (ms) of one grace note at [`bpm`]: an eighth of a
/// quarter — short enough to read as an ornament, long enough to sound, and
/// tempo-proportional so fast pieces keep their snap.
pub fn grace_ms_of(bpm: u16) -> f64 {
    (60000.0 / f64::from(bpm.max(1))) / 8.0
}

/// For each note, how many grace slots *before its position* it occupies:
/// 0 for ordinary notes; for a run of consecutive pitched graces sharing
/// staff/voice/position (all engraved before the same principal), the first of
/// the run is furthest back (`run_len`), the last closest (`1`) — so they play
/// in document order and resolve onto the principal's beat.
fn grace_back_offsets(notes: &[crate::model::NoteEvent]) -> Vec<u32> {
    let mut back = vec![0u32; notes.len()];
    let mut run: Vec<usize> = Vec::new();
    let mut run_key: Option<(u32, u32, u32)> = None;
    let flush = |run: &mut Vec<usize>, back: &mut Vec<u32>| {
        let len = run.len() as u32;
        for (k, &i) in run.iter().enumerate() {
            back[i] = len - k as u32;
        }
        run.clear();
    };
    for (ni, note) in notes.iter().enumerate() {
        let is_grace = note.is_grace && !note.is_rest && note.pitch.is_some();
        let key = (note.staff, note.voice, note.position_divisions);
        if is_grace {
            if run_key != Some(key) {
                flush(&mut run, &mut back);
                run_key = Some(key);
            }
            run.push(ni);
        } else {
            flush(&mut run, &mut back);
            run_key = None;
        }
    }
    flush(&mut run, &mut back);
    back
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
///
/// **Grace notes** (`<grace/>`) carry no musical duration — parsed as duration 0 at
/// their principal's position. Scheduling them verbatim made them inaudible
/// zero-length notes stacked on the principal, so each grace gets a short nominal
/// duration ([`grace_ms_of`]) played *before* its position (acciaccatura style),
/// consecutive graces stacking backwards; the principal's own timing is unchanged.
pub fn schedule(document: &ScoreDocument) -> PlaybackSchedule {
    let divisions = document.attributes.divisions.max(1);
    let bpm = tempo_of(document);
    let ms_per_division = (60000.0 / f64::from(bpm)) / f64::from(divisions);
    let grace_ms = grace_ms_of(bpm);

    let time = &document.attributes.time;
    let beat_type = if time.beat_type == 0 {
        4
    } else {
        time.beat_type
    };
    let divisions_per_measure = divisions * time.beats * 4 / beat_type;

    let mut notes: Vec<TimedNote> = Vec::new();
    let mut measure_start_ms: Vec<u32> = Vec::new();
    let mut written_measure: Vec<u32> = Vec::new();
    let mut song_end_ms = 0.0_f64;
    let mut measure_start_div: u32 = 0;

    // The playback order resolved at parse time: repeated sections once per
    // pass, voltas selected, jumps followed. Documents built by hand (tests)
    // may carry no order — written order one-to-one then, as before repeats.
    let order: Vec<crate::model::PlayedMeasure> = if document.play_order.is_empty() {
        (0..document.measures.len())
            .map(|i| crate::model::PlayedMeasure {
                written_index: i as u32,
                pass: 1,
            })
            .collect()
    } else {
        document.play_order.clone()
    };

    for slot in &order {
        let wi = slot.written_index as usize;
        let Some(written) = document.measures.get(wi) else {
            continue;
        };
        // A measure-repeat (`%`) slot replays its referenced measure's notes;
        // the correlation ids stay on the sounding (source) measure.
        let mi = written.repeats.measure_repeat_of.unwrap_or(wi as u32) as usize;
        let Some(measure) = document.measures.get(mi) else {
            continue;
        };
        measure_start_ms.push((f64::from(measure_start_div) * ms_per_division).round() as u32);
        written_measure.push(wi as u32);
        let grace_back = grace_back_offsets(&measure.notes);
        let mut measure_span = divisions_per_measure;
        for (ni, note) in measure.notes.iter().enumerate() {
            let end = note.position_divisions + note.duration_divisions;
            if end > measure_span {
                measure_span = end;
            }
            let mut start_ms =
                f64::from(measure_start_div + note.position_divisions) * ms_per_division;
            let mut duration_ms = f64::from(note.duration_divisions) * ms_per_division;
            if grace_back[ni] > 0 {
                start_ms = (start_ms - f64::from(grace_back[ni]) * grace_ms).max(0.0);
                duration_ms = grace_ms;
            }

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
        written_measure,
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

    /// An acciaccatura before a principal: no `<duration>`, so it parses at the
    /// principal's position with duration 0. The schedule must give it a short
    /// nominal duration *before* the principal, whose own timing is untouched.
    const GRACE_MEASURE: &str = r#"<?xml version="1.0"?>
<score-partwise version="3.1">
  <part-list><score-part id="P1"/></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>4</divisions><time><beats>4</beats><beat-type>4</beat-type></time></attributes>
      <direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>120</per-minute></metronome></direction-type></direction>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>8</duration><voice>1</voice><type>half</type></note>
      <note><grace slash="yes"/><pitch><step>B</step><octave>4</octave></pitch><voice>1</voice><type>eighth</type></note>
      <note><pitch><step>C</step><alter>1</alter><octave>5</octave></pitch><duration>4</duration><voice>1</voice><type>eighth</type></note>
    </measure>
  </part>
</score-partwise>"#;

    #[test]
    fn grace_is_parsed_and_scheduled_before_its_principal() {
        let doc = parse(GRACE_MEASURE.as_bytes()).unwrap();
        let grace = &doc.measures[0].notes[1];
        assert!(grace.is_grace);
        assert_eq!(grace.duration_divisions, 0);
        // Shares the principal's position: the cursor did not advance.
        assert_eq!(
            grace.position_divisions,
            doc.measures[0].notes[2].position_divisions
        );

        let s = schedule(&doc);
        // 120 bpm → grace_ms = 500/8 = 62.5ms. Principal C#5 at 1000ms.
        let g = s.notes.iter().find(|n| n.midi == 71).unwrap();
        let p = s.notes.iter().find(|n| n.midi == 73).unwrap();
        assert_eq!(p.onset_ms, 1000);
        assert_eq!(p.duration_ms, 500);
        assert_eq!(g.onset_ms, (1000.0 - grace_ms_of(120)).round() as u32);
        assert_eq!(g.duration_ms, grace_ms_of(120).round() as u32);
        // The schedule is onset-sorted, so the grace precedes its principal.
        assert!(g.onset_ms < p.onset_ms);
    }

    #[test]
    fn schedule_unrolls_a_repeat_and_maps_written_measures() {
        // ‖: C4 whole :‖ then D4 whole — the section sounds twice.
        let xml = r#"<?xml version="1.0"?><score-partwise version="3.1">
<part-list><score-part id="P1"/></part-list><part id="P1">
<measure number="1">
  <attributes><divisions>4</divisions><time><beats>4</beats><beat-type>4</beat-type></time></attributes>
  <direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>120</per-minute></metronome></direction-type></direction>
  <barline location="left"><repeat direction="forward"/></barline>
  <note><pitch><step>C</step><octave>4</octave></pitch><duration>16</duration><voice>1</voice><type>whole</type></note>
  <barline location="right"><repeat direction="backward"/></barline>
</measure>
<measure number="2">
  <note><pitch><step>D</step><octave>4</octave></pitch><duration>16</duration><voice>1</voice><type>whole</type></note>
</measure>
</part></score-partwise>"#;
        let doc = parse(xml.as_bytes()).unwrap();
        let s = schedule(&doc);
        // 120 bpm → whole = 2000ms. Three played slots: m1, m1 again, m2.
        assert_eq!(s.measure_start_ms, vec![0, 2000, 4000]);
        assert_eq!(s.written_measure, vec![0, 0, 1]);
        assert_eq!(s.song_end_ms, 6000);
        let onsets: Vec<(i32, u32)> = s.notes.iter().map(|n| (n.midi, n.onset_ms)).collect();
        assert_eq!(onsets, vec![(60, 0), (60, 2000), (62, 4000)]);
    }

    #[test]
    fn schedule_replays_measure_repeat_content() {
        // m1: C4 whole; m2: a `%` measure (no notes) replaying m1.
        let xml = r#"<?xml version="1.0"?><score-partwise version="3.1">
<part-list><score-part id="P1"/></part-list><part id="P1">
<measure number="1">
  <attributes><divisions>4</divisions><time><beats>4</beats><beat-type>4</beat-type></time></attributes>
  <direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>120</per-minute></metronome></direction-type></direction>
  <note><pitch><step>C</step><octave>4</octave></pitch><duration>16</duration><voice>1</voice><type>whole</type></note>
</measure>
<measure number="2">
  <attributes><measure-style><measure-repeat type="start">1</measure-repeat></measure-style></attributes>
</measure>
</part></score-partwise>"#;
        let doc = parse(xml.as_bytes()).unwrap();
        assert_eq!(doc.measures[1].repeats.measure_repeat_of, Some(0));
        let s = schedule(&doc);
        // The `%` slot sounds m1's content instead of a bar of silence.
        let onsets: Vec<(i32, u32)> = s.notes.iter().map(|n| (n.midi, n.onset_ms)).collect();
        assert_eq!(onsets, vec![(60, 0), (60, 2000)]);
        // Correlation ids point at the sounding (source) measure's glyphs.
        assert_eq!(s.notes[1].measure_index, 0);
        assert_eq!(s.written_measure, vec![0, 1]);
        assert_eq!(s.song_end_ms, 4000);
    }

    #[test]
    fn grace_at_time_zero_is_clamped() {
        // The grace opens the piece: nothing to steal from — clamp at 0.
        let xml = GRACE_MEASURE.replace(
            "<note><pitch><step>C</step><octave>4</octave></pitch><duration>8</duration><voice>1</voice><type>half</type></note>",
            "",
        );
        let doc = parse(xml.as_bytes()).unwrap();
        let s = schedule(&doc);
        let g = s.notes.iter().find(|n| n.midi == 71).unwrap();
        assert_eq!(g.onset_ms, 0);
        assert_eq!(g.duration_ms, grace_ms_of(120).round() as u32);
    }
}
