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

//! A musical content fingerprint — dedup that survives re-encoding.
//!
//! The SHA-256 of the canonical MusicXML catches only byte-identical files. This
//! fingerprint hashes the *music itself* — each note's onset, staff, pitch (MIDI)
//! and duration, normalised to a divisions-independent grid and canonically
//! ordered — so the same piece exported by different editors (MuseScore vs
//! LilyPond, different `divisions`) yields the same fingerprint. It ignores
//! lyrics, dynamics, and articulations, so two scores that differ only in words
//! (e.g. hymn verses over one tune) collapse to one — desirable for a
//! playable-notes corpus, but see the trade-off in the crawler README.
//!
//! Transposition is NOT normalised away: a piece and its transposition are
//! different playable scores and keep distinct fingerprints.

use cymbra_musicxml_core::ScoreDocument;

use crate::crawl::sha256_hex;
use crate::difficulty::pitch_midi;

/// A 1/24-of-a-quarter grid: fine enough for triplets and 32nd notes while
/// absorbing `divisions` differences between encodings.
const GRID: f64 = 24.0;

/// The musical content fingerprint (hex SHA-256) of a parsed score.
pub fn content_fingerprint(doc: &ScoreDocument) -> String {
    let div = doc.attributes.divisions.max(1) as f64;
    let mut canon = String::new();
    for m in &doc.measures {
        let mut tokens: Vec<String> = m
            .notes
            .iter()
            .map(|n| {
                let onset = grid(n.position_divisions, div);
                let dur = grid(n.duration_divisions, div);
                let pitch = match &n.pitch {
                    Some(p) if !n.is_rest => pitch_midi(p).to_string(),
                    _ => "r".to_string(),
                };
                // onset@staff:pitch:duration — sorted so voice/note ordering
                // differences between encodings don't change the fingerprint.
                format!("{onset}@{}:{pitch}:{dur}", n.staff)
            })
            .collect();
        tokens.sort();
        canon.push_str(&tokens.join(","));
        canon.push('|');
    }
    sha256_hex(canon.as_bytes())
}

/// Snaps a division count onto the shared [`GRID`], independent of `divisions`.
fn grid(divisions_ticks: u32, divisions: f64) -> i64 {
    ((divisions_ticks as f64) * GRID / divisions).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(xml: &str) -> ScoreDocument {
        cymbra_musicxml_core::parse(xml.as_bytes()).unwrap()
    }

    // The same two notes (C4, D4 quarters) but encoded with different
    // `divisions` (1 vs 4) — different bytes, identical music.
    const DIV1: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<time><beats>4</beats><beat-type>4</beat-type></time></attributes>
<note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
<note><pitch><step>D</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
</measure></part></score-partwise>"#;

    const DIV4: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Klavier</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>4</divisions>
<time><beats>4</beats><beat-type>4</beat-type></time></attributes>
<note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>quarter</type></note>
<note><pitch><step>D</step><octave>4</octave></pitch><duration>4</duration><type>quarter</type></note>
</measure></part></score-partwise>"#;

    // Different notes (E4, F4).
    const OTHER: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
<part id="P1"><measure number="1"><attributes><divisions>1</divisions>
<time><beats>4</beats><beat-type>4</beat-type></time></attributes>
<note><pitch><step>E</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
<note><pitch><step>F</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
</measure></part></score-partwise>"#;

    #[test]
    fn same_music_different_encoding_same_fingerprint() {
        assert_eq!(
            content_fingerprint(&parse(DIV1)),
            content_fingerprint(&parse(DIV4))
        );
    }

    #[test]
    fn different_music_different_fingerprint() {
        assert_ne!(
            content_fingerprint(&parse(DIV1)),
            content_fingerprint(&parse(OTHER))
        );
    }

    #[test]
    fn fingerprint_is_hex_sha256() {
        let fp = content_fingerprint(&parse(DIV1));
        assert_eq!(fp.len(), 64);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
