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
//! Everything the parser yields for free is captured now (a later search API
//! then needs no re-parse backfill): a normalised `title_norm` and `work_key`
//! for fuzzy search / dedup grouping, plus musical facets (`key_fifths`,
//! `time_sig`, `measure_count`, staff/piano info). Fields the parsed model does
//! not carry (`language`, `voicing`) are left to the source adapter.

use cymbra_musicxml_core::ScoreDocument;
use unicode_normalization::UnicodeNormalization;

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
    /// Grand-staff heuristic (`staves >= 2`) — a keyboard/piano proxy.
    pub is_piano: bool,
    pub staves: u32,
    pub key_fifths: i32,
    /// `beats/beat_type`, e.g. `4/4`.
    pub time_sig: String,
    pub measure_count: u32,
    /// Pitched (non-rest) note events.
    pub note_count: u32,
    /// Lyrics language — not in the parsed model; set by the source adapter.
    pub language: Option<String>,
    /// Choral voicing (e.g. SATB) — set by the source adapter.
    pub voicing: Option<String>,
}

/// Derives [`ScoreMetadata`] from a parsed document. Pure; never panics.
pub fn extract(doc: &ScoreDocument) -> ScoreMetadata {
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

    ScoreMetadata {
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
        language: None,
        voicing: None,
    }
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
        assert!(m.is_piano);
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
    fn normalize_folds_accents_case_and_whitespace() {
        assert_eq!(normalize_text("Éolienne  Op.  25"), "eolienne op. 25");
        assert_eq!(normalize_text("BÉLA  Bartók"), "bela bartok");
        assert_eq!(normalize_text("  trim  me  "), "trim me");
    }

    #[test]
    fn language_and_voicing_are_source_supplied() {
        let m = meta();
        assert_eq!(m.language, None);
        assert_eq!(m.voicing, None);
    }
}
