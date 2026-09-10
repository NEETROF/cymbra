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

//! The `pack.lingua` container: a tiny, deterministic, self-describing binary
//! envelope.
//!
//! ```text
//! magic  "LINGUAPK"          8 bytes
//! format u16 LE              container format version
//! meta   u32 LE len + bytes  the JSON PackMeta
//! count  u16 LE              number of sections
//! repeat count times:
//!   name u8 LE len + bytes   section name (ASCII)
//!   data u32 LE len + bytes  section payload
//! ```
//!
//! The writer emits sections in the exact order given, and integers are
//! fixed-width little-endian, so the same inputs always produce byte-identical
//! output — the reproducibility the pack pipeline relies on. This is the raw
//! envelope; meaning (which section is the FST, the ranks, …) lives in
//! [`super::pack`].

/// Container magic.
pub const MAGIC: &[u8; 8] = b"LINGUAPK";
/// Container format version (distinct from a pack's `pack_version`).
pub const FORMAT_VERSION: u16 = 1;

/// A named section of a container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// The section name (e.g. `"forms"`, `"gloss.zst"`).
    pub name: String,
    /// The raw payload.
    pub data: Vec<u8>,
}

/// A container decode failure.
#[derive(Debug, PartialEq, Eq)]
pub enum FormatError {
    /// The bytes do not start with the container magic.
    BadMagic,
    /// The container format version is not understood by this build.
    UnsupportedFormat { found: u16, supported: u16 },
    /// The bytes end before a declared length (truncated / corrupt).
    Truncated,
    /// A section name is not valid UTF-8.
    BadSectionName,
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::BadMagic => write!(f, "not a lingua pack (bad magic)"),
            FormatError::UnsupportedFormat { found, supported } => {
                write!(
                    f,
                    "pack container format v{found} unsupported (this build reads v{supported})"
                )
            }
            FormatError::Truncated => write!(f, "pack is truncated or corrupt"),
            FormatError::BadSectionName => write!(f, "pack has a non-UTF-8 section name"),
        }
    }
}

impl std::error::Error for FormatError {}

/// Writes a container from a JSON metadata blob and an ordered list of
/// `(name, data)` sections. Deterministic: identical inputs → identical bytes.
pub fn write_container(meta_json: &[u8], sections: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&(meta_json.len() as u32).to_le_bytes());
    out.extend_from_slice(meta_json);
    out.extend_from_slice(&(sections.len() as u16).to_le_bytes());
    for (name, data) in sections {
        let name = name.as_bytes();
        out.push(name.len() as u8);
        out.extend_from_slice(name);
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(data);
    }
    out
}

/// Reads a container, returning the metadata blob and its sections.
pub fn read_container(bytes: &[u8]) -> Result<(Vec<u8>, Vec<Section>), FormatError> {
    let mut cur = Cursor::new(bytes);
    if cur.take(8)? != MAGIC.as_slice() {
        return Err(FormatError::BadMagic);
    }
    let format = cur.u16()?;
    if format != FORMAT_VERSION {
        return Err(FormatError::UnsupportedFormat {
            found: format,
            supported: FORMAT_VERSION,
        });
    }
    let meta_len = cur.u32()? as usize;
    let meta = cur.take(meta_len)?.to_vec();
    let count = cur.u16()? as usize;
    let mut sections = Vec::with_capacity(count);
    for _ in 0..count {
        let name_len = cur.u8()? as usize;
        let name = std::str::from_utf8(cur.take(name_len)?)
            .map_err(|_| FormatError::BadSectionName)?
            .to_owned();
        let data_len = cur.u32()? as usize;
        let data = cur.take(data_len)?.to_vec();
        sections.push(Section { name, data });
    }
    Ok((meta, sections))
}

/// A bounds-checked forward reader over a byte slice.
struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], FormatError> {
        let end = self.pos.checked_add(n).ok_or(FormatError::Truncated)?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or(FormatError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, FormatError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, FormatError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, FormatError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_meta_and_sections_in_order() {
        let bytes = write_container(
            br#"{"k":1}"#,
            &[("forms", &[1, 2, 3]), ("gloss.zst", &[9, 8])],
        );
        let (meta, sections) = read_container(&bytes).expect("decode");
        assert_eq!(meta, br#"{"k":1}"#);
        assert_eq!(sections.len(), 2);
        assert_eq!(
            sections[0],
            Section {
                name: "forms".into(),
                data: vec![1, 2, 3]
            }
        );
        assert_eq!(
            sections[1],
            Section {
                name: "gloss.zst".into(),
                data: vec![9, 8]
            }
        );
    }

    #[test]
    fn writing_is_deterministic() {
        let a = write_container(b"{}", &[("a", b"x"), ("b", b"yy")]);
        let b = write_container(b"{}", &[("a", b"x"), ("b", b"yy")]);
        assert_eq!(a, b);
    }

    #[test]
    fn bad_magic_is_rejected() {
        assert_eq!(
            read_container(b"NOTAPACK............"),
            Err(FormatError::BadMagic)
        );
    }

    #[test]
    fn unsupported_format_is_rejected() {
        let mut bytes = write_container(b"{}", &[]);
        bytes[8] = 99; // corrupt the format version LSB
        assert!(matches!(
            read_container(&bytes),
            Err(FormatError::UnsupportedFormat {
                found: 99,
                supported: 1
            })
        ));
    }

    #[test]
    fn truncation_is_caught_not_panicked() {
        let bytes = write_container(b"{}", &[("forms", &[1, 2, 3, 4])]);
        for cut in 0..bytes.len() {
            // Every prefix shorter than the whole must error, never panic.
            assert!(read_container(&bytes[..cut]).is_err());
        }
        assert!(read_container(&bytes).is_ok());
    }
}
