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

//! Score structures exposed to Flutter and demo score generator.
//!
//! For the POC, the score is hard-coded on the Rust side: the Rust engine is the
//! source of truth for the "music to play", Flutter only draws it (staff or
//! waterfall).

/// A note placed in time.
pub struct Note {
    /// MIDI note number (0-127). 60 = C4 (middle C).
    pub pitch: u8,
    /// Start of the note from the beginning of the song, in milliseconds.
    pub start_ms: u64,
    /// Duration of the note, in milliseconds.
    pub duration_ms: u64,
}

/// A musical measure: a group of notes.
pub struct Measure {
    /// Measure index (0-based).
    pub index: u32,
    pub notes: Vec<Note>,
}

/// The complete score.
pub struct Score {
    /// Tempo in beats per minute.
    pub bpm: u32,
    pub measures: Vec<Measure>,
}

/// Generates a demo score: a short melody in C major over 4 measures (4/4),
/// scale up then down. Timestamps are consistent with the BPM.
pub fn demo_score() -> Score {
    const BPM: u32 = 80;
    // Duration of a quarter note in ms: 60_000 / BPM.
    let beat_ms: u64 = 60_000 / BPM as u64;
    // We leave a small gap between notes to clearly see the rectangles.
    let note_ms: u64 = (beat_ms as f64 * 0.9) as u64;

    // Sequences of MIDI pitches, 4 quarter notes per measure.
    // C D E F | G A B C5 | C5 B A G | F E D C
    let phrases: [[u8; 4]; 4] = [
        [60, 62, 64, 65],
        [67, 69, 71, 72],
        [72, 71, 69, 67],
        [65, 64, 62, 60],
    ];

    let mut measures = Vec::with_capacity(phrases.len());
    for (m, pitches) in phrases.iter().enumerate() {
        let measure_start = m as u64 * 4 * beat_ms;
        let notes = pitches
            .iter()
            .enumerate()
            .map(|(b, &pitch)| Note {
                pitch,
                start_ms: measure_start + b as u64 * beat_ms,
                duration_ms: note_ms,
            })
            .collect();
        measures.push(Measure {
            index: m as u32,
            notes,
        });
    }

    Score { bpm: BPM, measures }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_score_has_expected_shape() {
        let s = demo_score();
        assert_eq!(s.bpm, 80);
        assert_eq!(s.measures.len(), 4);
        for (i, m) in s.measures.iter().enumerate() {
            assert_eq!(m.index, i as u32);
            assert_eq!(m.notes.len(), 4, "4 quarter notes per measure");
        }
    }

    #[test]
    fn demo_score_first_phrase_is_c_d_e_f() {
        let s = demo_score();
        let pitches: Vec<u8> = s.measures[0].notes.iter().map(|n| n.pitch).collect();
        assert_eq!(pitches, vec![60, 62, 64, 65]);
    }

    #[test]
    fn demo_score_notes_are_monotonically_scheduled() {
        let s = demo_score();
        let beat_ms = 60_000 / 80; // 750ms
        // First note starts at 0; notes are one beat apart and shorter than a beat.
        let first = &s.measures[0].notes[0];
        assert_eq!(first.start_ms, 0);
        assert!(first.duration_ms < beat_ms, "gap left between notes");

        // Second measure starts one full bar (4 beats) in.
        assert_eq!(s.measures[1].notes[0].start_ms, 4 * beat_ms);
    }
}
