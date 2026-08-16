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

//! `reconcile-corpus` — one-off maintenance command.
//!
//! Finds corpus objects that no `catalog_scores` row references and, with
//! `--apply`, moves them to a quarantine prefix (reversible until `--purge`).
//!
//! Object keys used to embed a per-run UUID, so every re-crawl of unchanged
//! content wrote a second object while ingest deduplicated the row away;
//! production accumulated ~145 430 unreferenced objects, about half the corpus,
//! mirrored to S3 as well (change: fix-crawler-corpus-isolation). Keys are
//! content-derived now, so this cleans up the backlog rather than a live leak.
//!
//! Reuses the SAME env/config and object store as `cymbra-server`, so it sees
//! exactly the keyspace the server reads (local-first, S3 origin). The logic
//! lives in `cymbra_music::reconcile`; this is wiring.
//!
//! Usage (dry run by default — prints what WOULD move, writes nothing):
//!     cargo run -p cymbra-server --bin reconcile-corpus
//!     cargo run -p cymbra-server --bin reconcile-corpus -- --apply
//!     cargo run -p cymbra-server --bin reconcile-corpus -- --purge          # irreversible
//!     cargo run -p cymbra-server --bin reconcile-corpus -- --apply --max-removal-ratio 0.8

use std::sync::Arc;

use anyhow::{Context, Result};
use cymbra_music::pg::PgReconcileRepo;
use cymbra_music::reconcile::{QUARANTINE_PREFIX, ReconcileOptions, run_reconcile};
use cymbra_platform::config::Config;
use cymbra_storage::{LocalFirstStore, ObjectStorage, S3Params};

/// Parsed command-line options.
struct Opts {
    apply: bool,
    purge: bool,
    max_removal_ratio: f64,
    page_size: i64,
}

fn parse_opts() -> Result<Opts> {
    let mut o = Opts {
        apply: false,
        purge: false,
        max_removal_ratio: ReconcileOptions::default().max_removal_ratio,
        page_size: ReconcileOptions::default().page_size,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--apply" => o.apply = true,
            "--purge" => o.purge = true,
            "--max-removal-ratio" => {
                o.max_removal_ratio = args
                    .next()
                    .context("--max-removal-ratio needs a value")?
                    .parse()
                    .context("--max-removal-ratio must be a fraction, e.g. 0.75")?;
            }
            "--page-size" => {
                o.page_size = args
                    .next()
                    .context("--page-size needs a value")?
                    .parse()
                    .context("--page-size must be an integer")?;
            }
            "-h" | "--help" => {
                println!(
                    "reconcile-corpus [--apply] [--purge] [--max-removal-ratio <f>] [--page-size <n>]\n\
                     \n  Reports corpus objects no catalog row references.\n\
                     \n  Dry run by default (no writes).\
                     \n  --apply  moves them to `{QUARANTINE_PREFIX}` (reversible).\
                     \n  --purge  permanently deletes what is already quarantined.\
                     \n  --max-removal-ratio aborts if the share to remove exceeds it (default {:.2}).",
                    ReconcileOptions::default().max_removal_ratio
                );
                std::process::exit(0);
            }
            other => anyhow::bail!("unknown argument: {other} (try --help)"),
        }
    }
    anyhow::ensure!(
        !(o.apply && o.purge),
        "--apply and --purge are separate steps: quarantine first, review, then purge"
    );
    Ok(o)
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
        .context("CYMBRA_MUSIC_DATABASE_URL is required to read the catalog")?;
    let s3 = cfg
        .score_storage
        .as_ref()
        .context("CYMBRA_SCORE_S3_BUCKET (+ credentials) is required to reach the corpus")?;

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

    if opts.purge {
        return purge(storage.as_ref()).await;
    }

    if !opts.apply {
        tracing::warn!("DRY RUN — no writes. Re-run with --apply to quarantine.");
    }

    let repo = PgReconcileRepo::new(pool);
    let report = run_reconcile(
        storage.as_ref(),
        &repo,
        &ReconcileOptions {
            apply: opts.apply,
            max_removal_ratio: opts.max_removal_ratio,
            page_size: opts.page_size,
        },
    )
    .await?;

    println!(
        "corpus objects: {}\nreferenced by a row: {}\nunreferenced: {}\nquarantined: {}",
        report.objects_seen,
        report.referenced,
        report.unreferenced.len(),
        report.quarantined
    );
    // A sample, not the whole list: 145k lines helps nobody.
    for key in report.unreferenced.iter().take(20) {
        println!("  {key}");
    }
    if report.unreferenced.len() > 20 {
        println!("  … and {} more", report.unreferenced.len() - 20);
    }
    if !opts.apply && !report.unreferenced.is_empty() {
        println!("\nRe-run with --apply to move these to `{QUARANTINE_PREFIX}`.");
    }
    Ok(())
}

/// Permanently deletes everything under the quarantine prefix. Separate, explicit
/// and irreversible — the whole point of quarantining first.
async fn purge(storage: &dyn ObjectStorage) -> Result<()> {
    let keys = storage
        .list(QUARANTINE_PREFIX)
        .await
        .context("listing the quarantine")?;
    tracing::warn!(
        count = keys.len(),
        "purging quarantined objects — irreversible"
    );
    for key in &keys {
        storage
            .delete(key)
            .await
            .with_context(|| format!("purging {key}"))?;
    }
    println!("purged {} quarantined objects", keys.len());
    Ok(())
}
