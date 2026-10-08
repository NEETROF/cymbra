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

pub mod function_words;
pub mod language;
pub mod lemmatize;
pub mod lexicon;
pub mod percent;
pub mod pipeline;
mod spanish;
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
pub const ANALYZER_VERSION: &str = "1.1.0";

/// Spanish's analyser version: `1.0.0` when add-lingua-spanish-analysis wrote
/// its pre-pass (NFC, `al`/`del`) and its cascade (old spellings, enclitics, the
/// plural fallback), replacing the `0.1.0` baseline; `1.1.0` since
/// add-lingua-spanish-detection-guard keeps Catalan and Galician blocks out;
/// `1.2.0` since add-lingua-spanish-names sets a document's names aside.
pub const SPANISH_ANALYZER_VERSION: &str = "1.2.0";

/// French's analyser version: `0.1.0` while French is served by the baseline
/// analysis (add-lingua-french-baseline) — the rules that belong to no
/// language and the pack's forms, no elision or contraction split, no NFC, no
/// cascade, no function words, no names rule. A `0.x` version, as a language
/// served by the baseline carries; its own tokenisation and cascade bump it.
pub const FRENCH_ANALYZER_VERSION: &str = "0.1.0";
