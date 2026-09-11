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
//! from AGID (form→lemma), wordfreq (ranks) and kaikki (French glosses):
//! builds the FST, quantises ranks, compresses the offset-indexed glosses
//! with zstd, embeds the metadata and NOTICE, and enforces the licence
//! denylist and the size budget. Native-only — the reader that consumes the
//! output lives in `lingua-core` and stays WASM-clean.

pub mod licence;
pub mod manifest;

use std::path::Path;

use lingua_core::analysis::lexicon::{FstLexicon, build_lexicon_blobs};
use lingua_core::packs::format::write_container;
use lingua_core::packs::meta::PackMeta;
use lingua_core::packs::pack::section;
use serde::Deserialize;

/// The maximum size, in bytes, of a pack embedded in the extension (design
/// D4). Overshooting fails the build; the remedy is fewer glossed lemmas.
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
    /// Inflected form → lemma pairs (AGID-derived).
    pub form_lemma: Vec<(String, String)>,
    /// Lemmas that carry a frequency rank (wordfreq-derived); 1 = commonest.
    pub ranks: Vec<(String, u32)>,
    /// Lemma → native-language gloss (kaikki-derived).
    pub glosses: Vec<(String, String)>,
    /// The full attribution NOTICE text.
    pub notice: String,
    /// The sources actually used, for the licence guard.
    pub sources: Vec<licence::Source>,
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
    /// The assembled pack exceeds [`MAX_PACK_BYTES`].
    OverBudget { size: usize, budget: usize },
}

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
            BuildError::OverBudget { size, budget } => write!(
                f,
                "pack is {size} bytes, over the {budget}-byte budget; reduce glossed lemmas"
            ),
        }
    }
}

impl std::error::Error for BuildError {}

/// Loads a pack's inputs from a directory holding `manifest.json`,
/// `forms.tsv`, `freq.tsv`, `gloss.tsv` and `NOTICE`. Shared by the CLI and
/// the pipeline test.
pub fn inputs_from_dir(dir: &Path) -> std::io::Result<PackInputs> {
    let read = |name: &str| std::fs::read_to_string(dir.join(name));
    let manifest: Manifest = serde_json::from_str(&read("manifest.json")?)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(PackInputs {
        meta: manifest.meta,
        form_lemma: tsv_pairs(&read("forms.tsv")?),
        ranks: tsv_pairs(&read("freq.tsv")?)
            .into_iter()
            .filter_map(|(lemma, rank)| rank.parse::<u32>().ok().map(|r| (lemma, r)))
            .collect(),
        glosses: tsv_pairs(&read("gloss.tsv")?),
        notice: read("NOTICE")?,
        sources: manifest.sources,
    })
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

    let meta_json = serde_json::to_vec(&inputs.meta).expect("PackMeta serialises");
    let pack = write_container(
        &meta_json,
        &[
            (section::FORMS, &forms),
            (section::LEMMAS, pool.as_bytes()),
            (section::FREQ, &freq_bytes),
            (section::GLOSS_ZST, &gloss_zst),
            (section::NOTICE, inputs.notice.as_bytes()),
        ],
    );

    if pack.len() > MAX_PACK_BYTES {
        return Err(BuildError::OverBudget {
            size: pack.len(),
            budget: MAX_PACK_BYTES,
        });
    }
    Ok(pack)
}

/// Encodes the gloss index + payload and zstd-compresses it. Layout matches
/// the reader: `count u32 | count*(id u32, off u32, len u32) | utf8`.
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
            notice: "AGID (permissive), wordfreq (CC BY-SA), kaikki (CC BY-SA).".into(),
            sources: sources(),
        }
    }

    #[test]
    fn built_pack_round_trips_through_the_reader() {
        let bytes = build_pack(&inputs()).expect("build");
        let pack = Pack::load(&bytes).expect("load");
        assert_eq!(pack.meta().pair_key(), "en->fr");
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

    /// Staleness guard for the pack registry the Lingua ops console serves (change:
    /// add-lingua-back-office, task 3.3). Rebuilds the committed testdata pack and
    /// asserts the committed `packs-manifest.json` entry still describes it by content
    /// (version + size + NOTICE). A drift here means `scripts/lingua-data/build.sh
    /// emit-manifest` was not re-run and committed.
    #[test]
    fn the_committed_pack_manifest_matches_a_fresh_build() {
        use std::path::Path;
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dir = root.join("../../scripts/lingua-data/testdata/en-fr");
        let inputs = inputs_from_dir(&dir).expect("read testdata inputs");
        let bytes = build_pack(&inputs).expect("build testdata pack");

        let manifest =
            std::fs::read_to_string(root.join("../../backend/lingua/packs-manifest.json"))
                .expect("read committed packs-manifest.json");
        let v: serde_json::Value = serde_json::from_str(&manifest).expect("valid manifest json");
        let entry = v["packs"]
            .as_array()
            .expect("packs array")
            .iter()
            .find(|p| p["studied"] == *inputs.meta.studied && p["native"] == *inputs.meta.native)
            .expect("the testdata pair is listed in the committed manifest");

        let stale = "packs-manifest.json is stale — re-run \
                     `scripts/lingua-data/build.sh emit-manifest` and commit it";
        assert_eq!(entry["pack_version"], *inputs.meta.pack_version, "{stale}");
        assert_eq!(
            entry["analyzer_version"], *inputs.meta.analyzer_version,
            "{stale}"
        );
        assert_eq!(
            entry["size_bytes"].as_i64(),
            Some(bytes.len() as i64),
            "{stale}"
        );
        assert_eq!(
            entry["notice"].as_str(),
            Some(inputs.notice.as_str()),
            "{stale}"
        );
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

    #[test]
    fn over_budget_fails_with_a_clear_error() {
        let mut inp = inputs();
        // A gloss that stays over 5 MB even after zstd: a pseudo-random,
        // poorly-compressible byte soup rather than a repeating pattern.
        let mut big = String::with_capacity(9_000_000);
        let mut x: u32 = 0x1234_5678;
        for _ in 0..9_000_000 {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            // Spread over the 95 printable ASCII symbols: near-uniform entropy,
            // so zstd cannot bring 9 MB back under the 5 MB budget.
            big.push(char::from(b' ' + (x >> 17) as u8 % 95));
        }
        inp.glosses = vec![("run".into(), big)];
        match build_pack(&inp) {
            Err(BuildError::OverBudget { budget, size }) => {
                assert_eq!(budget, MAX_PACK_BYTES);
                assert!(size > budget);
            }
            other => panic!("expected OverBudget, got {other:?}"),
        }
    }
}
