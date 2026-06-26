//! Cymbra ID — composition root (binary `cymbra-id`).
//!
//! The **only** place modules are wired: it builds platform, constructs each
//! module's direct adapter, injects the `user` port into `auth`, installs the
//! internal-token interceptor, mounts the gRPC server adapters + the Axum
//! JWKS/health surface, and owns process lifecycle. Contains no business logic.
//!
//! Wiring lands in task group 5; this scaffold establishes the binary.

fn main() -> anyhow::Result<()> {
    // Reference each wired module so the dependency graph is real and visible.
    let _identity = cymbra_platform::AuthIdentity::default();
    println!(
        "cymbra-id scaffold — modules: auth (schema `{}`), user (schema `{}`); wiring lands in group 5",
        cymbra_auth::SCHEMA,
        cymbra_user::SCHEMA,
    );
    Ok(())
}
