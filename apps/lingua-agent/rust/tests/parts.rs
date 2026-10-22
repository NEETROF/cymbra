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

//! A pack's base or layer among the whole packs (split-lingua-pack-format D1): the plugin reads a
//! file's pair with `Pack::pair_in`, which refuses a part by name, so it skips the part as any file
//! the core cannot read and follows the whole packs. A layer read as its pair's pack would have been
//! filed first for its language (`en-fr.layer.lingua` sorts before `pack.lingua`) and evicted it.

use std::path::{Path, PathBuf};

use lingua_agent::engine::Library;
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::knowledge::profile::NativeLanguage;
use lingua_core::packs::pack::section;
use lingua_core::packs::{Pack, read_container};

const PACK_BYTES: &[u8] = include_bytes!("fixtures/pack.lingua");
const ES_PACK_BYTES: &[u8] = include_bytes!("fixtures/es-fr.lingua");

/// The en-fr fixture built as a base and a layer: pinned with the pool its whole pack carries, the
/// lemmas it glosses made the lemmas of their own forms (neither moves a byte of its whole pack),
/// and named apart by its `pack_version`.
fn parts() -> lingua_pack::SplitPack {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../scripts/lingua-data/testdata/en-fr");
    let mut inputs = lingua_pack::inputs_from_dir(&dir).expect("the en-fr fixture");
    let whole = lingua_pack::build_pack(&inputs).expect("its whole pack");
    let (_, sections) = read_container(&whole).unwrap();
    let pool = sections
        .iter()
        .find(|s| s.name == section::TAGS)
        .map(|s| String::from_utf8(s.data.clone()).unwrap())
        .expect("its pool");
    inputs.tag_pool = Some(pool.lines().map(str::to_owned).collect());
    let glossed: Vec<String> = inputs
        .glosses
        .iter()
        .map(|(lemma, _)| lemma.clone())
        .collect();
    for lemma in glossed {
        inputs.form_lemma.push((lemma.clone(), lemma));
    }
    inputs.meta.pack_version = "parts".into();
    lingua_pack::build_split(&inputs).expect("its parts")
}

fn installed(name: &str, files: &[(&str, &[u8])]) -> (PathBuf, Vec<PathBuf>) {
    let dir = std::env::temp_dir().join(format!("lingua-agent-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    for (file, bytes) in files {
        std::fs::write(dir.join(file), bytes).unwrap();
    }
    let mut paths: Vec<_> = files.iter().map(|(file, _)| dir.join(file)).collect();
    paths.sort();
    (dir, paths)
}

#[test]
fn spec_scenario_a_part_among_whole_packs_is_skipped() {
    let parts = parts();
    let (dir, files) = installed(
        "parts",
        &[
            ("en-fr.base.lingua", &parts.base),
            ("en-fr.layer.lingua", &parts.layer),
            ("es-fr.lingua", ES_PACK_BYTES),
            ("pack.lingua", PACK_BYTES),
        ],
    );
    let mut library = Library::from_files(files);
    // The whole packs are followed, English from `pack.lingua`, and nothing is skipped as glossed
    // in another language: a part is no pack at all.
    assert_eq!(
        library.languages(),
        vec![StudiedLanguage::English, StudiedLanguage::Spanish]
    );
    assert_eq!(library.native(), Some(NativeLanguage::French));
    assert!(library.skipped().is_empty());
    let english = library
        .pack(StudiedLanguage::English)
        .expect("pack.lingua loads");
    assert_eq!(
        english.meta(),
        Pack::load(PACK_BYTES).unwrap().meta(),
        "English is pack.lingua's"
    );
    // The parts alone: nothing to follow.
    let (alone, files) = installed(
        "parts-alone",
        &[
            ("en-fr.base.lingua", &parts.base),
            ("en-fr.layer.lingua", &parts.layer),
        ],
    );
    let library = Library::from_files(files);
    assert!(library.languages().is_empty());
    assert_eq!(library.native(), None);
    std::fs::remove_dir_all(&dir).ok();
    std::fs::remove_dir_all(&alone).ok();
}
