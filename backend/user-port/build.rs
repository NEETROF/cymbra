fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generates the `cymbra.user.v1` protobuf types + tonic client/server stubs.
    tonic_build::compile_protos("proto/user.proto")?;
    Ok(())
}
