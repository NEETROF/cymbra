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

//! Data packs — the versioned pair-keyed (studied → native) container format
//! and its reader.
//!
//! Filled by the `add-lingua-data-pack` change (spec `lingua-data-packs`);
//! this slot only reserves the module layout. The analysis pipeline already
//! consumes pack *contents* (the forms→lemmas FST) through
//! [`crate::analysis::lexicon`], which reads from plain byte slices so the
//! container can stay `include_bytes!`-compatible.
