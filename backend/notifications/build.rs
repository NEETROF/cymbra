fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generates the `cymbra.notifications.v1` protobuf types + tonic client/server
    // stubs (NotificationService — token registry, consent, timezone).
    tonic_build::configure().compile_protos(&["proto/notifications.proto"], &["proto"])?;
    Ok(())
}
