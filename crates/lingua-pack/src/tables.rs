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
//! `paris` and `durand` out: refine-lingua-fr-en-glosses D2).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::{
    LEXICAL_TABLE, Manifest, STUDIED_SIDE, TAG_POOL_TABLE, read_lines, read_senses, tsv_pairs,
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

/// Checks one studied folder against the pairs that read it.
fn check_studied(dir: &Path, pairs: &[Pair]) -> Result<(), String> {
    let lang = folder_name(dir);
    let named = reference_of(dir)?;
    let Some(reference) = pairs.iter().find(|p| p.name == named) else {
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
    let glossed = glossed(reference)?;
    if words == glossed {
        return Ok(());
    }
    // The other set the reference's reduction may write: its glossed lemmas less those it glosses
    // by a proper noun's senses alone — all of them, never some (*Names left out by halves*).
    let names = names_only(reference)?;
    let words_not_names: BTreeSet<String> = glossed.difference(&names).cloned().collect();
    if !names.is_empty() && words == words_not_names {
        return Ok(());
    }
    Err(format!(
        "{lang}/{LEXICAL_TABLE} is not the lemmas its reference {} glosses: {} only in \
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
/// glossed lemmas, or all of them but those it glosses by a proper noun's senses alone.
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
        assert_eq!(check_committed_tables(&root.0), Ok(vec!["fr-en".to_owned()]));
        let root = french("names-out", "lot\nmaison\n");
        assert_eq!(check_committed_tables(&root.0), Ok(vec!["fr-en".to_owned()]));
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
}
