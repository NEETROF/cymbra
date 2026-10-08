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

//! A second pack over a reference pair's studied tables, as a pack glossed in another native
//! language would be (add-lingua-pack-lexical-layer D6): `cross_native.rs` answers every probe of
//! a language's baseline through both. Spanish needs none since es-en's tables are committed
//! (add-lingua-pack-es-en): the real pack, built from `tables/es/` and `tables/es-en/`, is the
//! second pack there, and `card_gloss_language.rs` creates on it the cards a French-native
//! engine reviews. English glossed in Spanish stays synthetic until en-es's tables land
//! (change 22).
//!
//! The second pack:
//! - names another native language in its metadata;
//! - keeps about 70 % of the glosses (a deterministic hash) and glosses lemmas the reference
//!   does not;
//! - keeps fewer expressions and sense runs, and a few runs carry a part of speech no reading
//!   uses;
//! - names the reference's glossed lemmas as its dictionary words (`lexical.tsv`) and pins
//!   the reference's tag pool (`tags.tsv`).

use std::collections::BTreeSet;

use lingua_pack::{PackInputs, inputs_from_tables};

use super::Scenario;

/// FNV-1a: a hash that depends on the lemma alone, so the glosses kept never move.
fn fnv(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Whether `key` falls in the first `percent` of the hash's range.
fn kept(key: &str, percent: u64) -> bool {
    fnv(key) % 100 < percent
}

/// How the second pack differs from the reference, on the native side alone.
pub struct OtherNative {
    /// The reference pair, whose committed tables both packs are built from.
    pub pair: &'static str,
    /// The native language the second pack is glossed in.
    pub native: &'static str,
    /// Glosses the second pack keeps whatever the hash says.
    pub keep: &'static [&'static str],
    /// Ranked lemmas the reference does not gloss and the second pack does.
    pub also: &'static [&'static str],
    /// The part of speech of the runs of those new glosses: one no reading uses, outside the
    /// pinned pool.
    pub new_tag: &'static str,
    /// Sense tags the second pack's glosses never carry.
    pub dropped_tags: &'static [&'static str],
}

impl OtherNative {
    /// The reference pack's inputs, and the second pack's.
    pub fn inputs(&self) -> (PackInputs, PackInputs) {
        let root = Scenario::tables_root();
        let read = || {
            inputs_from_tables(&root, self.pair).unwrap_or_else(|e| panic!("{}: {e}", self.pair))
        };
        let reference = read();
        assert!(
            reference.tag_pool.is_some(),
            "{} pins its studied language's pool",
            self.pair
        );
        let glossed: BTreeSet<String> = reference.glosses.iter().map(|(l, _)| l.clone()).collect();
        // A reference's dictionary words are its glossed lemmas: its pack carries no lexical
        // section (asserted by `cross_native.rs`), whatever tables/<studied>/lexical.tsv repeats.
        assert_eq!(
            reference
                .lexical
                .as_ref()
                .map(|w| w.iter().cloned().collect::<BTreeSet<_>>()),
            Some(glossed.clone()),
            "{} is a reference",
            self.pair
        );

        let mut other = read();
        other.meta.native = self.native.into();
        other
            .glosses
            .retain(|(lemma, _)| self.keep.contains(&lemma.as_str()) || kept(lemma, 70));
        let new: Vec<(String, String)> = other
            .ranks
            .iter()
            .map(|(lemma, _)| lemma)
            .filter(|lemma| !glossed.contains(*lemma))
            .filter(|lemma| self.also.contains(&lemma.as_str()) || kept(&format!("{lemma}#"), 10))
            .map(|lemma| (lemma.clone(), format!("Gloss of {lemma}")))
            .collect();
        for lemma in self.also {
            assert!(
                new.iter().any(|(l, _)| l == lemma),
                "{lemma} is ranked and unglossed"
            );
        }
        let still: BTreeSet<String> = other.glosses.iter().map(|(l, _)| l.clone()).collect();
        other.senses.retain(|(lemma, runs)| {
            still.contains(lemma)
                && (self.keep.contains(&lemma.as_str()) || kept(&format!("{lemma}~"), 80))
                && runs
                    .iter()
                    .all(|(tag, _)| !self.dropped_tags.contains(&tag.as_str()))
        });
        // Glosses that say no gender: the builder gives a noun its readings' (D5). English's
        // nouns carry none; kept for a reference whose sense table genders its nouns.
        for (_, runs) in &mut other.senses {
            for (tag, _) in runs.iter_mut() {
                if tag.starts_with("NOUN|Gender=") {
                    *tag = "NOUN".into();
                }
            }
        }
        other.senses.extend(
            new.iter()
                .filter(|(lemma, _)| kept(&format!("{lemma}^"), 50))
                .map(|(lemma, _)| (lemma.clone(), vec![(self.new_tag.to_owned(), 1)])),
        );
        other.glosses.extend(new);
        other.expressions.retain(|(headword, _)| kept(headword, 70));
        other.lexical = Some(glossed.into_iter().collect());
        (reference, other)
    }
}

/// en-fr, and English glossed in Spanish.
pub const ENGLISH_IN_SPANISH: OtherNative = OtherNative {
    pair: "en-fr",
    native: "es",
    keep: &[],
    also: &[],
    new_tag: "NUM",
    dropped_tags: &[],
};
