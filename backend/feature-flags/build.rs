fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generates the `cymbra.flags.v1` protobuf types + tonic client/server stubs
    // for the shared, app-agnostic FlagService (client read + admin edits).
    tonic_build::configure().compile_protos(&["proto/flags.proto"], &["proto"])?;
    Ok(())
}
