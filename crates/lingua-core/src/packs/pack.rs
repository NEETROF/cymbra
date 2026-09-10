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

//! The pack reader: turns container bytes into a usable [`Pack`].

use std::collections::BTreeMap;
use std::io::Read;

use crate::analysis::ANALYZER_VERSION;
use crate::analysis::lexicon::{FstLexicon, Lexicon, LexiconError};
use crate::knowledge::state::FrequencyRanks;

use super::format::{FormatError, read_container};
use super::meta::PackMeta;

/// Canonical section names in a `pack.lingua`.
pub mod section {
    /// The form→lemma-id FST.
    pub const FORMS: &str = "forms";
    /// The newline-separated lemma pool (id = line index).
    pub const LEMMAS: &str = "lemmas";
    /// Per-lemma-id frequency ranks, `u32` LE (0 = unranked).
    pub const FREQ: &str = "freq";
    /// zstd-compressed, offset-indexed glosses.
    pub const GLOSS_ZST: &str = "gloss.zst";
    /// The attribution NOTICE (UTF-8).
    pub const NOTICE: &str = "notice";
}

/// Why a pack failed to load.
#[derive(Debug)]
pub enum PackError {
    /// The container envelope is bad.
    Format(FormatError),
    /// The metadata JSON is invalid.
    BadMeta(serde_json::Error),
    /// The pack was built for another analyser generation.
    IncompatibleAnalyzer { pack: String, core: String },
    /// A required section is missing.
    MissingSection(&'static str),
    /// The FST / lemma pool could not be loaded.
    Lexicon(LexiconError),
    /// A section's bytes are the wrong shape.
    Malformed(&'static str),
    /// Gloss decompression failed.
    Decompress,
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackError::Format(e) => write!(f, "{e}"),
            PackError::BadMeta(e) => write!(f, "invalid pack metadata: {e}"),
            PackError::IncompatibleAnalyzer { pack, core } => write!(
                f,
                "pack built for analyzer {pack} but this core is {core}; refusing to load"
            ),
            PackError::MissingSection(s) => write!(f, "pack is missing the {s:?} section"),
            PackError::Lexicon(e) => write!(f, "pack lexicon: {e}"),
            PackError::Malformed(s) => write!(f, "pack section {s:?} is malformed"),
            PackError::Decompress => write!(f, "pack glosses could not be decompressed"),
        }
    }
}

impl std::error::Error for PackError {}

/// A loaded language-pair data pack.
pub struct Pack {
    meta: PackMeta,
    lexicon: FstLexicon<Vec<u8>>,
    /// Rank per lemma id (0 = unranked).
    freq: Vec<u32>,
    /// Gloss per lemma id, for the lemmas that carry one.
    glosses: BTreeMap<u64, String>,
    notice: String,
}

impl Pack {
    /// Loads a pack from container bytes (an `include_bytes!`-compatible
    /// slice), refusing one built for an incompatible analyser generation so
    /// no partial analysis is ever produced.
    pub fn load(bytes: &[u8]) -> Result<Self, PackError> {
        let (meta_json, sections) = read_container(bytes).map_err(PackError::Format)?;
        let meta: PackMeta = serde_json::from_slice(&meta_json).map_err(PackError::BadMeta)?;
        if !analyzer_compatible(&meta.analyzer_version) {
            return Err(PackError::IncompatibleAnalyzer {
                pack: meta.analyzer_version.clone(),
                core: ANALYZER_VERSION.to_owned(),
            });
        }

        let take = |name: &'static str| -> Result<&[u8], PackError> {
            sections
                .iter()
                .find(|s| s.name == name)
                .map(|s| s.data.as_slice())
                .ok_or(PackError::MissingSection(name))
        };

        let forms = take(section::FORMS)?.to_vec();
        let lemmas = std::str::from_utf8(take(section::LEMMAS)?)
            .map_err(|_| PackError::Malformed(section::LEMMAS))?;
        let lexicon = FstLexicon::from_slices(forms, lemmas).map_err(PackError::Lexicon)?;

        let freq = parse_freq(take(section::FREQ)?, lexicon.lemma_count())?;
        let glosses = match sections.iter().find(|s| s.name == section::GLOSS_ZST) {
            Some(s) => parse_glosses(&s.data)?,
            None => BTreeMap::new(),
        };
        let notice = sections
            .iter()
            .find(|s| s.name == section::NOTICE)
            .map(|s| String::from_utf8_lossy(&s.data).into_owned())
            .unwrap_or_default();

        Ok(Self {
            meta,
            lexicon,
            freq,
            glosses,
            notice,
        })
    }

    /// The pack's metadata.
    pub fn meta(&self) -> &PackMeta {
        &self.meta
    }

    /// The form→lemma lexicon (feeds the lemmatisation cascade).
    pub fn lexicon(&self) -> &(impl Lexicon + '_) {
        &self.lexicon
    }

    /// The native-language gloss for a lemma, if the pack carries one.
    pub fn gloss(&self, lemma: &str) -> Option<&str> {
        let id = self.lexicon.id_of(lemma)?;
        self.glosses.get(&id).map(String::as_str)
    }

    /// The bundled attribution notice.
    pub fn notice(&self) -> &str {
        &self.notice
    }
}

impl FrequencyRanks for Pack {
    fn rank(&self, lemma: &str) -> Option<u32> {
        let id = self.lexicon.id_of(lemma)? as usize;
        match self.freq.get(id).copied() {
            Some(0) | None => None,
            Some(rank) => Some(rank),
        }
    }
}

/// A pack is compatible when its analyser version matches the running core.
/// (Exact match for now — the version is bumped on any behavioural change, so
/// mixing generations could shift counts; a looser policy can come later.)
fn analyzer_compatible(pack_version: &str) -> bool {
    pack_version == ANALYZER_VERSION
}

fn parse_freq(bytes: &[u8], lemma_count: usize) -> Result<Vec<u32>, PackError> {
    if bytes.len() != lemma_count * 4 {
        return Err(PackError::Malformed(section::FREQ));
    }
    Ok((0..lemma_count)
        .map(|i| {
            let word: [u8; 4] = bytes[i * 4..i * 4 + 4].try_into().unwrap();
            u32::from_le_bytes(word)
        })
        .collect())
}

/// Decompresses the gloss blob and reads its index. Decompressed layout:
/// `count u32 | count*(id u32, off u32, len u32) | utf8 bytes`, offsets
/// relative to the start of the utf8 payload.
fn parse_glosses(zst: &[u8]) -> Result<BTreeMap<u64, String>, PackError> {
    let raw = zstd_decode(zst)?;
    let bad = || PackError::Malformed(section::GLOSS_ZST);
    let read_u32 = |cur: &mut usize| -> Result<u32, PackError> {
        let end = cur.checked_add(4).ok_or_else(bad)?;
        let slice = raw.get(*cur..end).ok_or_else(bad)?;
        *cur = end;
        Ok(u32::from_le_bytes(slice.try_into().unwrap()))
    };
    let mut cur = 0usize;
    let count = read_u32(&mut cur)? as usize;
    let mut index = Vec::with_capacity(count);
    for _ in 0..count {
        let id = read_u32(&mut cur)?;
        let off = read_u32(&mut cur)? as usize;
        let len = read_u32(&mut cur)? as usize;
        index.push((id as u64, off, len));
    }
    let payload_start = cur;
    let mut glosses = BTreeMap::new();
    for (id, off, len) in index {
        let start = payload_start.checked_add(off).ok_or_else(bad)?;
        let end = start.checked_add(len).ok_or_else(bad)?;
        let slice = raw.get(start..end).ok_or_else(bad)?;
        let text = std::str::from_utf8(slice).map_err(|_| bad())?;
        glosses.insert(id, text.to_owned());
    }
    Ok(glosses)
}

fn zstd_decode(zst: &[u8]) -> Result<Vec<u8>, PackError> {
    let mut decoder =
        ruzstd::decoding::StreamingDecoder::new(zst).map_err(|_| PackError::Decompress)?;
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|_| PackError::Decompress)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lexicon::build_lexicon_blobs;
    use crate::packs::format::write_container;

    // Builds a gloss.zst section from (lemma_id, gloss) pairs, using the
    // C-backed `zstd` dev-dependency (never in the WASM build).
    fn build_gloss_zst(entries: &[(u32, &str)]) -> Vec<u8> {
        let mut payload = Vec::new();
        let mut index = Vec::new();
        for (id, gloss) in entries {
            let off = payload.len() as u32;
            payload.extend_from_slice(gloss.as_bytes());
            index.push((*id, off, gloss.len() as u32));
        }
        let mut raw = Vec::new();
        raw.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        for (id, off, len) in index {
            raw.extend_from_slice(&id.to_le_bytes());
            raw.extend_from_slice(&off.to_le_bytes());
            raw.extend_from_slice(&len.to_le_bytes());
        }
        raw.extend_from_slice(&payload);
        zstd::encode_all(raw.as_slice(), 19).expect("zstd encode")
    }

    fn meta_json(analyzer: &str) -> Vec<u8> {
        serde_json::to_vec(&PackMeta {
            studied: "en".into(),
            native: "fr".into(),
            pack_version: "2026.09.1".into(),
            analyzer_version: analyzer.into(),
            licences: vec![
                "AGID".into(),
                "wordfreq CC BY-SA".into(),
                "kaikki CC BY-SA".into(),
            ],
        })
        .unwrap()
    }

    // Assembles a small but complete en->fr pack for the current analyzer.
    fn sample_pack_bytes(analyzer: &str) -> Vec<u8> {
        let (forms, pool) = build_lexicon_blobs(
            &[("running", "run"), ("ran", "run"), ("cities", "city")],
            &["run", "city", "seldom"],
        )
        .expect("lexicon");
        // Ranks by lemma id: read the pool order to map lemma → id.
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let mut freq = vec![0u32; lex.lemma_count()];
        freq[lex.id_of("run").unwrap() as usize] = 500;
        freq[lex.id_of("city").unwrap() as usize] = 1_200;
        // `seldom` deliberately left unranked (0).
        let mut freq_bytes = Vec::new();
        for r in &freq {
            freq_bytes.extend_from_slice(&r.to_le_bytes());
        }
        let gloss = build_gloss_zst(&[
            (lex.id_of("run").unwrap() as u32, "courir"),
            (lex.id_of("city").unwrap() as u32, "ville"),
        ]);
        write_container(
            &meta_json(analyzer),
            &[
                (section::FORMS, &forms),
                (section::LEMMAS, pool.as_bytes()),
                (section::FREQ, &freq_bytes),
                (section::GLOSS_ZST, &gloss),
                (
                    section::NOTICE,
                    b"AGID: permissive. wordfreq: CC BY-SA. kaikki: CC BY-SA.",
                ),
            ],
        )
    }

    #[test]
    fn spec_loading_the_en_fr_pack_exposes_lemmas_ranks_and_glosses() {
        let pack = Pack::load(&sample_pack_bytes(ANALYZER_VERSION)).expect("load");
        assert_eq!(pack.meta().pair_key(), "en->fr");
        // Lemmatisation via the pack's lexicon.
        assert_eq!(pack.lexicon().lemma_of("running"), Some("run"));
        // Frequency ranks keyed by lemma.
        assert_eq!(pack.rank("run"), Some(500));
        assert_eq!(pack.rank("city"), Some(1_200));
        assert_eq!(pack.rank("seldom"), None); // present but unranked
        assert_eq!(pack.rank("absent"), None);
        // French glosses.
        assert_eq!(pack.gloss("run"), Some("courir"));
        assert_eq!(pack.gloss("city"), Some("ville"));
        assert_eq!(pack.gloss("seldom"), None);
        assert!(pack.notice().contains("CC BY-SA"));
    }

    #[test]
    fn spec_incompatible_pack_fails_with_no_partial_analysis() {
        let Err(err) = Pack::load(&sample_pack_bytes("0.0.0-from-another-era")) else {
            panic!("an incompatible pack must be refused");
        };
        assert!(matches!(err, PackError::IncompatibleAnalyzer { .. }));
    }

    #[test]
    fn a_missing_forms_section_is_reported() {
        let bytes = write_container(&meta_json(ANALYZER_VERSION), &[(section::NOTICE, b"x")]);
        let Err(err) = Pack::load(&bytes) else {
            panic!("expected a missing-section error")
        };
        assert!(matches!(err, PackError::MissingSection(section::FORMS)));
    }

    #[test]
    fn a_pack_without_glosses_still_loads() {
        let (forms, pool) = build_lexicon_blobs(&[("run", "run")], &[]).unwrap();
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let freq_bytes: Vec<u8> = vec![0u8; lex.lemma_count() * 4];
        let bytes = write_container(
            &meta_json(ANALYZER_VERSION),
            &[
                (section::FORMS, &forms),
                (section::LEMMAS, pool.as_bytes()),
                (section::FREQ, &freq_bytes),
            ],
        );
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(pack.gloss("run"), None);
        assert_eq!(pack.rank("run"), None);
    }
}
