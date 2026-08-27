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

//! SoundFont preset-header evidence (change: add-drum-audio-channel).
//!
//! Answers ONE question about a `.sf2`'s bytes: which preset banks it
//! declares — the family evidence behind "is this font a drum kit". Percussion
//! presets live in bank 128 (the General MIDI convention rustysynth resolves
//! the drum channel against); everything else is melodic. Pure and
//! allocation-light: it walks the RIFF chunk tree to the `pdta`/`phdr`
//! sub-chunk and reads each preset record's bank word — no sample data is
//! touched, so a multi-hundred-MB font costs the same as a tiny one.

use anyhow::{Result, bail};

mod exclusive_class;
pub use exclusive_class::{GenPatch, stereo_exclusive_class_patches};

/// The percussion preset bank of the SoundFont/General MIDI convention.
pub const PERCUSSION_BANK: u16 = 128;

/// What the preset headers declare — the family evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FamilyEvidence {
    /// At least one preset in bank 128 (a drum kit the drum channel can play).
    pub has_percussion_presets: bool,
    /// At least one preset in a melodic bank (anything but 128).
    pub has_melodic_presets: bool,
}

/// Reads the family evidence from a SoundFont's bytes.
///
/// Errors on anything that is not a well-formed `sfbk` RIFF carrying a `phdr`
/// sub-chunk — the caller must treat that as "cannot verify", never as either
/// family.
pub fn family_evidence(bytes: &[u8]) -> Result<FamilyEvidence> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"sfbk" {
        bail!("not a SoundFont (missing RIFF/sfbk header)");
    }
    let mut pos = 12usize;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into()?) as usize;
        let body = pos + 8;
        let end = body.min(bytes.len()).saturating_add(size).min(bytes.len());
        if id == b"LIST" && bytes.get(body..body + 4) == Some(b"pdta") {
            let mut sub = body + 4;
            while sub + 8 <= end {
                let sid = &bytes[sub..sub + 4];
                let ssize = u32::from_le_bytes(bytes[sub + 4..sub + 8].try_into()?) as usize;
                let sbody = sub + 8;
                if sid == b"phdr" {
                    return evidence_of_phdr(bytes.get(sbody..sbody + ssize.min(end - sbody)));
                }
                // Chunks are word-aligned: an odd size carries a pad byte.
                sub = sbody + ssize + (ssize & 1);
            }
        }
        pos = body + size + (size & 1);
    }
    bail!("no pdta/phdr preset headers found");
}

/// Each `phdr` record is 38 bytes: a 20-byte name, u16 preset, u16 bank, …
/// The final record is the EOP terminal and declares no preset.
fn evidence_of_phdr(body: Option<&[u8]>) -> Result<FamilyEvidence> {
    let Some(body) = body else {
        bail!("phdr sub-chunk truncated");
    };
    const RECORD: usize = 38;
    let records = body.len() / RECORD;
    if records < 2 {
        bail!("phdr carries no presets");
    }
    let mut evidence = FamilyEvidence {
        has_percussion_presets: false,
        has_melodic_presets: false,
    };
    // Every record but the terminal one.
    for i in 0..records - 1 {
        let at = i * RECORD + 22;
        let bank = u16::from_le_bytes(body[at..at + 2].try_into()?);
        if bank == PERCUSSION_BANK {
            evidence.has_percussion_presets = true;
        } else {
            evidence.has_melodic_presets = true;
        }
    }
    Ok(evidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal but well-formed `sfbk` whose `phdr` declares presets
    /// in `banks` (plus the EOP terminal record).
    fn sf2_with_banks(banks: &[u16]) -> Vec<u8> {
        let mut phdr = Vec::new();
        for (i, bank) in banks.iter().enumerate() {
            let mut rec = [0u8; 38];
            rec[..4].copy_from_slice(b"Pst\0");
            rec[20..22].copy_from_slice(&(i as u16).to_le_bytes());
            rec[22..24].copy_from_slice(&bank.to_le_bytes());
            phdr.extend_from_slice(&rec);
        }
        phdr.extend_from_slice(&[0u8; 38]); // EOP terminal
        let mut pdta = b"pdta".to_vec();
        pdta.extend_from_slice(b"phdr");
        pdta.extend_from_slice(&(phdr.len() as u32).to_le_bytes());
        pdta.extend_from_slice(&phdr);
        let mut out = b"RIFF".to_vec();
        out.extend_from_slice(&((4 + 8 + pdta.len()) as u32).to_le_bytes());
        out.extend_from_slice(b"sfbk");
        out.extend_from_slice(b"LIST");
        out.extend_from_slice(&(pdta.len() as u32).to_le_bytes());
        out.extend_from_slice(&pdta);
        out
    }

    #[test]
    fn a_kit_only_font_reads_percussion_only() {
        let e = family_evidence(&sf2_with_banks(&[128, 128])).unwrap();
        assert!(e.has_percussion_presets && !e.has_melodic_presets);
    }

    #[test]
    fn a_melodic_font_reads_melodic_only() {
        let e = family_evidence(&sf2_with_banks(&[0, 8, 1])).unwrap();
        assert!(!e.has_percussion_presets && e.has_melodic_presets);
    }

    #[test]
    fn a_full_gm_font_reads_both() {
        let e = family_evidence(&sf2_with_banks(&[0, 128])).unwrap();
        assert!(e.has_percussion_presets && e.has_melodic_presets);
    }

    #[test]
    fn junk_and_presetless_fonts_are_errors_never_a_family() {
        assert!(family_evidence(b"not a font").is_err());
        assert!(family_evidence(&sf2_with_banks(&[])).is_err());
        // A truncated RIFF that never reaches phdr.
        let mut cut = sf2_with_banks(&[0]);
        cut.truncate(16);
        assert!(family_evidence(&cut).is_err());
    }
}
