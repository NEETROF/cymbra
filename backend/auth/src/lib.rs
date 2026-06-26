//! `cymbra-auth` — the auth module **implementation**.
//!
//! Owns the `auth` schema (`local_credentials`), the `IdentityVerifier` port with
//! OIDC (Google/Apple) and local-credential adapters, the session/refresh logic
//! (in Redis, via `cymbra-platform`), and the direct + gRPC **server** adapters
//! implementing `cymbra-auth-port`. Reaches accounts only through
//! `cymbra-user-port` (never `cymbra-user`).
//!
//! Implemented across task group 4; this scaffold establishes the crate.

/// Marker for the auth module's schema (see migrations, task 4.2).
pub const SCHEMA: &str = "auth";
