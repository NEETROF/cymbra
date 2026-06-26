//! Structured logging init (task 2.2).
//!
//! `tracing` is the single logging API. This sets up the console subscriber
//! (env-filtered). The OpenTelemetry layers (traces + the logs signal) are added
//! in task group 6; secrets/tokens must never be logged (enforced by review +
//! the no-PII telemetry test, task 6.6).

use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

/// Initialize the global tracing subscriber. Idempotent-safe: a second call is a
/// no-op (returns `false`).
pub fn init() -> bool {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,cymbra=debug"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .try_init()
        .is_ok()
}
