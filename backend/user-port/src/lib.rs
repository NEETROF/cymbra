//! `cymbra-user-port` — the user module's **contract** crate.
//!
//! Carries the port trait (resolve-or-provision, link/unlink, list identities,
//! get/update/delete account, effective roles), its DTOs, the generated protobuf
//! types, and (task 3.5) the gRPC **client** adapter. The `cymbra-auth` module
//! depends on this crate to reach accounts — never on `cymbra-user` directly.

/// Generated protobuf messages + tonic client/server stubs for `cymbra.user.v1`.
pub mod proto {
    tonic::include_proto!("cymbra.user.v1");
}
