//! `cymbra-music` — the music-domain module.
//!
//! Owns the `music` Postgres schema: the whole music app's data (scores today,
//! practice/performance/etc. later) lives here, isolated from identity/auth and
//! jobs but sharing one schema + role so its tables can reference each other with
//! real FKs. Today it holds `catalog_scores` — the public, crawler-ingested
//! redistributable corpus — behind a [`repo::CatalogRepo`] (Postgres or an
//! in-memory fake), plus the user-upload `user_scores` + gRPC surface.
//!
//! The catalog surface is `anyhow`-based (it is written to directly by the
//! score-crawler tool, not over gRPC), so consumers need not depend on the
//! platform error type.

pub mod backfill;
pub mod catalog_edit;
pub mod catalog_limits;
pub mod catalog_search;
pub mod course;
pub mod curation_rewards;
pub mod curation_rewards_core;
pub mod curation_rewards_module;
pub mod global_leaderboard;
pub mod global_leaderboard_core;
pub mod global_leaderboard_grpc;
pub mod global_leaderboard_module;
pub mod grpc;
pub mod leaderboard;
pub mod leaderboard_core;
pub mod leaderboard_grpc;
pub mod leaderboard_module;
pub mod module;
pub mod pg;
pub mod pg_curation_rewards;
pub mod pg_global_leaderboard;
pub mod pg_leaderboard;
pub mod pg_play;
pub mod pg_user_scores;
pub mod play;
pub mod play_core;
pub mod play_grpc;
pub mod play_module;
pub mod repo;
pub mod score_rating;
pub mod soundfont;
pub mod soundfont_access;
pub mod soundfont_preview;
pub mod soundfont_synth;
pub mod user_library;
pub mod user_scores;
pub mod user_soundfont;

pub use backfill::{
    BackfillReport, BackfillRow, TitleBackfillRepo, TitleUpdate, plan_title_update,
    run_title_backfill,
};
pub use catalog_limits::CatalogAccessLimiter;
pub use catalog_search::{
    CatalogHit, CatalogSearchParams, CatalogSearchRepo, FakeCatalogRow, FakeCatalogSearchRepo,
};
pub use course::{Course, CourseRepo, CourseSummary, FakeCourseRepo, PgCourseRepo};
pub use curation_rewards::{
    ConsensusCandidate, CurationRewardsRepo, CurationRewardsSink, CuratorMetrics,
    FakeCurationRewardsRepo, GrantKind, LedgerEntry, SettleOutcome, SettleableRating, ShopItem,
};
pub use curation_rewards_core::{AwardKind, BADGES, BadgeDef, BadgeMetric, RewardConfig};
pub use curation_rewards_module::{CurationRewardsModule, CuratorRewards, RedeemResult};
pub use global_leaderboard::{
    FakeGlobalLeaderboardRepo, GlobalLeaderboardRepo, GlobalScore, GlobalSeasonBest,
    GlobalSeasonSink, SeasonBestRow,
};
pub use global_leaderboard_core::{GlobalConfig, Season};
pub use global_leaderboard_grpc::GlobalLeaderboardGrpc;
pub use global_leaderboard_module::{
    GlobalBoard, GlobalEntry, GlobalLeaderboardModule, Page, Seasons, snapshot_closed_season,
};
pub use grpc::ScoreGrpc;
pub use leaderboard::{
    BestCandidate, FakeLeaderboardRepo, LeaderboardBest, LeaderboardRepo, LeaderboardSink, Mode,
    StoredBest,
};
pub use leaderboard_grpc::LeaderboardGrpc;
pub use leaderboard_module::{Board, BoardEntry, LeaderboardModule, MyStanding};
pub use module::{ScoreModule, UploadInput};
pub use pg::{PgCatalogRepo, PgCatalogSearchRepo, PgScoreRatingRepo, PgTitleBackfillRepo};
pub use pg_curation_rewards::PgCurationRewardsRepo;
pub use pg_global_leaderboard::PgGlobalLeaderboardRepo;
pub use pg_leaderboard::PgLeaderboardRepo;
pub use pg_play::PgPlayRepo;
pub use pg_user_scores::{PgUserLibraryRepo, PgUserScoreRepo};
pub use play::{DayActivity, FakePlayRepo, PlayActivity, PlayRepo, PlaySession, SessionPoint};
pub use play_grpc::PlayGrpc;
pub use play_module::{PlayModule, RecordInput};
pub use repo::{CatalogEntry, CatalogRepo, FakeCatalogRepo, ScoreFacets, ScoreMeta};
pub use score_rating::{
    FakeScoreRatingRepo, RatingAggregate, RatingConfig, ScoreRatingRepo, Verdict,
};
pub use soundfont::{
    FakeSoundFontRepo, FontEntry, PgSoundFontRepo, SoundFontRepo, SoundFontStatusCounts, sha256_hex,
};
pub use soundfont_access::{Access, entitlement};
pub use soundfont_preview::{
    Event, Note, PREVIEW_SAMPLE_RATE, SampleSequence, encode_preview, preview_object_key,
    sample_sequence, scheduled_events, total_samples,
};
pub use soundfont_synth::{render_preview_pcm, render_preview_wav};
pub use user_library::{FakeUserLibraryRepo, UserLibraryRepo};
pub use user_scores::{FakeUserScoreRepo, UserScore, UserScoreRepo};
pub use user_soundfont::{
    FakeUserSoundFontRepo, PgUserSoundFontRepo, UserFontEntry, UserSoundFontRepo,
};

/// Generated protobuf messages + tonic client/server stubs for `cymbra.music.v1`
/// (the ScoreService — user uploads).
pub mod proto {
    tonic::include_proto!("cymbra.music.v1");
}

/// The module's Postgres schema.
pub const SCHEMA: &str = "music";

/// Embedded migrations for the `music` schema.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Connects a Postgres pool (used by the crawler's direct-ingestion path).
///
/// Pins `search_path = music` on every connection so that whatever role the
/// crawler uses (often a superuser via `CYMBRA_SCORE_DATABASE_URL`) records the
/// `_sqlx_migrations` ledger in the **same** `music` schema as the server's
/// `music_svc` pool — one shared ledger, so the second `MIGRATOR.run` only applies
/// new versions instead of forking a second ledger in `public` (design 5 / Option A).
pub async fn connect(database_url: &str, max_connections: u32) -> anyhow::Result<sqlx::PgPool> {
    use sqlx::Executor;
    Ok(sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET search_path = music").await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await?)
}
