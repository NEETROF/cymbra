//! The L2 invalidation bus (design D2): a coarse "flags changed" ping so every
//! server/worker instance refreshes its L1 snapshot within milliseconds of an
//! edit, well before the TTL backstop.
//!
//! Redis pub/sub is chosen because Redis is already a hard dependency (rate-limit
//! counters) — this adds no new infra. The trait is the seam; unit tests use a
//! fake ([`RecordingBus`]) and evaluation logic never depends on Redis being up
//! (a down bus just means edits propagate via the TTL instead).

use async_trait::async_trait;
use cymbra_platform::error::{AppError, Result};
use std::sync::Mutex;

/// The default channel a coarse invalidation is published on.
pub const DEFAULT_CHANNEL: &str = "cymbra:flags:changed";

/// Publishes a coarse invalidation ping on every override edit.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait InvalidationBus: Send + Sync {
    /// Publish "flags changed". Best-effort: a failure is logged by the caller and
    /// does not fail the edit (the DB write already succeeded; the TTL will catch
    /// other instances up).
    async fn publish(&self) -> Result<()>;
}

/// A no-op bus for defaults-only mode (no Redis / single instance) and tests where
/// the ping is irrelevant.
#[derive(Debug, Default, Clone)]
pub struct NoopBus;

#[async_trait]
impl InvalidationBus for NoopBus {
    async fn publish(&self) -> Result<()> {
        Ok(())
    }
}

/// An in-memory bus that counts publishes — used by unit tests to assert an edit
/// triggers invalidation without a live Redis.
#[derive(Debug, Default)]
pub struct RecordingBus {
    publishes: Mutex<u32>,
}

impl RecordingBus {
    pub fn count(&self) -> u32 {
        *self.publishes.lock().unwrap()
    }
}

#[async_trait]
impl InvalidationBus for RecordingBus {
    async fn publish(&self) -> Result<()> {
        *self.publishes.lock().unwrap() += 1;
        Ok(())
    }
}

fn map(e: redis::RedisError) -> AppError {
    AppError::Internal(anyhow::anyhow!("redis pubsub: {e}"))
}

/// Redis-backed [`InvalidationBus`]. Publishing rides the shared cheap-to-clone
/// connection manager; subscription is a separate dedicated connection (see
/// [`run_invalidation_listener`]).
#[derive(Clone)]
pub struct RedisInvalidationBus {
    mgr: redis::aio::ConnectionManager,
    channel: String,
}

impl RedisInvalidationBus {
    pub async fn connect(url: &str, channel: impl Into<String>) -> Result<Self> {
        let client = redis::Client::open(url)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("redis url: {e}")))?;
        let mgr = redis::aio::ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("redis connect: {e}")))?;
        Ok(Self {
            mgr,
            channel: channel.into(),
        })
    }
}

#[async_trait]
impl InvalidationBus for RedisInvalidationBus {
    async fn publish(&self) -> Result<()> {
        use redis::AsyncCommands;
        let mut c = self.mgr.clone();
        let _: () = c.publish(&self.channel, "1").await.map_err(map)?;
        Ok(())
    }
}

/// Subscribe to the invalidation channel forever, calling `on_change` on each
/// ping (typically `service.refresh()`). Runs on a dedicated pub/sub connection;
/// spawn it once per instance at the composition root. A dropped connection ends
/// the loop — the caller may respawn (the TTL backstop keeps L1 fresh meanwhile).
pub async fn run_invalidation_listener<F, Fut>(
    url: &str,
    channel: &str,
    mut on_change: F,
) -> Result<()>
where
    F: FnMut() -> Fut + Send,
    Fut: std::future::Future<Output = ()> + Send,
{
    use futures_util::StreamExt;
    let client = redis::Client::open(url)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("redis url: {e}")))?;
    let mut pubsub = client
        .get_async_pubsub()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("redis pubsub connect: {e}")))?;
    pubsub.subscribe(channel).await.map_err(map)?;
    let mut stream = pubsub.on_message();
    while stream.next().await.is_some() {
        on_change().await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_bus_is_ok() {
        assert!(NoopBus.publish().await.is_ok());
    }

    #[tokio::test]
    async fn recording_bus_counts() {
        let bus = RecordingBus::default();
        bus.publish().await.unwrap();
        bus.publish().await.unwrap();
        assert_eq!(bus.count(), 2);
    }
}
