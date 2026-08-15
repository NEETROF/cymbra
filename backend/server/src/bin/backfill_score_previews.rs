// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! `backfill-score-previews` — one-off maintenance command (change:
//! add-score-daily-access-rewards, design D7).
//!
//! Enqueues one `score_preview_render` job per ACCEPTED catalog piece that has no
//! rendered audio teaser yet (`preview_rendered_at IS NULL`), so the corpus
//! accepted before this change gets its clips without manual back-office clicks.
//! The worker renders them; the flags `catalog.preview.soundfont_id` /
//! `catalog.preview.max_ms` must be set or the jobs complete dormant.
//! Reuses the SAME env/config as `cymbra-server` (music DB, `music_svc` may
//! EXECUTE `jobs.enqueue`).
//!
//! Usage (dry run by default — prints how many WOULD be enqueued, writes nothing):
//!     cargo run -p cymbra-server --bin backfill-score-previews
//!     cargo run -p cymbra-server --bin backfill-score-previews -- --apply
//!     cargo run -p cymbra-server --bin backfill-score-previews -- --apply --limit 200

use anyhow::{Context, Result};
use cymbra_music::{CatalogSearchRepo, PgCatalogSearchRepo, enqueue_missing_previews};
use cymbra_platform::config::Config;

struct Opts {
    apply: bool,
    limit: i64,
}

fn parse_opts() -> Result<Opts> {
    let mut apply = false;
    let mut limit = 10_000i64;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--apply" => apply = true,
            "--limit" => {
                limit = args
                    .next()
                    .context("--limit needs a value")?
                    .parse()
                    .context("--limit must be an integer")?;
            }
            "-h" | "--help" => {
                println!(
                    "backfill-score-previews [--apply] [--limit <n>]\n\
                     \n  Dry run by default (no writes). --apply enqueues the render jobs.\n\
                     \n  --limit caps how many pieces are enqueued in one run (default 10000)."
                );
                std::process::exit(0);
            }
            other => anyhow::bail!("unknown argument: {other} (try --help)"),
        }
    }
    Ok(Opts { apply, limit })
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::from_filename("backend/.env").or_else(|_| dotenvy::dotenv());
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let opts = parse_opts()?;
    let cfg = Config::from_env()?;
    let db_url = cfg
        .music_database_url
        .as_deref()
        .context("CYMBRA_MUSIC_DATABASE_URL is required for the preview backfill")?;
    let pool = cymbra_music::connect(db_url, 4)
        .await
        .context("connecting to the music database")?;
    let catalog = PgCatalogSearchRepo::new(pool.clone());

    if !opts.apply {
        let ids = catalog.accepted_ids_missing_preview(opts.limit).await?;
        tracing::warn!("DRY RUN — no writes. Re-run with --apply to enqueue.");
        println!(
            "Backfill DRY RUN — {} accepted piece(s) without a preview (limit {})",
            ids.len(),
            opts.limit
        );
        return Ok(());
    }
    let enqueuer = cymbra_jobs::PgEnqueuer::new(pool);
    let ids = enqueue_missing_previews(&catalog, &enqueuer, opts.limit).await?;
    println!(
        "Backfill APPLIED — enqueued {} score_preview_render job(s) (limit {})",
        ids.len(),
        opts.limit
    );
    Ok(())
}
