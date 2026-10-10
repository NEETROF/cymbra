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

//! Forms→lemmas lexicon backed by an FST, readable from plain byte slices.
//!
//! The pack ships two blobs: an [`fst::Map`] keyed by lowercase surface form
//! whose value is a lemma id, and a lemma pool (newline-separated lowercase
//! lemmas, indexed by that id). Both are consumed as `AsRef<[u8]>` /
//! `&str` so `include_bytes!` / `include_str!` work unchanged under WASM.
//! The container format around these blobs arrives with
//! `add-lingua-data-pack`.

use std::collections::HashSet;

/// Read access to the studied language's lexicon.
///
/// The lemmatisation cascade validates its candidates against this, the
/// tokenizer gates single-letter tokens on it, and out-of-lexicon proper-noun
/// exclusion relies on it.
pub trait Lexicon {
    /// The lemma for a lowercase surface form, when the form is a known
    /// inflection (or a lemma itself listed as its own form).
    fn lemma_of(&self, form_lower: &str) -> Option<&str>;

    /// Whether `lemma` (lowercase) is a lemma of the lexicon.
    fn contains_lemma(&self, lemma: &str) -> bool;

    /// Whether the lowercase string resolves at all (as a form or a lemma).
    fn contains(&self, word_lower: &str) -> bool {
        self.lemma_of(word_lower).is_some() || self.contains_lemma(word_lower)
    }
}

/// Error building a [`FstLexicon`] from its two blobs.
#[derive(Debug)]
pub enum LexiconError {
    /// The FST bytes are not a valid `fst::Map`.
    InvalidFst(fst::Error),
    /// A form points to a lemma id outside the lemma pool.
    LemmaIdOutOfRange {
        form: String,
        id: u64,
        pool_len: usize,
    },
    /// The lemma pool is not in strictly ascending byte order: the lemma at id
    /// `at` sorts before the one above it, `previous`, or repeats it. A lemma is
    /// found by its place in the pool ([`FstLexicon::lemma_id`]), which only a
    /// sorted pool with no repeat gives (fix-lingua-lemma-lookup D1).
    LemmaPoolOutOfOrder {
        at: usize,
        lemma: String,
        previous: String,
    },
}

impl std::fmt::Display for LexiconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexiconError::InvalidFst(e) => write!(f, "invalid forms FST: {e}"),
            LexiconError::LemmaIdOutOfRange { form, id, pool_len } => write!(
                f,
                "form {form:?} points at lemma id {id} but the pool holds {pool_len} lemmas"
            ),
            LexiconError::LemmaPoolOutOfOrder {
                at,
                lemma,
                previous,
            } if lemma == previous => {
                write!(
                    f,
                    "the lemma pool names {lemma:?} twice, at ids {} and {at}",
                    at - 1
                )
            }
            LexiconError::LemmaPoolOutOfOrder {
                at,
                lemma,
                previous,
            } => write!(
                f,
                "the lemma pool is out of order at id {at}: {lemma:?} after {previous:?}"
            ),
        }
    }
}

impl std::error::Error for LexiconError {}

/// FST-backed lexicon over borrowed or owned blobs.
pub struct FstLexicon<D: AsRef<[u8]>> {
    forms: fst::Map<D>,
    lemma_pool: Vec<String>,
    lemma_set: HashSet<String>,
}

impl<D: AsRef<[u8]>> FstLexicon<D> {
    /// Builds a lexicon from the two pack blobs.
    ///
    /// `forms_fst` is an `fst::Map` (form → lemma id); `lemma_pool` is the
    /// newline-separated lemma list the ids index into. Both are validated once,
    /// here, so lookups can index without checking: the pool must be in strictly
    /// ascending byte order, as [`build_lexicon_blobs`] writes it, so that a
    /// lemma is found by binary search ([`FstLexicon::lemma_id`]); and every id
    /// stored in the map must point into it.
    pub fn from_slices(forms_fst: D, lemma_pool: &str) -> Result<Self, LexiconError> {
        let forms = fst::Map::new(forms_fst).map_err(LexiconError::InvalidFst)?;
        let lemma_pool: Vec<String> = lemma_pool
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect();
        if let Some(at) = lemma_pool.windows(2).position(|pair| pair[0] >= pair[1]) {
            return Err(LexiconError::LemmaPoolOutOfOrder {
                at: at + 1,
                lemma: lemma_pool[at + 1].clone(),
                previous: lemma_pool[at].clone(),
            });
        }
        let mut stream = forms.stream();
        while let Some((key, id)) = fst::Streamer::next(&mut stream) {
            if id as usize >= lemma_pool.len() {
                return Err(LexiconError::LemmaIdOutOfRange {
                    form: String::from_utf8_lossy(key).into_owned(),
                    id,
                    pool_len: lemma_pool.len(),
                });
            }
        }
        let lemma_set = lemma_pool.iter().cloned().collect();
        Ok(Self {
            forms,
            lemma_pool,
            lemma_set,
        })
    }
}

impl<D: AsRef<[u8]>> FstLexicon<D> {
    /// The lemma id the forms map `form_lower` to (an index into the lemma
    /// pool): the form lookup, the analysis's. A lemma's own spelling gives its
    /// own id only while the forms read it as itself — which the pack builder
    /// requires of every lemma of a pack it writes (fix-lingua-lemma-lookup D4)
    /// —, so a lemma is found among the lemmas by [`FstLexicon::lemma_id`]. The
    /// pack reads its estimates (rank, level, dictionary-word mark) through this
    /// lookup, for the lemma the forms read a string as (D3).
    pub fn id_of(&self, form_lower: &str) -> Option<u64> {
        self.forms.get(form_lower.as_bytes())
    }

    /// The id of `lemma` itself: its place in the lemma pool, found by binary
    /// search over the pool `from_slices` checked sorted. `None` for a string the
    /// pool does not hold as a lemma — a form of another lemma included, which
    /// [`FstLexicon::id_of`] reads as that other lemma (fix-lingua-lemma-lookup
    /// D1). What the pack files under a lemma is read by this id.
    pub fn lemma_id(&self, lemma: &str) -> Option<u64> {
        self.lemma_pool
            .binary_search_by(|probe| probe.as_str().cmp(lemma))
            .ok()
            .map(|at| at as u64)
    }

    /// The lemma at a given id, if in range.
    pub fn lemma_at(&self, id: u64) -> Option<&str> {
        self.lemma_pool.get(id as usize).map(String::as_str)
    }

    /// Number of lemmas in the pool.
    pub fn lemma_count(&self) -> usize {
        self.lemma_pool.len()
    }
}

impl<D: AsRef<[u8]>> Lexicon for FstLexicon<D> {
    fn lemma_of(&self, form_lower: &str) -> Option<&str> {
        self.forms
            .get(form_lower.as_bytes())
            .map(|id| self.lemma_pool[id as usize].as_str())
    }

    fn contains_lemma(&self, lemma: &str) -> bool {
        self.lemma_set.contains(lemma)
    }
}

/// Builds the two lexicon blobs from (form, lemma) pairs.
///
/// Shared by the test fixtures of this crate and by the offline pack builder
/// (`lingua-pack`). Lemmas absent from the pairs' forms are still valid pool
/// entries: pass them through `extra_lemmas`. The pool is written in strictly
/// ascending byte order, with no repeat, which [`FstLexicon::from_slices`]
/// checks.
pub fn build_lexicon_blobs(
    pairs: &[(&str, &str)],
    extra_lemmas: &[&str],
) -> Result<(Vec<u8>, String), fst::Error> {
    let mut lemma_pool: Vec<&str> = pairs
        .iter()
        .map(|(_, lemma)| *lemma)
        .chain(extra_lemmas.iter().copied())
        .collect();
    lemma_pool.sort_unstable();
    lemma_pool.dedup();

    let lemma_id = |lemma: &str| -> u64 {
        lemma_pool
            .binary_search(&lemma)
            .expect("lemma just inserted") as u64
    };

    // A lemma is always resolvable as its own form (identity inflection),
    // unless the pairs already map that surface elsewhere: each entry carries
    // whether it is a lemma's identity, and sorts after the pairs' entries for
    // the same form, so a form the pairs map reads as the pairs' lemma
    // (fix-lingua-lemma-lookup D5). Between two pairs for one form — which the
    // pack builder refuses, but a test fixture may write — the byte-wise first
    // lemma wins: the pool is sorted, so its id is the smaller.
    let mut entries: Vec<(&str, bool, u64)> = pairs
        .iter()
        .map(|(form, lemma)| (*form, false, lemma_id(lemma)))
        .collect();
    for lemma in &lemma_pool {
        entries.push((lemma, true, lemma_id(lemma)));
    }
    entries.sort_unstable();
    entries.dedup_by_key(|(form, _, _)| *form);

    let mut builder = fst::MapBuilder::memory();
    for (form, _, id) in entries {
        builder.insert(form, id)?;
    }
    Ok((builder.into_inner()?, lemma_pool.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_roundtrip_lookup_and_membership() {
        let (fst_bytes, pool) = build_lexicon_blobs(
            &[("went", "go"), ("cities", "city"), ("ran", "run")],
            &["seldom"],
        )
        .expect("build");
        let lex = FstLexicon::from_slices(fst_bytes.as_slice(), &pool).expect("load");

        assert_eq!(lex.lemma_of("went"), Some("go"));
        assert_eq!(lex.lemma_of("cities"), Some("city"));
        // Lemmas resolve as their own form.
        assert_eq!(lex.lemma_of("go"), Some("go"));
        assert_eq!(lex.lemma_of("seldom"), Some("seldom"));
        assert_eq!(lex.lemma_of("absent"), None);

        assert!(lex.contains_lemma("run"));
        assert!(!lex.contains_lemma("went"));
        assert!(lex.contains("went"));
        assert!(!lex.contains("absent"));
    }

    #[test]
    fn invalid_fst_bytes_are_rejected() {
        let err = FstLexicon::from_slices(b"not an fst".as_slice(), "go");
        assert!(matches!(err, Err(LexiconError::InvalidFst(_))));
    }

    #[test]
    fn out_of_range_lemma_id_is_rejected_at_load() {
        // Hand-build a map whose value exceeds the pool length.
        let mut builder = fst::MapBuilder::memory();
        builder.insert("went", 7).expect("insert");
        let bytes = builder.into_inner().expect("bytes");
        let err = FstLexicon::from_slices(bytes.as_slice(), "go");
        assert!(matches!(
            err,
            Err(LexiconError::LemmaIdOutOfRange { id: 7, .. })
        ));
    }

    #[test]
    fn error_messages_name_the_problem() {
        let mut builder = fst::MapBuilder::memory();
        builder.insert("went", 7).expect("insert");
        let bytes = builder.into_inner().expect("bytes");
        let Err(err) = FstLexicon::from_slices(bytes.as_slice(), "go") else {
            panic!("expected an out-of-range error");
        };
        let msg = err.to_string();
        assert!(msg.contains("went") && msg.contains('7'), "{msg}");
        let Err(err2) = FstLexicon::from_slices(b"junk".as_slice(), "") else {
            panic!("expected an invalid-FST error");
        };
        assert!(err2.to_string().contains("invalid forms FST"), "{err2}");
    }

    // — a lemma found among the lemmas (fix-lingua-lemma-lookup D1, D5) —

    fn load(pairs: &[(&str, &str)], extra: &[&str]) -> FstLexicon<Vec<u8>> {
        let (fst_bytes, pool) = build_lexicon_blobs(pairs, extra).expect("build");
        FstLexicon::from_slices(fst_bytes, &pool).expect("load")
    }

    fn empty_fst() -> Vec<u8> {
        fst::MapBuilder::memory()
            .into_inner()
            .expect("an empty map")
    }

    #[test]
    fn a_lemma_is_found_by_its_place_in_the_pool() {
        let lex = load(&[("saw", "see"), ("seen", "see")], &["go"]);
        assert_eq!(lex.lemma_id("go"), Some(0));
        assert_eq!(lex.lemma_id("see"), Some(1));
        assert_eq!(lex.lemma_at(1), Some("see"));
        // A lemma the forms read as itself: both lookups agree.
        assert_eq!(lex.lemma_id("see"), lex.id_of("see"));
    }

    #[test]
    fn a_form_of_another_lemma_is_no_lemma() {
        let lex = load(&[("saw", "see"), ("seen", "see")], &["go"]);
        // The form lookup reads `saw` as `see`; among the lemmas, `saw` is none.
        assert_eq!(lex.id_of("saw"), lex.lemma_id("see"));
        assert_eq!(lex.lemma_id("saw"), None);
        assert_eq!(lex.lemma_id("seen"), None);
    }

    #[test]
    fn an_absent_word_is_no_lemma() {
        let lex = load(&[("saw", "see")], &["go"]);
        for absent in ["absent", "", "Go", "goes"] {
            assert_eq!(lex.lemma_id(absent), None, "{absent:?}");
        }
    }

    #[test]
    fn a_pool_out_of_order_is_refused_naming_the_place_and_the_lemma() {
        let Err(err) = FstLexicon::from_slices(empty_fst(), "go\nseldom\ncity") else {
            panic!("a pool out of order loaded");
        };
        assert!(
            matches!(
                &err,
                LexiconError::LemmaPoolOutOfOrder { at: 2, lemma, previous }
                    if lemma == "city" && previous == "seldom"
            ),
            "{err:?}"
        );
        assert_eq!(
            err.to_string(),
            "the lemma pool is out of order at id 2: \"city\" after \"seldom\""
        );
    }

    #[test]
    fn a_pool_naming_a_lemma_twice_is_refused_naming_it() {
        let Err(err) = FstLexicon::from_slices(empty_fst(), "city\ngo\ngo\nrun") else {
            panic!("a pool repeating a lemma loaded");
        };
        assert!(
            matches!(&err, LexiconError::LemmaPoolOutOfOrder { at: 2, lemma, .. } if lemma == "go"),
            "{err:?}"
        );
        assert_eq!(
            err.to_string(),
            "the lemma pool names \"go\" twice, at ids 1 and 2"
        );
    }

    #[test]
    fn the_pool_is_ordered_byte_wise_as_the_builder_writes_it() {
        // `Zeta` (0x5A) sorts before `alpha`, `zèbre` before `été` (0xC3): byte order, not
        // alphabetical order.
        let pool = "Zeta\nalpha\nzèbre\nété";
        let lex = FstLexicon::from_slices(empty_fst(), pool).expect("a byte-wise sorted pool");
        assert_eq!(lex.lemma_id("été"), Some(3));
        assert_eq!(lex.lemma_id("zèbre"), Some(2));
        // The builder writes exactly that order.
        let (_, built) = build_lexicon_blobs(&[], &["été", "alpha", "zèbre", "Zeta"]).unwrap();
        assert_eq!(built, pool);
    }

    #[test]
    fn a_form_the_pairs_map_reads_as_their_lemma_not_as_a_lemma_spelled_alike() {
        // `saisie` sorts before `saisir`: its identity entry won on the smaller id, and the form
        // read as itself against the pairs. The pairs win now (D5), and `saisie` is still found
        // among the lemmas.
        let lex = load(&[("saisie", "saisir")], &["saisie"]);
        assert_eq!(lex.lemma_of("saisie"), Some("saisir"));
        assert!(lex.contains_lemma("saisie"));
        assert_eq!(lex.lemma_id("saisie"), Some(0));
        assert_eq!(lex.id_of("saisie"), Some(1));
        // `venue` sorts after `venir`: read as venir, as before.
        let lex = load(&[("venue", "venir")], &["venue"]);
        assert_eq!(lex.lemma_of("venue"), Some("venir"));
        assert_eq!(lex.lemma_id("venue"), Some(1));
        assert_eq!(lex.lemma_id("venir"), Some(0));
        // A lemma the pairs do not map elsewhere still reads as itself.
        assert_eq!(lex.lemma_of("venir"), Some("venir"));
    }

    #[test]
    fn between_two_pairs_for_one_form_the_byte_wise_first_lemma_wins() {
        // The pack builder refuses such a table; a fixture may still write one.
        let lex = load(&[("porte", "porter"), ("porte", "porte")], &[]);
        assert_eq!(lex.lemma_of("porte"), Some("porte"));
        let lex = load(&[("lu", "luire"), ("lu", "lire")], &[]);
        assert_eq!(lex.lemma_of("lu"), Some("lire"));
    }
}
