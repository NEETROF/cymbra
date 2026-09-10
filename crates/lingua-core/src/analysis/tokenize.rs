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

//! UAX #29 tokenisation with a per-studied-language pre-pass (design D2).
//!
//! `unicode-segmentation` finds word candidates; the English pre-pass then
//! absorbs surface quirks: `n't` contractions expand to their two words
//! (`don't` → `do` + `not`), edge apostrophes are stripped, and
//! single-letter tokens only survive when the lexicon knows them ("I", "a").
//! Adding a studied language later means adding a pre-pass, not touching the
//! tokeniser.

use serde::Serialize;
use unicode_segmentation::UnicodeSegmentation;

use super::language::StudiedLanguage;
use super::lexicon::Lexicon;

/// A countable token: the surface text (case preserved — the proper-noun
/// heuristic needs it) plus the byte span of the source word it came from.
/// The two halves of an expanded contraction share the same span.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Token {
    /// Surface text after the pre-pass (e.g. `do`, `not`, `Teams`).
    pub text: String,
    /// Byte offset of the source word in the analysed text.
    pub start: usize,
    /// Byte end (exclusive) of the source word in the analysed text.
    pub end: usize,
}

/// Contractions whose base changes when `n't` is peeled off. Everything else
/// follows the regular rule `Xn't` → `X` + `not`.
const IRREGULAR_CONTRACTIONS: &[(&str, &str)] = &[
    ("won't", "will"),
    ("can't", "can"),
    ("shan't", "shall"),
    ("ain't", "be"),
];

/// Tokenises `text` for the studied language.
///
/// Tokens containing a digit are dropped (identifiers, quantities, "3D"):
/// they are not vocabulary. Apostrophe variants (`’`) are normalised to `'`
/// before the pre-pass so typographic text behaves like plain text.
pub fn tokenize(
    text: &str,
    language: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> Vec<Token> {
    let StudiedLanguage::English = language;
    let mut tokens = Vec::new();
    for (start, word) in text.unicode_word_indices() {
        let end = start + word.len();
        let normalized = word.replace('\u{2019}', "'");
        let trimmed = normalized.trim_matches('\'');
        if trimmed.is_empty() || trimmed.chars().any(|c| c.is_ascii_digit()) {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if let Some((base, second)) = split_contraction(&lower) {
            // Preserve the original casing on the base's first letter so the
            // proper-noun heuristic still sees "Don't" as sentence-cased.
            let base_cased = match trimmed.chars().next() {
                Some(first) if first.is_uppercase() => {
                    let mut s = String::new();
                    s.extend(first.to_uppercase());
                    s.push_str(&base[first.len_utf8().min(base.len())..]);
                    s
                }
                _ => base.to_owned(),
            };
            tokens.push(Token {
                text: base_cased,
                start,
                end,
            });
            tokens.push(Token {
                text: second.to_owned(),
                start,
                end,
            });
            continue;
        }
        if single_letter_outside_lexicon(trimmed, &lower, lexicon) {
            continue;
        }
        tokens.push(Token {
            text: trimmed.to_owned(),
            start,
            end,
        });
    }
    tokens
}

/// `don't` → (`do`, `not`) — irregular table first, then the regular rule.
fn split_contraction(lower: &str) -> Option<(&str, &'static str)> {
    for (contraction, base) in IRREGULAR_CONTRACTIONS {
        if lower == *contraction {
            return Some((base, "not"));
        }
    }
    let base = lower.strip_suffix("n't")?;
    if base.is_empty() {
        None
    } else {
        Some((base, "not"))
    }
}

fn single_letter_outside_lexicon(
    trimmed: &str,
    lower: &str,
    lexicon: &(impl Lexicon + ?Sized),
) -> bool {
    trimmed.chars().count() == 1 && !lexicon.contains(lower)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lexicon::{FstLexicon, build_lexicon_blobs};

    fn lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) =
            build_lexicon_blobs(&[], &["a", "i", "do", "not", "ship", "code", "team"])
                .expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    fn texts(tokens: &[Token]) -> Vec<&str> {
        tokens.iter().map(|t| t.text.as_str()).collect()
    }

    #[test]
    fn spec_scenario_everyday_english() {
        let tokens = tokenize(
            "Teams don't ship code.",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(texts(&tokens), ["Teams", "do", "not", "ship", "code"]);
    }

    #[test]
    fn contraction_halves_share_the_source_span() {
        let text = "Teams don't ship.";
        let tokens = tokenize(text, StudiedLanguage::English, &lexicon());
        let dont_start = text.find("don't").expect("present");
        assert_eq!(tokens[1].text, "do");
        assert_eq!(tokens[2].text, "not");
        assert_eq!(
            (tokens[1].start, tokens[1].end),
            (dont_start, dont_start + "don't".len())
        );
        assert_eq!(
            (tokens[2].start, tokens[2].end),
            (tokens[1].start, tokens[1].end)
        );
    }

    #[test]
    fn irregular_contractions_change_their_base() {
        let tokens = tokenize(
            "They won't and can't.",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(texts(&tokens), ["They", "will", "not", "and", "can", "not"]);
    }

    #[test]
    fn cased_contraction_keeps_sentence_case() {
        let tokens = tokenize("Don't stop.", StudiedLanguage::English, &lexicon());
        assert_eq!(texts(&tokens), ["Do", "not", "stop"]);
    }

    #[test]
    fn typographic_apostrophes_behave_like_plain_ones() {
        let tokens = tokenize(
            "Teams don\u{2019}t ship.",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(texts(&tokens), ["Teams", "do", "not", "ship"]);
    }

    #[test]
    fn edge_apostrophes_are_stripped_but_internal_ones_kept() {
        let tokens = tokenize(
            "'tis the sailors' o'clock",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(texts(&tokens), ["tis", "the", "sailors", "o'clock"]);
    }

    #[test]
    fn single_letters_need_the_lexicon() {
        // "I" and "a" are in the lexicon; a stray "x" is not.
        let tokens = tokenize("I read a book x", StudiedLanguage::English, &lexicon());
        assert_eq!(texts(&tokens), ["I", "read", "a", "book"]);
    }

    #[test]
    fn digit_bearing_tokens_are_dropped() {
        let tokens = tokenize("2026 saw 3D movies", StudiedLanguage::English, &lexicon());
        assert_eq!(texts(&tokens), ["saw", "movies"]);
    }
}
