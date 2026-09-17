// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The reader's control over their Lingua data (change: add-lingua-privacy-controls):
//! erase it without touching the Cymbra account, and expose the erasure mark every
//! client reads before pushing. The drop rules live in [`data_core`](crate::data_core);
//! the sync modules consult the mark through [`ErasureMarks`].

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_platform::Result;

/// The erasure mark, as the sync modules need it (consumer-declared port).
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ErasureMarks: Send + Sync {
    /// The user's latest erasure mark (server epoch millis), 0 when they never erased.
    async fn erased_at(&self, user: &str) -> Result<i64>;
}

/// Storage port for the erasure itself.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DataRepo: Send + Sync {
    /// Delete the user's word statuses, declared levels, cards and daily stats and record
    /// `now` as their erasure mark, atomically. Returns the stored mark, which never moves
    /// backwards (a replay keeps the later of the two).
    async fn erase(&self, user: &str, now: i64) -> Result<i64>;
    /// The user's latest erasure mark, 0 when they never erased.
    async fn erased_at(&self, user: &str) -> Result<i64>;
}

/// Orchestrates the Lingua-only erasure over a [`DataRepo`].
pub struct DataModule {
    repo: Arc<dyn DataRepo>,
}

impl DataModule {
    pub fn new(repo: Arc<dyn DataRepo>) -> Self {
        Self { repo }
    }

    /// Erase the caller's Lingua data; returns the recorded mark.
    pub async fn erase_my_data(&self, user: &str, now: i64) -> Result<i64> {
        self.repo.erase(user, now).await
    }

    /// The caller's erasure mark (0 = never erased).
    pub async fn data_state(&self, user: &str) -> Result<i64> {
        self.repo.erased_at(user).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_platform::AppError;
    use mockall::predicate::eq;

    #[tokio::test]
    async fn erasing_passes_the_caller_and_the_server_time() {
        let mut repo = MockDataRepo::new();
        repo.expect_erase()
            .with(eq("u1"), eq(5_000))
            .times(1)
            .returning(|_, now| Ok(now));
        let module = DataModule::new(Arc::new(repo));
        assert_eq!(module.erase_my_data("u1", 5_000).await.unwrap(), 5_000);
    }

    #[tokio::test]
    async fn a_replayed_erasure_reports_the_stored_mark() {
        let mut repo = MockDataRepo::new();
        // The repo keeps the later mark; the module reports what was stored.
        repo.expect_erase().returning(|_, _| Ok(9_000));
        let module = DataModule::new(Arc::new(repo));
        assert_eq!(module.erase_my_data("u1", 5_000).await.unwrap(), 9_000);
    }

    #[tokio::test]
    async fn the_state_is_the_callers_mark() {
        let mut repo = MockDataRepo::new();
        repo.expect_erased_at()
            .with(eq("never"))
            .returning(|_| Ok(0));
        repo.expect_erased_at()
            .with(eq("erased"))
            .returning(|_| Ok(7_000));
        let module = DataModule::new(Arc::new(repo));
        assert_eq!(module.data_state("never").await.unwrap(), 0);
        assert_eq!(module.data_state("erased").await.unwrap(), 7_000);
    }

    #[tokio::test]
    async fn a_storage_failure_is_reported() {
        let mut repo = MockDataRepo::new();
        repo.expect_erase()
            .returning(|_, _| Err(AppError::Internal(anyhow::anyhow!("db down"))));
        let module = DataModule::new(Arc::new(repo));
        assert!(module.erase_my_data("u1", 1).await.is_err());
    }
}
