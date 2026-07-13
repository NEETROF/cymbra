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

//! `score-crawler` binary entry point.

use anyhow::Result;
use clap::Parser;
use tracing::info;

use score_crawler::cli::{Cli, init_tracing};
use score_crawler::config::Config;

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let config = if cli.config.exists() {
        Config::load(&cli.config)?
    } else {
        info!(path = %cli.config.display(), "no config file; using defaults");
        Config::default()
    };

    let sources = cli.resolve_sources(&config);
    info!(
        ?sources,
        limit = ?cli.limit,
        resume = cli.resume,
        user_agent = %config.user_agent(),
        "resolved crawl plan"
    );

    if sources.is_empty() {
        println!("No sources selected. Use --sources <a,b> or --all (see --help).");
        return Ok(());
    }

    // The concrete source adapters (network/git) and the object-store +
    // catalog ingestion attach in the next slice; the licence gate, conversion,
    // metadata/difficulty, and orchestration engine are complete and unit-tested
    // (see `cargo test -p score-crawler`). Report the resolved plan for now.
    println!(
        "score-crawler — {} source(s): {}",
        sources.len(),
        sources.join(", ")
    );
    println!("engine ready (licence gate · conversion · metadata · difficulty · orchestrator);");
    println!("adapters + ingestion land next.");
    Ok(())
}
