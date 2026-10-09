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

//! Per-block language gating (design D4).
//!
//! Real pages mix languages (native-language UI, quotations, code), so
//! detection runs per block of text, never per document. A block outside the
//! studied language — the user's native language included — is excluded from
//! the analysis entirely; a document with too little studied-language content
//! is *not analysable* rather than misleadingly scored.

use serde::{Deserialize, Serialize};

/// Languages the pipeline can study. Extended change by change; each variant
/// carries its own tokenisation pre-pass, lemmatisation cascade and analyser
/// version (generalise-lingua-analysis-by-language).
///
/// New variants go AFTER the existing ones: the derived order keys serialised
/// maps, so appending keeps every stored state byte-identical.
///
/// `Ord`/`Hash` so it can key the knowledge model's per-`(language, lemma)`
/// maps (`add-lingua-knowledge-model`); ordering keeps serialised state
/// deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StudiedLanguage {
    /// English.
    English,
    /// Spanish. Its analyser is a baseline until its own pre-pass and cascade
    /// land (add-lingua-spanish-analysis): no English rule ever runs on it.
    Spanish,
    /// French. Served by the baseline analysis until its own rules land
    /// (add-lingua-french-baseline): the rules that belong to no language and
    /// the pack's forms, nothing of English's or Spanish's.
    French,
}

impl StudiedLanguage {
    /// Every language the core can analyse, in declaration order.
    pub const ALL: [StudiedLanguage; 3] = [
        StudiedLanguage::English,
        StudiedLanguage::Spanish,
        StudiedLanguage::French,
    ];

    /// ISO 639-1 tag: `en`, `es`, `fr`. The form packs (`meta.studied`), the
    /// wire and the extension use.
    pub fn tag(self) -> &'static str {
        match self {
            StudiedLanguage::English => "en",
            StudiedLanguage::Spanish => "es",
            StudiedLanguage::French => "fr",
        }
    }

    /// The language a tag names, or `None` for a language the core has no
    /// analyser for. Exact: tags are normalised where they enter the system.
    pub fn from_tag(tag: &str) -> Option<StudiedLanguage> {
        StudiedLanguage::ALL.into_iter().find(|l| l.tag() == tag)
    }

    /// Version of this language's analysis pipeline. Each language has its
    /// own, so a change to one language's rules never invalidates another's
    /// pack or counts. Bump on ANY change that can alter this language's
    /// output (see [`crate::analysis::ANALYZER_VERSION`] for English's).
    pub fn analyzer_version(self) -> &'static str {
        match self {
            StudiedLanguage::English => crate::analysis::ANALYZER_VERSION,
            StudiedLanguage::Spanish => crate::analysis::SPANISH_ANALYZER_VERSION,
            StudiedLanguage::French => crate::analysis::FRENCH_ANALYZER_VERSION,
        }
    }

    fn whichlang_target(self) -> whichlang::Lang {
        match self {
            StudiedLanguage::English => whichlang::Lang::Eng,
            StudiedLanguage::Spanish => whichlang::Lang::Spa,
            StudiedLanguage::French => whichlang::Lang::Fra,
        }
    }
}

/// Blocks shorter than this (in bytes, once trimmed) are too small for
/// reliable detection; they are excluded from the analysis rather than
/// guessed at.
pub const MIN_BLOCK_BYTES: usize = 12;

/// A document whose studied-language blocks yield fewer counted tokens than
/// this is reported not analysable (spec: "without enough content in the
/// studied language").
pub const MIN_ANALYSABLE_TOKENS: usize = 10;

/// Whether one block of text is in the studied language.
pub fn block_is_studied(text: &str, studied: StudiedLanguage) -> bool {
    let trimmed = text.trim();
    if trimmed.len() < MIN_BLOCK_BYTES {
        return false;
    }
    detect(trimmed) == studied.whichlang_target()
}

/// whichlang's language for a trimmed block, with Spanish's guard: whichlang has no Catalan or
/// Galician class and reads many of their blocks as Spanish, so a block it reads as Spanish whose
/// Catalan or Galician function words outnumber its Spanish ones is not Spanish
/// (add-lingua-spanish-detection-guard).
fn detect(trimmed: &str) -> whichlang::Lang {
    let detected = whichlang::detect_language(trimmed);
    if detected == whichlang::Lang::Spa && iberian_neighbour(trimmed) {
        // Neither studied language: the block is excluded, as any other language's is.
        return whichlang::Lang::Por;
    }
    detected
}

/// Function words Catalan uses and Spanish does not (sorted for binary search).
const CATALAN_MARKERS: &[&str] = &[
    "amb", "aquest", "aquesta", "aquestes", "aquests", "dels", "després", "els", "encara", "eren",
    "fins", "fou", "havia", "llavors", "mateix", "molt", "més", "on", "pels", "per", "perquè",
    "però", "segons", "sempre", "seu", "seus", "seva", "seves", "són", "també", "tot", "és",
];

/// Function words Galician uses and Spanish does not (sorted for binary search).
const GALICIAN_MARKERS: &[&str] = &[
    "ao", "aos", "aínda", "cando", "coa", "coas", "da", "das", "do", "elas", "eles", "foi", "hai",
    "iso", "isto", "lle", "lles", "moi", "máis", "non", "nun", "nunha", "onde", "pola", "polas",
    "polo", "polos", "súa", "tamén", "unha", "unhas", "xa",
];

/// Function words Spanish uses and neither Catalan nor Galician does (sorted).
const SPANISH_MARKERS: &[&str] = &[
    "ahora", "aunque", "después", "entonces", "fue", "había", "hay", "las", "lo", "los", "muy",
    "más", "por", "su", "sus", "también", "y",
];

/// Whether a block whichlang reads as Spanish is more likely Catalan or Galician: either's
/// function words outnumber Spanish's. A tie, or no marker at all, stays Spanish.
fn iberian_neighbour(text: &str) -> bool {
    let (mut catalan, mut galician, mut spanish) = (0usize, 0usize, 0usize);
    for word in text.split(|c: char| !c.is_alphabetic() && c != '\'') {
        let lower = word.to_lowercase();
        // Catalan's elisions (`l'home`, `d'aquesta`) are markers of their own.
        if ["l'", "d'", "s'", "n'"]
            .iter()
            .any(|elision| lower.starts_with(elision) && lower.len() > 2)
        {
            catalan += 1;
            continue;
        }
        let lower = lower.trim_matches('\'');
        catalan += usize::from(CATALAN_MARKERS.binary_search(&lower).is_ok());
        galician += usize::from(GALICIAN_MARKERS.binary_search(&lower).is_ok());
        spanish += usize::from(SPANISH_MARKERS.binary_search(&lower).is_ok());
    }
    catalan > spanish || galician > spanish
}

/// The language of a document, chosen among `candidates` (add-lingua-language-routing D1).
///
/// Each block long enough to be detected votes, weighted by its trimmed length, for the
/// language whichlang finds in it, when that language is a candidate: a page's short chrome
/// cannot outvote its text. The most weight wins. A tie goes to `hint` (the document's declared
/// language) when it is among the tied, else to the earlier candidate. With no vote, `hint`
/// wins if it is a candidate, else the first candidate. A single candidate is returned without
/// detecting anything. `None` only when there is no candidate.
pub fn detect_document_language(
    blocks: &[&str],
    candidates: &[StudiedLanguage],
    hint: Option<StudiedLanguage>,
) -> Option<StudiedLanguage> {
    let first = *candidates.first()?;
    if candidates.len() == 1 {
        return Some(first);
    }
    let mut weight = vec![0usize; candidates.len()];
    for block in blocks {
        let trimmed = block.trim();
        if trimmed.len() < MIN_BLOCK_BYTES {
            continue;
        }
        let detected = detect(trimmed);
        if let Some(i) = candidates
            .iter()
            .position(|c| c.whichlang_target() == detected)
        {
            weight[i] += trimmed.len();
        }
    }
    Some(choose_language(candidates, &weight, hint))
}

/// The winner of a vote: `weight[i]` is what `candidates[i]` collected. The most weight wins;
/// a tie goes to `hint` when it is among the tied, else to the earlier candidate; with no
/// weight at all, `hint` if it is a candidate, else the first candidate. `candidates` is not
/// empty.
fn choose_language(
    candidates: &[StudiedLanguage],
    weight: &[usize],
    hint: Option<StudiedLanguage>,
) -> StudiedLanguage {
    let hinted = hint.filter(|h| candidates.contains(h));
    let best = weight.iter().copied().max().unwrap_or(0);
    if best == 0 {
        return hinted.unwrap_or(candidates[0]);
    }
    let tied: Vec<StudiedLanguage> = candidates
        .iter()
        .zip(weight)
        .filter(|(_, w)| **w == best)
        .map(|(c, _)| *c)
        .collect();
    match hinted {
        Some(h) if tied.contains(&h) => h,
        _ => tied[0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: StudiedLanguage = StudiedLanguage::English;
    const ES: StudiedLanguage = StudiedLanguage::Spanish;
    const FR: StudiedLanguage = StudiedLanguage::French;
    const SPANISH: &str =
        "Los equipos nunca entregan el viernes por la noche, es una regla antigua.";
    const ENGLISH: &str =
        "The quick brown fox jumps over the lazy dog every single morning of the week.";

    #[test]
    fn a_spanish_page_is_spanish_among_english_and_spanish() {
        let page = [
            SPANISH,
            "El faro se alza sobre las rocas desde hace más de un siglo.",
        ];
        assert_eq!(detect_document_language(&page, &[EN, ES], None), Some(ES));
        assert_eq!(detect_document_language(&page, &[ES, EN], None), Some(ES));
    }

    #[test]
    fn long_english_paragraphs_outweigh_a_short_spanish_quotation() {
        let long = ENGLISH.repeat(3);
        let page = [long.as_str(), long.as_str(), "«Hasta la vista, amigo mío»"];
        assert_eq!(detect_document_language(&page, &[ES, EN], None), Some(EN));
    }

    #[test]
    fn a_page_with_nothing_to_detect_goes_to_its_hint_else_the_first_candidate() {
        let page = ["Menu", "Accueil", "OK"];
        assert_eq!(
            detect_document_language(&page, &[EN, ES], Some(ES)),
            Some(ES)
        );
        assert_eq!(detect_document_language(&page, &[EN, ES], None), Some(EN));
        // A hint that is not a candidate decides nothing.
        assert_eq!(
            detect_document_language(&page, &[ES], Some(EN)),
            Some(ES),
            "one candidate wins whatever the hint"
        );
    }

    #[test]
    fn a_tie_goes_to_the_hint_then_to_the_reader_order() {
        assert_eq!(choose_language(&[ES, EN], &[40, 40], None), ES);
        assert_eq!(choose_language(&[ES, EN], &[40, 40], Some(EN)), EN);
        // A hint outside the tie decides nothing; the most weight still wins.
        assert_eq!(choose_language(&[ES, EN], &[10, 40], Some(ES)), EN);
        assert_eq!(choose_language(&[ES, EN], &[0, 0], Some(EN)), EN);
        assert_eq!(choose_language(&[ES, EN], &[0, 0], None), ES);
    }

    #[test]
    fn a_single_candidate_or_none() {
        assert_eq!(detect_document_language(&[SPANISH], &[EN], None), Some(EN));
        assert_eq!(detect_document_language(&[SPANISH], &[], Some(ES)), None);
    }

    #[test]
    fn english_block_is_studied() {
        assert!(block_is_studied(
            "The quick brown fox jumps over the lazy dog every single morning.",
            StudiedLanguage::English,
        ));
    }

    #[test]
    fn french_block_is_excluded() {
        assert!(!block_is_studied(
            "Les équipes ne livrent jamais le vendredi soir, c'est une règle ancienne.",
            StudiedLanguage::English,
        ));
    }

    #[test]
    fn a_spanish_block_is_studied_by_a_spanish_learner_only() {
        let block = "Los equipos nunca entregan el viernes por la noche, es una regla antigua.";
        assert!(block_is_studied(block, StudiedLanguage::Spanish));
        assert!(!block_is_studied(block, StudiedLanguage::English));
    }

    #[test]
    fn an_english_block_is_excluded_for_a_spanish_learner() {
        assert!(!block_is_studied(
            "The quick brown fox jumps over the lazy dog every single morning.",
            StudiedLanguage::Spanish,
        ));
    }

    #[test]
    fn tags_round_trip_and_unknown_tags_have_no_analyser() {
        for lang in StudiedLanguage::ALL {
            assert_eq!(StudiedLanguage::from_tag(lang.tag()), Some(lang));
        }
        assert_eq!(
            StudiedLanguage::ALL.map(StudiedLanguage::tag),
            ["en", "es", "fr"]
        );
        assert_eq!(StudiedLanguage::from_tag("fr"), Some(FR));
        assert_eq!(StudiedLanguage::from_tag("pt"), None);
        assert_eq!(
            StudiedLanguage::from_tag("EN"),
            None,
            "tags are normalised upstream"
        );
    }

    #[test]
    fn english_keeps_its_analyser_version_and_spanish_has_its_own() {
        // Each language reads its own constant; the numbers may meet (both were `1.1.0` after
        // add-lingua-spanish-detection-guard) without a pack of one passing for the other's.
        assert_eq!(StudiedLanguage::English.analyzer_version(), "1.1.0");
        assert_eq!(
            StudiedLanguage::Spanish.analyzer_version(),
            crate::analysis::SPANISH_ANALYZER_VERSION
        );
        // French has its own tokenisation pre-pass and the baseline's lemmas
        // (add-lingua-french-tokenisation D9); its cascade makes it `1.0.0`.
        assert_eq!(StudiedLanguage::French.analyzer_version(), "0.2.0");
        assert_eq!(
            StudiedLanguage::French.analyzer_version(),
            crate::analysis::FRENCH_ANALYZER_VERSION
        );
    }

    #[test]
    fn appending_spanish_keeps_english_first_and_its_serialised_name() {
        assert!(StudiedLanguage::English < StudiedLanguage::Spanish);
        assert_eq!(
            serde_json::to_string(&StudiedLanguage::English).unwrap(),
            "\"English\""
        );
    }

    #[test]
    fn appending_french_keeps_the_order_and_every_serialised_name() {
        // add-lingua-french-baseline D1: the derived order keys the knowledge model's maps and
        // serde writes the variants' names, so a state of English or Spanish does not move.
        assert!(EN < ES && ES < FR);
        assert_eq!(
            StudiedLanguage::ALL.map(|language| serde_json::to_value(language).unwrap()),
            ["English", "Spanish", "French"].map(serde_json::Value::from)
        );
    }

    const FRENCH: &str =
        "Les équipes ne livrent jamais le vendredi soir, c'est une règle ancienne.";

    #[test]
    fn spec_scenario_a_french_block_for_a_learner_of_french_only() {
        assert!(block_is_studied(FRENCH, FR));
        assert!(!block_is_studied(FRENCH, EN));
        assert!(!block_is_studied(FRENCH, ES));
        // An English or Spanish block is excluded for a learner of French.
        assert!(!block_is_studied(ENGLISH, FR));
        assert!(!block_is_studied(SPANISH, FR));
    }

    #[test]
    fn the_spanish_guard_leaves_french_detection_alone() {
        // `l'` and `d'` count as Catalan in the guard, which only judges a block read as Spanish.
        let elided = "L'homme d'aujourd'hui n'a plus le temps de lire, dit-elle.";
        assert_eq!(detect(elided), whichlang::detect_language(elided));
        assert_eq!(detect(elided), whichlang::Lang::Fra);
        assert!(block_is_studied(elided, FR));
        assert_eq!(
            detect_document_language(&[FRENCH, elided], &[ES, FR], None),
            Some(FR)
        );
        assert_eq!(
            detect_document_language(&[SPANISH], &[FR, ES], None),
            Some(ES)
        );
    }

    const CATALAN: &str =
        "El govern ha aprovat el pressupost amb el suport dels grups, però també amb crítiques.";
    const GALICIAN: &str = "Non hai unha solución sinxela, pero o concello xa traballa nela.";
    const SPANISH_PROSE: &str =
        "El gobierno aprobó el presupuesto con el apoyo de los grupos, pero también hubo críticas.";

    #[test]
    fn spec_scenario_a_catalan_paragraph_is_not_spanish() {
        assert!(!block_is_studied(CATALAN, ES));
        // `l'` and `d'` count as Catalan.
        assert!(iberian_neighbour(
            "L'home d'aquesta ciutat va arribar ahir."
        ));
    }

    #[test]
    fn spec_scenario_a_galician_paragraph_is_not_spanish() {
        assert!(!block_is_studied(GALICIAN, ES));
    }

    #[test]
    fn spec_scenario_spanish_prose_is_kept() {
        assert!(block_is_studied(SPANISH_PROSE, ES));
        assert!(!iberian_neighbour(SPANISH_PROSE));
    }

    #[test]
    fn spec_scenario_a_catalan_page_gives_spanish_no_vote() {
        // Nothing votes, so the reader's first candidate wins: never Spanish by Catalan's weight.
        assert_eq!(
            detect_document_language(&[CATALAN, CATALAN], &[EN, ES], None),
            Some(EN)
        );
        assert_eq!(
            detect_document_language(&[SPANISH_PROSE], &[EN, ES], None),
            Some(ES)
        );
    }

    #[test]
    fn spec_scenario_the_leak_the_guard_leaves() {
        // No marker of either kind: the guard cannot judge it, and does not refuse it.
        assert!(!iberian_neighbour("el convent del Carme"));
    }

    #[test]
    fn a_tie_or_no_marker_stays_spanish() {
        assert!(!iberian_neighbour("amb los"));
        assert!(!iberian_neighbour("la casa"));
        assert!(iberian_neighbour("amb els"));
    }

    #[test]
    fn the_guard_leaves_english_detection_alone() {
        let english = "Teams don't ship code on Friday, and they never will without review.";
        assert_eq!(detect(english), whichlang::detect_language(english));
        assert!(block_is_studied(english, EN));
    }

    #[test]
    fn the_marker_tables_are_sorted_for_binary_search() {
        for table in [CATALAN_MARKERS, GALICIAN_MARKERS, SPANISH_MARKERS] {
            for pair in table.windows(2) {
                assert!(
                    pair[0] < pair[1],
                    "{:?} must sort before {:?}",
                    pair[0],
                    pair[1]
                );
            }
        }
    }

    #[test]
    fn tiny_blocks_are_excluded_not_guessed() {
        assert!(!block_is_studied("OK", StudiedLanguage::English));
        assert!(!block_is_studied("  the  ", StudiedLanguage::English));
    }
}
