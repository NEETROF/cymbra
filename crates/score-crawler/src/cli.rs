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

//! Command-line interface + structured logging setup.

use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::sources::ALL_SOURCES;

/// A polite, resumable crawler for redistributable scores.
#[derive(Debug, Clone, Parser)]
#[command(name = "score-crawler", version, about)]
pub struct Cli {
    /// Comma-separated sources to crawl, e.g. `--sources openscore,mutopia`.
    #[arg(long, value_delimiter = ',')]
    pub sources: Option<Vec<String>>,
    /// Cap items per source (test runs).
    #[arg(long)]
    pub limit: Option<usize>,
    /// Crawl every known source.
    #[arg(long)]
    pub all: bool,
    /// Resume from the on-disk state cache (skip completed items).
    #[arg(long)]
    pub resume: bool,
    /// Launch the interactive terminal UI (pick sources, watch progress).
    #[arg(long)]
    pub tui: bool,
    /// Verbose (DEBUG) logging.
    #[arg(long)]
    pub verbose: bool,
    /// Path to `config.yaml`.
    #[arg(long, default_value = "config.yaml")]
    pub config: PathBuf,
}

impl Cli {
    /// Resolves which sources to crawl: `--all` wins, then `--sources`, then the
    /// config's `sources`. Unknown names are dropped (with the caller warned).
    pub fn resolve_sources(&self, cfg: &Config) -> Vec<String> {
        let requested: Vec<String> = if self.all {
            ALL_SOURCES.iter().map(|s| s.to_string()).collect()
        } else if let Some(s) = &self.sources {
            s.clone()
        } else {
            cfg.sources.clone()
        };
        requested
            .into_iter()
            .filter(|s| ALL_SOURCES.contains(&s.as_str()))
            .collect()
    }
}

/// Initialises `tracing` at INFO (or DEBUG under `--verbose`), honouring
/// `RUST_LOG` when set.
pub fn init_tracing(verbose: bool) {
    let default = if verbose { "debug" } else { "info" };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));
    // `try_init` so repeated calls in tests don't panic.
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli() -> Cli {
        Cli {
            sources: None,
            limit: None,
            all: false,
            resume: false,
            tui: false,
            verbose: false,
            config: "config.yaml".into(),
        }
    }

    #[test]
    fn all_expands_to_every_known_source() {
        let c = Cli { all: true, ..cli() };
        assert_eq!(
            c.resolve_sources(&Config::default()).len(),
            ALL_SOURCES.len()
        );
    }

    #[test]
    fn explicit_sources_win_and_unknown_dropped() {
        let c = Cli {
            sources: Some(vec!["openscore".into(), "bogus".into()]),
            ..cli()
        };
        assert_eq!(c.resolve_sources(&Config::default()), vec!["openscore"]);
    }

    #[test]
    fn falls_back_to_config_sources() {
        let cfg = Config {
            sources: vec!["mutopia".into()],
            ..Config::default()
        };
        assert_eq!(cli().resolve_sources(&cfg), vec!["mutopia"]);
    }

    #[test]
    fn cli_parses_flags() {
        let c = Cli::try_parse_from([
            "score-crawler",
            "--sources",
            "openscore,mutopia",
            "--limit",
            "10",
            "--resume",
            "--verbose",
        ])
        .unwrap();
        assert_eq!(c.sources.as_deref().unwrap().len(), 2);
        assert_eq!(c.limit, Some(10));
        assert!(c.resume && c.verbose);
    }
}
