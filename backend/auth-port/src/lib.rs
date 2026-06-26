//! `cymbra-auth-port` — the auth module's **contract** crate.
//!
//! This crate carries the port trait, its DTOs, the generated protobuf types,
//! and (task 4.10) the gRPC **client** adapter. Consumers — including the future
//! product backends — depend on this crate only, never on `cymbra-auth` (the
//! implementation). See `openspec/changes/add-cymbra-id` design D0/D1.

/// Generated protobuf messages + tonic client/server stubs for `cymbra.auth.v1`.
pub mod proto {
    tonic::include_proto!("cymbra.auth.v1");
}
