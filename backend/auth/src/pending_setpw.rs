//! Pending "set a password on an OIDC account" records (change: verify-before-
//! local-credential-link).
//!
//! The `SetLocalCredential` flow must not bind the email/`local` identity until the
//! emailed code is confirmed — otherwise a signed-in user could reserve (squat) an
//! arbitrary email with no proof of ownership. So the submitted `{user_id, email,
//! password hash}` is parked here, keyed by the verification token, with a TTL. On
//! verification the record is consumed and the credential is created + linked; an
//! abandoned set-password simply expires (nothing reserved, nothing to reap).

use async_trait::async_trait;
use cymbra_platform::cache::Cache;
use cymbra_platform::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// A password waiting to be bound to `user_id` once `email` is verified. Only the
/// argon2 **hash** is ever stored — never the plaintext.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingLocalCredential {
    pub user_id: String,
    pub email: String,
    pub password_hash: String,
}

/// Single-use, TTL'd store for pending set-password records, keyed by token.
#[async_trait]
pub trait PendingCredentialStore: Send + Sync {
    async fn put(&self, token: &str, pending: &PendingLocalCredential, ttl: Duration)
    -> Result<()>;
    /// Get-and-delete: a token can be consumed at most once.
    async fn take(&self, token: &str) -> Result<Option<PendingLocalCredential>>;
}

fn key(token: &str) -> String {
    format!("pending_setpw:{token}")
}

/// Cache-backed (Redis/Valkey) store; the entry auto-expires at `ttl`.
pub struct CachePendingStore {
    cache: Arc<dyn Cache>,
}

impl CachePendingStore {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl PendingCredentialStore for CachePendingStore {
    async fn put(
        &self,
        token: &str,
        pending: &PendingLocalCredential,
        ttl: Duration,
    ) -> Result<()> {
        let json = serde_json::to_string(pending).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("serialize pending set-password: {e}"))
        })?;
        self.cache.set_ex(&key(token), &json, ttl).await
    }

    async fn take(&self, token: &str) -> Result<Option<PendingLocalCredential>> {
        let k = key(token);
        let Some(json) = self.cache.get(&k).await? else {
            return Ok(None);
        };
        self.cache.del(&k).await?;
        let pending = serde_json::from_str(&json).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("deserialize pending set-password: {e}"))
        })?;
        Ok(Some(pending))
    }
}

/// In-memory store for unit tests, with inspection helpers. Hand-written (rather
/// than a mockall mock) because the round-trip tests need to read back the single
/// stored token — see the `rust-testing` skill's special-case guidance.
#[derive(Default)]
pub struct FakePendingStore {
    rows: std::sync::Mutex<std::collections::HashMap<String, PendingLocalCredential>>,
}

impl FakePendingStore {
    /// The single stored token (panics unless exactly one) — test convenience.
    pub fn only_token(&self) -> String {
        let rows = self.rows.lock().unwrap();
        assert_eq!(rows.len(), 1, "expected exactly one pending set-password");
        rows.keys().next().unwrap().clone()
    }

    /// All stored tokens — test convenience for contention scenarios.
    pub fn tokens(&self) -> Vec<String> {
        self.rows.lock().unwrap().keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.rows.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl PendingCredentialStore for FakePendingStore {
    async fn put(
        &self,
        token: &str,
        pending: &PendingLocalCredential,
        _ttl: Duration,
    ) -> Result<()> {
        self.rows
            .lock()
            .unwrap()
            .insert(token.into(), pending.clone());
        Ok(())
    }

    async fn take(&self, token: &str) -> Result<Option<PendingLocalCredential>> {
        Ok(self.rows.lock().unwrap().remove(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_platform::cache::FakeCache;

    fn rec() -> PendingLocalCredential {
        PendingLocalCredential {
            user_id: "u1".into(),
            email: "a@x.dev".into(),
            password_hash: "argon2hash".into(),
        }
    }

    #[tokio::test]
    async fn cache_store_round_trips_then_is_single_use() {
        let store = CachePendingStore::new(Arc::new(FakeCache::default()));
        store
            .put("tok", &rec(), Duration::from_secs(60))
            .await
            .unwrap();
        assert_eq!(store.take("tok").await.unwrap(), Some(rec()));
        // Consumed — a second take finds nothing.
        assert_eq!(store.take("tok").await.unwrap(), None);
    }

    #[tokio::test]
    async fn take_missing_token_is_none() {
        let store = CachePendingStore::new(Arc::new(FakeCache::default()));
        assert_eq!(store.take("nope").await.unwrap(), None);
    }

    #[tokio::test]
    async fn fake_store_inspects_single_token() {
        let store = FakePendingStore::default();
        store
            .put("tok", &rec(), Duration::from_secs(60))
            .await
            .unwrap();
        assert_eq!(store.only_token(), "tok");
        assert_eq!(store.take("tok").await.unwrap(), Some(rec()));
        assert!(store.is_empty());
    }
}
