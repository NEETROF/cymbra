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

//! Typed `config.yaml` for the crawler.
//!
//! Declares enabled sources, politeness knobs (per-host delay, concurrency,
//! contact), per-source quotas for test runs, and the object-store target
//! (local filesystem folder in dev ⇄ S3/MinIO in prod, matching the shared
//! `CYMBRA_SCORE_S3_*` config the score module introduces). Env vars override
//! the store backend so dev/prod differ by environment, not by editing the file.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Top-level crawler configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Enabled source names (empty = none until selected on the CLI).
    #[serde(default)]
    pub sources: Vec<String>,
    /// Per-host delay between requests (politeness). Default 2000 ms.
    #[serde(default = "default_delay_ms")]
    pub delay_ms: u64,
    /// Max concurrent in-flight requests. Default 2.
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    /// Contact (email/URL) embedded in the descriptive User-Agent.
    #[serde(default = "default_contact")]
    pub contact: String,
    /// Per-source item cap for test runs (`None` = unbounded).
    #[serde(default)]
    pub limit_per_source: Option<usize>,
    /// Where retained scores are written.
    #[serde(default)]
    pub store: StoreConfig,
    /// Optional Postgres connection for ingesting the provenance into the shared
    /// `score` catalog (`catalog_scores`). When absent, only the local corpus +
    /// manifest are written. Overridable via `CYMBRA_SCORE_DATABASE_URL`.
    #[serde(default)]
    pub catalog_database_url: Option<String>,
}

/// The object-store target + confidence prefixes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreConfig {
    #[serde(default)]
    pub backend: StoreBackend,
    #[serde(default = "default_safe_prefix")]
    pub safe_prefix: String,
    #[serde(default = "default_low_prefix")]
    pub low_confidence_prefix: String,
}

/// Object-store backend: a local folder in dev, S3/MinIO in prod.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StoreBackend {
    LocalFs {
        #[serde(default = "default_root")]
        root: PathBuf,
    },
    S3 {
        bucket: String,
        #[serde(default)]
        endpoint: Option<String>,
        #[serde(default)]
        region: Option<String>,
    },
}

impl Config {
    /// Loads and parses a `config.yaml`, then applies env overrides.
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading config {}", path.display()))?;
        let mut cfg = Self::from_yaml(&text)?;
        cfg.apply_env_overrides(&EnvSource::process());
        Ok(cfg)
    }

    /// Parses config from a YAML string (defaults fill any omitted field).
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text).context("parsing config yaml")
    }

    /// The descriptive User-Agent, including the contact so hosts can reach us.
    pub fn user_agent(&self) -> String {
        format!(
            "cymbra-score-crawler/{} (+{})",
            env!("CARGO_PKG_VERSION"),
            self.contact
        )
    }

    /// Applies overrides from the process environment. Call this even when no
    /// `config.yaml` exists so `CYMBRA_SCORE_*` still take effect.
    pub fn apply_process_env(&mut self) {
        self.apply_env_overrides(&EnvSource::process());
    }

    /// Overrides the store backend to S3 when `CYMBRA_SCORE_S3_BUCKET` is set,
    /// so dev (local folder) and prod (S3/MinIO) differ by environment.
    pub fn apply_env_overrides(&mut self, env: &EnvSource) {
        if let Some(bucket) = env.bucket.clone() {
            self.store.backend = StoreBackend::S3 {
                bucket,
                endpoint: env.endpoint.clone(),
                region: env.region.clone(),
            };
        }
        if env.catalog_url.is_some() {
            self.catalog_database_url = env.catalog_url.clone();
        }
    }
}

/// The `CYMBRA_SCORE_S3_*` values, read from the environment or supplied in
/// tests (so override logic is testable without mutating global env).
#[derive(Debug, Default, Clone)]
pub struct EnvSource {
    pub bucket: Option<String>,
    pub endpoint: Option<String>,
    pub region: Option<String>,
    pub catalog_url: Option<String>,
}

impl EnvSource {
    fn process() -> Self {
        Self {
            bucket: std::env::var("CYMBRA_SCORE_S3_BUCKET").ok(),
            endpoint: std::env::var("CYMBRA_SCORE_S3_ENDPOINT").ok(),
            region: std::env::var("CYMBRA_SCORE_S3_REGION").ok(),
            catalog_url: std::env::var("CYMBRA_SCORE_DATABASE_URL").ok(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            delay_ms: default_delay_ms(),
            concurrency: default_concurrency(),
            contact: default_contact(),
            limit_per_source: None,
            store: StoreConfig::default(),
            catalog_database_url: None,
        }
    }
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            backend: StoreBackend::default(),
            safe_prefix: default_safe_prefix(),
            low_confidence_prefix: default_low_prefix(),
        }
    }
}

impl Default for StoreBackend {
    fn default() -> Self {
        StoreBackend::LocalFs {
            root: default_root(),
        }
    }
}

fn default_delay_ms() -> u64 {
    2000
}
fn default_concurrency() -> usize {
    2
}
fn default_contact() -> String {
    "contact-not-set@example.org".to_string()
}
fn default_root() -> PathBuf {
    PathBuf::from("./output")
}
fn default_safe_prefix() -> String {
    "safe".to_string()
}
fn default_low_prefix() -> String {
    "low_confidence".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_yaml_uses_defaults() {
        let cfg = Config::from_yaml("{}").unwrap();
        assert_eq!(cfg.delay_ms, 2000);
        assert_eq!(cfg.concurrency, 2);
        assert_eq!(cfg.store.safe_prefix, "safe");
        assert!(matches!(cfg.store.backend, StoreBackend::LocalFs { .. }));
    }

    #[test]
    fn parses_a_populated_config() {
        let yaml = r#"
sources: [openscore, mutopia]
delay_ms: 3000
concurrency: 1
contact: "ops@cymbra.example"
limit_per_source: 50
store:
  backend:
    kind: local_fs
    root: /tmp/corpus
  safe_prefix: safe
  low_confidence_prefix: review
"#;
        let cfg = Config::from_yaml(yaml).unwrap();
        assert_eq!(cfg.sources, vec!["openscore", "mutopia"]);
        assert_eq!(cfg.delay_ms, 3000);
        assert_eq!(cfg.limit_per_source, Some(50));
        assert_eq!(cfg.store.low_confidence_prefix, "review");
        assert!(cfg.user_agent().contains("ops@cymbra.example"));
    }

    #[test]
    fn env_overrides_switch_to_s3() {
        let mut cfg = Config::default();
        let env = EnvSource {
            bucket: Some("cymbra-scores".into()),
            endpoint: Some("https://minio.local".into()),
            region: None,
            catalog_url: None,
        };
        cfg.apply_env_overrides(&env);
        match cfg.store.backend {
            StoreBackend::S3 {
                bucket, endpoint, ..
            } => {
                assert_eq!(bucket, "cymbra-scores");
                assert_eq!(endpoint.as_deref(), Some("https://minio.local"));
            }
            _ => panic!("expected S3 backend"),
        }
    }
}
