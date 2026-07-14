fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generates the `cymbra.music.v1` protobuf types + tonic client/server stubs
    // for the ScoreService (user uploads).
    tonic_build::compile_protos("proto/score.proto")?;
    Ok(())
}
