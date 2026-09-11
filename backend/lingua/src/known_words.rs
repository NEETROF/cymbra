// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The KnownWords domain module: a client outbox drained by last-write-wins into a
//! per-user monotonic change log, pulled back by cursor, bootstrapped by an
//! ETag-guarded snapshot. The clamp + LWW rules live in [`known_words_core`]; this
//! layer orchestrates and is host-tested against a fake repo.

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_platform::Result;

use crate::known_words_core::clamp_ts;

/// One status op from a client outbox (identity is the token's user, never here).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusOpInput {
    pub language: String,
    pub lemma: String,
    pub status: String,
    pub provenance: String,
    pub client_ts: i64,
    pub device_id: String,
}

/// A resolved status as it now stands on the server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusChange {
    pub language: String,
    pub lemma: String,
    pub status: String,
    pub updated_at: i64,
    pub sequence: i64,
}

/// Alias kept for the public surface; a status op as accepted from a client.
pub type Status = StatusOpInput;

/// Storage port (consumer-declared). The Postgres adapter and the fake both apply the
/// same last-write-wins rule from `known_words_core`.
#[async_trait]
pub trait KnownWordsRepo: Send + Sync {
    /// Apply one op (its `client_ts` is already clamped). Returns whether it won LWW and
    /// changed stored state; a winning op is assigned the next change sequence.
    async fn apply_op(&self, user: &str, op: &StatusOpInput) -> Result<bool>;
    /// The user's current tip cursor (max change sequence; 0 when empty).
    async fn tip_cursor(&self, user: &str) -> Result<i64>;
    /// Changes with `sequence > cursor`, in sequence order.
    async fn changes_since(&self, user: &str, cursor: i64) -> Result<Vec<StatusChange>>;
    /// Every current status for the user (the snapshot).
    async fn snapshot(&self, user: &str) -> Result<Vec<StatusChange>>;
}

/// Orchestrates status sync over a [`KnownWordsRepo`].
pub struct KnownWordsModule {
    repo: Arc<dyn KnownWordsRepo>,
}

impl KnownWordsModule {
    pub fn new(repo: Arc<dyn KnownWordsRepo>) -> Self {
        Self { repo }
    }

    /// Drain a client outbox: clamp each op's timestamp to `now`, apply LWW, and return
    /// the number that changed state plus the new tip cursor. Idempotent (a replayed
    /// batch changes nothing).
    pub async fn push_ops(
        &self,
        user: &str,
        ops: Vec<StatusOpInput>,
        now: i64,
    ) -> Result<(u64, i64)> {
        let mut applied = 0u64;
        for mut op in ops {
            op.client_ts = clamp_ts(op.client_ts, now);
            if self.repo.apply_op(user, &op).await? {
                applied += 1;
            }
        }
        Ok((applied, self.repo.tip_cursor(user).await?))
    }

    /// Pull changes after `cursor`, returning them and the cursor to use next time.
    pub async fn pull_changes(&self, user: &str, cursor: i64) -> Result<(Vec<StatusChange>, i64)> {
        let changes = self.repo.changes_since(user, cursor).await?;
        let next = changes.iter().map(|c| c.sequence).max().unwrap_or(cursor);
        Ok((changes, next))
    }

    /// A snapshot guarded by an ETag (the tip cursor as a string): when the client's
    /// ETag still matches, return `unchanged` with no body.
    pub async fn get_snapshot(
        &self,
        user: &str,
        etag: &str,
    ) -> Result<(bool, String, Vec<StatusChange>)> {
        let current = self.repo.tip_cursor(user).await?.to_string();
        if !etag.is_empty() && etag == current {
            return Ok((true, current, Vec::new()));
        }
        Ok((false, current, self.repo.snapshot(user).await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::known_words_core::wins;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Stored {
        status: String,
        updated_at: i64,
        device_id: String,
        sequence: i64,
    }

    #[derive(Default)]
    struct FakeKnownWordsRepo {
        // user -> (language, lemma) -> stored
        #[allow(clippy::type_complexity)]
        rows: Mutex<HashMap<String, HashMap<(String, String), Stored>>>,
        seq: Mutex<i64>,
    }

    #[async_trait]
    impl KnownWordsRepo for FakeKnownWordsRepo {
        async fn apply_op(&self, user: &str, op: &StatusOpInput) -> Result<bool> {
            let mut rows = self.rows.lock().unwrap();
            let per_user = rows.entry(user.to_owned()).or_default();
            let key = (op.language.clone(), op.lemma.clone());
            if let Some(existing) = per_user.get(&key)
                && !wins(
                    existing.updated_at,
                    &existing.device_id,
                    op.client_ts,
                    &op.device_id,
                )
            {
                return Ok(false);
            }
            let mut seq = self.seq.lock().unwrap();
            *seq += 1;
            per_user.insert(
                key,
                Stored {
                    status: op.status.clone(),
                    updated_at: op.client_ts,
                    device_id: op.device_id.clone(),
                    sequence: *seq,
                },
            );
            Ok(true)
        }

        async fn tip_cursor(&self, user: &str) -> Result<i64> {
            let rows = self.rows.lock().unwrap();
            Ok(rows
                .get(user)
                .into_iter()
                .flat_map(|m| m.values())
                .map(|s| s.sequence)
                .max()
                .unwrap_or(0))
        }

        async fn changes_since(&self, user: &str, cursor: i64) -> Result<Vec<StatusChange>> {
            let rows = self.rows.lock().unwrap();
            let mut out: Vec<StatusChange> = rows
                .get(user)
                .into_iter()
                .flat_map(|m| m.iter())
                .filter(|(_, s)| s.sequence > cursor)
                .map(|((lang, lemma), s)| StatusChange {
                    language: lang.clone(),
                    lemma: lemma.clone(),
                    status: s.status.clone(),
                    updated_at: s.updated_at,
                    sequence: s.sequence,
                })
                .collect();
            out.sort_by_key(|c| c.sequence);
            Ok(out)
        }

        async fn snapshot(&self, user: &str) -> Result<Vec<StatusChange>> {
            self.changes_since(user, 0).await
        }
    }

    fn op(lemma: &str, status: &str, ts: i64, device: &str) -> StatusOpInput {
        StatusOpInput {
            language: "en".into(),
            lemma: lemma.into(),
            status: status.into(),
            provenance: "manual".into(),
            client_ts: ts,
            device_id: device.into(),
        }
    }

    #[tokio::test]
    async fn push_then_pull_propagates() {
        let module = KnownWordsModule::new(Arc::new(FakeKnownWordsRepo::default()));
        let (applied, cursor) = module
            .push_ops("u1", vec![op("seldom", "known", 100, "mac")], 1_000)
            .await
            .unwrap();
        assert_eq!(applied, 1);
        // A second device pulls from the start and sees `seldom = known`.
        let (changes, next) = module.pull_changes("u1", 0).await.unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].status, "known");
        assert_eq!(next, cursor);
    }

    #[tokio::test]
    async fn conflict_resolves_by_latest_timestamp_and_both_converge() {
        let module = KnownWordsModule::new(Arc::new(FakeKnownWordsRepo::default()));
        // Mac marks known @100; iPhone marks learning @200 (later) — learning wins.
        module
            .push_ops("u1", vec![op("seldom", "known", 100, "mac")], 1_000)
            .await
            .unwrap();
        module
            .push_ops("u1", vec![op("seldom", "learning", 200, "iphone")], 1_000)
            .await
            .unwrap();
        let (snap, _, _) = module.get_snapshot("u1", "").await.unwrap();
        assert!(!snap); // full snapshot returned
        let statuses = module.repo.snapshot("u1").await.unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].status, "learning");
    }

    #[tokio::test]
    async fn replayed_batch_is_idempotent() {
        let module = KnownWordsModule::new(Arc::new(FakeKnownWordsRepo::default()));
        let batch = vec![
            op("seldom", "known", 100, "mac"),
            op("city", "known", 100, "mac"),
        ];
        let (a1, c1) = module.push_ops("u1", batch.clone(), 1_000).await.unwrap();
        assert_eq!(a1, 2);
        let (a2, c2) = module.push_ops("u1", batch, 1_000).await.unwrap();
        assert_eq!(a2, 0); // nothing changed
        assert_eq!(c1, c2); // cursor did not advance
    }

    #[tokio::test]
    async fn future_timestamp_is_clamped_so_a_later_honest_op_can_win() {
        let module = KnownWordsModule::new(Arc::new(FakeKnownWordsRepo::default()));
        // A skewed client pins @9999 but now=1000 → clamped to 1000.
        module
            .push_ops("u1", vec![op("seldom", "ignored", 9_999, "bad")], 1_000)
            .await
            .unwrap();
        // An honest later op @1500 then wins.
        let (applied, _) = module
            .push_ops("u1", vec![op("seldom", "known", 1_500, "good")], 2_000)
            .await
            .unwrap();
        assert_eq!(applied, 1);
        assert_eq!(module.repo.snapshot("u1").await.unwrap()[0].status, "known");
    }

    #[tokio::test]
    async fn snapshot_etag_short_circuits_when_unchanged() {
        let module = KnownWordsModule::new(Arc::new(FakeKnownWordsRepo::default()));
        module
            .push_ops("u1", vec![op("seldom", "known", 100, "mac")], 1_000)
            .await
            .unwrap();
        let (_, etag, _) = module.get_snapshot("u1", "").await.unwrap();
        let (unchanged, etag2, body) = module.get_snapshot("u1", &etag).await.unwrap();
        assert!(unchanged);
        assert_eq!(etag, etag2);
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn incremental_pull_only_returns_new_changes() {
        let module = KnownWordsModule::new(Arc::new(FakeKnownWordsRepo::default()));
        module
            .push_ops("u1", vec![op("seldom", "known", 100, "mac")], 1_000)
            .await
            .unwrap();
        let (_, cursor) = module.pull_changes("u1", 0).await.unwrap();
        module
            .push_ops("u1", vec![op("city", "known", 200, "mac")], 1_000)
            .await
            .unwrap();
        let (delta, _) = module.pull_changes("u1", cursor).await.unwrap();
        assert_eq!(delta.len(), 1);
        assert_eq!(delta[0].lemma, "city");
    }
}
