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
use super::meta::{BaseMeta, LayerMeta, PackMeta, Part, probe_part};

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
    /// (add-lingua-french-expression-keys D3), or `al menos` for `a el menos`
    /// (add-lingua-spanish-expression-keys D3). Written for a pack studying French
    /// or Spanish only, when one name differs. Optional and additive, like
    /// [`EXPR`]: a core that predates it ignores it and reports keys, as English
    /// does.
    pub const EXPR_NAMES_ZST: &str = "expr.names.zst";
    /// The other headwords of the keys several reach, for a pack studying French
    /// (pick-lingua-expression-written-as-the-page D1): `count u32 | count × (key id
    /// u32, other id u32)`, little-endian, sorted by key id and then in the order the
    /// builder ranks a key's headwords (add-lingua-french-expression-keys D6). A key id
    /// is one [`EXPR`] maps to, whose gloss and name are the headword that wins the key;
    /// an other id is one no key maps to, filed in [`EXPR_ZST`] and [`EXPR_NAMES_ZST`]
    /// like any expression — a headword of its own, with its gloss and its name.
    /// Written only for a pack studying French in which a key has other headwords.
    /// Optional and additive, like [`EXPR_NAMES_ZST`]: a core that predates it ignores
    /// it, and answers every key with the headword it answered before.
    pub const EXPR_OTHERS: &str = "expr.others";
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
    /// A layer's tags only its senses carry (split-lingua-pack-format D2): the last block of a
    /// whole pack's [`TAGS`] pool — outside its base's pool, sorted, one per line —, which
    /// assembly appends to the base's pool. Absent when the pair's senses carry none. Never in a
    /// whole pack.
    pub const SENSES_TAGS: &str = "senses.tags";
    /// The pack's dictionary words, when they are not the lemmas it glosses
    /// (add-lingua-pack-lexical-layer D1): one bit per lemma id, least
    /// significant bit first, `ceil(lemma count / 8)` bytes. Optional and
    /// additive, like [`LEVELS`]: a pack without it reads its glossed lemmas as
    /// its dictionary words, and a core that predates it ignores it and does
    /// the same.
    pub const LEXICAL: &str = "lexical";
    /// The attribution NOTICE (UTF-8).
    pub const NOTICE: &str = "notice";

    /// The sections a base holds: its studied language's (split-lingua-pack-format D2).
    pub const BASE: [&str; 7] = [FORMS, LEMMAS, FREQ, LEVELS, LEXICAL, TAGS, PARADIGMS_ZST];

    /// The sections a layer holds: its pair's (split-lingua-pack-format D2).
    pub const LAYER: [&str; 8] = [
        GLOSS_ZST,
        SENSES_ZST,
        SENSES_TAGS,
        EXPR,
        EXPR_ZST,
        EXPR_NAMES_ZST,
        EXPR_OTHERS,
        NOTICE,
    ];

    /// The part that holds the section `name` ([`BASE`], [`LAYER`]); `None` for a section this
    /// core does not know, which assembly passes on as a whole pack's reader ignores it (a
    /// table only a new interface reads is additive).
    pub fn part_of(name: &str) -> Option<crate::packs::meta::Part> {
        use crate::packs::meta::Part;
        if BASE.contains(&name) {
            Some(Part::Base)
        } else if LAYER.contains(&name) {
            Some(Part::Layer)
        } else {
            None
        }
    }
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
    /// The bytes are a base or a layer, given to a reader of whole packs
    /// (split-lingua-pack-format D1): it is assembled with the other part.
    NotAWholePack { part: Part, needs: Part },
    /// The bytes are not the part asked for: the other part, or a whole pack (`found` `None`).
    WrongPart { expected: Part, found: Option<Part> },
    /// A layer given with a base of another studied language (D3).
    LayerOfAnotherLanguage { base: String, layer: String },
    /// A layer built against another base than the one given (D3, D4).
    BaseVersionMismatch { base: String, layer: String },
    /// A part carrying a section the other part holds (D2): a base's `senses.tags`, a layer's
    /// `tags`.
    MisplacedSection { part: Part, section: String },
    /// A section carried twice, by both parts or twice by one.
    DuplicateSection(String),
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
            PackError::NotAWholePack { part, needs } => write!(
                f,
                "these bytes are a pack's {part}, not a whole pack: a {part} loads only assembled \
                 with its {needs}; refusing to load"
            ),
            PackError::WrongPart { expected, found } => match found {
                Some(found) => write!(f, "expected a pack's {expected}, found a {found}"),
                None => write!(f, "expected a pack's {expected}, found a whole pack"),
            },
            PackError::LayerOfAnotherLanguage { base, layer } => write!(
                f,
                "the layer studies {layer:?} and the base {base:?}; refusing to assemble"
            ),
            PackError::BaseVersionMismatch { base, layer } => write!(
                f,
                "the layer was built against base {layer}, and the base given is {base}; refusing \
                 to assemble"
            ),
            PackError::MisplacedSection { part, section } => write!(
                f,
                "the {part} carries the {section:?} section, which the {} holds; refusing to \
                 assemble",
                part.other()
            ),
            PackError::DuplicateSection(section) => write!(
                f,
                "the {section:?} section is carried twice; refusing to assemble"
            ),
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
    /// The form→lemma lexicon, carrying the rank per lemma id (0 = unranked), which the
    /// tokeniser reads too ([`Lexicon::lemma_rank`]).
    lexicon: FstLexicon<Vec<u8>>,
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
    /// (key id, other headword's id) pairs, sorted by key id and then in the
    /// builder's order ([`section::EXPR_OTHERS`]). Empty when the pack carries no
    /// such section.
    expression_others: Vec<(u64, u64)>,
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
        Self::from_sections(meta, pair, &sections)
    }

    /// Assembles a pack from its base and its layer (split-lingua-pack-format D3): the pack
    /// their whole container holds, every named section with the same bytes, refused for the
    /// reasons [`Pack::assemble_sections`] gives and then for those [`Pack::load`] gives a
    /// whole pack's sections, so no partial pack is ever produced.
    pub fn assemble(base: &[u8], layer: &[u8]) -> Result<Self, PackError> {
        let (meta, pair, sections) = assembled(base, layer)?;
        Self::from_sections(meta, pair, &sections)
    }

    /// The metadata and the sections a base and a layer assemble into, gathered by name, before
    /// any is parsed (split-lingua-pack-format D3): the base's sections and the layer's, the
    /// pool being the base's `tags` followed by the layer's [`section::SENSES_TAGS`], joined as a
    /// whole pack's pool is encoded; the metadata naming the layer's pair, `pack_version` and
    /// licences, and the base's analyser version and levels. Refused when either part is not
    /// what it is given as, when the layer studies another language than the base, when the
    /// base was built for another analyser generation of its language, when the layer is
    /// glossed in a native language this core does not know or in the language it studies,
    /// and when the layer was built against another base, in that order. What the builder's
    /// checks compare with the whole pack, section by name.
    pub fn assemble_sections(
        base: &[u8],
        layer: &[u8],
    ) -> Result<(PackMeta, Vec<Section>), PackError> {
        assembled(base, layer).map(|(meta, _, sections)| (meta, sections))
    }

    /// The pack a whole pack's metadata, its checked pair and its sections make: the path
    /// [`Pack::load`] takes once its sections are read, and [`Pack::assemble`] once they are
    /// gathered.
    fn from_sections(
        meta: PackMeta,
        pair: LanguagePair,
        sections: &[Section],
    ) -> Result<Self, PackError> {
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
        let lexicon = lexicon.with_ranks(freq);
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
        let expression_others = match sections.iter().find(|s| s.name == section::EXPR_OTHERS) {
            Some(s) => parse_expression_others(
                &s.data,
                expressions.as_ref(),
                &expression_glosses,
                &expression_names,
            )?,
            None => Vec::new(),
        };
        let notice = sections
            .iter()
            .find(|s| s.name == section::NOTICE)
            .map(|s| String::from_utf8_lossy(&s.data).into_owned())
            .unwrap_or_default();
        let grammar = parse_grammar(sections)?;

        Ok(Self {
            meta,
            pair,
            lexicon,
            levels,
            glosses,
            lexical,
            expressions,
            expression_glosses,
            expression_names,
            expression_others,
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

    /// Every headword filed under `key`, each with its gloss
    /// (pick-lingua-expression-written-as-the-page D1): the one that wins the key
    /// first — its name, or the key where the pack names it none — then the key's
    /// other headwords ([`section::EXPR_OTHERS`]), in the order the builder ranks
    /// them (add-lingua-french-expression-keys D6): `ça aller` lists `ça ira` and
    /// `ça va`. One entry for a key one headword reaches, and for every key of a pack
    /// without the section; none when the pack holds no such key.
    pub fn expression_headwords<'a>(&'a self, key: &'a str) -> Vec<(&'a str, &'a str)> {
        let Some(id) = self
            .expressions
            .as_ref()
            .and_then(|map| map.get(key.as_bytes()))
        else {
            return Vec::new();
        };
        let Some(gloss) = self.expression_glosses.get(&id) else {
            return Vec::new();
        };
        let name = self.expression_names.get(&id).map_or(key, String::as_str);
        std::iter::once((name, gloss.as_str()))
            .chain(self.other_headwords(id).filter_map(|other| {
                Some((
                    self.expression_names.get(other)?.as_str(),
                    self.expression_glosses.get(other)?.as_str(),
                ))
            }))
            .collect()
    }

    /// The gloss of the headword named `name` among those filed under `key`
    /// ([`Pack::expression_headwords`]), and the key's own gloss where none is so
    /// named (pick-lingua-expression-written-as-the-page D4): review reads a French
    /// card at the key its lemma reads as, and the card `ça va` shows `ça va`'s gloss,
    /// not `ça ira`'s, which wins the key. `None` when the pack holds no such key.
    pub fn expression_gloss_named(&self, key: &str, name: &str) -> Option<&str> {
        let id = self.expressions.as_ref()?.get(key.as_bytes())?;
        let own = self.expression_glosses.get(&id)?;
        if self.expression_names.get(&id).map_or(key, String::as_str) == name {
            return Some(own);
        }
        let named = self
            .other_headwords(id)
            .find(|other| self.expression_names.get(other).is_some_and(|n| n == name))
            .and_then(|other| self.expression_glosses.get(other));
        Some(named.unwrap_or(own))
    }

    /// The ids of the other headwords of the key whose id is `id`, in the builder's
    /// order: the run of [`section::EXPR_OTHERS`]'s pairs holding that key id.
    fn other_headwords(&self, id: u64) -> impl Iterator<Item = &u64> {
        let at = self.expression_others.partition_point(|&(key, _)| key < id);
        self.expression_others[at..]
            .iter()
            .take_while(move |&&(key, _)| key == id)
            .map(|(_, other)| other)
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
                let rank = *self.lexicon.ranks().get(id as usize)?;
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
                let rank = *self.lexicon.ranks().get(id as usize)?;
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
        match self.lexicon.ranks().get(id).copied() {
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
/// Reads [`section::EXPR_OTHERS`]: `count u32 | count × (key id u32, other id u32)`,
/// little-endian, into its (key id, other id) pairs, in the section's order
/// (pick-lingua-expression-written-as-the-page D1). The builder's ids are dense: a key's
/// below the number of keys `keys` holds, an other headword's from it. Refused as
/// malformed when its length is not its count's, when its pairs are not sorted by key id,
/// or when a pair holds a key id no key has, or an other id a key has or that carries no
/// gloss or no name: the section then says something the table does not hold, and the
/// pack is refused rather than misread.
fn parse_expression_others(
    bytes: &[u8],
    keys: Option<&fst::Map<Vec<u8>>>,
    glosses: &BTreeMap<u64, String>,
    names: &BTreeMap<u64, String>,
) -> Result<Vec<(u64, u64)>, PackError> {
    let bad = || PackError::Malformed(section::EXPR_OTHERS);
    let word = |at: usize| -> Option<u64> {
        let slice = bytes.get(at..at.checked_add(4)?)?;
        Some(u64::from(u32::from_le_bytes(slice.try_into().ok()?)))
    };
    let count = usize::try_from(word(0).ok_or_else(bad)?).map_err(|_| bad())?;
    if count.checked_mul(8).and_then(|n| n.checked_add(4)) != Some(bytes.len()) {
        return Err(bad());
    }
    let key_ids = 0..keys.map_or(0, |map| map.len() as u64);
    let mut others: Vec<(u64, u64)> = Vec::with_capacity(count);
    for pair in 0..count {
        let (key, other) = (
            word(4 + pair * 8).ok_or_else(bad)?,
            word(8 + pair * 8).ok_or_else(bad)?,
        );
        if !key_ids.contains(&key)
            || key_ids.contains(&other)
            || !glosses.contains_key(&other)
            || !names.contains_key(&other)
            || others.last().is_some_and(|&(before, _)| before > key)
        {
            return Err(bad());
        }
        others.push((key, other));
    }
    Ok(others)
}

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
/// its sections still unread. A base or a layer is refused by name before its metadata is read
/// as a whole pack's, which it is not (split-lingua-pack-format D1).
fn read_meta(bytes: &[u8]) -> Result<(PackMeta, LanguagePair, Vec<Section>), PackError> {
    let (meta_json, sections) = read_container(bytes).map_err(PackError::Format)?;
    if let Some(part) = probe_part(&meta_json).map_err(PackError::BadMeta)? {
        return Err(PackError::NotAWholePack {
            part,
            needs: part.other(),
        });
    }
    let meta: PackMeta = serde_json::from_slice(&meta_json).map_err(PackError::BadMeta)?;
    let studied = checked_studied(&meta.studied, &meta.analyzer_version)?;
    let pair = checked_pair(studied, &meta.native)?;
    Ok((meta, pair, sections))
}

/// The studied language a pack names, refused when this core cannot analyse it or when the
/// pack was built for another analyser generation of it.
fn checked_studied(tag: &str, analyzer_version: &str) -> Result<StudiedLanguage, PackError> {
    let studied =
        StudiedLanguage::from_tag(tag).ok_or_else(|| PackError::UnknownLanguage(tag.to_owned()))?;
    if !analyzer_compatible(studied, analyzer_version) {
        return Err(PackError::IncompatibleAnalyzer {
            pack: analyzer_version.to_owned(),
            core: studied.analyzer_version().to_owned(),
        });
    }
    Ok(studied)
}

/// The pair a pack serves, its native language refused when this core does not know it or when
/// it is the studied one. Checked after the studied language and the analyser, so their errors
/// keep their precedence (generalise-lingua-native-language D1).
fn checked_pair(studied: StudiedLanguage, native: &str) -> Result<LanguagePair, PackError> {
    let native = NativeLanguage::from_tag(native)
        .ok_or_else(|| PackError::UnknownNative(native.to_owned()))?;
    if native.studied() == Some(studied) {
        return Err(PackError::NativeStudied(studied));
    }
    Ok(LanguagePair { studied, native })
}

/// A part's metadata and sections, refused as [`PackError::WrongPart`] when the container holds
/// the other part or a whole pack.
pub(super) fn read_part<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    expected: Part,
) -> Result<(T, Vec<Section>), PackError> {
    let (meta_json, sections) = read_container(bytes).map_err(PackError::Format)?;
    let found = probe_part(&meta_json).map_err(PackError::BadMeta)?;
    if found != Some(expected) {
        return Err(PackError::WrongPart { expected, found });
    }
    let meta = serde_json::from_slice(&meta_json).map_err(PackError::BadMeta)?;
    Ok((meta, sections))
}

/// [`Pack::assemble_sections`], with the pair checked as `load` checks a whole pack's.
fn assembled(
    base: &[u8],
    layer: &[u8],
) -> Result<(PackMeta, LanguagePair, Vec<Section>), PackError> {
    let (base_meta, base_sections) = read_part::<BaseMeta>(base, Part::Base)?;
    let (layer_meta, layer_sections) = read_part::<LayerMeta>(layer, Part::Layer)?;
    if layer_meta.studied != base_meta.studied {
        return Err(PackError::LayerOfAnotherLanguage {
            base: base_meta.studied,
            layer: layer_meta.studied,
        });
    }
    let studied = checked_studied(&base_meta.studied, &base_meta.analyzer_version)?;
    let pair = checked_pair(studied, &layer_meta.native)?;
    if layer_meta.base_version != base_meta.base_version {
        return Err(PackError::BaseVersionMismatch {
            base: base_meta.base_version,
            layer: layer_meta.base_version,
        });
    }
    // Each part carries its own sections (D2), each once: a section on the wrong side, or twice,
    // would shadow or be dropped. A section this core does not know passes, as it does in a whole
    // pack.
    for (part, sections) in [(Part::Base, &base_sections), (Part::Layer, &layer_sections)] {
        if let Some(misplaced) = sections
            .iter()
            .find(|s| section::part_of(&s.name) == Some(part.other()))
        {
            return Err(PackError::MisplacedSection {
                part,
                section: misplaced.name.clone(),
            });
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    if let Some(twice) = base_sections
        .iter()
        .chain(&layer_sections)
        .find(|s| !seen.insert(s.name.as_str()))
    {
        return Err(PackError::DuplicateSection(twice.name.clone()));
    }
    // The pool: the base's blocks, then the layer's sense-only block, one tag per line with no
    // line after the last, as `encode_tag_pool` writes a whole pack's.
    let find = |sections: &[Section], name: &str| -> Vec<u8> {
        sections
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.data.clone())
            .unwrap_or_default()
    };
    let mut pool = find(&base_sections, section::TAGS);
    let senses = find(&layer_sections, section::SENSES_TAGS);
    if !pool.is_empty() && !senses.is_empty() {
        pool.push(b'\n');
    }
    pool.extend_from_slice(&senses);
    let mut sections: Vec<Section> = base_sections
        .into_iter()
        .chain(layer_sections)
        .filter(|s| s.name != section::TAGS && s.name != section::SENSES_TAGS)
        .collect();
    if !pool.is_empty() {
        sections.push(Section {
            name: section::TAGS.to_owned(),
            data: pool,
        });
    }
    let meta = PackMeta {
        studied: layer_meta.studied,
        native: layer_meta.native,
        pack_version: layer_meta.pack_version,
        analyzer_version: base_meta.analyzer_version,
        licences: layer_meta.licences,
        levels_estimated: base_meta.levels_estimated,
    };
    Ok((meta, pair, sections))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::analysis::lexicon::build_lexicon_blobs;
    use crate::analysis::{
        ANALYZER_VERSION, FRENCH_ANALYZER_VERSION, ITALIAN_ANALYZER_VERSION,
        SPANISH_ANALYZER_VERSION,
    };
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
        assert_eq!(pack.meta().analyzer_version, "1.3.0");
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
        // Spanish's is refused. English's version was French's own too, as it was Spanish's once
        // (`1.1.0` since add-lingua-french-detection-guard, `1.2.0` since
        // ignore-lingua-soft-hyphens, until leave-lingua-names-out-of-english-and-spanish-words
        // moved English to `1.3.0`), and is again since keep-lingua-french-acronyms-in-capitals
        // moved French to `1.3.0`: the numbers may meet.
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
        // A pack built at `1.2.0`, before French's acronyms, at `1.0.0`, before its detection
        // guard, at `0.2.0`, the version French had before its own analysis, or at the baseline's
        // `0.1.0`, read French otherwise: each is refused.
        for before in ["1.2.0", "1.0.0", "0.2.0", "0.1.0"] {
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
    fn spec_scenario_an_italian_pack_at_italian_s_analyser_version() {
        // add-lingua-italian-baseline: a pack studying Italian loads against Italian's own
        // version, its own analysis's since add-lingua-italian-analysis, and is glossed in
        // French, English or Spanish.
        assert!(!ITALIAN_ANALYZER_VERSION.starts_with("0."));
        let bytes = sample_pack_bytes_with("it", "fr", ITALIAN_ANALYZER_VERSION);
        let italian = Pack::load(&bytes).expect("an Italian pack at Italian's version loads");
        assert_eq!(italian.studied(), StudiedLanguage::Italian);
        assert_eq!(italian.meta().analyzer_version, ITALIAN_ANALYZER_VERSION);
        assert_eq!(italian.pair().key(), "it-fr");
        assert_eq!(Pack::studied_in(&bytes).unwrap(), StudiedLanguage::Italian);
        for native in ["en", "es"] {
            let bytes = sample_pack_bytes_with("it", native, ITALIAN_ANALYZER_VERSION);
            let pack = Pack::load(&bytes).expect("an it-en or it-es pack loads");
            assert_eq!(pack.pair().key(), format!("it-{native}"));
        }
        // Compared with Italian's own version, never another language's: an Italian pack at
        // another language's version is refused as built for another analyser generation —
        // Spanish's; French's and English's are Italian's own string since
        // keep-lingua-italian-acronyms-and-names, `1.3.0` —, and so is one built at `1.2.0`, before
        // Italian's acronym rule, at `1.1.0`, before its accent apostrophe
        // (read-lingua-italian-unmarked-spellings), at `1.0.0`, before Italian's detection guard
        // (add-lingua-italian-detection-guard), at `0.2.0`, the version Italian had before its own
        // analysis, or by the baseline's tokeniser, at `0.1.0`, which keeps `l'uomo` whole.
        for other in [
            FRENCH_ANALYZER_VERSION,
            ANALYZER_VERSION,
            SPANISH_ANALYZER_VERSION,
            "1.2.0",
            "1.1.0",
            "1.0.0",
            "0.2.0",
            "0.1.0",
        ]
        .into_iter()
        .filter(|other| *other != ITALIAN_ANALYZER_VERSION)
        {
            let refused = Pack::load(&sample_pack_bytes_with("it", "fr", other)).err();
            assert!(
                matches!(
                    refused,
                    Some(PackError::IncompatibleAnalyzer { ref pack, ref core })
                        if pack == other && core == ITALIAN_ANALYZER_VERSION
                ),
                "{other}"
            );
        }
        // And a French pack is compared with French's version, not Italian's: at Italian's version
        // before this rule it is refused naming French's.
        assert!(matches!(
            Pack::load(&sample_pack_bytes_with("fr", "en", "1.2.0")),
            Err(PackError::IncompatibleAnalyzer { core, .. }) if core == FRENCH_ANALYZER_VERSION
        ));
    }

    #[test]
    fn spec_scenario_an_italian_pack_glossed_in_a_language_the_core_does_not_know_as_native() {
        // add-lingua-italian-baseline D3: Italian is no native language before change 32, so a
        // pack studying Italian glossed in Italian is refused as glossed in an unknown native
        // language — the pack-glossed-in-its-own-language rule cannot fire for it yet.
        let bytes = sample_pack_bytes_with("it", "it", ITALIAN_ANALYZER_VERSION);
        let err = Pack::load(&bytes)
            .err()
            .expect("a pack glossed in Italian is refused");
        assert!(
            matches!(&err, PackError::UnknownNative(tag) if tag == "it"),
            "{err}"
        );
        assert!(err.to_string().contains("\"it\""), "{err}");
        assert!(matches!(
            Pack::pair_in(&bytes),
            Err(PackError::UnknownNative(tag)) if tag == "it"
        ));
        // A pack of another language glossed in Italian is refused the same way.
        assert!(matches!(
            Pack::load(&sample_pack_bytes_with("en", "it", ANALYZER_VERSION)),
            Err(PackError::UnknownNative(tag)) if tag == "it"
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
        // The lexicon carries the same ranks, by lemma, for the tokeniser
        // (read-lingua-italian-unmarked-spellings, at review); a form is no lemma.
        assert_eq!(pack.lexicon().lemma_rank("run"), Some(500));
        assert_eq!(pack.lexicon().lemma_rank("seldom"), None);
        assert_eq!(pack.lexicon().lemma_rank("running"), None);
        assert_eq!(pack.lexicon().lemma_rank("absent"), None);
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

    // — every headword of a shared key (pick-lingua-expression-written-as-the-page D1) —

    /// The `expr.others` section: `count u32 | count × (key id u32, other id u32)`.
    fn build_expr_others(pairs: &[(u32, u32)]) -> Vec<u8> {
        let mut raw = (pairs.len() as u32).to_le_bytes().to_vec();
        for (key, other) in pairs {
            raw.extend_from_slice(&key.to_le_bytes());
            raw.extend_from_slice(&other.to_le_bytes());
        }
        raw
    }

    /// A small fr-en pack laid out as the builder lays out a key several headwords reach:
    /// `tout de suite` (id 0, its own name) and `ça aller` (id 1), won by `ça ira`, whose other
    /// headword `ça va` is id 2, which no key maps to. `glosses` and `names` replace the
    /// tables' when given, `others` is the section's bytes when given.
    fn shared_key_pack(
        glosses: Option<&[(u32, &str)]>,
        names: Option<&[(u32, &str)]>,
        others: Option<&[u8]>,
    ) -> Vec<u8> {
        let (forms, pool) =
            build_lexicon_blobs(&[], &["ça", "aller", "tout", "de", "suite"]).expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &pool).unwrap();
        let freq_bytes = vec![0u8; lex.lemma_count() * 4];
        let mut keys = fst::MapBuilder::memory();
        keys.insert("tout de suite", 0).unwrap();
        keys.insert("ça aller", 1).unwrap();
        let keys = keys.into_inner().unwrap();
        let glosses = build_gloss_zst(glosses.unwrap_or(&[
            (0, "right away"),
            (1, "it will be all right"),
            (2, "how are you?"),
        ]));
        let names = build_gloss_zst(names.unwrap_or(&[(1, "ça ira"), (2, "ça va")]));
        let mut sections: Vec<(&str, &[u8])> = vec![
            (section::FORMS, &forms),
            (section::LEMMAS, pool.as_bytes()),
            (section::FREQ, &freq_bytes),
            (section::EXPR, &keys),
            (section::EXPR_ZST, &glosses),
            (section::EXPR_NAMES_ZST, &names),
        ];
        if let Some(others) = others {
            sections.push((section::EXPR_OTHERS, others));
        }
        write_container(
            &meta_json_with("fr", "en", FRENCH_ANALYZER_VERSION),
            &sections,
        )
    }

    #[test]
    fn spec_scenario_two_headwords_reach_one_key() {
        let pack = Pack::load(&shared_key_pack(
            None,
            None,
            Some(&build_expr_others(&[(1, 2)])),
        ))
        .unwrap();
        // The key answers the headword that wins it, its gloss and its name, as before…
        assert_eq!(pack.expression("ça aller"), Some("it will be all right"));
        assert_eq!(pack.expression_name("ça aller"), Some("ça ira"));
        // …and lists the other headword after it, with its own gloss.
        assert_eq!(
            pack.expression_headwords("ça aller"),
            [
                ("ça ira", "it will be all right"),
                ("ça va", "how are you?")
            ]
        );
        // A key one headword reaches lists it alone, under its key where the pack names it none;
        // a headword is no key, nor is what the pack does not hold.
        assert_eq!(
            pack.expression_headwords("tout de suite"),
            [("tout de suite", "right away")]
        );
        assert!(pack.expression_headwords("ça va").is_empty());
        assert!(pack.expression_headwords("au revoir").is_empty());
    }

    #[test]
    fn a_gloss_is_read_under_the_headword_it_is_named_by() {
        let pack = Pack::load(&shared_key_pack(
            None,
            None,
            Some(&build_expr_others(&[(1, 2)])),
        ))
        .unwrap();
        assert_eq!(
            pack.expression_gloss_named("ça aller", "ça va"),
            Some("how are you?")
        );
        assert_eq!(
            pack.expression_gloss_named("ça aller", "ça ira"),
            Some("it will be all right")
        );
        // A name no headword of the key carries reads the key's own gloss; a key the pack does not
        // hold, none.
        assert_eq!(
            pack.expression_gloss_named("ça aller", "ça allait"),
            Some("it will be all right")
        );
        assert_eq!(
            pack.expression_gloss_named("tout de suite", "tout de suite"),
            Some("right away")
        );
        assert_eq!(
            pack.expression_gloss_named("tout de suite", "ça va"),
            Some("right away")
        );
        assert_eq!(pack.expression_gloss_named("ça va", "ça va"), None);
        // Without the section, every name reads the key's own gloss.
        let older = Pack::load(&shared_key_pack(None, None, None)).unwrap();
        assert_eq!(
            older.expression_gloss_named("ça aller", "ça va"),
            Some("it will be all right")
        );
    }

    #[test]
    fn spec_scenario_a_core_that_does_not_read_the_section() {
        // A core that predates the section ignores it (`a_pack_carrying_an_unknown_section_loads_
        // unchanged`), and the other headword's id is one no key maps to: stripped of the section,
        // the pack answers every key with the headword, gloss and name it answers with it.
        let with = Pack::load(&shared_key_pack(
            None,
            None,
            Some(&build_expr_others(&[(1, 2)])),
        ))
        .unwrap();
        let older = Pack::load(&shared_key_pack(None, None, None)).unwrap();
        for key in ["ça aller", "tout de suite", "ça va", "ça ira"] {
            assert_eq!(older.expression(key), with.expression(key), "{key}");
            assert_eq!(
                older.expression_name(key),
                with.expression_name(key),
                "{key}"
            );
            assert_eq!(
                older.expression_headwords(key).first(),
                with.expression_headwords(key).first(),
                "{key}"
            );
        }
        assert_eq!(older.expression_headwords("ça aller").len(), 1);
    }

    #[test]
    fn a_malformed_others_section_is_refused_under_its_own_name() {
        let refused = |others: &[u8]| {
            matches!(
                Pack::load(&shared_key_pack(None, None, Some(others))),
                Err(PackError::Malformed(section::EXPR_OTHERS))
            )
        };
        // Truncated: no count, a count announcing a pair it does not carry, a stray byte.
        assert!(refused(b""));
        assert!(refused(&build_expr_others(&[(1, 2)])[..8]));
        let mut longer = build_expr_others(&[(1, 2)]);
        longer.push(0);
        assert!(refused(&longer));
        // A key id no key has, and an other id a key maps to.
        assert!(refused(&build_expr_others(&[(5, 2)])));
        assert!(refused(&build_expr_others(&[(1, 0)])));
        // Pairs out of key order.
        let unsorted = shared_key_pack(
            Some(&[
                (0, "right away"),
                (1, "it will be all right"),
                (2, "how are you?"),
                (3, "in a minute"),
            ]),
            Some(&[(1, "ça ira"), (2, "ça va"), (3, "tout à l'heure")]),
            Some(&build_expr_others(&[(1, 2), (0, 3)])),
        );
        assert!(matches!(
            Pack::load(&unsorted),
            Err(PackError::Malformed(section::EXPR_OTHERS))
        ));
        // In key order, each key lists its own.
        let two_keys = Pack::load(&shared_key_pack(
            Some(&[
                (0, "right away"),
                (1, "it will be all right"),
                (2, "how are you?"),
                (3, "in a minute"),
            ]),
            Some(&[(1, "ça ira"), (2, "ça va"), (3, "tout à l'heure")]),
            Some(&build_expr_others(&[(0, 3), (1, 2)])),
        ))
        .unwrap();
        assert_eq!(
            two_keys.expression_headwords("tout de suite"),
            [
                ("tout de suite", "right away"),
                ("tout à l'heure", "in a minute")
            ]
        );
        assert_eq!(
            two_keys.expression_headwords("ça aller"),
            [
                ("ça ira", "it will be all right"),
                ("ça va", "how are you?")
            ]
        );
        // An other id with no gloss, and one with a gloss and no name.
        assert!(refused(&build_expr_others(&[(1, 7)])));
        let unnamed = shared_key_pack(
            None,
            Some(&[(1, "ça ira")]),
            Some(&build_expr_others(&[(1, 2)])),
        );
        assert!(matches!(
            Pack::load(&unnamed),
            Err(PackError::Malformed(section::EXPR_OTHERS))
        ));
        // The well-formed section loads, and an empty one too.
        assert!(!refused(&build_expr_others(&[(1, 2)])));
        assert!(!refused(&build_expr_others(&[])));
        // A pack carrying the section without an expression table says something no key holds.
        let (meta, sections) = read_container(&shared_key_pack(
            None,
            None,
            Some(&build_expr_others(&[(1, 2)])),
        ))
        .unwrap();
        let tableless: Vec<(&str, &[u8])> = sections
            .iter()
            .filter(|s| s.name != section::EXPR)
            .map(|s| (s.name.as_str(), s.data.as_slice()))
            .collect();
        assert!(matches!(
            Pack::load(&write_container(&meta, &tableless)),
            Err(PackError::Malformed(section::EXPR_OTHERS))
        ));
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

    // — a base and a layer (split-lingua-pack-format) —

    /// The tags of the sample's pool its base holds: the pin and the readings' tags.
    const STUDIED_TAGS: [&str; 2] = ["VERB", PAST];

    /// A whole pack, and the base and the layer holding its sections (D2), built in memory from
    /// the same sections: the sample's lexicon, ranks, levels, dictionary words, a paradigm, an
    /// expression and a gloss whose runs use `sense_only` tags past the base's pool. The base
    /// is named `base_version`, the layer is built against `against`.
    struct Split {
        whole: Vec<u8>,
        base: Vec<u8>,
        layer: Vec<u8>,
    }

    fn split_with(
        studied: &str,
        native: &str,
        analyzer: &str,
        sense_only: &[&str],
        (base_version, against): (&str, &str),
    ) -> Split {
        use super::super::grammar::{
            FormEdit, SenseRun, encode_indexed, encode_paradigm, encode_runs, encode_tag_pool,
        };
        let (forms, lemmas) = build_lexicon_blobs(
            &[("running", "run"), ("ran", "run"), ("cities", "city")],
            &["run", "city", "seldom"],
        )
        .expect("lexicon");
        let lex = FstLexicon::from_slices(forms.clone(), &lemmas).unwrap();
        let id = |lemma: &str| lex.id_of(lemma).unwrap() as u32;
        let mut freq = vec![0u8; lex.lemma_count() * 4];
        freq[id("run") as usize * 4..][..4].copy_from_slice(&500u32.to_le_bytes());
        freq[id("city") as usize * 4..][..4].copy_from_slice(&1_200u32.to_le_bytes());
        let mut levels = vec![0u8; lex.lemma_count()];
        levels[id("run") as usize] = CefrLevel::A1.to_code();
        let lexical = vec![1u8 << id("run")];
        let gloss = build_gloss_zst(&[(id("run"), "courir; course"), (id("city"), "ville")]);
        let (expr, expr_zst) = build_expr_sections(&[("run city", "course urbaine")]);
        let all: Vec<String> = STUDIED_TAGS
            .iter()
            .chain(sense_only)
            .map(|t| (*t).to_owned())
            .collect();
        let pool = encode_tag_pool(&all);
        let base_pool = encode_tag_pool(&all[..STUDIED_TAGS.len()]);
        let senses_tags = encode_tag_pool(&all[STUDIED_TAGS.len()..]);
        let paradigm = encode_paradigm(&[ParadigmEntry::Reading {
            form: FormEdit::between("run", "ran").unwrap(),
            tag: 1,
        }]);
        let paradigms =
            zstd::encode_all(encode_indexed(&[(id("run"), paradigm)]).as_slice(), 19).unwrap();
        // The noun run points past the base's pool when the pair's senses carry a tag of their
        // own, at the verb otherwise.
        let noun = if sense_only.is_empty() { 0 } else { 2 };
        let runs = encode_runs(&[
            SenseRun { tag: 0, count: 1 },
            SenseRun {
                tag: noun,
                count: 1,
            },
        ]);
        let senses = zstd::encode_all(encode_indexed(&[(id("run"), runs)]).as_slice(), 19).unwrap();
        let notice: &[u8] = b"AGID: permissive. wordfreq: CC BY-SA. kaikki: CC BY-SA.";
        let licences = vec!["AGID".to_owned(), "kaikki CC BY-SA".to_owned()];
        let whole_meta = PackMeta {
            studied: studied.into(),
            native: native.into(),
            pack_version: "2026.10.21".into(),
            analyzer_version: analyzer.into(),
            licences: licences.clone(),
            levels_estimated: true,
        };
        let base_meta = BaseMeta {
            part: Part::Base,
            studied: studied.into(),
            analyzer_version: analyzer.into(),
            base_version: base_version.into(),
            levels_estimated: true,
        };
        let layer_meta = LayerMeta {
            part: Part::Layer,
            studied: studied.into(),
            native: native.into(),
            pack_version: "2026.10.21".into(),
            base_version: against.into(),
            licences,
        };
        // The builder's whole order: the two sides interleaved.
        let whole: Vec<(&str, &[u8])> = vec![
            (section::FORMS, &forms),
            (section::LEMMAS, lemmas.as_bytes()),
            (section::FREQ, &freq),
            (section::LEVELS, &levels),
            (section::LEXICAL, &lexical),
            (section::EXPR, &expr),
            (section::EXPR_ZST, &expr_zst),
            (section::TAGS, &pool),
            (section::PARADIGMS_ZST, &paradigms),
            (section::SENSES_ZST, &senses),
            (section::GLOSS_ZST, &gloss),
            (section::NOTICE, notice),
        ];
        let base: Vec<(&str, &[u8])> = vec![
            (section::FORMS, &forms),
            (section::LEMMAS, lemmas.as_bytes()),
            (section::FREQ, &freq),
            (section::LEVELS, &levels),
            (section::LEXICAL, &lexical),
            (section::TAGS, &base_pool),
            (section::PARADIGMS_ZST, &paradigms),
        ];
        let mut layer: Vec<(&str, &[u8])> =
            vec![(section::GLOSS_ZST, &gloss), (section::SENSES_ZST, &senses)];
        if !sense_only.is_empty() {
            layer.push((section::SENSES_TAGS, &senses_tags));
        }
        layer.extend([
            (section::EXPR, expr.as_slice()),
            (section::EXPR_ZST, expr_zst.as_slice()),
            (section::NOTICE, notice),
        ]);
        Split {
            whole: write_container(&serde_json::to_vec(&whole_meta).unwrap(), &whole),
            base: write_container(&serde_json::to_vec(&base_meta).unwrap(), &base),
            layer: write_container(&serde_json::to_vec(&layer_meta).unwrap(), &layer),
        }
    }

    fn split(sense_only: &[&str]) -> Split {
        split_with(
            "en",
            "fr",
            ANALYZER_VERSION,
            sense_only,
            ("base-digest", "base-digest"),
        )
    }

    /// A container's sections, by name.
    fn by_name(sections: Vec<Section>) -> BTreeMap<String, Vec<u8>> {
        sections.into_iter().map(|s| (s.name, s.data)).collect()
    }

    #[test]
    fn spec_scenario_the_assembled_pack_is_the_whole_pack() {
        for sense_only in [&["NOUN", "NUM"][..], &[]] {
            let parts = split(sense_only);
            let (whole_json, whole_sections) = read_container(&parts.whole).unwrap();
            let (meta, sections) = Pack::assemble_sections(&parts.base, &parts.layer).unwrap();
            // Every named section with the same bytes, the pool included, and no other.
            assert_eq!(by_name(sections), by_name(whole_sections), "{sense_only:?}");
            // The same metadata, written as the whole pack's.
            assert_eq!(serde_json::to_vec(&meta).unwrap(), whole_json);

            let whole = Pack::load(&parts.whole).unwrap();
            let assembled = Pack::assemble(&parts.base, &parts.layer).unwrap();
            assert_eq!(assembled.meta(), whole.meta());
            assert_eq!(assembled.pair(), whole.pair());
            assert_eq!(assembled.pair().key(), "en-fr");
            assert_eq!(assembled.gloss("run"), Some("courir; course"));
            assert_eq!(assembled.gloss("city"), whole.gloss("city"));
            assert_eq!(assembled.rank("city"), Some(1_200));
            assert_eq!(assembled.level("run"), Some(CefrLevel::A1));
            assert!(assembled.levels_estimated());
            assert_eq!(assembled.dictionary_words(), whole.dictionary_words());
            assert_eq!(assembled.dictionary_words(), [("run", 500)]);
            assert_eq!(
                assembled.expression("run city"),
                Some("course urbaine"),
                "the layer's expression table"
            );
            assert_eq!(assembled.notice(), whole.notice());
            let readings = |pack: &Pack| uds(&pack.readings("run", "ran"));
            assert_eq!(readings(&assembled), [PAST]);
            assert_eq!(readings(&assembled), readings(&whole));
            let runs = |pack: &Pack| -> Vec<(Option<String>, usize)> {
                pack.sense_runs("run")
                    .into_iter()
                    .map(|(tag, n)| (tag.map(Tag::to_ud), n))
                    .collect()
            };
            assert_eq!(runs(&assembled), runs(&whole));
            let noun = if sense_only.is_empty() {
                "VERB"
            } else {
                "NOUN"
            };
            assert_eq!(
                runs(&assembled),
                [(Some("VERB".to_owned()), 1), (Some(noun.to_owned()), 1)]
            );
        }
    }

    #[test]
    fn the_pool_is_the_base_s_then_the_layer_s_sense_tags() {
        let parts = split(&["NUM"]);
        let (_, sections) = Pack::assemble_sections(&parts.base, &parts.layer).unwrap();
        let sections = by_name(sections);
        assert_eq!(
            sections[section::TAGS],
            format!("VERB\n{PAST}\nNUM").into_bytes()
        );
        assert!(!sections.contains_key(section::SENSES_TAGS));
        // Without the layer's block, the pool is the base's alone.
        let (_, sections) = Pack::assemble_sections(&split(&[]).base, &split(&[]).layer).unwrap();
        assert_eq!(
            by_name(sections)[section::TAGS],
            format!("VERB\n{PAST}").into_bytes()
        );
        // A base without a pool takes the layer's block as the whole pool, and two parts
        // without either carry none.
        let strip = |bytes: &[u8], name: &str| {
            let (meta, sections) = read_container(bytes).unwrap();
            let kept: Vec<(&str, &[u8])> = sections
                .iter()
                .filter(|s| s.name != name)
                .map(|s| (s.name.as_str(), s.data.as_slice()))
                .collect();
            write_container(&meta, &kept)
        };
        let poolless = strip(&parts.base, section::TAGS);
        let (_, sections) = Pack::assemble_sections(&poolless, &parts.layer).unwrap();
        assert_eq!(by_name(sections)[section::TAGS], b"NUM");
        let senseless = strip(&parts.layer, section::SENSES_TAGS);
        let (_, sections) = Pack::assemble_sections(&poolless, &senseless).unwrap();
        assert!(!by_name(sections).contains_key(section::TAGS));
    }

    #[test]
    fn spec_scenario_a_part_given_to_a_reader_of_whole_packs() {
        let parts = split(&["NUM"]);
        for (bytes, part, needs) in [
            (&parts.base, Part::Base, Part::Layer),
            (&parts.layer, Part::Layer, Part::Base),
        ] {
            let refused = |result: Result<(), PackError>| {
                let err = result.expect_err("a part is no whole pack");
                let message = err.to_string();
                assert!(message.contains(&format!("a pack's {part}")), "{message}");
                assert!(message.contains(&format!("with its {needs}")), "{message}");
                assert!(matches!(
                    err,
                    PackError::NotAWholePack { part: p, needs: n } if p == part && n == needs
                ));
            };
            refused(Pack::load(bytes).map(|_| ()));
            refused(Pack::studied_in(bytes).map(|_| ()));
            refused(Pack::pair_in(bytes).map(|_| ()));
        }
        // The whole pack still loads, and says its pair.
        assert_eq!(Pack::pair_in(&parts.whole).unwrap().key(), "en-fr");
        // A part named otherwise than the two is bad metadata to every reader.
        let core = write_container(br#"{"part":"core","studied":"en"}"#, &[]);
        assert!(matches!(Pack::pair_in(&core), Err(PackError::BadMeta(_))));
    }

    #[test]
    fn each_part_is_refused_where_the_other_is_expected() {
        let parts = split(&[]);
        let refused = |base: &[u8], layer: &[u8]| {
            Pack::assemble(base, layer)
                .err()
                .expect("a part given for the other")
        };
        let swapped = refused(&parts.layer, &parts.base);
        assert!(matches!(
            swapped,
            PackError::WrongPart {
                expected: Part::Base,
                found: Some(Part::Layer)
            }
        ));
        assert!(swapped.to_string().contains("found a layer"), "{swapped}");
        let whole = refused(&parts.base, &parts.whole);
        assert!(matches!(
            whole,
            PackError::WrongPart {
                expected: Part::Layer,
                found: None
            }
        ));
        assert!(whole.to_string().contains("found a whole pack"), "{whole}");
        assert!(matches!(
            Pack::assemble(b"not a pack", &parts.layer),
            Err(PackError::Format(_))
        ));
    }

    #[test]
    fn spec_scenario_a_layer_of_another_language() {
        let english = split(&[]);
        let spanish = split_with(
            "es",
            "fr",
            SPANISH_ANALYZER_VERSION,
            &[],
            ("base-digest", "base-digest"),
        );
        let err = Pack::assemble(&english.base, &spanish.layer)
            .err()
            .expect("an es-fr layer is refused beside an English base");
        let message = err.to_string();
        assert!(
            message.contains("\"es\"") && message.contains("\"en\""),
            "{message}"
        );
        assert!(matches!(err, PackError::LayerOfAnotherLanguage { .. }));
        assert!(Pack::assemble(&spanish.base, &spanish.layer).is_ok());
    }

    #[test]
    fn spec_scenario_a_layer_built_against_another_base() {
        let parts = split_with("en", "es", ANALYZER_VERSION, &["NUM"], ("aaaa", "bbbb"));
        let err = Pack::assemble(&parts.base, &parts.layer)
            .err()
            .expect("a layer built against another base is refused");
        let message = err.to_string();
        assert!(
            message.contains("aaaa") && message.contains("bbbb"),
            "{message}"
        );
        assert!(matches!(
            &err,
            PackError::BaseVersionMismatch { base, layer } if base == "aaaa" && layer == "bbbb"
        ));
    }

    /// `bytes` with `extra` appended to its sections.
    fn with_section(bytes: &[u8], extra: (&str, &[u8])) -> Vec<u8> {
        let (meta, sections) = read_container(bytes).unwrap();
        let sections: Vec<(&str, &[u8])> = sections
            .iter()
            .map(|s| (s.name.as_str(), s.data.as_slice()))
            .chain(std::iter::once(extra))
            .collect();
        write_container(&meta, &sections)
    }

    #[test]
    fn a_section_on_the_wrong_side_or_carried_twice_is_refused_by_name() {
        let parts = split(&["NUM"]);
        // A base carrying the layer's sense tags, a layer carrying a pool.
        let err = Pack::assemble(
            &with_section(&parts.base, (section::SENSES_TAGS, b"NUM")),
            &parts.layer,
        )
        .err()
        .expect("a base's senses.tags is refused");
        assert!(err.to_string().contains("\"senses.tags\""), "{err}");
        assert!(err.to_string().contains("which the layer holds"), "{err}");
        assert!(matches!(
            &err,
            PackError::MisplacedSection { part: Part::Base, section } if section == section::SENSES_TAGS
        ));
        let err = Pack::assemble(
            &parts.base,
            &with_section(&parts.layer, (section::TAGS, b"X")),
        )
        .err()
        .expect("a layer's tags is refused");
        assert!(matches!(
            &err,
            PackError::MisplacedSection { part: Part::Layer, section } if section == section::TAGS
        ));
        // A section both parts carry, or one carries twice.
        let err = Pack::assemble(
            &with_section(&parts.base, ("future", b"x")),
            &with_section(&parts.layer, ("future", b"y")),
        )
        .err()
        .expect("a section carried twice is refused");
        assert!(err.to_string().contains("\"future\""), "{err}");
        assert!(matches!(&err, PackError::DuplicateSection(name) if name == "future"));
        assert!(matches!(
            Pack::assemble(&with_section(&parts.base, (section::FORMS, b"x")), &parts.layer),
            Err(PackError::DuplicateSection(name)) if name == section::FORMS
        ));
        // A section this core does not know is passed on, as a whole pack's reader ignores it.
        let future = with_section(&parts.layer, ("future", b"y"));
        let (_, sections) = Pack::assemble_sections(&parts.base, &future).unwrap();
        assert!(sections.iter().any(|s| s.name == "future"));
        assert!(Pack::assemble(&parts.base, &future).is_ok());
        assert_eq!(section::part_of(section::TAGS), Some(Part::Base));
        assert_eq!(section::part_of(section::SENSES_TAGS), Some(Part::Layer));
        assert_eq!(section::part_of("future"), None);
    }

    #[test]
    fn assembly_refuses_what_load_refuses_and_in_its_order() {
        let assemble = |studied, native, analyzer| {
            let parts = split_with(studied, native, analyzer, &[], ("x", "x"));
            Pack::assemble(&parts.base, &parts.layer).map(|_| ())
        };
        assert!(matches!(
            assemble("en", "de", ANALYZER_VERSION),
            Err(PackError::UnknownNative(tag)) if tag == "de"
        ));
        assert!(matches!(
            assemble("es", "es", SPANISH_ANALYZER_VERSION),
            Err(PackError::NativeStudied(StudiedLanguage::Spanish))
        ));
        assert!(matches!(
            assemble("en", "fr", "0.0.0-from-another-era"),
            Err(PackError::IncompatibleAnalyzer { .. })
        ));
        assert!(matches!(
            assemble("pt", "fr", ANALYZER_VERSION),
            Err(PackError::UnknownLanguage(tag)) if tag == "pt"
        ));
        // The analyser wins over the native language, and the native language over the base
        // version, as for a whole pack.
        assert!(matches!(
            assemble("en", "en", "0.0.0-from-another-era"),
            Err(PackError::IncompatibleAnalyzer { .. })
        ));
        let mismatched = split_with("en", "de", ANALYZER_VERSION, &[], ("x", "y"));
        assert!(matches!(
            Pack::assemble(&mismatched.base, &mismatched.layer),
            Err(PackError::UnknownNative(_))
        ));
        // A section the assembled pack cannot read is refused as `load` refuses it.
        let parts = split(&[]);
        let (meta, sections) = read_container(&parts.base).unwrap();
        let broken: Vec<(&str, &[u8])> = sections
            .iter()
            .map(|s| {
                let data: &[u8] = if s.name == section::FREQ {
                    b"xyz"
                } else {
                    &s.data
                };
                (s.name.as_str(), data)
            })
            .collect();
        assert!(matches!(
            Pack::assemble(&write_container(&meta, &broken), &parts.layer),
            Err(PackError::Malformed(section::FREQ))
        ));
    }
}
