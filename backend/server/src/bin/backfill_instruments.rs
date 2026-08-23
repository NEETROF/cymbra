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

//! `backfill-instruments` — the one-shot instrument re-derivation pass
//! (change: add-drums-access).
//!
//! Streams every catalog and user score's stored bytes back from the object
//! store, classifies each with the shared parser, and persists the derived
//! family (`keyboard` | `percussion` | `unknown`). This CANNOT be a SQL
//! migration (the tables hold an `object_key`, not the bytes), and translating
//! the retired `is_piano` flag would be wrong: the corpus already holds
//! percussion ingested despite the old gate, which a translation would record
//! `unknown` — and the drum gate serves `unknown` to everyone. **The drum flag
//! override must not be switched on before this pass has completed**; until
//! then real percussion rows may still read `unknown` and the gate is not a
//! boundary. Idempotent and resumable: re-runs are all `unchanged`.
//!
//! Usage (dry run by default — prints what WOULD change, writes nothing):
//!     cargo run -p cymbra-server --bin backfill-instruments
//!     cargo run -p cymbra-server --bin backfill-instruments -- --apply

use anyhow::{Context, Result};
use cymbra_music::{PgInstrumentBackfillRepo, ScoreTable, run_instrument_backfill};
use cymbra_platform::config::Config;
use cymbra_server::maintenance::score_object_store;

/// Parsed command-line options.
struct Opts {
    apply: bool,
    page_size: i64,
}

fn parse_opts() -> Result<Opts> {
    let mut apply = false;
    let mut page_size = 500i64;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--apply" => apply = true,
            "--page-size" => {
                page_size = args
                    .next()
                    .context("--page-size needs a value")?
                    .parse()
                    .context("--page-size must be an integer")?;
            }
            "-h" | "--help" => {
                println!(
                    "backfill-instruments [--apply] [--page-size <n>]\n\
                     \n  Dry run by default (no writes). --apply persists the re-derivation.\n\
                     \n  Covers BOTH music.catalog_scores and music.user_scores."
                );
                std::process::exit(0);
            }
            other => anyhow::bail!("unknown argument: {other} (try --help)"),
        }
    }
    Ok(Opts { apply, page_size })
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
        .context("CYMBRA_MUSIC_DATABASE_URL is required for the instrument backfill")?;

    let pool = cymbra_music::connect(db_url, 4)
        .await
        .context("connecting to the music database")?;
    let storage = score_object_store(&cfg)?;

    let repo = PgInstrumentBackfillRepo::new(pool);
    if !opts.apply {
        tracing::warn!("DRY RUN — no writes. Re-run with --apply to persist changes.");
    }
    for (label, table) in [
        ("catalog_scores", ScoreTable::Catalog),
        ("user_scores", ScoreTable::User),
    ] {
        let report =
            run_instrument_backfill(&repo, storage.as_ref(), table, opts.apply, opts.page_size)
                .await?;
        println!(
            "{label} {} — scanned: {}, {}: {}, unchanged: {}, unreadable (left as is): {}, errors: {}",
            if opts.apply { "APPLIED" } else { "DRY RUN" },
            report.scanned,
            if opts.apply {
                "rewritten"
            } else {
                "would rewrite"
            },
            report.updated,
            report.unchanged,
            report.unreadable,
            report.errors,
        );
    }
    Ok(())
}
