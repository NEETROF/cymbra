//! `cymbra-user` — the user module **implementation**.
//!
//! Owns the `user_account` schema (`users`, `user_identities`, `user_roles`),
//! the direct adapter implementing `cymbra-user-port`, and the gRPC **server**
//! adapter. Depends only on `cymbra-user-port` + `cymbra-platform`.
//!
//! Implemented across task group 3; this scaffold establishes the crate.

/// Marker for the user module's schema (see migrations, task 3.2).
pub const SCHEMA: &str = "user_account";
