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

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{info, warn};

use score_crawler::cli::{Cli, init_tracing};
use score_crawler::config::{Config, StoreBackend};
use score_crawler::crawl::{CrawlOutcome, Orchestrator};
use score_crawler::http::{Fetcher, HttpFetcher};
use score_crawler::output::OutputWriter;
use score_crawler::registry::build_adapters;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let config = if cli.config.exists() {
        Config::load(&cli.config)?
    } else {
        info!(path = %cli.config.display(), "no config file; using defaults");
        Config::default()
    };

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
            // S3/catalog ingestion lands with the backend score module.
            anyhow::bail!("S3 output backend not wired yet; use a local_fs store");
        }
    };

    let fetcher: Arc<dyn Fetcher> = Arc::new(
        HttpFetcher::new(config.user_agent(), Duration::from_millis(config.delay_ms))
            .context("building HTTP fetcher")?,
    );
    let checkout_root: PathBuf = root.join(".checkouts");
    let built = build_adapters(&sources, fetcher, &checkout_root);
    for name in &built.unsupported {
        warn!(source = %name, "adapter not implemented yet; skipping");
    }

    let limit = cli.limit.or(config.limit_per_source);
    let mut orchestrator = Orchestrator::new();
    let mut merged = CrawlOutcome::default();

    for adapter in &built.adapters {
        info!(source = adapter.name(), "preparing");
        if let Err(e) = adapter.prepare().await {
            warn!(source = adapter.name(), error = %e, "prepare failed; skipping source");
            continue;
        }
        let out = orchestrator.run(adapter.as_ref(), limit).await;
        info!(
            source = adapter.name(),
            accepted = out.stats.accepted,
            low_confidence = out.stats.low_confidence,
            rejected = out.stats.rejected,
            failed = out.stats.failed,
            "source complete"
        );
        merge(&mut merged, out);
    }

    let writer = OutputWriter::new(&root, safe_prefix, low_prefix);
    let summary = writer.write(&merged).context("writing corpus output")?;

    println!(
        "Done — safe: {}, low-confidence: {}, rejected/failed: {}. Output: {}",
        summary.safe,
        summary.low_confidence,
        summary.rejected,
        root.display()
    );
    Ok(())
}

/// Folds one adapter's outcome into the merged totals.
fn merge(into: &mut CrawlOutcome, from: CrawlOutcome) {
    into.prepared.extend(from.prepared);
    into.rejected.extend(from.rejected);
    into.stats.discovered += from.stats.discovered;
    into.stats.accepted += from.stats.accepted;
    into.stats.low_confidence += from.stats.low_confidence;
    into.stats.rejected += from.stats.rejected;
    into.stats.failed += from.stats.failed;
    into.stats.deduped += from.stats.deduped;
}
