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
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

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
    /// French (add-lingua-french-baseline), served by its own analysis since
    /// add-lingua-french-analysis: its tokenisation pre-pass, NFC, its cascade,
    /// its closed classes and its names rule — nothing of English's or Spanish's.
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
    detect(trimmed, &[studied]).is_some()
}

/// The language among `languages` whichlang finds in a trimmed block, unless that language's
/// guard refuses the block as a neighbour's (add-lingua-french-detection-guard D5). whichlang has
/// no class for Catalan, Galician, Occitan or Romanian and reads many of their blocks as Spanish
/// or French: Spanish's guard refuses Catalan and Galician (add-lingua-spanish-detection-guard),
/// French's Catalan, Occitan and Romanian; English has none. A guard runs only for the language
/// it protects, and only when that language is asked about: a reader of English pays for
/// neither, a reader of Spanish for Spanish's alone. `None` when whichlang finds none of
/// `languages`, or the guard refuses the block, which is then excluded as any other language's
/// is. The gate and the vote both ask here, so they never disagree.
fn detect(trimmed: &str, languages: &[StudiedLanguage]) -> Option<StudiedLanguage> {
    let detected = whichlang::detect_language(trimmed);
    let language = languages
        .iter()
        .copied()
        .find(|language| language.whichlang_target() == detected)?;
    let neighbour = match language {
        StudiedLanguage::English => false,
        StudiedLanguage::Spanish => iberian_neighbour(trimmed),
        StudiedLanguage::French => romance_neighbour(trimmed),
    };
    (!neighbour).then_some(language)
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

/// Function words Catalan, Occitan and Romanian write and French rarely does, sorted for binary
/// search (add-lingua-french-detection-guard D3): 152 — Catalan's (`amb`, `els`, `és`, `molt`,
/// `també`, `vaig`…), Occitan's (`lo`, `los`, `las`, `e`, `dins`, `èra`, `degun`, `çò`…), seven
/// both write (`al`, `amb`, `aquesta`, `aquestes`, `del`, `dels`, `una`), and Romanian's (`în`,
/// `să`, `și` and `şi`, `cu`, `pe`, `pentru`, `fost`…). Counted only where written in lowercase:
/// `El Niño`, `Los Angeles` and `Lo Pont de Montvert` are names.
///
/// Each word is kept as measured, one by one, on 148,267 blocks of Universal Dependencies,
/// Tatoeba, Wikipedia and Project Gutenberg text fetched on 2026-10-09 (the design's
/// Measurement names them; none is in the repository): it catches its language's blocks read as
/// French without refusing French. Left out, each measured: `es` (« tu es »), `on` (« on y
/// trouve »), `i` (Québécois « qu'i »), `mai` (the month), `ha` (hectares), `fou`, `o`, `pot`,
/// `uns` and `unes` (« les uns »), and French words or chat and Québécois spellings (`soi`, `com`,
/// `pus`, `jamai`, `hi`, `han`, `van`, `fins`). Franco-Provençal, Picard and Walloon have no
/// words here: they write French's own function words.
const ROMANCE_NEIGHBOUR_MARKERS: &[&str] = &[
    "această",
    "acest",
    "ahir",
    "aici",
    "això",
    "al",
    "allò",
    "als",
    "amb",
    "ansin",
    "aquel",
    "aquela",
    "aqueles",
    "aquell",
    "aquella",
    "aquelles",
    "aquells",
    "aquest",
    "aquesta",
    "aqueste",
    "aquestes",
    "aquests",
    "aquí",
    "aquò",
    "atât",
    "avui",
    "avètz",
    "care",
    "cel",
    "ceva",
    "coma",
    "cu",
    "când",
    "cât",
    "că",
    "dacă",
    "dambe",
    "dar",
    "dau",
    "decât",
    "degun",
    "dei",
    "deis",
    "del",
    "dels",
    "demà",
    "despre",
    "després",
    "deu",
    "din",
    "dins",
    "dintre",
    "după",
    "e",
    "ei",
    "el",
    "ella",
    "ells",
    "els",
    "em",
    "encara",
    "enqüèra",
    "ens",
    "eren",
    "estan",
    "este",
    "estic",
    "està",
    "estàs",
    "ets",
    "fi",
    "foarte",
    "foguèron",
    "foguèt",
    "foren",
    "fost",
    "fòrça",
    "havia",
    "havien",
    "ieu",
    "ja",
    "las",
    "lei",
    "leis",
    "lo",
    "los",
    "mas",
    "mateix",
    "mateixa",
    "meva",
    "meves",
    "molt",
    "més",
    "mă",
    "només",
    "nosaltres",
    "nòstra",
    "nòstre",
    "pasmens",
    "pe",
    "pel",
    "pels",
    "pentru",
    "per",
    "perquè",
    "perqué",
    "però",
    "puèi",
    "qual",
    "quan",
    "quin",
    "quina",
    "quora",
    "què",
    "res",
    "sau",
    "segons",
    "ser",
    "seus",
    "seva",
    "seves",
    "siá",
    "siás",
    "sunt",
    "sus",
    "sèm",
    "sètz",
    "són",
    "să",
    "també",
    "tampoc",
    "tanben",
    "teva",
    "tinc",
    "totjorn",
    "té",
    "ua",
    "una",
    "unde",
    "uèi",
    "vaig",
    "vòstra",
    "vòstre",
    "çò",
    "èi",
    "èra",
    "èran",
    "èsser",
    "és",
    "în",
    "şi",
    "și",
];

/// French function words Catalan, Occitan and Romanian rarely write, sorted for binary search
/// (add-lingua-french-detection-guard D3): 69, counted however they are written. Left out, each
/// measured: `les` (Catalan's article), `pas`, `son`, `mon`, `qui`, `on` and `ont` (Occitan
/// writes them), `ce` and `lui` (Romanian's *what* and *his*). The guard's own words, not the
/// closed classes of `function_words.rs`: they answer what French does not share with its
/// neighbours, not what a reader is not shown.
const FRENCH_MARKERS: &[&str] = &[
    "alors", "au", "aussi", "aux", "avait", "avant", "avec", "beaucoup", "bien", "cela", "ces",
    "cette", "chez", "comme", "dans", "depuis", "des", "donc", "dont", "du", "déjà", "elle",
    "elles", "encore", "est", "et", "fait", "ici", "il", "ils", "j", "jamais", "je", "le", "leur",
    "leurs", "là", "mais", "moi", "nous", "ou", "où", "parce", "pendant", "plus", "pour", "puis",
    "quoi", "rien", "sans", "sont", "sous", "suis", "sur", "toi", "toujours", "tous", "tout",
    "toute", "toutes", "trop", "très", "une", "vous", "à", "ça", "était", "été", "être",
];

/// The longest marker of either table, in bytes: a longer word is not looked up.
const LONGEST_MARKER_BYTES: usize = 9;

/// Whether a block whichlang reads as French is more likely Catalan, Occitan or Romanian: their
/// function words, counted together, outnumber French's own. A tie, or no marker at all, stays
/// French — a short line cannot be judged, and French comes first. A neighbour's word counts
/// only where it is written in lowercase; a French one however it is written.
///
/// The block is read as [`guard_words`] reads it; a word holding a combining mark is composed
/// (NFC) before it is looked up, so a decomposed `és` is Catalan's and a decomposed `été` is not
/// `e` + `te`.
fn romance_neighbour(text: &str) -> bool {
    let (mut neighbour, mut french) = (0usize, 0usize);
    let mut buf = String::new();
    guard_words(text, |word| {
        let decomposed = !word.is_ascii() && word.chars().any(is_combining_mark);
        if word.len() > LONGEST_MARKER_BYTES && !decomposed {
            return;
        }
        let lowercase = !word.chars().any(char::is_uppercase);
        let lower = if lowercase && !decomposed {
            word
        } else {
            buf.clear();
            buf.extend(word.nfc().flat_map(char::to_lowercase));
            buf.as_str()
        };
        if lowercase && ROMANCE_NEIGHBOUR_MARKERS.binary_search(&lower).is_ok() {
            neighbour += 1;
        } else if FRENCH_MARKERS.binary_search(&lower).is_ok() {
            french += 1;
        }
    });
    neighbour > french
}

/// Calls `f` with each word of `text` as French's guard reads it (add-lingua-french-detection-guard
/// D4): a run of letters and combining marks, joined across a hyphen, a middle dot (U+00B7), a
/// non-breaking hyphen (U+2011) or a full stop between two letters (`e-mail`, `étudiant·e·s`,
/// `col·lecció`); an apostrophe, a space or any other character ends it (`l'`, `qu'`), and a run
/// holding a digit is not a word (`2e`, `1re`). Splitting on every non-letter instead, as Spanish's
/// guard does, read `e` — Occitan's *and* — in `2e`, `e-mail` and `ingénieur-e-s`.
fn guard_words(text: &str, mut f: impl FnMut(&str)) {
    let mut start: Option<usize> = None;
    let mut digit = false;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c.is_alphabetic() || (!c.is_ascii() && is_combining_mark(c)) || c.is_numeric() {
            digit |= c.is_numeric();
            start.get_or_insert(i);
            continue;
        }
        let joins = matches!(c, '-' | '.' | '\u{00B7}' | '\u{2011}')
            && start.is_some()
            && chars.peek().is_some_and(|&(_, next)| next.is_alphabetic());
        if joins {
            continue;
        }
        if let Some(s) = start.take()
            && !digit
        {
            f(&text[s..i]);
        }
        digit = false;
    }
    if let Some(s) = start
        && !digit
    {
        f(&text[s..]);
    }
}

/// The language of a document, chosen among `candidates` (add-lingua-language-routing D1).
///
/// Each block long enough to be detected votes, weighted by its trimmed length, for the
/// candidate whichlang finds in it, unless that candidate's guard refuses the block (`detect`):
/// a page's short chrome cannot outvote its text, nor a Catalan paragraph vote French. The most weight wins. A tie goes to `hint` (the document's declared
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
        if let Some(language) = detect(trimmed, candidates)
            && let Some(i) = candidates.iter().position(|c| *c == language)
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
        // Each language reads its own constant; the numbers may meet (Spanish's was `1.1.0` after
        // add-lingua-spanish-detection-guard, French's is since add-lingua-french-detection-guard)
        // without a pack of one passing for the other's.
        assert_eq!(StudiedLanguage::English.analyzer_version(), "1.1.0");
        assert_eq!(
            StudiedLanguage::Spanish.analyzer_version(),
            crate::analysis::SPANISH_ANALYZER_VERSION
        );
        // French has its own analysis (add-lingua-french-analysis D5): its own constant, no
        // longer one of the baseline's `0.x` versions.
        assert_eq!(
            StudiedLanguage::French.analyzer_version(),
            crate::analysis::FRENCH_ANALYZER_VERSION
        );
        assert!(
            !StudiedLanguage::French.analyzer_version().starts_with("0."),
            "{}",
            StudiedLanguage::French.analyzer_version()
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
        assert_eq!(whichlang::detect_language(elided), whichlang::Lang::Fra);
        assert_eq!(detect(elided, &[ES, FR]), Some(FR));
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
        assert_eq!(whichlang::detect_language(english), whichlang::Lang::Eng);
        assert_eq!(detect(english, &[EN, ES, FR]), Some(EN));
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

    // add-lingua-french-detection-guard: Catalan, Occitan and Romanian are not read as French.
    const CATALAN_AS_FRENCH: &str =
        "Les tradicions de la ciutat són molt antigues i el carrer principal és ple de gent.";
    const OCCITAN_AS_FRENCH: &str =
        "Dins la vila, los carrièrs son estrechs e las ostals son vièlhas.";
    const ROMANIAN_AS_FRENCH: &str =
        "Guvernul a aprobat bugetul cu sprijinul grupurilor, dar și cu critici.";
    /// The French corpus's Occitan block (`pages-fr.txt`, the `mixte` page).
    const CORPUS_OCCITAN: &str = "Lo far se quilhava a la broa de la falèsa, e degun se remembrava pas de qual l'aviá bastit.";

    /// Each sentence a French guard's test offers as French is one whichlang reads as French, so
    /// that a whichlang update that stops reading it so fails here rather than letting the test
    /// pass for the wrong reason.
    fn read_as_french(block: &str) -> &str {
        assert_eq!(
            whichlang::detect_language(block),
            whichlang::Lang::Fra,
            "{block}"
        );
        block
    }

    /// A small French lexicon: the words of [`FRENCH`] the tokeniser keeps.
    fn french_lexicon() -> crate::analysis::lexicon::FstLexicon<Vec<u8>> {
        let (bytes, pool) = crate::analysis::lexicon::build_lexicon_blobs(
            &[("équipes", "équipe"), ("livrent", "livrer")],
            &[
                "le", "ne", "jamais", "vendredi", "soir", "ce", "être", "un", "règle",
            ],
        )
        .expect("build");
        crate::analysis::lexicon::FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    #[test]
    fn spec_scenario_a_catalan_paragraph_is_not_french() {
        assert!(!block_is_studied(read_as_french(CATALAN_AS_FRENCH), FR));
        assert!(romance_neighbour(CATALAN_AS_FRENCH));
        // A French analysis excludes it: only the French block's tokens are analysed.
        let blocks = [FRENCH, CATALAN_AS_FRENCH];
        match crate::analysis::pipeline::analyse_document(&blocks, FR, &french_lexicon()) {
            crate::analysis::pipeline::DocumentAnalysis::Analysed(tokens) => {
                assert!(tokens.len() >= MIN_ANALYSABLE_TOKENS);
                assert!(tokens.iter().all(|t| t.block == 0), "{tokens:?}");
            }
            other => panic!("expected the French block analysed, got {other:?}"),
        }
        // Without its French block, the page has nothing to analyse.
        assert_eq!(
            crate::analysis::pipeline::analyse_document(
                &[CATALAN_AS_FRENCH, CATALAN_AS_FRENCH],
                FR,
                &french_lexicon()
            ),
            crate::analysis::pipeline::DocumentAnalysis::NotAnalysable
        );
    }

    #[test]
    fn spec_scenario_an_occitan_paragraph_is_not_french() {
        assert!(!block_is_studied(read_as_french(OCCITAN_AS_FRENCH), FR));
        // The corpus's own: `e`, `degun` and `qual` against no French word; its `Lo` is a capital.
        assert!(!block_is_studied(read_as_french(CORPUS_OCCITAN), FR));
    }

    #[test]
    fn spec_scenario_a_romanian_paragraph_is_not_french() {
        assert!(!block_is_studied(read_as_french(ROMANIAN_AS_FRENCH), FR));
    }

    #[test]
    fn spec_scenario_french_naming_catalan_and_occitan_places_is_kept() {
        let places = "Nous avons visité Sant Joan de les Abadesses, puis Vilafranca del Penedès et Lo Pont de Montvert.";
        for block in [FRENCH, places] {
            assert!(block_is_studied(read_as_french(block), FR), "{block}");
        }
        // `del` is Catalan's, but `nous`, `puis` and `et` are French's.
        assert!(!romance_neighbour(places));
    }

    #[test]
    fn spec_scenario_regional_french_is_kept() {
        for block in [
            // Québécois: `i` is *il* there, and is no marker; `pus` neither.
            "Pis là, i m'a dit qu'y avait pus de place pantoute.",
            // Belgian.
            "Septante personnes sont venues, savez-vous, une fois.",
        ] {
            assert!(block_is_studied(read_as_french(block), FR), "{block}");
        }
    }

    #[test]
    fn spec_scenario_a_capital_makes_a_name() {
        // A neighbour's word written with a capital is a name's; in lowercase, each would be
        // refused (D4: counting capitals refused 47 French blocks of the measurement, not 26).
        for block in [
            "Que signifie \"El Niño\" en espagnol ?",
            "Que signifie \"E pluribus unum\" ?",
            "Quand eurent lieu les émeutes de Los Angeles ?",
        ] {
            assert!(block_is_studied(read_as_french(block), FR), "{block}");
            assert!(romance_neighbour(&block.to_lowercase()), "{block}");
        }
        assert!(!romance_neighbour("Los Angeles"));
        assert!(romance_neighbour("los carrièrs"));
        // French's own words count however they are written.
        assert!(!romance_neighbour("DANS los"));
    }

    #[test]
    fn spec_scenario_a_catalan_page_gives_french_no_vote() {
        // Nothing votes, so the reader's first candidate wins: never French by Catalan's weight.
        let page = [CATALAN_AS_FRENCH, OCCITAN_AS_FRENCH];
        assert_eq!(detect_document_language(&page, &[EN, FR], None), Some(EN));
        // A shorter English line than the two paragraphs outweighs them: they give French no
        // vote, whatever the reader's order.
        let english = "The lighthouse stood at the edge of the cliff.";
        assert!(english.len() < CATALAN_AS_FRENCH.len() + OCCITAN_AS_FRENCH.len());
        let page = [CATALAN_AS_FRENCH, OCCITAN_AS_FRENCH, english];
        assert_eq!(detect_document_language(&page, &[FR, EN], None), Some(EN));
        // A French page still votes French.
        assert_eq!(
            detect_document_language(&[FRENCH, english], &[EN, FR], None),
            Some(FR)
        );
    }

    #[test]
    fn spec_scenario_the_leak_the_french_guard_leaves() {
        // Occitan read as French, without a function word of either table: the guard cannot judge
        // it, and does not refuse it.
        let block = "Soi plan content de te tornar veire.";
        assert!(!romance_neighbour(block));
        assert!(block_is_studied(read_as_french(block), FR));
    }

    #[test]
    fn a_tie_or_no_marker_stays_french() {
        assert!(!romance_neighbour("amb nous"));
        assert!(!romance_neighbour("la casa"));
        assert!(!romance_neighbour(""));
        assert!(romance_neighbour("amb els"));
        // French's own « les uns », « les autres »: `uns` is no marker (D3).
        let uns = "Les uns disent oui, les autres non.";
        assert!(!romance_neighbour(uns));
        assert!(block_is_studied(read_as_french(uns), FR));
    }

    #[test]
    fn a_decomposed_block_is_read_as_composed() {
        let catalan: String = CATALAN_AS_FRENCH.nfd().collect();
        assert_ne!(catalan, CATALAN_AS_FRENCH);
        assert!(romance_neighbour(&catalan));
        // Read with its marks as separators, `été` would be `e` + `te`, and `e` is Occitan's *and*.
        let french: String = "La réunion a été reportée à la semaine prochaine."
            .nfd()
            .collect();
        assert_ne!(french, "La réunion a été reportée à la semaine prochaine.");
        assert!(!romance_neighbour(&french));
        // A decomposed word longer than any marker is composed before its length is judged.
        let long: String = "Vérifiez que la mémoire disponible suffit avant de télécharger."
            .nfd()
            .collect();
        assert!(!romance_neighbour(&long));
    }

    #[test]
    fn the_french_guard_reads_words_as_its_doc_says() {
        let words = |text: &str| {
            let mut words = Vec::new();
            guard_words(text, |w| words.push(w.to_owned()));
            words
        };
        assert_eq!(
            words("L'e-mail des étudiant·e·s, 2e cycle, XIIe — qu'i vient."),
            [
                "L",
                "e-mail",
                "des",
                "étudiant·e·s",
                "cycle",
                "XIIe",
                "qu",
                "i",
                "vient"
            ]
        );
        // A non-breaking hyphen and a full stop join between letters; a joiner at either end, or
        // before a digit, does not (`-e`, `e-`, `e-2` each give `e`), and `1-e` is one run, which
        // holds a digit.
        assert_eq!(
            words("col·lecció e\u{2011}mail U.S.A. -e e- 1-e A4 e-2"),
            ["col·lecció", "e\u{2011}mail", "U.S.A", "e", "e", "e"]
        );
        // A run holding a digit is dropped whole, wherever the digit is.
        assert_eq!(words("1re 2e XIIe x2y"), ["XIIe"]);
        // Combining marks stay in the word they follow.
        assert_eq!(words("e\u{301}te\u{301}"), ["e\u{301}te\u{301}"]);
        assert!(words("").is_empty() && words(" — ").is_empty());
    }

    #[test]
    fn the_french_tables_are_sorted_disjoint_and_short() {
        assert_eq!(ROMANCE_NEIGHBOUR_MARKERS.len(), 152);
        assert_eq!(FRENCH_MARKERS.len(), 69);
        for table in [ROMANCE_NEIGHBOUR_MARKERS, FRENCH_MARKERS] {
            for pair in table.windows(2) {
                assert!(
                    pair[0] < pair[1],
                    "{:?} must sort before {:?}",
                    pair[0],
                    pair[1]
                );
            }
            for word in table {
                // Looked up as the guard writes them: composed and in lowercase.
                assert_eq!(word.nfc().collect::<String>(), *word);
                assert_eq!(word.to_lowercase(), *word);
            }
        }
        for word in ROMANCE_NEIGHBOUR_MARKERS {
            assert!(FRENCH_MARKERS.binary_search(word).is_err(), "{word}");
        }
        let longest = ROMANCE_NEIGHBOUR_MARKERS
            .iter()
            .chain(FRENCH_MARKERS)
            .map(|w| w.len())
            .max();
        assert_eq!(longest, Some(LONGEST_MARKER_BYTES));
        // Left out after measuring (D3): French's own « les uns », Romanian's `ce` and `lui`.
        for word in ["uns", "unes", "es", "on", "i", "mai"] {
            assert!(
                ROMANCE_NEIGHBOUR_MARKERS.binary_search(&word).is_err(),
                "{word}"
            );
        }
        for word in ["ce", "lui", "les", "pas", "qui"] {
            assert!(FRENCH_MARKERS.binary_search(&word).is_err(), "{word}");
        }
    }

    #[test]
    fn whichlang_reads_the_neighbours_as_french() {
        // The guard's reason to exist: without it, each of these is French.
        for block in [
            CATALAN_AS_FRENCH,
            OCCITAN_AS_FRENCH,
            ROMANIAN_AS_FRENCH,
            CORPUS_OCCITAN,
        ] {
            read_as_french(block);
        }
    }

    /// Detection as it was before the French guard: whichlang's answer, a block read as Spanish
    /// that Spanish's guard refuses rewritten as Portuguese, and the candidate of that class.
    fn detect_before(trimmed: &str, languages: &[StudiedLanguage]) -> Option<StudiedLanguage> {
        let mut detected = whichlang::detect_language(trimmed);
        if detected == whichlang::Lang::Spa && iberian_neighbour(trimmed) {
            detected = whichlang::Lang::Por;
        }
        languages
            .iter()
            .copied()
            .find(|l| l.whichlang_target() == detected)
    }

    #[test]
    fn the_french_guard_leaves_english_and_spanish_alone() {
        // `detect` answers English and Spanish as before, for any candidates in any order; French
        // differs only where its guard refuses the block.
        let blocks = [
            ENGLISH,
            SPANISH,
            SPANISH_PROSE,
            CATALAN,
            GALICIAN,
            FRENCH,
            CATALAN_AS_FRENCH,
            OCCITAN_AS_FRENCH,
            ROMANIAN_AS_FRENCH,
            CORPUS_OCCITAN,
            "Il faro si ergeva sul bordo della scogliera, e nessuno ricordava chi l'avesse costruito.",
        ];
        let orders: [&[StudiedLanguage]; 11] = [
            &[EN],
            &[ES],
            &[FR],
            &[EN, ES],
            &[ES, EN],
            &[EN, FR],
            &[FR, EN],
            &[ES, FR],
            &[FR, ES],
            &[EN, ES, FR],
            &[FR, ES, EN],
        ];
        for block in blocks {
            for languages in orders {
                let before = detect_before(block, languages);
                let now = detect(block, languages);
                if before == Some(FR) && romance_neighbour(block) {
                    assert_eq!(now, None, "{block} {languages:?}");
                } else {
                    assert_eq!(now, before, "{block} {languages:?}");
                }
            }
        }
        // A Catalan block read as French answers nothing for a reader of English and Spanish.
        assert_eq!(detect(CATALAN_AS_FRENCH, &[EN, ES]), None);
        assert_eq!(detect(CATALAN_AS_FRENCH, &[EN, ES, FR]), None);
        assert_eq!(detect(SPANISH_PROSE, &[EN, ES, FR]), Some(ES));
        assert_eq!(detect(CATALAN, &[EN, ES, FR]), None);
        assert_eq!(detect(ENGLISH, &[FR, ES, EN]), Some(EN));
        assert_eq!(detect(FRENCH, &[EN, ES]), None);
    }

    #[test]
    fn tiny_blocks_are_excluded_not_guessed() {
        assert!(!block_is_studied("OK", StudiedLanguage::English));
        assert!(!block_is_studied("  the  ", StudiedLanguage::English));
    }
}
