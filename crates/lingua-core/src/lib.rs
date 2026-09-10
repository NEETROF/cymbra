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

//! Cymbra Lingua core.
//!
//! Deterministic analysis pipeline for the Lingua product family: text →
//! tokens → lemmas → classification → known-token percentage. Every consumer
//! (browser extension, Apple container app, agent plugin) links this same
//! crate, natively or through WASM, and MUST observe identical outputs at an
//! equal [`analysis::ANALYZER_VERSION`] and equal data pack — determinism is a
//! contract, not an aspiration (spec `lingua-analysis`).
//!
//! Module layout is laid down for the whole change stack; later changes fill
//! their slot without moving code:
//! - [`analysis`] — this change (`add-lingua-analysis`)
//! - [`knowledge`] — statuses, calibration, L1/L2 profile (`add-lingua-knowledge-model`)
//! - [`decks`] — cards, FSRS review, backup/restore (`add-lingua-decks-review`)
//! - [`packs`] — pack container format and reader (`add-lingua-data-pack`)

pub mod analysis;
pub mod decks;
pub mod knowledge;
pub mod packs;
