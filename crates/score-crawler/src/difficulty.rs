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
//!
//! The heuristic is **instrument-aware** (change: add-drum-scoring). The pitched
//! features — ambitus, melodic leaps, key accidentals, grand staff — count only
//! pitched notes, so on a drum part every one of them is zero and every groove
//! would grade Beginner regardless of content. A percussion score is therefore
//! scored from drum-shaped features instead ([`percussion_score`]): stroke
//! density, tempo, the fastest subdivision, limb simultaneity and kit breadth.
//!
//! Both paths feed the **same** [`estimate`] thresholds and emit the same three
//! levels, because the difficulty *weight* of the play rewards and the global
//! boards is deliberately one function over one scale
//! (`difficulty_weight_of`) — a percussion-specific axis would fork it and force
//! every consumer to know the instrument. The keyboard path is untouched.

use cymbra_musicxml_core::{DirectionKind, InstrumentKind, ScoreDocument, instrument_of};
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

/// Heuristic difficulty from parser features. For a keyboard score: note
/// density, shortest rhythmic value, tuplets, polyphony, pitch range + max leap,
/// key accidentals, staff count, and tempo. For a percussion score: stroke
/// density, fastest subdivision, limb simultaneity, kit breadth and tempo.
/// Non-authoritative.
///
/// The thresholds are shared by both instruments **on purpose** — see the module
/// docs. Changing them moves keyboard grades too.
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

/// The raw weighted difficulty score (higher = harder), instrument-aware.
/// Exposed for calibration.
pub fn difficulty_score(doc: &ScoreDocument) -> f64 {
    match instrument_of(doc) {
        InstrumentKind::Percussion => percussion_score(doc),
        // Keyboard *and* Unknown (mixed content, or no notes at all) keep the
        // pitched path: it is what those rows were graded with, and its terms
        // stay meaningful the moment a pitch exists.
        InstrumentKind::Keyboard | InstrumentKind::Unknown => keyboard_score(doc),
    }
}

/// The pitched-note heuristic. Unchanged since `add-score-crawler`: keyboard
/// grades (and therefore the keyboard difficulty weights) must not move when
/// percussion gains its own path.
fn keyboard_score(doc: &ScoreDocument) -> f64 {
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

/// The drum-shaped features a percussion score is graded from. Public so a
/// calibration test can read them one by one instead of only the blended score.
#[derive(Debug, Clone, PartialEq)]
pub struct PercussionFeatures {
    /// Sounding strokes per written measure (unpitched, non-rest, non-grace).
    pub density: f64,
    /// Fastest subdivision present, as a note-value denominator (`4` = quarter,
    /// `8` = eighth, `16` = sixteenth …; a tuplet lands between two of them).
    /// `4` when the score has nothing shorter than a quarter — or no strokes.
    pub subdivision: u32,
    /// Most distinct kit pieces struck at one onset — the limb count the bar
    /// demands at its busiest moment.
    pub max_simultaneous: u32,
    /// Distinct voices carrying strokes — the conventional stems-up/stems-down
    /// split of hands against feet, and the cheapest signal that the writing is
    /// genuinely multi-limb.
    pub voices: u32,
    /// Distinct kit pieces used anywhere in the score.
    pub kit_pieces: u32,
    /// Fastest metronome mark, when the score carries one.
    pub tempo_bpm: Option<u32>,
}

/// Extracts the drum-shaped features of a percussion score.
///
/// Grace notes are excluded: they occupy no musical time (`duration_divisions`
/// stays 0, the cursor does not advance), so counting them would both inflate
/// the density and — since they share the following note's position — fabricate
/// simultaneity out of a flam. They are outside the scored onsets for the same
/// reason (`add-drum-scoring` design).
///
/// Simultaneity is measured over *distinct* pieces at one (measure, position)
/// column, so a doubled unison stroke is one limb, and a note whose instrument
/// could not be resolved (`gm_number: None`) is skipped rather than counted as
/// some fabricated piece.
pub fn percussion_features(doc: &ScoreDocument) -> PercussionFeatures {
    let divisions = doc.attributes.divisions.max(1) as f64;
    let measures = doc.measures.len().max(1) as f64;

    use std::collections::{BTreeMap, BTreeSet};

    let mut strokes = 0u32;
    let mut kit_pieces: BTreeSet<u32> = BTreeSet::new();
    let mut voices: BTreeSet<u32> = BTreeSet::new();
    let mut shortest = f64::INFINITY;
    // (measure index, position within the measure) → the distinct pieces struck
    // there. Positions are running divisions set through backup/forward, so a
    // hand voice and a foot voice on the same beat land in the same column.
    let mut onsets: BTreeMap<(u32, u32), BTreeSet<u32>> = BTreeMap::new();

    for m in &doc.measures {
        for n in &m.notes {
            if n.is_rest || n.is_grace {
                continue;
            }
            let Some(u) = &n.unpitched else { continue };
            strokes += 1;
            voices.insert(n.voice);
            if n.duration_divisions > 0 {
                shortest = shortest.min(n.duration_divisions as f64);
            }
            if let Some(gm) = u.gm_number {
                kit_pieces.insert(gm);
                onsets
                    .entry((m.index, n.position_divisions))
                    .or_default()
                    .insert(gm);
            }
        }
    }

    // Note value denominator: a quarter lasts `divisions`, so denominator =
    // 4 * divisions / duration. Floored at a quarter — nothing slower than a
    // quarter makes a groove easier than one written in quarters — and capped
    // so a malformed duration cannot dominate the blend.
    let subdivision = if shortest.is_finite() && shortest > 0.0 {
        (4.0 * divisions / shortest).round().clamp(4.0, 64.0) as u32
    } else {
        4
    };

    PercussionFeatures {
        density: strokes as f64 / measures,
        subdivision,
        max_simultaneous: onsets.values().map(|p| p.len() as u32).max().unwrap_or(0),
        voices: voices.len() as u32,
        kit_pieces: kit_pieces.len() as u32,
        tempo_bpm: max_tempo_bpm(doc),
    }
}

/// The drum heuristic: the same 0-and-up scale the pitched path produces, so
/// [`estimate`]'s thresholds serve both instruments.
///
/// The weights are **calibrated against the authored bundled drum scores**
/// (`apps/music/assets/scores/{beginner,intermediate,advanced}`), each of which
/// must come out at its authored tier — see
/// `bundled_drum_scores_grade_as_authored`. When a bundled score and the
/// heuristic disagree, the weights move, never the score.
pub fn percussion_score(doc: &ScoreDocument) -> f64 {
    let f = percussion_features(doc);
    let mut score = 0.0;

    // Stroke density. A quarter-note beat runs ~7 strokes/measure, a busy
    // sixteenth groove ~15–20; the coefficient keeps that spread worth about
    // one level on its own — the *rate* of strokes matters, but a fill-heavy
    // bar is not automatically harder than a sparse one at twice the tempo.
    score += f.density * 0.10;

    // Fastest subdivision. Deliberately NOT linear in octaves: eighth-note
    // hi-hats are the beginner rock default and must stay nearly free, while
    // sixteenths are the step that separates a groove from an exercise. Bucketed
    // by range rather than by exact power of two so tuplet denominators (a
    // triplet eighth lands on 6) fall in the right bucket instead of nowhere.
    score += match f.subdivision {
        0..=4 => 0.0,
        5..=8 => 0.5,
        9..=16 => 1.6,
        17..=32 => 2.4,
        _ => 3.0,
    };

    // Limb simultaneity, in two shapes. The busiest column is the coordination
    // demand — two pieces at once is the first real one, three (two hands and a
    // foot) another step; and two-voice writing is the cheap corroborating
    // signal that hands and feet are notated against each other at all.
    score += match f.max_simultaneous {
        0 | 1 => 0.0,
        2 => 0.35,
        3 => 1.1,
        _ => 1.6,
    };
    score += if f.voices >= 2 { 0.25 } else { 0.0 };

    // Kit breadth beyond the three-piece core (hi-hat, snare, kick): every extra
    // piece is another aim point, and moving between them is the skill the
    // cascade tests. Capped so a wide kit alone cannot make a slow exercise
    // Advanced.
    score += f64::from(f.kit_pieces.saturating_sub(3)).min(6.0) * 0.45;

    // Tempo, in finer bands than the pitched path's two: drum difficulty tracks
    // tempo closely, and the whole authored range sits inside one keyboard band.
    score += match f.tempo_bpm {
        Some(bpm) if bpm > 160 => 1.5,
        Some(bpm) if bpm > 130 => 1.1,
        Some(bpm) if bpm > 110 => 0.8,
        Some(bpm) if bpm > 95 => 0.5,
        Some(bpm) if bpm > 80 => 0.25,
        _ => 0.0,
    };

    score
}

/// MIDI number for a pitch (middle C `C4` = 60).
pub(crate) fn pitch_midi(p: &cymbra_musicxml_core::Pitch) -> i32 {
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

    /// The keyboard path must not move when percussion gains its own
    /// (change: add-drum-scoring). These are the exact pre-change values of the
    /// pitched heuristic — the fixtures above plus a real bundled score — so any
    /// edit that leaks into the keyboard terms fails here rather than silently
    /// re-weighting every keyboard row and every difficulty-weighted reward.
    ///
    /// The bundled score's *tier* is the pre-existing keyboard calibration,
    /// warts and all; pinning the raw value is the point, not endorsing it.
    #[test]
    fn keyboard_scores_are_byte_for_byte_unchanged() {
        assert_eq!(difficulty_score(&parse(EASY)), 0.7000000000000001);
        assert_eq!(difficulty_score(&parse(HARD)), 13.708333333333334);
        assert_eq!(estimate(&parse(EASY)), Level::Beginner);
        assert_eq!(estimate(&parse(HARD)), Level::Advanced);

        let doc = bundled("beginner/ode_to_joy");
        assert_eq!(
            cymbra_musicxml_core::instrument_of(&doc),
            InstrumentKind::Keyboard
        );
        assert_eq!(difficulty_score(&doc), 7.3);
    }

    // --- percussion (change: add-drum-scoring) -----------------------------

    /// Parses one of the app's bundled scores from its real file. The four drum
    /// scores are the calibration corpus, so the test reads the shipping bytes
    /// rather than a trimmed copy that could drift away from them.
    fn bundled(rel: &str) -> ScoreDocument {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../apps/music/assets/scores/"
        );
        let bytes = std::fs::read(format!("{path}{rel}.musicxml"))
            .unwrap_or_else(|e| panic!("bundled score {rel} readable: {e}"));
        cymbra_musicxml_core::parse(&bytes).expect("bundled score parses")
    }

    /// The calibration gate (tasks 5.2). Each authored drum score must come out
    /// of the heuristic at the tier of the folder it ships in. When this fails,
    /// the WEIGHTS move — never the scores.
    #[test]
    fn bundled_drum_scores_grade_as_authored() {
        for (rel, expected) in [
            ("beginner/premiers_pas_batterie", Level::Beginner),
            ("beginner/rock_basique", Level::Beginner),
            ("intermediate/groove_ouvert", Level::Intermediate),
            ("advanced/autour_des_futs", Level::Advanced),
        ] {
            let doc = bundled(rel);
            assert_eq!(
                cymbra_musicxml_core::instrument_of(&doc),
                InstrumentKind::Percussion,
                "{rel} must classify as percussion for the drum path to run"
            );
            assert_eq!(
                estimate(&doc),
                expected,
                "{rel} graded {:?} (raw {:.3}), authored {expected:?}",
                estimate(&doc),
                difficulty_score(&doc)
            );
        }
    }

    /// The calibrated scores in ascending order, with margin to their
    /// thresholds — a drum score that merely *lands* on the right side of 3.0
    /// or 6.0 by 0.01 is not calibrated, it is lucky.
    #[test]
    fn bundled_drum_scores_are_strictly_ordered_with_margin() {
        let s: Vec<f64> = [
            "beginner/premiers_pas_batterie",
            "beginner/rock_basique",
            "intermediate/groove_ouvert",
            "advanced/autour_des_futs",
        ]
        .iter()
        .map(|r| difficulty_score(&bundled(r)))
        .collect();
        assert!(s.windows(2).all(|w| w[0] < w[1]), "ascending: {s:?}");
        assert!(
            s[1] < 2.75,
            "the harder beginner keeps room below 3.0: {s:?}"
        );
        assert!(
            s[2] > 3.25 && s[2] < 5.75,
            "the intermediate sits inside its band: {s:?}"
        );
        assert!(s[3] > 6.5, "the advanced clears 6.0 with margin: {s:?}");
    }

    /// A percussion part list declaring the five pieces the fixtures use.
    /// `<midi-unpitched>` is ONE-based in MusicXML, so each element value is the
    /// General MIDI number plus one (kick 36, snare 38, closed hi-hat 42, mid
    /// tom 47, crash 49).
    const KIT: &str = r#"<part-list><score-part id="P1">
    <score-instrument id="I36"><instrument-name>Bass Drum 1</instrument-name></score-instrument>
    <score-instrument id="I38"><instrument-name>Snare Drum</instrument-name></score-instrument>
    <score-instrument id="I42"><instrument-name>Closed Hi-Hat</instrument-name></score-instrument>
    <score-instrument id="I47"><instrument-name>Mid Tom</instrument-name></score-instrument>
    <score-instrument id="I49"><instrument-name>Crash Cymbal 1</instrument-name></score-instrument>
    <midi-instrument id="I36"><midi-unpitched>37</midi-unpitched></midi-instrument>
    <midi-instrument id="I38"><midi-unpitched>39</midi-unpitched></midi-instrument>
    <midi-instrument id="I42"><midi-unpitched>43</midi-unpitched></midi-instrument>
    <midi-instrument id="I47"><midi-unpitched>48</midi-unpitched></midi-instrument>
    <midi-instrument id="I49"><midi-unpitched>50</midi-unpitched></midi-instrument>
    </score-part></part-list>"#;

    /// One unpitched note on the declared piece `instr`.
    fn stroke(instr: &str, dur: u32, ty: &str, chord: bool, voice: u32, extra: &str) -> String {
        let chord = if chord { "<chord/>" } else { "" };
        format!(
            "<note>{extra}{chord}<unpitched><display-step>C</display-step>\
             <display-octave>5</display-octave></unpitched><duration>{dur}</duration>\
             <instrument id=\"{instr}\"/><voice>{voice}</voice><type>{ty}</type></note>"
        )
    }

    fn rest(dur: u32, ty: &str, voice: u32) -> String {
        format!(
            "<note><rest/><duration>{dur}</duration><voice>{voice}</voice><type>{ty}</type></note>"
        )
    }

    /// Wraps measure bodies into a percussion score (divisions of 4 = one
    /// quarter, so a sixteenth is `duration 1`).
    fn drum_score(tempo: Option<u32>, measures: &[String]) -> ScoreDocument {
        let tempo = tempo.map_or(String::new(), |bpm| {
            format!(
                "<direction><direction-type><metronome><beat-unit>quarter</beat-unit>\
                 <per-minute>{bpm}</per-minute></metronome></direction-type></direction>"
            )
        });
        let head = format!(
            "<attributes><divisions>4</divisions>\
             <time><beats>4</beats><beat-type>4</beat-type></time>\
             <clef><sign>percussion</sign><line>2</line></clef></attributes>{tempo}"
        );
        let body: String = measures
            .iter()
            .enumerate()
            .map(|(i, m)| {
                format!(
                    "<measure number=\"{}\">{}{m}</measure>",
                    i + 1,
                    if i == 0 { head.as_str() } else { "" }
                )
            })
            .collect();
        parse(&format!(
            "<?xml version=\"1.0\"?><score-partwise version=\"4.0\">{KIT}<part id=\"P1\">{body}</part></score-partwise>"
        ))
    }

    /// Sixteenth hi-hats through the bar, a crash on beat 1, a snare on beats 2
    /// and 4, and kicks under beats 2 and 3 in a second voice — dense, fast and
    /// three limbs at once.
    fn dense_groove_measure() -> String {
        let mut m = String::new();
        for i in 0..16 {
            m.push_str(&stroke("I42", 1, "16th", false, 1, ""));
            if i == 0 {
                m.push_str(&stroke("I49", 1, "16th", true, 1, ""));
            }
            if i == 4 || i == 12 {
                m.push_str(&stroke("I38", 1, "16th", true, 1, ""));
            }
        }
        m.push_str("<backup><duration>16</duration></backup>");
        m.push_str(&rest(4, "quarter", 2));
        m.push_str(&stroke("I36", 4, "quarter", false, 2, ""));
        m.push_str(&stroke("I36", 4, "quarter", false, 2, ""));
        m.push_str(&rest(4, "quarter", 2));
        m
    }

    /// A dense, fast, multi-limb groove must NOT grade Beginner — the whole
    /// point of the instrument switch. Under the pitched heuristic it would
    /// (every pitched term is zero on a drum part).
    #[test]
    fn dense_fast_multi_limb_groove_is_not_beginner() {
        let doc = drum_score(Some(150), &[dense_groove_measure(), dense_groove_measure()]);
        let f = percussion_features(&doc);
        assert_eq!(f.subdivision, 16);
        assert_eq!(f.max_simultaneous, 3); // hi-hat + snare + kick on beat 2
        assert_eq!(f.kit_pieces, 4);
        assert_ne!(estimate(&doc), Level::Beginner);
    }

    /// Why the instrument switch exists. On a drum part every pitched term —
    /// density, ambitus, melodic leap, key accidentals, grand staff — counts
    /// zero notes, so the pitched heuristic reads almost nothing and mis-grades
    /// the corpus: it puts the authored *Intermediate* groove in the Beginner
    /// band (< 3.0) and the authored *Advanced* solo below the Advanced
    /// threshold (< 6.0). Pinned so the switch cannot be quietly reverted.
    #[test]
    fn the_pitched_path_mis_grades_the_authored_drum_scores() {
        assert!(keyboard_score(&bundled("intermediate/groove_ouvert")) < 3.0);
        assert!(keyboard_score(&bundled("advanced/autour_des_futs")) < 6.0);
    }

    /// A two-piece whole-note exercise — snare and kick struck together, once a
    /// bar, no tempo mark — stays Beginner: simultaneity alone is not difficulty.
    #[test]
    fn two_piece_whole_note_exercise_is_beginner() {
        let bar = format!(
            "{}<backup><duration>16</duration></backup>{}",
            stroke("I38", 16, "whole", false, 1, ""),
            stroke("I36", 16, "whole", false, 2, "")
        );
        let doc = drum_score(None, &[bar.clone(), bar]);
        let f = percussion_features(&doc);
        assert_eq!(f.subdivision, 4); // nothing faster than a quarter
        assert_eq!(f.kit_pieces, 2);
        assert_eq!(f.density, 2.0);
        assert_eq!(estimate(&doc), Level::Beginner);
    }

    /// Provenance is instrument-blind: a source-declared grade wins on a drum
    /// score exactly as on a keyboard one, and is never relabelled `heuristic`.
    #[test]
    fn source_grade_wins_over_the_percussion_heuristic() {
        let doc = drum_score(Some(150), &[dense_groove_measure(), dense_groove_measure()]);
        // The heuristic disagrees — otherwise this test would prove nothing.
        assert_ne!(estimate(&doc), Level::Beginner);
        let a = assess(&doc, Some(Level::Beginner));
        assert_eq!(a.level, Some(Level::Beginner));
        assert_eq!(a.source, Some(LevelSource::Source));

        // Without one, the drum estimate is recorded as the guess it is.
        let a = assess(&doc, None);
        assert_eq!(a.source, Some(LevelSource::Heuristic));
        assert_eq!(a.level, Some(estimate(&doc)));
    }

    /// Grace notes occupy no musical time and share the following note's
    /// position, so counting them would both inflate the density and turn a flam
    /// into fabricated simultaneity. A doubled unison is one limb, not two.
    #[test]
    fn features_skip_grace_notes_and_collapse_unisons() {
        let bar = format!(
            "{}{}{}",
            stroke("I38", 0, "eighth", false, 1, "<grace/>"),
            stroke("I38", 16, "whole", false, 1, ""),
            stroke("I38", 16, "whole", true, 1, ""), // same piece, same column
        );
        let f = percussion_features(&drum_score(None, &[bar]));
        assert_eq!(f.density, 2.0, "the grace note is not a stroke");
        assert_eq!(
            f.subdivision, 4,
            "the grace note's value is not the fastest"
        );
        assert_eq!(f.max_simultaneous, 1, "a unison double is one piece");
    }

    /// A percussion score with no resolvable instrument table still grades:
    /// unresolved notes count as strokes but claim no piece, so the heuristic
    /// leans on what it can actually read instead of fabricating a kit.
    #[test]
    fn unresolved_percussion_notes_claim_no_kit_piece() {
        let doc = parse(
            r#"<?xml version="1.0"?><score-partwise version="4.0">
            <part-list><score-part id="P1"><part-name>Drumset</part-name></score-part></part-list>
            <part id="P1"><measure number="1">
            <attributes><divisions>4</divisions><clef><sign>percussion</sign><line>2</line></clef></attributes>
            <note><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched>
            <duration>4</duration><voice>1</voice><type>quarter</type></note>
            </measure></part></score-partwise>"#,
        );
        let f = percussion_features(&doc);
        assert_eq!(f.density, 1.0);
        assert_eq!(f.kit_pieces, 0);
        assert_eq!(f.max_simultaneous, 0);
        assert_eq!(estimate(&doc), Level::Beginner);
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
