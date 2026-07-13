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

//! Resumable on-disk crawl state.
//!
//! Records the item ids already completed (so `--resume` skips them without
//! re-downloading) and the SHA-256 content hashes already seen (so dedup
//! survives restarts and can seed the orchestrator via
//! [`crate::crawl::Orchestrator::with_seen`]). Persisted as JSON.

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// The durable crawl state.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrawlState {
    /// Item ids (`source:source_item_id`) fully processed.
    #[serde(default)]
    pub completed: HashSet<String>,
    /// Canonical content hashes already ingested (dedup).
    #[serde(default)]
    pub seen_hashes: HashSet<String>,
}

impl CrawlState {
    /// Loads state from `path`, returning an empty state if the file is absent.
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .with_context(|| format!("parsing crawl state {}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("reading crawl state {}", path.display())),
        }
    }

    /// Writes state to `path` (creating parent dirs).
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating state dir {}", parent.display()))?;
        }
        let json = serde_json::to_vec_pretty(self).context("serialising crawl state")?;
        std::fs::write(path, json)
            .with_context(|| format!("writing crawl state {}", path.display()))
    }

    /// Whether `item_id` was already completed in a prior run.
    pub fn is_completed(&self, item_id: &str) -> bool {
        self.completed.contains(item_id)
    }

    /// Marks `item_id` completed.
    pub fn mark_completed(&mut self, item_id: impl Into<String>) {
        self.completed.insert(item_id.into());
    }

    /// Records a content hash; returns `true` if it was newly seen.
    pub fn mark_seen(&mut self, hash: impl Into<String>) -> bool {
        self.seen_hashes.insert(hash.into())
    }

    /// A copy of the seen-hash set, e.g. to seed the orchestrator on resume.
    pub fn seen_hashes(&self) -> HashSet<String> {
        self.seen_hashes.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique temp path per test (no external tempfile dep).
    fn temp_path(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("score_crawler_state_{tag}.json"))
    }

    #[test]
    fn missing_file_loads_empty() {
        let p = temp_path("missing_unique_abc");
        let _ = std::fs::remove_file(&p);
        let s = CrawlState::load(&p).unwrap();
        assert!(s.completed.is_empty() && s.seen_hashes.is_empty());
    }

    #[test]
    fn round_trips_through_disk() {
        let p = temp_path("roundtrip_unique_def");
        let _ = std::fs::remove_file(&p);

        let mut s = CrawlState::default();
        s.mark_completed("openscore:1");
        assert!(s.mark_seen("hashA"));
        assert!(!s.mark_seen("hashA")); // already seen
        s.save(&p).unwrap();

        let loaded = CrawlState::load(&p).unwrap();
        assert!(loaded.is_completed("openscore:1"));
        assert!(!loaded.is_completed("openscore:2"));
        assert!(loaded.seen_hashes().contains("hashA"));
        assert_eq!(loaded, s);

        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn corrupt_file_is_an_error() {
        let p = temp_path("corrupt_unique_ghi");
        std::fs::write(&p, b"not json").unwrap();
        assert!(CrawlState::load(&p).is_err());
        let _ = std::fs::remove_file(&p);
    }
}
