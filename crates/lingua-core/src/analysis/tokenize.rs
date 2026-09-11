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
    /// Surface text after the pre-pass (e.g. `do`, `not`, `Teams`). For a
    /// hyphenated compound it is the whole run, hyphens included (`repo-wide`).
    pub text: String,
    /// Byte offset of the source word in the analysed text.
    pub start: usize,
    /// Byte end (exclusive) of the source word in the analysed text.
    pub end: usize,
    /// The surfaces of a hyphenated compound's pieces (`["repo", "wide"]`),
    /// empty for an ordinary single-word token. A compound the lexicon does not
    /// know as a unit is classified from these — the reader understands it only
    /// as well as its weakest part.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<String>,
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
    let words: Vec<(usize, &str)> = text.unicode_word_indices().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let (start, first) = words[i];
        let mut end = start + first.len();
        // Absorb pieces joined to this one by a single hyphen: `repo-wide`,
        // `state-of-the-art`. Exactly one `-` is the hyphenation mark — a space
        // ends the run, and so does a `--`/`---` run (the ASCII em-dash / range,
        // not a compound).
        let mut j = i + 1;
        while j < words.len() {
            let (next_start, next) = words[j];
            if &text[end..next_start] != "-" {
                break;
            }
            end = next_start + next.len();
            j += 1;
        }
        if j - i >= 2 {
            push_compound(
                &mut tokens,
                &text[start..end],
                &words[i..j],
                start,
                end,
                lexicon,
            );
        } else {
            push_word(&mut tokens, first, start, end, lexicon);
        }
        i = j;
    }
    tokens
}

/// Emits an ordinary single-word token, applying the English pre-pass:
/// apostrophe normalisation/trimming, `n't` expansion, the digit drop and the
/// single-letter-needs-the-lexicon rule.
fn push_word(
    tokens: &mut Vec<Token>,
    word: &str,
    start: usize,
    end: usize,
    lexicon: &(impl Lexicon + ?Sized),
) {
    let normalized = word.replace('\u{2019}', "'");
    let trimmed = normalized.trim_matches('\'');
    if trimmed.is_empty() || trimmed.chars().any(|c| c.is_ascii_digit()) {
        return;
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
            parts: Vec::new(),
        });
        tokens.push(Token {
            text: second.to_owned(),
            start,
            end,
            parts: Vec::new(),
        });
        return;
    }
    if single_letter_outside_lexicon(trimmed, &lower, lexicon) {
        return;
    }
    tokens.push(Token {
        text: trimmed.to_owned(),
        start,
        end,
        parts: Vec::new(),
    });
}

/// Emits one token spanning a whole hyphenated compound, recording its pieces'
/// surfaces. Contractions and the single-letter rule do not apply inside a
/// compound — it stands or falls as a unit (so `x-ray`, `e-mail` survive).
///
/// A digit anywhere in the run means an identifier/quantity, not a compound
/// word (`utf-8`, `well-being-2`). Rather than drop the whole run — which would
/// swallow clean neighbours like `well`/`being` — it degrades to per-piece
/// tokenisation, identical to no fusion: the clean pieces survive, the
/// digit-bearing ones are dropped by [`push_word`]'s own digit rule.
fn push_compound(
    tokens: &mut Vec<Token>,
    whole: &str,
    pieces: &[(usize, &str)],
    start: usize,
    end: usize,
    lexicon: &(impl Lexicon + ?Sized),
) {
    if whole.chars().any(|c| c.is_ascii_digit()) {
        for (p_start, word) in pieces {
            push_word(tokens, word, *p_start, p_start + word.len(), lexicon);
        }
        return;
    }
    tokens.push(Token {
        text: whole.to_owned(),
        start,
        end,
        parts: pieces.iter().map(|(_, w)| (*w).to_owned()).collect(),
    });
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

    #[test]
    fn hyphenated_compound_is_one_token_spanning_the_whole_run() {
        let text = "The read-only flag is set.";
        let tokens = tokenize(text, StudiedLanguage::English, &lexicon());
        assert_eq!(texts(&tokens), ["The", "read-only", "flag", "is", "set"]);
        let compound = &tokens[1];
        assert_eq!(compound.text, "read-only");
        assert_eq!(compound.parts, ["read", "only"]);
        // The span covers the whole compound, hyphen included.
        assert_eq!(&text[compound.start..compound.end], "read-only");
    }

    #[test]
    fn multi_hyphen_compound_absorbs_every_piece() {
        let tokens = tokenize(
            "A state-of-the-art design.",
            StudiedLanguage::English,
            &lexicon(),
        );
        let compound = tokens
            .iter()
            .find(|t| t.text.contains('-'))
            .expect("compound present");
        assert_eq!(compound.text, "state-of-the-art");
        assert_eq!(compound.parts, ["state", "of", "the", "art"]);
    }

    #[test]
    fn a_hyphen_between_spaces_is_not_a_compound() {
        // "code - team" is two words and a stray dash, not "code-team".
        let tokens = tokenize("ship code - team", StudiedLanguage::English, &lexicon());
        assert_eq!(texts(&tokens), ["ship", "code", "team"]);
        assert!(tokens.iter().all(|t| t.parts.is_empty()));
    }

    #[test]
    fn a_double_hyphen_is_an_em_dash_not_a_compound() {
        // "wait--what" / "cost--benefit" are em-dashes: two words, not one.
        let tokens = tokenize(
            "ship code--team and code---ship now",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(
            texts(&tokens),
            ["ship", "code", "team", "and", "code", "ship", "now"]
        );
        assert!(tokens.iter().all(|t| t.parts.is_empty()));
    }

    #[test]
    fn a_digit_bearing_run_degrades_to_its_clean_pieces() {
        // A digit means an identifier: keep the clean pieces (as if unfused),
        // drop only the digit-bearing one — never swallow `well`/`being`.
        let tokens = tokenize(
            "ship well-being-2 and utf-8 code",
            StudiedLanguage::English,
            &lexicon(),
        );
        assert_eq!(
            texts(&tokens),
            ["ship", "well", "being", "and", "utf", "code"]
        );
        // The salvaged pieces are ordinary single-word tokens, not compounds.
        assert!(tokens.iter().all(|t| t.parts.is_empty()));
    }

    #[test]
    fn a_single_letter_piece_survives_inside_a_compound() {
        // "x" alone is dropped, but "x-ray" is a word — the compound stands as
        // a unit rather than being pruned piece by piece.
        let tokens = tokenize("an x-ray scan", StudiedLanguage::English, &lexicon());
        let compound = tokens
            .iter()
            .find(|t| t.text.contains('-'))
            .expect("compound present");
        assert_eq!(compound.text, "x-ray");
        assert_eq!(compound.parts, ["x", "ray"]);
    }
}
