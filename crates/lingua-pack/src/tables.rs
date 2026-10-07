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

//! The committed tables of every pair, held to their studied language
//! (add-lingua-pack-lexical-layer D3, D4).
//!
//! Two packs of one studied language must analyse it alike whatever native language they
//! are glossed in. So every pair built from committed tables carries its studied language's
//! pinned tag pool (`tags.tsv`), and two pairs of one studied language agree on it, on their
//! studied tables ([`STUDIED_TABLES`]) and on their dictionary words: a pair's `lexical.tsv`
//! when it has one, else the lemmas its `gloss.tsv` glosses — which is what its pack reads as
//! its dictionary words. Together they decide the lemma ids and how a form's readings are
//! stored.

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::{LEXICAL_TABLE, Manifest, TAG_POOL_TABLE, read_lines, tsv_pairs};

/// The tables of a pair's studied side: its forms, their ranks, their levels and their
/// readings. They belong to the studied language, so two pairs of one studied language hold
/// them byte for byte alike — copies, until `split-lingua-pack-tables-by-language` keeps them
/// once per studied language (language matrix programme, M24). `level.tsv` is optional: two
/// such pairs both have it or both lack it.
pub const STUDIED_TABLES: [&str; 4] = ["forms.tsv", "freq.tsv", "level.tsv", "grammar.tsv"];

/// What one pair's tables say of its studied language.
struct Pair {
    name: String,
    dir: PathBuf,
    studied: String,
    pin: Vec<String>,
    dictionary_words: BTreeSet<String>,
}

fn read_pair(dir: &Path) -> Result<Pair, String> {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let read = |file: &str| {
        std::fs::read_to_string(dir.join(file)).map_err(|e| format!("{name}/{file}: {e}"))
    };
    let manifest: Manifest = serde_json::from_str(&read("manifest.json")?)
        .map_err(|e| format!("{name}/manifest.json: {e}"))?;
    let optional = |file: &str| read_lines(dir, file).map_err(|e| format!("{name}/{file}: {e}"));
    let Some(pin) = optional(TAG_POOL_TABLE)? else {
        return Err(format!(
            "{name} has no {TAG_POOL_TABLE}: every pair built from committed tables carries its \
             studied language's pinned tag pool (scripts/lingua-data/SOURCES.md)"
        ));
    };
    let dictionary_words = match optional(LEXICAL_TABLE)? {
        Some(words) => {
            if words.windows(2).any(|w| w[0] >= w[1]) {
                return Err(format!(
                    "{name}/{LEXICAL_TABLE} is not byte-sorted with each lemma once"
                ));
            }
            words.into_iter().collect()
        }
        None => tsv_pairs(&read("gloss.tsv")?)
            .into_iter()
            .map(|(lemma, _)| lemma)
            .collect(),
    };
    Ok(Pair {
        name,
        dir: dir.to_path_buf(),
        studied: manifest.meta.studied,
        pin,
        dictionary_words,
    })
}

/// One of a pair's studied tables, byte for byte, or `None` when the pair has no such file.
fn studied_table(pair: &Pair, table: &str) -> Result<Option<Vec<u8>>, String> {
    match std::fs::read(pair.dir.join(table)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}/{table}: {e}", pair.name)),
    }
}

/// The dictionary words `a` holds and `b` does not.
fn only_in<'a>(a: &'a Pair, b: &Pair) -> Vec<&'a String> {
    a.dictionary_words
        .iter()
        .filter(|word| !b.dictionary_words.contains(*word))
        .collect()
}

/// A few of `words`, for a message.
fn sample(words: &[&String]) -> String {
    let shown: Vec<&str> = words.iter().take(5).map(|w| w.as_str()).collect();
    shown.join(", ")
}

/// Checks every pair under `root` (each folder holding a `manifest.json`): each carries a
/// `tags.tsv`, and pairs of one studied language agree on it, on their studied tables and on
/// their dictionary words.
/// Answers the pairs read, by name; an error names the pair at fault, or both pairs that
/// disagree.
pub fn check_committed_tables(root: &Path) -> Result<Vec<String>, String> {
    let mut dirs: Vec<_> = std::fs::read_dir(root)
        .map_err(|e| format!("{}: {e}", root.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|dir| dir.join("manifest.json").is_file())
        .collect();
    dirs.sort();
    let pairs = dirs
        .iter()
        .map(|dir| read_pair(dir))
        .collect::<Result<Vec<_>, _>>()?;
    for (i, pair) in pairs.iter().enumerate() {
        let Some(first) = pairs[..i].iter().find(|p| p.studied == pair.studied) else {
            continue;
        };
        if first.pin != pair.pin {
            return Err(format!(
                "{} and {} both study {:?} but pin different tag pools ({TAG_POOL_TABLE}): a \
                 form's readings would be stored differently",
                first.name, pair.name, pair.studied
            ));
        }
        for table in STUDIED_TABLES {
            if studied_table(first, table)? != studied_table(pair, table)? {
                return Err(format!(
                    "{} and {} both study {:?} but their {table} differ: a form's readings \
                     would be stored differently. A pair glossed in another native language \
                     copies its reference's studied tables.",
                    first.name, pair.name, pair.studied
                ));
            }
        }
        if first.dictionary_words != pair.dictionary_words {
            let (in_first, in_pair) = (only_in(first, pair), only_in(pair, first));
            return Err(format!(
                "{} and {} both study {:?} but hold different dictionary words: {} only in {} \
                 ({}), {} only in {} ({}). A pair glossed in another native language names its \
                 reference's glossed lemmas in {LEXICAL_TABLE}.",
                first.name,
                pair.name,
                pair.studied,
                in_first.len(),
                first.name,
                sample(&in_first),
                in_pair.len(),
                pair.name,
                sample(&in_pair),
            ));
        }
    }
    Ok(pairs.into_iter().map(|pair| pair.name).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A scratch folder of pairs, removed when dropped.
    struct Root(PathBuf);

    impl Root {
        fn new(tag: &str) -> Root {
            let dir =
                std::env::temp_dir().join(format!("lingua-committed-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Root(dir)
        }

        /// A pair studying `studied`, glossing `glossed`, with the optional tables given.
        fn pair(
            &self,
            name: &str,
            studied: &str,
            glossed: &[&str],
            tags: Option<&str>,
            lexical: Option<&str>,
        ) {
            let dir = self.0.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            let native = name.rsplit('-').next().unwrap();
            let manifest = serde_json::json!({
                "meta": {"studied": studied, "native": native, "pack_version": "t",
                         "analyzer_version": "t", "licences": []},
                "sources": []
            });
            std::fs::write(dir.join("manifest.json"), manifest.to_string()).unwrap();
            let gloss: String = glossed.iter().map(|l| format!("{l}\tG\n")).collect();
            std::fs::write(dir.join("gloss.tsv"), gloss).unwrap();
            if let Some(tags) = tags {
                std::fs::write(dir.join(TAG_POOL_TABLE), tags).unwrap();
            }
            if let Some(lexical) = lexical {
                std::fs::write(dir.join(LEXICAL_TABLE), lexical).unwrap();
            }
        }
    }

    impl Root {
        /// Writes `text` as `file` of the pair `name`.
        fn table(&self, name: &str, file: &str, text: &str) {
            std::fs::write(self.0.join(name).join(file), text).unwrap();
        }
    }

    impl Drop for Root {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const POOL: &str = "NOUN\nVERB\n";

    #[test]
    fn pairs_of_one_language_that_agree_pass() {
        let root = Root::new("agree");
        root.pair("es-fr", "es", &["casa", "dios"], Some(POOL), None);
        // Glossed in English, `augusto` too; its dictionary words are es-fr's.
        root.pair(
            "es-en",
            "es",
            &["augusto", "casa"],
            Some(POOL),
            Some("casa\ndios\n"),
        );
        // Another studied language pins its own pool.
        root.pair("en-fr", "en", &["run"], Some("VERB\n"), None);
        // A folder without a manifest is no pair.
        std::fs::create_dir_all(root.0.join("work")).unwrap();
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
    fn spec_scenario_two_packs_of_one_language_disagree() {
        let root = Root::new("disagree");
        root.pair("es-fr", "es", &["casa", "dios"], Some(POOL), None);
        root.pair(
            "es-en",
            "es",
            &["augusto", "casa"],
            Some(POOL),
            Some("augusto\ncasa\n"),
        );
        let err = check_committed_tables(&root.0).unwrap_err();
        assert!(
            err.contains("es-en and es-fr") && err.contains("dictionary words"),
            "{err}"
        );
        assert!(err.contains("augusto") && err.contains("dios"), "{err}");
        // Without a lexical table, the glossed lemmas are the dictionary words.
        let root = Root::new("disagree-glosses");
        root.pair("es-fr", "es", &["casa", "dios"], Some(POOL), None);
        root.pair("es-en", "es", &["augusto", "casa"], Some(POOL), None);
        assert!(check_committed_tables(&root.0).is_err());
    }

    #[test]
    fn two_pairs_of_one_language_pinning_different_pools_fail() {
        let root = Root::new("pools");
        root.pair("es-fr", "es", &["casa"], Some(POOL), None);
        root.pair("es-en", "es", &["casa"], Some("NOUN\nNUM\nVERB\n"), None);
        let err = check_committed_tables(&root.0).unwrap_err();
        assert!(
            err.contains("es-en and es-fr") && err.contains(TAG_POOL_TABLE),
            "{err}"
        );
    }

    #[test]
    fn spec_scenario_two_packs_of_one_language_store_readings_differently() {
        // Their pins and dictionary words agree; a studied table does not, so lemma ids or
        // readings would be stored differently.
        for (table, fr, en) in [
            ("forms.tsv", Some("casas\tcasa\n"), Some("casas\tcasar\n")),
            ("freq.tsv", Some("casa\t1\n"), Some("casa\t2\n")),
            (
                "grammar.tsv",
                Some("casa\tcasa\tNOUN\n"),
                Some("casa\tcasa\tVERB\n"),
            ),
            // Levels in one pair, none in the other.
            ("level.tsv", Some("casa\tA1\n"), None),
        ] {
            let root = Root::new(&format!("studied-{table}"));
            root.pair("es-fr", "es", &["casa"], Some(POOL), None);
            root.pair("es-en", "es", &["casa"], Some(POOL), Some("casa\n"));
            for (pair, text) in [("es-fr", fr), ("es-en", en)] {
                if let Some(text) = text {
                    root.table(pair, table, text);
                }
            }
            let err = check_committed_tables(&root.0).unwrap_err();
            assert!(
                err.contains("es-en and es-fr") && err.contains(table),
                "{table}: {err}"
            );
            // The same table, copied, passes.
            root.table("es-en", table, fr.unwrap());
            assert!(check_committed_tables(&root.0).is_ok(), "{table}");
        }
    }

    #[test]
    fn an_unreadable_studied_table_fails() {
        let root = Root::new("unreadable");
        root.pair("es-fr", "es", &["casa"], Some(POOL), None);
        root.pair("es-en", "es", &["casa"], Some(POOL), None);
        // A folder where the table should be.
        std::fs::create_dir_all(root.0.join("es-fr/forms.tsv")).unwrap();
        let err = check_committed_tables(&root.0).unwrap_err();
        assert!(err.contains("es-fr/forms.tsv"), "{err}");
    }

    #[test]
    fn a_pair_without_its_pinned_pool_fails() {
        let root = Root::new("unpinned");
        root.pair("en-fr", "en", &["run"], None, None);
        let err = check_committed_tables(&root.0).unwrap_err();
        assert!(
            err.contains("en-fr") && err.contains(TAG_POOL_TABLE),
            "{err}"
        );
    }

    #[test]
    fn a_lexical_table_out_of_order_fails() {
        let root = Root::new("unsorted");
        root.pair("es-en", "es", &["casa"], Some(POOL), Some("dios\ncasa\n"));
        let err = check_committed_tables(&root.0).unwrap_err();
        assert!(err.contains("es-en/lexical.tsv"), "{err}");
        let root = Root::new("twice");
        root.pair("es-en", "es", &["casa"], Some(POOL), Some("casa\ncasa\n"));
        assert!(check_committed_tables(&root.0).is_err());
    }

    #[test]
    fn an_unreadable_root_or_manifest_fails() {
        let root = Root::new("broken");
        assert!(check_committed_tables(&root.0.join("absent")).is_err());
        std::fs::create_dir_all(root.0.join("en-fr")).unwrap();
        std::fs::write(root.0.join("en-fr/manifest.json"), "{").unwrap();
        let err = check_committed_tables(&root.0).unwrap_err();
        assert!(err.contains("en-fr/manifest.json"), "{err}");
    }
}
