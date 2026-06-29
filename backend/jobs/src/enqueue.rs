//! The enqueue seam (task 2.3). [`EnqueueRequest`] is the engine-agnostic
//! description of a job to enqueue, built purely from a [`JobSpec`] + a payload
//! (host-testable). [`Enqueuer`] is the producer-facing port; producers depend on
//! the trait and use the in-memory [`FakeEnqueuer`] in unit tests. The
//! transactional, sqlxmq-backed implementation lives behind this seam in
//! `sqlxmq_engine` (coverage-excluded) so the engine stays swappable (design D1).

use std::time::Duration;

use async_trait::async_trait;
use serde::Serialize;

use crate::error::Result;
use crate::registry::JobSpec;
use crate::retry::RetryPolicy;

/// An engine-agnostic request to enqueue one job. Maps directly onto the
/// `jobs.enqueue(...)` SQL wrapper (and onto sqlxmq's `JobBuilder`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnqueueRequest {
    pub name: String,
    pub channel_name: String,
    pub channel_args: String,
    pub ordered: bool,
    /// Number of retries after the first attempt (sqlxmq semantics).
    pub retries: i32,
    pub retry_backoff: Duration,
    pub delay: Duration,
    pub payload_json: String,
}

impl EnqueueRequest {
    /// Build a request for `spec`, serializing `payload` to JSON and resolving
    /// the retry policy (a runtime override from `jobs.retry_policy`, else the
    /// spec's built-in default).
    pub fn for_job<T: Serialize>(
        spec: &JobSpec,
        payload: &T,
        retry_override: Option<&RetryPolicy>,
    ) -> Result<Self> {
        let retry = retry_override.unwrap_or_else(|| spec.default_retry());
        Ok(Self {
            name: spec.name().to_string(),
            channel_name: spec.channel().name(),
            channel_args: String::new(),
            ordered: spec.channel().is_ordered(),
            retries: retry.sqlxmq_retries() as i32,
            retry_backoff: retry.base_backoff(),
            delay: Duration::ZERO,
            payload_json: serde_json::to_string(payload)?,
        })
    }

    /// Enqueue this job after a delay (e.g. a scheduled occurrence in the future).
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
}

/// Producer-facing enqueue port. The sqlxmq-backed implementation enqueues inside
/// the caller's business transaction (design D1); the fake records requests.
#[async_trait]
pub trait Enqueuer: Send + Sync {
    async fn enqueue(&self, req: EnqueueRequest) -> Result<uuid::Uuid>;
}

/// In-memory enqueuer for unit tests: records every request and returns a stable
/// nil id. No database required.
#[derive(Default)]
pub struct FakeEnqueuer {
    pub recorded: std::sync::Mutex<Vec<EnqueueRequest>>,
}

impl FakeEnqueuer {
    /// All requests enqueued so far (cloned for assertions).
    pub fn requests(&self) -> Vec<EnqueueRequest> {
        self.recorded.lock().unwrap().clone()
    }
}

#[async_trait]
impl Enqueuer for FakeEnqueuer {
    async fn enqueue(&self, req: EnqueueRequest) -> Result<uuid::Uuid> {
        self.recorded.lock().unwrap().push(req);
        Ok(uuid::Uuid::nil())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{self, VERIFICATION_EMAIL};
    use serde::Serialize;

    #[derive(Serialize)]
    struct EmailPayload<'a> {
        to: &'a str,
        token: &'a str,
    }

    #[test]
    fn for_job_maps_spec_and_serializes_payload() {
        let spec = registry::spec(VERIFICATION_EMAIL).unwrap();
        let req = EnqueueRequest::for_job(
            &spec,
            &EmailPayload {
                to: "a@x.dev",
                token: "tok",
            },
            None,
        )
        .unwrap();
        assert_eq!(req.name, VERIFICATION_EMAIL);
        assert_eq!(req.channel_name, "auth.email");
        assert!(!req.ordered);
        assert_eq!(req.retries, 4); // 5 attempts → 4 retries
        assert!(req.payload_json.contains("a@x.dev"));
        assert_eq!(req.delay, Duration::ZERO);
    }

    #[test]
    fn retry_override_wins_over_default() {
        let spec = registry::spec(VERIFICATION_EMAIL).unwrap();
        let override_policy = RetryPolicy::new(2, Duration::from_secs(5), Duration::from_secs(5));
        let req =
            EnqueueRequest::for_job(&spec, &serde_json::json!({}), Some(&override_policy)).unwrap();
        assert_eq!(req.retries, 1);
        assert_eq!(req.retry_backoff, Duration::from_secs(5));
    }

    #[test]
    fn with_delay_sets_delay() {
        let spec = registry::spec(VERIFICATION_EMAIL).unwrap();
        let req = EnqueueRequest::for_job(&spec, &serde_json::json!({}), None)
            .unwrap()
            .with_delay(Duration::from_secs(30));
        assert_eq!(req.delay, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn fake_records_requests() {
        let spec = registry::spec(VERIFICATION_EMAIL).unwrap();
        let req = EnqueueRequest::for_job(&spec, &serde_json::json!({"k":1}), None).unwrap();
        let fake = FakeEnqueuer::default();
        let id = fake.enqueue(req.clone()).await.unwrap();
        assert_eq!(id, uuid::Uuid::nil());
        assert_eq!(fake.requests(), vec![req]);
    }
}
