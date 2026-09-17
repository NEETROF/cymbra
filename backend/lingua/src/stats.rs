// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The Stats domain module: per-device daily aggregates upserted idempotently, read
//! back consolidated (summed across devices) for the authenticated user. The sum lives
//! in [`stats_core`]; this layer orchestrates and is host-tested against a fake repo.

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_platform::Result;

use crate::data::ErasureMarks;
use crate::data_core::day_predates_erasure;
use crate::stats_core::consolidate;
pub use crate::stats_core::{ConsolidatedStat, DailyStat};

/// Storage port for daily stats.
#[async_trait]
pub trait StatsRepo: Send + Sync {
    /// Replace-by-key upsert of one device's daily row (idempotent; never additive).
    async fn upsert(&self, user: &str, stat: &DailyStat) -> Result<()>;
    /// The user's per-device rows over an inclusive day range, optionally one language.
    async fn range(
        &self,
        user: &str,
        from_day: i32,
        to_day: i32,
        language: Option<&str>,
    ) -> Result<Vec<DailyStat>>;
}

/// Orchestrates stats sync over a [`StatsRepo`].
pub struct StatsModule {
    repo: Arc<dyn StatsRepo>,
    marks: Arc<dyn ErasureMarks>,
}

impl StatsModule {
    pub fn new(repo: Arc<dyn StatsRepo>, marks: Arc<dyn ErasureMarks>) -> Self {
        Self { repo, marks }
    }

    /// Idempotent upsert of a batch of per-device daily rows; returns how many were
    /// stored. Rows for a day before the user's erasure are dropped
    /// (add-lingua-privacy-controls) but the batch is still acknowledged.
    pub async fn upsert_stats(&self, user: &str, stats: Vec<DailyStat>) -> Result<u64> {
        let erased_at = self.marks.erased_at(user).await?;
        let mut n = 0u64;
        for stat in stats {
            if day_predates_erasure(stat.day, erased_at) {
                continue;
            }
            self.repo.upsert(user, &stat).await?;
            n += 1;
        }
        Ok(n)
    }

    /// Consolidated read: summed across devices, one value per (day, language).
    pub async fn get_stats(
        &self,
        user: &str,
        from_day: i32,
        to_day: i32,
        language: Option<&str>,
    ) -> Result<Vec<ConsolidatedStat>> {
        let rows = self.repo.range(user, from_day, to_day, language).await?;
        Ok(consolidate(&rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MockErasureMarks;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeStatsRepo {
        // user -> (day, language, device) -> row
        #[allow(clippy::type_complexity)]
        rows: Mutex<HashMap<String, HashMap<(i32, String, String), DailyStat>>>,
    }

    #[async_trait]
    impl StatsRepo for FakeStatsRepo {
        async fn upsert(&self, user: &str, stat: &DailyStat) -> Result<()> {
            let mut rows = self.rows.lock().unwrap();
            rows.entry(user.to_owned()).or_default().insert(
                (stat.day, stat.language.clone(), stat.device_id.clone()),
                stat.clone(),
            );
            Ok(())
        }

        async fn range(
            &self,
            user: &str,
            from_day: i32,
            to_day: i32,
            language: Option<&str>,
        ) -> Result<Vec<DailyStat>> {
            let rows = self.rows.lock().unwrap();
            Ok(rows
                .get(user)
                .into_iter()
                .flat_map(|m| m.values())
                .filter(|s| s.day >= from_day && s.day <= to_day)
                .filter(|s| language.is_none_or(|l| s.language == l))
                .cloned()
                .collect())
        }
    }

    fn marks(erased_at: i64) -> Arc<dyn ErasureMarks> {
        let mut marks = MockErasureMarks::new();
        marks.expect_erased_at().returning(move |_| Ok(erased_at));
        Arc::new(marks)
    }

    fn never_erased() -> Arc<dyn ErasureMarks> {
        marks(0)
    }

    fn stat(day: i32, device: &str, reviews: u32) -> DailyStat {
        DailyStat {
            day,
            language: "en".into(),
            device_id: device.into(),
            exposures: 0,
            words_learned: 0,
            reviews_done: reviews,
        }
    }

    #[tokio::test]
    async fn two_devices_same_day_are_summed_on_read() {
        let module = StatsModule::new(Arc::new(FakeStatsRepo::default()), never_erased());
        module
            .upsert_stats(
                "u1",
                vec![stat(20_000, "mac", 20), stat(20_000, "iphone", 10)],
            )
            .await
            .unwrap();
        let out = module
            .get_stats("u1", 19_999, 20_001, Some("en"))
            .await
            .unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].reviews_done, 30);
    }

    #[tokio::test]
    async fn replayed_upsert_replaces_never_adds() {
        let module = StatsModule::new(Arc::new(FakeStatsRepo::default()), never_erased());
        module
            .upsert_stats("u1", vec![stat(20_000, "mac", 20)])
            .await
            .unwrap();
        module
            .upsert_stats("u1", vec![stat(20_000, "mac", 20)])
            .await
            .unwrap(); // same key re-pushed
        let out = module.get_stats("u1", 20_000, 20_000, None).await.unwrap();
        assert_eq!(out[0].reviews_done, 20); // replaced, not 40
    }

    #[tokio::test]
    async fn range_and_language_filter_apply() {
        let module = StatsModule::new(Arc::new(FakeStatsRepo::default()), never_erased());
        module
            .upsert_stats(
                "u1",
                vec![
                    stat(20_000, "mac", 5),
                    DailyStat {
                        language: "fr".into(),
                        ..stat(20_000, "mac", 9)
                    },
                    stat(20_050, "mac", 1),
                ],
            )
            .await
            .unwrap();
        let en = module
            .get_stats("u1", 20_000, 20_010, Some("en"))
            .await
            .unwrap();
        assert_eq!(en.len(), 1);
        assert_eq!(en[0].reviews_done, 5);
    }

    #[tokio::test]
    async fn days_before_the_erasure_day_are_not_stored() {
        // Erased at 01:00 UTC on day 20000: day 19999 is dropped, the erasure day is kept.
        let erased_at = 20_000 * 86_400_000 + 3_600_000;
        let module = StatsModule::new(Arc::new(FakeStatsRepo::default()), marks(erased_at));
        let stored = module
            .upsert_stats("u1", vec![stat(19_999, "mac", 7), stat(20_000, "mac", 2)])
            .await
            .unwrap();
        assert_eq!(stored, 1);
        let out = module.get_stats("u1", 19_990, 20_010, None).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].day, 20_000);
    }
}
