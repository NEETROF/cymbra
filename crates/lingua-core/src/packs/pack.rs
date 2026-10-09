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

use crate::analysis::language::StudiedLanguage;
use crate::analysis::lexicon::{FstLexicon, Lexicon, LexiconError};
use crate::knowledge::level::{CefrLevel, CefrLevels};
use crate::knowledge::profile::{LanguagePair, NativeLanguage};
use crate::knowledge::state::FrequencyRanks;

use super::format::{FormatError, Section, read_container};
use super::grammar::{
    IndexedBlob, ParadigmEntry, Tag, decode_paradigm, decode_runs, decode_tag_pool,
};
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
    /// forms, lowercase, joined by single spaces; for French, the pieces French's
    /// analysis reads in the headword, its determiners as written
    /// (add-lingua-french-expression-keys) — to an expression id.
    /// Optional and additive, like [`LEVELS`]: written only when the pair's
    /// sources hold expressions, ignored by a core that does not read it.
    pub const EXPR: &str = "expr";
    /// zstd-compressed, offset-indexed expression glosses, in the layout
    /// [`GLOSS_ZST`] uses, keyed by the id [`EXPR`] maps to.
    pub const EXPR_ZST: &str = "expr.zst";
    /// zstd-compressed, offset-indexed expression names, in the layout
    /// [`GLOSS_ZST`] uses, keyed by the id [`EXPR`] maps to: the headword as the
    /// dictionary writes it (`au revoir`), for the expressions whose key differs
    /// from it (`à le revoir`) — a key that is its own headword has none
    /// (add-lingua-french-expression-keys D3). Written for a pack studying French
    /// only, when one name differs. Optional and additive, like [`EXPR`]: a core
    /// that predates it ignores it and reports keys, as English and Spanish do.
    pub const EXPR_NAMES_ZST: &str = "expr.names.zst";
    /// The grammar tag pool: one Universal Dependencies tag per line, id =
    /// line index (`add-lingua-word-grammar`). Optional and additive, like
    /// [`LEVELS`]; the two grammar blobs point into it.
    pub const TAGS: &str = "tags";
    /// zstd-compressed paradigms, keyed by lemma id: each dictionary form's
    /// readings (a form and its tag) and the other dictionary forms a form the
    /// analysis files under it is also a reading of.
    pub const PARADIGMS_ZST: &str = "paradigms.zst";
    /// zstd-compressed gloss runs, keyed by lemma id: the tag of each run of
    /// consecutive senses of the lemma's gloss, in gloss order.
    pub const SENSES_ZST: &str = "senses.zst";
    /// The pack's dictionary words, when they are not the lemmas it glosses
    /// (add-lingua-pack-lexical-layer D1): one bit per lemma id, least
    /// significant bit first, `ceil(lemma count / 8)` bytes. Optional and
    /// additive, like [`LEVELS`]: a pack without it reads its glossed lemmas as
    /// its dictionary words, and a core that predates it ignores it and does
    /// the same.
    pub const LEXICAL: &str = "lexical";
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
    /// The pack was built for another analyser generation of its language.
    IncompatibleAnalyzer { pack: String, core: String },
    /// The pack studies a language this core has no analyser for.
    UnknownLanguage(String),
    /// The pack is glossed in a native language this core does not know
    /// (generalise-lingua-native-language D1).
    UnknownNative(String),
    /// The pack is glossed in the language it studies.
    NativeStudied(StudiedLanguage),
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
            PackError::UnknownLanguage(tag) => write!(
                f,
                "pack studies {tag:?}, a language this core cannot analyse; refusing to load"
            ),
            PackError::UnknownNative(tag) => write!(
                f,
                "pack is glossed in {tag:?}, a native language this core does not know; refusing to load"
            ),
            PackError::NativeStudied(language) => write!(
                f,
                "pack studies {:?} and is glossed in it too; refusing to load",
                language.tag()
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
    /// The pair the pack serves, from `meta.studied` and `meta.native`: the
    /// studied language's analyser is the one every analysis of this pack
    /// runs, and its glosses are written in the native language.
    pair: LanguagePair,
    lexicon: FstLexicon<Vec<u8>>,
    /// Rank per lemma id (0 = unranked).
    freq: Vec<u32>,
    /// CEFR level code per lemma id (0 = no level, 1..=6 = A1..=C2). Empty when
    /// the pack carries no level table.
    levels: Vec<u8>,
    /// Gloss per lemma id, for the lemmas that carry one.
    glosses: BTreeMap<u64, String>,
    /// The lexical table: one bit per lemma id, set for a dictionary word.
    /// `None` when the pack carries none, and its glossed lemmas stand in.
    lexical: Option<Vec<u8>>,
    /// Expression key → expression id. `None` when the pack carries no
    /// expression table.
    expressions: Option<fst::Map<Vec<u8>>>,
    /// Gloss per expression id, in the same layout as `glosses`. Empty when
    /// the pack carries no expression table.
    expression_glosses: BTreeMap<u64, String>,
    /// Name per expression id, in the same layout as `glosses`, for the
    /// expressions named otherwise than their key. Empty when the pack carries
    /// no names.
    expression_names: BTreeMap<u64, String>,
    /// The grammar tables, when the pack carries them.
    grammar: Grammar,
    notice: String,
}

/// The three grammar sections, read leniently: an absent or unreadable
/// section answers nothing rather than refusing the pack.
#[derive(Default)]
struct Grammar {
    /// Tag pool, `None` where a line is outside the vocabulary this core knows.
    tags: Vec<Option<Tag>>,
    paradigms: Option<IndexedBlob>,
    senses: Option<IndexedBlob>,
}

impl Pack {
    /// Loads a pack from container bytes (an `include_bytes!`-compatible
    /// slice), refusing one whose studied language this core cannot analyse,
    /// one built for another analyser generation of that language, or one
    /// glossed in a native language this core does not know or in the language
    /// it studies, so no partial analysis is ever produced.
    pub fn load(bytes: &[u8]) -> Result<Self, PackError> {
        let (meta, pair, sections) = read_meta(bytes)?;

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
        let lexical = match sections.iter().find(|s| s.name == section::LEXICAL) {
            Some(s) => Some(parse_lexical(&s.data, lexicon.lemma_count())?),
            None => None,
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
        let expression_names = match sections.iter().find(|s| s.name == section::EXPR_NAMES_ZST) {
            Some(s) => parse_glosses(&s.data, section::EXPR_NAMES_ZST)?,
            None => BTreeMap::new(),
        };
        let notice = sections
            .iter()
            .find(|s| s.name == section::NOTICE)
            .map(|s| String::from_utf8_lossy(&s.data).into_owned())
            .unwrap_or_default();
        let grammar = parse_grammar(&sections)?;

        Ok(Self {
            meta,
            pair,
            lexicon,
            freq,
            levels,
            glosses,
            lexical,
            expressions,
            expression_glosses,
            expression_names,
            grammar,
            notice,
        })
    }

    /// The language a pack's bytes are for, read from its metadata alone
    /// (add-lingua-agent-languages D1): refused for the reasons `load` refuses
    /// it — the envelope, the metadata, a language this core cannot analyse,
    /// another analyser generation — without building the lexicon or reading a
    /// section, so a surface holding several packs loads only the one it needs.
    pub fn studied_in(bytes: &[u8]) -> Result<StudiedLanguage, PackError> {
        read_meta(bytes).map(|(_, pair, _)| pair.studied)
    }

    /// The pair a pack's bytes serve, read from its metadata alone, as
    /// [`Pack::studied_in`] reads its language and refused for the same reasons
    /// `load` refuses it (generalise-lingua-native-language D1).
    pub fn pair_in(bytes: &[u8]) -> Result<LanguagePair, PackError> {
        read_meta(bytes).map(|(_, pair, _)| pair)
    }

    /// The pack's metadata.
    pub fn meta(&self) -> &PackMeta {
        &self.meta
    }

    /// The language the pack is for (`meta.studied`, checked at load).
    pub fn studied(&self) -> StudiedLanguage {
        self.pair.studied
    }

    /// The language the pack's glosses are written in (`meta.native`, checked
    /// at load).
    pub fn native(&self) -> NativeLanguage {
        self.pair.native
    }

    /// The pair the pack serves: its studied and its native language.
    pub fn pair(&self) -> LanguagePair {
        self.pair
    }

    /// The form→lemma lexicon (feeds the lemmatisation cascade).
    pub fn lexicon(&self) -> &(impl Lexicon + '_) {
        &self.lexicon
    }

    /// The native-language gloss of the dictionary form `lemma`, if the pack
    /// carries one: its own, found among the pack's lemmas. A string the pack
    /// holds as no lemma — a form of another word (`saw`, of *see*) or a word it
    /// does not hold — has none, never the gloss of the word its spelling is a
    /// form of (fix-lingua-lemma-lookup D2).
    pub fn gloss(&self, lemma: &str) -> Option<&str> {
        let id = self.lexicon.lemma_id(lemma)?;
        self.glosses.get(&id).map(String::as_str)
    }

    /// The native-language gloss of a multi-word expression, keyed by its
    /// dictionary form: the words' lemmas, lowercase, joined by single spaces
    /// (`starting point` is looked up as `start point`); for a pack studying
    /// French, the key [`french_expression_key`](crate::engine::french_expression_key)
    /// makes of its headword (`au revoir` is looked up as `à le revoir`). `None`
    /// when the pack carries no expression table, or holds no such key.
    pub fn expression(&self, key: &str) -> Option<&str> {
        let id = self.expressions.as_ref()?.get(key.as_bytes())?;
        self.expression_glosses.get(&id).map(String::as_str)
    }

    /// The name of the expression filed under `key` — its headword as the
    /// dictionary writes it (`au revoir` for `à le revoir`) — when the pack names
    /// it otherwise than its key; `None` when the key is its own name, the pack
    /// holds no such key, or carries no names (add-lingua-french-expression-keys
    /// D3).
    pub fn expression_name(&self, key: &str) -> Option<&str> {
        let id = self.expressions.as_ref()?.get(key.as_bytes())?;
        self.expression_names.get(&id).map(String::as_str)
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

    /// Whether the pack carries grammar tables.
    pub fn has_grammar(&self) -> bool {
        self.grammar.paradigms.is_some() || self.grammar.senses.is_some()
    }

    /// The paradigm filed under a dictionary form, found among the pack's
    /// lemmas, decoded; empty when the pack has none for it, or holds no such
    /// lemma — a form of another word reads no other word's paradigm
    /// (fix-lingua-lemma-lookup D2).
    fn paradigm(&self, lemma: &str) -> Vec<ParadigmEntry> {
        let Some(blob) = &self.grammar.paradigms else {
            return Vec::new();
        };
        self.lexicon
            .lemma_id(lemma)
            .and_then(|id| blob.get(u32::try_from(id).ok()?))
            .and_then(decode_paradigm)
            .unwrap_or_default()
    }

    fn tag(&self, id: u16) -> Option<&Tag> {
        self.grammar.tags.get(id as usize)?.as_ref()
    }

    /// The readings of `form` (lowercase) as the dictionary form `lemma`, in
    /// the pack's order. Empty when the pack carries no grammar, holds no
    /// such form, holds `lemma` as no lemma of its own, or reads none of its
    /// tags.
    pub fn readings(&self, lemma: &str, form: &str) -> Vec<Tag> {
        self.paradigm(lemma)
            .into_iter()
            .filter_map(|entry| match entry {
                ParadigmEntry::Reading { form: edit, tag } if edit.apply(lemma) == form => {
                    self.tag(tag).cloned()
                }
                _ => None,
            })
            .collect()
    }

    /// The other dictionary forms `form` (lowercase) is also a reading of, as
    /// the pack files them under `lemma` — the dictionary form the analysis
    /// reads `form` as — each with its readings of `form`. A dictionary form
    /// whose readings this core cannot name is left out.
    pub fn other_readings(&self, lemma: &str, form: &str) -> Vec<(String, Vec<Tag>)> {
        self.paradigm(lemma)
            .into_iter()
            .filter_map(|entry| match entry {
                ParadigmEntry::Also { form: edit, other } if edit.apply(lemma) == form => {
                    let other = self.lexicon.lemma_at(other as u64)?.to_owned();
                    let readings = self.readings(&other, form);
                    (!readings.is_empty()).then_some((other, readings))
                }
                _ => None,
            })
            .collect()
    }

    /// The runs of a dictionary form's gloss: consecutive senses sharing a
    /// part of speech, each with its tag (`None` when this core cannot read
    /// it) and the number of senses it covers. Empty when the pack carries
    /// no runs for the word, or holds it as no lemma of its own: the runs
    /// follow [`Pack::gloss`] (fix-lingua-lemma-lookup D2).
    pub fn sense_runs(&self, lemma: &str) -> Vec<(Option<&Tag>, usize)> {
        let Some(blob) = &self.grammar.senses else {
            return Vec::new();
        };
        self.lexicon
            .lemma_id(lemma)
            .and_then(|id| blob.get(u32::try_from(id).ok()?))
            .and_then(decode_runs)
            .unwrap_or_default()
            .into_iter()
            .map(|run| (self.tag(run.tag), run.count as usize))
            .collect()
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

    /// Whether those levels are estimated from word frequency rather than taken
    /// from a CEFR list (add-lingua-spanish-levels): the pack's metadata says so.
    pub fn levels_estimated(&self) -> bool {
        self.has_levels() && self.meta.levels_estimated
    }

    /// The pack's ranked dictionary words with their frequency rank: the ranked lemmas
    /// that are dictionary words of the pack ([`Pack::is_dictionary_word`]) or carry a
    /// CEFR level, in ascending lemma-id order. A ranked token that is neither is mostly
    /// a name or noise ("london", "www"), which a vocabulary size does not count. Feeds
    /// the estimated vocabulary size and a CEFR list's typical vocabularies, so neither
    /// depends on the native language the pack is glossed in.
    pub fn dictionary_words(&self) -> Vec<(&str, u32)> {
        (0..self.lexicon.lemma_count() as u64)
            .filter_map(|id| {
                let rank = *self.freq.get(id as usize)?;
                let leveled = self.levels.get(id as usize).is_some_and(|&code| code != 0);
                if rank == 0 || !(leveled || self.is_dictionary_id(id)) {
                    return None;
                }
                Some((self.lexicon.lemma_at(id)?, rank))
            })
            .collect()
    }

    /// Whether `lemma` is one of the pack's dictionary words — a word of its studied
    /// language rather than a name or noise (add-lingua-pack-lexical-layer D2). The
    /// lexical table says so when the pack carries one; a pack without one reads its
    /// glossed lemmas as its dictionary words, which for a lemma of the pack is exactly
    /// `self.gloss(lemma).is_some()`. A string the lexicon does not hold is none.
    ///
    /// An estimate, like [`FrequencyRanks::rank`] and [`CefrLevels::level`]: it reads
    /// through the spelling, for the lemma the pack's forms read `lemma` as, so that a
    /// string the analysis returns as a form of a dictionary word is no name
    /// (fix-lingua-lemma-lookup D3). Every lemma of a pack the builder writes reads as
    /// itself, so for a lemma it is that lemma's own mark.
    pub fn is_dictionary_word(&self, lemma: &str) -> bool {
        self.lexicon
            .id_of(lemma)
            .is_some_and(|id| self.is_dictionary_id(id))
    }

    /// [`Pack::is_dictionary_word`], by lemma id.
    fn is_dictionary_id(&self, id: u64) -> bool {
        match &self.lexical {
            Some(bits) => bits
                .get((id / 8) as usize)
                .is_some_and(|byte| byte & (1 << (id % 8)) != 0),
            None => self.glosses.contains_key(&id),
        }
    }
}

impl FrequencyRanks for Pack {
    /// The frequency rank of `lemma`, an estimate: read through the spelling, for the
    /// lemma the pack's forms read `lemma` as (fix-lingua-lemma-lookup D3) — a reader
    /// who knows *strange* is estimated to know `strangers`, which the analysis reads
    /// as `stranger`, a form of *strange*. A lemma of a pack the builder writes reads as
    /// itself, so its rank is its own.
    fn rank(&self, lemma: &str) -> Option<u32> {
        let id = self.lexicon.id_of(lemma)? as usize;
        match self.freq.get(id).copied() {
            Some(0) | None => None,
            Some(rank) => Some(rank),
        }
    }
}

impl CefrLevels for Pack {
    /// The CEFR level of `lemma`, an estimate read through the spelling like
    /// [`FrequencyRanks::rank`] (fix-lingua-lemma-lookup D3).
    fn level(&self, lemma: &str) -> Option<CefrLevel> {
        let id = self.lexicon.id_of(lemma)? as usize;
        CefrLevel::from_code(self.levels.get(id).copied()?)
    }
}

/// A pack is compatible when its analyser version matches the running core's
/// analyser for the pack's language. (Exact match for now — the version is
/// bumped on any behavioural change, so mixing generations could shift counts;
/// a looser policy can come later.)
fn analyzer_compatible(studied: StudiedLanguage, pack_version: &str) -> bool {
    pack_version == studied.analyzer_version()
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

/// One bit per lemma id, least significant bit first (add-lingua-pack-lexical-layer D1).
/// The section length must be exactly `ceil(lemma_count / 8)`, like [`parse_levels`], so a
/// table built for another lemma pool is refused rather than misread.
fn parse_lexical(bytes: &[u8], lemma_count: usize) -> Result<Vec<u8>, PackError> {
    if bytes.len() != lemma_count.div_ceil(8) {
        return Err(PackError::Malformed(section::LEXICAL));
    }
    Ok(bytes.to_vec())
}

/// Decompresses a gloss blob and reads its index. Decompressed layout:
/// `count u32 | count*(id u32, off u32, len u32) | utf8 bytes`, offsets
/// relative to the start of the utf8 payload. The lemma glosses
/// ([`section::GLOSS_ZST`]) and the expression glosses
/// ([`section::EXPR_ZST`]) share it, and so do the expression names
/// ([`section::EXPR_NAMES_ZST`]); `name` is the section a malformed blob
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

/// Reads the grammar sections. They are additive: each one is optional, and
/// a pack carrying a blob this core cannot index is refused like any other
/// malformed section, so a corrupt pack never answers half its grammar.
fn parse_grammar(sections: &[super::format::Section]) -> Result<Grammar, PackError> {
    let find = |name: &str| sections.iter().find(|s| s.name == name);
    let tags = match find(section::TAGS) {
        Some(s) => decode_tag_pool(&s.data).ok_or(PackError::Malformed(section::TAGS))?,
        None => Vec::new(),
    };
    let blob = |name: &'static str| -> Result<Option<IndexedBlob>, PackError> {
        match find(name) {
            Some(s) => IndexedBlob::parse(zstd_decode(&s.data)?)
                .map(Some)
                .ok_or(PackError::Malformed(name)),
            None => Ok(None),
        }
    };
    Ok(Grammar {
        tags,
        paradigms: blob(section::PARADIGMS_ZST)?,
        senses: blob(section::SENSES_ZST)?,
    })
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

/// A pack's metadata and studied language, checked as `load` checks them, with
/// its sections still unread.
fn read_meta(bytes: &[u8]) -> Result<(PackMeta, LanguagePair, Vec<Section>), PackError> {
    let (meta_json, sections) = read_container(bytes).map_err(PackError::Format)?;
    let meta: PackMeta = serde_json::from_slice(&meta_json).map_err(PackError::BadMeta)?;
    let studied = StudiedLanguage::from_tag(&meta.studied)
        .ok_or_else(|| PackError::UnknownLanguage(meta.studied.clone()))?;
    if !analyzer_compatible(studied, &meta.analyzer_version) {
        return Err(PackError::IncompatibleAnalyzer {
            pack: meta.analyzer_version.clone(),
            core: studied.analyzer_version().to_owned(),
        });
    }
    // After the studied language and the analyser, so their errors keep their precedence
    // (generalise-lingua-native-language D1).
    let native = NativeLanguage::from_tag(&meta.native)
        .ok_or_else(|| PackError::UnknownNative(meta.native.clone()))?;
    if native.studied() == Some(studied) {
        return Err(PackError::NativeStudied(studied));
    }
    Ok((meta, LanguagePair { studied, native }, sections))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::analysis::lexicon::build_lexicon_blobs;
    use crate::analysis::{ANALYZER_VERSION, FRENCH_ANALYZER_VERSION, SPANISH_ANALYZER_VERSION};
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
        meta_json_for("en", analyzer)
    }

    fn meta_json_for(studied: &str, analyzer: &str) -> Vec<u8> {
        meta_json_with(studied, "fr", analyzer)
    }

    fn meta_json_with(studied: &str, native: &str, analyzer: &str) -> Vec<u8> {
        serde_json::to_vec(&PackMeta {
            studied: studied.into(),
            native: native.into(),
            pack_version: "2026.09.1".into(),
            analyzer_version: analyzer.into(),
            levels_estimated: false,
            licences: vec![
                "AGID".into(),
                "wordfreq CC BY-SA".into(),
                "kaikki CC BY-SA".into(),
            ],
        })
        .unwrap()
    }

    // Assembles a small but complete en-fr pack for the current analyzer.
    fn sample_pack_bytes(analyzer: &str) -> Vec<u8> {
        sample_pack_bytes_for("en", analyzer)
    }

    // The same pack, for another studied language.
    pub(crate) fn sample_pack_bytes_for(studied: &str, analyzer: &str) -> Vec<u8> {
        sample_pack_bytes_with(studied, "fr", analyzer)
    }

    // The same pack, for another pair.
    pub(crate) fn sample_pack_bytes_with(studied: &str, native: &str, analyzer: &str) -> Vec<u8> {
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
            &meta_json_with(studied, native, analyzer),
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
    fn spec_scenario_loading_the_en_fr_pack_names_english() {
        let pack = Pack::load(&sample_pack_bytes(ANALYZER_VERSION)).expect("load");
        assert_eq!(pack.studied(), StudiedLanguage::English);
        assert_eq!(pack.meta().analyzer_version, "1.1.0");
    }

    #[test]
    fn spec_scenario_the_en_fr_pack_names_french() {
        let bytes = sample_pack_bytes(ANALYZER_VERSION);
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(pack.native(), NativeLanguage::French);
        assert_eq!(pack.pair().key(), "en-fr");
        assert_eq!(
            pack.pair(),
            LanguagePair {
                studied: StudiedLanguage::English,
                native: NativeLanguage::French
            }
        );
        // Its metadata alone says the same.
        assert_eq!(Pack::pair_in(&bytes).unwrap(), pack.pair());
        // Another pair is data, not code.
        let es_en = Pack::load(&sample_pack_bytes_with(
            "es",
            "en",
            SPANISH_ANALYZER_VERSION,
        ))
        .expect("an es-en pack loads");
        assert_eq!(es_en.native(), NativeLanguage::English);
        assert_eq!(es_en.pair().key(), "es-en");
    }

    #[test]
    fn spec_scenario_a_native_language_the_core_does_not_know() {
        for bytes in [
            sample_pack_bytes_with("en", "de", ANALYZER_VERSION),
            sample_pack_bytes_with("en", "FR", ANALYZER_VERSION),
        ] {
            match Pack::load(&bytes) {
                Err(err @ PackError::UnknownNative(_)) => {
                    assert!(err.to_string().contains("refusing to load"), "{err}");
                }
                Err(other) => panic!("expected UnknownNative, got {other}"),
                Ok(_) => panic!("a pack glossed in an unknown language loaded"),
            }
            assert!(matches!(
                Pack::pair_in(&bytes),
                Err(PackError::UnknownNative(_))
            ));
        }
        let Err(err) = Pack::load(&sample_pack_bytes_with("en", "de", ANALYZER_VERSION)) else {
            panic!("refused");
        };
        assert!(matches!(&err, PackError::UnknownNative(tag) if tag == "de"));
        assert!(err.to_string().contains("\"de\""), "{err}");
    }

    #[test]
    fn spec_scenario_a_pack_glossed_in_the_language_it_studies() {
        for (studied, analyzer, language) in [
            ("en", ANALYZER_VERSION, StudiedLanguage::English),
            ("es", SPANISH_ANALYZER_VERSION, StudiedLanguage::Spanish),
            ("fr", FRENCH_ANALYZER_VERSION, StudiedLanguage::French),
        ] {
            let bytes = sample_pack_bytes_with(studied, studied, analyzer);
            match Pack::load(&bytes) {
                Err(err @ PackError::NativeStudied(_)) => {
                    assert!(matches!(err, PackError::NativeStudied(l) if l == language));
                    assert!(err.to_string().contains(&format!("{studied:?}")), "{err}");
                }
                Err(other) => panic!("expected NativeStudied, got {other}"),
                Ok(_) => panic!("a pack glossed in the language it studies loaded"),
            }
            assert!(matches!(
                Pack::pair_in(&bytes),
                Err(PackError::NativeStudied(_))
            ));
        }
    }

    #[test]
    fn the_studied_language_and_the_analyser_are_checked_before_the_native_language() {
        // An unknown studied language wins over an unknown native language.
        assert!(matches!(
            Pack::load(&sample_pack_bytes_with("pt", "de", ANALYZER_VERSION)),
            Err(PackError::UnknownLanguage(tag)) if tag == "pt"
        ));
        // Another analyser generation wins over a native language equal to the studied one.
        assert!(matches!(
            Pack::load(&sample_pack_bytes_with(
                "en",
                "en",
                "0.0.0-from-another-era"
            )),
            Err(PackError::IncompatibleAnalyzer { .. })
        ));
        assert!(matches!(
            Pack::pair_in(&sample_pack_bytes_with("es", "de", "0.1.0")),
            Err(PackError::IncompatibleAnalyzer { .. })
        ));
        // Metadata that does not parse is still refused as such.
        let bad = write_container(b"{not json", &[(section::NOTICE, b"x")]);
        assert!(matches!(Pack::pair_in(&bad), Err(PackError::BadMeta(_))));
    }

    #[test]
    fn spec_scenario_a_pack_for_a_language_the_core_cannot_analyse() {
        match Pack::load(&sample_pack_bytes_for("pt", ANALYZER_VERSION)) {
            Err(err @ PackError::UnknownLanguage(_)) => {
                assert!(matches!(&err, PackError::UnknownLanguage(tag) if tag == "pt"));
                assert!(err.to_string().contains("\"pt\""), "{err}");
            }
            Err(other) => panic!("expected UnknownLanguage, got {other}"),
            Ok(_) => panic!("a pack the core cannot analyse loaded"),
        }
    }

    #[test]
    fn spec_scenario_versions_are_compared_within_a_language() {
        let spanish = Pack::load(&sample_pack_bytes_for("es", SPANISH_ANALYZER_VERSION))
            .expect("a Spanish pack at Spanish's version loads");
        assert_eq!(spanish.studied(), StudiedLanguage::Spanish);
        // A Spanish pack built at the baseline Spanish replaced is refused against Spanish's own
        // version, whatever English's is.
        match Pack::load(&sample_pack_bytes_for("es", "0.1.0")) {
            Err(PackError::IncompatibleAnalyzer { pack, core }) => {
                assert_eq!(pack, "0.1.0");
                assert_eq!(core, SPANISH_ANALYZER_VERSION);
            }
            Err(other) => panic!("expected IncompatibleAnalyzer, got {other}"),
            Ok(_) => panic!("a Spanish pack at the baseline's version loaded"),
        }
        assert!(Pack::load(&sample_pack_bytes_for("en", ANALYZER_VERSION)).is_ok());
    }

    #[test]
    fn spec_scenario_a_french_pack_at_french_s_analyser_version() {
        // add-lingua-french-baseline: a pack studying French loads against French's own version,
        // whatever English's is, and is glossed in English or Spanish. Its own analysis left the
        // baseline's `0.x` versions (add-lingua-french-analysis D5).
        assert!(!FRENCH_ANALYZER_VERSION.starts_with("0."));
        let bytes = sample_pack_bytes_with("fr", "en", FRENCH_ANALYZER_VERSION);
        let french = Pack::load(&bytes).expect("a French pack at French's version loads");
        assert_eq!(french.studied(), StudiedLanguage::French);
        assert_eq!(french.meta().analyzer_version, FRENCH_ANALYZER_VERSION);
        assert_eq!(french.pair().key(), "fr-en");
        assert_eq!(Pack::studied_in(&bytes).unwrap(), StudiedLanguage::French);
        let fr_es = Pack::load(&sample_pack_bytes_with("fr", "es", FRENCH_ANALYZER_VERSION))
            .expect("a fr-es pack loads");
        assert_eq!(fr_es.pair().key(), "fr-es");
        // Compared with French's own version, never another language's: a French pack at
        // Spanish's is refused. English's `1.1.0` is French's own since
        // add-lingua-french-detection-guard, as it was Spanish's once: the numbers may meet.
        assert_ne!(SPANISH_ANALYZER_VERSION, FRENCH_ANALYZER_VERSION);
        match Pack::load(&sample_pack_bytes_with(
            "fr",
            "en",
            SPANISH_ANALYZER_VERSION,
        )) {
            Err(PackError::IncompatibleAnalyzer { pack, core }) => {
                assert_eq!(pack, SPANISH_ANALYZER_VERSION);
                assert_eq!(core, FRENCH_ANALYZER_VERSION);
            }
            Err(other) => panic!("expected IncompatibleAnalyzer, got {other}"),
            Ok(_) => panic!("a French pack at Spanish's version loaded"),
        }
        // A pack built at `1.0.0`, before French's detection guard, at `0.2.0`, the version French
        // had before its own analysis, or at the baseline's `0.1.0`, read French otherwise: each
        // is refused.
        for before in ["1.0.0", "0.2.0", "0.1.0"] {
            match Pack::load(&sample_pack_bytes_with("fr", "en", before)) {
                Err(PackError::IncompatibleAnalyzer { pack, core }) => {
                    assert_eq!(pack, before);
                    assert_eq!(core, FRENCH_ANALYZER_VERSION);
                }
                Err(other) => panic!("expected IncompatibleAnalyzer, got {other}"),
                Ok(_) => panic!("a French pack at {before} loaded"),
            }
        }
        // Portuguese still has no analyser.
        assert!(matches!(
            Pack::load(&sample_pack_bytes_with("pt", "en", FRENCH_ANALYZER_VERSION)),
            Err(PackError::UnknownLanguage(tag)) if tag == "pt"
        ));
    }

    #[test]
    fn spec_loading_the_en_fr_pack_exposes_lemmas_ranks_and_glosses() {
        let pack = Pack::load(&sample_pack_bytes(ANALYZER_VERSION)).expect("load");
        assert_eq!(pack.meta().pair_key(), "en-fr");
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

    /// A pack of four lemmas: `run` (rank 500, glossed), `nuance` (rank 4,000, B2), `www`
    /// (rank 900, neither glossed nor levelled) and `seldom` (C1, unranked). `lexical`, when
    /// given, is the lexical table's bits, set for the lemmas it names; `None` writes no table.
    fn dictionary_pack(lexical: Option<&[&str]>) -> Vec<u8> {
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
        let mut bits = vec![0u8; lex.lemma_count().div_ceil(8)];
        for lemma in lexical.unwrap_or_default() {
            bits[id(lemma) / 8] |= 1 << (id(lemma) % 8);
        }
        let mut sections: Vec<(&str, &[u8])> = vec![
            (section::FORMS, &forms),
            (section::LEMMAS, pool.as_bytes()),
            (section::FREQ, &freq_bytes),
            (section::LEVELS, &levels),
            (section::GLOSS_ZST, &gloss),
            (section::NOTICE, b"AGID. wordfreq. kaikki."),
        ];
        if lexical.is_some() {
            sections.push((section::LEXICAL, &bits));
        }
        write_container(&meta_json(ANALYZER_VERSION), &sections)
    }

    #[test]
    fn dictionary_words_are_the_ranked_lemmas_with_a_gloss_or_a_level() {
        let pack = Pack::load(&dictionary_pack(None)).expect("load");
        assert_eq!(
            pack.dictionary_words(),
            vec![("nuance", 4_000), ("run", 500)]
        );
    }

    #[test]
    fn without_a_lexical_table_the_dictionary_words_are_the_glossed_lemmas() {
        // add-lingua-pack-lexical-layer D2: today's rule, exactly.
        let pack = Pack::load(&dictionary_pack(None)).expect("load");
        for lemma in ["nuance", "run", "seldom", "www", "absent"] {
            assert_eq!(
                pack.is_dictionary_word(lemma),
                pack.gloss(lemma).is_some(),
                "{lemma}"
            );
        }
        assert!(pack.is_dictionary_word("run"));
    }

    #[test]
    fn spec_scenario_a_lexical_table_decides_the_dictionary_words() {
        // The table names `www` and not `run`: the bit decides, not the gloss.
        let pack = Pack::load(&dictionary_pack(Some(&["www"]))).expect("load");
        // A glossed lemma whose bit is off is not a dictionary word; its gloss stays.
        assert!(!pack.is_dictionary_word("run"));
        assert_eq!(pack.gloss("run"), Some("courir"));
        // An unglossed ranked lemma whose bit is on is one.
        assert!(pack.is_dictionary_word("www"));
        assert_eq!(pack.gloss("www"), None);
        // A levelled lemma still counts in a vocabulary size, bit or no bit.
        assert!(!pack.is_dictionary_word("nuance"));
        assert_eq!(
            pack.dictionary_words(),
            vec![("nuance", 4_000), ("www", 900)]
        );
        // A lemma the lexicon does not hold is no dictionary word.
        assert!(!pack.is_dictionary_word("absent"));
        // A table that names every glossed lemma says what the glosses say.
        let same = Pack::load(&dictionary_pack(Some(&["run"]))).expect("load");
        assert_eq!(
            same.dictionary_words(),
            Pack::load(&dictionary_pack(None))
                .unwrap()
                .dictionary_words()
        );
    }

    #[test]
    fn a_lexical_table_of_the_wrong_length_is_refused() {
        let (meta, sections) = read_container(&dictionary_pack(Some(&["run"]))).unwrap();
        for length in [0usize, 2] {
            let bad = vec![0xffu8; length];
            let rewritten: Vec<(&str, &[u8])> = sections
                .iter()
                .map(|s| {
                    let data: &[u8] = if s.name == section::LEXICAL {
                        &bad
                    } else {
                        &s.data
                    };
                    (s.name.as_str(), data)
                })
                .collect();
            assert!(
                matches!(
                    Pack::load(&write_container(&meta, &rewritten)),
                    Err(PackError::Malformed(section::LEXICAL))
                ),
                "{length} bytes"
            );
        }
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

    // — expression names (add-lingua-french-expression-keys D3) —

    /// The name section for a table built by [`build_expr_sections`] from `keys`: each
    /// (key, name) pair filed under the id the key's sorted position gives it.
    fn build_expr_names(keys: &[&str], names: &[(&str, &str)]) -> Vec<u8> {
        let mut sorted = keys.to_vec();
        sorted.sort_unstable();
        let mut entries: Vec<(u32, &str)> = names
            .iter()
            .map(|(key, name)| {
                let id = sorted.binary_search(key).expect("a key of the table");
                (id as u32, *name)
            })
            .collect();
        entries.sort_unstable();
        build_gloss_zst(&entries)
    }

    /// A small fr-en pack holding `à le revoir` (named `au revoir`) and `tout de suite` (its
    /// own name), with the names section when `names` is given.
    fn french_expression_pack(names: Option<&[u8]>) -> Vec<u8> {
        let (forms, pool) = build_lexicon_blobs(&[], &["à", "le", "revoir", "tout", "de", "suite"])
            .expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let freq_bytes = vec![0u8; lex.lemma_count() * 4];
        let (keys, glosses) =
            build_expr_sections(&[("à le revoir", "goodbye"), ("tout de suite", "right away")]);
        let mut sections: Vec<(&str, &[u8])> = vec![
            (section::FORMS, &forms),
            (section::LEMMAS, pool.as_bytes()),
            (section::FREQ, &freq_bytes),
            (section::EXPR, &keys),
            (section::EXPR_ZST, &glosses),
        ];
        if let Some(names) = names {
            sections.push((section::EXPR_NAMES_ZST, names));
        }
        write_container(
            &meta_json_with("fr", "en", FRENCH_ANALYZER_VERSION),
            &sections,
        )
    }

    fn french_names() -> Vec<u8> {
        build_expr_names(
            &["à le revoir", "tout de suite"],
            &[("à le revoir", "au revoir")],
        )
    }

    #[test]
    fn a_pack_without_the_names_section_names_no_expression_and_loads_as_before() {
        let pack = Pack::load(&french_expression_pack(None)).expect("load");
        assert_eq!(pack.expression("à le revoir"), Some("goodbye"));
        assert_eq!(pack.expression_name("à le revoir"), None);
        // Nor does a pack without an expression table at all.
        let bare = Pack::load(&sample_pack_bytes(ANALYZER_VERSION)).expect("load");
        assert_eq!(bare.expression_name("give up"), None);
    }

    #[test]
    fn a_pack_with_the_names_section_names_its_expressions() {
        let pack = Pack::load(&french_expression_pack(Some(&french_names()))).expect("load");
        assert_eq!(pack.expression_name("à le revoir"), Some("au revoir"));
        // A key that is its own name carries none; a name is no key; a key the table does not
        // hold has no name.
        assert_eq!(pack.expression_name("tout de suite"), None);
        assert_eq!(pack.expression("tout de suite"), Some("right away"));
        assert_eq!(pack.expression_name("au revoir"), None);
        assert_eq!(pack.expression("au revoir"), None);
        assert_eq!(pack.expression_name("à le soir"), None);
    }

    #[test]
    fn a_malformed_names_section_is_refused_under_its_own_name() {
        let truncated = zstd::encode_all(&1u32.to_le_bytes()[..], 19).expect("zstd encode");
        assert!(matches!(
            Pack::load(&french_expression_pack(Some(&truncated))),
            Err(PackError::Malformed(section::EXPR_NAMES_ZST))
        ));
    }

    #[test]
    fn a_core_ignoring_the_names_section_reads_the_same_table() {
        // A core that predates the section ignores it, as it ignores any section it does not
        // know (`a_pack_carrying_an_unknown_section_loads_unchanged`): stripped of its names,
        // the pack answers every key with the same gloss — only the names are missing.
        let named = Pack::load(&french_expression_pack(Some(&french_names()))).expect("load");
        let (meta, sections) =
            read_container(&french_expression_pack(Some(&french_names()))).expect("read");
        let unnamed: Vec<(&str, &[u8])> = sections
            .iter()
            .filter(|s| s.name != section::EXPR_NAMES_ZST)
            .map(|s| (s.name.as_str(), s.data.as_slice()))
            .collect();
        let older = Pack::load(&write_container(&meta, &unnamed)).expect("load");
        for key in ["à le revoir", "tout de suite", "au revoir"] {
            assert_eq!(older.expression(key), named.expression(key), "{key}");
            assert_eq!(older.expression_name(key), None, "{key}");
        }
    }

    /// The sample pack plus whatever grammar sections a test names.
    fn sample_pack_with(extra: &[(&str, &[u8])]) -> Vec<u8> {
        let (meta, mut sections) = read_container(&sample_pack_bytes(ANALYZER_VERSION)).unwrap();
        let owned: Vec<(String, Vec<u8>)> = sections
            .drain(..)
            .map(|s| (s.name, s.data))
            .chain(extra.iter().map(|(n, d)| ((*n).to_owned(), d.to_vec())))
            .collect();
        let borrowed: Vec<(&str, &[u8])> = owned
            .iter()
            .map(|(n, d)| (n.as_str(), d.as_slice()))
            .collect();
        write_container(&meta, &borrowed)
    }

    #[test]
    fn a_pack_without_grammar_answers_no_reading_and_no_run() {
        let pack = Pack::load(&sample_pack_bytes(ANALYZER_VERSION)).expect("load");
        assert!(!pack.has_grammar());
        assert!(pack.readings("run", "ran").is_empty());
        assert!(pack.other_readings("run", "ran").is_empty());
        assert!(pack.sense_runs("run").is_empty());
    }

    #[test]
    fn a_pack_with_grammar_reads_readings_and_runs_by_dictionary_form() {
        use super::super::grammar::{
            FormEdit, ParadigmEntry, SenseRun, encode_indexed, encode_paradigm, encode_runs,
            encode_tag_pool,
        };
        let lex = Pack::load(&sample_pack_bytes(ANALYZER_VERSION)).expect("load");
        let run = lex.lexicon.id_of("run").unwrap() as u32;
        let tags = encode_tag_pool(&[
            "VERB".into(),
            "VERB|Mood=Ind|Tense=Past|VerbForm=Fin".into(),
        ]);
        let paradigm = encode_paradigm(&[ParadigmEntry::Reading {
            form: FormEdit::between("run", "ran").unwrap(),
            tag: 1,
        }]);
        let paradigms =
            zstd::encode_all(encode_indexed(&[(run, paradigm)]).as_slice(), 19).unwrap();
        let runs = encode_runs(&[SenseRun { tag: 0, count: 1 }]);
        let senses = zstd::encode_all(encode_indexed(&[(run, runs)]).as_slice(), 19).unwrap();
        let pack = Pack::load(&sample_pack_with(&[
            (section::TAGS, &tags),
            (section::PARADIGMS_ZST, &paradigms),
            (section::SENSES_ZST, &senses),
        ]))
        .expect("load");
        assert!(pack.has_grammar());
        let readings: Vec<String> = pack
            .readings("run", "ran")
            .iter()
            .map(|t| t.to_ud())
            .collect();
        assert_eq!(readings, ["VERB|Mood=Ind|Tense=Past|VerbForm=Fin"]);
        assert!(pack.readings("run", "running").is_empty());
        assert!(
            pack.readings("city", "cities").is_empty(),
            "no paradigm for city"
        );
        let runs: Vec<(Option<String>, usize)> = pack
            .sense_runs("run")
            .into_iter()
            .map(|(t, n)| (t.map(|t| t.to_ud()), n))
            .collect();
        assert_eq!(runs, [(Some("VERB".to_owned()), 1)]);
        assert!(pack.sense_runs("unlisted").is_empty());
    }

    // — a dictionary form is read as itself (fix-lingua-lemma-lookup) —

    const PAST: &str = "VERB|Mood=Ind|Tense=Past|VerbForm=Fin";

    /// A pack whose forms read `saw` and `seen` as *see*, and `are` and `is` as *be*: *see*
    /// ranked 200 and A1, glossed with a verb's sense and a noun's, its runs saying so and its
    /// paradigm naming `saw` its past tense; *be* ranked 2 and glossed. Neither `saw` nor `are`
    /// is a lemma of the pack.
    fn see_pack() -> Vec<u8> {
        use super::super::grammar::{
            FormEdit, ParadigmEntry, SenseRun, encode_indexed, encode_paradigm, encode_runs,
            encode_tag_pool,
        };
        use crate::knowledge::level::CefrLevel;
        let (forms, pool) = build_lexicon_blobs(
            &[("saw", "see"), ("seen", "see"), ("are", "be"), ("is", "be")],
            &[],
        )
        .expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let id = |lemma: &str| lex.lemma_id(lemma).expect("a lemma of the pack") as u32;
        let mut freq = vec![0u32; lex.lemma_count()];
        freq[id("see") as usize] = 200;
        freq[id("be") as usize] = 2;
        let freq_bytes: Vec<u8> = freq.iter().flat_map(|r| r.to_le_bytes()).collect();
        let mut levels = vec![0u8; lex.lemma_count()];
        levels[id("see") as usize] = CefrLevel::A1.to_code();
        let gloss = build_gloss_zst(&[(id("be"), "Être"), (id("see"), "Voir; Siège")]);
        let tags = encode_tag_pool(&["NOUN".into(), "VERB".into(), PAST.into()]);
        let paradigm = encode_paradigm(&[ParadigmEntry::Reading {
            form: FormEdit::between("see", "saw").unwrap(),
            tag: 2,
        }]);
        let paradigms =
            zstd::encode_all(encode_indexed(&[(id("see"), paradigm)]).as_slice(), 19).unwrap();
        let runs = encode_runs(&[SenseRun { tag: 1, count: 1 }, SenseRun { tag: 0, count: 1 }]);
        let senses = zstd::encode_all(encode_indexed(&[(id("see"), runs)]).as_slice(), 19).unwrap();
        write_container(
            &meta_json(ANALYZER_VERSION),
            &[
                (section::FORMS, &forms),
                (section::LEMMAS, pool.as_bytes()),
                (section::FREQ, &freq_bytes),
                (section::LEVELS, &levels),
                (section::GLOSS_ZST, &gloss),
                (section::TAGS, &tags),
                (section::PARADIGMS_ZST, &paradigms),
                (section::SENSES_ZST, &senses),
                (section::NOTICE, b"kaikki: CC BY-SA."),
            ],
        )
    }

    fn uds(tags: &[Tag]) -> Vec<String> {
        tags.iter().map(Tag::to_ud).collect()
    }

    #[test]
    fn spec_scenario_a_form_asked_about_as_its_own_dictionary_form() {
        let pack = Pack::load(&see_pack()).expect("load");
        // `saw` is held only as a form of see: nothing of see's entry is read for it.
        assert_eq!(pack.gloss("saw"), None);
        assert!(pack.sense_runs("saw").is_empty());
        assert!(pack.readings("saw", "saw").is_empty());
        assert!(pack.other_readings("saw", "saw").is_empty());
        // see's own entry, read as before.
        assert_eq!(pack.gloss("see"), Some("Voir; Siège"));
        assert_eq!(uds(&pack.readings("see", "saw")), [PAST]);
        let runs: Vec<(Option<String>, usize)> = pack
            .sense_runs("see")
            .into_iter()
            .map(|(tag, count)| (tag.map(Tag::to_ud), count))
            .collect();
        assert_eq!(
            runs,
            [(Some("VERB".to_owned()), 1), (Some("NOUN".to_owned()), 1)]
        );
    }

    #[test]
    fn spec_scenario_the_gloss_of_a_form() {
        let pack = Pack::load(&see_pack()).expect("load");
        assert_eq!(pack.gloss("are"), None);
        assert_eq!(pack.gloss("be"), Some("Être"));
        // A word the pack does not hold has none either.
        assert_eq!(pack.gloss("absent"), None);
    }

    #[test]
    fn the_estimates_keep_reading_through_the_spelling() {
        use crate::knowledge::level::CefrLevel;
        let pack = Pack::load(&see_pack()).expect("load");
        // Rank, level and dictionary-word mark: see's, for the form the pack reads as see.
        assert_eq!(pack.rank("saw"), Some(200));
        assert_eq!(pack.rank("saw"), pack.rank("see"));
        assert_eq!(pack.level("saw"), Some(CefrLevel::A1));
        assert!(pack.is_dictionary_word("saw"));
        assert_eq!(pack.rank("are"), Some(2));
        // Its gloss does not follow them.
        assert_eq!(pack.gloss("saw"), None);
        // A word the pack does not hold has none of them.
        assert_eq!(pack.rank("absent"), None);
        assert_eq!(pack.level("absent"), None);
        assert!(!pack.is_dictionary_word("absent"));
    }

    #[test]
    fn spec_scenario_a_lemma_list_out_of_order() {
        let (meta, sections) = read_container(&see_pack()).unwrap();
        for (pool, named) in [
            ("see\nbe", "\"be\" after \"see\""),
            ("be\nbe\nsee", "\"be\" twice"),
        ] {
            let rewritten: Vec<(&str, &[u8])> = sections
                .iter()
                .map(|s| {
                    let data: &[u8] = if s.name == section::LEMMAS {
                        pool.as_bytes()
                    } else {
                        &s.data
                    };
                    (s.name.as_str(), data)
                })
                .collect();
            match Pack::load(&write_container(&meta, &rewritten)) {
                Err(err @ PackError::Lexicon(LexiconError::LemmaPoolOutOfOrder { .. })) => {
                    assert!(err.to_string().contains(named), "{err}");
                    assert!(err.to_string().starts_with("pack lexicon: "), "{err}");
                }
                Err(other) => panic!("{pool:?}: expected LemmaPoolOutOfOrder, got {other}"),
                Ok(_) => panic!("{pool:?}: a pack whose lemma list is out of order loaded"),
            }
        }
        // The pack as built loads.
        assert!(Pack::load(&see_pack()).is_ok());
    }

    #[test]
    fn a_malformed_grammar_section_is_refused_under_its_own_name() {
        assert!(matches!(
            Pack::load(&sample_pack_with(&[(section::TAGS, &[0xff, 0xfe])])),
            Err(PackError::Malformed(section::TAGS))
        ));
        let truncated = zstd::encode_all(&1u32.to_le_bytes()[..], 19).expect("zstd encode");
        assert!(matches!(
            Pack::load(&sample_pack_with(&[(section::PARADIGMS_ZST, &truncated)])),
            Err(PackError::Malformed(section::PARADIGMS_ZST))
        ));
        assert!(matches!(
            Pack::load(&sample_pack_with(&[(section::SENSES_ZST, &truncated)])),
            Err(PackError::Malformed(section::SENSES_ZST))
        ));
        assert!(matches!(
            Pack::load(&sample_pack_with(&[(section::SENSES_ZST, b"not zstd")])),
            Err(PackError::Decompress)
        ));
    }

    #[test]
    fn a_packs_language_reads_from_its_metadata_alone() {
        // add-lingua-agent-languages D1: the language, without loading the pack.
        assert_eq!(
            Pack::studied_in(&sample_pack_bytes_for("en", ANALYZER_VERSION)).unwrap(),
            StudiedLanguage::English
        );
        assert_eq!(
            Pack::studied_in(&sample_pack_bytes_for("es", SPANISH_ANALYZER_VERSION)).unwrap(),
            StudiedLanguage::Spanish
        );
        // Nothing past the metadata is read: a pack missing its sections still says its language,
        // and its pair.
        let bare = write_container(&meta_json(ANALYZER_VERSION), &[(section::NOTICE, b"x")]);
        assert_eq!(Pack::studied_in(&bare).unwrap(), StudiedLanguage::English);
        assert_eq!(Pack::pair_in(&bare).unwrap().key(), "en-fr");
        let bare_es_en = write_container(
            &meta_json_with("es", "en", SPANISH_ANALYZER_VERSION),
            &[(section::NOTICE, b"x")],
        );
        assert_eq!(
            Pack::pair_in(&bare_es_en).unwrap(),
            LanguagePair {
                studied: StudiedLanguage::Spanish,
                native: NativeLanguage::English
            }
        );
        assert!(Pack::load(&bare).is_err());
    }

    #[test]
    fn a_packs_language_is_refused_as_load_refuses_it() {
        assert!(matches!(
            Pack::studied_in(&sample_pack_bytes_for("pt", ANALYZER_VERSION)),
            Err(PackError::UnknownLanguage(tag)) if tag == "pt"
        ));
        assert!(matches!(
            Pack::studied_in(&sample_pack_bytes_for("es", "0.1.0")),
            Err(PackError::IncompatibleAnalyzer { .. })
        ));
        assert!(matches!(
            Pack::studied_in(b"not a pack"),
            Err(PackError::Format(_))
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
