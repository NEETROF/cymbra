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

pub mod catalog_search;
pub mod grpc;
pub mod module;
pub mod pg;
pub mod pg_user_scores;
pub mod repo;
pub mod user_library;
pub mod user_scores;

pub use catalog_search::{
    CatalogHit, CatalogSearchParams, CatalogSearchRepo, FakeCatalogRow, FakeCatalogSearchRepo,
};
pub use grpc::ScoreGrpc;
pub use module::{ScoreModule, UploadInput};
pub use pg::{PgCatalogRepo, PgCatalogSearchRepo};
pub use pg_user_scores::{PgUserLibraryRepo, PgUserScoreRepo};
pub use repo::{CatalogEntry, CatalogRepo, FakeCatalogRepo};
pub use user_library::{FakeUserLibraryRepo, UserLibraryRepo};
pub use user_scores::{FakeUserScoreRepo, UserScore, UserScoreRepo};

/// Generated protobuf messages + tonic client/server stubs for `cymbra.music.v1`
/// (the ScoreService — user uploads).
pub mod proto {
    tonic::include_proto!("cymbra.music.v1");
}

/// The module's Postgres schema.
pub const SCHEMA: &str = "music";

/// Embedded migrations for the `music` schema.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Connects a Postgres pool (used by the crawler's direct-ingestion path).
///
/// Pins `search_path = music` on every connection so that whatever role the
/// crawler uses (often a superuser via `CYMBRA_SCORE_DATABASE_URL`) records the
/// `_sqlx_migrations` ledger in the **same** `music` schema as the server's
/// `music_svc` pool — one shared ledger, so the second `MIGRATOR.run` only applies
/// new versions instead of forking a second ledger in `public` (design 5 / Option A).
pub async fn connect(database_url: &str, max_connections: u32) -> anyhow::Result<sqlx::PgPool> {
    use sqlx::Executor;
    Ok(sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET search_path = music").await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await?)
}
