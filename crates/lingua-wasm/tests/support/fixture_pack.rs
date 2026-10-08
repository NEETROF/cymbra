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

//! The fixture pack, and the same pack with its metadata rewritten to study another language
//! glossed in another native language (generalise-lingua-native-language): the analyser only reads
//! the forms, so English ones serve here. Shared by `languages_wasm.rs` (wasm) and `reprofile.rs`
//! (host), which include it by path: `support/mod.rs` reads files and serves the host baselines.

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::packs::{PackMeta, read_container, write_container};

pub const PACK: &[u8] = include_bytes!("../fixtures/pack.lingua");

/// The fixture pack, its metadata rewritten to study `studied` glossed in `native`.
pub fn rewritten(studied: StudiedLanguage, native: &str) -> Vec<u8> {
    let (meta, sections) = read_container(PACK).unwrap();
    let mut meta: PackMeta = serde_json::from_slice(&meta).unwrap();
    meta.studied = studied.tag().into();
    meta.analyzer_version = studied.analyzer_version().into();
    meta.native = native.into();
    let sections: Vec<(&str, &[u8])> = sections
        .iter()
        .map(|s| (s.name.as_str(), s.data.as_slice()))
        .collect();
    write_container(&serde_json::to_vec(&meta).unwrap(), &sections)
}
