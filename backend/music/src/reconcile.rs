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

//! Corpus↔catalog reconciliation: find the corpus objects no `catalog_scores`
//! row references, and (only when asked) move them out of the way.
//!
//! Why this exists: object keys used to embed a per-run UUID, so every re-crawl
//! of unchanged content wrote a *second* object while ingest deduplicated the
//! row away. Production ended up with ~145 430 unreferenced objects — about half
//! the corpus, mirrored to S3 as well (change: fix-crawler-corpus-isolation).
//! Keys are content-derived now, so the leak is closed; this cleans up what it
//! already produced.
//!
//! Safety is the point of this module, not throughput. A correct run removes
//! roughly half the corpus, so a wrong one is a catastrophe: the reference set is
//! read first and the run aborts if it looks implausible, and removal is a
//! *quarantine move*, reversible until a separate purge.

use std::collections::BTreeSet;

use anyhow::{Context, Result};
use async_trait::async_trait;
use cymbra_storage::ObjectStorage;

/// The prefixes that hold servable corpus objects — the same allow-list the S3
/// mirror uses.
///
/// `user-scores/` is included, but note what that demands of the [`ReconcileRepo`]
/// implementation: its objects are referenced by `user_scores`, **not** by
/// `catalog_scores`. Any prefix listed here must have every table that references
/// it covered by the repo's query, or its objects read as orphans.
pub const CORPUS_PREFIXES: [&str; 3] = ["safe/", "low_confidence/", "user-scores/"];

/// Where quarantined objects are moved. Outside every corpus prefix, so a
/// quarantined object is no longer mirrored and no longer looks servable.
pub const QUARANTINE_PREFIX: &str = "quarantine/";

/// Reads the set of object keys the catalog references.
///
/// Paged like [`crate::backfill::TitleBackfillRepo`] so a 150k-row catalog is
/// streamed rather than materialised in one query. The Postgres implementation
/// is [`crate::pg::PgReconcileRepo`]; a fake backs the tests here.
#[async_trait]
pub trait ReconcileRepo: Send + Sync {
    /// One page of `(id, object_key)` ordered by `id`, with `id > after`
    /// (keyset paging; empty `after` → from the start), up to `limit` rows.
    async fn page_object_keys(&self, after: &str, limit: i64) -> Result<Vec<(String, String)>>;
}

/// How a reconciliation run is bounded.
#[derive(Debug, Clone)]
pub struct ReconcileOptions {
    /// Perform the quarantine moves. Without it nothing is written.
    pub apply: bool,
    /// Abort if the share of objects to remove exceeds this fraction of the
    /// corpus. Guards against a mis-scoped or partially-failed reference read.
    pub max_removal_ratio: f64,
    /// Catalog rows per page.
    pub page_size: i64,
}

impl Default for ReconcileOptions {
    fn default() -> Self {
        Self {
            apply: false,
            // The known backlog is ~50% of the corpus, so the default has to sit
            // above it while still catching "the query returned almost nothing".
            max_removal_ratio: 0.75,
            page_size: 500,
        }
    }
}

/// What a run found (and did, when applying).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReconcileReport {
    pub objects_seen: usize,
    pub referenced: usize,
    /// Keys present in the store that no row references.
    pub unreferenced: Vec<String>,
    /// How many were actually moved to quarantine (0 on a dry run).
    pub quarantined: usize,
}

/// Finds unreferenced corpus objects and, when `opts.apply`, quarantines them.
///
/// Reasons over the *set* of referenced keys rather than row by row: an object
/// referenced by any row must survive, whatever happened to other rows.
pub async fn run_reconcile(
    store: &dyn ObjectStorage,
    repo: &dyn ReconcileRepo,
    opts: &ReconcileOptions,
) -> Result<ReconcileReport> {
    // 1. The reference set, first — everything downstream is a subtraction from it.
    let referenced = referenced_keys(repo, opts.page_size).await?;
    anyhow::ensure!(
        !referenced.is_empty(),
        "the catalog reported zero referenced object keys — refusing to treat the \
         whole corpus as unreferenced; check the database connection and retry"
    );

    // 2. Everything the store holds under the servable prefixes.
    //
    // Per prefix, because a prefix whose objects are *entirely* unreferenced is
    // the signature of a missing reference source, not of a prefix full of
    // garbage. That is exactly how all six user uploads were quarantined in
    // production: `user-scores/` was scanned while its keys live in a table the
    // repo did not read. Aborting here turns that class of mistake into a loud
    // failure instead of silent data loss.
    let mut objects = BTreeSet::new();
    for prefix in CORPUS_PREFIXES {
        let found: Vec<String> = store
            .list(prefix)
            .await
            .with_context(|| format!("listing corpus objects under {prefix}"))?;
        if !found.is_empty() && !found.iter().any(|k| referenced.contains(k)) {
            anyhow::bail!(
                "prefix {prefix} holds {} objects and NOT ONE is referenced — refusing to \
                 continue: a whole prefix being orphaned almost always means a table that \
                 references these objects is missing from the reference query, not that \
                 every object is garbage",
                found.len()
            );
        }
        objects.extend(found);
    }

    let unreferenced: Vec<String> = objects
        .iter()
        .filter(|k| !referenced.contains(*k))
        .cloned()
        .collect();

    let mut report = ReconcileReport {
        objects_seen: objects.len(),
        referenced: referenced.len(),
        unreferenced,
        quarantined: 0,
    };

    if !opts.apply || report.unreferenced.is_empty() {
        return Ok(report);
    }

    // 3. Ratio guard, checked only when about to write.
    let ratio = report.unreferenced.len() as f64 / report.objects_seen.max(1) as f64;
    anyhow::ensure!(
        ratio <= opts.max_removal_ratio,
        "would remove {} of {} objects ({:.1}%), above the {:.1}% safety threshold — \
         re-run with a higher --max-removal-ratio only if that is genuinely expected",
        report.unreferenced.len(),
        report.objects_seen,
        ratio * 100.0,
        opts.max_removal_ratio * 100.0
    );

    // 4. Quarantine: copy aside, then remove. Reversible until purged.
    for key in &report.unreferenced {
        let bytes = store
            .get(key)
            .await
            .with_context(|| format!("reading {key} before quarantine"))?;
        store
            .put(&quarantine_key(key), bytes)
            .await
            .with_context(|| format!("quarantining {key}"))?;
        store
            .delete(key)
            .await
            .with_context(|| format!("removing {key} after quarantine"))?;
        report.quarantined += 1;
    }
    Ok(report)
}

/// The quarantine location for a corpus key, preserving its shape so a restore is
/// a straight prefix strip.
pub fn quarantine_key(key: &str) -> String {
    format!("{QUARANTINE_PREFIX}{key}")
}

/// Pages the catalog and collects every referenced object key.
async fn referenced_keys(repo: &dyn ReconcileRepo, page_size: i64) -> Result<BTreeSet<String>> {
    let mut keys = BTreeSet::new();
    let mut after = String::new();
    loop {
        let page = repo
            .page_object_keys(&after, page_size)
            .await
            .context("reading a page of catalog object keys")?;
        let Some((last_id, _)) = page.last().cloned() else {
            break;
        };
        for (_, key) in page {
            keys.insert(key);
        }
        after = last_id;
    }
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_storage::FakeStore;

    /// Catalog rows keyed by id, in id order.
    #[derive(Default)]
    struct FakeRepo {
        rows: Vec<(String, String)>,
    }

    #[async_trait]
    impl ReconcileRepo for FakeRepo {
        async fn page_object_keys(&self, after: &str, limit: i64) -> Result<Vec<(String, String)>> {
            Ok(self
                .rows
                .iter()
                .filter(|(id, _)| id.as_str() > after)
                .take(limit as usize)
                .cloned()
                .collect())
        }
    }

    fn repo(rows: &[(&str, &str)]) -> FakeRepo {
        FakeRepo {
            rows: rows
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
        }
    }

    async fn store_with(keys: &[&str]) -> FakeStore {
        let s = FakeStore::default();
        for k in keys {
            s.put(k, b"bytes".to_vec()).await.unwrap();
        }
        s
    }

    #[tokio::test]
    async fn dry_run_reports_without_writing() {
        let store = store_with(&["safe/aa/kept.mxl", "safe/bb/orphan.mxl"]).await;
        let repo = repo(&[("id1", "safe/aa/kept.mxl")]);

        let r = run_reconcile(&store, &repo, &ReconcileOptions::default())
            .await
            .unwrap();

        assert_eq!(r.unreferenced, vec!["safe/bb/orphan.mxl".to_string()]);
        assert_eq!(r.quarantined, 0);
        assert_eq!(store.len(), 2, "a dry run must not remove anything");
        assert!(store.contains("safe/bb/orphan.mxl"));
    }

    #[tokio::test]
    async fn apply_quarantines_only_the_unreferenced() {
        let store = store_with(&["safe/aa/kept.mxl", "safe/bb/orphan.mxl"]).await;
        let repo = repo(&[("id1", "safe/aa/kept.mxl")]);
        let opts = ReconcileOptions {
            apply: true,
            ..Default::default()
        };

        let r = run_reconcile(&store, &repo, &opts).await.unwrap();

        assert_eq!(r.quarantined, 1);
        assert!(
            store.contains("safe/aa/kept.mxl"),
            "referenced must survive"
        );
        assert!(!store.contains("safe/bb/orphan.mxl"));
        assert!(
            store.contains("quarantine/safe/bb/orphan.mxl"),
            "removal must be reversible until purged"
        );
    }

    #[tokio::test]
    async fn an_object_referenced_by_any_row_survives() {
        // Two rows point at the same object (one is later deleted in spirit):
        // set-based reasoning must keep it whatever happens to individual rows.
        let store = store_with(&["safe/aa/shared.mxl"]).await;
        let repo = repo(&[("id1", "safe/aa/shared.mxl"), ("id2", "safe/aa/shared.mxl")]);
        let opts = ReconcileOptions {
            apply: true,
            ..Default::default()
        };

        let r = run_reconcile(&store, &repo, &opts).await.unwrap();

        assert!(r.unreferenced.is_empty());
        assert!(store.contains("safe/aa/shared.mxl"));
    }

    #[tokio::test]
    async fn empty_reference_set_aborts() {
        let store = store_with(&["safe/aa/a.mxl"]).await;
        let opts = ReconcileOptions {
            apply: true,
            ..Default::default()
        };

        let err = run_reconcile(&store, &repo(&[]), &opts).await.unwrap_err();

        assert!(err.to_string().contains("zero referenced object keys"));
        assert!(store.contains("safe/aa/a.mxl"), "nothing may be removed");
    }

    #[tokio::test]
    async fn over_threshold_aborts_without_writing() {
        // 3 of 4 unreferenced = 75%, above a 50% threshold.
        let store = store_with(&[
            "safe/aa/kept.mxl",
            "safe/bb/o1.mxl",
            "safe/cc/o2.mxl",
            "safe/dd/o3.mxl",
        ])
        .await;
        let repo = repo(&[("id1", "safe/aa/kept.mxl")]);
        let opts = ReconcileOptions {
            apply: true,
            max_removal_ratio: 0.5,
            ..Default::default()
        };

        let err = run_reconcile(&store, &repo, &opts).await.unwrap_err();

        assert!(err.to_string().contains("safety threshold"), "{err}");
        assert_eq!(store.len(), 4, "the guard must fire before any write");
    }

    /// The production incident of 2026-08-16, as a test: `user-scores/` objects
    /// are referenced by a different table, so a repo that reads only the catalog
    /// makes the whole prefix look orphaned. That must abort, not delete.
    #[tokio::test]
    async fn a_wholly_unreferenced_prefix_aborts() {
        let store = store_with(&[
            "safe/aa/kept.mxl",
            "user-scores/u1/a.musicxml",
            "user-scores/u1/b.musicxml",
        ])
        .await;
        // Catalog rows only — the user_scores table is missing from the query.
        let repo = repo(&[("id1", "safe/aa/kept.mxl")]);
        let opts = ReconcileOptions {
            apply: true,
            ..Default::default()
        };

        let err = run_reconcile(&store, &repo, &opts).await.unwrap_err();

        assert!(err.to_string().contains("NOT ONE is referenced"), "{err}");
        assert!(store.contains("user-scores/u1/a.musicxml"));
        assert!(store.contains("user-scores/u1/b.musicxml"));
    }

    /// A prefix with a mix of referenced and unreferenced objects is normal work,
    /// not a missing reference source.
    #[tokio::test]
    async fn a_partially_referenced_prefix_proceeds() {
        let store = store_with(&[
            "user-scores/u1/kept.musicxml",
            "user-scores/u1/gone.musicxml",
        ])
        .await;
        let repo = repo(&[("id1", "user-scores/u1/kept.musicxml")]);
        let opts = ReconcileOptions {
            apply: true,
            ..Default::default()
        };

        let r = run_reconcile(&store, &repo, &opts).await.unwrap();

        assert_eq!(r.quarantined, 1);
        assert!(store.contains("user-scores/u1/kept.musicxml"));
        assert!(!store.contains("user-scores/u1/gone.musicxml"));
    }

    #[tokio::test]
    async fn objects_outside_the_corpus_prefixes_are_ignored() {
        // A work artefact that somehow landed in the store is not the
        // reconciler's business — it only reasons about servable prefixes.
        let store = store_with(&["safe/aa/kept.mxl", "manifest.json"]).await;
        let repo = repo(&[("id1", "safe/aa/kept.mxl")]);

        let r = run_reconcile(&store, &repo, &ReconcileOptions::default())
            .await
            .unwrap();

        assert_eq!(r.objects_seen, 1);
        assert!(r.unreferenced.is_empty());
    }

    #[tokio::test]
    async fn paging_collects_every_reference() {
        let store = store_with(&["safe/a/1.mxl", "safe/b/2.mxl", "safe/c/3.mxl"]).await;
        let repo = repo(&[
            ("id1", "safe/a/1.mxl"),
            ("id2", "safe/b/2.mxl"),
            ("id3", "safe/c/3.mxl"),
        ]);
        // Page size 1 forces three round-trips plus the terminating empty page.
        let opts = ReconcileOptions {
            page_size: 1,
            ..Default::default()
        };

        let r = run_reconcile(&store, &repo, &opts).await.unwrap();

        assert_eq!(r.referenced, 3);
        assert!(r.unreferenced.is_empty());
    }
}
