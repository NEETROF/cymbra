//! `cymbra-score` — the score module.
//!
//! Owns the `score` Postgres schema. Today it holds `catalog_scores` — the
//! public, crawler-ingested redistributable corpus — behind a [`repo::CatalogRepo`]
//! (Postgres or an in-memory fake). User uploads (`user_scores` + a gRPC surface)
//! are added to this same schema by the user-upload change.
//!
//! The catalog surface is `anyhow`-based (it is written to directly by the
//! score-crawler tool, not over gRPC), so consumers need not depend on the
//! platform error type.

pub mod pg;
pub mod repo;

pub use pg::PgCatalogRepo;
pub use repo::{CatalogEntry, CatalogRepo, FakeCatalogRepo};

/// The module's Postgres schema.
pub const SCHEMA: &str = "score";

/// Embedded migrations for the `score` schema.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Connects a Postgres pool (used by the crawler's direct-ingestion path).
pub async fn connect(database_url: &str, max_connections: u32) -> anyhow::Result<sqlx::PgPool> {
    Ok(sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?)
}
