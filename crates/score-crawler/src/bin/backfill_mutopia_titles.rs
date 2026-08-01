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

//! One-off backfill: re-derive already-crawled Mutopia catalog titles from their
//! source `.ly` `\header`.
//!
//! The crawler skips content it has already ingested (dedup by sha/fingerprint,
//! `ON CONFLICT DO NOTHING`), so re-running it does NOT fix rows titled from the
//! old filename fallback ("bwv 1001 1"). The shared stored-bytes title backfill
//! can't fix them either — the converted `.mxl` has no `<work-title>`. This tool
//! reads each row's source `.ly` from a MutopiaProject checkout and updates the
//! title (and its search keys) in place, never touching a curator-edited row. Safe
//! to re-run; `--dry-run` reports without writing.
//!
//! Config mirrors the crawler: `CYMBRA_SCORE_DATABASE_URL` for the catalog DB; the
//! checkout defaults to the local store's `.checkouts/mutopia` (override with
//! `--checkout`).

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use tracing::info;

use cymbra_music::PgTitleBackfillRepo;
use score_crawler::backfill::run_mutopia_title_backfill;
use score_crawler::cli::init_tracing;
use score_crawler::config::{Config, StoreBackend};
use score_crawler::sources::SourceAdapter;
use score_crawler::sources::mutopia::MutopiaSource;

#[derive(Parser, Debug)]
#[command(about = "Backfill Mutopia catalog titles from the source .ly headers")]
struct Cli {
    /// Config file (optional; `CYMBRA_SCORE_*` env still applies without one).
    #[arg(long, default_value = "config.yaml")]
    config: PathBuf,
    /// MutopiaProject checkout dir (defaults to the local store's
    /// `.checkouts/mutopia`; cloned/updated if absent).
    #[arg(long)]
    checkout: Option<PathBuf>,
    /// Rows per keyset page.
    #[arg(long, default_value_t = 500)]
    page_size: i64,
    /// Actually write the updates. Without it this is a dry run (prints what would
    /// change, writes nothing) — matching the `backfill-titles` convention.
    #[arg(long)]
    apply: bool,
    /// Verbose (debug) logging.
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let mut config = if cli.config.exists() {
        Config::load(&cli.config)?
    } else {
        Config::default()
    };
    config.apply_process_env();

    let checkout = cli
        .checkout
        .or_else(|| match &config.store.backend {
            StoreBackend::LocalFs { root } => Some(root.join(".checkouts").join("mutopia")),
            StoreBackend::S3 { .. } => None,
        })
        .context("no checkout dir: pass --checkout or configure a local_fs store")?;

    let url = config
        .catalog_database_url
        .clone()
        .context("no catalog DB: set CYMBRA_SCORE_DATABASE_URL")?;

    // Ensure the MutopiaProject checkout exists (clone/update), so every row's
    // `source_item_id` resolves to a real `.ly`.
    info!(checkout = %checkout.display(), "preparing Mutopia checkout");
    MutopiaSource::new(&checkout)
        .prepare()
        .await
        .context("preparing Mutopia checkout")?;

    let pool = cymbra_music::connect(&url, 4)
        .await
        .context("connecting to the score catalog database")?;
    cymbra_music::MIGRATOR
        .run(&pool)
        .await
        .context("running score catalog migrations")?;
    let repo = PgTitleBackfillRepo::new(pool);

    let report = run_mutopia_title_backfill(&repo, &checkout, cli.apply, cli.page_size).await?;

    let verb = if cli.apply { "updated" } else { "would update" };
    println!(
        "Mutopia title backfill — scanned: {}, {verb}: {}, unchanged: {}, no title: {}, errors: {}",
        report.scanned, report.updated, report.unchanged, report.no_title, report.errors
    );
    Ok(())
}
