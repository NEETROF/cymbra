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

//! `backfill-titles` — one-off maintenance command.
//!
//! Recomputes catalog display titles from each score's stored `.mxl`, repairing
//! rows a git corpus ingested under an opaque filename id (OpenScore Lieder:
//! `lc28971056`) instead of the embedded `<work-title>`. Reuses the SAME env/config
//! and object store as `cymbra-server`, so it reads exactly the bytes the app
//! serves. See `cymbra_music::backfill` for the logic (title + `title_norm` +
//! `work_key` are rewritten together so search stays consistent).
//!
//! Usage (dry run by default — prints what WOULD change, writes nothing):
//!     cargo run -p cymbra-server --bin backfill-titles
//!     cargo run -p cymbra-server --bin backfill-titles -- --source openscore
//!     cargo run -p cymbra-server --bin backfill-titles -- --apply         # writes
//!     cargo run -p cymbra-server --bin backfill-titles -- --apply --source openscore

use std::sync::Arc;

use anyhow::{Context, Result};
use cymbra_music::{PgTitleBackfillRepo, run_title_backfill};
use cymbra_platform::config::Config;
use cymbra_storage::{LocalFirstStore, ObjectStorage, S3Params};

/// Parsed command-line options.
struct Opts {
    apply: bool,
    source: Option<String>,
    page_size: i64,
}

fn parse_opts() -> Result<Opts> {
    let mut apply = false;
    let mut source = None;
    let mut page_size = 500i64;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--apply" => apply = true,
            "--source" => {
                source = Some(args.next().context("--source needs a value")?);
            }
            "--page-size" => {
                page_size = args
                    .next()
                    .context("--page-size needs a value")?
                    .parse()
                    .context("--page-size must be an integer")?;
            }
            "-h" | "--help" => {
                println!(
                    "backfill-titles [--apply] [--source <name>] [--page-size <n>]\n\
                     \n  Dry run by default (no writes). --apply persists the rewrites.\n\
                     \n  --source scopes the scan to one provenance, e.g. openscore."
                );
                std::process::exit(0);
            }
            other => anyhow::bail!("unknown argument: {other} (try --help)"),
        }
    }
    Ok(Opts {
        apply,
        source,
        page_size,
    })
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
        .context("CYMBRA_MUSIC_DATABASE_URL is required for the title backfill")?;
    let s3 = cfg
        .score_storage
        .as_ref()
        .context("CYMBRA_SCORE_S3_BUCKET (+ credentials) is required to read stored scores")?;

    let pool = cymbra_music::connect(db_url, 4)
        .await
        .context("connecting to the music database")?;
    let storage: Arc<dyn ObjectStorage> = Arc::new(
        LocalFirstStore::from_config(
            &cfg.score_local_root,
            &S3Params {
                bucket: s3.bucket.clone(),
                endpoint: s3.endpoint.clone(),
                region: s3.region.clone(),
                access_key: s3.access_key.clone(),
                secret_key: s3.secret_key.clone(),
                allow_http: s3.allow_http,
            },
        )
        .context("building the score object store")?,
    );

    let repo = PgTitleBackfillRepo::new(pool);
    if !opts.apply {
        tracing::warn!("DRY RUN — no writes. Re-run with --apply to persist changes.");
    }
    let report = run_title_backfill(
        &repo,
        storage.as_ref(),
        opts.source.as_deref(),
        opts.apply,
        opts.page_size,
    )
    .await?;

    println!(
        "Backfill {} — scanned: {}, {}: {}, unchanged: {}, no embedded title: {}, errors: {}",
        if opts.apply { "APPLIED" } else { "DRY RUN" },
        report.scanned,
        if opts.apply {
            "rewritten"
        } else {
            "would rewrite"
        },
        report.updated,
        report.unchanged,
        report.no_title,
        report.errors,
    );
    Ok(())
}
