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
}
