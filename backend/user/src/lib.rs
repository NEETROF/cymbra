//! `cymbra-user` — the user module **implementation**.
//!
//! Owns the `user_account` schema (`users`, `user_identities`, `user_roles`), the
//! [`module::UserModule`] direct adapter over a [`repo::UserRepo`] (Postgres or a
//! fake), and the gRPC **server** adapter. Depends only on `cymbra-user-port` +
//! `cymbra-platform`.

pub mod grpc;
pub mod module;
pub mod pg;
pub mod repo;
pub mod version_core;

pub use grpc::UserGrpc;
pub use module::UserModule;
pub use pg::PgUserRepo;
pub use repo::{FakeUserRepo, UserRepo};

/// The user module's Postgres schema.
pub const SCHEMA: &str = "user_account";

/// Embedded migrations for the `user_account` schema (task 3.2).
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
