//! `cymbra-jobs` — Cymbra's durable async-job substrate (change:
//! add-job-infrastructure).
//!
//! Engine-agnostic by design (D1): this crate owns the **policy** (channels,
//! retries, scheduling, dead-lettering) and the **enqueue seam**; the concrete
//! queue engine (sqlxmq) and the job handlers live in the `cymbra-worker` binary.
//! The pure policy modules ([`channel`], [`retry`], [`schedule`], [`dlq`],
//! [`registry`], and [`EnqueueRequest`]) are host-testable; the DB glue
//! ([`engine`]) sits behind the seam and is coverage-excluded like the other I/O
//! adapters.

pub mod attempt;
pub mod channel;
pub mod dlq;
pub mod engine;
pub mod enqueue;
pub mod error;
pub mod registry;
pub mod retry;
pub mod schedule;
pub mod scheduler;

pub use attempt::{AttemptOutcome, AttemptStart, HISTORY_RETENTION_DAYS};
pub use channel::{Channel, Ordering};
pub use dlq::{DeadLetter, is_exhausted};
pub use engine::{
    PgEnqueuer, begin_attempt, dead_letter_sweep, finish_attempt, load_retry_policy, prune_history,
    tracked, transactional_enqueue,
};
pub use enqueue::{EnqueueRequest, Enqueuer, FakeEnqueuer};
pub use error::{JobError, Result};
pub use registry::{
    JobSpec, ORPHAN_REAP, PURGE_USER, SESSION_REAP, VERIFICATION_EMAIL, protected_kinds, spec,
};
pub use retry::RetryPolicy;
pub use schedule::{MissedRun, Schedule, bucket, dedup_key};
pub use scheduler::run_scheduler_tick;

/// Embedded migrations for the `jobs` schema: the vendored sqlxmq `mq_*` objects
/// plus the `jobs.*` control tables, the `jobs.enqueue` wrapper, and the
/// observability views. Run by `cymbra-worker` as the `worker_svc` role.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
