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

//! The closed-class words of a studied language (change `add-lingua-phrase-gloss`,
//! design D3).
//!
//! A word-by-word gloss of a selection is only worth showing for the words that
//! carry meaning on their own: glossing `of` or `have` inside a phrase tells the
//! reader nothing. The tables below hold the English closed classes as
//! **dictionary forms** — the check runs after lemmatisation, so `is` reaches it
//! as `be` and the `n't` of a contraction as `not` — in the six classes the spec
//! names. They are a judgement, pinned by one test per class; a frequency cut
//! would not do, since the hundred commonest lemmas hold `time`, `people`,
//! `say` and `know`, exactly the words worth a row.
//!
//! Every table MUST stay sorted: lookups binary-search them directly (a test
//! enforces the ordering). Adding a studied language means adding its tables,
//! like its tokeniser pre-pass.

use super::language::StudiedLanguage;

/// Articles and the other determiners: possessives, demonstratives and the
/// quantifiers that stand before a noun (`some`, `each`, `every`…).
const DETERMINERS: &[&str] = &[
    "a", "all", "an", "another", "any", "both", "each", "either", "every", "few", "his", "its",
    "many", "much", "my", "neither", "other", "our", "several", "some", "such", "that", "the",
    "their", "these", "this", "those", "your",
];

/// Personal, reflexive, possessive, relative, interrogative and indefinite
/// pronouns. `i` is the lowercase lemma the cascade gives `I`.
const PRONOUNS: &[&str] = &[
    "anybody",
    "anyone",
    "anything",
    "everybody",
    "everyone",
    "everything",
    "he",
    "her",
    "hers",
    "herself",
    "him",
    "himself",
    "i",
    "it",
    "itself",
    "me",
    "mine",
    "myself",
    "nobody",
    "none",
    "nothing",
    "oneself",
    "ours",
    "ourselves",
    "she",
    "somebody",
    "someone",
    "something",
    "theirs",
    "them",
    "themselves",
    "they",
    "us",
    "we",
    "what",
    "whatever",
    "which",
    "whichever",
    "who",
    "whoever",
    "whom",
    "whose",
    "you",
    "yours",
    "yourself",
    "yourselves",
];

/// Prepositions, and the particles of phrasal verbs (`give up`, `set off`,
/// `run away`), which are the same short words.
const PREPOSITIONS_AND_PARTICLES: &[&str] = &[
    "about",
    "above",
    "across",
    "after",
    "against",
    "along",
    "amid",
    "among",
    "around",
    "at",
    "away",
    "before",
    "behind",
    "below",
    "beneath",
    "beside",
    "besides",
    "between",
    "beyond",
    "by",
    "despite",
    "down",
    "during",
    "except",
    "for",
    "from",
    "in",
    "inside",
    "into",
    "near",
    "of",
    "off",
    "on",
    "onto",
    "out",
    "outside",
    "over",
    "per",
    "since",
    "through",
    "throughout",
    "till",
    "to",
    "toward",
    "towards",
    "under",
    "underneath",
    "until",
    "unto",
    "up",
    "upon",
    "via",
    "with",
    "within",
    "without",
];

/// Coordinating and subordinating conjunctions.
const CONJUNCTIONS: &[&str] = &[
    "although", "and", "as", "because", "but", "if", "nor", "or", "so", "than", "though", "unless",
    "when", "whenever", "where", "whereas", "wherever", "whether", "while", "yet",
];

/// Auxiliaries and modals. `be`, `have` and `do` are the lemmas the irregulars
/// table gives `is`, `has`, `did`, `'s`, `'ve`…; `will` and `would` catch `'ll`
/// and `'d`.
const AUXILIARIES_AND_MODALS: &[&str] = &[
    "be", "can", "could", "do", "have", "may", "might", "must", "ought", "shall", "should", "will",
    "would",
];

/// Negation. `not` is what the tokeniser's pre-pass makes of every `n't`;
/// `cannot` is the one negated modal the tokeniser cannot split. `never` is
/// left out on purpose: it is an adverb with content of its own.
const NEGATION: &[&str] = &["cannot", "no", "not"];

/// The six English tables, checked in turn by [`is_function_word`].
const ENGLISH: &[&[&str]] = &[
    DETERMINERS,
    PRONOUNS,
    PREPOSITIONS_AND_PARTICLES,
    CONJUNCTIONS,
    AUXILIARIES_AND_MODALS,
    NEGATION,
];

/// Whether a dictionary form is a closed-class word of the studied language:
/// an article or other determiner, a pronoun, a preposition or particle, a
/// conjunction, an auxiliary or modal, or a negation. `lemma` is what the
/// lemmatisation cascade produced (lowercase), never a surface form.
pub fn is_function_word(lemma: &str, studied: StudiedLanguage) -> bool {
    let StudiedLanguage::English = studied;
    ENGLISH
        .iter()
        .any(|table| table.binary_search(&lemma).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: StudiedLanguage = StudiedLanguage::English;

    fn all_function_words(lemmas: &[&str]) {
        for lemma in lemmas {
            assert!(is_function_word(lemma, EN), "{lemma:?} is a function word");
        }
    }

    #[test]
    fn every_table_is_sorted_and_free_of_duplicates() {
        for table in ENGLISH {
            for pair in table.windows(2) {
                assert!(
                    pair[0] < pair[1],
                    "{:?} must sort before {:?}",
                    pair[0],
                    pair[1]
                );
            }
        }
        // A lemma belongs to one class: listing it twice would hide a table
        // that went stale.
        let mut all: Vec<&str> = ENGLISH.iter().flat_map(|t| t.iter().copied()).collect();
        let total = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), total, "a lemma is listed in two classes");
    }

    #[test]
    fn articles_and_other_determiners_are_function_words() {
        // Articles, a possessive, a demonstrative and quantifier-determiners.
        all_function_words(&[
            "a", "an", "the", "my", "their", "this", "those", "some", "every",
        ]);
    }

    #[test]
    fn pronouns_are_function_words() {
        // Personal, reflexive, relative, interrogative, indefinite.
        all_function_words(&[
            "i",
            "it",
            "them",
            "myself",
            "who",
            "which",
            "what",
            "something",
        ]);
    }

    #[test]
    fn prepositions_and_particles_are_function_words() {
        all_function_words(&[
            "in", "of", "with", "up", "off", "out", "away", "down", "over",
        ]);
    }

    #[test]
    fn conjunctions_are_function_words() {
        all_function_words(&["and", "but", "or", "because", "although", "if", "while"]);
    }

    #[test]
    fn auxiliaries_and_modals_are_function_words() {
        // The dictionary forms `is`/`has`/`did` reach after lemmatisation.
        all_function_words(&["be", "have", "do", "will", "would", "can", "could", "must"]);
    }

    #[test]
    fn negation_is_a_function_word_but_never_is_not() {
        all_function_words(&["not", "no"]);
        assert!(!is_function_word("never", EN), "an adverb with content");
    }

    #[test]
    fn high_frequency_content_words_are_not_function_words() {
        // The commonest lemmas of English hold exactly the words worth a row.
        for lemma in [
            "time", "say", "people", "person", "know", "spite", "city", "give",
        ] {
            assert!(!is_function_word(lemma, EN), "{lemma:?} carries meaning");
        }
    }
}
