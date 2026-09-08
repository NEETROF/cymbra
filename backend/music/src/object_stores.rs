//! The object stores the music maintenance binaries read.
//!
//! They must reach exactly the keyspace the server reads, so each store is built here
//! once rather than copied into each `bin`. This lived in the composition root until
//! group 5 of harden-module-boundaries; both stores hold music objects, so it belongs
//! to the module that owns them.

use std::sync::Arc;

use anyhow::Context;
use cymbra_platform::config::Config;
use cymbra_storage::{LocalFirstStore, ObjectStorage, S3Params};

/// The S3 connection fields, copied from whichever config struct holds them.
///
/// `ScoreStorageConfig` and `SoundfontStorageConfig` declare the same six fields but
/// are distinct types, so this is a field-by-field copy either way; writing it once as
/// a macro is what stops the two builders below from being near-identical.
macro_rules! s3_params {
    ($cfg:expr) => {
        S3Params {
            bucket: $cfg.bucket.clone(),
            endpoint: $cfg.endpoint.clone(),
            region: $cfg.region.clone(),
            access_key: $cfg.access_key.clone(),
            secret_key: $cfg.secret_key.clone(),
            allow_http: $cfg.allow_http,
        }
    };
}

/// Build a local-first store: warm reads from `local_root`, the S3 bucket as the
/// durable origin. `what` names the store in the error, so a failure still says which.
fn local_first(
    what: &str,
    local_root: &str,
    s3: S3Params,
) -> anyhow::Result<Arc<dyn ObjectStorage>> {
    Ok(Arc::new(
        LocalFirstStore::from_config(local_root, &s3)
            .with_context(|| format!("building the {what} object store"))?,
    ))
}

/// The score object store the server itself uses. The local root lives on `Config`
/// here, and on the config struct itself for SoundFonts — the one real difference.
pub fn score_object_store(cfg: &Config) -> anyhow::Result<Arc<dyn ObjectStorage>> {
    let s3 = cfg
        .score_storage
        .as_ref()
        .context("CYMBRA_SCORE_S3_BUCKET (+ credentials) is required to reach the score corpus")?;
    local_first("score", &cfg.score_local_root, s3_params!(s3))
}

/// The PRIVATE SoundFont object store the server itself uses (same keyspace as the
/// delivery/upload routes), for the `verify-soundfont-families` one-shot ops pass
/// (change: add-drum-audio-channel).
pub fn soundfont_object_store(cfg: &Config) -> anyhow::Result<Arc<dyn ObjectStorage>> {
    let sf = cfg.soundfont_storage.as_ref().context(
        "CYMBRA_SOUNDFONT_S3_BUCKET (+ credentials) is required to reach the SoundFont store",
    )?;
    local_first("SoundFont", &sf.local_root, s3_params!(sf))
}
