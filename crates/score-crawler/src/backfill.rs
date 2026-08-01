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

//! Mutopia title backfill (`backfill_mutopia_titles` binary).
//!
//! The shared stored-bytes title backfill (`cymbra_music::backfill`) re-derives a
//! row's title from its `.mxl`'s `<work-title>`. That repairs OpenScore Lieder
//! (embedded title shadowed by a filename id) but NOT Mutopia: the LilyPond→
//! MusicXML conversion drops the title, so the converted `.mxl` carries none and
//! the stored-bytes backfill leaves those rows on their filename fallback
//! ("bwv 1001 1"). Mutopia's real title lives only in the source `.ly` `\header`.
//!
//! This backfill reuses the same [`TitleBackfillRepo`] seam (keyset paging,
//! `edited_by` anti-clobber, and the `title`/`title_norm`/`work_key` update that
//! keeps search consistent), but takes the title from the checked-out `.ly` via
//! each row's `source_item_id` instead of the object store. The composer is
//! already correct on existing rows (the MusicXML carries it), so only the title
//! moves; `work_key`/`title_norm` are recomputed from the new title and that
//! existing composer through the same [`derive_keys`] the ingest uses.
//!
//! The pure decision ([`plan_mutopia_title`]) and the orchestration
//! ([`run_mutopia_title_backfill`]) are host-testable; the Postgres seam and the
//! git checkout are the only untested I/O.

use std::path::Path;

use anyhow::{Context, Result};
use cymbra_music::backfill::{BackfillReport, TitleBackfillRepo, TitleUpdate};
use cymbra_music::catalog_edit::derive_keys;

use crate::sources::mutopia::header_title_composer;

/// What to do with one Mutopia row, decided purely from its `.ly` header and its
/// stored values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TitlePlan {
    /// The header title already matches the stored one — nothing to do.
    Unchanged,
    /// The header carries no title — keep the adapter fallback (should not happen
    /// for a real Mutopia piece, but never overwrite with nothing).
    NoTitle,
    /// Rewrite the title (and its recomputed search keys).
    Update(TitleUpdate),
}

/// Decide a row's fate from its source `.ly` and current stored title/composer.
///
/// Prefers the movement-distinct `mutopiatitle` over the shared `title` (via
/// [`header_title_composer`]). The recomputed `title_norm`/`work_key` come from the
/// new title and the row's existing composer through the same [`derive_keys`] the
/// ingest and curator edits use, so a rewritten row stays findable.
pub fn plan_mutopia_title(
    ly: &str,
    current_title: Option<&str>,
    current_composer: Option<&str>,
) -> TitlePlan {
    let (header_title, _) = header_title_composer(ly);
    let Some(title) = header_title else {
        return TitlePlan::NoTitle;
    };
    if Some(title.as_str()) == current_title {
        return TitlePlan::Unchanged;
    }
    let keys = derive_keys(Some(&title), current_composer);
    TitlePlan::Update(TitleUpdate {
        title,
        title_norm: keys.title_norm,
        work_key: keys.work_key,
    })
}

/// Recompute Mutopia titles from the checked-out `.ly` headers, updating each row
/// in place. `apply == false` is a dry run (plans and counts, writes nothing). A
/// single unreadable `.ly` or failed update is logged and skipped, never fatal.
/// Idempotent: a second run over corrected rows is all `unchanged`.
pub async fn run_mutopia_title_backfill(
    repo: &dyn TitleBackfillRepo,
    checkout: &Path,
    apply: bool,
    page_size: i64,
) -> Result<BackfillReport> {
    let mut report = BackfillReport::default();
    let mut after = String::new();
    loop {
        let rows = repo
            .page(&after, Some("mutopia"), page_size)
            .await
            .context("paging catalog for Mutopia backfill")?;
        let Some(last) = rows.last() else { break };
        after = last.id.clone();

        for row in rows {
            report.scanned += 1;
            let path = checkout.join(&row.source_item_id);
            let ly = match std::fs::read_to_string(&path) {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(id = %row.id, path = %path.display(), error = %e,
                        "backfill: source .ly unreadable, skipping");
                    report.errors += 1;
                    continue;
                }
            };
            match plan_mutopia_title(&ly, row.title.as_deref(), row.composer.as_deref()) {
                TitlePlan::NoTitle => report.no_title += 1,
                TitlePlan::Unchanged => report.unchanged += 1,
                TitlePlan::Update(update) => {
                    if apply && let Err(e) = repo.update_title(&row.id, &update).await {
                        tracing::warn!(id = %row.id, error = %e,
                            "backfill: update failed, skipping");
                        report.errors += 1;
                        continue;
                    }
                    tracing::info!(id = %row.id, from = ?row.title, to = %update.title,
                        applied = apply, "backfill: Mutopia title rewritten");
                    report.updated += 1;
                }
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use cymbra_music::backfill::BackfillRow;
    use std::sync::Mutex;

    const BWV: &str = "\\header {\n  title = \"Sonata I BWV 1001\"\n  \
        mutopiatitle = \"BWV 1001 Adagio\"\n  composer = \"Johann Sebastian Bach (1685-1750)\"\n}";

    #[test]
    fn plans_update_from_header_reusing_current_composer_for_keys() {
        let plan = plan_mutopia_title(BWV, Some("bwv 1001 1"), Some("Johann Sebastian Bach"));
        let TitlePlan::Update(u) = plan else {
            panic!("expected an update, got {plan:?}");
        };
        assert_eq!(u.title, "BWV 1001 Adagio"); // mutopiatitle wins
        assert_eq!(u.title_norm.as_deref(), Some("bwv 1001 adagio"));
        // work_key = normalized(current composer)::normalized(new title).
        assert_eq!(u.work_key, "johann sebastian bach::bwv 1001 adagio");
    }

    #[test]
    fn unchanged_and_no_title_are_left_alone() {
        assert_eq!(
            plan_mutopia_title(BWV, Some("BWV 1001 Adagio"), None),
            TitlePlan::Unchanged
        );
        assert_eq!(
            plan_mutopia_title("{ c'4 }", Some("x"), None),
            TitlePlan::NoTitle
        );
    }

    /// In-memory [`TitleBackfillRepo`]: one keyset page of rows, recording updates.
    struct FakeRepo {
        rows: Vec<BackfillRow>,
        applied: Mutex<Vec<(String, TitleUpdate)>>,
    }

    #[async_trait]
    impl TitleBackfillRepo for FakeRepo {
        async fn page(
            &self,
            after: &str,
            source: Option<&str>,
            _limit: i64,
        ) -> Result<Vec<BackfillRow>> {
            assert_eq!(source, Some("mutopia"), "backfill must scope to mutopia");
            Ok(self
                .rows
                .iter()
                .filter(|r| r.id.as_str() > after)
                .cloned()
                .collect())
        }

        async fn update_title(&self, id: &str, update: &TitleUpdate) -> Result<()> {
            self.applied
                .lock()
                .unwrap()
                .push((id.to_string(), update.clone()));
            Ok(())
        }
    }

    fn row(id: &str, rel: &str, title: Option<&str>) -> BackfillRow {
        BackfillRow {
            id: id.into(),
            object_key: format!("safe/mutopia/{id}.mxl"),
            title: title.map(Into::into),
            source_item_id: rel.into(),
            composer: Some("J. S. Bach".into()),
        }
    }

    #[tokio::test]
    async fn run_rewrites_only_stale_titles_and_tallies() {
        let dir = std::env::temp_dir().join(format!("mutopia-backfill-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.ly"), BWV).unwrap();
        std::fs::write(dir.join("b.ly"), BWV).unwrap();
        std::fs::write(dir.join("c.ly"), "{ c'4 }").unwrap(); // no header title

        let repo = FakeRepo {
            rows: vec![
                row("id1", "a.ly", Some("bwv 1001 1")),      // stale → rewrite
                row("id2", "b.ly", Some("BWV 1001 Adagio")), // already correct
                row("id3", "c.ly", Some("whatever")),        // no header title → kept
                row("id4", "missing.ly", Some("x")),         // unreadable → error
            ],
            applied: Mutex::new(Vec::new()),
        };

        let report = run_mutopia_title_backfill(&repo, &dir, true, 100)
            .await
            .unwrap();
        assert_eq!(report.scanned, 4);
        assert_eq!(report.updated, 1);
        assert_eq!(report.unchanged, 1);
        assert_eq!(report.no_title, 1);
        assert_eq!(report.errors, 1);

        let applied = repo.applied.lock().unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].0, "id1");
        assert_eq!(applied[0].1.title, "BWV 1001 Adagio");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn dry_run_counts_but_writes_nothing() {
        let dir = std::env::temp_dir().join(format!("mutopia-backfill-dry-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.ly"), BWV).unwrap();
        let repo = FakeRepo {
            rows: vec![row("id1", "a.ly", Some("bwv 1001 1"))],
            applied: Mutex::new(Vec::new()),
        };

        let report = run_mutopia_title_backfill(&repo, &dir, false, 100)
            .await
            .unwrap();
        assert_eq!(report.updated, 1); // would rewrite
        assert!(repo.applied.lock().unwrap().is_empty()); // wrote nothing

        std::fs::remove_dir_all(&dir).ok();
    }
}
