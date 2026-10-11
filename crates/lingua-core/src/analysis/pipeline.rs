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

//! Document-level orchestration: language gating → tokenisation →
//! lemmatisation, block by block.
//!
//! Outputs are plain ordered vectors — serialisable, order-stable, and
//! therefore byte-for-byte comparable across runs and compilation targets
//! (the determinism contract, design D5).

use serde::Serialize;

use super::french;
use super::language::{MIN_ANALYSABLE_TOKENS, StudiedLanguage, block_is_studied};
use super::lemmatize::lemmatize;
use super::lexicon::Lexicon;
use super::tokenize::{Token, tokenize};

/// One analysed token occurrence, in document order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysedToken {
    /// Index of the source block in the submitted document.
    pub block: usize,
    /// Surface text after the tokeniser's pre-pass (case preserved).
    pub surface: String,
    /// The lemma the cascade produced (lowercase).
    pub lemma: String,
    /// Part lemmas of a hyphenated compound the lexicon does not know as a
    /// unit, empty otherwise. When present, the classifier judges the token by
    /// its weakest part rather than by `lemma` alone.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<String>,
    /// Byte span of the source word inside its block.
    pub start: usize,
    /// Byte end (exclusive) of the source word inside its block.
    pub end: usize,
}

/// Result of analysing a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DocumentAnalysis {
    /// Too little studied-language content to score honestly.
    NotAnalysable,
    /// The studied-language tokens, in document order.
    Analysed(Vec<AnalysedToken>),
}

/// Analyses a document supplied as blocks of text (paragraphs, DOM blocks…).
///
/// Blocks outside the studied language are excluded entirely; when the
/// remaining studied content yields fewer than
/// [`MIN_ANALYSABLE_TOKENS`](super::language::MIN_ANALYSABLE_TOKENS) tokens,
/// the document is [`DocumentAnalysis::NotAnalysable`].
pub fn analyse_document(
    blocks: &[&str],
    studied: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> DocumentAnalysis {
    let mut out = Vec::new();
    for (block_idx, block) in blocks.iter().enumerate() {
        if !block_is_studied(block, studied) {
            continue;
        }
        let tokens = tokenize(block, studied, lexicon);
        // Which tokens stand alone in capitals, read in the block (keep-lingua-french-acronyms-in-capitals D1).
        let alone = alone_in_capitals(block, &tokens, studied);
        for (token, alone) in tokens.into_iter().zip(alone) {
            let (lemma, parts) = resolve_lemmas(&token, alone, studied, lexicon);
            out.push(AnalysedToken {
                block: block_idx,
                surface: token.text,
                lemma,
                parts,
                start: token.start,
                end: token.end,
            });
        }
    }
    if out.len() < MIN_ANALYSABLE_TOKENS {
        return DocumentAnalysis::NotAnalysable;
    }
    DocumentAnalysis::Analysed(out)
}

/// Resolves a token's lemma, and — for a hyphenated compound the lexicon does
/// not know as a unit — its part lemmas.
///
/// A compound the lexicon recognises (`e-mail`, `x-ray` — every lemma carries
/// an identity form, so `lemma_of` catches ranked/glossed compounds too) is a
/// single lexical unit: its own lemma, no parts. An unknown compound
/// (`repo-wide`, `type-safe`) keeps the lowercased surface as its lemma — the
/// single-word suffix cascade has no business stemming it — and carries its
/// parts so the classifier can judge it by its weakest one.
///
/// A French word standing alone in capitals (`alone`, [`alone_in_capitals`]'s answer
/// for the token) is first looked up as written, and read as the acronym the pack
/// lists, its capitals kept, the cascade not run (`french::acronym`,
/// keep-lingua-french-acronyms-in-capitals D1, step 3); anything else goes to the
/// cascade as before.
///
/// Crate-visible so the phrase gloss (`engine::gloss_phrase`) resolves a
/// selection's tokens exactly as a page's, without the gates above.
pub(crate) fn resolve_lemmas(
    token: &Token,
    alone: bool,
    studied: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> (String, Vec<String>) {
    if alone
        && studied == StudiedLanguage::French
        && token.parts.is_empty()
        && let Some(acronym) = french::acronym(&token.text, lexicon)
    {
        return (acronym, Vec::new());
    }
    if token.parts.is_empty() {
        return (lemmatize(&token.text, studied, lexicon), Vec::new());
    }
    let whole = token.text.replace('\u{2019}', "'").to_lowercase();
    if let Some(lemma) = lexicon.lemma_of(&whole) {
        return (lemma.to_owned(), Vec::new());
    }
    let parts = token
        .parts
        .iter()
        .map(|piece| lemmatize(piece, studied, lexicon))
        .collect();
    (whole, parts)
}

/// What the analysis reads in an expression's headword: its tokens, through the
/// studied language's own pre-pass, each paired with its dictionary form resolved
/// as a page's token is ([`resolve_lemmas`]) — the reading [`analyse_document`]
/// and the phrase gloss make of the same words on a page.
///
/// `None` when a word of the headword, split at its spaces, starts no token: a
/// single letter the lexicon does not list (`compte en t`), a word holding a
/// digit. The reading would then be shorter than the headword and match words
/// it does not hold (add-lingua-french-expression-keys D6).
///
/// The pack builder keys a French or Spanish expression on it, through
/// [`reading_expression_key`](crate::engine::reading_expression_key), and so does
/// review, finding such an expression card by its name (D1, D3;
/// add-lingua-spanish-expression-keys D1, D7).
pub fn headword_reading(
    headword: &str,
    studied: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> Option<Vec<(Token, String)>> {
    let tokens = tokenize(headword, studied, lexicon);
    let alone = alone_in_capitals(headword, &tokens, studied);
    let mut start = 0;
    for word in headword.split(' ') {
        let end = start + word.len();
        if !word.is_empty() && !tokens.iter().any(|t| (start..end).contains(&t.start)) {
            return None;
        }
        start = end + 1;
    }
    Some(
        tokens
            .into_iter()
            .zip(alone)
            .map(|(token, alone)| {
                let (lemma, _) = resolve_lemmas(&token, alone, studied, lexicon);
                (token, lemma)
            })
            .collect(),
    )
}

/// How a token's source is written, for [`alone_in_capitals`]: the letters of its
/// span, read without its soft hyphens, apostrophes and combining marks — none of
/// which is a letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Written {
    /// Fewer than two letters: a single letter (`A`, `L'`, `W`), passed over.
    Short,
    /// Two letters or more, a lowercase one among them.
    Lowercase,
    /// Two letters or more, none of them lowercase.
    Capitals,
}

/// How `token`'s source in `text` is written. It stops at the first lowercase
/// letter once two letters are read — most words at their second letter.
fn written(text: &str, token: &Token) -> Written {
    let source = text.get(token.start..token.end).unwrap_or(&token.text);
    let mut letters = 0usize;
    let mut lowercase = false;
    for c in source.chars().filter(|c| c.is_alphabetic()) {
        letters += 1;
        lowercase |= c.is_lowercase();
        if lowercase && letters >= 2 {
            return Written::Lowercase;
        }
    }
    match (letters >= 2, lowercase) {
        (false, _) => Written::Short,
        (true, true) => Written::Lowercase,
        (true, false) => Written::Capitals,
    }
}

/// For each of `tokens`, read from `text`, whether it is a French word standing
/// alone in capitals (keep-lingua-french-acronyms-in-capitals D1, steps 1 and 2),
/// which [`resolve_lemmas`] then looks up as written, as an acronym of the pack:
///
/// 1. **written wholly in capitals** — one word, no hyphenated run (whose parts are
///    read as before), whose source span holds two letters or more and no lowercase
///    letter, read without its soft hyphens (`CA`, `ÉTÉ`, the `CA` of « CA-14 »; not
///    `Ca`, `A`, `L'`, « LS-CA »);
/// 2. **standing alone in capitals** — the nearest token before it and the nearest
///    after it whose source holds two letters or more and is not its own span (the
///    two pieces of « au » share one) are each absent or not written wholly in
///    capitals. Single letters, numbers and punctuation are passed over: the
///    tokeniser drops the last two, and a single letter is too short to count.
///
/// The owner's « a line not itself in capitals » is read at the word's neighbours,
/// not at the whole of `text`: a page's block, a selection, a card's written word and
/// an expression's headword have no lines of their own, and read at the block the
/// rule would take a shouted « ça » inside a line of lowercase words (« LES RÉSEAUX
/// SOCIAUX CA RENDS… ») for the acronym; at its neighbours it does not, and a selection
/// reads a word as its page does. The four callers — `analyse_document`,
/// [`headword_reading`], `engine::gloss_phrase` and `engine::word_grammar` — read
/// their own text through it, so the four read a word alike.
///
/// Every answer is no for a text read as English or Spanish, without reading it:
/// their analyses do not move.
pub(crate) fn alone_in_capitals(
    text: &str,
    tokens: &[Token],
    studied: StudiedLanguage,
) -> Vec<bool> {
    if studied != StudiedLanguage::French {
        return vec![false; tokens.len()];
    }
    let shapes: Vec<Written> = tokens.iter().map(|token| written(text, token)).collect();
    (0..tokens.len())
        .map(|i| {
            shapes[i] == Written::Capitals
                && tokens[i].parts.is_empty()
                && !neighbour_in_capitals(i, (0..i).rev(), tokens, &shapes)
                && !neighbour_in_capitals(i, i + 1..tokens.len(), tokens, &shapes)
        })
        .collect()
}

/// Whether token `i`'s nearest neighbour among `others` (indexes, nearest first) is
/// written in capitals: the first whose source holds two letters or more and is not
/// `i`'s own span. No neighbour is none.
fn neighbour_in_capitals(
    i: usize,
    mut others: impl Iterator<Item = usize>,
    tokens: &[Token],
    shapes: &[Written],
) -> bool {
    let span = (tokens[i].start, tokens[i].end);
    others
        .find(|&j| shapes[j] != Written::Short && (tokens[j].start, tokens[j].end) != span)
        .is_some_and(|j| shapes[j] == Written::Capitals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lexicon::{FstLexicon, build_lexicon_blobs};

    fn lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) = build_lexicon_blobs(
            &[("teams", "team"), ("ships", "ship"), ("jumps", "jump")],
            &[
                "a", "i", "the", "quick", "brown", "fox", "over", "lazy", "dog", "do", "not",
                "code", "every", "single", "morning",
            ],
        )
        .expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    #[test]
    fn spec_scenario_fully_french_page_is_not_analysable() {
        let blocks = [
            "Les équipes ne livrent jamais le vendredi soir, c'est une règle ancienne.",
            "La revue de code reste obligatoire pour chaque changement proposé.",
        ];
        assert_eq!(
            analyse_document(&blocks, StudiedLanguage::English, &lexicon()),
            DocumentAnalysis::NotAnalysable
        );
    }

    #[test]
    fn french_blocks_are_excluded_english_ones_analysed() {
        let blocks = [
            "The quick brown fox jumps over the lazy dog every single morning.",
            "Les équipes ne livrent jamais le vendredi soir, c'est une règle ancienne.",
        ];
        match analyse_document(&blocks, StudiedLanguage::English, &lexicon()) {
            DocumentAnalysis::Analysed(tokens) => {
                assert!(
                    tokens.iter().all(|t| t.block == 0),
                    "only the English block counts"
                );
                assert!(tokens.len() >= 10);
                let jumps = tokens.iter().find(|t| t.surface == "jumps").expect("jumps");
                assert_eq!(jumps.lemma, "jump");
            }
            other => panic!("expected analysed document, got {other:?}"),
        }
    }

    #[test]
    fn spans_index_into_their_own_block() {
        let block = "The quick brown fox jumps over the lazy dog every single morning.";
        match analyse_document(&[block], StudiedLanguage::English, &lexicon()) {
            DocumentAnalysis::Analysed(tokens) => {
                for t in &tokens {
                    assert_eq!(&block[t.start..t.end], t.surface, "span must match surface");
                }
            }
            other => panic!("expected analysed document, got {other:?}"),
        }
    }

    #[test]
    fn an_unknown_compound_keeps_its_parts_a_known_one_is_a_unit() {
        // `x-ray` is a listed lexicon compound (a unit — its own lemma, no
        // parts); `code-quick` is not (split into part lemmas).
        let lexicon = {
            let (bytes, pool) =
                build_lexicon_blobs(&[("x-ray", "x-ray")], &["the", "fox", "code", "quick"])
                    .expect("build");
            FstLexicon::from_slices(bytes, &pool).expect("load")
        };
        let block = "The x-ray beats the code-quick fox that ran over the lazy dog today now.";
        match analyse_document(&[block], StudiedLanguage::English, &lexicon) {
            DocumentAnalysis::Analysed(tokens) => {
                let xray = tokens.iter().find(|t| t.surface == "x-ray").expect("x-ray");
                assert_eq!(xray.lemma, "x-ray");
                assert!(xray.parts.is_empty(), "a listed compound is a lexical unit");

                let cq = tokens
                    .iter()
                    .find(|t| t.surface == "code-quick")
                    .expect("code-quick");
                assert_eq!(cq.lemma, "code-quick", "unknown compound is not stemmed");
                assert_eq!(cq.parts, ["code", "quick"]);
                // The span still slices back to the surface.
                assert_eq!(&block[cq.start..cq.end], "code-quick");
            }
            other => panic!("expected analysed document, got {other:?}"),
        }
    }

    // — a headword's reading (add-lingua-french-expression-keys D1, D6) —

    /// A French lexicon holding the words of the headwords below, `t` and `c` left out.
    fn french_lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) = build_lexicon_blobs(
            &[("a", "avoir"), ("est", "être")],
            &[
                "à", "le", "revoir", "coup", "de", "œil", "abord", "que", "être", "ce", "il", "y",
                "avoir", "compte", "en", "vitamine",
            ],
        )
        .expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    /// A headword's reading as (surface, dictionary form) pairs.
    fn reading(
        headword: &str,
        studied: StudiedLanguage,
        lexicon: &FstLexicon<Vec<u8>>,
    ) -> Option<Vec<(String, String)>> {
        headword_reading(headword, studied, lexicon).map(|tokens| {
            tokens
                .into_iter()
                .map(|(token, lemma)| (token.text, lemma))
                .collect()
        })
    }

    fn pairs(expected: &[(&str, &str)]) -> Option<Vec<(String, String)>> {
        Some(
            expected
                .iter()
                .map(|(surface, lemma)| ((*surface).to_owned(), (*lemma).to_owned()))
                .collect(),
        )
    }

    #[test]
    fn a_french_headword_is_read_as_a_page_is() {
        let lexicon = french_lexicon();
        let fr = StudiedLanguage::French;
        // A contracted article is `à` + `le`, in the case it was written in.
        assert_eq!(
            reading("au revoir", fr, &lexicon),
            pairs(&[("à", "à"), ("le", "le"), ("revoir", "revoir")])
        );
        assert_eq!(
            reading("Au revoir", fr, &lexicon),
            pairs(&[("À", "à"), ("le", "le"), ("revoir", "revoir")])
        );
        // An elided word is the word it stands for, its own token.
        assert_eq!(
            reading("coup d'œil", fr, &lexicon),
            pairs(&[("coup", "coup"), ("de", "de"), ("œil", "œil")])
        );
        assert_eq!(
            reading("d'abord", fr, &lexicon),
            pairs(&[("de", "de"), ("abord", "abord")])
        );
        // An inversion is read as words, the typographic apostrophe as the straight one.
        assert_eq!(
            reading("Qu\u{2019}est-ce que", fr, &lexicon),
            pairs(&[
                ("Que", "que"),
                ("est", "être"),
                ("ce", "ce"),
                ("que", "que")
            ])
        );
        // Each token is resolved to its dictionary form.
        assert_eq!(
            reading("il y a", fr, &lexicon),
            pairs(&[("il", "il"), ("y", "y"), ("a", "avoir")])
        );
    }

    #[test]
    fn a_headword_part_of_which_gives_no_token_has_no_reading() {
        // `t` and `c` are single letters the lexicon does not list: the tokeniser drops them,
        // and what remains (`compte en`) is not the headword.
        let lexicon = french_lexicon();
        assert_eq!(
            reading("compte en t", StudiedLanguage::French, &lexicon),
            None
        );
        assert_eq!(
            reading("vitamine c", StudiedLanguage::French, &lexicon),
            None
        );
        // A word holding a digit gives none either.
        assert_eq!(reading("coup 2", StudiedLanguage::French, &lexicon), None);
        // A headword's extra spaces are no words.
        assert_eq!(
            reading(" coup  de ", StudiedLanguage::French, &lexicon),
            pairs(&[("coup", "coup"), ("de", "de")])
        );
    }

    // — a word standing alone in capitals (keep-lingua-french-acronyms-in-capitals D1) —

    /// A French lexicon holding the acronym `CA` as its own lemma, `ca` as a form of `ça`
    /// (read-lingua-french-ca-as-ca-cedilla), `ÉTÉ` as an acronym too (for the composed
    /// lookup), and the words of the texts below; `w` is a single letter it lists.
    fn acronym_lexicon() -> FstLexicon<Vec<u8>> {
        let (bytes, pool) = build_lexicon_blobs(
            &[
                ("CA", "CA"),
                ("ÉTÉ", "ÉTÉ"),
                ("ca", "ça"),
                ("a", "avoir"),
                ("voté", "voter"),
                ("suffit", "suffire"),
                ("été", "être"),
            ],
            &[
                "le", "ça", "avoir", "voter", "budget", "il", "crier", "suffire", "w", "à",
                "marché", "aller", "conseil", "être", "dernier", "chicago", "ls", "onu",
            ],
        )
        .expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    /// Each token of `text` read as French, with its answer.
    fn alone(text: &str) -> Vec<(String, bool)> {
        let lexicon = acronym_lexicon();
        let tokens = tokenize(text, StudiedLanguage::French, &lexicon);
        let answers = alone_in_capitals(text, &tokens, StudiedLanguage::French);
        assert_eq!(answers.len(), tokens.len());
        tokens.into_iter().map(|t| t.text).zip(answers).collect()
    }

    /// The tokens of `text` that stand alone in capitals.
    fn alone_words(text: &str) -> Vec<String> {
        alone(text)
            .into_iter()
            .filter_map(|(word, alone)| alone.then_some(word))
            .collect()
    }

    /// Each token of a one-block French page `text` with its dictionary form.
    fn page_lemmas(text: &str) -> Vec<(String, String)> {
        let lexicon = acronym_lexicon();
        let tokens = tokenize(text, StudiedLanguage::French, &lexicon);
        tokens
            .iter()
            .zip(alone_in_capitals(text, &tokens, StudiedLanguage::French))
            .map(|(token, alone)| {
                let (lemma, _) = resolve_lemmas(token, alone, StudiedLanguage::French, &lexicon);
                (token.text.clone(), lemma)
            })
            .collect()
    }

    fn lemma_of(text: &str, surface: &str) -> Option<String> {
        page_lemmas(text)
            .into_iter()
            .find(|(written, _)| written == surface)
            .map(|(_, lemma)| lemma)
    }

    #[test]
    fn spec_scenario_the_acronym_in_a_line_of_lowercase_words() {
        assert_eq!(alone_words("Le CA a voté le budget."), ["CA"]);
        assert_eq!(
            lemma_of("Le CA a voté le budget.", "CA").as_deref(),
            Some("CA")
        );
        // Its neighbours are the words, never the punctuation.
        assert_eq!(alone_words("« CA », dit-il."), ["CA"]);
    }

    #[test]
    fn spec_scenario_a_headline_in_capitals() {
        // `LE` and `VOTÉ` beside it are in capitals; the single letter `A` is passed over.
        assert!(alone_words("LE CA A VOTÉ LE BUDGET").is_empty());
        assert_eq!(
            lemma_of("LE CA A VOTÉ LE BUDGET", "CA").as_deref(),
            Some("ça")
        );
    }

    #[test]
    fn spec_scenario_a_shout_in_a_line_of_lowercase_words() {
        assert!(alone_words("Il a crié : CA SUFFIT !").is_empty());
        assert_eq!(
            lemma_of("Il a crié : CA SUFFIT !", "CA").as_deref(),
            Some("ça")
        );
        // A run of capitals after it is enough, and so is one before it.
        assert!(alone_words("il a dit OUAH CA chauffe").is_empty());
    }

    #[test]
    fn spec_scenario_a_capital_initial() {
        assert!(alone_words("Ca fait du bien.").is_empty());
        assert_eq!(lemma_of("Ca fait du bien.", "Ca").as_deref(), Some("ça"));
        assert_eq!(lemma_of("ca va", "ca").as_deref(), Some("ça"));
    }

    #[test]
    fn spec_scenario_a_word_in_capitals_the_table_does_not_hold() {
        // `ONU` stands alone in capitals, but the pack lists no form `ONU`: the cascade reads it.
        assert_eq!(alone_words("L'ONU a voté"), ["ONU"]);
        assert_eq!(lemma_of("L'ONU a voté", "ONU").as_deref(), Some("onu"));
        // A form the pack holds in capitals as another word's is no acronym: none is written,
        // the builder refusing it, but the lookup asks for the form's own lemma.
        let (bytes, pool) =
            build_lexicon_blobs(&[("CA", "ça"), ("ca", "ça")], &["le"]).expect("build");
        let lexicon = FstLexicon::from_slices(bytes, &pool).expect("load");
        let tokens = tokenize("le CA vote", StudiedLanguage::French, &lexicon);
        let (lemma, _) = resolve_lemmas(&tokens[1], true, StudiedLanguage::French, &lexicon);
        assert_eq!(lemma, "ça");
    }

    #[test]
    fn a_single_letter_and_a_number_are_passed_over() {
        // `W` is a single letter the lexicon keeps, `02` a number the tokeniser drops.
        assert_eq!(alone_words("le CA W 02 du dossier"), ["CA"]);
        // « CA-14 »: a run holding a digit is read piece by piece, `CA` a word of its own.
        assert_eq!(alone_words("le croiseur Chicago (CA-14)"), ["CA"]);
        assert_eq!(
            lemma_of("le croiseur Chicago (CA-14)", "CA").as_deref(),
            Some("CA")
        );
        // A single letter in capitals is never a word in capitals.
        assert!(alone_words("le W du dossier").is_empty());
    }

    #[test]
    fn the_two_pieces_of_au_count_once() {
        // `AU` is `À` + `le` sharing one span: each piece reads past the other, to the words
        // beside the contraction.
        assert_eq!(
            alone("va AU marché"),
            [
                ("va".to_owned(), false),
                ("À".to_owned(), true),
                ("le".to_owned(), true),
                ("marché".to_owned(), false)
            ]
        );
        // Neither piece reads as an acronym: the pack lists none, and the cascade reads them.
        assert_eq!(
            page_lemmas("va AU marché"),
            [
                ("va".to_owned(), "va".to_owned()),
                ("À".to_owned(), "à".to_owned()),
                ("le".to_owned(), "le".to_owned()),
                ("marché".to_owned(), "marché".to_owned())
            ]
        );
        // And as a neighbour, the contraction in capitals is one word in capitals.
        assert!(alone_words("le CA AU marché").is_empty());
    }

    #[test]
    fn a_word_with_no_neighbour_stands_alone() {
        // A one-word block, a selection of the word alone, a card's written word.
        assert_eq!(alone_words("CA"), ["CA"]);
        assert_eq!(lemma_of("CA", "CA").as_deref(), Some("CA"));
        assert_eq!(alone_words("CA !"), ["CA"]);
        assert_eq!(alone_words("le CA"), ["CA"]);
        assert_eq!(alone_words("CA le"), ["CA"]);
    }

    #[test]
    fn a_soft_hyphen_is_no_letter() {
        // « C‧A »: the soft hyphen is read as nowhere, the token is `CA` and stands alone.
        assert_eq!(alone_words("le C\u{AD}A vote"), ["CA"]);
        assert_eq!(lemma_of("le C\u{AD}A vote", "CA").as_deref(), Some("CA"));
        // A soft hyphen in a lowercase neighbour leaves it lowercase.
        assert_eq!(alone_words("le CA vo\u{AD}te"), ["CA"]);
    }

    #[test]
    fn a_decomposed_capital_is_read_composed() {
        // « ÉTÉ » written with combining accents: its letters are E, T, E — capitals —, and the
        // lookup reads the token composed, as the pack writes it.
        let text = "le E\u{301}TE\u{301} dernier";
        assert_eq!(alone_words(text), ["ÉTÉ"]);
        assert_eq!(lemma_of(text, "ÉTÉ").as_deref(), Some("ÉTÉ"));
        // In a line of capitals it is `été`, read by the cascade as the tables read it.
        assert_eq!(
            lemma_of("LE E\u{301}TE\u{301} DERNIER", "ÉTÉ").as_deref(),
            Some("être")
        );
    }

    #[test]
    fn a_hyphenated_run_is_read_by_its_parts_as_before() {
        // « LS-CA » is one token of two parts: never a word in capitals, its parts read as
        // before, `CA` as `ça`; and as a neighbour it is written in capitals.
        assert!(alone_words("le LS-CA de série").is_empty());
        let lexicon = acronym_lexicon();
        let text = "le LS-CA de série";
        let tokens = tokenize(text, StudiedLanguage::French, &lexicon);
        let run = tokens.iter().find(|t| t.text == "LS-CA").expect("LS-CA");
        assert_eq!(
            resolve_lemmas(run, false, StudiedLanguage::French, &lexicon),
            ("ls-ca".to_owned(), vec!["ls".to_owned(), "ça".to_owned()])
        );
        assert!(alone_words("le CA LS-CA").is_empty());
    }

    #[test]
    fn a_french_headword_reads_capitals_as_a_page_does() {
        let lexicon = acronym_lexicon();
        let fr = StudiedLanguage::French;
        assert_eq!(
            reading("le CA", fr, &lexicon),
            pairs(&[("le", "le"), ("CA", "CA")])
        );
        assert_eq!(
            reading("ÇA SUFFIT", fr, &lexicon),
            pairs(&[("ÇA", "ça"), ("SUFFIT", "suffire")])
        );
        assert_eq!(
            reading("ça suffit", fr, &lexicon),
            pairs(&[("ça", "ça"), ("suffit", "suffire")])
        );
    }

    #[test]
    fn english_and_spanish_text_reads_capitals_as_before() {
        // No answer is yes for English or Spanish, whatever the text, and a lexicon listing
        // `US` or `CA` as written changes nothing: their tokens are lowercased as before.
        let (bytes, pool) = build_lexicon_blobs(
            &[("US", "US"), ("CA", "CA")],
            &["the", "army", "us", "el", "ca", "votó"],
        )
        .expect("build");
        let lexicon = FstLexicon::from_slices(bytes, &pool).expect("load");
        for (text, studied) in [
            ("the US army", StudiedLanguage::English),
            ("the CA army", StudiedLanguage::English),
            ("el CA votó", StudiedLanguage::Spanish),
            ("el US votó", StudiedLanguage::Spanish),
        ] {
            let tokens = tokenize(text, studied, &lexicon);
            assert!(
                alone_in_capitals(text, &tokens, studied)
                    .iter()
                    .all(|alone| !alone),
                "{text}"
            );
            for token in &tokens {
                let read = resolve_lemmas(token, false, studied, &lexicon);
                assert_eq!(
                    resolve_lemmas(token, true, studied, &lexicon),
                    read,
                    "{text}"
                );
                assert_eq!(read.0, read.0.to_lowercase(), "{text}");
            }
        }
    }

    #[test]
    fn an_english_and_a_spanish_headword_read_as_their_tokens() {
        // English's pre-pass expands `n't`; Spanish's splits `al` into `a` + `el`. Only French's
        // pack keys its expressions on a reading; these show the function is every language's.
        let english = lexicon();
        assert_eq!(
            reading("do not jumps", StudiedLanguage::English, &english),
            pairs(&[("do", "do"), ("not", "not"), ("jumps", "jump")])
        );
        assert_eq!(
            reading("don't code", StudiedLanguage::English, &english),
            pairs(&[("do", "do"), ("not", "not"), ("code", "code")])
        );
        let spanish = {
            let (bytes, pool) =
                build_lexicon_blobs(&[], &["a", "el", "pie", "de", "letra"]).expect("build");
            FstLexicon::from_slices(bytes, &pool).expect("load")
        };
        assert_eq!(
            reading("al pie de la letra", StudiedLanguage::Spanish, &spanish),
            pairs(&[
                ("a", "a"),
                ("el", "el"),
                ("pie", "pie"),
                ("de", "de"),
                ("la", "la"),
                ("letra", "letra")
            ])
        );
    }
}
