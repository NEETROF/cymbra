// `build_client(false)`: nothing in this workspace calls a module over gRPC.
// The boundary between modules is a Rust trait, and an internal transport is
// written when a module is actually split out — not kept warm in case (change:
// harden-module-boundaries, group 7). The apps generate their own Dart/TS
// clients from the same `.proto`, which is the external contract.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generates the `cymbra.music.v1` protobuf types + the tonic SERVER stub.
    // All `.proto` files share the package, so they compile into one generated
    // module in a single call (ScoreService — user uploads; PlayService — play-
    // activity stats; LeaderboardService — per-piece rankings;
    // GlobalLeaderboardService — the seasonal global rankings). A separate call
    // per file would overwrite the shared output.
    tonic_build::configure()
        .build_client(false)
        .compile_protos(
            &[
                "proto/score.proto",
                "proto/play.proto",
                "proto/leaderboard.proto",
                "proto/global_leaderboard.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}
