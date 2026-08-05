fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generates the `cymbra.analytics.v1` protobuf types + tonic client/server
    // stubs (UsageService — batched feature-usage ingestion).
    tonic_build::configure().compile_protos(&["proto/usage.proto"], &["proto"])?;
    Ok(())
}
