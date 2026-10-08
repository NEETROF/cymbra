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

/// Spanish articles and other determiners: possessives, demonstratives, quantifiers.
/// Inflected forms too, which a pack may keep as their own lemmas (design D5).
const ES_DETERMINERS: &[&str] = &[
    "alguna",
    "algunas",
    "alguno",
    "algunos",
    "algún",
    "ambas",
    "ambos",
    "aquel",
    "aquella",
    "aquellas",
    "aquellos",
    "cada",
    "cualquier",
    "cualquiera",
    "demasiada",
    "demasiadas",
    "demasiado",
    "demasiados",
    "el",
    "esa",
    "esas",
    "ese",
    "esos",
    "esta",
    "estas",
    "este",
    "estos",
    "la",
    "las",
    "lo",
    "los",
    "mi",
    "mis",
    "misma",
    "mismas",
    "mismo",
    "mismos",
    "mucha",
    "muchas",
    "mucho",
    "muchos",
    "ninguna",
    "ningunas",
    "ninguno",
    "ningunos",
    "ningún",
    "nuestra",
    "nuestras",
    "nuestro",
    "nuestros",
    "otra",
    "otras",
    "otro",
    "otros",
    "poca",
    "pocas",
    "poco",
    "pocos",
    "su",
    "sus",
    "tanta",
    "tantas",
    "tanto",
    "tantos",
    "toda",
    "todas",
    "todo",
    "todos",
    "tu",
    "tus",
    "un",
    "una",
    "unas",
    "unos",
    "varias",
    "varios",
    "vuestra",
    "vuestras",
    "vuestro",
    "vuestros",
];

/// Spanish personal, clitic, prepositional, possessive, relative, interrogative and
/// indefinite pronouns.
const ES_PRONOUNS: &[&str] = &[
    "algo", "alguien", "aquello", "conmigo", "consigo", "contigo", "cual", "cuales", "cuya",
    "cuyas", "cuyo", "cuyos", "cuál", "cuáles", "cuánta", "cuántas", "cuánto", "cuántos", "ella",
    "ellas", "ello", "ellos", "eso", "esto", "la", "las", "le", "les", "lo", "los", "me", "mí",
    "mía", "mías", "mío", "míos", "nada", "nadie", "nos", "nosotras", "nosotros", "os", "que",
    "quien", "quienes", "quién", "quiénes", "qué", "se", "suya", "suyas", "suyo", "suyos", "sí",
    "te", "ti", "tuya", "tuyas", "tuyo", "tuyos", "tú", "usted", "ustedes", "vos", "vosotras",
    "vosotros", "yo", "él",
];

/// Spanish prepositions.
const ES_PREPOSITIONS: &[&str] = &[
    "a", "ante", "bajo", "con", "contra", "de", "desde", "durante", "en", "entre", "excepto",
    "hacia", "hasta", "mediante", "para", "por", "salvo", "según", "sin", "sobre", "tras",
];

/// Spanish coordinating and subordinating conjunctions.
const ES_CONJUNCTIONS: &[&str] = &[
    "aunque", "como", "cuando", "donde", "e", "mientras", "ni", "o", "pero", "porque", "pues",
    "que", "si", "sino", "u", "y",
];

/// Spanish auxiliaries and modals, as the forms tables lemmatise their forms.
const ES_AUXILIARIES_AND_MODALS: &[&str] = &["deber", "estar", "haber", "poder", "ser", "soler"];

/// Spanish negation. `nunca` is left out, as English's `never` is: an adverb with
/// content of its own.
const ES_NEGATION: &[&str] = &["no"];

/// The six Spanish tables (add-lingua-spanish-analysis D5).
const SPANISH: &[&[&str]] = &[
    ES_DETERMINERS,
    ES_PRONOUNS,
    ES_PREPOSITIONS,
    ES_CONJUNCTIONS,
    ES_AUXILIARIES_AND_MODALS,
    ES_NEGATION,
];

/// Whether a dictionary form is a closed-class word of the studied language:
/// an article or other determiner, a pronoun, a preposition or particle, a
/// conjunction, an auxiliary or modal, or a negation. `lemma` is what the
/// lemmatisation cascade produced (lowercase), never a surface form.
///
/// Each language is judged by its own tables only: `de` is not an English
/// function word, nor `the` a Spanish one. French has none while it is the
/// baseline (add-lingua-french-baseline D2): `le`, `de`, `ne`, `pas` count like
/// any other word.
pub fn is_function_word(lemma: &str, studied: StudiedLanguage) -> bool {
    let tables: &[&[&str]] = match studied {
        StudiedLanguage::English => ENGLISH,
        StudiedLanguage::Spanish => SPANISH,
        StudiedLanguage::French => &[],
    };
    tables
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
    fn each_language_is_judged_by_its_own_tables() {
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        for lemma in ["de", "la", "el", "y", "que", "haber", "no"] {
            assert!(
                is_function_word(lemma, ES),
                "{lemma:?} is a Spanish function word"
            );
            assert!(!is_function_word(lemma, EN) || lemma == "no", "{lemma:?}");
        }
        for lemma in ["the", "of", "casa", "padre", "nunca", "decir"] {
            assert!(
                !is_function_word(lemma, ES),
                "{lemma:?} is not a Spanish function word"
            );
        }
        all_function_words(&["the", "of"]);
    }

    #[test]
    fn spec_scenario_no_french_word_is_a_function_word_yet() {
        const FR: StudiedLanguage = StudiedLanguage::French;
        for lemma in [
            "pas", "ne", "le", "de", "la", "et", "que", "être", "avoir", "the",
        ] {
            assert!(!is_function_word(lemma, FR), "{lemma:?}");
        }
        // English and Spanish answer as before.
        assert!(is_function_word("the", EN));
        assert!(is_function_word("de", StudiedLanguage::Spanish));
    }

    #[test]
    fn spec_scenario_a_spanish_phrase() {
        // "la casa de mi padre", lemmatised: the closed classes are flagged, the nouns are not.
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        let flagged: Vec<bool> = ["la", "casa", "de", "mi", "padre"]
            .iter()
            .map(|lemma| is_function_word(lemma, ES))
            .collect();
        assert_eq!(flagged, [true, false, true, true, false]);
    }

    #[test]
    fn spanish_classes_hold_what_they_name() {
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        for (table, sample) in [
            (ES_DETERMINERS, &["el", "una", "mis", "esta", "todos"][..]),
            (ES_PRONOUNS, &["yo", "nos", "les", "quién", "nadie"][..]),
            (ES_PREPOSITIONS, &["a", "de", "según", "tras"][..]),
            (ES_CONJUNCTIONS, &["y", "pero", "aunque", "u"][..]),
            (
                ES_AUXILIARIES_AND_MODALS,
                &["haber", "ser", "estar", "poder"][..],
            ),
            (ES_NEGATION, &["no"][..]),
        ] {
            for word in sample {
                assert!(table.binary_search(word).is_ok(), "{word:?}");
                assert!(is_function_word(word, ES), "{word:?}");
            }
        }
    }

    #[test]
    fn every_table_is_sorted_and_free_of_duplicates() {
        for table in ENGLISH.iter().chain(SPANISH) {
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
