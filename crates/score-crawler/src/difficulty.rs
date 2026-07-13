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

//! Difficulty estimation — source-first, heuristic fallback, provenance-tracked.
//!
//! Estimating symbolic difficulty is an open problem, so a value is never
//! recorded as authoritative unless the source supplied it. [`assess`] prefers a
//! source-declared grade (`level_source = source`); otherwise it computes a
//! cheap heuristic from parser features and marks it `level_source = heuristic`.
//! The heuristic is intentionally non-authoritative; its thresholds will need
//! calibration. A later curation pass or ML model overwrites only `heuristic`
//! rows, never a real grade.

use cymbra_musicxml_core::{DirectionKind, ScoreDocument};
use serde::{Deserialize, Serialize};

/// The three practice levels, matching the app's `PracticeLevel` vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Beginner,
    Intermediate,
    Advanced,
}

/// Where a difficulty value came from (kept so a heuristic guess is never
/// mistaken for a real grade).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelSource {
    /// The source library supplied an explicit grade.
    Source,
    /// Estimated from the score by [`estimate`].
    Heuristic,
    /// Set by a human curator.
    Manual,
}

/// A difficulty assessment: a level and its provenance, both `None` when unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DifficultyAssessment {
    pub level: Option<Level>,
    pub source: Option<LevelSource>,
}

impl DifficultyAssessment {
    /// Unknown difficulty (no source grade, heuristic not run/inconclusive).
    pub fn unknown() -> Self {
        Self {
            level: None,
            source: None,
        }
    }
}

/// Chooses a difficulty for a score: a source-declared grade wins; otherwise the
/// heuristic estimate is used. A heuristic value is never labelled `source`.
pub fn assess(doc: &ScoreDocument, source_grade: Option<Level>) -> DifficultyAssessment {
    match source_grade {
        Some(level) => DifficultyAssessment {
            level: Some(level),
            source: Some(LevelSource::Source),
        },
        None => DifficultyAssessment {
            level: Some(estimate(doc)),
            source: Some(LevelSource::Heuristic),
        },
    }
}

/// Heuristic difficulty from parser features: note density, shortest rhythmic
/// value, tuplets, polyphony, pitch range + max leap, key accidentals, staff
/// count, and tempo. Non-authoritative.
pub fn estimate(doc: &ScoreDocument) -> Level {
    let score = difficulty_score(doc);
    if score < 3.0 {
        Level::Beginner
    } else if score < 6.0 {
        Level::Intermediate
    } else {
        Level::Advanced
    }
}

/// The raw weighted difficulty score (higher = harder). Exposed for calibration.
pub fn difficulty_score(doc: &ScoreDocument) -> f64 {
    let divisions = doc.attributes.divisions.max(1) as f64;
    let measures = doc.measures.len().max(1) as f64;

    let mut pitched = 0u32;
    let mut has_fast = false;
    let mut has_tuplet = false;
    let mut has_chord = false;
    let mut min_midi = i32::MAX;
    let mut max_midi = i32::MIN;
    // Track the last pitch per staff to measure melodic leaps within a hand.
    let mut last_by_staff: std::collections::BTreeMap<u32, i32> = std::collections::BTreeMap::new();
    let mut max_leap = 0i32;

    for m in &doc.measures {
        for n in &m.notes {
            if n.tuplet.is_some() {
                has_tuplet = true;
            }
            if n.is_chord {
                has_chord = true;
            }
            // A note at or below a sixteenth (duration*4 <= a quarter) is "fast".
            if (n.duration_divisions as f64) * 4.0 <= divisions {
                has_fast = true;
            }
            if let Some(p) = &n.pitch
                && !n.is_rest
            {
                pitched += 1;
                let midi = pitch_midi(p);
                min_midi = min_midi.min(midi);
                max_midi = max_midi.max(midi);
                if let Some(prev) = last_by_staff.insert(n.staff, midi)
                    && !n.is_chord
                {
                    max_leap = max_leap.max((midi - prev).abs());
                }
            }
        }
    }

    let density = pitched as f64 / measures;
    let range = if max_midi >= min_midi {
        (max_midi - min_midi) as f64
    } else {
        0.0
    };
    let accidentals = doc.attributes.key_fifths.unsigned_abs() as f64;
    let grand = doc.staves >= 2;
    let tempo = max_tempo_bpm(doc);

    let mut score = 0.0;
    score += density * 0.5;
    score += if has_fast { 2.0 } else { 0.0 };
    score += if has_tuplet { 1.0 } else { 0.0 };
    score += if has_chord { 1.0 } else { 0.0 };
    score += if grand { 1.5 } else { 0.0 };
    score += accidentals * 0.4;
    score += (range / 12.0) * 0.5;
    score += (max_leap as f64 / 12.0) * 0.7;
    score += match tempo {
        Some(bpm) if bpm > 140 => 1.0,
        Some(bpm) if bpm > 90 => 0.5,
        _ => 0.0,
    };
    score
}

/// MIDI number for a pitch (middle C `C4` = 60).
fn pitch_midi(p: &cymbra_musicxml_core::Pitch) -> i32 {
    let step = match p.step.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => 0,
    };
    (p.octave + 1) * 12 + step + p.alter
}

/// The fastest metronome mark in the document, if any.
fn max_tempo_bpm(doc: &ScoreDocument) -> Option<u32> {
    doc.measures
        .iter()
        .flat_map(|m| &m.directions)
        .filter_map(|d| match &d.kind {
            DirectionKind::Metronome { per_minute, .. } => Some(*per_minute),
            _ => None,
        })
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(xml: &str) -> ScoreDocument {
        cymbra_musicxml_core::parse(xml.as_bytes()).unwrap()
    }

    // Sparse, single-staff, C major, whole notes → Beginner.
    const EASY: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>4</divisions><key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef></attributes>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>16</duration><type>whole</type></note>
    </measure>
    <measure number="2">
      <note><pitch><step>D</step><octave>4</octave></pitch><duration>16</duration><type>whole</type></note>
    </measure>
  </part>
</score-partwise>"#;

    // Dense, two-staff grand staff, 5 sharps, 16th notes, tuplets, big leaps,
    // fast tempo → Advanced.
    const HARD: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>4</divisions><key><fifths>5</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time><staves>2</staves>
        <clef number="1"><sign>G</sign><line>2</line></clef>
        <clef number="2"><sign>F</sign><line>4</line></clef></attributes>
      <direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>168</per-minute></metronome></direction-type></direction>
      <note><pitch><step>C</step><octave>6</octave></pitch><duration>1</duration><type>16th</type><staff>1</staff>
        <time-modification><actual-notes>3</actual-notes><normal-notes>2</normal-notes></time-modification></note>
      <note><pitch><step>C</step><octave>3</octave></pitch><duration>1</duration><type>16th</type><staff>1</staff></note>
      <note><pitch><step>G</step><octave>5</octave></pitch><duration>1</duration><type>16th</type><staff>2</staff></note>
      <note><pitch><step>E</step><octave>2</octave></pitch><duration>1</duration><type>16th</type><staff>2</staff></note>
    </measure>
  </part>
</score-partwise>"#;

    #[test]
    fn easy_score_is_beginner() {
        assert_eq!(estimate(&parse(EASY)), Level::Beginner);
    }

    #[test]
    fn hard_score_is_advanced() {
        assert_eq!(estimate(&parse(HARD)), Level::Advanced);
    }

    #[test]
    fn harder_scores_score_higher() {
        assert!(difficulty_score(&parse(HARD)) > difficulty_score(&parse(EASY)));
    }

    #[test]
    fn source_grade_wins_and_is_marked_source() {
        let a = assess(&parse(HARD), Some(Level::Beginner));
        assert_eq!(a.level, Some(Level::Beginner));
        assert_eq!(a.source, Some(LevelSource::Source));
    }

    #[test]
    fn without_source_grade_heuristic_is_flagged() {
        let a = assess(&parse(EASY), None);
        assert_eq!(a.level, Some(Level::Beginner));
        assert_eq!(a.source, Some(LevelSource::Heuristic));
    }

    #[test]
    fn middle_c_is_midi_60() {
        let p = cymbra_musicxml_core::Pitch {
            step: 'C',
            octave: 4,
            alter: 0,
        };
        assert_eq!(pitch_midi(&p), 60);
    }
}
