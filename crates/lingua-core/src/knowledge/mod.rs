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

//! Knowledge model (spec `lingua-knowledge-model`).
//!
//! What the learner knows, per `(studied language, lemma)`: explicit
//! statuses with provenance, implicit "known" below a frequency-rank
//! calibration, multi-candidate resolution in the learner's favour, the
//! L1/L2 profile, frequency-rank calibration for the cold start, and
//! exposure counters.
//!
//! This is the type vocabulary every later surface shares; keeping it in the
//! core (not in each shell) is what will make merging the local stores
//! mechanical at the sync change. Everything here is pure and free of a
//! clock — timestamps are passed in — so it stays host-testable and
//! WASM-identical.

pub mod exposure;
pub mod level;
pub mod profile;
pub mod state;
pub mod status;

pub use exposure::{Exposure, ExposureCounters};
pub use level::{CefrLevel, CefrLevels};
pub use profile::{LanguagePair, NativeLanguage, Profile};
pub use state::{BandStats, FrequencyRanks, KnowledgeState, MapFrequencyRanks};
pub use status::{KnownSource, Status};
