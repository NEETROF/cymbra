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
use crate::knowledge::level::{CefrLevel, CefrLevels};
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
    /// Per-lemma-id CEFR level, one `u8` each (0 = no level, 1..=6 = A1..=C2).
    /// Optional: absent for pairs with no licence-clean CEFR data. Additive, so
    /// a pack without it loads on any core and a pack with it loads on an older
    /// core that simply ignores the section.
    pub const LEVELS: &str = "levels";
    /// Multi-word expressions: an FST mapping a key — the words' dictionary
    /// forms, lowercase, joined by single spaces — to an expression id.
    /// Optional and additive, like [`LEVELS`]: written only when the pair's
    /// sources hold expressions, ignored by a core that does not read it.
    pub const EXPR: &str = "expr";
    /// zstd-compressed, offset-indexed expression glosses, in the layout
    /// [`GLOSS_ZST`] uses, keyed by the id [`EXPR`] maps to.
    pub const EXPR_ZST: &str = "expr.zst";
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
    /// CEFR level code per lemma id (0 = no level, 1..=6 = A1..=C2). Empty when
    /// the pack carries no level table.
    levels: Vec<u8>,
    /// Gloss per lemma id, for the lemmas that carry one.
    glosses: BTreeMap<u64, String>,
    /// Expression key → expression id. `None` when the pack carries no
    /// expression table.
    expressions: Option<fst::Map<Vec<u8>>>,
    /// Gloss per expression id, in the same layout as `glosses`. Empty when
    /// the pack carries no expression table.
    expression_glosses: BTreeMap<u64, String>,
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
        let levels = match sections.iter().find(|s| s.name == section::LEVELS) {
            Some(s) => parse_levels(&s.data, lexicon.lemma_count())?,
            None => Vec::new(),
        };
        let glosses = match sections.iter().find(|s| s.name == section::GLOSS_ZST) {
            Some(s) => parse_glosses(&s.data, section::GLOSS_ZST)?,
            None => BTreeMap::new(),
        };
        // The expression table is two sections that travel together, both
        // optional: a pair whose sources hold no expression ships neither, and
        // a core that predates them ignores them. Each is parsed where it is
        // found, so a half-written pack answers no expression rather than
        // refusing to load at all.
        let expressions = match sections.iter().find(|s| s.name == section::EXPR) {
            Some(s) => Some(
                fst::Map::new(s.data.clone()).map_err(|_| PackError::Malformed(section::EXPR))?,
            ),
            None => None,
        };
        let expression_glosses = match sections.iter().find(|s| s.name == section::EXPR_ZST) {
            Some(s) => parse_glosses(&s.data, section::EXPR_ZST)?,
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
            levels,
            glosses,
            expressions,
            expression_glosses,
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

    /// The native-language gloss of a multi-word expression, keyed by its
    /// dictionary form: the words' lemmas, lowercase, joined by single spaces
    /// (`starting point` is looked up as `start point`). `None` when the pack
    /// carries no expression table, or holds no such key.
    pub fn expression(&self, key: &str) -> Option<&str> {
        let id = self.expressions.as_ref()?.get(key.as_bytes())?;
        self.expression_glosses.get(&id).map(String::as_str)
    }

    /// Whether the pack carries an expression table, so a caller can skip the
    /// lookups entirely on a pair that has none.
    pub fn has_expressions(&self) -> bool {
        self.expressions.is_some()
    }

    /// The bundled attribution notice.
    pub fn notice(&self) -> &str {
        &self.notice
    }

    /// The lemmas whose frequency rank is within `[lo, hi]` inclusive, each with
    /// its gloss when the pack carries one, in ascending lemma-id order.
    /// Unranked lemmas (rank 0) are skipped. Enumerates a frequency band for the
    /// stats ladder and level-targeted seeding on pairs without CEFR data; the
    /// per-CEFR-level enumerator is [`Pack::lemmas_at_level`].
    pub fn lemmas_in_rank_band(&self, lo: u32, hi: u32) -> Vec<(&str, Option<&str>)> {
        (0..self.lexicon.lemma_count() as u64)
            .filter_map(|id| {
                let rank = *self.freq.get(id as usize)?;
                if rank == 0 || rank < lo || rank > hi {
                    return None;
                }
                let lemma = self.lexicon.lemma_at(id)?;
                let gloss = self.glosses.get(&id).map(String::as_str);
                Some((lemma, gloss))
            })
            .collect()
    }

    /// The lemmas tagged with a given CEFR `level`, each with its gloss when the
    /// pack carries one, in ascending lemma-id order. Empty when the pack has no
    /// level table. Feeds the CEFR ladder's per-level counts and level-targeted
    /// seeding. When ordering matters (commonest-first seeding) the caller sorts
    /// the result by `rank`.
    pub fn lemmas_at_level(&self, level: CefrLevel) -> Vec<(&str, Option<&str>)> {
        let code = level.to_code();
        (0..self.lexicon.lemma_count() as u64)
            .filter_map(|id| {
                if self.levels.get(id as usize).copied() != Some(code) {
                    return None;
                }
                let lemma = self.lexicon.lemma_at(id)?;
                let gloss = self.glosses.get(&id).map(String::as_str);
                Some((lemma, gloss))
            })
            .collect()
    }

    /// Whether the pack carries a CEFR level table (i.e. levels are available
    /// for this pair). When false, callers fall back to frequency bands.
    pub fn has_levels(&self) -> bool {
        !self.levels.is_empty()
    }

    /// The pack's dictionary words with their frequency rank: the ranked lemmas that
    /// carry a gloss or a CEFR level, in ascending lemma-id order. A ranked token with
    /// neither is mostly a name or noise ("london", "www"), which a vocabulary size does
    /// not count. Feeds the estimated vocabulary size.
    pub fn dictionary_words(&self) -> Vec<(&str, u32)> {
        (0..self.lexicon.lemma_count() as u64)
            .filter_map(|id| {
                let rank = *self.freq.get(id as usize)?;
                let leveled = self.levels.get(id as usize).is_some_and(|&code| code != 0);
                if rank == 0 || !(leveled || self.glosses.contains_key(&id)) {
                    return None;
                }
                Some((self.lexicon.lemma_at(id)?, rank))
            })
            .collect()
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

impl CefrLevels for Pack {
    fn level(&self, lemma: &str) -> Option<CefrLevel> {
        let id = self.lexicon.id_of(lemma)? as usize;
        CefrLevel::from_code(self.levels.get(id).copied()?)
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

/// One `u8` CEFR-level code per lemma id (0 = no level, 1..=6 = A1..=C2). The
/// section length must match the lemma count exactly, like [`parse_freq`].
fn parse_levels(bytes: &[u8], lemma_count: usize) -> Result<Vec<u8>, PackError> {
    if bytes.len() != lemma_count {
        return Err(PackError::Malformed(section::LEVELS));
    }
    Ok(bytes.to_vec())
}

/// Decompresses a gloss blob and reads its index. Decompressed layout:
/// `count u32 | count*(id u32, off u32, len u32) | utf8 bytes`, offsets
/// relative to the start of the utf8 payload. The lemma glosses
/// ([`section::GLOSS_ZST`]) and the expression glosses
/// ([`section::EXPR_ZST`]) share it; `name` is the section a malformed blob
/// is reported under, so the error names the table actually at fault.
fn parse_glosses(zst: &[u8], name: &'static str) -> Result<BTreeMap<u64, String>, PackError> {
    let raw = zstd_decode(zst)?;
    let bad = || PackError::Malformed(name);
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
    use crate::packs::format::{read_container, write_container};

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

    // Builds the two sections of an expression table from (key, gloss) pairs:
    // the key FST, byte-wise sorted as `fst::MapBuilder` demands, and the
    // matching gloss blob — ids dense from 0, in that sorted order, which is
    // what the builder emits.
    fn build_expr_sections(entries: &[(&str, &str)]) -> (Vec<u8>, Vec<u8>) {
        let mut sorted = entries.to_vec();
        sorted.sort_unstable();
        let mut keys = fst::MapBuilder::memory();
        for (id, (key, _)) in sorted.iter().enumerate() {
            keys.insert(key, id as u64).expect("sorted, unique keys");
        }
        let glosses: Vec<(u32, &str)> = sorted
            .iter()
            .enumerate()
            .map(|(id, (_, gloss))| (id as u32, *gloss))
            .collect();
        (keys.into_inner().expect("fst"), build_gloss_zst(&glosses))
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
    fn rank_band_enumeration_lists_ranked_lemmas_with_glosses_in_id_order() {
        let pack = Pack::load(&sample_pack_bytes(ANALYZER_VERSION)).expect("load");
        // run (500) and city (1,200) are in [1, 2000]; seldom (unranked) is
        // skipped. Lemma ids follow the FST's sorted order (city < run < seldom).
        let band = pack.lemmas_in_rank_band(1, 2_000);
        assert_eq!(band, vec![("city", Some("ville")), ("run", Some("courir"))]);
        // A tighter band excludes run (rank 500 < 600).
        assert_eq!(
            pack.lemmas_in_rank_band(600, 2_000),
            vec![("city", Some("ville"))]
        );
        // Pack reports no CEFR levels yet (format slice pending).
        assert_eq!(pack.level("run"), None);
    }

    #[test]
    fn dictionary_words_are_the_ranked_lemmas_with_a_gloss_or_a_level() {
        use crate::knowledge::level::CefrLevel;
        let (forms, pool) =
            build_lexicon_blobs(&[], &["nuance", "run", "seldom", "www"]).expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let id = |lemma: &str| lex.id_of(lemma).unwrap() as usize;
        let mut freq = vec![0u32; lex.lemma_count()];
        freq[id("run")] = 500;
        freq[id("nuance")] = 4_000;
        freq[id("www")] = 900; // ranked, but neither glossed nor levelled
        // `seldom` is levelled but unranked.
        let mut levels = vec![0u8; lex.lemma_count()];
        levels[id("nuance")] = CefrLevel::B2.to_code();
        levels[id("seldom")] = CefrLevel::C1.to_code();
        let freq_bytes: Vec<u8> = freq.iter().flat_map(|r| r.to_le_bytes()).collect();
        let gloss = build_gloss_zst(&[(id("run") as u32, "courir")]);
        let bytes = write_container(
            &meta_json(ANALYZER_VERSION),
            &[
                (section::FORMS, &forms),
                (section::LEMMAS, pool.as_bytes()),
                (section::FREQ, &freq_bytes),
                (section::LEVELS, &levels),
                (section::GLOSS_ZST, &gloss),
                (section::NOTICE, b"AGID. wordfreq. kaikki."),
            ],
        );
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(
            pack.dictionary_words(),
            vec![("nuance", 4_000), ("run", 500)]
        );
    }

    #[test]
    fn a_pack_with_a_levels_section_exposes_and_enumerates_levels() {
        use crate::knowledge::level::{CefrLevel, CefrLevels};
        let (forms, pool) = build_lexicon_blobs(
            &[("running", "run"), ("cities", "city")],
            &["run", "city", "seldom"],
        )
        .expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let freq_bytes = vec![0u8; lex.lemma_count() * 4]; // ranks irrelevant here
        let mut levels = vec![0u8; lex.lemma_count()];
        levels[lex.id_of("city").unwrap() as usize] = CefrLevel::A1.to_code();
        levels[lex.id_of("run").unwrap() as usize] = CefrLevel::A2.to_code();
        levels[lex.id_of("seldom").unwrap() as usize] = CefrLevel::B2.to_code();
        let gloss = build_gloss_zst(&[(lex.id_of("run").unwrap() as u32, "courir")]);
        let bytes = write_container(
            &meta_json(ANALYZER_VERSION),
            &[
                (section::FORMS, &forms),
                (section::LEMMAS, pool.as_bytes()),
                (section::FREQ, &freq_bytes),
                (section::LEVELS, &levels),
                (section::GLOSS_ZST, &gloss),
                (section::NOTICE, b"x"),
            ],
        );
        let pack = Pack::load(&bytes).expect("load");
        assert!(pack.has_levels());
        assert_eq!(pack.level("city"), Some(CefrLevel::A1));
        assert_eq!(pack.level("run"), Some(CefrLevel::A2));
        assert_eq!(pack.level("seldom"), Some(CefrLevel::B2));
        assert_eq!(pack.level("absent"), None);
        // Enumeration is in ascending lemma-id order, with glosses when present.
        assert_eq!(pack.lemmas_at_level(CefrLevel::A1), vec![("city", None)]);
        assert_eq!(
            pack.lemmas_at_level(CefrLevel::A2),
            vec![("run", Some("courir"))]
        );
        assert_eq!(
            pack.lemmas_at_level(CefrLevel::C2),
            Vec::<(&str, Option<&str>)>::new()
        );
    }

    #[test]
    fn a_levels_section_of_the_wrong_length_is_rejected() {
        let (forms, pool) = build_lexicon_blobs(&[("run", "run")], &[]).unwrap();
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let freq_bytes = vec![0u8; lex.lemma_count() * 4];
        let bad_levels = vec![1u8; lex.lemma_count() + 3]; // wrong length
        let bytes = write_container(
            &meta_json(ANALYZER_VERSION),
            &[
                (section::FORMS, &forms),
                (section::LEMMAS, pool.as_bytes()),
                (section::FREQ, &freq_bytes),
                (section::LEVELS, &bad_levels),
            ],
        );
        assert!(matches!(
            Pack::load(&bytes),
            Err(PackError::Malformed(section::LEVELS))
        ));
    }

    #[test]
    fn a_pack_with_an_expression_table_answers_its_keys() {
        let (forms, pool) =
            build_lexicon_blobs(&[], &["give", "up", "start", "point"]).expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let freq_bytes = vec![0u8; lex.lemma_count() * 4]; // ranks irrelevant here
        let (keys, glosses) = build_expr_sections(&[
            ("give up", "Abandonner"),
            ("start point", "Point de départ"),
        ]);
        let bytes = write_container(
            &meta_json(ANALYZER_VERSION),
            &[
                (section::FORMS, &forms),
                (section::LEMMAS, pool.as_bytes()),
                (section::FREQ, &freq_bytes),
                (section::EXPR, &keys),
                (section::EXPR_ZST, &glosses),
                (section::NOTICE, b"kaikki: CC BY-SA."),
            ],
        );
        let pack = Pack::load(&bytes).expect("load");
        assert!(pack.has_expressions());
        assert_eq!(pack.expression("give up"), Some("Abandonner"));
        assert_eq!(pack.expression("start point"), Some("Point de départ"));
        // A key the table does not hold, and one of its words on its own.
        assert_eq!(pack.expression("gave up"), None);
        assert_eq!(pack.expression("give"), None);
    }

    #[test]
    fn a_pack_without_the_expression_sections_answers_no_expression() {
        let pack = Pack::load(&sample_pack_bytes(ANALYZER_VERSION)).expect("load");
        assert!(!pack.has_expressions());
        assert_eq!(pack.expression("give up"), None);
        // The tables it does carry are untouched by their absence.
        assert_eq!(pack.gloss("run"), Some("courir"));
    }

    #[test]
    fn a_pack_carrying_an_unknown_section_loads_unchanged() {
        // The additive contract the expression table rests on: the reader takes
        // the sections it knows by name and ignores the rest, so a pack gaining
        // a table loads on a core built before that table existed.
        let original = sample_pack_bytes(ANALYZER_VERSION);
        let (meta, sections) = read_container(&original).expect("read");
        let mut with_extra: Vec<(&str, &[u8])> = sections
            .iter()
            .map(|s| (s.name.as_str(), s.data.as_slice()))
            .collect();
        with_extra.push(("table.from.the.future", b"\x00\x01\x02"));
        let pack = Pack::load(&write_container(&meta, &with_extra)).expect("load");
        assert_eq!(pack.gloss("run"), Some("courir"));
        assert_eq!(pack.gloss("city"), Some("ville"));
        assert_eq!(pack.rank("city"), Some(1_200));
        assert!(pack.notice().contains("CC BY-SA"));
        assert!(!pack.has_expressions());
    }

    #[test]
    fn a_malformed_expression_table_is_refused() {
        let (forms, pool) = build_lexicon_blobs(&[], &["give", "up"]).expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let freq_bytes = vec![0u8; lex.lemma_count() * 4];
        let (keys, glosses) = build_expr_sections(&[("give up", "Abandonner")]);
        let pack_with = |keys: &[u8], glosses: &[u8]| {
            write_container(
                &meta_json(ANALYZER_VERSION),
                &[
                    (section::FORMS, &forms),
                    (section::LEMMAS, pool.as_bytes()),
                    (section::FREQ, &freq_bytes),
                    (section::EXPR, keys),
                    (section::EXPR_ZST, glosses),
                ],
            )
        };
        // A key index that is not an FST at all.
        assert!(matches!(
            Pack::load(&pack_with(b"not an fst", &glosses)),
            Err(PackError::Malformed(section::EXPR))
        ));
        // A blob announcing an entry it does not carry — reported under its own
        // section name, not the lemma glosses' one.
        let truncated = zstd::encode_all(&1u32.to_le_bytes()[..], 19).expect("zstd encode");
        assert!(matches!(
            Pack::load(&pack_with(&keys, &truncated)),
            Err(PackError::Malformed(section::EXPR_ZST))
        ));
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
