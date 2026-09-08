//! The object stores the music maintenance binaries read.
//!
//! They must reach exactly the keyspace the server reads, so each store is built
//! here once rather than copied into each `bin`. This lived in the composition root
//! until group 5 of harden-module-boundaries; both stores hold music objects, so it
//! belongs to the module that owns them.

use std::sync::Arc;

use cymbra_platform::config::Config;
use cymbra_storage::{LocalFirstStore, ObjectStorage, S3Params};

/// The score object store the server itself uses: local-first reads with the
/// S3 bucket as the durable origin.
pub fn score_object_store(cfg: &Config) -> anyhow::Result<Arc<dyn ObjectStorage>> {
    use anyhow::Context;
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

/// The PRIVATE SoundFont object store the server itself uses (same keyspace
/// as the delivery/upload routes), for the `verify-soundfont-families`
/// one-shot ops pass (change: add-drum-audio-channel).
pub fn soundfont_object_store(cfg: &Config) -> anyhow::Result<Arc<dyn ObjectStorage>> {
    use anyhow::Context;
    let sf = cfg.soundfont_storage.as_ref().context(
        "CYMBRA_SOUNDFONT_S3_BUCKET (+ credentials) is required to reach the SoundFont store",
    )?;
    Ok(Arc::new(
        LocalFirstStore::from_config(
            &sf.local_root,
            &S3Params {
                bucket: sf.bucket.clone(),
                endpoint: sf.endpoint.clone(),
                region: sf.region.clone(),
                access_key: sf.access_key.clone(),
                secret_key: sf.secret_key.clone(),
                allow_http: sf.allow_http,
            },
        )
        .context("building the SoundFont object store")?,
    ))
}
