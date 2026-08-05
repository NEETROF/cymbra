fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generates the `cymbra.music.v1` protobuf types + tonic client/server stubs.
    // All `.proto` files share the package, so they compile into one generated
    // module in a single call (ScoreService — user uploads; PlayService — play-
    // activity stats; LeaderboardService — per-piece rankings). A separate call
    // per file would overwrite the shared output.
    tonic_build::configure().compile_protos(
        &[
            "proto/score.proto",
            "proto/play.proto",
            "proto/leaderboard.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
