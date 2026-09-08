//! Start-up shared by the one-off maintenance binaries.
//!
//! They run outside the server, so each has to load the environment, install a log
//! subscriber and open its own pool. That preamble was copied into all five, and the
//! move in group 5 turned it into flagged duplication — which is fair: it was always
//! duplicated, it was just spread across a directory nobody read end to end.

use anyhow::Context;
use cymbra_platform::config::Config;
use sqlx::PgPool;

/// Load `backend/.env` (falling back to the ambient one) and install the log
/// subscriber, honouring `RUST_LOG` with an `info` default.
pub fn init() {
    let _ = dotenvy::from_filename("backend/.env").or_else(|_| dotenvy::dotenv());
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}

/// Open a small music pool. `purpose` completes "…is required for {purpose}", so a
/// missing URL still says which pass could not run. `max_conns` stays explicit: the
/// read-only passes deliberately take fewer than the writing ones.
pub async fn music_pool(cfg: &Config, purpose: &str, max_conns: u32) -> anyhow::Result<PgPool> {
    let db_url = cfg
        .music_database_url
        .as_deref()
        .with_context(|| format!("CYMBRA_MUSIC_DATABASE_URL is required for {purpose}"))?;
    crate::connect(db_url, max_conns)
        .await
        .context("connecting to the music database")
}
