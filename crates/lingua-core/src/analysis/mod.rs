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

pub mod language;
pub mod lemmatize;
pub mod lexicon;
pub mod percent;
pub mod pipeline;
pub mod tokenize;

/// Version of the analysis pipeline.
///
/// Bump on ANY change that can alter an output (tokenisation rule, cascade
/// order, irregulars table, language-gating thresholds…). Profiles, synced
/// counts and the future community TextProfiles are only comparable at an
/// equal version — a silent behavioural drift here corrupts every count built
/// on top (design D5).
pub const ANALYZER_VERSION: &str = "1.0.0";
