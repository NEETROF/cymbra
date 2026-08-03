//! `cymbra-analytics` — the analytics-domain module (change: add-feature-usage-
//! analytics).
//!
//! Owns the dedicated `analytics` Postgres schema (design D10): first-party
//! feature-usage telemetry, deliberately decoupled from identity. Every row carries
//! a period-salted pseudonymous `user_bucket` (design D2) — never a raw `user_id`
//! and never a cross-schema FK — so there is nothing to relate to `music` /
//! `user_account`. It exposes:
//!
//! - the batched [`grpc::UsageGrpc`] `UsageService` ingestion adapter (design D5);
//! - the pure, host-testable [`usage_core`] (period-salted bucketing + per-event
//!   validation, design D2/D7);
//! - the raw-event [`repo::UsageEventRepo`] (mockall-doubled) over
//!   `analytics.usage_events`; and
//! - the daily [`rollup`] + retention purge SQL the `cymbra-worker` jobs run.

pub mod grpc;
pub mod read;
pub mod repo;
pub mod rollup;
pub mod usage_core;

pub use grpc::UsageGrpc;
pub use read::{
    ActionCount, DeviceClassUsers, PgUsageReadRepo, PlatformUsers, UsageQuery, UsageReadRepo,
    UsersSummary,
};
pub use repo::{PgUsageEventRepo, UsageEventRepo, UsageRow};
pub use rollup::{purge_expired, rollup_closed_days};
pub use usage_core::{Invalid, RawEvent, ValidEvent, user_bucket, validate};

/// Generated `cymbra.analytics.v1` protobuf messages + tonic client/server stubs
/// (the UsageService — batched feature-usage ingestion).
pub mod proto {
    tonic::include_proto!("cymbra.analytics.v1");
}

/// The module's Postgres schema.
pub const SCHEMA: &str = "analytics";

/// Embedded migrations for the `analytics` schema.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Connects a Postgres pool pinned to `search_path = analytics` (so a privileged
/// role records the `_sqlx_migrations` ledger in the `analytics` schema, mirroring
/// `cymbra-music::connect`).
pub async fn connect(database_url: &str, max_connections: u32) -> anyhow::Result<sqlx::PgPool> {
    use sqlx::Executor;
    Ok(sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET search_path = analytics").await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await?)
}
