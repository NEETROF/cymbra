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

//! Data packs — the versioned pair-keyed (studied → native) container and its
//! reader (spec `lingua-data-packs`).
//!
//! A `pack.lingua` bundles, for one language pair: metadata (the pair, the
//! `pack_version`, the compatible `analyzer_version`, licences), a
//! form→lemma FST + its lemma pool, a frequency table (ranks keyed by lemma
//! id), zstd-compressed French glosses (offset-indexed by lemma id), and a
//! NOTICE. This module is the **reader** only: it decodes the container from
//! an `include_bytes!`-compatible byte slice, refuses a pack built for an
//! incompatible analyser generation, and exposes the pack's contents to the
//! analysis and knowledge layers. Gloss decompression uses the pure-Rust
//! `ruzstd`, so the reader stays WASM-clean. The offline *builder* lives in
//! the native `lingua-pack` crate; [`format::write_container`] is the shared
//! low-level writer both it and the tests use.

pub mod format;
pub mod meta;
pub mod pack;

pub use format::{FormatError, Section, read_container, write_container};
pub use meta::PackMeta;
pub use pack::{Pack, PackError};
