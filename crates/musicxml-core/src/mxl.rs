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

//! Compressed MusicXML (`.mxl`) container decoding.
//!
//! An `.mxl` is a ZIP whose `META-INF/container.xml` points at the internal
//! `.musicxml` rootfile. `decode` extracts that rootfile's bytes so the plain
//! and compressed paths validate the same canonical XML. All reads are bounded
//! (a `.mxl` is attacker-controllable — guard against zip bombs).

use std::io::{Cursor, Read};

use anyhow::{Result, anyhow};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// Upper bound (bytes) on any single decompressed entry — a zip-bomb guard.
pub const MAX_DECOMPRESSED: u64 = 32 * 1024 * 1024;

/// True when the buffer begins with the ZIP local-file magic (`PK\x03\x04`),
/// i.e. looks like a compressed `.mxl` rather than plain XML.
pub fn is_mxl(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04")
}

/// Decodes an `.mxl` archive to its internal MusicXML bytes: reads
/// `META-INF/container.xml`, resolves the first `<rootfile full-path=…>`, and
/// returns that entry. Bounded by [`MAX_DECOMPRESSED`] per entry.
pub fn decode(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| anyhow!("mxl: not a valid zip: {e}"))?;

    // Resolve the rootfile path from the container manifest, dropping the
    // borrow before re-opening the archive by the resolved name.
    let inner_path = {
        let mut container = zip
            .by_name("META-INF/container.xml")
            .map_err(|e| anyhow!("mxl: missing META-INF/container.xml: {e}"))?;
        guard(container.size())?;
        let mut xml = String::new();
        container
            .by_ref()
            .take(MAX_DECOMPRESSED)
            .read_to_string(&mut xml)
            .map_err(|e| anyhow!("mxl: unreadable container.xml: {e}"))?;
        rootfile_path(&xml)?
    };

    let mut root = zip
        .by_name(&inner_path)
        .map_err(|e| anyhow!("mxl: rootfile '{inner_path}' not in archive: {e}"))?;
    guard(root.size())?;
    let mut out = Vec::new();
    root.by_ref()
        .take(MAX_DECOMPRESSED)
        .read_to_end(&mut out)
        .map_err(|e| anyhow!("mxl: unreadable rootfile: {e}"))?;
    Ok(out)
}

/// Rejects an entry whose declared uncompressed size exceeds the guard.
fn guard(declared: u64) -> Result<()> {
    if declared > MAX_DECOMPRESSED {
        return Err(anyhow!(
            "mxl: entry too large ({declared} > {MAX_DECOMPRESSED} bytes)"
        ));
    }
    Ok(())
}

/// Extracts the first `rootfile` `full-path` attribute from a container.xml.
fn rootfile_path(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) if e.name().as_ref() == b"rootfile" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"full-path" {
                        let path = attr
                            .unescape_value()
                            .map_err(|e| anyhow!("mxl: bad full-path: {e}"))?
                            .into_owned();
                        if path.is_empty() {
                            return Err(anyhow!("mxl: empty rootfile full-path"));
                        }
                        return Ok(path);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("mxl: malformed container.xml: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Err(anyhow!("mxl: no rootfile in container.xml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Builds a minimal spec-shaped `.mxl` in memory: `META-INF/container.xml`
    /// pointing at `score.musicxml`.
    fn build_mxl(container: &str, inner_name: &str, inner: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default();
            zip.start_file("META-INF/container.xml", opts).unwrap();
            zip.write_all(container.as_bytes()).unwrap();
            zip.start_file(inner_name, opts).unwrap();
            zip.write_all(inner.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        buf
    }

    const CONTAINER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container><rootfiles>
  <rootfile full-path="score.musicxml" media-type="application/vnd.recordare.musicxml+xml"/>
</rootfiles></container>"#;

    #[test]
    fn detects_zip_magic() {
        assert!(is_mxl(b"PK\x03\x04rest"));
        assert!(!is_mxl(b"<?xml"));
        assert!(!is_mxl(b""));
    }

    #[test]
    fn decodes_rootfile_bytes() {
        let mxl = build_mxl(CONTAINER, "score.musicxml", "<score-partwise/>");
        let inner = decode(&mxl).unwrap();
        assert_eq!(inner, b"<score-partwise/>");
    }

    #[test]
    fn rejects_non_zip() {
        assert!(decode(b"not a zip").is_err());
    }

    #[test]
    fn rejects_missing_rootfile_entry() {
        // container names a rootfile that isn't actually in the archive.
        let mxl = build_mxl(CONTAINER, "other.musicxml", "<score-partwise/>");
        assert!(decode(&mxl).is_err());
    }

    #[test]
    fn rejects_container_without_rootfile() {
        let bad = r#"<container><rootfiles></rootfiles></container>"#;
        let mxl = build_mxl(bad, "score.musicxml", "<score-partwise/>");
        assert!(decode(&mxl).is_err());
    }
}
