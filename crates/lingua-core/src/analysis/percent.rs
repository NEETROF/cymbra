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

//! Known-token percentage over caller-supplied classifications.
//!
//! Counts **occurrences** (tokens), never distinct words — the honest-count
//! rule this product exists for. The statuses that produce these
//! classifications arrive with `add-lingua-knowledge-model`; this change
//! fixes the arithmetic contract: ignored counts as known, learning counts
//! as not known, out-of-lexicon proper nouns leave the count entirely.

use serde::{Deserialize, Serialize};

use super::lexicon::Lexicon;

/// How one token occurrence counts toward the page percentage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenClass {
    /// The lemma is known (explicitly, or implicitly via calibration).
    Known,
    /// The lemma is ignored by the user — counts as known.
    Ignored,
    /// The lemma is being learned — counts as NOT known.
    Learning,
    /// The lemma is unknown.
    Unknown,
    /// Out-of-lexicon proper noun — excluded from the count.
    ProperNounOutOfLexicon,
}

/// Token tallies for one analysed text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Coverage {
    /// Occurrences that entered the count (proper nouns excluded).
    pub counted: u32,
    /// Occurrences counting as known (known + ignored).
    pub known: u32,
}

impl Coverage {
    /// Folds one classification into the tallies.
    pub fn add(&mut self, class: TokenClass) {
        match class {
            TokenClass::Known | TokenClass::Ignored => {
                self.counted += 1;
                self.known += 1;
            }
            TokenClass::Learning | TokenClass::Unknown => self.counted += 1,
            TokenClass::ProperNounOutOfLexicon => {}
        }
    }

    /// Known percentage rounded to the nearest integer; `None` when nothing
    /// was counted (an empty or fully-excluded text has no honest
    /// percentage).
    pub fn percent_rounded(&self) -> Option<u8> {
        if self.counted == 0 {
            return None;
        }
        Some(((self.known * 200 + self.counted) / (self.counted * 2)) as u8)
    }
}

/// Tallies a stream of classifications.
pub fn coverage(classes: impl IntoIterator<Item = TokenClass>) -> Coverage {
    let mut cov = Coverage::default();
    for class in classes {
        cov.add(class);
    }
    cov
}

/// The shared proper-noun heuristic: sentence-cased surface whose lowercase
/// form resolves nowhere in the lexicon. Callers MUST use this single rule so
/// every surface excludes the same tokens.
pub fn is_out_of_lexicon_proper_noun(
    surface: &str,
    lemma: &str,
    lexicon: &(impl Lexicon + ?Sized),
) -> bool {
    surface.chars().next().is_some_and(char::is_uppercase)
        && !lexicon.contains(&surface.to_lowercase())
        && !lexicon.contains_lemma(lemma)
}

/// The proper-noun heuristic for a hyphenated compound the lexicon does not
/// know as a unit: sentence-cased *and* no part is a known lexicon word — a
/// genuine name like `Jean-Pierre`, not an ordinary compound like `Read-only`
/// whose parts are known.
///
/// The whole compound is by construction absent from the lexicon (that is why
/// it has parts), so [`is_out_of_lexicon_proper_noun`] applied to the whole
/// would exclude *every* sentence-initial compound. This mirrors the rule at
/// the part level instead, so a capitalized compound is excluded exactly when
/// its separate parts would each have been — and counting no longer depends on
/// whether the compound happens to start a sentence.
pub fn compound_is_out_of_lexicon_proper_noun(
    surface: &str,
    part_lemmas: &[String],
    lexicon: &(impl Lexicon + ?Sized),
) -> bool {
    surface.chars().next().is_some_and(char::is_uppercase)
        && part_lemmas.iter().all(|p| !lexicon.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lexicon::{FstLexicon, build_lexicon_blobs};

    #[test]
    fn spec_scenario_repeated_unknown_word() {
        // 10 counted tokens, 2 occurrences of the same unknown lemma, 8 known.
        let classes = [TokenClass::Known; 8]
            .into_iter()
            .chain([TokenClass::Unknown, TokenClass::Unknown]);
        assert_eq!(coverage(classes).percent_rounded(), Some(80));
    }

    #[test]
    fn ignored_counts_known_learning_counts_unknown_proper_nouns_leave() {
        let cov = coverage([
            TokenClass::Known,
            TokenClass::Ignored,
            TokenClass::Learning,
            TokenClass::Unknown,
            TokenClass::ProperNounOutOfLexicon,
        ]);
        assert_eq!(
            cov,
            Coverage {
                counted: 4,
                known: 2
            }
        );
        assert_eq!(cov.percent_rounded(), Some(50));
    }

    #[test]
    fn empty_count_has_no_percentage() {
        assert_eq!(coverage([]).percent_rounded(), None);
        assert_eq!(
            coverage([TokenClass::ProperNounOutOfLexicon]).percent_rounded(),
            None
        );
    }

    #[test]
    fn rounding_is_nearest_not_truncation() {
        // 2/3 known = 66.67% → 67.
        let cov = coverage([TokenClass::Known, TokenClass::Known, TokenClass::Unknown]);
        assert_eq!(cov.percent_rounded(), Some(67));
    }

    #[test]
    fn proper_noun_heuristic_needs_case_and_absence() {
        let (bytes, pool) = build_lexicon_blobs(&[("teams", "team")], &["france"]).expect("build");
        let lex = FstLexicon::from_slices(bytes, &pool).expect("load");
        // Cased but the lowercase form is a known inflection → not excluded.
        assert!(!is_out_of_lexicon_proper_noun("Teams", "team", &lex));
        // Cased and lowercase is a lexicon lemma → not excluded.
        assert!(!is_out_of_lexicon_proper_noun("France", "france", &lex));
        // Cased and resolvable nowhere → excluded.
        assert!(is_out_of_lexicon_proper_noun("Cymbra", "cymbra", &lex));
        // Lowercase never triggers the heuristic.
        assert!(!is_out_of_lexicon_proper_noun("cymbra", "cymbra", &lex));
    }

    #[test]
    fn compound_proper_noun_needs_case_and_no_known_part() {
        let (bytes, pool) =
            build_lexicon_blobs(&[("teams", "team")], &["france", "read", "only"]).expect("build");
        let lex = FstLexicon::from_slices(bytes, &pool).expect("load");
        let parts = |ws: &[&str]| ws.iter().map(|w| (*w).to_owned()).collect::<Vec<_>>();
        // Cased, but a part is a known word → an ordinary compound, not a name.
        assert!(!compound_is_out_of_lexicon_proper_noun(
            "Read-only",
            &parts(&["read", "only"]),
            &lex
        ));
        // Cased and no part is known → a hyphenated name, excluded.
        assert!(compound_is_out_of_lexicon_proper_noun(
            "Jean-Pierre",
            &parts(&["jean", "pierre"]),
            &lex
        ));
        // Lowercase never triggers it, even when no part is known.
        assert!(!compound_is_out_of_lexicon_proper_noun(
            "jean-pierre",
            &parts(&["jean", "pierre"]),
            &lex
        ));
    }
}
