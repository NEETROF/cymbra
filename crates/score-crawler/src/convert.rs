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

//! Conversion to validated, spec-compliant compressed MusicXML (`.mxl`).
//!
//! Native MusicXML is validated (genuinely MusicXML, not arbitrary XML) via the
//! shared [`cymbra_musicxml_core`] and compressed into a proper `.mxl`
//! container (`META-INF/container.xml` → internal `score.musicxml`), then every
//! produced `.mxl` is re-opened and re-parsed to confirm it round-trips. The
//! external converters (MuseScore CLI, Verovio, `python-ly`) attach here as
//! subprocess steps; MIDI is never treated as a score source.

use std::io::{Cursor, Write};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

/// Internal member name for the score inside the `.mxl` container.
const INNER_NAME: &str = "score.musicxml";

/// The spec `META-INF/container.xml` pointing at [`INNER_NAME`].
const CONTAINER_XML: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    "<container>\n",
    "  <rootfiles>\n",
    "    <rootfile full-path=\"score.musicxml\" ",
    "media-type=\"application/vnd.recordare.musicxml+xml\"/>\n",
    "  </rootfiles>\n",
    "</container>\n",
);

/// The origin format of a harvested item, before conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginFormat {
    MusicXml,
    Mxl,
    MuseScore,
    LilyPond,
    Mei,
}

/// The conversion outcome recorded per item in the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionStatus {
    /// Produced a validated `.mxl`.
    Converted,
    /// Conversion to MusicXML failed/degraded; the original source was kept
    /// instead (e.g. LilyPond `.ly` + PDF) rather than emitting dubious output.
    FailedKeptSource,
    /// Conversion failed and nothing usable was produced.
    Failed,
    /// Not converted by policy (e.g. a MIDI-only item).
    Skipped,
}

/// A successful conversion: the `.mxl` bytes plus its recorded status.
#[derive(Debug, Clone)]
pub struct Converted {
    pub mxl: Vec<u8>,
    pub status: ConversionStatus,
}

/// Validates native MusicXML (rejecting non-MusicXML XML), compresses it to a
/// spec-compliant `.mxl`, and verifies the result re-parses.
pub fn convert_native(musicxml: &[u8]) -> Result<Converted> {
    // Reject well-formed-but-not-MusicXML input up front.
    cymbra_musicxml_core::validate(musicxml)
        .map_err(|r| anyhow!("input is not valid playable MusicXML: {r}"))?;
    let mxl = compress_to_mxl(musicxml)?;
    verify_mxl(&mxl).context("produced .mxl failed re-parse verification")?;
    Ok(Converted {
        mxl,
        status: ConversionStatus::Converted,
    })
}

/// Accepts an already-compressed `.mxl`: verifies it re-parses (and is genuine
/// MusicXML inside) rather than trusting the source blindly.
pub fn accept_mxl(mxl: &[u8]) -> Result<Converted> {
    verify_mxl(mxl).context(".mxl failed re-parse verification")?;
    Ok(Converted {
        mxl: mxl.to_vec(),
        status: ConversionStatus::Converted,
    })
}

/// Central conversion dispatch by origin format. Native MusicXML and `.mxl` are
/// handled in-process; the external-binary formats (MuseScore, LilyPond, MEI)
/// are wired as subprocess steps in a later slice and currently return an error
/// the orchestrator isolates as a per-item failure. MIDI is never a score
/// source and must not reach this function.
pub fn convert_any(origin: OriginFormat, bytes: &[u8]) -> Result<Converted> {
    match origin {
        OriginFormat::MusicXml => convert_native(bytes),
        OriginFormat::Mxl => accept_mxl(bytes),
        OriginFormat::MuseScore | OriginFormat::LilyPond | OriginFormat::Mei => Err(anyhow!(
            "external converter for {origin:?} is not wired yet (MuseScore CLI / python-ly / Verovio)"
        )),
    }
}

/// Builds a spec-compliant `.mxl` (ZIP: `META-INF/container.xml` → internal
/// `score.musicxml`) from raw MusicXML bytes.
pub fn compress_to_mxl(musicxml: &[u8]) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opts: zip::write::FileOptions<()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("META-INF/container.xml", opts)
            .context("mxl: start container.xml")?;
        zip.write_all(CONTAINER_XML.as_bytes())
            .context("mxl: write container.xml")?;
        zip.start_file(INNER_NAME, opts)
            .context("mxl: start score entry")?;
        zip.write_all(musicxml).context("mxl: write score entry")?;
        zip.finish().context("mxl: finalize archive")?;
    }
    Ok(buf)
}

/// Re-opens a `.mxl`, decodes its rootfile, and confirms it parses as MusicXML.
pub fn verify_mxl(mxl: &[u8]) -> Result<()> {
    if !cymbra_musicxml_core::mxl::is_mxl(mxl) {
        return Err(anyhow!("not a .mxl (missing zip magic)"));
    }
    let inner = cymbra_musicxml_core::mxl::decode(mxl).context("mxl: decode rootfile")?;
    cymbra_musicxml_core::parse(&inner).context("mxl: internal score does not parse")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCORE: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>1</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note>
    </measure>
  </part>
</score-partwise>"#;

    #[test]
    fn native_musicxml_converts_and_verifies() {
        let c = convert_native(SCORE.as_bytes()).unwrap();
        assert_eq!(c.status, ConversionStatus::Converted);
        assert!(cymbra_musicxml_core::mxl::is_mxl(&c.mxl));
        // The produced .mxl round-trips back to a parseable score.
        verify_mxl(&c.mxl).unwrap();
    }

    #[test]
    fn container_points_at_internal_score() {
        let mxl = compress_to_mxl(SCORE.as_bytes()).unwrap();
        let inner = cymbra_musicxml_core::mxl::decode(&mxl).unwrap();
        assert_eq!(inner, SCORE.as_bytes());
    }

    #[test]
    fn non_musicxml_xml_is_rejected() {
        let not_score = b"<html><body>hello</body></html>";
        assert!(convert_native(not_score).is_err());
    }

    #[test]
    fn empty_or_garbage_is_rejected() {
        assert!(convert_native(b"").is_err());
        assert!(convert_native(b"not xml at all").is_err());
    }

    #[test]
    fn accept_mxl_verifies_roundtrip() {
        let mxl = compress_to_mxl(SCORE.as_bytes()).unwrap();
        let c = accept_mxl(&mxl).unwrap();
        assert_eq!(c.status, ConversionStatus::Converted);
    }

    #[test]
    fn verify_rejects_non_mxl() {
        assert!(verify_mxl(SCORE.as_bytes()).is_err());
    }
}
