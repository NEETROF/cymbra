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

//! The offline pack builder (spec `lingua-data-packs`).
//!
//! Assembles a versioned `pack.lingua` from the tables the pipeline derives
//! from ESDB and kaikki (form→lemma), wordfreq (ranks) and kaikki (French glosses):
//! builds the FST, quantises ranks, compresses the offset-indexed glosses
//! with zstd, keys the multi-word expressions through the core's own
//! lemmatiser, files each form's grammar under its dictionary form, embeds the
//! metadata and NOTICE, and enforces the licence denylist and the size budget. Native-only — the reader that consumes the
//! output lives in `lingua-core` and stays WASM-clean.
//!
//! The same pass also writes the pack as two parts (split-lingua-pack-format): a *base*, its
//! studied language's sections, and a *layer*, its pair's, which the core assembles at load into
//! the whole pack's sections, byte for byte ([`build_split`]).

mod lexical;
pub mod licence;
pub mod measure;
pub mod tables;
mod tags;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::lemmatize::lemmatize;
use lingua_core::analysis::lexicon::{FstLexicon, Lexicon, build_lexicon_blobs};
use lingua_core::analysis::pipeline::headword_reading;
use lingua_core::engine::{expression_piece, reading_expression_key};
use lingua_core::knowledge::level::CefrLevel;
use lingua_core::knowledge::profile::NativeLanguage;
use lingua_core::packs::format::{read_container, write_container};
use lingua_core::packs::grammar::{
    FormEdit, ParadigmEntry, SenseRun, Tag, encode_indexed, encode_paradigm, encode_runs,
    encode_tag_pool,
};
use lingua_core::packs::meta::{BaseMeta, LayerMeta, PackMeta, Part};
use lingua_core::packs::pack::section;
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// The maximum size, in bytes, of a pack embedded in the extension (design
/// D4). Overshooting fails the build; what to take from first is fixed by the
/// size-budget requirement and reported by [`BuildError::OverBudget`].
pub const MAX_PACK_BYTES: usize = 5 * 1024 * 1024;

/// A pack's build manifest, alongside its derived tables on disk.
#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    /// The pair + versions + declared licences.
    pub meta: PackMeta,
    /// The sources used, for the licence guard.
    pub sources: Vec<licence::Source>,
}

/// Everything the builder needs to assemble one pair's pack.
pub struct PackInputs {
    /// The pair + versions + declared licences.
    pub meta: PackMeta,
    /// Inflected form → lemma pairs (ESDB-derived, completed by kaikki's form links).
    pub form_lemma: Vec<(String, String)>,
    /// Lemmas that carry a frequency rank (wordfreq-derived); 1 = commonest.
    pub ranks: Vec<(String, u32)>,
    /// Lemma → native-language gloss (kaikki-derived).
    pub glosses: Vec<(String, String)>,
    /// Lemma → CEFR level (CEFR-J A1–B2 + Octanove C1–C2), when the pair has
    /// licence-clean CEFR data. Empty otherwise (no level table is emitted).
    pub levels: Vec<(String, CefrLevel)>,
    /// Multi-word headword → native-language gloss (kaikki-derived), as the
    /// source spells it: `breaking point`, `gave up`. The builder is what keys
    /// them, because only it holds the lexicon the cascade needs. Empty when
    /// the pair's sources hold no expression (no expression table is emitted).
    pub expressions: Vec<(String, String)>,
    /// The readings of inflected forms (`grammar.tsv`, ESDB- and
    /// kaikki-derived). Empty when the pair's sources describe no grammar (no
    /// grammar table is emitted).
    pub readings: Vec<GrammarReading>,
    /// The part of speech of each run of senses of a gloss (`senses.tsv`,
    /// kaikki-derived).
    pub senses: Vec<GlossRuns>,
    /// The full attribution NOTICE text.
    pub notice: String,
    /// The sources actually used, for the licence guard.
    pub sources: Vec<licence::Source>,
    /// The pack's dictionary words (`lexical.tsv`, one lemma per line), when they are not
    /// the lemmas it glosses: a pack glossed in another native language than its studied
    /// language's reference pack names that pack's glossed lemmas
    /// (add-lingua-pack-lexical-layer D1, D3). `None` reads the glossed lemmas as the
    /// dictionary words, and so does a list that names exactly them: no lexical table is
    /// written either way.
    pub lexical: Option<Vec<String>>,
    /// The studied language's pinned tag pool (`tags.tsv`, one Universal Dependencies tag
    /// per line, in its own order): the pool's first tags, so a form's readings are stored
    /// alike whatever tags the senses carry (D4). `None` keeps a single sorted pool.
    pub tag_pool: Option<Vec<String>>,
}

/// A gloss's runs, as `senses.tsv` states them: the dictionary form, then
/// `(tag, senses)` for each run of consecutive senses, in gloss order.
pub type GlossRuns = (String, Vec<(String, u8)>);

/// One reading, as `grammar.tsv` states it: `form<TAB>lemma<TAB>tag<TAB>other|-`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarReading {
    /// The written form, lowercase.
    pub form: String,
    /// The dictionary form it is a reading of.
    pub lemma: String,
    /// The Universal Dependencies tag, as the reducer writes it.
    pub tag: String,
    /// Whether the reducer finds the relation believable, so that the pack may
    /// name `lemma` as another dictionary form of a form the analysis reads as
    /// something else (`add-lingua-word-grammar`, design D2).
    pub other: bool,
}

/// Why a build was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum BuildError {
    /// A source carries a licence on the denylist (GPL/AGPL/NC).
    DeniedLicence(String),
    /// The NOTICE is empty or omits a source's attribution.
    NoticeIncomplete(String),
    /// The FST could not be built from the form→lemma pairs.
    Fst(String),
    /// A grammar table is outside the vocabulary or disagrees with the glosses.
    Grammar(String),
    /// The pack writes a lexical table, and a dictionary word or a glossed lemma is
    /// neither the lemma of a form nor a ranked lemma: no native language's glosses may
    /// add a lemma to the studied language's lexicon (add-lingua-pack-lexical-layer D3).
    Lexical(String),
    /// The forms table lists one form with two lemmas or more: the core reads a form as
    /// one lemma (M8), and which one would be the byte-wise first, which no table decides
    /// (fix-lingua-lemma-lookup D4). `more` counts the other forms listed so.
    FormWithTwoLemmas {
        form: String,
        lemmas: Vec<String>,
        more: usize,
    },
    /// A lemma of the pack — of the forms table, the ranks or the glosses — whose own
    /// spelling the forms table reads as another lemma (`venue` as *venir*): what the pack
    /// files under it would never be read by its spelling, and its spelling would read the
    /// other word's (fix-lingua-lemma-lookup D4). `more` counts the other lemmas read so.
    LemmaReadsAsAnother {
        lemma: String,
        reads_as: String,
        more: usize,
    },
    /// A form or a lemma holding a capital that is no acronym of French's table
    /// (keep-lingua-french-acronyms-in-capitals D2): only a pack studying French holds
    /// one, written wholly in capitals, two letters or more, and its own lemma (`CA` →
    /// `CA`). Every other lookup of the core lowercases first, so any other capital
    /// would be a form no reader reaches — a stray capital is an error.
    StrayCapital(String),
    /// The pack studies a language the core has no analyser for: no core
    /// could load it (`Pack::load` refuses it too).
    UnknownLanguage(String),
    /// The pack is glossed in a native language the core does not know: no core
    /// could load it (`Pack::load` refuses it too, generalise-lingua-native-language D1).
    UnknownNative(String),
    /// The pack is glossed in the language it studies, which no core loads either.
    NativeStudied(String),
    /// The assembled pack exceeds [`MAX_PACK_BYTES`].
    OverBudget {
        size: usize,
        budget: usize,
        /// What the operator must take from, in the arbitration order the
        /// size-budget requirement fixes: the expression table when the pack
        /// carries one, gloss coverage otherwise — never the FST or the
        /// frequencies, which every page analysis depends on.
        reduce: &'static str,
    },
    /// A base and a layer are built only from inputs carrying the studied language's pinned tag
    /// pool, [`TAG_POOL_TABLE`] (split-lingua-pack-format D5): without one the pool is a single
    /// sorted union of the readings' and the senses' tags, which no base and layer reproduce.
    NoPinnedPool,
    /// The base would depend on the pair it is built with (split-lingua-pack-format D6): two
    /// pairs of one language would build two bases. Names the cause.
    BaseFollowsPair(String),
    /// A base or a layer exceeds [`MAX_PACK_BYTES`] (split-lingua-pack-format D8): neither part
    /// is written.
    PartOverBudget {
        part: Part,
        size: usize,
        budget: usize,
        /// What the part carries to reduce: a layer's expression table when it has one, else
        /// its glossed lemmas; a base's readings. `None` for a base carrying no readings, whose
        /// forms and frequencies every analysis depends on.
        reduce: Option<&'static str>,
    },
}

/// The remedies [`BuildError::OverBudget`] names, in arbitration order.
const REDUCE_EXPRESSIONS: &str = "the expression table (longest entries, then rarest)";
const REDUCE_READINGS: &str = "the grammar readings (those of the rarest dictionary forms first)";
const REDUCE_GLOSSES: &str = "glossed lemmas";

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::DeniedLicence(s) => {
                write!(f, "source {s:?} has a denied licence (GPL/AGPL/NC)")
            }
            BuildError::NoticeIncomplete(s) => {
                write!(f, "NOTICE is missing the attribution for {s:?}")
            }
            BuildError::Fst(e) => write!(f, "could not build the forms FST: {e}"),
            BuildError::Grammar(e) => write!(f, "grammar tables: {e}"),
            BuildError::Lexical(e) => write!(f, "lexical table: {e}"),
            BuildError::FormWithTwoLemmas { form, lemmas, more } => {
                let lemmas: Vec<String> = lemmas.iter().map(|l| format!("{l:?}")).collect();
                write!(
                    f,
                    "the forms table lists the form {form:?} with {} lemmas: {}{}",
                    lemmas.len(),
                    lemmas.join(", "),
                    and_more(*more, "form")
                )
            }
            BuildError::LemmaReadsAsAnother {
                lemma,
                reads_as,
                more,
            } => write!(
                f,
                "the lemma {lemma:?} reads as {reads_as:?}: the forms table lists it as a form of \
                 {reads_as:?}{}",
                and_more(*more, "lemma")
            ),
            BuildError::StrayCapital(form) => write!(
                f,
                "the form {form:?} holds a capital: only a pack studying French holds one, an \
                 acronym written wholly in capitals, two letters or more, that is its own lemma"
            ),
            BuildError::UnknownLanguage(tag) => write!(
                f,
                "the pack studies {tag:?}, a language the core cannot analyse"
            ),
            BuildError::UnknownNative(tag) => write!(
                f,
                "the pack is glossed in {tag:?}, a native language the core does not know"
            ),
            BuildError::NativeStudied(tag) => {
                write!(f, "the pack studies {tag:?} and is glossed in it too")
            }
            BuildError::OverBudget {
                size,
                budget,
                reduce,
            } => write!(
                f,
                "pack is {size} bytes, over the {budget}-byte budget; reduce {reduce}"
            ),
            BuildError::NoPinnedPool => write!(
                f,
                "a base and a layer are built only from inputs carrying the studied language's \
                 pinned tag pool, {TAG_POOL_TABLE}; these carry none"
            ),
            BuildError::BaseFollowsPair(why) => write!(
                f,
                "the base would depend on the pair: {why}; a base is built from its studied \
                 language's tables alone, the same for every pair of the language"
            ),
            BuildError::PartOverBudget {
                part,
                size,
                budget,
                reduce: Some(reduce),
            } => write!(
                f,
                "the {part} is {size} bytes, over the {budget}-byte budget; reduce {reduce}"
            ),
            BuildError::PartOverBudget {
                part,
                size,
                budget,
                reduce: None,
            } => write!(
                f,
                "the {part} is {size} bytes, over the {budget}-byte budget, and carries nothing \
                 to reduce: its forms and frequencies are what every analysis depends on"
            ),
        }
    }
}

impl std::error::Error for BuildError {}

/// Whether `word` is written as an acronym row of French's table is: two letters or
/// more, none of them lowercase (keep-lingua-french-acronyms-in-capitals D2).
fn is_acronym(word: &str) -> bool {
    let mut letters = word.chars().filter(|c| c.is_alphabetic());
    letters.clone().count() >= 2 && !letters.any(char::is_lowercase)
}

/// Refuses a capital outside an acronym row (keep-lingua-french-acronyms-in-capitals D2):
/// a form or a lemma holding one is accepted only in a pack studying French, written as an
/// acronym is, and its own lemma — a pair's form and lemma the same string, a ranked or
/// glossed lemma being its own form. The first string holding a stray capital is named.
fn check_capitals<'a>(
    studied: StudiedLanguage,
    pairs: &[(&'a str, &'a str)],
    lemmas: impl IntoIterator<Item = &'a str>,
) -> Result<(), BuildError> {
    let capital = |word: &str| word.chars().any(char::is_uppercase);
    let acronym = |word: &str| studied == StudiedLanguage::French && is_acronym(word);
    for (form, lemma) in pairs {
        if (capital(form) || capital(lemma)) && !(form == lemma && acronym(form)) {
            let named = if capital(form) { form } else { lemma };
            return Err(BuildError::StrayCapital((*named).to_owned()));
        }
    }
    match lemmas
        .into_iter()
        .find(|lemma| capital(lemma) && !acronym(lemma))
    {
        Some(lemma) => Err(BuildError::StrayCapital(lemma.to_owned())),
        None => Ok(()),
    }
}

/// `, and 3 more forms` when a refusal stands for others: empty when it names them all.
fn and_more(more: usize, what: &str) -> String {
    match more {
        0 => String::new(),
        1 => format!(", and 1 more {what}"),
        n => format!(", and {n} more {what}s"),
    }
}

/// Loads a pack's inputs from one directory holding both sides: `manifest.json`,
/// `forms.tsv`, `freq.tsv`, `gloss.tsv` and `NOTICE`, and the optional tables
/// beside them — the `testdata/` fixtures and the tests' scratch folders.
pub fn inputs_from_dir(dir: &Path) -> std::io::Result<PackInputs> {
    inputs_from_dirs(dir, dir)
}

/// The tables of a pair's studied side (split-lingua-pack-tables-by-language, M24): they
/// belong to its studied language and are kept once per studied language, in
/// `tables/<studied>/`, whatever native language a pair is glossed in.
pub const STUDIED_SIDE: [&str; 6] = [
    "forms.tsv",
    "freq.tsv",
    "level.tsv",
    "grammar.tsv",
    TAG_POOL_TABLE,
    LEXICAL_TABLE,
];

/// The tables of a pair's native side, kept per pair in `tables/<pair>/`.
pub const PAIR_SIDE: [&str; 5] = [
    "manifest.json",
    "gloss.tsv",
    "mwe.tsv",
    "senses.tsv",
    "NOTICE",
];

/// A committed pair's studied language, as its folder names it: `es-fr` studies `es`.
pub fn studied_of(pair: &str) -> &str {
    pair.split_once('-').map_or(pair, |(studied, _)| studied)
}

/// Loads a committed pair's inputs from the tables root (`scripts/lingua-data/tables`):
/// its studied side from `<root>/<studied>/`, the studied language being the first side of the
/// pair's name, and its native side from `<root>/<pair>/` (split-lingua-pack-tables-by-language).
pub fn inputs_from_tables(root: &Path, pair: &str) -> std::io::Result<PackInputs> {
    inputs_from_dirs(&root.join(studied_of(pair)), &root.join(pair))
}

/// Loads a pack's inputs from two directories: the studied side ([`STUDIED_SIDE`]) from
/// `studied`, the native side ([`PAIR_SIDE`]) from `pair`. The same directory twice reads a
/// folder that holds both, as the `testdata/` fixtures do.
///
/// Two directories must each hold their own side alone: a table on the wrong side is refused,
/// by path, rather than read from one side and silently left to drift on the other. The
/// studied directory is named after the language the pair's manifest studies.
pub fn inputs_from_dirs(studied: &Path, pair: &Path) -> std::io::Result<PackInputs> {
    let invalid = |message: String| std::io::Error::new(std::io::ErrorKind::InvalidData, message);
    let two = studied != pair;
    if two {
        if !studied.is_dir() {
            return Err(invalid(format!(
                "{} holds no studied tables: {} is built from that folder too",
                studied.display(),
                pair.display()
            )));
        }
        let misplaced = STUDIED_SIDE
            .iter()
            .map(|t| pair.join(t))
            .chain(PAIR_SIDE.iter().map(|t| studied.join(t)))
            .find(|path| path.exists());
        if let Some(path) = misplaced {
            // The reference pair's reduction is what writes the studied folder, when the
            // folder records one (`studied.json`); the message names it, so the person knows
            // which reduction the misplaced table belongs to.
            let written_by = tables::reference_of(studied)
                .map(|reference| format!(" (written by {reference}'s reduction)"))
                .unwrap_or_default();
            return Err(invalid(format!(
                "{} is on the wrong side: the studied side is read from {} alone{}, the \
                 native side from {} alone",
                path.display(),
                studied.display(),
                written_by,
                pair.display()
            )));
        }
    }
    // A required file that is missing names its path: a pair is read from two folders, so
    // "No such file or directory" alone would not say which side lacks it.
    let read_at = |path: std::path::PathBuf| {
        std::fs::read_to_string(&path)
            .map_err(|e| std::io::Error::new(e.kind(), format!("{}: {e}", path.display())))
    };
    let read = |name: &str| read_at(pair.join(name));
    let read_studied = |name: &str| read_at(studied.join(name));
    let manifest: Manifest = serde_json::from_str(&read("manifest.json")?)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if two {
        let folder = |dir: &Path| {
            dir.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        let language = folder(studied);
        if manifest.meta.studied != language {
            return Err(invalid(format!(
                "{}/manifest.json studies {:?}, but {} is built from the studied tables of {:?} \
                 ({}): a pair's folder is named <studied>-<native>",
                folder(pair),
                manifest.meta.studied,
                folder(pair),
                language,
                studied.display()
            )));
        }
    }
    Ok(PackInputs {
        meta: manifest.meta,
        form_lemma: tsv_pairs(&read_studied("forms.tsv")?),
        ranks: tsv_pairs(&read_studied("freq.tsv")?)
            .into_iter()
            .filter_map(|(lemma, rank)| rank.parse::<u32>().ok().map(|r| (lemma, r)))
            .collect(),
        glosses: tsv_pairs(&read("gloss.tsv")?),
        levels: read_levels(studied)?,
        expressions: read_expressions(pair)?,
        readings: read_readings(studied)?,
        senses: read_senses(pair)?,
        notice: read("NOTICE")?,
        sources: manifest.sources,
        lexical: read_lines(studied, LEXICAL_TABLE)?,
        tag_pool: read_lines(studied, TAG_POOL_TABLE)?,
    })
}

/// The file naming a studied language's dictionary words (add-lingua-pack-lexical-layer D1):
/// its reference pair's glossed lemmas, kept in `tables/<studied>/` beside the studied tables
/// (split-lingua-pack-tables-by-language). No reducer writes it: `pack_sources.py split` derives
/// it when the reference pair is reduced. A pack whose dictionary words are its glossed lemmas
/// carries no lexical section.
pub const LEXICAL_TABLE: &str = "lexical.tsv";

/// The file pinning a studied language's tag pool (add-lingua-pack-lexical-layer D4). No
/// reducer writes it.
pub const TAG_POOL_TABLE: &str = "tags.tsv";

/// Reads an optional one-column table: its non-empty lines, trimmed, in file order, or
/// `None` when the pair has no such file.
fn read_lines(dir: &Path, name: &str) -> std::io::Result<Option<Vec<String>>> {
    Ok(read_optional(dir, name)?.map(|text| {
        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect()
    }))
}

/// Reads the optional `level.tsv` (`lemma<TAB>A1..C2`). A pair without CEFR data
/// simply has no file → no levels. Unrecognised level labels are skipped.
fn read_levels(dir: &Path) -> std::io::Result<Vec<(String, CefrLevel)>> {
    match std::fs::read_to_string(dir.join("level.tsv")) {
        Ok(text) => Ok(tsv_pairs(&text)
            .into_iter()
            .filter_map(|(lemma, label)| CefrLevel::from_label(&label).map(|lvl| (lemma, lvl)))
            .collect()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

/// Reads the optional `mwe.tsv` (`headword<TAB>gloss`). A pair whose sources
/// hold no multi-word entry simply has no file → no expressions, exactly as for
/// `level.tsv`.
fn read_expressions(dir: &Path) -> std::io::Result<Vec<(String, String)>> {
    match std::fs::read_to_string(dir.join("mwe.tsv")) {
        Ok(text) => Ok(tsv_pairs(&text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

/// Reads an optional table: its text, or `None` when the pair has no such file.
fn read_optional(dir: &Path, name: &str) -> std::io::Result<Option<String>> {
    match std::fs::read_to_string(dir.join(name)) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn invalid(name: &str, line: usize, what: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("{name} line {line}: {what}"),
    )
}

/// Reads the optional `grammar.tsv` (`form<TAB>lemma<TAB>tag<TAB>other|-`).
/// A pair without grammar has no file → no readings. A malformed line fails
/// the read: the reducer writes this file, so a bad line is a bug to see.
fn read_readings(dir: &Path) -> std::io::Result<Vec<GrammarReading>> {
    let Some(text) = read_optional(dir, "grammar.tsv")? else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate().filter(|(_, l)| !l.is_empty()) {
        let fields: Vec<&str> = line.split('\t').collect();
        let [form, lemma, tag, other] = fields[..] else {
            return Err(invalid("grammar.tsv", i + 1, "expected 4 fields"));
        };
        let other = match other {
            "other" => true,
            "-" => false,
            _ => {
                return Err(invalid(
                    "grammar.tsv",
                    i + 1,
                    "the 4th field is `other` or `-`",
                ));
            }
        };
        out.push(GrammarReading {
            form: form.to_owned(),
            lemma: lemma.to_owned(),
            tag: tag.to_owned(),
            other,
        });
    }
    Ok(out)
}

/// Reads the optional `senses.tsv` (`lemma<TAB>tag:count[<TAB>tag:count…]`).
fn read_senses(dir: &Path) -> std::io::Result<Vec<GlossRuns>> {
    let Some(text) = read_optional(dir, "senses.tsv")? else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate().filter(|(_, l)| !l.is_empty()) {
        let mut fields = line.split('\t');
        let lemma = fields.next().unwrap_or_default().to_owned();
        let mut runs = Vec::new();
        for field in fields {
            let parsed = field
                .rsplit_once(':')
                .and_then(|(tag, count)| Some((tag.to_owned(), count.parse::<u8>().ok()?)));
            let Some(run) = parsed else {
                return Err(invalid("senses.tsv", i + 1, "a run is `tag:count`"));
            };
            runs.push(run);
        }
        if lemma.is_empty() || runs.is_empty() {
            return Err(invalid("senses.tsv", i + 1, "expected a word and its runs"));
        }
        out.push((lemma, runs));
    }
    Ok(out)
}

/// Parses `a<TAB>b` lines, skipping blank lines and lines without a tab.
pub fn tsv_pairs(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(a, b)| (a.trim().to_owned(), b.trim().to_owned()))
        .filter(|(a, _)| !a.is_empty())
        .collect()
}

/// Builds a `pack.lingua` from its inputs, enforcing licence hygiene and the
/// size budget. Deterministic: identical inputs produce byte-identical output.
pub fn build_pack(inputs: &PackInputs) -> Result<Vec<u8>, BuildError> {
    whole_pack(inputs, &build_sections(inputs)?)
}

/// A pack built as its two parts (split-lingua-pack-format D2): its studied language's base and
/// its pair's layer, which the core assembles into the whole pack's sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPack {
    /// The studied language's sections, the same whichever pair of the language built it.
    pub base: Vec<u8>,
    /// The pair's sections, built against `base`.
    pub layer: Vec<u8>,
}

/// Builds a pack as a base and a layer (split-lingua-pack-format D2, D4, D5, D6, D8), from the
/// sections [`build_pack`] writes, so the parts cannot drift from the whole.
///
/// The base holds the studied language's sections and the pool's first two blocks — the pinned
/// pool and the readings' tags outside it —; the layer the pair's, the pool's sense-only block as
/// `senses.tags`, the pair's whole NOTICE, and the `base_version` of its base. The ids the
/// layer's runs store are those of the whole pool, so the assembled pool is the whole pack's.
/// Refused, naming [`TAG_POOL_TABLE`], for inputs without a pinned pool; refused, naming why,
/// when the base would depend on the pair ([`BuildError::BaseFollowsPair`]); refused, writing
/// neither part, when either exceeds [`MAX_PACK_BYTES`], naming the part and what it carries to
/// reduce.
pub fn build_split(inputs: &PackInputs) -> Result<SplitPack, BuildError> {
    split_pack(inputs, &split_sections(inputs)?)
}

/// Builds a pack whole and as a base and a layer, from one pass over its inputs
/// (split-lingua-pack-format D8): the whole pack [`build_pack`] writes and the parts
/// [`build_split`] writes, laid out from the same sections. Refused as either refuses.
pub fn build_whole_and_split(inputs: &PackInputs) -> Result<(Vec<u8>, SplitPack), BuildError> {
    let built = split_sections(inputs)?;
    Ok((whole_pack(inputs, &built)?, split_pack(inputs, &built)?))
}

/// The sections of inputs a split accepts (split-lingua-pack-format D5, D6): they pin their
/// studied language's pool, and nothing of the pair reaches the base. The base's lemmas are the
/// studied tables' only when every glossed lemma is the lemma of a form or a ranked lemma — the
/// builder adds a glossed lemma to the lexicon otherwise —, and the base carries its dictionary
/// words only when they are not the pair's glossed lemmas — the builder leaves the lexical table
/// out otherwise, and the reader then takes the layer's glosses for them. Either would make two
/// pairs of one language build two bases, which the split refuses by construction rather than
/// leave to the committed-tables checks.
fn split_sections(inputs: &PackInputs) -> Result<Built, BuildError> {
    if inputs.tag_pool.is_none() {
        return Err(BuildError::NoPinnedPool);
    }
    let studied: BTreeSet<&str> = inputs
        .form_lemma
        .iter()
        .map(|(_, lemma)| lemma.as_str())
        .chain(inputs.ranks.iter().map(|(lemma, _)| lemma.as_str()))
        .collect();
    if let Some((lemma, _)) = inputs
        .glosses
        .iter()
        .find(|(lemma, _)| !studied.contains(lemma.as_str()))
    {
        return Err(BuildError::BaseFollowsPair(format!(
            "{lemma:?} is glossed but is neither the lemma of a form nor a ranked lemma, so the \
             pair's glosses would add it to the base's lemmas"
        )));
    }
    let built = build_sections(inputs)?;
    if inputs.lexical.is_some() && built.lexical.is_empty() {
        return Err(BuildError::BaseFollowsPair(format!(
            "the dictionary words ({LEXICAL_TABLE}) are the lemmas the pair glosses, so the base \
             would carry no lexical table and take the layer's glosses for them"
        )));
    }
    Ok(built)
}

/// The whole pack: every section in [`LAYOUT`]'s order, held to [`MAX_PACK_BYTES`].
fn whole_pack(inputs: &PackInputs, built: &Built) -> Result<Vec<u8>, BuildError> {
    let meta_json = serde_json::to_vec(&inputs.meta).expect("PackMeta serialises");
    let pack = write_container(&meta_json, &built.sections(None));
    if pack.len() > MAX_PACK_BYTES {
        return Err(BuildError::OverBudget {
            size: pack.len(),
            budget: MAX_PACK_BYTES,
            reduce: if !built.expr.is_empty() {
                REDUCE_EXPRESSIONS
            } else if !built.paradigms.is_empty() {
                REDUCE_READINGS
            } else {
                REDUCE_GLOSSES
            },
        });
    }
    Ok(pack)
}

/// The base and the layer: each part's sections in [`LAYOUT`]'s order, the layer naming the
/// base's digest, each held to [`MAX_PACK_BYTES`] with the remedy it carries.
fn split_pack(inputs: &PackInputs, built: &Built) -> Result<SplitPack, BuildError> {
    let base = built.sections(Some(Part::Base));
    // The digest leaves the metadata out, so the base's, written with it, has the same.
    let version = base_version(&write_container(b"", &base)).expect("a container");
    let base_meta = BaseMeta {
        part: Part::Base,
        studied: inputs.meta.studied.clone(),
        analyzer_version: inputs.meta.analyzer_version.clone(),
        base_version: version.clone(),
        levels_estimated: inputs.meta.levels_estimated,
    };
    let layer_meta = LayerMeta {
        part: Part::Layer,
        studied: inputs.meta.studied.clone(),
        native: inputs.meta.native.clone(),
        pack_version: inputs.meta.pack_version.clone(),
        base_version: version,
        licences: inputs.meta.licences.clone(),
    };
    let split = SplitPack {
        base: write_container(
            &serde_json::to_vec(&base_meta).expect("BaseMeta serialises"),
            &base,
        ),
        layer: write_container(
            &serde_json::to_vec(&layer_meta).expect("LayerMeta serialises"),
            &built.sections(Some(Part::Layer)),
        ),
    };
    part_budget(
        Part::Base,
        split.base.len(),
        (!built.paradigms.is_empty()).then_some(REDUCE_READINGS),
    )?;
    part_budget(
        Part::Layer,
        split.layer.len(),
        Some(if built.expr.is_empty() {
            REDUCE_GLOSSES
        } else {
            REDUCE_EXPRESSIONS
        }),
    )?;
    Ok(split)
}

/// A part held to [`MAX_PACK_BYTES`] (split-lingua-pack-format D8), refused naming it and what
/// it carries to reduce when it exceeds it.
fn part_budget(part: Part, size: usize, reduce: Option<&'static str>) -> Result<(), BuildError> {
    if size > MAX_PACK_BYTES {
        return Err(BuildError::PartOverBudget {
            part,
            size,
            budget: MAX_PACK_BYTES,
            reduce,
        });
    }
    Ok(())
}

/// A base's `base_version` (split-lingua-pack-format D4): the sha256, in lowercase hex, of its
/// container's bytes from its section count to its end. The metadata is left out, so the
/// digest is written into it, and any reader holding the file recomputes it. `None` when the
/// bytes are no container.
pub fn base_version(container: &[u8]) -> Option<String> {
    read_container(container).ok()?;
    let meta_len = u32::from_le_bytes(container.get(10..14)?.try_into().ok()?) as usize;
    let sections = container.get(14usize.checked_add(meta_len)?..)?;
    Some(
        Sha256::digest(sections)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )
}

/// Every section a pack may hold, in the order a whole pack lays them out — the order every pin
/// records —, with whether it is written when empty (split-lingua-pack-format D2, D8). `senses.tags`,
/// which no whole pack holds, sits beside the pool it ends. The one table that decides each
/// container: a whole pack writes every row but `senses.tags`; a part writes the rows
/// [`section::part_of`] gives it, in this order.
const LAYOUT: [(&str, bool); 15] = [
    (section::FORMS, true),
    (section::LEMMAS, true),
    (section::FREQ, true),
    (section::LEVELS, false),
    (section::LEXICAL, false),
    (section::EXPR, false),
    (section::EXPR_ZST, false),
    (section::EXPR_NAMES_ZST, false),
    (section::EXPR_OTHERS, false),
    (section::TAGS, false),
    (section::SENSES_TAGS, false),
    (section::PARADIGMS_ZST, false),
    (section::SENSES_ZST, false),
    (section::GLOSS_ZST, true),
    (section::NOTICE, true),
];

/// Every section of a pack, built once from its inputs (split-lingua-pack-format D8): what
/// [`build_pack`] writes as one container and [`build_split`] as two. An empty blob is a section
/// not written, unless [`LAYOUT`] writes it anyway.
struct Built {
    forms: Vec<u8>,
    lemmas: String,
    freq: Vec<u8>,
    levels: Vec<u8>,
    lexical: Vec<u8>,
    expr: Vec<u8>,
    expr_zst: Vec<u8>,
    expr_names: Vec<u8>,
    expr_others: Vec<u8>,
    /// The whole pool, a whole pack's `tags`.
    tags: Vec<u8>,
    /// The pool's studied blocks, a base's `tags` (the whole pool without a pin, which no split
    /// accepts).
    base_tags: Vec<u8>,
    /// The pool's sense-only block, a layer's `senses.tags`.
    senses_tags: Vec<u8>,
    paradigms: Vec<u8>,
    senses: Vec<u8>,
    gloss: Vec<u8>,
    notice: String,
}

impl Built {
    /// The bytes of the section `name` in a whole pack (`part` `None`) or in a part: the pool is
    /// the whole pack's in a whole pack and its studied blocks in a base.
    fn section(&self, name: &str, part: Option<Part>) -> &[u8] {
        match name {
            section::FORMS => &self.forms,
            section::LEMMAS => self.lemmas.as_bytes(),
            section::FREQ => &self.freq,
            section::LEVELS => &self.levels,
            section::LEXICAL => &self.lexical,
            section::EXPR => &self.expr,
            section::EXPR_ZST => &self.expr_zst,
            section::EXPR_NAMES_ZST => &self.expr_names,
            section::EXPR_OTHERS => &self.expr_others,
            section::TAGS if part.is_none() => &self.tags,
            section::TAGS => &self.base_tags,
            section::SENSES_TAGS => &self.senses_tags,
            section::PARADIGMS_ZST => &self.paradigms,
            section::SENSES_ZST => &self.senses,
            section::GLOSS_ZST => &self.gloss,
            section::NOTICE => self.notice.as_bytes(),
            _ => &[],
        }
    }

    /// The sections of a whole pack (`part` `None`) or of a part, in [`LAYOUT`]'s order.
    fn sections(&self, part: Option<Part>) -> Vec<(&'static str, &[u8])> {
        LAYOUT
            .iter()
            .filter(|(name, _)| match part {
                None => *name != section::SENSES_TAGS,
                Some(part) => section::part_of(name) == Some(part),
            })
            .map(|&(name, written)| (name, written, self.section(name, part)))
            .filter(|(_, written, data)| *written || !data.is_empty())
            .map(|(name, _, data)| (name, data))
            .collect()
    }
}

/// Builds every section of a pack from its inputs, enforcing licence hygiene and the lexicon's
/// rules; the budget is the caller's, whole pack or part.
fn build_sections(inputs: &PackInputs) -> Result<Built, BuildError> {
    // Licence hygiene first: refuse a denied source, and require the NOTICE to
    // name every source used.
    for source in &inputs.sources {
        if licence::is_denied(source.licence) {
            return Err(BuildError::DeniedLicence(source.name.clone()));
        }
        if !inputs.notice.contains(&source.name) {
            return Err(BuildError::NoticeIncomplete(source.name.clone()));
        }
    }
    // The studied language picks the analyser the build reads forms with: the
    // one every analysis of this pack will run (generalise-lingua-analysis-by-language).
    let studied = StudiedLanguage::from_tag(&inputs.meta.studied)
        .ok_or_else(|| BuildError::UnknownLanguage(inputs.meta.studied.clone()))?;
    // The native language its glosses are written in: refused as the core refuses it at load, so
    // no build produces a pack no core loads (generalise-lingua-native-language D1).
    let native = NativeLanguage::from_tag(&inputs.meta.native)
        .ok_or_else(|| BuildError::UnknownNative(inputs.meta.native.clone()))?;
    if native.studied() == Some(studied) {
        return Err(BuildError::NativeStudied(inputs.meta.native.clone()));
    }

    // FST + lemma pool. Lemmas absent from the form→lemma pairs (e.g. glossed
    // or ranked lemmas with no listed inflection) are added to the pool so
    // their id exists.
    let pairs: Vec<(&str, &str)> = inputs
        .form_lemma
        .iter()
        .map(|(f, l)| (f.as_str(), l.as_str()))
        .collect();
    // One form, one lemma (M8): a form listed with two lemmas would be read as the byte-wise
    // first, which no table decides (fix-lingua-lemma-lookup D4).
    let mut lemmas_of: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (form, lemma) in &pairs {
        lemmas_of.entry(form).or_default().insert(lemma);
    }
    let mut ambiguous = lemmas_of.iter().filter(|(_, lemmas)| lemmas.len() > 1);
    if let Some((form, lemmas)) = ambiguous.next() {
        return Err(BuildError::FormWithTwoLemmas {
            form: (*form).to_owned(),
            lemmas: lemmas.iter().map(|l| (*l).to_owned()).collect(),
            more: ambiguous.count(),
        });
    }
    let mut extra: Vec<&str> = inputs
        .ranks
        .iter()
        .map(|(l, _)| l.as_str())
        .chain(inputs.glosses.iter().map(|(l, _)| l.as_str()))
        .collect();
    extra.sort_unstable();
    extra.dedup();
    // A capital only in an acronym row of a pack studying French, its own lemma
    // (keep-lingua-french-acronyms-in-capitals D2).
    check_capitals(studied, &pairs, extra.iter().copied())?;
    let (forms, pool) =
        build_lexicon_blobs(&pairs, &extra).map_err(|e| BuildError::Fst(e.to_string()))?;
    let lex = FstLexicon::from_slices(forms.as_slice(), &pool)
        .map_err(|e| BuildError::Fst(e.to_string()))?;
    // Every lemma of the pack reads as itself (fix-lingua-lemma-lookup D4): a lemma whose own
    // spelling the forms read as another would keep what is filed under it out of the reader's
    // reach, its spelling reading the other word's estimates. Checked on the lexicon written,
    // where a form the pairs map reads as their lemma whatever the byte order (D5). After it,
    // a lemma's own place (`lemma_id`) is the id its spelling reads (`id_of`).
    let mut elsewhere = pool.lines().filter_map(|lemma| {
        let reads_as = lex.lemma_of(lemma)?;
        (reads_as != lemma).then_some((lemma, reads_as))
    });
    if let Some((lemma, reads_as)) = elsewhere.next() {
        return Err(BuildError::LemmaReadsAsAnother {
            lemma: lemma.to_owned(),
            reads_as: reads_as.to_owned(),
            more: elsewhere.count(),
        });
    }

    // Every lemma-keyed section is filed at the lemma's own place in the lemma list, the id
    // the core reads it by (fix-lingua-lemma-lookup D4).
    // Ranks by lemma id (0 = unranked).
    let mut freq = vec![0u32; lex.lemma_count()];
    for (lemma, rank) in &inputs.ranks {
        if let Some(id) = lex.lemma_id(lemma) {
            freq[id as usize] = *rank;
        }
    }
    let mut freq_bytes = Vec::with_capacity(freq.len() * 4);
    for r in &freq {
        freq_bytes.extend_from_slice(&r.to_le_bytes());
    }

    // Offset-indexed glosses, sorted by lemma id, then zstd-compressed.
    let mut entries: Vec<(u32, &str)> = inputs
        .glosses
        .iter()
        .filter_map(|(l, g)| lex.lemma_id(l).map(|id| (id as u32, g.as_str())))
        .collect();
    entries.sort_unstable_by_key(|(id, _)| *id);
    let gloss_zst = compress_glosses(&entries);

    // The pack's dictionary words, when they are not its glossed lemmas (D1, D3).
    let lexical_bytes = lexical::lexical_table(inputs, &lex, &entries)?.unwrap_or_default();

    // CEFR level code per lemma id (0 = no level). Emitted as an optional
    // section ONLY when the pair has CEFR data, so a level-less pack stays
    // byte-for-byte identical to before this feature. A level written for a
    // string that is no lemma of the pack is filed under no lemma — never under
    // the lemma that string is a form of (fix-lingua-lemma-lookup D4).
    let levels_bytes: Vec<u8> = if inputs.levels.is_empty() {
        Vec::new()
    } else {
        let mut levels = vec![0u8; lex.lemma_count()];
        for (lemma, level) in &inputs.levels {
            if let Some(id) = lex.lemma_id(lemma) {
                levels[id as usize] = level.to_code();
            }
        }
        levels
    };

    // Expression table, keyed by the lexicon the build has just assembled, so a
    // key is exactly what the reader's own cascade makes of the words on the
    // page (design D1). Two sections, like the glosses: an FST of keys → dense
    // ids, and the zstd-compressed glosses those ids index. Emitted ONLY when
    // an entry survives, so a pack for a pair without expressions — or one whose
    // entries the lexicon cannot reach — stays byte-for-byte what it was.
    // French and Spanish are keyed through their reading and named by their
    // headwords (add-lingua-french-expression-keys, add-lingua-spanish-expression-keys);
    // English at its spaces, with no names.
    let read = matches!(studied, StudiedLanguage::French | StudiedLanguage::Spanish);
    let mut keyed: Vec<(String, &str, &str, usize)> = inputs
        .expressions
        .iter()
        .filter_map(|(headword, gloss)| {
            let key = expression_key(headword, studied, &lex)?;
            let written = if read {
                written_pieces(headword, studied, &lex)
            } else {
                0
            };
            Some((key, headword.as_str(), gloss.as_str(), written))
        })
        .collect();
    // Sorted by key for the FST, and within a key by "is the key itself" first:
    // `break point` keeps the key `breaking point` also reaches, so the gloss
    // that survives is the dictionary spelling's (design D2). For French and
    // Spanish, the headword with the most tokens written as their key piece comes
    // next — `boîte à gants` over `boite à gants`, whose `boite` is read `boîte`
    // (add-lingua-french-expression-keys D6), `soy inglés` over `soy inglesa`
    // (add-lingua-spanish-expression-keys D3); English counts none, so its order
    // is as before. Alphabetical order settles the rest, so the winner never
    // depends on the input's order.
    keyed.sort_unstable_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| (a.1 != a.0).cmp(&(b.1 != b.0)))
            .then_with(|| b.3.cmp(&a.3))
            .then_with(|| a.1.cmp(b.1))
    });
    // A pack studying French or Spanish keeps every headword of a key, not only the one that
    // wins it (pick-lingua-expression-written-as-the-page D1, opened to Spanish by
    // pick-lingua-spanish-expression-written-as-the-page D1): each headword after a key's first
    // is an expression of its own, its gloss and its name filed under an id after the keys' —
    // in key order, and in the order above within a key — which no key maps to, so a core that
    // does not read `expr.others` answers every key as before. `ça va` and `ça ira` both read
    // `ça aller`, `hacer tiempo` and `hace tiempo` both `hacer tiempo`: the key answers its
    // winner, and the other is kept beside it. English keeps the first alone, byte for byte.
    let mut others: Vec<(u32, &str, &str)> = Vec::new();
    let mut firsts: Vec<(String, &str, &str, usize)> = Vec::with_capacity(keyed.len());
    for entry in keyed {
        if firsts.last().is_some_and(|first| first.0 == entry.0) {
            if matches!(studied, StudiedLanguage::French | StudiedLanguage::Spanish) {
                others.push(((firsts.len() - 1) as u32, entry.1, entry.2));
            }
        } else {
            firsts.push(entry);
        }
    }
    let keyed = firsts;
    let other_id = |at: usize| (keyed.len() + at) as u32;
    // A French or Spanish expression is named by the headword that won its key,
    // wherever the two differ (add-lingua-french-expression-keys D3,
    // add-lingua-spanish-expression-keys D3): `au revoir` for `à le revoir`,
    // `al menos` for `a el menos`. Same layout as the glosses, by the same ids;
    // written only for a pack studying French or Spanish that names one, so an
    // English pack gains no byte.
    let names: Vec<(u32, &str)> = if read {
        keyed
            .iter()
            .enumerate()
            .filter(|(_, (key, headword, _, _))| key != headword)
            .map(|(id, (_, headword, _, _))| (id as u32, *headword))
            .chain(
                others
                    .iter()
                    .enumerate()
                    .map(|(at, (_, headword, _))| (other_id(at), *headword)),
            )
            .collect()
    } else {
        Vec::new()
    };
    let expr_names = if names.is_empty() {
        Vec::new()
    } else {
        compress_glosses(&names)
    };
    let (expr_fst, expr_zst) = if keyed.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        let mut builder = fst::MapBuilder::memory();
        for (id, (key, _, _, _)) in keyed.iter().enumerate() {
            // Byte-wise sorted and unique is all a `MapBuilder` asks of its
            // input, and the sort above has just made the keys both. Unlike the
            // forms FST, whose pairs come from a file, nothing here can fail.
            builder
                .insert(key, id as u64)
                .expect("keys sorted and unique");
        }
        let entries: Vec<(u32, &str)> = keyed
            .iter()
            .enumerate()
            .map(|(id, (_, _, gloss, _))| (id as u32, *gloss))
            .chain(
                others
                    .iter()
                    .enumerate()
                    .map(|(at, (_, _, gloss))| (other_id(at), *gloss)),
            )
            .collect();
        (
            builder.into_inner().expect("in-memory FST"),
            compress_glosses(&entries),
        )
    };

    // `expr.others`: each key's other headwords, `count u32 | count × (key id u32, other id
    // u32)`, little-endian, sorted by key id and then in the order above — written only where a
    // key has others, so a French or Spanish pack without any, and every English pack, gains
    // no byte (pick-lingua-expression-written-as-the-page D1,
    // pick-lingua-spanish-expression-written-as-the-page D1).
    let expr_others: Vec<u8> = if others.is_empty() {
        Vec::new()
    } else {
        let mut raw = Vec::with_capacity(4 + others.len() * 8);
        raw.extend_from_slice(&(others.len() as u32).to_le_bytes());
        for (at, (key, _, _)) in others.iter().enumerate() {
            raw.extend_from_slice(&key.to_le_bytes());
            raw.extend_from_slice(&other_id(at).to_le_bytes());
        }
        raw
    };

    let grammar = grammar_sections(inputs, studied, &lex)?;
    let (base_tags, senses_tags) = grammar.split_pool();

    Ok(Built {
        forms,
        lemmas: pool,
        freq: freq_bytes,
        levels: levels_bytes,
        lexical: lexical_bytes,
        expr: expr_fst,
        expr_zst,
        expr_names,
        expr_others,
        tags: encode_tag_pool(&grammar.pool),
        base_tags,
        senses_tags,
        paradigms: grammar.paradigms,
        senses: grammar.senses,
        gloss: gloss_zst,
        notice: inputs.notice.clone(),
    })
}

/// The three grammar sections, empty when the inputs carry no grammar: the tag pool, whose
/// first `studied` tags are its studied blocks when the language pins its pool, and the two
/// blobs that index it.
#[derive(Default)]
struct GrammarSections {
    pool: Vec<String>,
    /// [`tags::TagPool::studied`].
    studied: Option<usize>,
    paradigms: Vec<u8>,
    senses: Vec<u8>,
}

impl GrammarSections {
    /// The pool's studied blocks — the base's `tags` — and its sense-only block — the layer's
    /// `senses.tags` —, each encoded as the whole pool is (split-lingua-pack-format D2, D5). Without
    /// a pin, which no split accepts, the whole pool and nothing.
    fn split_pool(&self) -> (Vec<u8>, Vec<u8>) {
        let at = self.studied.unwrap_or(self.pool.len());
        (
            encode_tag_pool(&self.pool[..at]),
            encode_tag_pool(&self.pool[at..]),
        )
    }
}

/// Files the grammar tables under the lexicon the build has just assembled
/// (`add-lingua-word-grammar`, design D3).
///
/// Every tag is parsed strictly, so a code outside the vocabulary fails the
/// build by name. A reading is filed under its own dictionary form; a
/// believable reading of another dictionary form is also filed, as such,
/// under the one the core's own cascade reads the form as — which is what the
/// card will be keyed by. Each is filed at its dictionary form's own place in
/// the lemma list (`lemma_id`), where the core reads it
/// (fix-lingua-lemma-lookup D4): readings and runs whose dictionary form the
/// lexicon does not hold as a lemma are dropped, never filed under the lemma
/// that string is a form of. Each run must cover exactly its word's gloss, and
/// a noun's runs carry the gender its readings give it ([`tags::noun_runs`]).
fn grammar_sections(
    inputs: &PackInputs,
    studied: StudiedLanguage,
    lex: &FstLexicon<&[u8]>,
) -> Result<GrammarSections, BuildError> {
    // A language that pins its pool writes it, readings or not, so a base never owes its pool
    // to a pair's senses (split-lingua-pack-format D5); without a pin, as before.
    if inputs.readings.is_empty() && inputs.senses.is_empty() && inputs.tag_pool.is_none() {
        return Ok(GrammarSections::default());
    }
    let fail = |what: String| BuildError::Grammar(what);

    let canonical = |text: &str| -> Result<String, BuildError> {
        Tag::parse_strict(text)
            .map(|tag| tag.to_ud())
            .map_err(|e| fail(format!("tag {text:?}: {e}")))
    };
    let senses = tags::noun_runs(inputs)?;
    let mut reading_tags = BTreeSet::new();
    for reading in &inputs.readings {
        reading_tags.insert(canonical(&reading.tag)?);
    }
    let mut sense_tags = BTreeSet::new();
    for (_, runs) in &senses {
        for (tag, _) in runs {
            sense_tags.insert(tag.clone());
        }
    }
    let tags::TagPool {
        tags: pool,
        studied: studied_tags,
    } = tags::tag_pool(inputs.tag_pool.as_deref(), reading_tags, sense_tags)?;
    if pool.len() > u16::MAX as usize {
        return Err(fail(format!(
            "{} distinct tags, over {}",
            pool.len(),
            u16::MAX
        )));
    }
    let index: BTreeMap<&str, u16> = pool
        .iter()
        .enumerate()
        .map(|(id, tag)| (tag.as_str(), id as u16))
        .collect();
    let tag_id = |text: &str| -> Result<u16, BuildError> {
        let ud = canonical(text)?;
        Ok(*index.get(ud.as_str()).expect("pooled above"))
    };
    let lemma_id =
        |lemma: &str| -> Option<u32> { lex.lemma_id(lemma).and_then(|id| u32::try_from(id).ok()) };
    let edit = |base: &str, form: &str| {
        FormEdit::between(base, form)
            .ok_or_else(|| fail(format!("form {form:?} is too far from {base:?} to store")))
    };

    let mut paradigms: BTreeMap<u32, BTreeSet<ParadigmEntry>> = BTreeMap::new();
    let mut by_form: BTreeMap<&str, Vec<&GrammarReading>> = BTreeMap::new();
    for reading in &inputs.readings {
        let Some(id) = lemma_id(&reading.lemma) else {
            continue;
        };
        paradigms
            .entry(id)
            .or_default()
            .insert(ParadigmEntry::Reading {
                form: edit(&reading.lemma, &reading.form)?,
                tag: tag_id(&reading.tag)?,
            });
        by_form
            .entry(reading.form.as_str())
            .or_default()
            .push(reading);
    }
    for (form, readings) in &by_form {
        let resolved = lemmatize(form, studied, lex);
        let Some(under) = lemma_id(&resolved) else {
            continue;
        };
        for reading in readings {
            if !reading.other || reading.lemma == resolved {
                continue;
            }
            let Some(other) = lemma_id(&reading.lemma) else {
                continue;
            };
            paradigms
                .entry(under)
                .or_default()
                .insert(ParadigmEntry::Also {
                    form: edit(&resolved, form)?,
                    other,
                });
        }
    }
    let paradigms: Vec<(u32, Vec<u8>)> = paradigms
        .into_iter()
        .map(|(id, entries)| {
            (
                id,
                encode_paradigm(&entries.into_iter().collect::<Vec<_>>()),
            )
        })
        .collect();

    let glosses: BTreeMap<&str, &str> = inputs
        .glosses
        .iter()
        .map(|(l, g)| (l.as_str(), g.as_str()))
        .collect();
    let mut runs_by_id: BTreeMap<u32, Vec<u8>> = BTreeMap::new();
    for &(lemma, ref runs) in &senses {
        let Some(gloss) = glosses.get(lemma) else {
            return Err(fail(format!("runs for {lemma:?}, which has no gloss")));
        };
        let senses = gloss.split("; ").count();
        let covered: usize = runs.iter().map(|(_, count)| *count as usize).sum();
        if covered != senses || runs.iter().any(|(_, count)| *count == 0) {
            return Err(fail(format!(
                "the runs of {lemma:?} cover {covered} senses, its gloss holds {senses}"
            )));
        }
        let Some(id) = lemma_id(lemma) else {
            continue;
        };
        let encoded = runs
            .iter()
            .map(|(tag, count)| {
                Ok(SenseRun {
                    tag: tag_id(tag)?,
                    count: *count,
                })
            })
            .collect::<Result<Vec<_>, BuildError>>()?;
        runs_by_id.insert(id, encode_runs(&encoded));
    }
    let runs: Vec<(u32, Vec<u8>)> = runs_by_id.into_iter().collect();

    let compress = |entries: &[(u32, Vec<u8>)]| {
        if entries.is_empty() {
            Vec::new()
        } else {
            zstd::encode_all(encode_indexed(entries).as_slice(), 19).expect("zstd encode")
        }
    };
    Ok(GrammarSections {
        paradigms: compress(&paradigms),
        senses: compress(&runs),
        pool,
        studied: studied_tags,
    })
}

/// An expression's key: its words' dictionary forms, lowercase, joined by
/// single spaces (`starting point` → `start point`).
///
/// `None` when the lexicon does not hold one of those forms. `lemmatize` never
/// fails — its last resort is the lowercased word itself — so this is not a
/// failure to catch but the membership test the requirement asks for: a key no
/// reading can produce would sit in the pack unreachable for ever.
///
/// A French or Spanish headword is keyed by the core's own reading of its
/// language, [`reading_expression_key`] — its pre-pass and cascade, its
/// determiners as written, two to seven tokens (`au revoir` → `à le revoir`,
/// add-lingua-french-expression-keys D1, D2, D4, D6; `al menos` → `a el menos`,
/// `tener en cuenta` → `tener en contar`, add-lingua-spanish-expression-keys D1,
/// D2, D4) —, the function review finds such an expression card's gloss through.
/// English keeps the split at spaces. An Italian headword is keyed through the
/// same reading, which the baseline gives it (add-lingua-italian-baseline D2): its
/// words as the tokeniser reads them, each piece its dictionary form, as an English
/// key's are, and no name.
fn expression_key(headword: &str, studied: StudiedLanguage, lex: &impl Lexicon) -> Option<String> {
    if studied != StudiedLanguage::English {
        return reading_expression_key(headword, studied, lex);
    }
    let mut words: Vec<String> = Vec::new();
    for word in headword.split_whitespace() {
        let lemma = lemmatize(word, studied, lex);
        if !lex.contains_lemma(&lemma) {
            return None;
        }
        words.push(lemma);
    }
    (!words.is_empty()).then(|| words.join(" "))
}

/// How many of a French or Spanish headword's tokens its key writes as the
/// headword does — each token's lowercased text equal to its key piece — which
/// decides between two headwords of one key (add-lingua-french-expression-keys
/// D6, add-lingua-spanish-expression-keys D3): `boîte à gants` writes more than
/// `boite à gants`, its `boite` keyed `boîte`; `soy inglés` more than
/// `soy inglesa`, its `inglesa` keyed `inglés`.
fn written_pieces(headword: &str, studied: StudiedLanguage, lex: &impl Lexicon) -> usize {
    headword_reading(headword, studied, lex)
        .unwrap_or_default()
        .iter()
        .filter(|(token, lemma)| {
            token.text.replace('\u{2019}', "'").to_lowercase()
                == expression_piece(&token.text, lemma, studied)
        })
        .count()
}

/// Encodes a gloss index + payload and zstd-compresses it. Layout matches the
/// reader: `count u32 | count*(id u32, off u32, len u32) | utf8`. Shared by the
/// per-lemma glosses, the expression table and a French or Spanish pack's expression names,
/// which are the same shape keyed by expression id.
fn compress_glosses(entries: &[(u32, &str)]) -> Vec<u8> {
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
    // Level 19: high ratio, deterministic for a given zstd version.
    zstd::encode_all(raw.as_slice(), 19).expect("zstd encode")
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingua_core::analysis::lexicon::Lexicon;
    use lingua_core::analysis::{
        ANALYZER_VERSION, FRENCH_ANALYZER_VERSION, SPANISH_ANALYZER_VERSION,
    };
    use lingua_core::engine::french_expression_key;
    use lingua_core::knowledge::state::FrequencyRanks;
    use lingua_core::packs::Pack;
    use lingua_core::packs::meta::{BaseMeta, LayerMeta};

    fn sources() -> Vec<licence::Source> {
        vec![
            licence::Source {
                name: "AGID".into(),
                licence: licence::Licence::Permissive,
            },
            licence::Source {
                name: "wordfreq".into(),
                licence: licence::Licence::CcBySa,
            },
            licence::Source {
                name: "kaikki".into(),
                licence: licence::Licence::CcBySa,
            },
        ]
    }

    fn inputs() -> PackInputs {
        PackInputs {
            meta: PackMeta {
                studied: "en".into(),
                native: "fr".into(),
                pack_version: "2026.09.1".into(),
                analyzer_version: ANALYZER_VERSION.into(),
                levels_estimated: false,
                licences: vec![
                    "AGID".into(),
                    "wordfreq CC BY-SA".into(),
                    "kaikki CC BY-SA".into(),
                ],
            },
            form_lemma: vec![
                ("running".into(), "run".into()),
                ("ran".into(), "run".into()),
                ("cities".into(), "city".into()),
            ],
            ranks: vec![("run".into(), 500), ("city".into(), 1_200)],
            glosses: vec![
                ("run".into(), "courir".into()),
                ("city".into(), "ville".into()),
            ],
            levels: vec![],
            expressions: vec![],
            readings: vec![],
            senses: vec![],
            notice: "AGID (permissive), wordfreq (CC BY-SA), kaikki (CC BY-SA).".into(),
            sources: sources(),
            lexical: None,
            tag_pool: None,
        }
    }

    /// The fixture plus the words an expression needs a lemma for: `start`
    /// (reached from `starting`), `point`, and `break` (from `breaking`).
    fn inputs_with_expression_words() -> PackInputs {
        let mut inp = inputs();
        inp.form_lemma.extend([
            ("starting".into(), "start".into()),
            ("breaking".into(), "break".into()),
        ]);
        inp.ranks.extend([
            ("start".into(), 300),
            ("point".into(), 400),
            ("break".into(), 800),
        ]);
        inp
    }

    /// A built pack's expression table, as (key, gloss) pairs in key order.
    /// Decoded here from the two sections rather than through the reader, which
    /// is the other half of this change.
    fn expressions_of(bytes: &[u8]) -> Vec<(String, String)> {
        let (_, sections) = lingua_core::packs::format::read_container(bytes).expect("decode");
        let find = |name: &str| {
            sections
                .iter()
                .find(|s| s.name == name)
                .map(|s| s.data.clone())
        };
        let (Some(keys), Some(blob)) = (find(section::EXPR), find(section::EXPR_ZST)) else {
            return Vec::new();
        };
        let raw = zstd::decode_all(blob.as_slice()).expect("zstd decode");
        let le = |at: usize| u32::from_le_bytes(raw[at..at + 4].try_into().unwrap()) as usize;
        let count = le(0);
        let payload = 4 + count * 12;
        let glosses: std::collections::BTreeMap<u64, String> = (0..count)
            .map(|i| {
                let (id, off, len) = (le(4 + i * 12), le(8 + i * 12), le(12 + i * 12));
                let text = String::from_utf8(raw[payload + off..payload + off + len].to_vec());
                (id as u64, text.expect("utf-8 gloss"))
            })
            .collect();

        let map = fst::Map::new(keys).expect("expression FST");
        let mut out = Vec::new();
        let mut stream = map.stream();
        while let Some((key, id)) = fst::Streamer::next(&mut stream) {
            let key = String::from_utf8(key.to_vec()).expect("utf-8 key");
            out.push((key, glosses[&id].clone()));
        }
        out
    }

    #[test]
    fn built_pack_round_trips_through_the_reader() {
        let bytes = build_pack(&inputs()).expect("build");
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(pack.meta().pair_key(), "en-fr");
        assert_eq!(pack.lexicon().lemma_of("running"), Some("run"));
        assert_eq!(pack.rank("run"), Some(500));
        assert_eq!(pack.gloss("city"), Some("ville"));
        assert!(pack.notice().contains("kaikki"));
    }

    #[test]
    fn spec_rebuilding_is_byte_for_byte_identical() {
        assert_eq!(
            build_pack(&inputs()).unwrap(),
            build_pack(&inputs()).unwrap()
        );
    }

    #[test]
    fn a_level_less_pack_is_byte_identical_to_before_the_levels_section() {
        // With no levels, no LEVELS section is emitted — the guarantee that this
        // feature does not disturb packs for pairs without CEFR data.
        let bytes = build_pack(&inputs()).unwrap();
        let (_, sections) = lingua_core::packs::format::read_container(&bytes).expect("decode");
        assert!(sections.iter().all(|s| s.name != section::LEVELS));
    }

    #[test]
    fn built_pack_carries_cefr_levels_when_present() {
        use lingua_core::knowledge::level::{CefrLevel, CefrLevels};
        let mut inp = inputs();
        inp.levels = vec![
            ("run".into(), CefrLevel::A1),
            ("city".into(), CefrLevel::A2),
        ];
        inp.sources.push(licence::Source {
            name: "CEFR-J".into(),
            licence: licence::Licence::Permissive,
        });
        inp.sources.push(licence::Source {
            name: "Octanove".into(),
            licence: licence::Licence::CcBySa,
        });
        inp.notice
            .push_str(" CEFR-J (permissive + citation), Octanove (CC BY-SA 4.0).");
        let bytes = build_pack(&inp).expect("build");
        let pack = Pack::load(&bytes).expect("load");
        assert!(pack.has_levels());
        assert_eq!(pack.level("run"), Some(CefrLevel::A1));
        assert_eq!(pack.level("city"), Some(CefrLevel::A2));
        assert_eq!(
            pack.lemmas_at_level(CefrLevel::A1),
            vec![("run", Some("courir"))]
        );
    }

    #[test]
    fn an_expression_is_keyed_by_its_words_dictionary_forms() {
        let mut inp = inputs_with_expression_words();
        inp.expressions = vec![("starting point".into(), "Point de départ".into())];
        let bytes = build_pack(&inp).expect("build");
        assert_eq!(
            expressions_of(&bytes),
            vec![("start point".into(), "Point de départ".into())]
        );
    }

    #[test]
    fn the_dictionary_spelling_wins_a_key_two_entries_reach() {
        let mut inp = inputs_with_expression_words();
        // Both lemmatise to `break point`; the entry that IS the key keeps it,
        // whichever order the source listed them in.
        inp.expressions = vec![
            ("breaking point".into(), "Point de rupture".into()),
            ("break point".into(), "Point d'arrêt".into()),
        ];
        let first = build_pack(&inp).expect("build");
        assert_eq!(
            expressions_of(&first),
            vec![("break point".into(), "Point d'arrêt".into())]
        );
        inp.expressions.reverse();
        assert_eq!(build_pack(&inp).expect("build"), first);
    }

    #[test]
    fn an_expression_holding_an_unknown_word_is_left_out() {
        let mut inp = inputs_with_expression_words();
        // `gun` is no lemma of this pack: the reader's analysis could never
        // produce the key, so the entry would be unreachable.
        inp.expressions = vec![("starting gun".into(), "Pistolet de départ".into())];
        let bytes = build_pack(&inp).expect("build");
        assert!(expressions_of(&bytes).is_empty());
        assert_eq!(
            bytes,
            build_pack(&inputs_with_expression_words()).unwrap(),
            "an entry nothing can reach must leave no trace in the pack"
        );
    }

    #[test]
    fn an_expression_less_pack_is_byte_identical_to_before_the_expression_section() {
        // With no expression, no section is emitted — the guarantee that this
        // feature does not disturb packs for pairs whose sources hold none.
        let bytes = build_pack(&inputs()).unwrap();
        let (_, sections) = lingua_core::packs::format::read_container(&bytes).expect("decode");
        assert!(
            sections
                .iter()
                .all(|s| s.name != section::EXPR && s.name != section::EXPR_ZST)
        );
    }

    #[test]
    fn expressions_need_no_new_source_and_leave_the_notice_alone() {
        // The entries come from the kaikki dump the glosses already come from,
        // so the licence guard has nothing new to clear.
        let plain = inputs_with_expression_words();
        let mut inp = inputs_with_expression_words();
        inp.expressions = vec![("starting point".into(), "Point de départ".into())];
        assert_eq!(inp.sources, plain.sources);
        assert_eq!(inp.notice, plain.notice);

        let bytes = build_pack(&inp).expect("build");
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(pack.notice(), plain.notice);
    }

    #[test]
    fn a_denied_source_licence_fails_the_build() {
        let mut inp = inputs();
        inp.sources.push(licence::Source {
            name: "Apertium".into(),
            licence: licence::Licence::Gpl,
        });
        assert_eq!(
            build_pack(&inp),
            Err(BuildError::DeniedLicence("Apertium".into()))
        );
    }

    #[test]
    fn a_notice_missing_a_source_fails_the_build() {
        let mut inp = inputs();
        inp.notice = "AGID and wordfreq only.".into(); // omits kaikki
        assert_eq!(
            build_pack(&inp),
            Err(BuildError::NoticeIncomplete("kaikki".into()))
        );
    }

    /// A gloss that stays over 5 MB even after zstd: a pseudo-random,
    /// poorly-compressible byte soup rather than a repeating pattern.
    fn incompressible_gloss() -> String {
        let mut big = String::with_capacity(9_000_000);
        let mut x: u32 = 0x1234_5678;
        for _ in 0..9_000_000 {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            // Spread over the 95 printable ASCII symbols: near-uniform entropy,
            // so zstd cannot bring 9 MB back under the 5 MB budget.
            big.push(char::from(b' ' + (x >> 17) as u8 % 95));
        }
        big
    }

    #[test]
    fn over_budget_fails_with_a_clear_error() {
        let mut inp = inputs();
        inp.glosses = vec![("run".into(), incompressible_gloss())];
        match build_pack(&inp) {
            Err(e @ BuildError::OverBudget { budget, size, .. }) => {
                assert_eq!(budget, MAX_PACK_BYTES);
                assert!(size > budget);
                // No expression table to give way: gloss coverage is what is at
                // fault, and the FST and the frequencies are never named.
                assert!(e.to_string().contains("glossed lemmas"), "{e}");
            }
            other => panic!("expected OverBudget, got {other:?}"),
        }
    }

    // — French expression keys (add-lingua-french-expression-keys) —

    /// A fr-en pack's inputs: the words the French headwords below are made of, `t` left out,
    /// `peut-être` listed whole and `boite` a spelling filed under `boîte`; no expression yet.
    fn french_inputs() -> PackInputs {
        let mut inp = inputs();
        inp.meta.studied = "fr".into();
        inp.meta.native = "en".into();
        inp.meta.analyzer_version = FRENCH_ANALYZER_VERSION.into();
        inp.form_lemma = [
            ("a", "avoir"),
            ("la", "le"),
            ("les", "le"),
            ("mains", "main"),
            ("gants", "gant"),
            ("boite", "boîte"),
            ("peut-être", "peut-être"),
        ]
        .iter()
        .map(|(form, lemma)| ((*form).into(), (*lemma).into()))
        .collect();
        inp.ranks = [
            "à",
            "le",
            "revoir",
            "coup",
            "de",
            "œil",
            "abord",
            "il",
            "y",
            "avoir",
            "haut",
            "main",
            "tout",
            "suite",
            "compte",
            "en",
            "boîte",
            "gant",
            "peut-être",
        ]
        .iter()
        .zip(1..)
        .map(|(lemma, rank)| ((*lemma).into(), rank))
        .collect();
        inp.glosses = vec![("revoir".into(), "to see again".into())];
        inp
    }

    /// `french_inputs` holding `expressions`, each glossed by its own headword.
    fn french_pack(expressions: &[&str]) -> Vec<u8> {
        let mut inp = french_inputs();
        inp.expressions = expressions
            .iter()
            .map(|headword| ((*headword).into(), format!("gloss of {headword}")))
            .collect();
        build_pack(&inp).expect("a French pack builds")
    }

    /// A built pack's expressions as (key, name, gloss), the name `None` where the pack
    /// carries none for the key.
    fn named_expressions_of(bytes: &[u8]) -> Vec<(String, Option<String>, String)> {
        let pack = Pack::load(bytes).expect("load");
        expressions_of(bytes)
            .into_iter()
            .map(|(key, gloss)| {
                let name = pack.expression_name(&key).map(str::to_owned);
                (key, name, gloss)
            })
            .collect()
    }

    fn named(key: &str, name: Option<&str>, headword: &str) -> (String, Option<String>, String) {
        (
            key.into(),
            name.map(str::to_owned),
            format!("gloss of {headword}"),
        )
    }

    #[test]
    fn spec_scenario_french_expressions_are_keyed_as_french_is_read() {
        // A contracted article, an elision, an inflected word, and a headword that is its key.
        let bytes = french_pack(&["au revoir", "coup d'œil", "il y a", "tout de suite"]);
        assert_eq!(
            named_expressions_of(&bytes),
            [
                named("coup de œil", Some("coup d'œil"), "coup d'œil"),
                named("il y avoir", Some("il y a"), "il y a"),
                named("tout de suite", None, "tout de suite"),
                named("à le revoir", Some("au revoir"), "au revoir"),
            ]
        );
        // The key is what review finds the card's gloss through.
        let pack = Pack::load(&bytes).unwrap();
        let key = french_expression_key("au revoir", pack.lexicon()).expect("a key");
        assert_eq!(pack.expression(&key), Some("gloss of au revoir"));
    }

    #[test]
    fn spec_scenario_a_word_the_pre_pass_splits_is_an_expression() {
        // `d'abord` reads as two tokens; `peut-être`, listed whole, as one.
        let bytes = french_pack(&["d'abord", "peut-être"]);
        assert_eq!(
            named_expressions_of(&bytes),
            [named("de abord", Some("d'abord"), "d'abord")]
        );
    }

    #[test]
    fn spec_scenario_determiners_keep_their_written_form() {
        let bytes = french_pack(&["à la", "haut la main", "haut les mains"]);
        assert_eq!(
            named_expressions_of(&bytes),
            [
                named("haut la main", None, "haut la main"),
                named("haut les main", Some("haut les mains"), "haut les mains"),
                named("à la", None, "à la"),
            ]
        );
    }

    #[test]
    fn spec_scenario_a_french_key_longer_than_the_window_or_missing_a_word_is_left_out() {
        // Eight tokens are beyond French's window, seven within it; `compte en t` would key
        // `compte en` without its `t`, which the lexicon does not list.
        let seven = ["coup"; 7].join(" ");
        let eight = ["coup"; 8].join(" ");
        let bytes = french_pack(&[&seven, &eight, "compte en t"]);
        assert_eq!(named_expressions_of(&bytes), [named(&seven, None, &seven)]);
        assert_eq!(
            french_pack(&["compte en t"]),
            build_pack(&french_inputs()).unwrap(),
            "an entry nothing can reach leaves no trace in the pack"
        );
    }

    #[test]
    fn spec_scenario_two_spellings_reach_one_key() {
        // `boite` is read `boîte`: both headwords reach `boîte à gant`, and the one writing more
        // of its tokens as the key does keeps it, whichever order the source lists them in.
        let first = french_pack(&["boite à gants", "boîte à gants"]);
        assert_eq!(
            named_expressions_of(&first),
            [named(
                "boîte à gant",
                Some("boîte à gants"),
                "boîte à gants"
            )]
        );
        assert_eq!(french_pack(&["boîte à gants", "boite à gants"]), first);
        // The other spelling is kept beside it, with its gloss
        // (pick-lingua-expression-written-as-the-page D1).
        assert_eq!(
            Pack::load(&first)
                .unwrap()
                .expression_headwords("boîte à gant"),
            [
                ("boîte à gants", "gloss of boîte à gants"),
                ("boite à gants", "gloss of boite à gants")
            ]
        );
        // The headword written as the key still comes first.
        let bytes = french_pack(&["haut les main", "haut les mains"]);
        assert_eq!(
            named_expressions_of(&bytes),
            [named("haut les main", None, "haut les main")]
        );
        // `boîte` and `à` are written as their key pieces; `boite` and `gants` are not.
        let fr = StudiedLanguage::French;
        assert_eq!(written_pieces("boîte à gants", fr, &lex_of(&first)), 2);
        assert_eq!(written_pieces("boite à gants", fr, &lex_of(&first)), 1);
        // Then byte order, between headwords writing as many: the straight apostrophe first.
        let bytes = french_pack(&["coup d\u{2019}œil", "coup d'œil"]);
        assert_eq!(
            named_expressions_of(&bytes),
            [named("coup de œil", Some("coup d'œil"), "coup d'œil")]
        );
    }

    /// A built pack's lexicon, for the builder's own helpers.
    fn lex_of(bytes: &[u8]) -> FstLexicon<Vec<u8>> {
        let (_, sections) = lingua_core::packs::format::read_container(bytes).expect("decode");
        let find = |name: &str| {
            sections
                .iter()
                .find(|s| s.name == name)
                .map(|s| s.data.clone())
                .expect(name)
        };
        let pool = String::from_utf8(find(section::LEMMAS)).expect("utf-8 pool");
        FstLexicon::from_slices(find(section::FORMS), &pool).expect("lexicon")
    }

    #[test]
    fn a_french_pack_whose_every_headword_is_its_key_carries_no_names() {
        let bytes = french_pack(&["tout de suite", "haut la main"]);
        assert_eq!(expressions_of(&bytes).len(), 2);
        let (_, sections) = lingua_core::packs::format::read_container(&bytes).expect("decode");
        assert!(sections.iter().all(|s| s.name != section::EXPR_NAMES_ZST));
        // Nor does one with no expression at all.
        let (_, sections) =
            lingua_core::packs::format::read_container(&french_pack(&[])).expect("decode");
        assert!(sections.iter().all(|s| s.name != section::EXPR_NAMES_ZST));
    }

    #[test]
    fn an_english_pack_carries_no_names() {
        // `starting point` is keyed `start point`, yet named nothing: English packs carry no
        // names, so they keep their bytes.
        let mut inp = inputs_with_expression_words();
        inp.expressions = vec![("starting point".into(), "Point de départ".into())];
        let bytes = build_pack(&inp).expect("build");
        let (_, sections) = lingua_core::packs::format::read_container(&bytes).expect("decode");
        assert!(sections.iter().all(|s| s.name != section::EXPR_NAMES_ZST));
        assert_eq!(
            expressions_of(&bytes),
            vec![("start point".into(), "Point de départ".into())]
        );
    }

    // — every headword of a shared key (pick-lingua-expression-written-as-the-page) —

    /// `french_inputs` plus the words of `ça va`, `ça ira`, `allez-y`, `allons-y` and `vas-y`.
    fn shared_key_pack(expressions: &[&str]) -> Vec<u8> {
        let mut inp = french_inputs();
        inp.form_lemma.extend(
            [
                ("va", "aller"),
                ("ira", "aller"),
                ("allez", "aller"),
                ("allons", "aller"),
                ("vas", "aller"),
            ]
            .iter()
            .map(|(form, lemma)| ((*form).into(), (*lemma).into())),
        );
        inp.ranks
            .extend([("ça".into(), 100), ("aller".into(), 101)]);
        inp.expressions = expressions
            .iter()
            .map(|headword| ((*headword).into(), format!("gloss of {headword}")))
            .collect();
        build_pack(&inp).expect("a French pack builds")
    }

    fn headwords_of(bytes: &[u8], key: &str) -> Vec<(String, String)> {
        Pack::load(bytes)
            .expect("load")
            .expression_headwords(key)
            .into_iter()
            .map(|(name, gloss)| (name.to_owned(), gloss.to_owned()))
            .collect()
    }

    fn headword(name: &str) -> (String, String) {
        (name.to_owned(), format!("gloss of {name}"))
    }

    #[test]
    fn spec_scenario_two_headwords_reach_one_key_and_both_are_kept() {
        let bytes = shared_key_pack(&["ça va", "ça ira", "au revoir"]);
        // The key answers `ça ira`, which wins it, with its gloss and its name, as before…
        assert_eq!(
            named_expressions_of(&bytes),
            [
                named("à le revoir", Some("au revoir"), "au revoir"),
                named("ça aller", Some("ça ira"), "ça ira"),
            ]
        );
        // …and the pack holds `ça va` with its own gloss, listed as the key's other headword.
        assert_eq!(
            headwords_of(&bytes, "ça aller"),
            [headword("ça ira"), headword("ça va")]
        );
        // Whichever order the source lists them in.
        assert_eq!(shared_key_pack(&["au revoir", "ça ira", "ça va"]), bytes);
    }

    #[test]
    fn spec_scenario_three_headwords_reach_one_key() {
        let bytes = shared_key_pack(&["vas-y", "allons-y", "allez-y"]);
        assert_eq!(
            headwords_of(&bytes, "aller y"),
            [headword("allez-y"), headword("allons-y"), headword("vas-y")]
        );
    }

    #[test]
    fn spec_scenario_a_key_one_headword_reaches_lists_nothing() {
        let bytes = shared_key_pack(&["au revoir", "ça va"]);
        assert_eq!(headwords_of(&bytes, "à le revoir"), [headword("au revoir")]);
        assert_eq!(headwords_of(&bytes, "ça aller"), [headword("ça va")]);
        // No key has others: the pack carries no section, and is the one it was before.
        let (_, sections) = lingua_core::packs::format::read_container(&bytes).expect("decode");
        assert!(sections.iter().all(|s| s.name != section::EXPR_OTHERS));
    }

    #[test]
    fn the_others_section_lists_each_key_s_other_headwords_in_order() {
        // The keys sort `aller y` (0), `à le revoir` (1), `ça aller` (2); the other headwords
        // take the ids after them, in key order and the builder's order within a key.
        let bytes = shared_key_pack(&[
            "ça va",
            "ça ira",
            "au revoir",
            "vas-y",
            "allons-y",
            "allez-y",
        ]);
        let (_, sections) = lingua_core::packs::format::read_container(&bytes).expect("decode");
        let others = sections
            .iter()
            .find(|s| s.name == section::EXPR_OTHERS)
            .map(|s| s.data.clone())
            .expect("the section");
        let mut want = 3u32.to_le_bytes().to_vec();
        for (key, other) in [(0u32, 3u32), (0, 4), (2, 5)] {
            want.extend_from_slice(&key.to_le_bytes());
            want.extend_from_slice(&other.to_le_bytes());
        }
        assert_eq!(others, want);
        // No key maps to an other headword's id: the FST holds the three keys alone.
        assert_eq!(
            expressions_of(&bytes)
                .into_iter()
                .map(|(key, _)| key)
                .collect::<Vec<_>>(),
            ["aller y", "à le revoir", "ça aller"]
        );
    }

    #[test]
    fn spec_scenario_a_core_that_does_not_read_the_section() {
        // Stripped of the section, as a core that predates it reads it, the pack answers every
        // key as the pack built without the other headwords — what the builder made before.
        let bytes = shared_key_pack(&[
            "ça va",
            "ça ira",
            "au revoir",
            "vas-y",
            "allons-y",
            "allez-y",
        ]);
        let (meta, sections) = lingua_core::packs::format::read_container(&bytes).expect("decode");
        let kept: Vec<(&str, &[u8])> = sections
            .iter()
            .filter(|s| s.name != section::EXPR_OTHERS)
            .map(|s| (s.name.as_str(), s.data.as_slice()))
            .collect();
        let older = Pack::load(&write_container(&meta, &kept)).expect("load");
        let before = Pack::load(&shared_key_pack(&["ça ira", "au revoir", "allez-y"])).unwrap();
        for key in ["aller y", "à le revoir", "ça aller", "ça va", "allons-y"] {
            assert_eq!(older.expression(key), before.expression(key), "{key}");
            assert_eq!(
                older.expression_name(key),
                before.expression_name(key),
                "{key}"
            );
            assert_eq!(
                older.expression_headwords(key),
                before.expression_headwords(key),
                "{key}"
            );
        }
    }

    #[test]
    fn spec_scenario_english_packs_carry_no_others() {
        // Two English entries reach one key: the pack keeps the first alone, byte for byte the
        // pack built from it alone, and carries no section (a Spanish pack keeps both since
        // pick-lingua-spanish-expression-written-as-the-page D1).
        let mut inp = inputs_with_expression_words();
        inp.expressions = vec![
            ("breaking point".into(), "Point de rupture".into()),
            ("break point".into(), "Point d'arrêt".into()),
        ];
        let english = build_pack(&inp).expect("build");
        inp.expressions.remove(0);
        assert_eq!(english, build_pack(&inp).expect("build"));
        let (_, sections) = lingua_core::packs::format::read_container(&english).expect("decode");
        assert!(sections.iter().all(|s| s.name != section::EXPR_OTHERS));
    }

    // — Spanish expression keys (add-lingua-spanish-expression-keys) —

    /// An es-fr pack's inputs: the words the Spanish headwords below are made of, M8 filing
    /// `la`, `las` and `los` under `el`, `cuenta` under `contar` and `inglesa` under `inglés`;
    /// no expression yet.
    fn spanish_inputs() -> PackInputs {
        let mut inp = inputs();
        inp.meta.studied = "es".into();
        inp.meta.native = "fr".into();
        inp.meta.analyzer_version = SPANISH_ANALYZER_VERSION.into();
        inp.form_lemma = [
            ("la", "el"),
            ("las", "el"),
            ("los", "el"),
            ("cuenta", "contar"),
            ("armas", "arma"),
            ("soy", "ser"),
            ("inglesa", "inglés"),
        ]
        .iter()
        .map(|(form, lemma)| ((*form).into(), (*lemma).into()))
        .collect();
        inp.ranks = [
            "a", "el", "menos", "tener", "en", "contar", "vez", "arma", "fin", "y", "cabo", "de",
            "ser", "inglés", "después",
        ]
        .iter()
        .zip(1..)
        .map(|(lemma, rank)| ((*lemma).into(), rank))
        .collect();
        inp.glosses = vec![("menos".into(), "moins".into())];
        inp
    }

    /// `spanish_inputs` holding `expressions`, each glossed by its own headword.
    fn spanish_pack(expressions: &[&str]) -> Vec<u8> {
        let mut inp = spanish_inputs();
        inp.expressions = expressions
            .iter()
            .map(|headword| ((*headword).into(), format!("gloss of {headword}")))
            .collect();
        build_pack(&inp).expect("a Spanish pack builds")
    }

    #[test]
    fn spec_scenario_a_spanish_contraction_and_an_inflected_word() {
        let bytes = spanish_pack(&["al menos", "tener en cuenta", "después de"]);
        assert_eq!(
            named_expressions_of(&bytes),
            [
                named("a el menos", Some("al menos"), "al menos"),
                named("después de", None, "después de"),
                named(
                    "tener en contar",
                    Some("tener en cuenta"),
                    "tener en cuenta"
                ),
            ]
        );
        // The key is what review finds a named card's gloss through.
        let pack = Pack::load(&bytes).unwrap();
        let key =
            reading_expression_key("tener en cuenta", StudiedLanguage::Spanish, pack.lexicon())
                .expect("a key");
        assert_eq!(pack.expression(&key), Some("gloss of tener en cuenta"));
    }

    #[test]
    fn spec_scenario_spanish_determiners_keep_their_written_form() {
        let bytes = spanish_pack(&["a la vez", "a las armas", "al arma"]);
        assert_eq!(
            named_expressions_of(&bytes),
            [
                named("a el arma", Some("al arma"), "al arma"),
                named("a la vez", None, "a la vez"),
                named("a las arma", Some("a las armas"), "a las armas"),
            ]
        );
    }

    #[test]
    fn spec_scenario_seven_tokens_and_a_key_longer_than_the_window() {
        // « al fin y al cabo » is five words and seven tokens: kept. Eight tokens are beyond
        // Spanish's window; a word the lexicon does not hold leaves the entry out too.
        let eight = ["vez"; 8].join(" ");
        let bytes = spanish_pack(&[
            "al fin y al cabo",
            &eight,
            "al fin y al cabo de",
            "en mente",
        ]);
        assert_eq!(
            named_expressions_of(&bytes),
            [named(
                "a el fin y a el cabo",
                Some("al fin y al cabo"),
                "al fin y al cabo"
            )]
        );
        assert_eq!(
            spanish_pack(&[&eight, "en mente"]),
            build_pack(&spanish_inputs()).unwrap(),
            "an entry nothing can reach leaves no trace in the pack"
        );
    }

    #[test]
    fn spec_scenario_two_spanish_headwords_reach_one_key() {
        // `inglesa` is filed under `inglés`: both headwords reach `ser inglés`, and the one
        // writing more of its tokens as the key does keeps it, whichever order the source
        // lists them in. Byte order alone would name the key by the feminine.
        let first = spanish_pack(&["soy inglesa", "soy inglés"]);
        assert_eq!(
            named_expressions_of(&first),
            [named("ser inglés", Some("soy inglés"), "soy inglés")]
        );
        assert_eq!(spanish_pack(&["soy inglés", "soy inglesa"]), first);
        let es = StudiedLanguage::Spanish;
        assert_eq!(written_pieces("soy inglés", es, &lex_of(&first)), 1);
        assert_eq!(written_pieces("soy inglesa", es, &lex_of(&first)), 0);
        // The headword written as the key still comes first.
        let bytes = spanish_pack(&["ser inglesa", "ser inglés"]);
        assert_eq!(
            named_expressions_of(&bytes),
            [named("ser inglés", None, "ser inglés")]
        );
        // The other is kept beside the winner, with its own gloss
        // (pick-lingua-spanish-expression-written-as-the-page D1).
        assert_eq!(
            headwords_of(&first, "ser inglés"),
            [headword("soy inglés"), headword("soy inglesa")]
        );
    }

    // — every headword of a shared Spanish key (pick-lingua-spanish-expression-written-as-the-page) —

    /// `spanish_inputs` plus the words of `hacer tiempo`, `hace tiempo`, `echar a perder`,
    /// `echado a perder` and `echarse a perder`, each headword glossed by itself.
    fn shared_spanish_key_pack(expressions: &[&str]) -> Vec<u8> {
        let mut inp = spanish_inputs();
        inp.form_lemma.extend(
            [("hace", "hacer"), ("echado", "echar")]
                .iter()
                .map(|(form, lemma)| ((*form).into(), (*lemma).into())),
        );
        inp.ranks.extend([
            ("hacer".into(), 100),
            ("tiempo".into(), 101),
            ("echar".into(), 102),
            ("perder".into(), 103),
        ]);
        inp.expressions = expressions
            .iter()
            .map(|headword| ((*headword).into(), format!("gloss of {headword}")))
            .collect();
        build_pack(&inp).expect("a Spanish pack builds")
    }

    #[test]
    fn spec_scenario_two_headwords_reach_one_spanish_key() {
        let bytes = shared_spanish_key_pack(&["hace tiempo", "hacer tiempo", "al menos"]);
        // The key answers `hacer tiempo`, which wins it, with its gloss, as before…
        let pack = Pack::load(&bytes).unwrap();
        assert_eq!(
            pack.expression("hacer tiempo"),
            Some("gloss of hacer tiempo")
        );
        assert_eq!(pack.expression_name("hacer tiempo"), None);
        // …and the pack holds `hace tiempo` with its own gloss, listed as the key's other headword.
        assert_eq!(
            headwords_of(&bytes, "hacer tiempo"),
            [headword("hacer tiempo"), headword("hace tiempo")]
        );
        // Whichever order the source lists them in.
        assert_eq!(
            shared_spanish_key_pack(&["al menos", "hacer tiempo", "hace tiempo"]),
            bytes
        );
    }

    #[test]
    fn spec_scenario_three_headwords_reach_one_spanish_key() {
        let bytes =
            shared_spanish_key_pack(&["echarse a perder", "echado a perder", "echar a perder"]);
        assert_eq!(
            headwords_of(&bytes, "echar a perder"),
            [
                headword("echar a perder"),
                headword("echado a perder"),
                headword("echarse a perder")
            ]
        );
    }

    #[test]
    fn spec_scenario_a_spanish_key_one_headword_reaches_lists_nothing() {
        let bytes = shared_spanish_key_pack(&["tener en cuenta", "hace tiempo"]);
        assert_eq!(
            headwords_of(&bytes, "tener en contar"),
            [headword("tener en cuenta")]
        );
        assert_eq!(
            headwords_of(&bytes, "hacer tiempo"),
            [headword("hace tiempo")]
        );
        // No key has others: the pack carries no section.
        let (_, sections) = lingua_core::packs::format::read_container(&bytes).expect("decode");
        assert!(sections.iter().all(|s| s.name != section::EXPR_OTHERS));
    }

    #[test]
    fn spec_scenario_a_core_that_does_not_read_the_spanish_section() {
        // Stripped of the section, as a core that predates it reads it, the Spanish pack answers
        // every key as the pack built without the other headwords — what the builder made before.
        let all = [
            "hace tiempo",
            "hacer tiempo",
            "tener en cuenta",
            "echarse a perder",
            "echado a perder",
            "echar a perder",
        ];
        let bytes = shared_spanish_key_pack(&all);
        let (meta, sections) = lingua_core::packs::format::read_container(&bytes).expect("decode");
        assert!(sections.iter().any(|s| s.name == section::EXPR_OTHERS));
        let kept: Vec<(&str, &[u8])> = sections
            .iter()
            .filter(|s| s.name != section::EXPR_OTHERS)
            .map(|s| (s.name.as_str(), s.data.as_slice()))
            .collect();
        let older = Pack::load(&write_container(&meta, &kept)).expect("load");
        let before = Pack::load(&shared_spanish_key_pack(&[
            "hacer tiempo",
            "tener en cuenta",
            "echar a perder",
        ]))
        .unwrap();
        for key in [
            "hacer tiempo",
            "tener en contar",
            "echar a perder",
            "hace tiempo",
            "echado a perder",
        ] {
            assert_eq!(older.expression(key), before.expression(key), "{key}");
            assert_eq!(
                older.expression_name(key),
                before.expression_name(key),
                "{key}"
            );
            assert_eq!(
                older.expression_headwords(key),
                before.expression_headwords(key),
                "{key}"
            );
        }
    }

    #[test]
    fn a_spanish_pack_carries_names_only_where_a_key_is_not_its_headword() {
        let bytes = spanish_pack(&["al menos", "a la vez"]);
        let pack = Pack::load(&bytes).unwrap();
        assert_eq!(pack.expression_name("a el menos"), Some("al menos"));
        assert_eq!(pack.expression_name("a la vez"), None);
        // A Spanish pack whose every headword is its key carries no names section.
        let (_, sections) =
            lingua_core::packs::format::read_container(&spanish_pack(&["a la vez"]))
                .expect("decode");
        assert!(sections.iter().all(|s| s.name != section::EXPR_NAMES_ZST));
    }

    #[test]
    fn over_budget_names_the_expression_table_when_the_pack_carries_one() {
        let mut inp = inputs();
        // `run` and `city` are both lemmas here, so the entry reaches the table.
        inp.expressions = vec![("run city".into(), incompressible_gloss())];
        match build_pack(&inp) {
            Err(e @ BuildError::OverBudget { .. }) => {
                let msg = e.to_string();
                assert!(msg.contains("expression table"), "{msg}");
                assert!(!msg.contains("glossed lemmas"), "{msg}");
            }
            other => panic!("expected OverBudget, got {other:?}"),
        }
    }

    // — every lemma read as itself (fix-lingua-lemma-lookup D4) —

    fn pairs(rows: &[(&str, &str)]) -> Vec<(String, String)> {
        rows.iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect()
    }

    #[test]
    fn spec_scenario_a_lemma_whose_own_spelling_reads_as_another() {
        // `venue` ranked, the forms table reading the form `venue` as venir.
        let mut inp = french_inputs();
        inp.form_lemma
            .extend(pairs(&[("venue", "venir"), ("viens", "venir")]));
        inp.ranks
            .extend([("venir".into(), 60), ("venue".into(), 1_900)]);
        let err = build_pack(&inp).expect_err("a ranked lemma read as another");
        assert_eq!(
            err,
            BuildError::LemmaReadsAsAnother {
                lemma: "venue".into(),
                reads_as: "venir".into(),
                more: 0,
            }
        );
        assert_eq!(
            err.to_string(),
            "the lemma \"venue\" reads as \"venir\": the forms table lists it as a form of \"venir\""
        );
    }

    #[test]
    fn spec_scenario_the_acronym_is_a_form_in_capitals_its_own_lemma() {
        // keep-lingua-french-acronyms-in-capitals D2: `CA` → `CA`, a dictionary word glossed and
        // unranked, beside `ca` → `ça`; read back as written, `ca` as before.
        let mut inp = french_inputs();
        inp.form_lemma
            .extend(pairs(&[("CA", "CA"), ("ca", "ça"), ("ça", "ça")]));
        inp.ranks.push(("ça".into(), 40));
        inp.glosses.extend(pairs(&[
            ("CA", "board of directors; revenue, turnover, net sales"),
            ("ça", "that"),
        ]));
        let bytes = build_pack(&inp).expect("an acronym row builds");
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(pack.lexicon().lemma_of("CA"), Some("CA"));
        assert_eq!(pack.lexicon().lemma_of("ca"), Some("ça"));
        assert_eq!(
            pack.gloss("CA"),
            Some("board of directors; revenue, turnover, net sales")
        );
        assert_eq!(pack.gloss("ca"), None);
        assert_eq!(pack.rank("CA"), None);
        // Glossed with no form listed, the acronym is its own form all the same.
        let mut inp = french_inputs();
        inp.glosses.push(("ÉTÉ".into(), "a gloss".into()));
        let pack = Pack::load(&build_pack(&inp).expect("builds")).expect("load");
        assert_eq!(pack.lexicon().lemma_of("ÉTÉ"), Some("ÉTÉ"));
    }

    #[test]
    fn spec_scenario_a_stray_capital() {
        // Any other capital fails the build and names the form: `Paris`, `Ca`, `CA` read as `ça`,
        // `ca` read as `CA`, a single letter in capitals, a capital glossed or ranked.
        let refused = |inp: &PackInputs, named: &str| {
            let err = build_pack(inp).expect_err("a stray capital");
            assert_eq!(err, BuildError::StrayCapital(named.into()));
            assert!(err.to_string().contains(&format!("{named:?}")), "{err}");
            assert!(err.to_string().contains("holds a capital"), "{err}");
        };
        for (form, lemma, named) in [
            ("Paris", "paris", "Paris"),
            ("Paris", "Paris", "Paris"),
            ("Ca", "Ca", "Ca"),
            ("CA", "ça", "CA"),
            ("ca", "CA", "CA"),
            ("A", "A", "A"),
        ] {
            let mut inp = french_inputs();
            inp.form_lemma.push((form.into(), lemma.into()));
            refused(&inp, named);
        }
        let mut inp = french_inputs();
        inp.glosses.push(("Ca".into(), "that".into()));
        refused(&inp, "Ca");
        let mut inp = french_inputs();
        inp.ranks.push(("Paris".into(), 900));
        refused(&inp, "Paris");
        // A pack studying English or Spanish holds no form in capitals, an acronym's shape or not.
        let mut inp = inputs();
        inp.form_lemma.push(("US".into(), "US".into()));
        refused(&inp, "US");
        let mut inp = spanish_inputs();
        inp.form_lemma.push(("CA".into(), "CA".into()));
        refused(&inp, "CA");
        // The guard runs before the forms are read: the pack studying French builds without them.
        assert!(build_pack(&french_inputs()).is_ok());
    }

    #[test]
    fn a_glossed_lemma_or_a_forms_table_lemma_read_as_another_is_refused_too() {
        // Glossed, not ranked.
        let mut inp = french_inputs();
        inp.form_lemma.push(("venue".into(), "venir".into()));
        inp.ranks.push(("venir".into(), 60));
        inp.glosses.push(("venue".into(), "coming, arrival".into()));
        assert!(matches!(
            build_pack(&inp),
            Err(BuildError::LemmaReadsAsAnother { lemma, reads_as, .. })
                if lemma == "venue" && reads_as == "venir"
        ));
        // The lemma of another form of the table.
        let mut inp = french_inputs();
        inp.form_lemma
            .extend(pairs(&[("venue", "venir"), ("venues", "venue")]));
        assert!(matches!(
            build_pack(&inp),
            Err(BuildError::LemmaReadsAsAnother { lemma, .. }) if lemma == "venue"
        ));
        // Whatever the byte order: `donnée` sorts before `donner`, whose form the table says it
        // is (`build_lexicon_blobs` lets the pairs win, D5). The first is named, the rest counted.
        let mut inp = french_inputs();
        inp.form_lemma
            .extend(pairs(&[("donnée", "donner"), ("venue", "venir")]));
        inp.ranks.extend([
            ("donner".into(), 70),
            ("donnée".into(), 1_711),
            ("venir".into(), 60),
            ("venue".into(), 1_900),
        ]);
        let err = build_pack(&inp).expect_err("two ranked lemmas read as others");
        assert_eq!(
            err,
            BuildError::LemmaReadsAsAnother {
                lemma: "donnée".into(),
                reads_as: "donner".into(),
                more: 1,
            }
        );
        assert!(err.to_string().ends_with(", and 1 more lemma"), "{err}");
    }

    #[test]
    fn spec_scenario_a_form_listed_with_two_lemmas() {
        let mut inp = french_inputs();
        inp.form_lemma
            .extend(pairs(&[("porte", "porte"), ("porte", "porter")]));
        let err = build_pack(&inp).expect_err("a form with two lemmas");
        assert_eq!(
            err,
            BuildError::FormWithTwoLemmas {
                form: "porte".into(),
                lemmas: vec!["porte".into(), "porter".into()],
                more: 0,
            }
        );
        assert_eq!(
            err.to_string(),
            "the forms table lists the form \"porte\" with 2 lemmas: \"porte\", \"porter\""
        );
        // Two of them: the first form byte-wise is named, the other counted.
        inp.form_lemma
            .extend(pairs(&[("lu", "lire"), ("lu", "luire")]));
        let err = build_pack(&inp).expect_err("two forms with two lemmas");
        assert!(
            matches!(&err, BuildError::FormWithTwoLemmas { form, more: 1, .. } if form == "lu"),
            "{err:?}"
        );
        assert!(err.to_string().ends_with(", and 1 more form"), "{err}");
    }

    #[test]
    fn a_form_listed_twice_with_one_lemma_builds() {
        let mut inp = french_inputs();
        inp.form_lemma
            .extend(pairs(&[("portes", "porte"), ("portes", "porte")]));
        let pack = Pack::load(&build_pack(&inp).expect("one lemma, listed twice")).unwrap();
        assert_eq!(pack.lexicon().lemma_of("portes"), Some("porte"));
    }

    #[test]
    fn spec_scenario_a_level_written_for_a_form() {
        use lingua_core::knowledge::level::{CefrLevel, CefrLevels};
        // `donnée` no lemma of the pack, the forms table reading it as donner: its B1, written
        // after donner's A1, is filed under no lemma — where it overwrote donner's.
        let mut inp = french_inputs();
        inp.form_lemma.push(("donnée".into(), "donner".into()));
        inp.ranks.push(("donner".into(), 70));
        inp.levels = vec![
            ("donner".into(), CefrLevel::A1),
            ("donnée".into(), CefrLevel::B1),
        ];
        let pack = Pack::load(&build_pack(&inp).expect("build")).expect("load");
        assert!(!pack.lexicon().contains_lemma("donnée"));
        assert_eq!(pack.level("donner"), Some(CefrLevel::A1));
        assert!(pack.lemmas_at_level(CefrLevel::B1).is_empty());
        assert_eq!(pack.lemmas_at_level(CefrLevel::A1), vec![("donner", None)]);
        // The spelling's estimate is donner's, the lemma the forms read it as (D3).
        assert_eq!(pack.level("donnée"), Some(CefrLevel::A1));
    }

    #[test]
    fn what_is_filed_under_a_lemma_is_what_the_core_reads_for_it() {
        // Every lemma-keyed section at the lemma's own place: rank, gloss, level, lexical bit,
        // readings and runs of a lemma of a built pack are read back by its own spelling.
        use lingua_core::knowledge::level::{CefrLevel, CefrLevels};
        let mut inp = inputs_with_grammar();
        inp.levels = vec![("leave".into(), CefrLevel::A2)];
        // A lexical table naming `leaf` (unglossed) and not `run` (glossed): it is written.
        inp.lexical = Some(vec!["leave".into(), "leaf".into()]);
        let bytes = build_pack(&inp).expect("build");
        assert!(section_of(&bytes, section::LEXICAL).is_some());
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(pack.rank("leave"), Some(371));
        assert_eq!(pack.gloss("leave"), Some("Partir; Congé"));
        assert_eq!(pack.level("leave"), Some(CefrLevel::A2));
        assert!(pack.is_dictionary_word("leave"));
        assert!(pack.is_dictionary_word("leaf"));
        assert!(!pack.is_dictionary_word("run"));
        assert_eq!(ud(&pack.readings("leave", "leaves")), [THIRD_SINGULAR]);
        assert_eq!(pack.sense_runs("leave").len(), 2);
        // A form of a lemma reads the lemma's estimates and nothing of its entry.
        assert_eq!(pack.rank("running"), Some(500));
        assert_eq!(pack.gloss("running"), None);
        assert!(pack.readings("running", "running").is_empty());
        assert!(pack.sense_runs("running").is_empty());
    }

    // — grammar tables (`add-lingua-word-grammar`) —

    fn reading(form: &str, lemma: &str, tag: &str, other: bool) -> GrammarReading {
        GrammarReading {
            form: form.into(),
            lemma: lemma.into(),
            tag: tag.into(),
            other,
        }
    }

    const PAST: &str = "VERB|Mood=Ind|Tense=Past|VerbForm=Fin";
    const THIRD_SINGULAR: &str = "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin";

    /// The fixture plus `leave`/`leaf`, and the readings the scenarios need.
    fn inputs_with_grammar() -> PackInputs {
        let mut inp = inputs();
        inp.form_lemma.push(("leaves".into(), "leave".into()));
        inp.ranks
            .extend([("leave".into(), 371), ("leaf".into(), 3_465)]);
        inp.glosses.push(("leave".into(), "Partir; Congé".into()));
        inp.readings = vec![
            reading("ran", "run", PAST, true),
            reading("running", "run", "VERB|VerbForm=Ger", true),
            reading("cities", "city", "NOUN|Number=Plur", true),
            reading("leaves", "leave", THIRD_SINGULAR, false),
            reading("leaves", "leaf", "NOUN|Number=Plur", true),
        ];
        inp.senses = vec![
            ("run".into(), vec![("VERB".into(), 1)]),
            ("leave".into(), vec![("VERB".into(), 1), ("NOUN".into(), 1)]),
        ];
        inp
    }

    fn ud(tags: &[lingua_core::packs::grammar::Tag]) -> Vec<String> {
        tags.iter().map(|t| t.to_ud()).collect()
    }

    #[test]
    fn a_pack_built_with_grammar_answers_readings_and_runs() {
        let bytes = build_pack(&inputs_with_grammar()).expect("build");
        let pack = Pack::load(&bytes).expect("load");
        assert!(pack.has_grammar());
        assert_eq!(ud(&pack.readings("run", "ran")), [PAST]);
        assert_eq!(ud(&pack.readings("leave", "leaves")), [THIRD_SINGULAR]);
        let runs: Vec<(String, usize)> = pack
            .sense_runs("leave")
            .into_iter()
            .map(|(t, n)| (t.unwrap().to_ud(), n))
            .collect();
        assert_eq!(runs, [("VERB".to_owned(), 1), ("NOUN".to_owned(), 1)]);
    }

    #[test]
    fn a_believable_reading_of_another_word_is_filed_under_the_word_the_analysis_reads() {
        let pack = Pack::load(&build_pack(&inputs_with_grammar()).unwrap()).unwrap();
        let others = pack.other_readings("leave", "leaves");
        assert_eq!(others.len(), 1);
        assert_eq!(others[0].0, "leaf");
        assert_eq!(ud(&others[0].1), ["NOUN|Number=Plur"]);
    }

    #[test]
    fn a_reading_that_is_not_believable_is_never_named_as_another_word() {
        let mut inp = inputs_with_grammar();
        for r in &mut inp.readings {
            r.other = false;
        }
        let pack = Pack::load(&build_pack(&inp).unwrap()).unwrap();
        assert!(pack.other_readings("leave", "leaves").is_empty());
        // Its own reading is still there.
        assert_eq!(ud(&pack.readings("leaf", "leaves")), ["NOUN|Number=Plur"]);
    }

    #[test]
    fn a_reading_of_a_word_the_lexicon_does_not_hold_is_dropped() {
        let mut inp = inputs_with_grammar();
        inp.readings.push(reading("swam", "swim", PAST, true));
        let pack = Pack::load(&build_pack(&inp).unwrap()).unwrap();
        assert!(pack.readings("swim", "swam").is_empty());
    }

    #[test]
    fn a_code_outside_the_vocabulary_fails_the_build_by_name() {
        let mut inp = inputs_with_grammar();
        inp.readings
            .push(reading("ran", "run", "VERB|Aspect=Perf", false));
        match build_pack(&inp) {
            Err(BuildError::Grammar(msg)) => assert!(msg.contains("Aspect"), "{msg}"),
            other => panic!("expected a grammar error, got {other:?}"),
        }
        let mut inp = inputs_with_grammar();
        inp.senses.push(("city".into(), vec![("NOM".into(), 1)]));
        match build_pack(&inp) {
            Err(e @ BuildError::Grammar(_)) => assert!(e.to_string().contains("NOM"), "{e}"),
            other => panic!("expected a grammar error, got {other:?}"),
        }
    }

    #[test]
    fn runs_that_disagree_with_their_gloss_fail_the_build_by_word() {
        let mut inp = inputs_with_grammar();
        inp.senses = vec![("leave".into(), vec![("VERB".into(), 1)])];
        match build_pack(&inp) {
            Err(BuildError::Grammar(msg)) => {
                assert!(msg.contains("\"leave\""), "{msg}");
                assert!(msg.contains("cover 1") && msg.contains("holds 2"), "{msg}");
            }
            other => panic!("expected a grammar error, got {other:?}"),
        }
        let mut inp = inputs_with_grammar();
        inp.senses = vec![("leaf".into(), vec![("NOUN".into(), 1)])];
        match build_pack(&inp) {
            Err(BuildError::Grammar(msg)) => assert!(msg.contains("no gloss"), "{msg}"),
            other => panic!("expected a grammar error, got {other:?}"),
        }
    }

    #[test]
    fn a_grammar_less_pack_is_byte_identical_to_before_the_grammar_sections() {
        let bytes = build_pack(&inputs()).unwrap();
        let (_, sections) = lingua_core::packs::format::read_container(&bytes).expect("decode");
        assert!(sections.iter().all(|s| {
            s.name != section::TAGS
                && s.name != section::PARADIGMS_ZST
                && s.name != section::SENSES_ZST
        }));
    }

    #[test]
    fn a_pack_with_grammar_is_rebuilt_byte_for_byte() {
        assert_eq!(
            build_pack(&inputs_with_grammar()).unwrap(),
            build_pack(&inputs_with_grammar()).unwrap()
        );
    }

    #[test]
    fn grammar_needs_no_new_source_and_leaves_the_notice_alone() {
        let plain = inputs();
        let inp = inputs_with_grammar();
        assert_eq!(inp.sources, plain.sources);
        let pack = Pack::load(&build_pack(&inp).unwrap()).unwrap();
        assert_eq!(pack.notice(), plain.notice);
    }

    #[test]
    fn a_pack_whose_language_the_core_cannot_analyse_is_refused_at_build() {
        let mut inp = inputs();
        inp.meta.studied = "pt".into();
        assert_eq!(
            build_pack(&inp),
            Err(BuildError::UnknownLanguage("pt".into()))
        );
        assert!(
            BuildError::UnknownLanguage("pt".into())
                .to_string()
                .contains("\"pt\"")
        );
    }

    #[test]
    fn spec_scenario_a_native_language_the_core_does_not_know_is_refused_at_build() {
        for tag in ["de", "it", "FR", ""] {
            let mut inp = inputs();
            inp.meta.native = tag.into();
            assert_eq!(
                build_pack(&inp),
                Err(BuildError::UnknownNative(tag.into())),
                "{tag:?}"
            );
        }
        let message = BuildError::UnknownNative("de".into()).to_string();
        assert!(message.contains("\"de\""), "{message}");
        // The studied language is still checked first.
        let mut inp = inputs();
        inp.meta.studied = "pt".into();
        inp.meta.native = "de".into();
        assert_eq!(
            build_pack(&inp),
            Err(BuildError::UnknownLanguage("pt".into()))
        );
    }

    #[test]
    fn spec_scenario_a_pack_glossed_in_the_language_it_studies_is_refused_at_build() {
        let mut inp = inputs();
        inp.meta.native = "en".into();
        assert_eq!(
            build_pack(&inp),
            Err(BuildError::NativeStudied("en".into()))
        );
        assert!(
            BuildError::NativeStudied("en".into())
                .to_string()
                .contains("\"en\"")
        );
        // Another native language the core knows builds a pack the core loads.
        inp.meta.native = "es".into();
        let pack = Pack::load(&build_pack(&inp).expect("an en-es pack builds")).expect("loads");
        assert_eq!(pack.pair().key(), "en-es");
        // A French pack glossed in French, now that French is studied (add-lingua-french-baseline).
        let mut french = inputs();
        french.meta.studied = "fr".into();
        french.meta.native = "fr".into();
        french.meta.analyzer_version = StudiedLanguage::French.analyzer_version().into();
        assert_eq!(
            build_pack(&french),
            Err(BuildError::NativeStudied("fr".into()))
        );
    }

    #[test]
    fn a_french_pack_builds_at_french_s_analyser_version_and_loads() {
        // add-lingua-french-baseline: the builder neither stamps nor checks the analyser version
        // (the core compares it at load); a French pack stamped with French's version is one the
        // core accepts, whichever version French's rules have made it.
        let mut inp = inputs();
        inp.meta.studied = "fr".into();
        inp.meta.native = "en".into();
        inp.meta.analyzer_version = FRENCH_ANALYZER_VERSION.into();
        inp.form_lemma = vec![
            ("est".into(), "être".into()),
            ("l'homme".into(), "homme".into()),
        ];
        inp.ranks = vec![("être".into(), 3), ("homme".into(), 90)];
        inp.glosses = vec![("homme".into(), "man".into())];
        let bytes = build_pack(&inp).expect("a fr-en pack builds");
        let pack = Pack::load(&bytes).expect("and loads");
        assert_eq!(pack.studied(), StudiedLanguage::French);
        assert_eq!(pack.pair().key(), "fr-en");
        assert_eq!(pack.meta().analyzer_version, FRENCH_ANALYZER_VERSION);
        assert_eq!(pack.lexicon().lemma_of("l'homme"), Some("homme"));
        assert_eq!(pack.gloss("homme"), Some("man"));
        // Stamped with another language's version, it builds and the core refuses it: Spanish's,
        // since English's version is French's own too (add-lingua-french-detection-guard,
        // ignore-lingua-soft-hyphens).
        inp.meta.analyzer_version = lingua_core::analysis::SPANISH_ANALYZER_VERSION.into();
        let other = build_pack(&inp).expect("the builder does not check the version");
        assert!(Pack::load(&other).is_err());
    }

    #[test]
    fn spec_scenario_a_romance_pack_fits_the_vocabulary() {
        let mut inp = inputs();
        inp.meta.studied = "es".into();
        inp.meta.analyzer_version = StudiedLanguage::Spanish.analyzer_version().into();
        inp.form_lemma = vec![("dijéramos".into(), "decir".into())];
        inp.ranks = vec![
            ("decir".into(), 50),
            ("leche".into(), 900),
            ("yo".into(), 10),
            ("me".into(), 30),
        ];
        inp.glosses = vec![("leche".into(), "Lait".into())];
        inp.readings = vec![
            reading(
                "dijéramos",
                "decir",
                "VERB|Mood=Sub|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin",
                true,
            ),
            reading(
                "me",
                "yo",
                "PRON|Case=Dat|Number=Sing|Person=1|PronType=Prs",
                true,
            ),
        ];
        inp.senses = vec![("leche".into(), vec![("NOUN|Gender=Fem".into(), 1)])];
        let pack = Pack::load(&build_pack(&inp).expect("build")).expect("load");
        assert_eq!(
            ud(&pack.readings("decir", "dijéramos")),
            ["VERB|Mood=Sub|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin"]
        );
        assert_eq!(
            ud(&pack.readings("yo", "me")),
            ["PRON|Case=Dat|Number=Sing|Person=1|PronType=Prs"]
        );
        let runs: Vec<String> = pack
            .sense_runs("leche")
            .into_iter()
            .map(|(t, _)| t.unwrap().to_ud())
            .collect();
        assert_eq!(runs, ["NOUN|Gender=Fem"]);
    }

    #[test]
    fn inputs_from_dir_reads_the_grammar_tables_and_refuses_a_malformed_line() {
        let dir = std::env::temp_dir().join(format!("lingua-grammar-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = serde_json::json!({
            "meta": {"studied": "en", "native": "fr", "pack_version": "t",
                     "analyzer_version": ANALYZER_VERSION, "licences": []},
            "sources": []
        });
        std::fs::write(dir.join("manifest.json"), manifest.to_string()).unwrap();
        for (name, text) in [
            ("forms.tsv", "ran\trun\n"),
            ("freq.tsv", "run\t1\n"),
            ("gloss.tsv", "run\tCourir; Course\n"),
            ("NOTICE", "notice"),
            ("grammar.tsv", format!("ran\trun\t{PAST}\tother\n").as_str()),
            ("senses.tsv", "run\tVERB:1\tNOUN:1\n"),
        ] {
            std::fs::write(dir.join(name), text).unwrap();
        }
        let inp = inputs_from_dir(&dir).expect("read");
        assert_eq!(inp.readings, [reading("ran", "run", PAST, true)]);
        assert_eq!(
            inp.senses,
            [(
                "run".to_owned(),
                vec![("VERB".to_owned(), 1), ("NOUN".to_owned(), 1)]
            )]
        );
        std::fs::write(dir.join("grammar.tsv"), "ran\trun\n").unwrap();
        assert!(inputs_from_dir(&dir).is_err());
        std::fs::write(
            dir.join("grammar.tsv"),
            format!("ran\trun\t{PAST}\tmaybe\n"),
        )
        .unwrap();
        assert!(inputs_from_dir(&dir).is_err());
        std::fs::write(dir.join("grammar.tsv"), "").unwrap();
        std::fs::write(dir.join("senses.tsv"), "run\tVERB\n").unwrap();
        assert!(inputs_from_dir(&dir).is_err());
        std::fs::write(dir.join("senses.tsv"), "run\n").unwrap();
        assert!(inputs_from_dir(&dir).is_err());
        std::fs::remove_file(dir.join("senses.tsv")).unwrap();
        std::fs::remove_file(dir.join("grammar.tsv")).unwrap();
        let inp = inputs_from_dir(&dir).expect("tables are optional");
        assert!(inp.readings.is_empty() && inp.senses.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn over_budget_names_the_grammar_readings_when_they_push_the_pack_over() {
        let mut inp = inputs();
        let mut x: u32 = 0x9e37_79b9;
        inp.readings = (0..40_000)
            .map(|i| {
                // Near-uniform printable ASCII, as `incompressible_gloss`, up to
                // the 255 bytes a stored suffix may hold.
                let mut form = format!("run{i}");
                for _ in 0..240 {
                    x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    form.push(char::from(b' ' + (x >> 17) as u8 % 95));
                }
                reading(&form, "run", PAST, false)
            })
            .collect();
        match build_pack(&inp) {
            Err(e @ BuildError::OverBudget { .. }) => {
                let msg = e.to_string();
                assert!(msg.contains("grammar readings"), "{msg}");
                assert!(!msg.contains("glossed lemmas"), "{msg}");
            }
            Err(other) => panic!("expected OverBudget, got {other:?}"),
            Ok(pack) => panic!("expected OverBudget, got a {}-byte pack", pack.len()),
        }
    }

    // — the lexical table, the pinned pool, a noun's gender (add-lingua-pack-lexical-layer) —

    /// A built pack's section, by name.
    fn section_of(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
        let (_, sections) = lingua_core::packs::format::read_container(bytes).expect("decode");
        sections
            .into_iter()
            .find(|s| s.name == name)
            .map(|s| s.data)
    }

    fn owned(lemmas: &[&str]) -> Option<Vec<String>> {
        Some(lemmas.iter().map(|l| (*l).to_owned()).collect())
    }

    #[test]
    fn spec_scenario_a_lexical_table_that_says_what_the_glosses_say() {
        let without = build_pack(&inputs()).unwrap();
        // The lemmas the pack glosses, in any order: no table, the same bytes.
        for lexical in [&["city", "run"][..], &["run", "city", "run"]] {
            let mut inp = inputs();
            inp.lexical = owned(lexical);
            assert_eq!(build_pack(&inp).unwrap(), without, "{lexical:?}");
        }
        assert!(section_of(&without, section::LEXICAL).is_none());
    }

    #[test]
    fn a_lexical_table_that_differs_is_written_and_read_back() {
        let mut inp = inputs();
        // `city` is glossed but no dictionary word; `seldom`, ranked and unglossed, is one.
        inp.ranks.push(("seldom".into(), 5_100));
        inp.lexical = owned(&["run", "seldom"]);
        let bytes = build_pack(&inp).expect("build");
        assert_eq!(bytes, build_pack(&inp).unwrap(), "deterministic");
        let pack = Pack::load(&bytes).expect("load");
        // city, run, seldom: ids 0, 1, 2; one byte, bits 1 and 2.
        assert_eq!(section_of(&bytes, section::LEXICAL), Some(vec![0b110]));
        assert!(pack.is_dictionary_word("run"));
        assert!(pack.is_dictionary_word("seldom"));
        assert!(!pack.is_dictionary_word("city"));
        assert_eq!(pack.gloss("city"), Some("ville"));
        assert_eq!(
            pack.dictionary_words(),
            vec![("run", 500), ("seldom", 5_100)]
        );
        // The studied sections do not move with it.
        let plain = {
            let mut inp = inputs();
            inp.ranks.push(("seldom".into(), 5_100));
            build_pack(&inp).unwrap()
        };
        for name in [section::FORMS, section::LEMMAS, section::FREQ] {
            assert_eq!(section_of(&bytes, name), section_of(&plain, name), "{name}");
        }
    }

    #[test]
    fn spec_scenario_an_older_core() {
        // A pack carrying a lexical table: `city` is glossed and no dictionary word, `seldom`
        // is a dictionary word with no gloss.
        let mut inp = inputs();
        inp.ranks.push(("seldom".into(), 5_100));
        let plain = build_pack(&inp).unwrap();
        inp.lexical = owned(&["run", "seldom"]);
        let bytes = build_pack(&inp).unwrap();
        // A core built before this change reads the container whatever its sections are named,
        // and looks up by name only the sections it knows, never `lexical`: what it reads is
        // the container without that section — byte for byte the pack built without a
        // lexical.tsv.
        let (meta, sections) =
            lingua_core::packs::format::read_container(&bytes).expect("the container reads");
        assert!(sections.iter().any(|s| s.name == section::LEXICAL));
        let known: Vec<(&str, &[u8])> = sections
            .iter()
            .filter(|s| s.name != section::LEXICAL)
            .map(|s| (s.name.as_str(), s.data.as_slice()))
            .collect();
        let read_by_an_older_core = write_container(&meta, &known);
        assert_eq!(read_by_an_older_core, plain);
        // The pack loads, and that core reads its glossed lemmas as its dictionary words.
        let pack = Pack::load(&read_by_an_older_core).expect("the pack loads");
        for lemma in ["city", "run", "seldom", "gun"] {
            assert_eq!(
                pack.is_dictionary_word(lemma),
                pack.gloss(lemma).is_some(),
                "{lemma}"
            );
        }
        assert_eq!(pack.dictionary_words(), vec![("city", 1_200), ("run", 500)]);
    }

    #[test]
    fn spec_scenario_a_gloss_outside_the_lexicon() {
        // `leaf` is glossed, and no form maps to it and no rank lists it.
        let mut inp = inputs();
        inp.glosses.push(("leaf".into(), "Feuille".into()));
        inp.lexical = owned(&["run"]);
        match build_pack(&inp) {
            Err(e @ BuildError::Lexical(_)) => {
                assert!(e.to_string().contains("\"leaf\""), "{e}");
                assert!(e.to_string().contains("lexical table"), "{e}");
            }
            other => panic!("expected a lexical error, got {other:?}"),
        }
        // A dictionary word outside the lexicon is refused by name too.
        let mut inp = inputs();
        inp.lexical = owned(&["run", "gun"]);
        match build_pack(&inp) {
            Err(BuildError::Lexical(msg)) => assert!(msg.contains("\"gun\""), "{msg}"),
            other => panic!("expected a lexical error, got {other:?}"),
        }
        // Without a table, the outlier stays legal (the testdata fixture has three, on
        // purpose); a table that says what the glosses say is no table, and refuses nothing.
        let mut inp = inputs();
        inp.glosses.push(("leaf".into(), "Feuille".into()));
        let without = build_pack(&inp).expect("an outlier builds without a table");
        inp.lexical = owned(&["city", "leaf", "run"]);
        assert_eq!(build_pack(&inp).unwrap(), without);
    }

    #[test]
    fn inputs_from_dir_reads_the_lexical_table_and_the_pinned_pool() {
        let dir = std::env::temp_dir().join(format!("lingua-lexical-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = serde_json::json!({
            "meta": {"studied": "en", "native": "fr", "pack_version": "t",
                     "analyzer_version": ANALYZER_VERSION, "licences": []},
            "sources": []
        });
        std::fs::write(dir.join("manifest.json"), manifest.to_string()).unwrap();
        for (name, text) in [
            ("forms.tsv", "ran\trun\n"),
            ("freq.tsv", "run\t1\n"),
            ("gloss.tsv", "run\tCourir\n"),
            ("NOTICE", "notice"),
        ] {
            std::fs::write(dir.join(name), text).unwrap();
        }
        let inp = inputs_from_dir(&dir).expect("read");
        assert_eq!((inp.lexical, inp.tag_pool), (None, None));
        std::fs::write(dir.join(LEXICAL_TABLE), "run\n\n  walk \n").unwrap();
        std::fs::write(dir.join(TAG_POOL_TABLE), "VERB\nNOUN|Number=Plur\n").unwrap();
        let inp = inputs_from_dir(&dir).expect("read");
        assert_eq!(inp.lexical, owned(&["run", "walk"]));
        assert_eq!(inp.tag_pool, owned(&["VERB", "NOUN|Number=Plur"]));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn spec_scenario_a_pair_reads_its_studied_side_from_its_language_s_folder() {
        // split-lingua-pack-tables-by-language: tables/<studied>/ and tables/<pair>/.
        let root = std::env::temp_dir().join(format!("lingua-split-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (en, en_fr) = (root.join("en"), root.join("en-fr"));
        std::fs::create_dir_all(&en).unwrap();
        std::fs::create_dir_all(&en_fr).unwrap();
        let manifest = serde_json::json!({
            "meta": {"studied": "en", "native": "fr", "pack_version": "t",
                     "analyzer_version": ANALYZER_VERSION, "licences": []},
            "sources": []
        });
        std::fs::write(en_fr.join("manifest.json"), manifest.to_string()).unwrap();
        for (dir, name, text) in [
            (&en, "forms.tsv", "ran\trun\n"),
            (&en, "freq.tsv", "run\t1\n"),
            (&en, TAG_POOL_TABLE, "VERB\n"),
            (&en, LEXICAL_TABLE, "run\n"),
            (&en_fr, "gloss.tsv", "run\tCourir\n"),
            (&en_fr, "NOTICE", "notice"),
        ] {
            std::fs::write(dir.join(name), text).unwrap();
        }
        let inp = inputs_from_tables(&root, "en-fr").expect("read");
        assert_eq!(inp.form_lemma, [("ran".to_owned(), "run".to_owned())]);
        assert_eq!(inp.glosses, [("run".to_owned(), "Courir".to_owned())]);
        assert_eq!(
            (inp.lexical, inp.tag_pool),
            (owned(&["run"]), owned(&["VERB"]))
        );
        // A studied table left in the pair's folder is refused, by path; the folder's
        // record names the reduction that writes the studied side.
        std::fs::write(en_fr.join("forms.tsv"), "ran\trun\n").unwrap();
        let err = inputs_from_tables(&root, "en-fr")
            .err()
            .expect("refused")
            .to_string();
        assert!(
            err.contains("en-fr/forms.tsv") && err.contains("wrong side"),
            "{err}"
        );
        assert!(!err.contains("reduction"), "no record yet: {err}");
        std::fs::write(
            en.join(tables::STUDIED_RECORD),
            "{\"reference\": \"en-fr\"}\n",
        )
        .unwrap();
        let err = inputs_from_tables(&root, "en-fr")
            .err()
            .expect("refused")
            .to_string();
        assert!(err.contains("written by en-fr's reduction"), "{err}");
        std::fs::remove_file(en.join(tables::STUDIED_RECORD)).unwrap();
        std::fs::remove_file(en_fr.join("forms.tsv")).unwrap();
        // A required file missing on either side is named, with its side.
        for (dir, name) in [(&en, "freq.tsv"), (&en_fr, "gloss.tsv")] {
            let kept = std::fs::read_to_string(dir.join(name)).unwrap();
            std::fs::remove_file(dir.join(name)).unwrap();
            let err = inputs_from_tables(&root, "en-fr")
                .err()
                .expect("missing")
                .to_string();
            assert!(
                err.contains(&format!("{}", dir.join(name).display())),
                "{name}: {err}"
            );
            std::fs::write(dir.join(name), kept).unwrap();
        }
        // So is a native table in the studied folder.
        std::fs::write(en.join("gloss.tsv"), "run\tCourir\n").unwrap();
        let err = inputs_from_tables(&root, "en-fr")
            .err()
            .expect("refused")
            .to_string();
        assert!(
            err.contains("en/gloss.tsv") && err.contains("wrong side"),
            "{err}"
        );
        std::fs::remove_file(en.join("gloss.tsv")).unwrap();
        // A pair whose studied language has no folder: the error names the folder to add.
        std::fs::rename(&en_fr, root.join("de-fr")).unwrap();
        let err = inputs_from_tables(&root, "de-fr")
            .err()
            .expect("no studied folder")
            .to_string();
        assert!(
            err.contains(&format!("{}", root.join("de").display())) && err.contains("de-fr"),
            "{err}"
        );
        // spec: *A pair whose manifest studies another language* — a pair named es-en whose
        // manifest studies English fails, naming the pair and both languages.
        std::fs::rename(root.join("de-fr"), root.join("es-en")).unwrap();
        std::fs::create_dir_all(root.join("es")).unwrap();
        for name in ["forms.tsv", "freq.tsv"] {
            std::fs::copy(en.join(name), root.join("es").join(name)).unwrap();
        }
        let err = inputs_from_tables(&root, "es-en")
            .err()
            .expect("refused")
            .to_string();
        assert!(
            err.contains("es-en/manifest.json studies \"en\"") && err.contains("of \"es\""),
            "{err}"
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_two_folder_build_equals_the_single_folder_build() {
        // split-lingua-pack-tables-by-language D4: the testdata fixtures hold both sides in one
        // folder; filed by side into two folders, they build the same bytes.
        for pair in ["en-fr", "es-fr"] {
            let single = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../scripts/lingua-data/testdata")
                .join(pair);
            let root = std::env::temp_dir()
                .join(format!("lingua-two-folders-{pair}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            let (studied, own) = (root.join(studied_of(pair)), root.join(pair));
            std::fs::create_dir_all(&studied).unwrap();
            std::fs::create_dir_all(&own).unwrap();
            for entry in std::fs::read_dir(&single).unwrap() {
                let name = entry.unwrap().file_name();
                let side = if STUDIED_SIDE.contains(&name.to_str().unwrap()) {
                    &studied
                } else {
                    &own
                };
                std::fs::copy(single.join(&name), side.join(&name)).unwrap();
            }
            let one = build_pack(&inputs_from_dir(&single).expect("read")).expect("build");
            let two =
                build_pack(&inputs_from_tables(&root, pair).expect("read two")).expect("build");
            assert!(one == two, "{pair}: the two-folder build differs");
            assert!(
                std::fs::read_dir(&studied).unwrap().count() >= 2,
                "{pair}: the studied side is read from its own folder"
            );
            std::fs::remove_dir_all(&root).unwrap();
        }
    }

    /// The pool a built pack carries, one tag per line.
    fn pool_of(bytes: &[u8]) -> Vec<String> {
        let tags = section_of(bytes, section::TAGS).expect("a tag pool");
        String::from_utf8(tags)
            .unwrap()
            .split('\n')
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn spec_scenario_a_sense_part_of_speech_the_first_pack_never_used() {
        let reference = build_pack(&inputs_with_grammar()).unwrap();
        let pin = pool_of(&reference);
        // `city`'s gloss gains a run tagged NUM, which no reading uses and the pin lacks.
        let numbered = || {
            let mut inp = inputs_with_grammar();
            inp.senses.push(("city".into(), vec![("NUM".into(), 1)]));
            inp
        };
        let mut pinned = numbered();
        pinned.tag_pool = Some(pin.clone());
        let bytes = build_pack(&pinned).expect("build");
        // Its readings are stored as the reference's, and the pool only grows at its end.
        assert_eq!(
            section_of(&bytes, section::PARADIGMS_ZST),
            section_of(&reference, section::PARADIGMS_ZST)
        );
        let pool = pool_of(&bytes);
        assert_eq!(pool[..pin.len()], pin[..]);
        assert_eq!(pool[pin.len()..], ["NUM"]);
        let pack = Pack::load(&bytes).expect("load");
        let runs: Vec<String> = pack
            .sense_runs("city")
            .into_iter()
            .map(|(t, _)| t.unwrap().to_ud())
            .collect();
        assert_eq!(runs, ["NUM"]);
        assert_eq!(ud(&pack.readings("run", "ran")), [PAST]);
        // Without the pin, the single sorted pool moves every reading filed after NUM.
        let unpinned = build_pack(&numbered()).unwrap();
        assert_ne!(
            section_of(&unpinned, section::PARADIGMS_ZST),
            section_of(&reference, section::PARADIGMS_ZST)
        );
        // The pin that is the pool builds the bytes the pack had without it.
        let mut same = inputs_with_grammar();
        same.tag_pool = Some(pin);
        assert_eq!(build_pack(&same).unwrap(), reference);
    }

    #[test]
    fn spec_scenario_fewer_sense_tags_leave_the_readings_alone() {
        let reference = build_pack(&inputs_with_grammar()).unwrap();
        let mut fewer = inputs_with_grammar();
        // No run tagged VERB any more: `run`'s gloss has none, `leave`'s keeps its noun run.
        fewer.senses = vec![("leave".into(), vec![("NOUN".into(), 2)])];
        fewer.tag_pool = Some(pool_of(&reference));
        let bytes = build_pack(&fewer).expect("build");
        assert_eq!(
            section_of(&bytes, section::PARADIGMS_ZST),
            section_of(&reference, section::PARADIGMS_ZST)
        );
        assert_eq!(pool_of(&bytes), pool_of(&reference));
    }

    #[test]
    fn a_pinned_pool_comes_first_then_reading_tags_then_sense_only_tags() {
        let mut inp = inputs_with_grammar();
        inp.senses.push(("city".into(), vec![("NUM".into(), 1)]));
        inp.tag_pool = Some(vec!["VERB|VerbForm=Ger".into(), "X".into()]);
        let bytes = build_pack(&inp).expect("build");
        assert_eq!(
            pool_of(&bytes),
            [
                "VERB|VerbForm=Ger",
                "X",
                // The readings' other tags, sorted.
                "NOUN|Number=Plur",
                THIRD_SINGULAR,
                PAST,
                // Then the tags only senses carry, sorted.
                "NOUN",
                "NUM",
                "VERB",
            ]
        );
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(ud(&pack.readings("run", "running")), ["VERB|VerbForm=Ger"]);
        assert_eq!(ud(&pack.readings("leave", "leaves")), [THIRD_SINGULAR]);
    }

    #[test]
    fn a_pinned_pool_is_canonical_and_lists_each_tag_once() {
        for (pin, named) in [
            (vec!["VERB|VerbForm=Ger|Mood=Ind"], "VerbForm=Ger|Mood=Ind"),
            (vec!["VERB", "VERB"], "twice"),
            (vec!["NOM"], "NOM"),
        ] {
            let mut inp = inputs_with_grammar();
            inp.tag_pool = Some(pin.iter().map(|t| (*t).to_owned()).collect());
            match build_pack(&inp) {
                Err(e @ BuildError::Grammar(_)) => {
                    assert!(e.to_string().contains(named), "{e}");
                    assert!(e.to_string().contains("pinned tag"), "{e}");
                }
                other => panic!("expected a grammar error for {pin:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn spec_scenario_a_reading_outside_the_pinned_pool() {
        // Spanish's pinned pool, as committed beside the es-fr tables: it lacks the clitic's
        // dative tag.
        const CLITIC: &str = "PRON|Case=Dat|Number=Sing|Person=1|PronType=Prs";
        let es = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/tables/es");
        let pin = read_lines(&es, TAG_POOL_TABLE)
            .expect("read")
            .expect("tables/es pins Spanish's tag pool");
        assert!(!pin.iter().any(|tag| tag == CLITIC), "{CLITIC} is pinned");
        // The archived Romance pack, built with that pin.
        let mut inp = inputs();
        inp.meta.studied = "es".into();
        inp.meta.analyzer_version = StudiedLanguage::Spanish.analyzer_version().into();
        inp.form_lemma = vec![("dijéramos".into(), "decir".into())];
        inp.ranks = vec![
            ("decir".into(), 50),
            ("yo".into(), 10),
            ("leche".into(), 900),
        ];
        inp.glosses = vec![("leche".into(), "Lait".into())];
        inp.readings = vec![
            reading(
                "dijéramos",
                "decir",
                "VERB|Mood=Sub|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin",
                true,
            ),
            reading(
                "me",
                "yo",
                "PRON|Case=Dat|Number=Sing|Person=1|PronType=Prs",
                true,
            ),
        ];
        inp.senses = vec![("leche".into(), vec![("NOUN".into(), 1)])];
        inp.tag_pool = Some(pin.clone());
        let bytes = build_pack(&inp).expect("build");
        // The pin first, then the one reading tag it lacks.
        let pool = pool_of(&bytes);
        assert_eq!(pool[..pin.len()], pin[..]);
        assert_eq!(pool[pin.len()..], [CLITIC]);
        // The core reads that reading back, and the pinned one beside it.
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(ud(&pack.readings("yo", "me")), [CLITIC]);
        assert_eq!(
            ud(&pack.readings("decir", "dijéramos")),
            ["VERB|Mood=Sub|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin"]
        );
    }

    /// A Spanish pack: `casa` and `estudiante`, each read as itself, glossed with one noun
    /// run each, as `runs` tags them.
    fn spanish_nouns(casa: &str, estudiante: &str) -> PackInputs {
        let mut inp = inputs();
        inp.meta.studied = "es".into();
        inp.meta.native = "en".into();
        inp.meta.analyzer_version = StudiedLanguage::Spanish.analyzer_version().into();
        inp.form_lemma = vec![("casas".into(), "casa".into())];
        inp.ranks = vec![("casa".into(), 300), ("estudiante".into(), 900)];
        inp.glosses = vec![
            ("casa".into(), "House".into()),
            ("estudiante".into(), "Student".into()),
        ];
        inp.readings = vec![
            reading("casa", "casa", "NOUN|Gender=Fem|Number=Sing", false),
            reading("casas", "casa", "NOUN|Gender=Fem|Number=Plur", false),
            reading(
                "estudiante",
                "estudiante",
                "NOUN|Gender=Fem|Number=Sing",
                false,
            ),
            reading(
                "estudiante",
                "estudiante",
                "NOUN|Gender=Masc|Number=Sing",
                false,
            ),
        ];
        inp.senses = vec![
            ("casa".into(), vec![(casa.into(), 1)]),
            ("estudiante".into(), vec![(estudiante.into(), 1)]),
        ];
        inp
    }

    fn runs_of(pack: &Pack, lemma: &str) -> Vec<String> {
        pack.sense_runs(lemma)
            .into_iter()
            .map(|(t, _)| t.unwrap().to_ud())
            .collect()
    }

    #[test]
    fn spec_scenario_a_bare_noun_run_gains_the_gender_its_readings_give() {
        let bytes = build_pack(&spanish_nouns("NOUN", "NOUN")).expect("build");
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(runs_of(&pack, "casa"), ["NOUN|Gender=Fem"]);
        // The same pack whatever the glosses said of the gender they agree with.
        assert_eq!(
            build_pack(&spanish_nouns("NOUN|Gender=Fem", "NOUN")).unwrap(),
            bytes
        );
    }

    #[test]
    fn spec_scenario_a_noun_of_both_genders_keeps_a_bare_run() {
        let bare = build_pack(&spanish_nouns("NOUN", "NOUN")).unwrap();
        assert_eq!(runs_of(&Pack::load(&bare).unwrap(), "estudiante"), ["NOUN"]);
        // Whatever gender the sense table names, readings of both genders leave the run
        // bare: the pack is the one built from bare runs, whatever the glosses said.
        for named in ["NOUN|Gender=Masc", "NOUN|Gender=Fem"] {
            let bytes = build_pack(&spanish_nouns("NOUN", named)).expect(named);
            assert_eq!(
                runs_of(&Pack::load(&bytes).unwrap(), "estudiante"),
                ["NOUN"],
                "{named}"
            );
            assert_eq!(bytes, bare, "{named}");
        }
    }

    #[test]
    fn spec_scenario_a_noun_read_with_no_gender() {
        // `leche`, ranked and glossed, with no reading at all: its run is neither given a
        // gender nor stripped of one.
        let leche = |run: &str| {
            let mut inp = spanish_nouns("NOUN", "NOUN");
            inp.ranks.push(("leche".into(), 1_500));
            inp.glosses.push(("leche".into(), "Milk".into()));
            inp.senses.push(("leche".into(), vec![(run.into(), 1)]));
            let pack = Pack::load(&build_pack(&inp).expect(run)).unwrap();
            runs_of(&pack, "leche")
        };
        assert_eq!(leche("NOUN|Gender=Fem"), ["NOUN|Gender=Fem"]);
        assert_eq!(leche("NOUN"), ["NOUN"]);
        // A noun with no gendered reading, an English one, keeps its run as it is.
        let pack = Pack::load(&build_pack(&inputs_with_grammar()).unwrap()).unwrap();
        assert_eq!(runs_of(&pack, "leave"), ["VERB", "NOUN"]);
    }

    #[test]
    fn spec_scenario_a_run_its_readings_contradict() {
        match build_pack(&spanish_nouns("NOUN|Gender=Masc", "NOUN")) {
            Err(e @ BuildError::Grammar(_)) => {
                let msg = e.to_string();
                assert!(msg.contains("\"casa\""), "{msg}");
                assert!(
                    msg.contains("Gender=Masc") && msg.contains("Gender=Fem"),
                    "{msg}"
                );
            }
            other => panic!("expected a grammar error, got {other:?}"),
        }
    }

    // — a base and a layer (split-lingua-pack-format) —

    /// The grammar fixture with a pinned pool holding two of its readings' tags — so its pool
    /// has three blocks: the pin, the two other readings' tags, and `NOUN` and `VERB`, which
    /// only its senses carry —, a level, and an expression.
    fn pinned_inputs() -> PackInputs {
        let mut inp = inputs_with_grammar();
        inp.tag_pool = Some(vec!["VERB|VerbForm=Ger".into(), PAST.into()]);
        inp.levels = vec![("run".into(), CefrLevel::A1)];
        inp.expressions = vec![("run city".into(), "course urbaine".into())];
        inp
    }

    /// A container's section names, in its order.
    fn names_of(bytes: &[u8]) -> Vec<String> {
        let (_, sections) = read_container(bytes).expect("a container");
        sections.into_iter().map(|s| s.name).collect()
    }

    /// A container's sections, by name.
    fn by_name(sections: Vec<lingua_core::packs::Section>) -> BTreeMap<String, Vec<u8>> {
        sections.into_iter().map(|s| (s.name, s.data)).collect()
    }

    #[test]
    fn spec_scenario_the_same_pack_whole_or_in_parts() {
        let inp = pinned_inputs();
        let whole = build_pack(&inp).expect("whole");
        let split = build_split(&inp).expect("split");
        // Assembled, the parts hold every named section of the whole pack with the same bytes,
        // the pool included, and its metadata as the whole pack writes it.
        let (meta, sections) =
            Pack::assemble_sections(&split.base, &split.layer).expect("assemble");
        let (whole_meta, whole_sections) = read_container(&whole).unwrap();
        assert_eq!(by_name(sections), by_name(whole_sections));
        assert_eq!(serde_json::to_vec(&meta).unwrap(), whole_meta);
        // What each part holds (D2), in the whole pack's order, `senses.tags` where the pool
        // ends.
        assert_eq!(
            names_of(&split.base),
            ["forms", "lemmas", "freq", "levels", "tags", "paradigms.zst"]
        );
        assert_eq!(
            names_of(&split.layer),
            [
                "expr",
                "expr.zst",
                "senses.tags",
                "senses.zst",
                "gloss.zst",
                "notice"
            ]
        );
        // The base's pool is the pin and the readings' other tags; the layer's sense tags are
        // the tags only the senses carry, sorted (D5).
        assert_eq!(
            section_of(&split.base, section::TAGS).unwrap(),
            format!("VERB|VerbForm=Ger\n{PAST}\nNOUN|Number=Plur\n{THIRD_SINGULAR}").into_bytes()
        );
        assert_eq!(
            section_of(&split.layer, section::SENSES_TAGS).unwrap(),
            b"NOUN\nVERB"
        );
        // The layer names the base it was built with, whose digest any reader recomputes (D4).
        let (base_meta, layer_meta) = (
            BaseMeta::read(&split.base).unwrap(),
            LayerMeta::read(&split.layer).unwrap(),
        );
        assert_eq!(layer_meta.base_version, base_meta.base_version);
        assert_eq!(
            base_version(&split.base),
            Some(base_meta.base_version.clone())
        );
        assert_eq!(base_meta.base_version.len(), 64);
        assert_eq!(
            (
                base_meta.studied.as_str(),
                base_meta.analyzer_version.as_str()
            ),
            ("en", ANALYZER_VERSION)
        );
        assert_eq!(layer_meta.pair_key(), "en-fr");
        assert_eq!(layer_meta.pack_version, inp.meta.pack_version);
        assert_eq!(layer_meta.licences, inp.meta.licences);
        // The assembled pack reads as the whole one.
        let pack = Pack::assemble(&split.base, &split.layer).expect("load");
        assert_eq!(ud(&pack.readings("run", "ran")), [PAST]);
        let runs: Vec<String> = pack
            .sense_runs("leave")
            .into_iter()
            .map(|(t, _)| t.unwrap().to_ud())
            .collect();
        assert_eq!(runs, ["VERB", "NOUN"]);
        assert_eq!(pack.expression("run city"), Some("course urbaine"));
        // Deterministic, as the whole pack is.
        assert_eq!(build_split(&inp).unwrap(), split);
    }

    #[test]
    fn a_base_depends_on_the_studied_side_alone() {
        // Another native side — glosses, runs carrying another tag of their own, expressions,
        // NOTICE, pair and version —: the same base, another layer.
        let inp = pinned_inputs();
        let mut other = pinned_inputs();
        other.meta.native = "es".into();
        other.meta.pack_version = "2026.10.21".into();
        other.glosses = vec![
            ("run".into(), "correr".into()),
            ("city".into(), "ciudad".into()),
            ("leave".into(), "Salir; Permiso".into()),
        ];
        other.senses = vec![
            ("run".into(), vec![("VERB".into(), 1)]),
            ("leave".into(), vec![("VERB".into(), 1), ("ADV".into(), 1)]),
        ];
        other.expressions.clear();
        other.notice.push_str(" Wikcionario.");
        let (a, b) = (build_split(&inp).unwrap(), build_split(&other).unwrap());
        assert_eq!(a.base, b.base);
        assert_ne!(a.layer, b.layer);
        assert_eq!(
            section_of(&b.layer, section::SENSES_TAGS).unwrap(),
            b"ADV\nVERB"
        );
        // es's layer assembles with en-fr's base: they are one base.
        let pack = Pack::assemble(&a.base, &b.layer).expect("one base");
        assert_eq!(pack.pair().key(), "en-es");
        assert_eq!(pack.gloss("city"), Some("ciudad"));
    }

    #[test]
    fn spec_scenario_inputs_without_a_pinned_pool() {
        let inp = inputs_with_grammar();
        assert!(inp.tag_pool.is_none());
        match build_split(&inp) {
            Err(e @ BuildError::NoPinnedPool) => {
                assert!(e.to_string().contains(TAG_POOL_TABLE), "{e}");
            }
            other => panic!("expected NoPinnedPool, got {other:?}"),
        }
        assert!(build_pack(&inp).is_ok(), "the whole pack builds as before");
    }

    #[test]
    fn one_pass_builds_the_whole_pack_and_its_parts() {
        let inp = pinned_inputs();
        let (whole, split) = build_whole_and_split(&inp).expect("both");
        assert_eq!(whole, build_pack(&inp).unwrap());
        assert_eq!(split, build_split(&inp).unwrap());
        // Refused as `build_split` refuses.
        assert_eq!(
            build_whole_and_split(&inputs_with_grammar()),
            Err(BuildError::NoPinnedPool)
        );
    }

    #[test]
    fn spec_scenario_inputs_that_would_make_the_base_the_pair_s() {
        // A glossed lemma the studied tables do not hold: the pair's glosses would add a lemma
        // to the base.
        let mut inp = pinned_inputs();
        inp.glosses.push(("seldom".into(), "rarement".into()));
        let err = build_split(&inp).unwrap_err();
        assert!(matches!(err, BuildError::BaseFollowsPair(_)), "{err:?}");
        assert!(err.to_string().contains("\"seldom\" is glossed"), "{err}");
        assert!(build_pack(&inp).is_ok(), "the whole pack builds as before");
        // Dictionary words that are the pair's glossed lemmas: the base would carry no lexical
        // table, and read the layer's glosses as its words.
        let mut inp = pinned_inputs();
        inp.lexical = Some(vec!["city".into(), "leave".into(), "run".into()]);
        let err = build_split(&inp).unwrap_err();
        assert!(err.to_string().contains(LEXICAL_TABLE), "{err}");
        let whole = build_pack(&inp).unwrap();
        assert!(section_of(&whole, section::LEXICAL).is_none());
        // Dictionary words of their own: the base carries them, and splits.
        inp.lexical = Some(vec!["city".into(), "run".into()]);
        let split = build_split(&inp).unwrap();
        assert!(section_of(&split.base, section::LEXICAL).is_some());
    }

    #[test]
    fn a_language_that_pins_its_pool_writes_it_readings_or_not() {
        // No readings, no runs: the pin alone is the pool, in the whole pack and in the base.
        let mut inp = inputs();
        inp.tag_pool = Some(vec!["NOUN".into(), "VERB".into()]);
        let whole = build_pack(&inp).unwrap();
        assert_eq!(section_of(&whole, section::TAGS).unwrap(), b"NOUN\nVERB");
        let split = build_split(&inp).unwrap();
        assert_eq!(
            section_of(&split.base, section::TAGS).unwrap(),
            b"NOUN\nVERB"
        );
        assert_eq!(names_of(&split.layer), ["gloss.zst", "notice"]);
        let (_, sections) = Pack::assemble_sections(&split.base, &split.layer).unwrap();
        assert_eq!(
            by_name(sections),
            by_name(read_container(&whole).unwrap().1)
        );
        // Without a pin, nothing moves: no grammar, no pool.
        assert!(section_of(&build_pack(&inputs()).unwrap(), section::TAGS).is_none());
        // A pin with nothing in it writes no pool, and builds the parts.
        inp.tag_pool = Some(Vec::new());
        let split = build_split(&inp).unwrap();
        assert!(section_of(&split.base, section::TAGS).is_none());
    }

    /// The part `build_split` refuses as over budget, and its message.
    fn over_budget_part(inp: &PackInputs) -> (Part, Option<&'static str>, String) {
        match build_split(inp) {
            Err(
                e @ BuildError::PartOverBudget {
                    part,
                    size,
                    budget,
                    reduce,
                },
            ) => {
                assert_eq!(budget, MAX_PACK_BYTES);
                assert!(size > budget);
                (part, reduce, e.to_string())
            }
            Err(other) => panic!("expected PartOverBudget, got {other:?}"),
            Ok(split) => panic!(
                "expected PartOverBudget, got a {}-byte base and a {}-byte layer",
                split.base.len(),
                split.layer.len()
            ),
        }
    }

    #[test]
    fn spec_scenario_a_layer_over_budget() {
        // Its glosses: the layer names its glossed lemmas.
        let mut inp = inputs();
        inp.tag_pool = Some(Vec::new());
        inp.glosses = vec![("run".into(), incompressible_gloss())];
        let (part, reduce, message) = over_budget_part(&inp);
        assert_eq!((part, reduce), (Part::Layer, Some(REDUCE_GLOSSES)));
        assert!(message.starts_with("the layer is "), "{message}");
        assert!(message.contains("glossed lemmas"), "{message}");
        // Its expression table: the layer names it.
        let mut inp = inputs_with_expression_words();
        inp.tag_pool = Some(Vec::new());
        inp.expressions = vec![("run city".into(), incompressible_gloss())];
        let (part, reduce, message) = over_budget_part(&inp);
        assert_eq!((part, reduce), (Part::Layer, Some(REDUCE_EXPRESSIONS)));
        assert!(message.contains("expression table"), "{message}");
    }

    #[test]
    fn spec_scenario_a_base_over_budget() {
        // Its readings: the base names the readings of its rarest dictionary forms.
        let mut inp = inputs();
        inp.tag_pool = Some(Vec::new());
        let mut x: u32 = 0x9e37_79b9;
        inp.readings = (0..40_000)
            .map(|i| {
                let mut form = format!("run{i}");
                for _ in 0..240 {
                    x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    form.push(char::from(b' ' + (x >> 17) as u8 % 95));
                }
                reading(&form, "run", PAST, false)
            })
            .collect();
        let (part, reduce, message) = over_budget_part(&inp);
        assert_eq!((part, reduce), (Part::Base, Some(REDUCE_READINGS)));
        assert!(message.starts_with("the base is "), "{message}");
        assert!(message.contains("grammar readings"), "{message}");
        // A base carrying no readings names no remedy: its forms and frequencies are what every
        // analysis depends on.
        let err = part_budget(Part::Base, MAX_PACK_BYTES + 1, None).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("carries nothing to reduce"), "{message}");
        assert!(message.contains("forms and frequencies"), "{message}");
        assert!(part_budget(Part::Base, MAX_PACK_BYTES, None).is_ok());
    }

    #[test]
    fn a_base_version_is_read_from_a_container_alone() {
        let split = build_split(&pinned_inputs()).unwrap();
        // The metadata is left out: the same sections under other metadata, the same digest.
        let (_, sections) = read_container(&split.base).unwrap();
        let sections: Vec<(&str, &[u8])> = sections
            .iter()
            .map(|s| (s.name.as_str(), s.data.as_slice()))
            .collect();
        assert_eq!(
            base_version(&write_container(b"{}", &sections)),
            base_version(&split.base)
        );
        assert_eq!(base_version(b"not a pack"), None);
    }
}
