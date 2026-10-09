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

//! Runs the pack pipeline over the committed tiny testdata (no download):
//! proves it builds from real on-disk tables, is reproducible, stays under
//! budget, and — with the analyser version aligned — round-trips through the
//! `lingua-core` reader. The full EN→FR pack uses the same code path over the
//! real AGID/wordfreq/kaikki tables in CI/dev.

use std::path::PathBuf;

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::lexicon::Lexicon;
use lingua_core::analysis::{ANALYZER_VERSION, FRENCH_ANALYZER_VERSION};
use lingua_core::knowledge::state::FrequencyRanks;
use lingua_core::packs::Pack;
use lingua_pack::{MAX_PACK_BYTES, build_pack, inputs_from_dir};

fn testdata_dir() -> PathBuf {
    // crates/lingua-pack/ -> ../../scripts/lingua-data/testdata/en-fr
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/lingua-data/testdata/en-fr")
}

#[test]
fn pipeline_build_is_reproducible_and_under_budget() {
    let inputs = inputs_from_dir(&testdata_dir()).expect("read testdata");
    let a = build_pack(&inputs).expect("build a");
    let b = build_pack(&inputs).expect("build b");
    assert_eq!(
        a, b,
        "two builds over the same sources must be byte-identical"
    );
    assert!(a.len() < MAX_PACK_BYTES);
}

#[test]
fn pipeline_output_round_trips_through_the_reader() {
    let mut inputs = inputs_from_dir(&testdata_dir()).expect("read testdata");
    // Align the fixture with whatever analyser this build declares, so the
    // reader accepts it regardless of version bumps.
    inputs.meta.analyzer_version = ANALYZER_VERSION.to_owned();
    let bytes = build_pack(&inputs).expect("build");

    let pack = Pack::load(&bytes).expect("load");
    assert_eq!(pack.meta().pair_key(), "en-fr");
    assert_eq!(pack.lexicon().lemma_of("running"), Some("run"));
    assert_eq!(pack.rank("run"), Some(500));
    assert_eq!(pack.gloss("conundrum"), Some("casse-tête"));
    assert!(pack.notice().contains("CC BY-SA"));
    // A manifest that says nothing of its levels: not estimated.
    assert!(!pack.meta().levels_estimated);
}

#[test]
fn every_testdata_fixture_builds_and_names_its_pair() {
    // The tiny fixtures, one folder per pair (`build.sh --testdata`): es-fr's, and es-en's
    // (add-lingua-pack-es-en D6), which `gen:pack` builds once the pair ships, and fr-en's, the
    // French baseline's (add-lingua-french-baseline D5). Each builds under budget and names its
    // pair; each studies with its own language's analyser.
    let root = testdata_dir().parent().unwrap().to_path_buf();
    let mut pairs: Vec<String> = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains('-'))
        .collect();
    pairs.sort();
    assert!(pairs.iter().any(|p| p == "es-en"), "{pairs:?}");
    for pair in &pairs {
        let mut inputs =
            inputs_from_dir(&root.join(pair)).unwrap_or_else(|e| panic!("{pair}: {e}"));
        let (studied, native) = pair.split_once('-').unwrap();
        assert_eq!(
            (inputs.meta.studied.as_str(), inputs.meta.native.as_str()),
            (studied, native),
            "{pair}"
        );
        inputs.meta.analyzer_version = StudiedLanguage::from_tag(studied)
            .unwrap_or_else(|| panic!("{pair} studies a language the core analyses"))
            .analyzer_version()
            .to_owned();
        let bytes = build_pack(&inputs).unwrap_or_else(|e| panic!("build {pair}: {e}"));
        assert!(bytes.len() < MAX_PACK_BYTES, "{pair}");
        let pack = Pack::load(&bytes).unwrap_or_else(|e| panic!("load {pair}: {e}"));
        assert_eq!(pack.meta().pair_key(), *pair);
        assert!(
            pack.gloss("casa").is_some() || studied != "es",
            "{pair} glosses casa"
        );
    }
}

#[test]
fn the_french_fixture_studies_french_at_its_analyser_version() {
    // add-lingua-french-baseline D5: hand-written tables, stamped with French's analyser version
    // as committed (not re-stamped here), glossed in English, no grammar and no senses. A French
    // rule that bumps the version bumps the committed manifest with it
    // (add-lingua-french-tokenisation: `0.2.0`).
    let dir = testdata_dir().parent().unwrap().join("fr-en");
    let inputs = inputs_from_dir(&dir).expect("read the fr-en fixture");
    assert!(inputs.readings.is_empty() && inputs.senses.is_empty());
    assert!(
        !inputs.meta.levels_estimated,
        "a fixture's levels decide nothing"
    );
    assert!(inputs.meta.pack_version.contains("fixture"));
    let bytes = build_pack(&inputs).expect("build");
    assert!(bytes.len() < MAX_PACK_BYTES);
    let pack = Pack::load(&bytes).expect("the core loads it at French's version");
    assert_eq!(pack.studied(), StudiedLanguage::French);
    assert_eq!(pack.meta().analyzer_version, FRENCH_ANALYZER_VERSION);
    assert_eq!(pack.meta().pair_key(), "fr-en");
    // The single-letter words the tokeniser keeps only when the lexicon lists them.
    for single in ["a", "à", "y"] {
        assert!(pack.lexicon().contains(single), "{single:?}");
    }
    assert_eq!(pack.lexicon().lemma_of("est"), Some("être"));
    assert_eq!(pack.gloss("maison"), Some("house, home"));
    assert!(pack.notice().contains("hand-written"));
}

#[test]
fn pipeline_reads_the_optional_expression_table() {
    let inputs = inputs_from_dir(&testdata_dir()).expect("read testdata");
    // `mwe.tsv` reaches the builder as the reducer wrote it, spellings and all:
    // `city running` is still a line of its own here, and only the builder,
    // holding the lexicon, folds it into the key `city run`.
    assert!(inputs.expressions.contains(&(
        "city running".to_owned(),
        "Course à pied en ville".to_owned()
    )));
    assert!(build_pack(&inputs).expect("build").len() < MAX_PACK_BYTES);
}

#[test]
fn pipeline_reads_the_optional_grammar_tables() {
    // `grammar.tsv` and `senses.tsv` reach the builder as the reducer wrote
    // them; the builder files `leaves`' reading of `leaf` under `leave`, the
    // dictionary form the analysis reads `leaves` as.
    let mut inputs = inputs_from_dir(&testdata_dir()).expect("read testdata");
    inputs.meta.analyzer_version = ANALYZER_VERSION.to_owned();
    assert!(!inputs.readings.is_empty() && !inputs.senses.is_empty());
    let pack = Pack::load(&build_pack(&inputs).expect("build")).expect("load");
    assert!(pack.has_grammar());
    assert_eq!(
        pack.readings("go", "went")
            .iter()
            .map(|t| t.to_ud())
            .collect::<Vec<_>>(),
        ["VERB|Mood=Ind|Tense=Past|VerbForm=Fin"]
    );
    let others = pack.other_readings("leave", "leaves");
    assert_eq!(others.len(), 1);
    assert_eq!(others[0].0, "leaf");
    assert_eq!(pack.sense_runs("can").len(), 2);
}
