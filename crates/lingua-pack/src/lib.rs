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
use lingua_core::knowledge::level::CefrLevel;
use lingua_core::knowledge::profile::NativeLanguage;
use lingua_core::packs::format::write_container;
use lingua_core::packs::grammar::{
    FormEdit, ParadigmEntry, SenseRun, Tag, encode_indexed, encode_paradigm, encode_runs,
    encode_tag_pool,
};
use lingua_core::packs::meta::PackMeta;
use lingua_core::packs::pack::section;
use serde::Deserialize;

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
        }
    }
}

impl std::error::Error for BuildError {}

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
            return Err(invalid(format!(
                "{} is on the wrong side: the studied side is read from {} alone, the native \
                 side from {} alone",
                path.display(),
                studied.display(),
                pair.display()
            )));
        }
    }
    let read = |name: &str| std::fs::read_to_string(pair.join(name));
    let read_studied = |name: &str| std::fs::read_to_string(studied.join(name));
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
    let mut extra: Vec<&str> = inputs
        .ranks
        .iter()
        .map(|(l, _)| l.as_str())
        .chain(inputs.glosses.iter().map(|(l, _)| l.as_str()))
        .collect();
    extra.sort_unstable();
    extra.dedup();
    let (forms, pool) =
        build_lexicon_blobs(&pairs, &extra).map_err(|e| BuildError::Fst(e.to_string()))?;
    let lex = FstLexicon::from_slices(forms.as_slice(), &pool)
        .map_err(|e| BuildError::Fst(e.to_string()))?;

    // Ranks by lemma id (0 = unranked).
    let mut freq = vec![0u32; lex.lemma_count()];
    for (lemma, rank) in &inputs.ranks {
        if let Some(id) = lex.id_of(lemma) {
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
        .filter_map(|(l, g)| lex.id_of(l).map(|id| (id as u32, g.as_str())))
        .collect();
    entries.sort_unstable_by_key(|(id, _)| *id);
    let gloss_zst = compress_glosses(&entries);

    // The pack's dictionary words, when they are not its glossed lemmas (D1, D3).
    let lexical_bytes = lexical::lexical_table(inputs, &lex, &entries)?.unwrap_or_default();

    // CEFR level code per lemma id (0 = no level). Emitted as an optional
    // section ONLY when the pair has CEFR data, so a level-less pack stays
    // byte-for-byte identical to before this feature.
    let levels_bytes: Vec<u8> = if inputs.levels.is_empty() {
        Vec::new()
    } else {
        let mut levels = vec![0u8; lex.lemma_count()];
        for (lemma, level) in &inputs.levels {
            if let Some(id) = lex.id_of(lemma) {
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
    let mut keyed: Vec<(String, &str, &str)> = inputs
        .expressions
        .iter()
        .filter_map(|(headword, gloss)| {
            expression_key(headword, studied, &lex)
                .map(|key| (key, headword.as_str(), gloss.as_str()))
        })
        .collect();
    // Sorted by key for the FST, and within a key by "is the key itself" first:
    // `break point` keeps the key `breaking point` also reaches, so the gloss
    // that survives is the dictionary spelling's (design D2). Alphabetical order
    // settles the rest, so the winner never depends on the input's order.
    keyed.sort_unstable_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| (a.1 != a.0).cmp(&(b.1 != b.0)))
            .then_with(|| a.1.cmp(b.1))
    });
    keyed.dedup_by(|a, b| a.0 == b.0);
    let (expr_fst, expr_zst) = if keyed.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        let mut builder = fst::MapBuilder::memory();
        for (id, (key, _, _)) in keyed.iter().enumerate() {
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
            .map(|(id, (_, _, gloss))| (id as u32, *gloss))
            .collect();
        (
            builder.into_inner().expect("in-memory FST"),
            compress_glosses(&entries),
        )
    };

    let grammar = grammar_sections(inputs, studied, &lex)?;

    let meta_json = serde_json::to_vec(&inputs.meta).expect("PackMeta serialises");
    let mut sections: Vec<(&str, &[u8])> = vec![
        (section::FORMS, forms.as_slice()),
        (section::LEMMAS, pool.as_bytes()),
        (section::FREQ, freq_bytes.as_slice()),
    ];
    if !levels_bytes.is_empty() {
        sections.push((section::LEVELS, levels_bytes.as_slice()));
    }
    if !lexical_bytes.is_empty() {
        sections.push((section::LEXICAL, lexical_bytes.as_slice()));
    }
    if !expr_fst.is_empty() {
        sections.push((section::EXPR, expr_fst.as_slice()));
        sections.push((section::EXPR_ZST, expr_zst.as_slice()));
    }
    if !grammar.tags.is_empty() {
        sections.push((section::TAGS, grammar.tags.as_slice()));
    }
    if !grammar.paradigms.is_empty() {
        sections.push((section::PARADIGMS_ZST, grammar.paradigms.as_slice()));
    }
    if !grammar.senses.is_empty() {
        sections.push((section::SENSES_ZST, grammar.senses.as_slice()));
    }
    sections.push((section::GLOSS_ZST, gloss_zst.as_slice()));
    sections.push((section::NOTICE, inputs.notice.as_bytes()));
    let pack = write_container(&meta_json, &sections);

    if pack.len() > MAX_PACK_BYTES {
        return Err(BuildError::OverBudget {
            size: pack.len(),
            budget: MAX_PACK_BYTES,
            reduce: if !expr_fst.is_empty() {
                REDUCE_EXPRESSIONS
            } else if !grammar.paradigms.is_empty() {
                REDUCE_READINGS
            } else {
                REDUCE_GLOSSES
            },
        });
    }
    Ok(pack)
}

/// The three grammar sections, empty when the inputs carry no grammar.
#[derive(Default)]
struct GrammarSections {
    tags: Vec<u8>,
    paradigms: Vec<u8>,
    senses: Vec<u8>,
}

/// Files the grammar tables under the lexicon the build has just assembled
/// (`add-lingua-word-grammar`, design D3).
///
/// Every tag is parsed strictly, so a code outside the vocabulary fails the
/// build by name. A reading is filed under its own dictionary form; a
/// believable reading of another dictionary form is also filed, as such,
/// under the one the core's own cascade reads the form as — which is what the
/// card will be keyed by. Readings whose dictionary form the lexicon does not
/// hold are dropped. Each run must cover exactly its word's gloss, and a noun's
/// runs carry the gender its readings give it ([`tags::noun_runs`]).
fn grammar_sections(
    inputs: &PackInputs,
    studied: StudiedLanguage,
    lex: &FstLexicon<&[u8]>,
) -> Result<GrammarSections, BuildError> {
    if inputs.readings.is_empty() && inputs.senses.is_empty() {
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
    let pool = tags::tag_pool(inputs.tag_pool.as_deref(), reading_tags, sense_tags)?;
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
    let lemma_id = |lemma: &str| -> Option<u32> {
        if !lex.contains_lemma(lemma) {
            return None;
        }
        lex.id_of(lemma).and_then(|id| u32::try_from(id).ok())
    };
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
        tags: encode_tag_pool(&pool),
        paradigms: compress(&paradigms),
        senses: compress(&runs),
    })
}

/// An expression's key: its words' dictionary forms, lowercase, joined by
/// single spaces (`starting point` → `start point`).
///
/// `None` when the lexicon does not hold one of those forms. `lemmatize` never
/// fails — its last resort is the lowercased word itself — so this is not a
/// failure to catch but the membership test the requirement asks for: a key no
/// reading can produce would sit in the pack unreachable for ever.
fn expression_key(headword: &str, studied: StudiedLanguage, lex: &impl Lexicon) -> Option<String> {
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

/// Encodes a gloss index + payload and zstd-compresses it. Layout matches the
/// reader: `count u32 | count*(id u32, off u32, len u32) | utf8`. Shared by the
/// per-lemma glosses and the expression table, which is the same shape keyed by
/// expression id.
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
    use lingua_core::analysis::ANALYZER_VERSION;
    use lingua_core::analysis::lexicon::Lexicon;
    use lingua_core::knowledge::state::FrequencyRanks;
    use lingua_core::packs::Pack;

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
        // A studied table left in the pair's folder is refused, by path.
        std::fs::write(en_fr.join("forms.tsv"), "ran\trun\n").unwrap();
        let err = inputs_from_tables(&root, "en-fr")
            .err()
            .expect("refused")
            .to_string();
        assert!(
            err.contains("en-fr/forms.tsv") && err.contains("wrong side"),
            "{err}"
        );
        std::fs::remove_file(en_fr.join("forms.tsv")).unwrap();
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
}
