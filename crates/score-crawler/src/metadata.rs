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

//! Search-ready metadata derived from the parsed score at ingest time.
//!
//! The musical facets and normalised keys (`title_norm`, `work_key`,
//! `key_fifths`, `time_sig`, `measure_count`, staff/piano info) come from the
//! shared [`ScoreSummary`] derivation in `musicxml-core`, so the crawler's
//! `catalog_scores` and user uploads' `user_scores` are computed identically.
//! Fields the parsed model does not carry (`language`, `voicing`) are left to
//! the source adapter.

use cymbra_musicxml_core::{ScoreDocument, ScoreFacets, ScoreSummary};

/// Search/facet + musical metadata for one score, to be persisted alongside its
/// provenance in `catalog_scores`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreMetadata {
    pub title: Option<String>,
    pub composer: Option<String>,
    /// Accent-/case-folded title for typo-tolerant search.
    pub title_norm: Option<String>,
    /// Normalised `composer::title` key for dedup / grouping the same work
    /// across sources.
    pub work_key: String,
    /// The score's instrument family, derived from the notation alone
    /// (change: add-drums-access — replaces the `is_piano` staff proxy).
    pub instrument: cymbra_musicxml_core::InstrumentKind,
    pub staves: u32,
    pub key_fifths: i32,
    /// `beats/beat_type`, e.g. `4/4`.
    pub time_sig: String,
    pub measure_count: u32,
    /// Pitched (non-rest) note events.
    pub note_count: u32,
    /// Derived musical facets (for the search filters + generated cover).
    pub facets: ScoreFacets,
    /// Lyrics language — not in the parsed model; set by the source adapter.
    pub language: Option<String>,
    /// Choral voicing (e.g. SATB) — set by the source adapter.
    pub voicing: Option<String>,
}

/// Derives [`ScoreMetadata`] from a parsed document by delegating the musical
/// facets and normalised keys to the shared [`ScoreSummary`], then attaching the
/// source-supplied fields (`language`, `voicing`). Pure; never panics.
pub fn extract(doc: &ScoreDocument) -> ScoreMetadata {
    let s = ScoreSummary::from_document(doc);
    let facets = ScoreFacets::from_document(doc);
    ScoreMetadata {
        title: s.title,
        composer: s.composer,
        title_norm: s.title_norm,
        work_key: s.work_key,
        instrument: s.instrument,
        staves: s.staves,
        key_fifths: s.key_fifths,
        time_sig: s.time_sig,
        measure_count: s.measure_count,
        note_count: s.note_count,
        facets,
        language: None,
        voicing: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn meta() -> ScoreMetadata {
        let doc = cymbra_musicxml_core::parse(SCORE.as_bytes()).unwrap();
        extract(&doc)
    }

    #[test]
    fn captures_musical_facets() {
        let m = meta();
        assert_eq!(m.key_fifths, -3);
        assert_eq!(m.time_sig, "9/8");
        assert_eq!(m.staves, 2);
        assert_eq!(m.instrument, cymbra_musicxml_core::InstrumentKind::Keyboard);
        assert_eq!(m.measure_count, 2);
        assert_eq!(m.note_count, 2); // two pitched notes, the rest excluded
    }

    #[test]
    fn normalizes_title_and_work_key() {
        let m = meta();
        assert_eq!(m.title.as_deref(), Some("Clair de Lune"));
        assert_eq!(m.title_norm.as_deref(), Some("clair de lune"));
        assert_eq!(m.work_key, "claude debussy::clair de lune");
    }

    #[test]
    fn language_and_voicing_are_source_supplied() {
        let m = meta();
        assert_eq!(m.language, None);
        assert_eq!(m.voicing, None);
    }

    /// Task 3.9 of add-drums-access: a percussion score flows through the
    /// crawler's shared validate + extract path — no instrument condition is
    /// added to admission (the whitelist stays the only gate), and the derived
    /// metadata records the family the backend's enforcement keys on.
    #[test]
    fn a_percussion_score_is_ingestable_and_classified() {
        const DRUMS: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <work><work-title>Basic Groove</work-title></work>
  <part-list><score-part id="P1">
    <score-instrument id="P1-I38"><instrument-name>Snare Drum</instrument-name></score-instrument>
    <midi-instrument id="P1-I38"><midi-unpitched>39</midi-unpitched></midi-instrument>
  </score-part></part-list>
  <part id="P1"><measure number="1">
    <attributes><divisions>1</divisions><clef><sign>percussion</sign><line>2</line></clef></attributes>
    <note><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched><duration>4</duration><instrument id="P1-I38"/><voice>1</voice></note>
  </measure></part></score-partwise>"#;
        // The shared admission gate accepts it (unpitched notes are playable)…
        let summary = cymbra_musicxml_core::validate(DRUMS.as_bytes()).unwrap();
        assert!(summary.note_count > 0);
        // …and the crawler's derivation records the percussion family.
        let doc = cymbra_musicxml_core::parse(DRUMS.as_bytes()).unwrap();
        let m = extract(&doc);
        assert_eq!(m.instrument, cymbra_musicxml_core::InstrumentKind::Percussion);
        assert_eq!(m.title.as_deref(), Some("Basic Groove"));
    }

}
