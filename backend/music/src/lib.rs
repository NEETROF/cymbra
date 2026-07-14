//! `cymbra-music` — the music-domain module.
//!
//! Owns the `music` Postgres schema: the whole music app's data (scores today,
//! practice/performance/etc. later) lives here, isolated from identity/auth and
//! jobs but sharing one schema + role so its tables can reference each other with
//! real FKs. Today it holds `catalog_scores` — the public, crawler-ingested
//! redistributable corpus — behind a [`repo::CatalogRepo`] (Postgres or an
//! in-memory fake), plus the user-upload `user_scores` + gRPC surface.
//!
//! The catalog surface is `anyhow`-based (it is written to directly by the
//! score-crawler tool, not over gRPC), so consumers need not depend on the
//! platform error type.

pub mod pg;
pub mod repo;

pub use pg::PgCatalogRepo;
pub use repo::{CatalogEntry, CatalogRepo, FakeCatalogRepo};

/// The module's Postgres schema.
pub const SCHEMA: &str = "music";

/// Embedded migrations for the `music` schema.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Connects a Postgres pool (used by the crawler's direct-ingestion path).
pub async fn connect(database_url: &str, max_connections: u32) -> anyhow::Result<sqlx::PgPool> {
    Ok(sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?)
}
