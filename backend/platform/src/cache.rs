//! Redis client/port for ephemeral auth state (task 2.7): sessions/refresh,
//! rate-limit counters, email throttles. A [`Cache`] trait keeps modules testable
//! with [`FakeCache`] (no Redis).

use crate::error::{AppError, Result};
use async_trait::async_trait;
use redis::AsyncCommands;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

/// Minimal key/value cache surface used across the backend.
#[async_trait]
pub trait Cache: Send + Sync {
    /// Atomically increment `key`, setting its TTL on first creation. Returns the
    /// post-increment value (for rate-limiting).
    async fn incr_with_ttl(&self, key: &str, ttl: Duration) -> Result<u64>;
    /// Set `key` = `value` with an expiry.
    async fn set_ex(&self, key: &str, value: &str, ttl: Duration) -> Result<()>;
    /// Get `key` if present and unexpired.
    async fn get(&self, key: &str) -> Result<Option<String>>;
    /// Delete `key`.
    async fn del(&self, key: &str) -> Result<()>;
    /// Reachability probe for readiness.
    async fn ping(&self) -> bool;
}

fn map(e: redis::RedisError) -> AppError {
    AppError::Internal(anyhow::anyhow!("redis: {e}"))
}

/// Redis-backed [`Cache`] over a cheap-to-clone connection manager.
#[derive(Clone)]
pub struct RedisCache {
    mgr: redis::aio::ConnectionManager,
}

impl RedisCache {
    pub async fn connect(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("redis url: {e}")))?;
        let mgr = redis::aio::ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("redis connect: {e}")))?;
        Ok(Self { mgr })
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn incr_with_ttl(&self, key: &str, ttl: Duration) -> Result<u64> {
        let mut c = self.mgr.clone();
        let n: u64 = c.incr(key, 1u64).await.map_err(map)?;
        if n == 1 {
            let _: () = c.expire(key, ttl.as_secs() as i64).await.map_err(map)?;
        }
        Ok(n)
    }

    async fn set_ex(&self, key: &str, value: &str, ttl: Duration) -> Result<()> {
        let mut c = self.mgr.clone();
        let _: () = c.set_ex(key, value, ttl.as_secs()).await.map_err(map)?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut c = self.mgr.clone();
        let v: Option<String> = c.get(key).await.map_err(map)?;
        Ok(v)
    }

    async fn del(&self, key: &str) -> Result<()> {
        let mut c = self.mgr.clone();
        let _: () = c.del(key).await.map_err(map)?;
        Ok(())
    }

    async fn ping(&self) -> bool {
        let mut c = self.mgr.clone();
        let r: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut c).await;
        r.is_ok()
    }
}

/// In-memory [`Cache`] for unit tests (TTL ignored).
#[derive(Default)]
pub struct FakeCache {
    store: Mutex<HashMap<String, String>>,
}

#[async_trait]
impl Cache for FakeCache {
    async fn incr_with_ttl(&self, key: &str, _ttl: Duration) -> Result<u64> {
        let mut s = self.store.lock().unwrap();
        let n = s.get(key).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0) + 1;
        s.insert(key.to_string(), n.to_string());
        Ok(n)
    }
    async fn set_ex(&self, key: &str, value: &str, _ttl: Duration) -> Result<()> {
        self.store.lock().unwrap().insert(key.into(), value.into());
        Ok(())
    }
    async fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self.store.lock().unwrap().get(key).cloned())
    }
    async fn del(&self, key: &str) -> Result<()> {
        self.store.lock().unwrap().remove(key);
        Ok(())
    }
    async fn ping(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_incr_and_kv() {
        let c = FakeCache::default();
        assert_eq!(
            c.incr_with_ttl("k", Duration::from_secs(1)).await.unwrap(),
            1
        );
        assert_eq!(
            c.incr_with_ttl("k", Duration::from_secs(1)).await.unwrap(),
            2
        );
        c.set_ex("a", "b", Duration::from_secs(1)).await.unwrap();
        assert_eq!(c.get("a").await.unwrap().as_deref(), Some("b"));
        c.del("a").await.unwrap();
        assert_eq!(c.get("a").await.unwrap(), None);
    }
}
