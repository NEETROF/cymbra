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
//! Declares enabled sources, per-source quotas for test runs, and the
//! object-store target
//! (local filesystem folder in dev ⇄ S3/MinIO in prod, matching the shared
//! `CYMBRA_SCORE_S3_*` config the score module introduces). Env vars override
//! the store backend so dev/prod differ by environment, not by editing the file.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::convert::{ConverterBackend, Converters};

/// Top-level crawler configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Enabled source names (empty = none until selected on the CLI).
    #[serde(default)]
    pub sources: Vec<String>,
    /// Per-source item cap for test runs (`None` = unbounded).
    #[serde(default)]
    pub limit_per_source: Option<usize>,
    /// Where retained scores are written.
    #[serde(default)]
    pub store: StoreConfig,
    /// Where the crawler keeps files that are ITS OWN, not the corpus': source
    /// checkouts and the per-run artefacts (manifest export, rejection log).
    ///
    /// Resolved **independently** of the store root and never derived from it.
    /// It used to be `<store root>/.checkouts`, which was harmless while the root
    /// was a staging directory but put 4.4 GB of git clones inside the served
    /// corpus once the root became `SCORES_DIR` (change:
    /// fix-crawler-corpus-isolation). The default is a sibling of the store
    /// default, and [`Config::resolved_work_dir`] refuses a value nested in the
    /// corpus root, so a wrong mount fails loudly instead of polluting silently.
    /// Overridable via `CYMBRA_SCORE_WORK_DIR`.
    #[serde(default = "default_work_dir")]
    pub work_dir: PathBuf,
    /// Optional Postgres connection for ingesting the provenance into the shared
    /// `score` catalog (`catalog_scores`). When absent, only the local corpus +
    /// manifest are written. Overridable via `CYMBRA_SCORE_DATABASE_URL`.
    #[serde(default)]
    pub catalog_database_url: Option<String>,
    /// How the external converters (MuseScore/Verovio/python-ly) are invoked.
    #[serde(default)]
    pub converters: ConverterSettings,
}

/// External-converter settings: run the binaries locally or inside Docker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConverterSettings {
    #[serde(default)]
    pub backend: ConverterBackendCfg,
    #[serde(default = "default_musescore_image")]
    pub musescore_image: String,
    #[serde(default = "default_verovio_image")]
    pub verovio_image: String,
    #[serde(default = "default_lilypond_image")]
    pub lilypond_image: String,
}

/// Serde-facing converter backend (`local` | `docker`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConverterBackendCfg {
    #[default]
    Local,
    Docker,
}

impl Default for ConverterSettings {
    fn default() -> Self {
        Self {
            backend: ConverterBackendCfg::Local,
            musescore_image: default_musescore_image(),
            verovio_image: default_verovio_image(),
            lilypond_image: default_lilypond_image(),
        }
    }
}

impl ConverterSettings {
    /// Maps to the runtime [`Converters`] used by the conversion pipeline.
    pub fn to_converters(&self) -> Converters {
        Converters {
            backend: match self.backend {
                ConverterBackendCfg::Local => ConverterBackend::Local,
                ConverterBackendCfg::Docker => ConverterBackend::Docker,
            },
            musescore_image: self.musescore_image.clone(),
            verovio_image: self.verovio_image.clone(),
            lilypond_image: self.lilypond_image.clone(),
        }
    }
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

impl Default for Config {
    // Hand-written (not derived) because `work_dir` must default to a real path:
    // a derived `PathBuf::default()` would be empty, i.e. the process cwd, which
    // is exactly the kind of silent-and-wrong default this change exists to remove.
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            limit_per_source: None,
            store: StoreConfig::default(),
            work_dir: default_work_dir(),
            catalog_database_url: None,
            converters: ConverterSettings::default(),
        }
    }
}

impl Config {
    /// The validated work location: where source checkouts and run artefacts go.
    ///
    /// Fails when it resolves inside the corpus root — the failure this change
    /// was written for. Comparison is lexical (both sides normalised, `..`
    /// collapsed) because neither directory necessarily exists yet, so it cannot
    /// rely on `canonicalize`.
    pub fn resolved_work_dir(&self) -> Result<PathBuf> {
        let work = normalize(&self.work_dir);
        match &self.store.backend {
            StoreBackend::LocalFs { root } => {
                let root = normalize(root);
                anyhow::ensure!(
                    !work.starts_with(&root),
                    "work dir {} is inside the corpus root {}: the served corpus must hold \
                     only servable objects — point CYMBRA_SCORE_WORK_DIR outside it",
                    work.display(),
                    root.display(),
                );
            }
            // An S3 corpus has no local root to be nested in.
            StoreBackend::S3 { .. } => {}
        }
        Ok(work)
    }

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
        if let Some(dir) = env.work_dir.clone() {
            self.work_dir = PathBuf::from(dir);
        }
        // Converter backend + images: let the deployment pick `docker` (and the
        // MuseScore/Verovio/python-ly images) without editing config.yaml — e.g.
        // the compose openscore service that converts `.mscx` via a MuseScore
        // container.
        match env.converter_backend.as_deref() {
            Some("docker") => self.converters.backend = ConverterBackendCfg::Docker,
            Some("local") => self.converters.backend = ConverterBackendCfg::Local,
            _ => {}
        }
        if let Some(img) = env.musescore_image.clone() {
            self.converters.musescore_image = img;
        }
        if let Some(img) = env.verovio_image.clone() {
            self.converters.verovio_image = img;
        }
        if let Some(img) = env.lilypond_image.clone() {
            self.converters.lilypond_image = img;
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
    pub work_dir: Option<String>,
    pub converter_backend: Option<String>,
    pub musescore_image: Option<String>,
    pub verovio_image: Option<String>,
    pub lilypond_image: Option<String>,
}

impl EnvSource {
    fn process() -> Self {
        let non_empty = |k: &str| std::env::var(k).ok().filter(|s| !s.is_empty());
        Self {
            bucket: std::env::var("CYMBRA_SCORE_S3_BUCKET").ok(),
            endpoint: std::env::var("CYMBRA_SCORE_S3_ENDPOINT").ok(),
            region: std::env::var("CYMBRA_SCORE_S3_REGION").ok(),
            // An empty value means "no catalog DB" (local corpus only).
            catalog_url: non_empty("CYMBRA_SCORE_DATABASE_URL"),
            work_dir: non_empty("CYMBRA_SCORE_WORK_DIR"),
            converter_backend: non_empty("CYMBRA_SCORE_CONVERTER_BACKEND"),
            musescore_image: non_empty("CYMBRA_SCORE_MUSESCORE_IMAGE"),
            verovio_image: non_empty("CYMBRA_SCORE_VEROVIO_IMAGE"),
            lilypond_image: non_empty("CYMBRA_SCORE_LILYPOND_IMAGE"),
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

/// Absolute, lexically-cleaned form of `p` (`.` dropped, `..` collapsed), used to
/// compare two directories that may not exist yet. Purely lexical: it does not
/// touch the filesystem and does not resolve symlinks.
fn normalize(p: &std::path::Path) -> PathBuf {
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(p)
    };
    let mut out = PathBuf::new();
    for c in abs.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

fn default_root() -> PathBuf {
    PathBuf::from("./output")
}
/// Sibling of [`default_root`], never a child of it — the whole point of the
/// setting is that the work location cannot silently follow the corpus root.
fn default_work_dir() -> PathBuf {
    PathBuf::from("./work")
}
fn default_safe_prefix() -> String {
    "safe".to_string()
}
fn default_low_prefix() -> String {
    "low_confidence".to_string()
}
// Docker image placeholders — override with real images carrying the CLI tools.
fn default_musescore_image() -> String {
    "cymbra/musescore".to_string()
}
fn default_verovio_image() -> String {
    "cymbra/verovio".to_string()
}
fn default_lilypond_image() -> String {
    "cymbra/python-ly".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_yaml_uses_defaults() {
        let cfg = Config::from_yaml("{}").unwrap();
        assert_eq!(cfg.store.safe_prefix, "safe");
        assert!(matches!(cfg.store.backend, StoreBackend::LocalFs { .. }));
    }

    #[test]
    fn parses_a_populated_config() {
        let yaml = r#"
sources: [openscore, mutopia]
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
        assert_eq!(cfg.limit_per_source, Some(50));
        assert_eq!(cfg.store.low_confidence_prefix, "review");
    }

    #[test]
    fn parses_docker_converter_settings() {
        let yaml = r#"
converters:
  backend: docker
  musescore_image: myorg/musescore:4
  lilypond_image: myorg/python-ly:latest
"#;
        let cfg = Config::from_yaml(yaml).unwrap();
        assert_eq!(cfg.converters.backend, ConverterBackendCfg::Docker);
        let c = cfg.converters.to_converters();
        assert_eq!(c.backend, ConverterBackend::Docker);
        assert_eq!(c.musescore_image, "myorg/musescore:4");
        // Unspecified images fall back to defaults.
        assert_eq!(c.verovio_image, "cymbra/verovio");
    }

    #[test]
    fn defaults_to_local_converters() {
        let cfg = Config::from_yaml("{}").unwrap();
        assert_eq!(cfg.converters.backend, ConverterBackendCfg::Local);
    }

    #[test]
    fn env_overrides_switch_to_s3() {
        let mut cfg = Config::default();
        let env = EnvSource {
            bucket: Some("cymbra-musics".into()),
            endpoint: Some("https://minio.local".into()),
            ..Default::default()
        };
        cfg.apply_env_overrides(&env);
        match cfg.store.backend {
            StoreBackend::S3 {
                bucket, endpoint, ..
            } => {
                assert_eq!(bucket, "cymbra-musics");
                assert_eq!(endpoint.as_deref(), Some("https://minio.local"));
            }
            _ => panic!("expected S3 backend"),
        }
    }

    // --- work location (change: fix-crawler-corpus-isolation) ---

    #[test]
    fn default_work_dir_is_not_inside_the_corpus_root() {
        let cfg = Config::default();
        let work = cfg.resolved_work_dir().expect("default must be accepted");
        let root = match &cfg.store.backend {
            StoreBackend::LocalFs { root } => normalize(root),
            _ => panic!("expected the local_fs default"),
        };
        assert!(
            !work.starts_with(&root),
            "default work dir {} must not sit inside the corpus root {}",
            work.display(),
            root.display()
        );
    }

    #[test]
    fn work_dir_nested_in_the_corpus_root_is_refused() {
        let mut cfg = Config::default();
        cfg.store.backend = StoreBackend::LocalFs {
            root: PathBuf::from("/var/lib/cymbra/scores"),
        };
        // Exactly the production failure: checkouts under the served corpus.
        cfg.work_dir = PathBuf::from("/var/lib/cymbra/scores/.checkouts");
        let err = cfg.resolved_work_dir().expect_err("must be refused");
        assert!(
            err.to_string().contains("inside the corpus root"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn work_dir_equal_to_the_corpus_root_is_refused() {
        let mut cfg = Config::default();
        cfg.store.backend = StoreBackend::LocalFs {
            root: PathBuf::from("/var/lib/cymbra/scores"),
        };
        cfg.work_dir = PathBuf::from("/var/lib/cymbra/scores");
        assert!(cfg.resolved_work_dir().is_err());
    }

    #[test]
    fn work_dir_reaching_out_with_dotdot_is_accepted() {
        let mut cfg = Config::default();
        cfg.store.backend = StoreBackend::LocalFs {
            root: PathBuf::from("/var/lib/cymbra/scores"),
        };
        // Lexically inside until `..` is collapsed — normalisation must see it out.
        cfg.work_dir = PathBuf::from("/var/lib/cymbra/scores/../crawler-work");
        let work = cfg.resolved_work_dir().expect("must be accepted");
        assert_eq!(work, PathBuf::from("/var/lib/cymbra/crawler-work"));
    }

    #[test]
    fn work_dir_is_independent_of_the_corpus_root() {
        let mut cfg = Config::default();
        cfg.store.backend = StoreBackend::LocalFs {
            root: PathBuf::from("/srv/corpus"),
        };
        cfg.work_dir = PathBuf::from("/srv/crawler-work");
        assert_eq!(
            cfg.resolved_work_dir().unwrap(),
            PathBuf::from("/srv/crawler-work")
        );
    }

    #[test]
    fn s3_corpus_has_no_local_root_to_nest_in() {
        let mut cfg = Config::default();
        cfg.store.backend = StoreBackend::S3 {
            bucket: "cymbra-scores".into(),
            endpoint: None,
            region: None,
        };
        cfg.work_dir = PathBuf::from("/anywhere");
        assert!(cfg.resolved_work_dir().is_ok());
    }

    #[test]
    fn env_override_sets_the_work_dir() {
        let mut cfg = Config::default();
        let env = EnvSource {
            work_dir: Some("/srv/crawler-work".into()),
            ..Default::default()
        };
        cfg.apply_env_overrides(&env);
        assert_eq!(cfg.work_dir, PathBuf::from("/srv/crawler-work"));
    }

    #[test]
    fn work_dir_parses_from_yaml() {
        let cfg = Config::from_yaml("work_dir: /srv/crawler-work").unwrap();
        assert_eq!(cfg.work_dir, PathBuf::from("/srv/crawler-work"));
    }

    #[test]
    fn env_overrides_select_docker_converter_and_image() {
        let mut cfg = Config::default();
        assert_eq!(cfg.converters.backend, ConverterBackendCfg::Local);
        let env = EnvSource {
            converter_backend: Some("docker".into()),
            musescore_image: Some("cymbra/musescore".into()),
            ..Default::default()
        };
        cfg.apply_env_overrides(&env);
        assert_eq!(cfg.converters.backend, ConverterBackendCfg::Docker);
        assert_eq!(cfg.converters.musescore_image, "cymbra/musescore");
        // Unset image keeps its default.
        assert_eq!(cfg.converters.verovio_image, default_verovio_image());
    }
}
