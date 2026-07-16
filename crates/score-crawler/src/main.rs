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

//! `score-crawler` binary entry point: resolve config + sources, run each
//! enabled adapter through the license-first orchestrator, and write the vetted
//! corpus + manifests to the configured local output root.

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{info, warn};

use score_crawler::cli::{Cli, init_tracing};
use score_crawler::config::{Config, StoreBackend};
use score_crawler::output::OutputWriter;
use score_crawler::registry::build_adapters;
use score_crawler::run::run_all;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let mut config = if cli.config.exists() {
        Config::load(&cli.config)?
    } else {
        info!(path = %cli.config.display(), "no config file; using defaults");
        Config::default()
    };
    // Apply CYMBRA_SCORE_* env overrides even when there is no config file.
    config.apply_process_env();
    // Install the converter backend (local binaries vs Docker images).
    score_crawler::convert::init_converters(config.converters.to_converters());
    let limit = cli.limit.or(config.limit_per_source);

    let sources = cli.resolve_sources(&config);
    if sources.is_empty() {
        println!("No sources selected. Use --sources <a,b> or --all (see --help).");
        return Ok(());
    }

    let (root, safe_prefix, low_prefix) = match &config.store.backend {
        StoreBackend::LocalFs { root } => (
            root.clone(),
            config.store.safe_prefix.clone(),
            config.store.low_confidence_prefix.clone(),
        ),
        StoreBackend::S3 { .. } => {
            anyhow::bail!("S3 output backend not wired yet; use a local_fs store")
        }
    };

    let built = build_adapters(&sources, &root.join(".checkouts"));
    for name in &built.unsupported {
        warn!(source = %name, "adapter not implemented yet; skipping");
    }

    let outcome = run_all(&built.adapters, limit).await;
    let (summary, entries) = OutputWriter::new(&root, safe_prefix, low_prefix)
        .write(&outcome)
        .context("writing corpus output")?;

    // Ingest the provenance into the shared `score` catalog when configured.
    let ingested = if let Some(url) = &config.catalog_database_url {
        let pool = cymbra_music::connect(url, 4)
            .await
            .context("connecting to the score catalog database")?;
        cymbra_music::MIGRATOR
            .run(&pool)
            .await
            .context("running score catalog migrations")?;
        let repo = cymbra_music::PgCatalogRepo::new(pool);
        Some(
            score_crawler::catalog::ingest(&repo, &entries)
                .await
                .context("ingesting into the score catalog")?,
        )
    } else {
        None
    };

    println!(
        "Done — safe: {}, low-confidence: {}, rejected/failed: {}. Output: {}",
        summary.safe,
        summary.low_confidence,
        summary.rejected,
        root.display()
    );
    match ingested {
        Some(n) => println!("Catalog rows inserted: {n}"),
        None => println!("(no catalog DB configured — set CYMBRA_SCORE_DATABASE_URL to ingest)"),
    }
    Ok(())
}
