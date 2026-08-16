fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generates the `cymbra.plans.v1` protobuf types + tonic client/server stubs
    // for PlanService (client read + redeem + purchases, and the admin surface).
    tonic_build::configure().compile_protos(&["proto/plans.proto"], &["proto"])?;
    Ok(())
}
