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

//! The committed tables, kept once per studied language (split-lingua-pack-tables-by-language,
//! language matrix programme M24; add-lingua-pack-lexical-layer D3, D4).
//!
//! Two packs of one studied language must analyse it alike whatever native language they
//! are glossed in. So the studied side of the tables ([`STUDIED_SIDE`]: forms, ranks, levels,
//! readings, the pinned tag pool and the dictionary words) lives once, in `tables/<studied>/`,
//! and every pair of that language reads it from there: two pairs cannot disagree on it,
//! because there is one copy. `tables/<pair>/` holds the native side ([`PAIR_SIDE`]) with its
//! pin and README, and no table of its studied language.
//!
//! The studied tables are written by one pair's reduction only — the language's reference
//! pair, which `tables/<studied>/studied.json` names; its `pin.json` is their provenance. The
//! dictionary words (`lexical.tsv`) are that pair's glossed lemmas — or, as its reduction writes
//! them, every one but those it glosses by a proper noun's senses alone (French's, which leave
//! `paris` and `durand` out: refine-lingua-fr-en-glosses D2) — and, in either set, the
//! language's acronym rows, forms in capitals that are their own lemmas, glossed or not
//! (French's `CA`, Italian's `NATO` and `USA`: keep-lingua-italian-acronyms-and-names D5).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use lingua_core::packs::format::{Section, read_container};
use lingua_core::packs::meta::BaseMeta;

use crate::{
    LEXICAL_TABLE, Manifest, STUDIED_SIDE, TAG_POOL_TABLE, read_lines, read_optional, read_senses,
    tsv_pairs,
};

/// The file of a studied folder naming its reference pair.
pub const STUDIED_RECORD: &str = "studied.json";

/// What `tables/<studied>/studied.json` says.
#[derive(serde::Deserialize)]
struct StudiedRecord {
    /// The pair whose reduction writes the studied tables, and whose glossed lemmas are the
    /// language's dictionary words.
    reference: String,
}

/// One pair's folder.
struct Pair {
    name: String,
    dir: PathBuf,
    studied: String,
}

/// A folder of the tables root is a pair's when it is named `<studied>-<native>`; any other
/// is a studied language's, which names its reference pair.
fn is_pair(name: &str) -> bool {
    name.contains('-')
}

fn folder_name(dir: &Path) -> String {
    dir.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn read_pair(dir: &Path) -> Result<Pair, String> {
    let name = folder_name(dir);
    let text = std::fs::read_to_string(dir.join("manifest.json"))
        .map_err(|e| format!("{name}/manifest.json: {e}"))?;
    let manifest: Manifest =
        serde_json::from_str(&text).map_err(|e| format!("{name}/manifest.json: {e}"))?;
    let (studied, native) = (manifest.meta.studied, manifest.meta.native);
    if name != format!("{studied}-{native}") {
        return Err(format!(
            "{name}/manifest.json names a pack studying {studied:?} glossed in {native:?}, but \
             its folder is {name}: a pair's folder is named <studied>-<native>, and its studied \
             tables are read from <studied>/"
        ));
    }
    Ok(Pair {
        name,
        dir: dir.to_path_buf(),
        studied,
    })
}

/// The reference pair a studied folder names.
pub(crate) fn reference_of(dir: &Path) -> Result<String, String> {
    let lang = folder_name(dir);
    let record: StudiedRecord = serde_json::from_str(
        &std::fs::read_to_string(dir.join(STUDIED_RECORD))
            .map_err(|e| format!("{lang}/{STUDIED_RECORD}: {e}"))?,
    )
    .map_err(|e| format!("{lang}/{STUDIED_RECORD}: {e}"))?;
    Ok(record.reference)
}

/// The lemmas a pair glosses: its `gloss.tsv`'s first column, as the builder reads it.
fn glossed(pair: &Pair) -> Result<BTreeSet<String>, String> {
    let text = std::fs::read_to_string(pair.dir.join("gloss.tsv"))
        .map_err(|e| format!("{}/gloss.tsv: {e}", pair.name))?;
    Ok(tsv_pairs(&text)
        .into_iter()
        .map(|(lemma, _)| lemma)
        .collect())
}

/// The lemmas a pair glosses by a proper noun's senses alone: every run of their `senses.tsv`
/// row is `PROPN` (refine-lingua-fr-en-glosses D2). A pair without `senses.tsv` has none.
fn names_only(pair: &Pair) -> Result<BTreeSet<String>, String> {
    let runs = read_senses(&pair.dir).map_err(|e| format!("{}/senses.tsv: {e}", pair.name))?;
    Ok(runs
        .into_iter()
        .filter(|(_, runs)| runs.iter().all(|(tag, _)| tag == "PROPN"))
        .map(|(lemma, _)| lemma)
        .collect())
}

/// A studied folder's acronym rows (keep-lingua-french-acronyms-in-capitals D2,
/// keep-lingua-italian-acronyms-and-names D2, D5): the forms of its `forms.tsv` written as
/// the builder accepts an acronym — wholly in capitals, two letters or more
/// ([`crate::is_acronym`]) —, each its own lemma, as its reference's reduction writes them
/// (`CA`, `NATO`, `USA`). A form merely holding a capital (`Roma`) is none. A folder without
/// forms has none.
fn acronym_rows(dir: &Path) -> Result<BTreeSet<String>, String> {
    let text = read_optional(dir, "forms.tsv")
        .map_err(|e| format!("{}/forms.tsv: {e}", folder_name(dir)))?
        .unwrap_or_default();
    Ok(tsv_pairs(&text)
        .into_iter()
        .filter(|(form, lemma)| form == lemma && crate::is_acronym(form))
        .map(|(form, _)| form)
        .collect())
}

/// A few of `words`, for a message.
fn sample<'a>(words: impl Iterator<Item = &'a String>) -> String {
    words
        .take(5)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

/// The files of a folder, by name, but hidden ones (`.DS_Store`).
fn files_of(dir: &Path) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.'))
        .collect();
    names.sort();
    Ok(names)
}

/// Checks a pair's folder against its studied language's: the folder exists, and the pair
/// holds no table of it.
fn check_pair(root: &Path, pair: &Pair) -> Result<(), String> {
    let lang = &pair.studied;
    let studied = root.join(lang);
    if !studied.join(STUDIED_RECORD).is_file() {
        return Err(format!(
            "{} studies {lang:?}, but {lang}/ holds no studied tables ({lang}/{STUDIED_RECORD}): \
             add {lang}/, which the reduction of {lang:?}'s reference pair writes \
             (scripts/lingua-data/build.sh --reduce <pair>)",
            pair.name
        ));
    }
    let reference = reference_of(&studied)?;
    if let Some(table) = STUDIED_SIDE
        .iter()
        .chain(&[STUDIED_RECORD])
        .find(|t| pair.dir.join(t).exists())
    {
        return Err(format!(
            "{}/{table} is a table of the studied language {lang:?}: it is kept once, in \
             {lang}/, which {reference}'s reduction writes, and read from there by every pair \
             studying {lang:?}. Remove it from {}/.",
            pair.name, pair.name
        ));
    }
    Ok(())
}

/// Whether a studied folder awaits its reference's first reduction (add-lingua-italian-forms-tables
/// D9): it holds the inputs a person commits — its record and its pinned tag pool — and nothing
/// else, and no folder of the reference it names exists yet, which that reduction writes whole. No
/// pair reads such a folder: a pair beside it holds tables of its own, which the folder lacks.
fn awaits_its_first_reduction(dir: &Path, reference: &str) -> Result<bool, String> {
    let root = dir.parent().unwrap_or(dir);
    Ok(files_of(dir)? == [STUDIED_RECORD, TAG_POOL_TABLE] && !root.join(reference).exists())
}

/// Checks one studied folder against the pairs that read it.
fn check_studied(dir: &Path, pairs: &[Pair]) -> Result<(), String> {
    let lang = folder_name(dir);
    let named = reference_of(dir)?;
    let Some(reference) = pairs.iter().find(|p| p.name == named) else {
        if awaits_its_first_reduction(dir, &named)? {
            return Ok(());
        }
        return Err(format!(
            "{lang}/{STUDIED_RECORD} names {named:?} as the pair that writes it, which has no \
             tables"
        ));
    };
    if reference.studied != lang {
        return Err(format!(
            "{lang}/{STUDIED_RECORD} names {}, which studies {:?}",
            reference.name, reference.studied
        ));
    }
    // Exactly the studied tables and the record: nothing of a pair's side, no pin, no README.
    if let Some(stray) = files_of(dir)?
        .into_iter()
        .find(|name| name != STUDIED_RECORD && !STUDIED_SIDE.contains(&name.as_str()))
    {
        return Err(format!(
            "{lang}/{stray} does not belong in a studied language's folder, which holds {} and \
             {STUDIED_RECORD} alone (written by {}'s reduction); a pair's own tables, pin and \
             README are kept in its folder",
            STUDIED_SIDE.join(", "),
            reference.name
        ));
    }
    let optional = |file: &str| read_lines(dir, file).map_err(|e| format!("{lang}/{file}: {e}"));
    if optional(TAG_POOL_TABLE)?.is_none() {
        return Err(format!(
            "{lang} has no {TAG_POOL_TABLE}: every studied language pins its tag pool, which \
             {}'s reduction keeps (scripts/lingua-data/SOURCES.md)",
            reference.name
        ));
    }
    let Some(words) = optional(LEXICAL_TABLE)? else {
        return Err(format!(
            "{lang} has no {LEXICAL_TABLE}: its dictionary words are {}'s glossed lemmas, which \
             build.sh writes there when it reduces {}",
            reference.name, reference.name
        ));
    };
    if words.windows(2).any(|w| w[0] >= w[1]) {
        return Err(format!(
            "{lang}/{LEXICAL_TABLE} is not byte-sorted with each lemma once: reduce {} again \
             (build.sh writes it)",
            reference.name
        ));
    }
    let words: BTreeSet<String> = words.into_iter().collect();
    // The acronym rows are dictionary words whatever their sense runs and whether glossed
    // (keep-lingua-italian-acronyms-and-names D5): `NATO`, unglossed, and `USA`, glossed by a proper
    // noun's run, beside French's `CA`, glossed by a noun's.
    let rows = acronym_rows(dir)?;
    let glossed: BTreeSet<String> = glossed(reference)?.union(&rows).cloned().collect();
    if words == glossed {
        return Ok(());
    }
    // The other set the reference's reduction may write: its glossed lemmas less those it glosses
    // by a proper noun's senses alone — all of them, never some (*Names left out by halves*) —,
    // the acronym rows kept.
    let names: BTreeSet<String> = names_only(reference)?.difference(&rows).cloned().collect();
    let words_not_names: BTreeSet<String> = glossed.difference(&names).cloned().collect();
    if !names.is_empty() && words == words_not_names {
        return Ok(());
    }
    let and_rows = if rows.is_empty() {
        String::new()
    } else {
        format!(" (its {} acronym rows among them)", rows.len())
    };
    Err(format!(
        "{lang}/{LEXICAL_TABLE} is not the lemmas its reference {} glosses{and_rows}: {} only in \
         {LEXICAL_TABLE} ({}), {} only in {}/gloss.tsv ({}); nor those less the {} it glosses by \
         a proper noun's senses alone: {} only in {LEXICAL_TABLE} ({}), {} left out ({}). Reduce \
         {} again (build.sh writes it).",
        reference.name,
        words.difference(&glossed).count(),
        sample(words.difference(&glossed)),
        glossed.difference(&words).count(),
        reference.name,
        sample(glossed.difference(&words)),
        names.len(),
        words.difference(&words_not_names).count(),
        sample(words.difference(&words_not_names)),
        words_not_names.difference(&words).count(),
        sample(words_not_names.difference(&words)),
        reference.name,
    ))
}

/// Checks the tables root. Every folder is a pair's (named `<studied>-<native>` after its
/// manifest) or a studied language's (holding `studied.json`); anything else fails. Each pair
/// finds its studied language's folder beside it and holds no table of it. Each studied folder
/// names an existing reference pair of its language, holds its studied tables and
/// `studied.json` alone, pins its tag pool, and holds as its dictionary words that pair's
/// glossed lemmas, or all of them but those it glosses by a proper noun's senses alone — its
/// acronym rows in either set (keep-lingua-italian-acronyms-and-names D5).
/// Answers the pairs read, by name; an error names the folder and the file at fault, and the
/// reference pair whose reduction writes a studied table.
pub fn check_committed_tables(root: &Path) -> Result<Vec<String>, String> {
    let mut dirs: Vec<_> = std::fs::read_dir(root)
        .map_err(|e| format!("{}: {e}", root.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|dir| dir.is_dir() && !folder_name(dir).starts_with('.'))
        .collect();
    dirs.sort();
    let (pair_dirs, studied_dirs): (Vec<_>, Vec<_>) =
        dirs.iter().partition(|dir| is_pair(&folder_name(dir)));
    if let Some(dir) = studied_dirs
        .iter()
        .find(|dir| !dir.join(STUDIED_RECORD).is_file())
    {
        let name = folder_name(dir);
        return Err(format!(
            "{name} is neither a pair (a folder named <studied>-<native>) nor a studied \
             language ({name}/{STUDIED_RECORD} names the pair that writes it)"
        ));
    }
    let pairs = pair_dirs
        .iter()
        .map(|dir| read_pair(dir))
        .collect::<Result<Vec<_>, _>>()?;
    for pair in &pairs {
        check_pair(root, pair)?;
    }
    for dir in &studied_dirs {
        check_studied(dir, &pairs)?;
    }
    Ok(pairs.into_iter().map(|p| p.name).collect())
}

/// The first section, by name, that two containers' sections do not hold alike: `None` when every
/// named section has the same bytes in both, whatever their order. Says how it differs — absent
/// from one, or its size in each (`differs (N vs M bytes)`, the same size twice when the bytes
/// alone differ) — so a parity check names what moved.
pub fn first_section_difference(ours: &[Section], theirs: &[Section]) -> Option<String> {
    let names: BTreeSet<&str> = ours.iter().chain(theirs).map(|s| s.name.as_str()).collect();
    fn find<'a>(sections: &'a [Section], name: &str) -> Option<&'a [u8]> {
        sections
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.data.as_slice())
    }
    names
        .into_iter()
        .find_map(|name| match (find(ours, name), find(theirs, name)) {
            (Some(a), Some(b)) if a == b => None,
            (Some(a), Some(b)) => Some(format!(
                "the {name:?} section differs ({} vs {} bytes)",
                a.len(),
                b.len()
            )),
            (Some(a), None) => Some(format!(
                "the {name:?} section is present ({} bytes) vs absent",
                a.len()
            )),
            (None, _) => Some(format!("the {name:?} section is absent vs present")),
        })
}

/// One base per studied language (split-lingua-pack-format D6): the base `pair` builds is
/// byte for byte the one its language's `reference` builds, so the language's base can ship once.
/// An error names both pairs and what differs — a field of their metadata, which a pair's manifest
/// sets (`analyzer_version`, `levels_estimated`), or the first section by name — and either is
/// refused when it is no base.
pub fn check_one_base(reference: (&str, &[u8]), pair: (&str, &[u8])) -> Result<(), String> {
    let read = |(name, bytes): (&str, &[u8])| {
        let meta = BaseMeta::read(bytes).map_err(|e| format!("{name}'s base: {e}"))?;
        let (_, sections) = read_container(bytes).map_err(|e| format!("{name}'s base: {e}"))?;
        Ok::<_, String>((meta, sections))
    };
    let ((theirs, their_sections), (ours, our_sections)) = (read(reference)?, read(pair)?);
    let differs = |what: &str, here: &dyn std::fmt::Debug, there: &dyn std::fmt::Debug| {
        format!(
            "{}'s base is not {}'s, its studied language's reference: {what} is {here:?} in {} \
             and {there:?} in {}",
            pair.0, reference.0, pair.0, reference.0
        )
    };
    if ours.studied != theirs.studied {
        return Err(differs("studied", &ours.studied, &theirs.studied));
    }
    if ours.analyzer_version != theirs.analyzer_version {
        return Err(differs(
            "analyzer_version",
            &ours.analyzer_version,
            &theirs.analyzer_version,
        ));
    }
    if ours.levels_estimated != theirs.levels_estimated {
        return Err(differs(
            "levels_estimated",
            &ours.levels_estimated,
            &theirs.levels_estimated,
        ));
    }
    if let Some(difference) = first_section_difference(&our_sections, &their_sections) {
        return Err(format!(
            "{}'s base is not {}'s, its studied language's reference: {difference}, in {} against {}",
            pair.0, reference.0, pair.0, reference.0
        ));
    }
    if ours.base_version != theirs.base_version {
        return Err(differs(
            "base_version",
            &ours.base_version,
            &theirs.base_version,
        ));
    }
    if pair.1 != reference.1 {
        return Err(format!(
            "{}'s base is not {}'s, its studied language's reference: their containers differ \
             though their fields and sections are the same",
            pair.0, reference.0
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A scratch tables root, removed when dropped.
    struct Root(PathBuf);

    impl Root {
        fn new(tag: &str) -> Root {
            let dir =
                std::env::temp_dir().join(format!("lingua-committed-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Root(dir)
        }

        /// A pair glossing `glossed`.
        fn pair(&self, name: &str, glossed: &[&str]) {
            let dir = self.0.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            let (studied, native) = name.split_once('-').unwrap();
            let manifest = serde_json::json!({
                "meta": {"studied": studied, "native": native, "pack_version": "t",
                         "analyzer_version": "t", "licences": []},
                "sources": []
            });
            std::fs::write(dir.join("manifest.json"), manifest.to_string()).unwrap();
            let gloss: String = glossed.iter().map(|l| format!("{l}\tG\n")).collect();
            std::fs::write(dir.join("gloss.tsv"), gloss).unwrap();
        }

        /// A studied folder written by `reference`, with the optional tables given.
        fn studied(&self, lang: &str, reference: &str, tags: Option<&str>, lexical: Option<&str>) {
            let dir = self.0.join(lang);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join(STUDIED_RECORD),
                serde_json::json!({ "reference": reference }).to_string(),
            )
            .unwrap();
            if let Some(tags) = tags {
                std::fs::write(dir.join(TAG_POOL_TABLE), tags).unwrap();
            }
            if let Some(lexical) = lexical {
                std::fs::write(dir.join(LEXICAL_TABLE), lexical).unwrap();
            }
        }

        fn write(&self, path: &str, text: &str) {
            std::fs::write(self.0.join(path), text).unwrap();
        }

        fn remove(&self, path: &str) {
            std::fs::remove_file(self.0.join(path)).unwrap();
        }

        fn error(&self) -> String {
            check_committed_tables(&self.0).expect_err("the check fails")
        }
    }

    impl Drop for Root {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const POOL: &str = "NOUN\nVERB\n";

    /// Spanish read by es-fr, its reference, and es-en; English by en-fr.
    fn matrix(tag: &str) -> Root {
        let root = Root::new(tag);
        root.pair("es-fr", &["casa", "dios"]);
        // Glossed in English, `augusto` too; its dictionary words are es-fr's.
        root.pair("es-en", &["augusto", "casa"]);
        root.studied("es", "es-fr", Some(POOL), Some("casa\ndios\n"));
        for table in ["forms.tsv", "freq.tsv", "grammar.tsv", "level.tsv"] {
            root.write(&format!("es/{table}"), "casa\tcasa\n");
        }
        root.pair("en-fr", &["run"]);
        root.studied("en", "en-fr", Some("VERB\n"), Some("run\n"));
        root
    }

    #[test]
    fn pairs_reading_one_studied_folder_pass() {
        let root = matrix("agree");
        // A hidden file or folder is no table.
        root.write("es/.DS_Store", "");
        std::fs::create_dir_all(root.0.join(".cache")).unwrap();
        assert_eq!(
            check_committed_tables(&root.0),
            Ok(vec![
                "en-fr".to_owned(),
                "es-en".to_owned(),
                "es-fr".to_owned()
            ])
        );
    }

    #[test]
    fn spec_scenario_a_studied_table_left_in_a_pair_s_folder() {
        // Named: the pair, the file and the studied language's reference pair.
        for table in STUDIED_SIDE.iter().chain(&[STUDIED_RECORD]) {
            let root = matrix(&format!("copy-{table}"));
            root.write(&format!("es-en/{table}"), "casa\n");
            let err = root.error();
            assert!(
                err.contains(&format!("es-en/{table}")) && err.contains("es-fr's reduction"),
                "{err}"
            );
        }
    }

    #[test]
    fn a_studied_folder_awaiting_its_reference_s_first_reduction() {
        // Italian before it-fr's first update (add-lingua-italian-forms-tables D9): its record and
        // its pinned pool, committed by hand; no it-fr folder yet. No pair reads it.
        let root = matrix("first-reduction");
        root.studied("it", "it-fr", Some("NOUN\n"), None);
        assert_eq!(
            check_committed_tables(&root.0),
            Ok(vec![
                "en-fr".to_owned(),
                "es-en".to_owned(),
                "es-fr".to_owned()
            ])
        );
        // A table beside them is a reduction's, which needs its reference's tables.
        root.write("it/forms.tsv", "casa\tcasa\n");
        assert!(root.error().contains("names \"it-fr\""), "{}", root.error());
        root.remove("it/forms.tsv");
        // Without its pinned pool, it is no folder awaiting anything.
        root.remove("it/tags.tsv");
        assert!(
            root.error().contains("which has no tables"),
            "{}",
            root.error()
        );
        root.write("it/tags.tsv", "NOUN\n");
        // A folder of the reference's that holds no tables is the reference's own failure.
        std::fs::create_dir_all(root.0.join("it-fr")).unwrap();
        assert!(check_committed_tables(&root.0).is_err());
    }

    #[test]
    fn a_studied_folder_holds_its_six_tables_and_its_record_alone() {
        for stray in [
            "gloss.tsv",
            "mwe.tsv",
            "senses.tsv",
            "NOTICE",
            "manifest.json",
            "pin.json",
            "README.md",
        ] {
            let root = matrix(&format!("stray-{stray}"));
            root.write(&format!("es/{stray}"), "casa\tG\n");
            let err = root.error();
            assert!(
                err.contains(&format!("es/{stray} does not belong")) && err.contains("es-fr"),
                "{err}"
            );
        }
    }

    #[test]
    fn spec_scenario_dictionary_words_that_are_not_the_reference_s() {
        let root = matrix("lexical");
        // A lemma es-fr does not gloss.
        root.write("es/lexical.tsv", "augusto\ncasa\ndios\n");
        let err = root.error();
        assert!(
            err.contains("reference es-fr") && err.contains("1 only in lexical.tsv (augusto)"),
            "{err}"
        );
        // One es-fr glosses, left out.
        root.write("es/lexical.tsv", "casa\n");
        let err = root.error();
        assert!(
            err.contains("reference es-fr") && err.contains("1 only in es-fr/gloss.tsv (dios)"),
            "{err}"
        );
        // Missing.
        root.remove("es/lexical.tsv");
        let err = root.error();
        assert!(
            err.contains("es has no lexical.tsv") && err.contains("es-fr"),
            "{err}"
        );
    }

    /// French read by fr-en, which glosses `paris` and `lyon` by a proper noun's senses alone,
    /// `lot` by a common noun's beside a name's, and `maison`.
    fn french(tag: &str, lexical: &str) -> Root {
        let root = Root::new(tag);
        root.pair("fr-en", &["lot", "lyon", "maison", "paris"]);
        root.write(
            "fr-en/senses.tsv",
            "lot\tNOUN:2\tPROPN:1\nlyon\tPROPN:1\nmaison\tNOUN:1\nparis\tPROPN:2\n",
        );
        root.studied("fr", "fr-en", Some(POOL), Some(lexical));
        root
    }

    #[test]
    fn the_reference_s_glossed_lemmas_or_all_but_its_names_pass() {
        // refine-lingua-fr-en-glosses D2: either set, as the reference's reduction writes it.
        let root = french("names-all", "lot\nlyon\nmaison\nparis\n");
        assert_eq!(
            check_committed_tables(&root.0),
            Ok(vec!["fr-en".to_owned()])
        );
        let root = french("names-out", "lot\nmaison\n");
        assert_eq!(
            check_committed_tables(&root.0),
            Ok(vec!["fr-en".to_owned()])
        );
    }

    #[test]
    fn spec_scenario_names_left_out_by_halves() {
        // `paris` left out, `lyon` listed: neither set — named, with fr-en.
        let root = french("names-halves", "lot\nlyon\nmaison\n");
        let err = root.error();
        assert!(
            err.contains("reference fr-en")
                && err.contains("less the 2 it glosses by a proper noun's senses alone")
                && err.contains("1 only in lexical.tsv (lyon)"),
            "{err}"
        );
        // A word with a common sense beside a name's is no name: leaving it out fails too.
        let root = french("names-word", "maison\n");
        let err = root.error();
        assert!(err.contains("1 left out (lot)"), "{err}");
        // A reference with no name glosses one set alone (Spanish, `spec_scenario_dictionary_words_
        // that_are_not_the_reference_s`); an unreadable senses.tsv fails, named.
        let root = french("names-broken", "lot\nmaison\n");
        root.write("fr-en/senses.tsv", "lot\n");
        assert!(root.error().contains("fr-en/senses.tsv"));
    }

    /// Italian read by it-fr, which glosses `roma` and `firenze` by a proper noun's senses alone,
    /// `marco` by a name's beside a noun's and `USA` by a proper noun's; `tables/it/forms.tsv` holds
    /// the acronym rows `NATO` and `USA` (keep-lingua-italian-acronyms-and-names D5), `nato` read as
    /// *nascere* beside them.
    fn italian(tag: &str, glossed: &[&str], lexical: &str) -> Root {
        let root = Root::new(tag);
        root.pair("it-fr", glossed);
        root.write(
            "it-fr/senses.tsv",
            "firenze\tPROPN:1\nmarco\tPROPN:1\tNOUN:1\nroma\tPROPN:1\nUSA\tPROPN:2\n",
        );
        root.studied("it", "it-fr", Some(POOL), Some(lexical));
        root.write(
            "it/forms.tsv",
            "NATO\tNATO\nUSA\tUSA\nfirenze\tfirenze\nmarco\tmarco\nnato\tnascere\nroma\troma\n",
        );
        root
    }

    fn italian_passes(root: Root) {
        assert_eq!(
            check_committed_tables(&root.0),
            Ok(vec!["it-fr".to_owned()])
        );
    }

    #[test]
    fn spec_scenario_the_rows_are_dictionary_words() {
        // Before change 16 glosses anything, the rows alone (D6): `NATO` glossed by no sense run.
        italian_passes(italian("it-rows", &[], "NATO\nUSA\n"));
        // `USA` glossed by a proper noun's run alone is a dictionary word all the same.
        italian_passes(italian("it-usa", &["USA"], "NATO\nUSA\n"));
    }

    #[test]
    fn spec_scenario_every_glossed_lemma_or_every_one_but_the_names() {
        // D5: either set the checks accept for every studied language, the rows in both.
        let all = ["USA", "firenze", "marco", "roma"];
        italian_passes(italian("it-all", &all, "NATO\nUSA\nfirenze\nmarco\nroma\n"));
        italian_passes(italian("it-names-out", &all, "NATO\nUSA\nmarco\n"));
    }

    #[test]
    fn spec_scenario_the_checks_hold_italian_s_words() {
        let all = ["USA", "firenze", "marco", "roma"];
        // The rows left out: named.
        let err = italian("it-no-rows", &[], "").error();
        assert!(
            err.contains("reference it-fr glosses (its 2 acronym rows among them)")
                && err.contains("2 only in it-fr/gloss.tsv (NATO, USA)"),
            "{err}"
        );
        let err = italian("it-no-usa", &all, "NATO\nmarco\n").error();
        assert!(err.contains("1 left out (USA)"), "{err}");
        // A name that is no row listed while another is left out: named.
        let err = italian("it-halves", &all, "NATO\nUSA\nmarco\nroma\n").error();
        assert!(
            err.contains("less the 2 it glosses by a proper noun's senses alone")
                && err.contains("1 only in lexical.tsv (roma)"),
            "{err}"
        );
        // A word with a common sense beside a name's left out: named.
        let err = italian("it-word-out", &all, "NATO\nUSA\n").error();
        assert!(err.contains("1 left out (marco)"), "{err}");
        // A capital that is no form's own lemma is no row.
        let root = italian("it-stray", &[], "NATO\nUSA\n");
        root.write("it/forms.tsv", "NATO\tNATO\nUSA\tuso\n");
        let err = root.error();
        assert!(err.contains("1 only in lexical.tsv (USA)"), "{err}");
        // Nor is a form its own lemma that merely holds a capital, or a single letter in capitals:
        // a row is written as the builder accepts an acronym.
        let root = italian("it-not-acronyms", &[], "I\nNATO\nRoma\nUSA\n");
        root.write("it/forms.tsv", "I\tI\nNATO\tNATO\nRoma\tRoma\nUSA\tUSA\n");
        let err = root.error();
        assert!(err.contains("2 only in lexical.tsv (I, Roma)"), "{err}");
    }

    #[test]
    fn a_studied_language_without_its_pinned_pool_fails() {
        let root = Root::new("unpinned");
        root.pair("en-fr", &["run"]);
        root.studied("en", "en-fr", None, Some("run\n"));
        let err = root.error();
        assert!(
            err.contains("en has no") && err.contains(TAG_POOL_TABLE),
            "{err}"
        );
    }

    #[test]
    fn spec_scenario_a_listed_pair_whose_studied_language_has_no_tables() {
        let root = Root::new("orphan");
        root.pair("es-en", &["casa"]);
        let err = root.error();
        assert!(
            err.contains("es-en studies \"es\"") && err.contains("add es/"),
            "{err}"
        );
    }

    #[test]
    fn spec_scenario_a_pair_whose_manifest_studies_another_language() {
        let root = matrix("misnamed");
        root.write(
            "es-en/manifest.json",
            &serde_json::json!({"meta": {"studied": "en", "native": "en", "pack_version": "t",
                "analyzer_version": "t", "licences": []}, "sources": []})
            .to_string(),
        );
        let err = root.error();
        assert!(
            err.contains("es-en/manifest.json names a pack studying \"en\"")
                && err.contains("its folder is es-en"),
            "{err}"
        );
    }

    #[test]
    fn a_reference_that_is_missing_or_studies_another_language_fails() {
        let root = Root::new("reference");
        root.pair("en-fr", &["run"]);
        root.studied("en", "en-es", Some(POOL), Some("run\n"));
        assert!(root.error().contains("en-es"));
        root.pair("es-fr", &["casa"]);
        root.studied("en", "es-fr", Some(POOL), Some("run\n"));
        root.studied("es", "es-fr", Some(POOL), Some("casa\n"));
        let err = root.error();
        assert!(err.contains("en/studied.json names es-fr"), "{err}");
    }

    #[test]
    fn a_folder_that_is_neither_a_pair_nor_a_studied_language_fails() {
        let root = matrix("neither");
        std::fs::create_dir_all(root.0.join("work")).unwrap();
        let err = root.error();
        assert!(
            err.contains("work is neither a pair") && err.contains("work/studied.json"),
            "{err}"
        );
        // A pair's folder without its manifest.
        let root = matrix("no-manifest");
        root.remove("es-en/manifest.json");
        assert!(root.error().contains("es-en/manifest.json"));
    }

    #[test]
    fn a_lexical_table_out_of_order_fails() {
        let root = matrix("unsorted");
        root.write("es/lexical.tsv", "dios\ncasa\n");
        let err = root.error();
        assert!(err.contains("es/lexical.tsv"), "{err}");
        root.write("es/lexical.tsv", "casa\ncasa\ndios\n");
        assert!(root.error().contains("byte-sorted"));
    }

    #[test]
    fn an_unreadable_root_or_record_fails() {
        let root = Root::new("broken");
        assert!(check_committed_tables(&root.0.join("absent")).is_err());
        std::fs::create_dir_all(root.0.join("en-fr")).unwrap();
        std::fs::write(root.0.join("en-fr/manifest.json"), "{").unwrap();
        let err = root.error();
        assert!(err.contains("en-fr/manifest.json"), "{err}");
        let root = matrix("broken-record");
        root.write("es/studied.json", "{");
        let err = root.error();
        assert!(err.contains("es/studied.json"), "{err}");
        // A reference pair whose glosses cannot be read.
        let root = matrix("broken-gloss");
        root.remove("es-fr/gloss.tsv");
        assert!(root.error().contains("es-fr/gloss.tsv"));
    }

    // — one base per studied language (split-lingua-pack-format D6) —

    use lingua_core::packs::format::write_container;
    use lingua_core::packs::meta::{LayerMeta, Part};

    /// A base's container: `meta` over the sections given.
    fn base(meta: &BaseMeta, sections: &[(&str, &[u8])]) -> Vec<u8> {
        write_container(&serde_json::to_vec(meta).unwrap(), sections)
    }

    fn spanish(levels_estimated: bool, analyzer_version: &str) -> BaseMeta {
        BaseMeta {
            part: Part::Base,
            studied: "es".into(),
            analyzer_version: analyzer_version.into(),
            base_version: "v".into(),
            levels_estimated,
        }
    }

    const SECTIONS: [(&str, &[u8]); 2] = [("forms", b"abc"), ("lemmas", b"casa")];

    #[test]
    fn spec_scenario_a_pair_whose_manifest_disagrees_with_its_language_s_reference() {
        let reference = base(&spanish(true, "1.5.0"), &SECTIONS);
        assert_eq!(
            check_one_base(("es-fr", &reference), ("es-en", &reference)),
            Ok(())
        );
        for (other, field) in [
            (spanish(false, "1.5.0"), "levels_estimated"),
            (spanish(true, "1.4.0"), "analyzer_version"),
        ] {
            let err = check_one_base(("es-fr", &reference), ("es-en", &base(&other, &SECTIONS)))
                .unwrap_err();
            assert!(err.contains("es-en's base is not es-fr's"), "{err}");
            assert!(err.contains(field), "{err}");
        }
        let french = BaseMeta {
            studied: "fr".into(),
            ..spanish(true, "1.5.0")
        };
        let err = check_one_base(("es-fr", &reference), ("fr-en", &base(&french, &SECTIONS)))
            .unwrap_err();
        assert!(err.contains("studied"), "{err}");
    }

    #[test]
    fn a_base_whose_sections_differ_names_the_first_that_does() {
        let meta = spanish(true, "1.5.0");
        let reference = base(&meta, &SECTIONS);
        let other = base(&meta, &[("forms", b"abc"), ("lemmas", b"casas")]);
        let err = check_one_base(("es-fr", &reference), ("es-en", &other)).unwrap_err();
        assert!(
            err.contains("the \"lemmas\" section differs (5 vs 4 bytes)"),
            "{err}"
        );
        let missing = base(&meta, &SECTIONS[..1]);
        let err = check_one_base(("es-fr", &reference), ("es-en", &missing)).unwrap_err();
        assert!(
            err.contains("the \"lemmas\" section is absent vs present"),
            "{err}"
        );
        let err = check_one_base(("es-fr", &missing), ("es-en", &reference)).unwrap_err();
        assert!(
            err.contains("the \"lemmas\" section is present (4 bytes) vs absent"),
            "{err}"
        );
        // The same sections in another order: another digest.
        let reordered = base(
            &BaseMeta {
                base_version: "w".into(),
                ..meta.clone()
            },
            &[SECTIONS[1], SECTIONS[0]],
        );
        let err = check_one_base(("es-fr", &reference), ("es-en", &reordered)).unwrap_err();
        assert!(err.contains("base_version"), "{err}");
        // Every field and section the same, the containers not.
        let reordered = base(&meta, &[SECTIONS[1], SECTIONS[0]]);
        let err = check_one_base(("es-fr", &reference), ("es-en", &reordered)).unwrap_err();
        assert!(err.contains("their containers differ"), "{err}");
    }

    #[test]
    fn a_part_that_is_no_base_is_refused_by_name() {
        let reference = base(&spanish(true, "1.5.0"), &SECTIONS);
        let layer = write_container(
            &serde_json::to_vec(&LayerMeta {
                part: Part::Layer,
                studied: "es".into(),
                native: "en".into(),
                pack_version: "p".into(),
                base_version: "v".into(),
                licences: vec![],
            })
            .unwrap(),
            &[],
        );
        let err = check_one_base(("es-fr", &reference), ("es-en", &layer)).unwrap_err();
        assert!(err.starts_with("es-en's base: "), "{err}");
        let err = check_one_base(("es-fr", b"not a pack"), ("es-en", &reference)).unwrap_err();
        assert!(err.starts_with("es-fr's base: "), "{err}");
    }
}
