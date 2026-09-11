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

//! The pack-registry manifest the Lingua ops console serves (change: add-lingua-back-
//! office): the value type plus the pure merge / staleness logic. Host-tested here; the
//! thin CLI that does the file I/O and builds the packs is
//! `src/bin/lingua-pack-manifest.rs`.

use serde::{Deserialize, Serialize};

/// One published pack's registry entry.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct PackManifestEntry {
    pub studied: String,
    pub native: String,
    pub pack_version: String,
    pub analyzer_version: String,
    pub built_at: String,
    pub size_bytes: i64,
    pub notice: String,
}

impl PackManifestEntry {
    /// Everything but the build date — what "the pack has not changed" means.
    pub fn same_content(&self, o: &PackManifestEntry) -> bool {
        self.studied == o.studied
            && self.native == o.native
            && self.pack_version == o.pack_version
            && self.analyzer_version == o.analyzer_version
            && self.size_bytes == o.size_bytes
            && self.notice == o.notice
    }
}

/// The committed registry document.
#[derive(Serialize, Deserialize, Default)]
pub struct PackManifest {
    pub packs: Vec<PackManifestEntry>,
}

/// Merge freshly-built entries with the committed manifest: preserve each entry's
/// recorded `built_at` when its content is unchanged (so a reproducible rebuild diffs
/// empty), then order deterministically by pair.
pub fn merge(
    mut fresh: Vec<PackManifestEntry>,
    committed: &PackManifest,
) -> Vec<PackManifestEntry> {
    for e in &mut fresh {
        if let Some(prev) = committed.packs.iter().find(|p| e.same_content(p)) {
            e.built_at = prev.built_at.clone();
        }
    }
    fresh.sort_by(|a, b| (&a.studied, &a.native).cmp(&(&b.studied, &b.native)));
    fresh
}

/// Whether the committed manifest is stale relative to freshly-built entries — the
/// content differs (build date ignored), so `emit-manifest` must be re-run and committed.
pub fn is_stale(fresh: &[PackManifestEntry], committed: &PackManifest) -> bool {
    fresh.len() != committed.packs.len()
        || !fresh
            .iter()
            .all(|e| committed.packs.iter().any(|c| e.same_content(c)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(version: &str, size: i64, built_at: &str) -> PackManifestEntry {
        PackManifestEntry {
            studied: "en".into(),
            native: "fr".into(),
            pack_version: version.into(),
            analyzer_version: "1.0.0".into(),
            built_at: built_at.into(),
            size_bytes: size,
            notice: "AGID; wordfreq; kaikki.".into(),
        }
    }

    #[test]
    fn same_content_ignores_the_build_date() {
        assert!(entry("1.0.0", 100, "2026-01-01").same_content(&entry("1.0.0", 100, "2026-09-11")));
        assert!(!entry("1.0.0", 100, "x").same_content(&entry("1.0.0", 101, "x"))); // size differs
        assert!(!entry("1.0.0", 100, "x").same_content(&entry("2.0.0", 100, "x"))); // version differs
    }

    #[test]
    fn merge_preserves_built_at_for_unchanged_packs() {
        let committed = PackManifest {
            packs: vec![entry("1.0.0", 100, "2026-01-01")],
        };
        // A fresh build with the same content but a newer stamp keeps the old date.
        let merged = merge(vec![entry("1.0.0", 100, "2026-09-11")], &committed);
        assert_eq!(merged[0].built_at, "2026-01-01");
    }

    #[test]
    fn merge_uses_the_fresh_date_when_content_changed() {
        let committed = PackManifest {
            packs: vec![entry("1.0.0", 100, "2026-01-01")],
        };
        // A different size ⇒ the pack changed ⇒ the new stamp wins.
        let merged = merge(vec![entry("1.0.0", 200, "2026-09-11")], &committed);
        assert_eq!(merged[0].built_at, "2026-09-11");
    }

    #[test]
    fn merge_orders_by_pair() {
        let fresh = vec![
            PackManifestEntry {
                native: "es".into(),
                ..entry("1", 1, "d")
            },
            PackManifestEntry {
                native: "de".into(),
                ..entry("1", 1, "d")
            },
        ];
        let merged = merge(fresh, &PackManifest::default());
        assert_eq!(merged[0].native, "de");
        assert_eq!(merged[1].native, "es");
    }

    #[test]
    fn is_stale_ignores_the_build_date_but_catches_content_and_count() {
        let committed = PackManifest {
            packs: vec![entry("1.0.0", 100, "2026-01-01")],
        };
        // Same content, different date ⇒ not stale.
        assert!(!is_stale(&[entry("1.0.0", 100, "2026-09-11")], &committed));
        // Changed size ⇒ stale.
        assert!(is_stale(&[entry("1.0.0", 200, "2026-09-11")], &committed));
        // Extra pack ⇒ stale (count differs).
        assert!(is_stale(
            &[entry("1.0.0", 100, "d"), entry("2.0.0", 50, "d")],
            &committed
        ));
    }
}
