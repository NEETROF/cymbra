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

//! One-shot facet backfill (change: score-catalog-facets).
//!
//! Re-reads each stored score object from the SAME object store the server uses,
//! derives its musical facets, and `UPDATE`s the row — no re-crawl. Idempotent
//! (only rows with unset facets are touched), so it is safe to re-run after a
//! crawl. Reuses the server's `Config`, so it points at the same DB + store:
//!
//!   cargo run -p cymbra-server --bin backfill_facets

use std::sync::Arc;

use cymbra_platform::config::Config;
use cymbra_platform::db;
use cymbra_storage::{LocalFirstStore, ObjectStorage, S3Params};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::from_filename("backend/.env").or_else(|_| dotenvy::dotenv());
    let cfg = Config::from_env()?;

    let db_url = cfg
        .music_database_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CYMBRA_MUSIC_DATABASE_URL is required"))?;
    let pool = db::connect(db_url, 4).await?;

    let s3 = cfg.score_storage.as_ref().ok_or_else(|| {
        anyhow::anyhow!("CYMBRA_SCORE_S3_BUCKET (+ score storage) is required to read objects")
    })?;
    let storage: Arc<dyn ObjectStorage> = Arc::new(LocalFirstStore::from_config(
        &cfg.score_local_root,
        &S3Params {
            bucket: s3.bucket.clone(),
            endpoint: s3.endpoint.clone(),
            region: s3.region.clone(),
            access_key: s3.access_key.clone(),
            secret_key: s3.secret_key.clone(),
            allow_http: s3.allow_http,
        },
    )?);

    let stats = cymbra_music::backfill_all(&pool, storage.as_ref()).await?;
    println!(
        "facet backfill complete — updated: {}, skipped: {}",
        stats.updated, stats.skipped
    );
    Ok(())
}
