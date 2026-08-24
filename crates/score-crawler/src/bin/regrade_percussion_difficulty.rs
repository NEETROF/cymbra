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

//! `regrade-percussion-difficulty` — the one-shot percussion re-grade pass
//! (change: add-drum-scoring).
//!
//! The difficulty heuristic used to read pitched features only, so on a drum
//! part every term counted zero notes and the row was graded Beginner *by
//! degeneracy*. The catalog `level` feeds one shared difficulty weight (play
//! rewards and the global season score), so those rows must be re-graded with
//! the instrument-aware heuristic **before** percussion becomes scorable —
//! otherwise the first paid drum runs meet dishonest weights and the drum corpus
//! becomes the cheapest farming target in the catalog.
//!
//! Scope and safety: catalog rows only, `instrument = 'percussion'` and
//! `level_source = 'heuristic'` only. A `source` or `manual` grade is never
//! read, never written; a re-graded row stays `heuristic`. A row whose stored
//! family disagrees with its bytes is left alone, so running this before
//! `backfill-instruments` under-covers but never mis-grades. Idempotent and
//! resumable: re-runs are all `unchanged`.
//!
//! It ships in the crawler image rather than the server's because the heuristic
//! it applies is the crawler's; the store/DB env contract is the server's, read
//! through the shared `cymbra-platform` config exactly like the sibling passes.
//!
//! Usage (dry run by default — prints what WOULD change, writes nothing):
//!     cargo run -p score-crawler --bin regrade-percussion-difficulty
//!     cargo run -p score-crawler --bin regrade-percussion-difficulty -- --apply

use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use cymbra_music::PgDifficultyBackfillRepo;
use cymbra_platform::config::Config;
use cymbra_storage::{LocalFirstStore, ObjectStorage, S3Params};
use score_crawler::backfill::run_percussion_regrade;
use score_crawler::cli::init_tracing;

#[derive(Parser, Debug)]
#[command(about = "Re-grade catalog percussion rows graded by the keyboard heuristic")]
struct Cli {
    /// Rows per keyset page.
    #[arg(long, default_value_t = 500)]
    page_size: i64,
    /// Actually write the re-grades. Without it this is a dry run (prints what
    /// would change, writes nothing) — the `backfill-*` convention.
    #[arg(long)]
    apply: bool,
    /// Verbose (debug) logging.
    #[arg(short, long)]
    verbose: bool,
}

/// The corpus object store, from the same `CYMBRA_SCORE_*` environment the
/// backend's maintenance passes read (sibling: `cymbra_server::maintenance::
/// score_object_store`). Rebuilt here rather than imported so this bin does not
/// drag the whole server composition root into the crawler image.
fn score_object_store(cfg: &Config) -> Result<Arc<dyn ObjectStorage>> {
    let s3 = cfg
        .score_storage
        .as_ref()
        .context("CYMBRA_SCORE_S3_BUCKET (+ credentials) is required to reach the score corpus")?;
    Ok(Arc::new(
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
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let _ = dotenvy::from_filename("backend/.env").or_else(|_| dotenvy::dotenv());
    init_tracing(cli.verbose);

    let cfg = Config::from_env()?;
    let db_url = cfg
        .music_database_url
        .as_deref()
        .context("CYMBRA_MUSIC_DATABASE_URL is required for the percussion re-grade")?;

    let pool = cymbra_music::connect(db_url, 4)
        .await
        .context("connecting to the music database")?;
    let storage = score_object_store(&cfg)?;
    let repo = PgDifficultyBackfillRepo::new(pool);

    if !cli.apply {
        tracing::warn!("DRY RUN — no writes. Re-run with --apply to persist the re-grades.");
    }
    let report = run_percussion_regrade(&repo, storage.as_ref(), cli.apply, cli.page_size).await?;
    println!(
        "percussion re-grade {} — scanned: {}, {}: {}, unchanged: {}, \
         not percussion (left as is): {}, unreadable (left as is): {}, errors: {}",
        if cli.apply { "APPLIED" } else { "DRY RUN" },
        report.scanned,
        if cli.apply {
            "re-graded"
        } else {
            "would re-grade"
        },
        report.updated,
        report.unchanged,
        report.not_percussion,
        report.unreadable,
        report.errors,
    );
    Ok(())
}
