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

//! Text-analysis pipeline (spec `lingua-analysis`).
//!
//! text → [`tokenize`] → [`lemmatize`] → classification (supplied by the
//! caller until `add-lingua-knowledge-model`) → [`percent`]. Language gating
//! happens per block ([`language`]); [`pipeline`] orchestrates a whole
//! document.
//!
//! Everything here is pure, allocation-light and free of platform
//! dependencies: the same code must run natively and under WASM with
//! byte-for-byte identical outputs at an equal [`ANALYZER_VERSION`] and equal
//! pack. Keep outputs order-stable (no iteration over unordered maps ends up
//! in a result).

mod french;
pub mod function_words;
mod italian;
pub mod language;
pub mod lemmatize;
pub mod lexicon;
pub mod percent;
pub mod pipeline;
mod spanish;

pub(crate) use spanish::carries_enclitic;
pub mod tokenize;

/// Version of the analysis pipeline.
///
/// Bump on ANY change that can alter an output (tokenisation rule, cascade
/// order, irregulars table, language-gating thresholds…). Profiles, synced
/// counts and the future community TextProfiles are only comparable at an
/// equal version — a silent behavioural drift here corrupts every count built
/// on top (design D5).
///
/// This is ENGLISH's version: each studied language has its own
/// ([`language::StudiedLanguage::analyzer_version`]), so a change to one
/// language's rules never moves another's (generalise-lingua-analysis-by-language).
/// `1.2.0` since ignore-lingua-soft-hyphens reads a word without its soft hyphens
/// (U+00AD), a rule every studied language shares; `1.3.0` since
/// leave-lingua-names-out-of-english-and-spanish-words sets an English document's
/// names aside (`engine::document_names`).
pub const ANALYZER_VERSION: &str = "1.3.0";

/// Spanish's analyser version: `1.0.0` when add-lingua-spanish-analysis wrote
/// its pre-pass (NFC, `al`/`del`) and its cascade (old spellings, enclitics, the
/// plural fallback), replacing the `0.1.0` baseline; `1.1.0` since
/// add-lingua-spanish-detection-guard keeps Catalan and Galician blocks out;
/// `1.2.0` since add-lingua-spanish-names sets a document's names aside;
/// `1.3.0` since add-lingua-spanish-occitan-guard keeps Occitan blocks out;
/// `1.4.0` since refine-lingua-spanish-galician-markers keeps Spanish's *da* and
/// *das* from reading as Galician; `1.5.0` since ignore-lingua-soft-hyphens
/// reads a word without its soft hyphens.
pub const SPANISH_ANALYZER_VERSION: &str = "1.5.0";

/// French's analyser version: `0.1.0` when French was served by the baseline
/// analysis alone (add-lingua-french-baseline); `0.2.0` since
/// add-lingua-french-tokenisation gives it its own tokenisation pre-pass — the
/// narrow no-break space (U+202F) a space, an elided word split from the word it
/// is joined to and read as the word it stands for, each piece with its own span
/// (`l'homme` → `le` + `homme`), `au`/`aux` → `à` + `le`/`les`, `du`/`des`
/// whole, a hyphenated inversion read as words (`dit-il` → `dit` + `il`);
/// `1.0.0` since add-lingua-french-analysis gives it its own analysis, leaving
/// the baseline's `0.x` versions: its pre-pass reads every word in NFC, its
/// cascade reads the pack's forms then an unlisted lowercase plural as its
/// unlisted singular (`analysis/french.rs`), its six closed classes flag a phrase gloss's
/// function words, « pas » among them, and a French document's names are set
/// aside (`engine::document_names`); `1.1.0` since add-lingua-french-detection-guard
/// keeps Catalan, Occitan and Romanian blocks out (`language::romance_neighbour`);
/// `1.2.0` since ignore-lingua-soft-hyphens reads a word without its soft hyphens;
/// `1.3.0` since keep-lingua-french-acronyms-in-capitals reads a word standing alone
/// in capitals as the acronym its pack lists.
pub const FRENCH_ANALYZER_VERSION: &str = "1.3.0";

/// Italian's analyser version: `0.1.0` while Italian was served by the baseline analysis alone
/// (add-lingua-italian-baseline), a `0.x` version as the spec requires of a language whose rules
/// are not written. The baseline is the rules that belong to no language — segmentation, the
/// hyphen run, the compound rule, the digit drop, the edge-apostrophe trim, the single-letter
/// rule — and the pack's form→lemma lookup, else the lowercased form. `0.2.0` since
/// add-lingua-italian-tokenisation gives it its own tokenisation pre-pass (I7): an elided word
/// split from the word it is joined to and read as the word it stands for, each piece with its
/// own span (`l'uomo` → `lo` + `uomo`, `c'è` → `ci` + `è`); the articulated prepositions of
/// `di` whole (`del`, `della`; `dell'` read as `dello`), the others split into the preposition
/// and the article sharing the span (`nel` → `in` + `il`, `all'` → `a` + `lo`); a truncation
/// read as its word, its apostrophe in its span (`po'` → `poco`). `1.0.0` since
/// add-lingua-italian-analysis gives it its own analysis, leaving the baseline's `0.x` versions:
/// its pre-pass reads every word in NFC; its cascade (`analysis/italian.rs`) reads the pack's
/// forms, then a verb's clitic pronouns off an unlisted form (the enclitic rule, a regular past
/// participle among its bases: `dirmelo` → `dire`, `dammi` → `dare`, `eccolo` → `ecco`,
/// `sviluppatasi` → `sviluppare`), then the form — there is no plural rule, the one the design
/// proposed having been measured out (its D6); its six closed classes flag a phrase gloss's function
/// words, `non` among them; and an Italian document's names are set aside
/// (`engine::document_names`). `1.1.0` since add-lingua-italian-detection-guard: a block whichlang
/// reads as Italian is not Italian when the function words of its neighbours — Catalan, Galician,
/// Venetian, Sicilian, Neapolitan, Sardinian, Corsican, Friulian, Latin, Occitan, Romanian —
/// outnumber its Italian ones (`language::italic_neighbour`), and such a block is no longer counted.
/// `1.2.0` since read-lingua-italian-unmarked-spellings reads an apostrophe that stands for an
/// accent as the accented word the pack lists, one token spanning the apostrophe (`perche'` →
/// `perché`, `E'` → `È`, `c'e'` → `ci` + `è`), change 6's truncations first (`da'` → `dai`),
/// never on sixteen literary apocopes (`ne'`, `de'`, `que'`, `ca'` …), never for a form whose
/// word the pack ranks beyond 23,000 (`vita'` stays `vita`, not the rare `vità`), the commoner of
/// two accents the pack ranks (`se'` → `sé`), and no closing quote read in a truncation's or an
/// accent spelling's apostrophe (`tokenize::italian_accented`, `tokenize::inside_a_quote`).
/// `1.3.0` since keep-lingua-italian-acronyms-and-names reads a word standing alone in capitals
/// as the acronym its pack lists, French's rule opened to Italian (`NATO` in « della NATO », not
/// *nascere*; `pipeline::alone_in_capitals`). What it still lacks, a rule of Italian's own that
/// bumps this version when it lands: no expression keys of its own.
pub const ITALIAN_ANALYZER_VERSION: &str = "1.3.0";
