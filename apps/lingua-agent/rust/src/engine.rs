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

//! The analysis brain, compiled natively — the same `lingua-core` (same
//! `analyzer_version`) as the extension. Loads the data pack and analyses text into
//! classified tokens; paths resolve under `~/.lingua/` (overridable by env for tests).

use std::path::PathBuf;

use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::engine::{PageAnalysis, analyse_page};
use lingua_core::knowledge::state::KnowledgeState;
use lingua_core::packs::Pack;

/// The MVP studies English only.
pub const EN: StudiedLanguage = StudiedLanguage::English;

/// The plugin's data directory: `$LINGUA_HOME`, else `$HOME/.lingua`.
pub fn lingua_home() -> PathBuf {
    if let Ok(dir) = std::env::var("LINGUA_HOME") {
        return PathBuf::from(dir);
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".lingua")
}

/// The data pack path: `$LINGUA_PACK`, else `<lingua_home>/pack.lingua`.
pub fn pack_path() -> PathBuf {
    std::env::var("LINGUA_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|_| lingua_home().join("pack.lingua"))
}

/// Loads the pack, or `None` when it is absent/unreadable/incompatible (degrade).
pub fn load_pack() -> Option<Pack> {
    let bytes = std::fs::read(pack_path()).ok()?;
    Pack::load(&bytes).ok()
}

/// Analyses a block of text (a reply) against the pack and knowledge state. Splits the
/// text into line blocks; short/non-English input analyses to `analysable: false`.
pub fn analyse(pack: &Pack, knowledge: &KnowledgeState, text: &str) -> PageAnalysis {
    let blocks: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    analyse_page(&blocks, EN, pack, knowledge)
}
