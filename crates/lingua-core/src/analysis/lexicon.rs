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
}

impl std::fmt::Display for LexiconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexiconError::InvalidFst(e) => write!(f, "invalid forms FST: {e}"),
            LexiconError::LemmaIdOutOfRange { form, id, pool_len } => write!(
                f,
                "form {form:?} points at lemma id {id} but the pool holds {pool_len} lemmas"
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
    /// newline-separated lemma list the ids index into. Every id stored in
    /// the map is validated against the pool once, here, so lookups can
    /// index without checking.
    pub fn from_slices(forms_fst: D, lemma_pool: &str) -> Result<Self, LexiconError> {
        let forms = fst::Map::new(forms_fst).map_err(LexiconError::InvalidFst)?;
        let lemma_pool: Vec<String> = lemma_pool
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect();
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
    /// The lemma id a form maps to (the index into the lemma pool). A lemma
    /// resolves to its own id. Used by the data pack to key frequency ranks
    /// and glosses by lemma id.
    pub fn id_of(&self, form_lower: &str) -> Option<u64> {
        self.forms.get(form_lower.as_bytes())
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
/// Shared by the test fixtures of this crate and, later, by the offline pack
/// pipeline (`scripts/lingua-data`). Lemmas absent from the pairs' forms are
/// still valid pool entries: pass them through `extra_lemmas`.
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

    let mut entries: Vec<(&str, u64)> = pairs
        .iter()
        .map(|(form, lemma)| (*form, lemma_id(lemma)))
        .collect();
    // A lemma is always resolvable as its own form (identity inflection),
    // unless the pairs already map that surface elsewhere.
    for lemma in &lemma_pool {
        entries.push((lemma, lemma_id(lemma)));
    }
    entries.sort_unstable();
    entries.dedup_by_key(|(form, _)| *form);

    let mut builder = fst::MapBuilder::memory();
    for (form, id) in entries {
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
}
