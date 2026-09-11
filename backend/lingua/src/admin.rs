// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The Lingua ops-console module: aggregates over the `lingua` schema (tiles, a
//! per-day series, the pack registry) for the back office. Aggregates only — the
//! repository returns counts grouped by day/language and never a row attributable to an
//! account (the privacy allow-list). The window parse + series formatting live in
//! [`admin_core`]; the pack registry in [`pack_registry`]. Host-tested against a fake
//! repo; the Postgres adapter is in `pg_admin`.

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_platform::Result;

use crate::admin_core::{SeriesMetric, SeriesPoint, Usage, parse_window, series_points};
use crate::pack_registry::{self, DataPack};

/// Storage port for the ops aggregates (consumer-declared). Every method returns
/// counts only — the SQL groups by day and studied language, never by account.
#[async_trait]
pub trait LinguaAdminRepo: Send + Sync {
    /// Tiles + per-language breakdown over an inclusive epoch-day window.
    async fn usage(&self, from_day: i32, to_day: i32) -> Result<Usage>;
    /// Raw `(epoch_day, value)` points for one metric over the window, optionally for a
    /// single studied language.
    async fn series(
        &self,
        from_day: i32,
        to_day: i32,
        metric: SeriesMetric,
        language: Option<&str>,
    ) -> Result<Vec<(i32, i64)>>;
}

/// Orchestrates the ops console over a [`LinguaAdminRepo`] + the embedded pack registry.
pub struct LinguaAdminModule {
    repo: Arc<dyn LinguaAdminRepo>,
    packs: Vec<DataPack>,
}

impl LinguaAdminModule {
    /// Parses the embedded pack registry once; an invalid committed manifest fails here
    /// (at boot) rather than on a request.
    pub fn new(repo: Arc<dyn LinguaAdminRepo>) -> Result<Self> {
        Ok(Self {
            repo,
            packs: pack_registry::embedded()?,
        })
    }

    /// Aggregate tiles + breakdown for a `yyyy-mm-dd` window.
    pub async fn get_usage(&self, from: &str, to: &str) -> Result<Usage> {
        let (f, t) = parse_window(from, to)?;
        self.repo.usage(f, t).await
    }

    /// A per-day series for one metric over a `yyyy-mm-dd` window.
    pub async fn get_series(
        &self,
        from: &str,
        to: &str,
        metric: SeriesMetric,
        language: Option<&str>,
    ) -> Result<Vec<SeriesPoint>> {
        let (f, t) = parse_window(from, to)?;
        let rows = self.repo.series(f, t, metric, language).await?;
        Ok(series_points(rows))
    }

    /// The read-only pack registry.
    pub fn list_packs(&self) -> &[DataPack] {
        &self.packs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_core::{LanguageUsage, parse_iso_day};
    use cymbra_platform::AppError;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeRepo {
        usage: Usage,
        // (day, value) rows the series call should echo back (unsorted on purpose).
        series_rows: Vec<(i32, i64)>,
        seen: Mutex<Vec<(i32, i32, Option<String>)>>, // (from, to, language) of series calls
    }

    #[async_trait]
    impl LinguaAdminRepo for FakeRepo {
        async fn usage(&self, _from: i32, _to: i32) -> Result<Usage> {
            Ok(self.usage.clone())
        }
        async fn series(
            &self,
            from: i32,
            to: i32,
            _metric: SeriesMetric,
            language: Option<&str>,
        ) -> Result<Vec<(i32, i64)>> {
            self.seen
                .lock()
                .unwrap()
                .push((from, to, language.map(str::to_owned)));
            Ok(self.series_rows.clone())
        }
    }

    fn module(repo: FakeRepo) -> LinguaAdminModule {
        LinguaAdminModule::new(Arc::new(repo)).expect("embedded manifest parses")
    }

    #[tokio::test]
    async fn usage_passes_through_and_a_bad_window_is_rejected() {
        let repo = FakeRepo {
            usage: Usage {
                active_accounts: 4,
                words_learned: 20,
                reviews: 30,
                by_language: vec![LanguageUsage {
                    language: "en".into(),
                    active_accounts: 4,
                    words_learned: 20,
                    reviews: 30,
                }],
            },
            ..Default::default()
        };
        let m = module(repo);
        let u = m.get_usage("2026-09-01", "2026-09-30").await.unwrap();
        assert_eq!(u.active_accounts, 4);
        assert_eq!(u.by_language[0].language, "en");
        // A backwards window never reaches the repo.
        assert!(matches!(
            m.get_usage("2026-09-30", "2026-09-01").await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn series_is_formatted_sorted_and_scopes_language() {
        let base = parse_iso_day("2026-09-10").unwrap();
        let repo = Arc::new(FakeRepo {
            series_rows: vec![(base + 1, 3), (base, 1)],
            ..Default::default()
        });
        let m = LinguaAdminModule::new(repo.clone() as Arc<dyn LinguaAdminRepo>).unwrap();
        let pts = m
            .get_series(
                "2026-09-10",
                "2026-09-12",
                SeriesMetric::Reviews,
                Some("en"),
            )
            .await
            .unwrap();
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0].day, "2026-09-10"); // sorted ascending by day
        assert_eq!(pts[1].value, 3);
        // The window and language filter reached the repo verbatim.
        let seen = repo.seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].0, base);
        assert_eq!(seen[0].2.as_deref(), Some("en"));
    }

    #[tokio::test]
    async fn list_packs_reads_the_embedded_registry() {
        let m = module(FakeRepo::default());
        assert!(!m.list_packs().is_empty());
        assert!(m.list_packs().iter().all(|p| p.size_bytes > 0));
    }
}
