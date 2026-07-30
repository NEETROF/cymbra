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

//! Shared score metadata: the derived [`ScoreSummary`] and text normalisation.
//!
//! One derivation, three consumers — the app preview, the backend upload record,
//! and the crawler's `catalog_scores` — so title/composer/key/time/piano facets
//! are computed identically everywhere (no client/server/crawler drift). Every
//! field is derived purely from the parsed document; nothing is ever taken from
//! an external (client-supplied) claim.

use unicode_normalization::UnicodeNormalization;

use crate::ScoreDocument;

/// A parsed score's derived summary — everything the parser yields for free,
/// surfaced to a caller (preview header, upload record, catalog row) without a
/// re-parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreSummary {
    pub title: Option<String>,
    pub composer: Option<String>,
    /// Accent-/case-folded title for typo-tolerant search.
    pub title_norm: Option<String>,
    /// Normalised `composer::title` key for dedup / grouping the same work
    /// across sources.
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
}

/// Musical facets derived from a parsed score for catalog search filters
/// (change: score-catalog-facets). Kept separate from [`ScoreSummary`] (which is
/// bridged to the app) so it can grow without touching the app FFI. Every field
/// is derived purely from the parse; a signal absent from the file yields
/// `None`/`false`, never a fabricated value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScoreFacets {
    /// Smallest note value present, as a power-of-two denominator
    /// (`4`=quarter, `8`=eighth, `16`=sixteenth, …). `None` when the score has no
    /// sounding notes. The "rien de plus rapide que X" filter is `min_note_value <= X`.
    pub min_note_value: Option<u8>,
    /// Any note carries a tuplet (triplet, …).
    pub has_tuplets: bool,
    /// Any note carries one or more dots (dotted rhythm).
    pub has_dotted: bool,
    /// Any note is a chord member (simultaneous pitches).
    pub has_chords: bool,
    /// Lowest / highest sounding pitch as MIDI numbers (C4 = 60) — the ambitus.
    /// `None` when there are no sounding notes.
    pub lowest_midi: Option<u8>,
    pub highest_midi: Option<u8>,
    /// Number of staves (1, or 2 for a grand staff).
    pub staff_count: u8,
    /// Count of pitched (non-rest) note events.
    pub note_count: u32,
    /// The score's marked tempo (the first metronome mark's per-minute value),
    /// or `None` when the file carries no tempo marking (never the playback default).
    pub tempo_bpm: Option<u16>,
    /// Any dynamics marking (pp…ff) is present.
    pub has_dynamics: bool,
}

impl ScoreSummary {
    /// Derives the summary from a parsed document. Pure; never panics.
    pub fn from_document(doc: &ScoreDocument) -> Self {
        let title = doc.meta.title.clone();
        let composer = doc.meta.composer.clone();
        let title_norm = title.as_deref().map(normalize_text);
        let composer_norm = composer.as_deref().map(normalize_text).unwrap_or_default();
        let work_key = format!(
            "{}::{}",
            composer_norm,
            title_norm.clone().unwrap_or_default()
        );

        let note_count = doc
            .measures
            .iter()
            .flat_map(|m| &m.notes)
            .filter(|n| n.pitch.is_some() && !n.is_rest)
            .count() as u32;

        ScoreSummary {
            title,
            composer,
            title_norm,
            work_key,
            is_piano: doc.staves >= 2,
            staves: doc.staves,
            key_fifths: doc.attributes.key_fifths,
            time_sig: format!(
                "{}/{}",
                doc.attributes.time.beats, doc.attributes.time.beat_type
            ),
            measure_count: doc.measures.len() as u32,
            note_count,
        }
    }
}

impl ScoreFacets {
    /// Derives the musical facets from a parsed document. Pure; never panics.
    pub fn from_document(doc: &crate::ScoreDocument) -> Self {
        let notes = NoteScan::of(doc);
        let (tempo_bpm, has_dynamics) = scan_directions(doc);
        ScoreFacets {
            min_note_value: notes.max_denom,
            has_tuplets: notes.has_tuplets,
            has_dotted: notes.has_dotted,
            has_chords: notes.has_chords,
            lowest_midi: notes.lowest,
            highest_midi: notes.highest,
            staff_count: doc.staves.min(u8::MAX as u32) as u8,
            note_count: notes.note_count,
            tempo_bpm,
            has_dynamics,
        }
    }
}

/// Per-note facets aggregated in one pass over the sounding notes.
#[derive(Default)]
struct NoteScan {
    note_count: u32,
    has_tuplets: bool,
    has_dotted: bool,
    has_chords: bool,
    lowest: Option<u8>,
    highest: Option<u8>,
    /// Smallest note value = largest denominator seen across sounding notes.
    max_denom: Option<u8>,
}

impl NoteScan {
    fn of(doc: &crate::ScoreDocument) -> Self {
        let divisions = doc.attributes.divisions;
        let mut s = NoteScan::default();
        for note in doc.measures.iter().flat_map(|m| &m.notes) {
            s.has_chords |= note.is_chord;
            s.has_dotted |= note.dots > 0;
            s.has_tuplets |= note.tuplet.is_some();
            // Only sounding (pitched, non-rest) notes drive counts, ambitus, and
            // the fastest-value derivation — rests and grace notes are ignored.
            let Some(pitch) = &note.pitch else { continue };
            if note.is_rest {
                continue;
            }
            s.note_count += 1;
            if let Some(midi) = pitch_to_midi(pitch) {
                s.lowest = Some(s.lowest.map_or(midi, |l| l.min(midi)));
                s.highest = Some(s.highest.map_or(midi, |h| h.max(midi)));
            }
            if let Some(denom) = note_value_denominator(note, divisions) {
                s.max_denom = Some(s.max_denom.map_or(denom, |d| d.max(denom)));
            }
        }
        s
    }
}

/// The first metronome mark's per-minute (matches the app player's tempo
/// readout; `None` when the score carries none) and whether any dynamics appear.
fn scan_directions(doc: &crate::ScoreDocument) -> (Option<u16>, bool) {
    let mut tempo_bpm = None;
    let mut has_dynamics = false;
    for dir in doc.measures.iter().flat_map(|m| &m.directions) {
        match &dir.kind {
            crate::DirectionKind::Metronome { per_minute, .. }
                if tempo_bpm.is_none() && *per_minute > 0 =>
            {
                tempo_bpm = Some((*per_minute).min(u16::MAX as u32) as u16);
            }
            crate::DirectionKind::Dynamics(_) => has_dynamics = true,
            _ => {}
        }
    }
    (tempo_bpm, has_dynamics)
}

/// MIDI number of a pitch (C4 = 60), or `None` if the step is not A–G.
fn pitch_to_midi(p: &crate::Pitch) -> Option<u8> {
    let semitone = match p.step.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let midi = (p.octave + 1) * 12 + semitone + p.alter;
    u8::try_from(midi).ok()
}

/// The note's value as a power-of-two denominator (`4`=quarter, `8`=eighth, …):
/// the notated `<type>` when present, else derived from `duration/divisions`.
/// Ignores dots (the base value drives the "fastest note" facet). `None` for a
/// zero-duration (grace) note with no type.
fn note_value_denominator(note: &crate::NoteEvent, divisions: u32) -> Option<u8> {
    if let Some(t) = note.note_type.as_deref() {
        return note_type_denominator(t);
    }
    // Fallback: quarters = duration/divisions ⇒ denominator = 4 * divisions / duration,
    // snapped to the nearest power of two in [1, 128].
    if note.duration_divisions == 0 || divisions == 0 {
        return None;
    }
    let ratio = (4.0 * divisions as f64) / note.duration_divisions as f64;
    let denom = nearest_power_of_two(ratio).clamp(1, 128);
    Some(denom as u8)
}

/// Maps a MusicXML note-type token to its power-of-two denominator.
fn note_type_denominator(t: &str) -> Option<u8> {
    Some(match t {
        "maxima" | "long" | "breve" => 1, // treat longer-than-whole as whole
        "whole" => 1,
        "half" => 2,
        "quarter" => 4,
        "eighth" => 8,
        "16th" => 16,
        "32nd" => 32,
        "64th" => 64,
        "128th" => 128,
        _ => return None,
    })
}

/// Nearest power of two to `x` (x > 0), by rounding `log2` to the nearest integer.
fn nearest_power_of_two(x: f64) -> u32 {
    if x <= 1.0 {
        return 1;
    }
    let exp = x.log2().round() as u32;
    1u32 << exp
}

/// Lowercases, strips diacritics (NFD then drop combining marks), and collapses
/// runs of whitespace to single spaces — the canonical form used for fuzzy
/// search and dedup keys.
pub fn normalize_text(s: &str) -> String {
    let folded: String = s
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase();
    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// True for Unicode combining marks (the diacritics NFD splits off).
fn is_combining_mark(c: char) -> bool {
    ('\u{0300}'..='\u{036F}').contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    const SCORE: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <work><work-title>Clair de Lune</work-title></work>
  <identification><creator type="composer">Claude Debussy</creator></identification>
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>2</divisions>
        <key><fifths>-3</fifths></key>
        <time><beats>9</beats><beat-type>8</beat-type></time>
        <staves>2</staves>
        <clef number="1"><sign>G</sign><line>2</line></clef>
        <clef number="2"><sign>F</sign><line>4</line></clef>
      </attributes>
      <note><pitch><step>D</step><octave>4</octave></pitch><duration>2</duration><staff>1</staff></note>
      <note><rest/><duration>2</duration><staff>2</staff></note>
    </measure>
    <measure number="2">
      <note><pitch><step>E</step><octave>4</octave></pitch><duration>2</duration><staff>1</staff></note>
    </measure>
  </part>
</score-partwise>"#;

    fn summary() -> ScoreSummary {
        ScoreSummary::from_document(&parse(SCORE.as_bytes()).unwrap())
    }

    #[test]
    fn captures_musical_facets() {
        let s = summary();
        assert_eq!(s.key_fifths, -3);
        assert_eq!(s.time_sig, "9/8");
        assert_eq!(s.staves, 2);
        assert!(s.is_piano);
        assert_eq!(s.measure_count, 2);
        assert_eq!(s.note_count, 2); // two pitched notes, the rest excluded
    }

    #[test]
    fn derives_title_composer_and_work_key() {
        let s = summary();
        assert_eq!(s.title.as_deref(), Some("Clair de Lune"));
        assert_eq!(s.composer.as_deref(), Some("Claude Debussy"));
        assert_eq!(s.title_norm.as_deref(), Some("clair de lune"));
        assert_eq!(s.work_key, "claude debussy::clair de lune");
    }

    #[test]
    fn single_staff_is_not_piano() {
        let xml = r#"<score-partwise><part-list><score-part id="P1"/></part-list>
        <part id="P1"><measure number="1">
          <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration></note>
        </measure></part></score-partwise>"#;
        let s = ScoreSummary::from_document(&parse(xml.as_bytes()).unwrap());
        assert!(!s.is_piano);
        assert_eq!(s.staves, 1);
        assert_eq!(s.work_key, "::"); // no title, no composer
    }

    #[test]
    fn normalize_folds_accents_case_and_whitespace() {
        assert_eq!(normalize_text("Éolienne  Op.  25"), "eolienne op. 25");
        assert_eq!(normalize_text("BÉLA  Bartók"), "bela bartok");
        assert_eq!(normalize_text("  trim  me  "), "trim me");
    }

    // --- musical facets (change: score-catalog-facets) --------------------

    /// A score exercising chord, dot, tuplet, note types, a metronome mark, and a
    /// dynamics marking — plus an ambitus from A3 (57) up to G5 (79).
    const FACETS: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"/></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>4</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
      </attributes>
      <direction><direction-type>
        <metronome><beat-unit>quarter</beat-unit><per-minute>96</per-minute></metronome>
      </direction-type></direction>
      <direction><direction-type><dynamics><mf/></dynamics></direction-type></direction>
      <note><pitch><step>A</step><octave>3</octave></pitch><duration>2</duration><type>eighth</type></note>
      <note><chord/><pitch><step>C</step><octave>4</octave></pitch><duration>2</duration><type>eighth</type></note>
      <note><pitch><step>G</step><octave>5</octave></pitch><duration>1</duration><type>16th</type></note>
      <note><pitch><step>E</step><octave>4</octave></pitch><duration>6</duration><type>quarter</type><dot/></note>
      <note><pitch><step>D</step><octave>4</octave></pitch><duration>1</duration><type>eighth</type>
        <time-modification><actual-notes>3</actual-notes><normal-notes>2</normal-notes></time-modification>
      </note>
    </measure>
  </part>
</score-partwise>"#;

    #[test]
    fn derives_rhythmic_textural_and_tempo_facets() {
        let s = ScoreFacets::from_document(&parse(FACETS.as_bytes()).unwrap());
        assert_eq!(s.min_note_value, Some(16)); // fastest is a sixteenth
        assert!(s.has_chords);
        assert!(s.has_dotted);
        assert!(s.has_tuplets);
        assert!(s.has_dynamics);
        assert_eq!(s.tempo_bpm, Some(96));
        // Ambitus: A3 = 57 … G5 = 79.
        assert_eq!(s.lowest_midi, Some(57));
        assert_eq!(s.highest_midi, Some(79));
        assert_eq!(s.staff_count, 1);
    }

    #[test]
    fn smallest_note_value_falls_back_to_duration_when_untyped() {
        // divisions=4 (per quarter). A duration-1 note with no <type> is a
        // sixteenth (4*4/1 = 16); a duration-8 note is a half (4*4/8 = 2).
        let xml = r#"<score-partwise><part-list><score-part id="P1"/></part-list>
        <part id="P1"><measure number="1">
          <attributes><divisions>4</divisions></attributes>
          <note><pitch><step>C</step><octave>4</octave></pitch><duration>8</duration></note>
          <note><pitch><step>D</step><octave>4</octave></pitch><duration>1</duration></note>
        </measure></part></score-partwise>"#;
        let s = ScoreFacets::from_document(&parse(xml.as_bytes()).unwrap());
        assert_eq!(s.min_note_value, Some(16)); // the sixteenth wins
    }

    #[test]
    fn absent_tempo_and_empty_score_are_unknown_not_fabricated() {
        // No metronome, no dynamics, only a rest → tempo/ambitus/min-value unknown.
        let xml = r#"<score-partwise><part-list><score-part id="P1"/></part-list>
        <part id="P1"><measure number="1">
          <attributes><divisions>1</divisions></attributes>
          <note><rest/><duration>4</duration></note>
        </measure></part></score-partwise>"#;
        let s = ScoreFacets::from_document(&parse(xml.as_bytes()).unwrap());
        assert_eq!(s.tempo_bpm, None);
        assert!(!s.has_dynamics);
        assert_eq!(s.min_note_value, None);
        assert_eq!(s.lowest_midi, None);
        assert_eq!(s.highest_midi, None);
        assert_eq!(s.note_count, 0);
    }

    #[test]
    fn ambitus_reflects_inferred_key_signature() {
        // An unmarked score (no <alter>/<accidental>) under three flats: a bare E4
        // sounds E♭4 = 63 and a bare A4 sounds A♭4 = 68, so the ambitus reflects the
        // inferred pitches — proof that pitch_to_midi consumes the inferred alteration.
        let xml = r#"<score-partwise><part-list><score-part id="P1"/></part-list>
        <part id="P1"><measure number="1">
          <attributes><divisions>1</divisions><key><fifths>-3</fifths></key></attributes>
          <note><pitch><step>E</step><octave>4</octave></pitch><duration>1</duration></note>
          <note><pitch><step>A</step><octave>4</octave></pitch><duration>1</duration></note>
        </measure></part></score-partwise>"#;
        let s = ScoreFacets::from_document(&parse(xml.as_bytes()).unwrap());
        assert_eq!(s.lowest_midi, Some(63)); // E♭4, not E♮ (64)
        assert_eq!(s.highest_midi, Some(68)); // A♭4, not A♮ (69)
    }
}
