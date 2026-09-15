// `build_client(false)`: nothing in this workspace calls a module over gRPC.
// The boundary between modules is a Rust trait, and an internal transport is
// written when a module is actually split out — not kept warm in case (change:
// harden-module-boundaries, group 7). The back office generates its own TS client
// from the same `.proto`, which is the external contract.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generates the `cymbra.jobs.v1` protobuf types + the tonic SERVER stub
    // (JobsAdminService — the back-office Jobs console; change: add-admin-jobs-console).
    tonic_build::configure()
        .build_client(false)
        .compile_protos(&["proto/jobs_admin.proto"], &["proto"])?;
    Ok(())
}
