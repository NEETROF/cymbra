//! `cymbra-auth` — the auth module **implementation**.
//!
//! Owns the `auth` schema (`local_credentials`, `sessions`), the `IdentityVerifier`
//! ([`verifier`]), the local-credential store ([`creds`]), durable Postgres-backed
//! sessions with reuse detection ([`session`] + [`session_pg`]), the
//! [`module::AuthModule`] direct adapter, and the gRPC **server** adapter. Reaches
//! accounts only through `cymbra-user-port`.

pub mod creds;
pub mod creds_pg;
pub mod grpc;
pub mod module;
pub mod session;
pub mod session_pg;
pub mod verifier;

pub use creds::{Credential, CredentialRepo, FakeCredentialRepo};
pub use creds_pg::PgCredentialRepo;
pub use grpc::AuthGrpc;
pub use module::{AuthConfig, AuthModule};
pub use session::{FakeSessionStore, Rotated, SessionInfo, SessionStore};
pub use session_pg::{PgSessionStore, reap_expired_sessions};
pub use verifier::{FakeOidcVerifier, OidcProviderCfg, OidcVerifier, RealOidcVerifier};

/// The auth module's Postgres schema.
pub const SCHEMA: &str = "auth";

/// Embedded migrations for the `auth` schema (task 4.2).
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
