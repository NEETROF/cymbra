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

//! Score audio-teaser renderer (change: add-score-daily-access-rewards, design
//! D7): the orchestration the worker job, the back-office regenerate route and
//! the backfill share — resolve the piece and the configured font, render the
//! bounded clip, store it beside the score bytes, stamp the row's marker.
//!
//! I/O glue over the pure [`crate::score_preview`] helpers; excluded from the
//! coverage gate like the SoundFont preview glue. The outcome enum makes the
//! "dormant" states (no font configured, a piece that sounds nothing) explicit
//! so callers log them rather than fail.

use std::sync::Arc;

use cymbra_jobs::{EnqueueRequest, Enqueuer};
use cymbra_platform::{AppError, Result};
use cymbra_storage::{ObjectStorage, StorageError};

use crate::catalog_search::CatalogSearchRepo;
use crate::score_preview::{
    ScorePreviewConfigSource, render_score_preview_wav, score_preview_object_key,
};
use crate::soundfont::SoundFontRepo;

/// What one render attempt did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderOutcome {
    /// The clip was rendered, stored, and the row marked.
    Rendered { bytes: usize },
    /// No font is configured (or it is unknown / not accepted): the teasers are
    /// dormant. Nothing stored, nothing marked.
    Dormant(String),
    /// The piece sounds nothing within the bound (an empty schedule): nothing
    /// stored; the row is left unmarked so a later re-render can retry.
    Silent,
}

/// The renderer over the score store (MusicXML in, WAV out), the SoundFont store
/// (font bytes), the catalog (piece + marker) and the font catalog (font row).
pub struct ScorePreviewRenderer {
    score_store: Arc<dyn ObjectStorage>,
    soundfont_store: Arc<dyn ObjectStorage>,
    catalog: Arc<dyn CatalogSearchRepo>,
    fonts: Arc<dyn SoundFontRepo>,
    config: Arc<dyn ScorePreviewConfigSource>,
}

impl ScorePreviewRenderer {
    pub fn new(
        score_store: Arc<dyn ObjectStorage>,
        soundfont_store: Arc<dyn ObjectStorage>,
        catalog: Arc<dyn CatalogSearchRepo>,
        fonts: Arc<dyn SoundFontRepo>,
        config: Arc<dyn ScorePreviewConfigSource>,
    ) -> Self {
        Self {
            score_store,
            soundfont_store,
            catalog,
            fonts,
            config,
        }
    }

    /// Render + store + mark the teaser of `catalog_id` (any moderation status —
    /// the caller decides who may ask). Not-found for an unknown piece.
    pub async fn render_and_store(&self, catalog_id: &str) -> Result<RenderOutcome> {
        let cfg = self.config.score_preview_config();
        if cfg.soundfont_id.trim().is_empty() {
            return Ok(RenderOutcome::Dormant(
                "catalog.preview.soundfont_id is unset".into(),
            ));
        }
        let Some(font) = self.fonts.lookup(&cfg.soundfont_id).await? else {
            return Ok(RenderOutcome::Dormant(format!(
                "preview font {} not found",
                cfg.soundfont_id
            )));
        };
        if !font.is_accepted() {
            return Ok(RenderOutcome::Dormant(format!(
                "preview font {} is not accepted",
                cfg.soundfont_id
            )));
        }
        let obj = self
            .catalog
            .object_ref(catalog_id, true)
            .await?
            .ok_or_else(|| AppError::NotFound("catalog score not found".into()))?;
        let font_bytes = self
            .soundfont_store
            .get(&font.object_key)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("read preview font: {e}")))?;
        let score_bytes = self
            .score_store
            .get(&obj.object_key)
            .await
            .map_err(|e| match e {
                StorageError::NotFound(_) => {
                    AppError::FailedPrecondition("catalog score bytes not available yet".into())
                }
                other => AppError::Internal(anyhow::anyhow!("read catalog score: {other}")),
            })?;
        let max_ms = cfg.max_ms;
        // The synth is CPU-bound and blocking (rustysynth); keep it off the async
        // runtime's threads.
        let wav = tokio::task::spawn_blocking(move || {
            render_score_preview_wav(&font_bytes, &score_bytes, max_ms)
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("render task: {e}")))?
        .map_err(|e| AppError::Internal(anyhow::anyhow!("render preview: {e}")))?;
        let Some(wav) = wav else {
            return Ok(RenderOutcome::Silent);
        };
        let bytes = wav.len();
        self.score_store
            .put(&score_preview_object_key(catalog_id), wav)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("store preview: {e}")))?;
        self.catalog.set_preview_rendered(catalog_id, true).await?;
        Ok(RenderOutcome::Rendered { bytes })
    }
}

/// Build the `score_preview_render` enqueue request for `catalog_id` (change:
/// add-score-daily-access-rewards).
pub fn preview_render_request(catalog_id: &str) -> Result<EnqueueRequest> {
    let spec = cymbra_jobs::registry::spec(cymbra_jobs::registry::SCORE_PREVIEW_RENDER)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("score_preview_render spec missing")))?;
    EnqueueRequest::for_job(
        &spec,
        &serde_json::json!({ "catalog_id": catalog_id }),
        None,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("build enqueue request: {e}")))
}

/// The backfill (design D7): enqueue one render job for every `accepted` piece
/// without a rendered marker, up to `limit`. Returns the ids enqueued. Idempotent
/// across re-runs: a rendered piece is marked and drops out of the work list; an
/// enqueued-but-not-yet-rendered piece would be enqueued again (the render is
/// idempotent too), so run it once and let the queue drain.
pub async fn enqueue_missing_previews(
    catalog: &dyn CatalogSearchRepo,
    enqueuer: &dyn Enqueuer,
    limit: i64,
) -> Result<Vec<String>> {
    let ids = catalog.accepted_ids_missing_preview(limit).await?;
    for id in &ids {
        enqueuer
            .enqueue(preview_render_request(id)?)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("enqueue preview render: {e}")))?;
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_search::{FakeCatalogRow, FakeCatalogSearchRepo};
    use crate::score_preview::{FixedScorePreviewConfig, ScorePreviewConfig};
    use crate::soundfont::FakeSoundFontRepo;
    use cymbra_jobs::FakeEnqueuer;
    use cymbra_storage::FakeStore;

    const A: &str = "11111111-1111-7111-8111-111111111111";
    const B: &str = "22222222-2222-7222-8222-222222222222";
    const P: &str = "44444444-4444-7444-8444-444444444444";

    fn catalog() -> Arc<FakeCatalogSearchRepo> {
        Arc::new(FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new(A, "A", "X", None),
            FakeCatalogRow::new(B, "B", "X", None).with_preview(true),
            FakeCatalogRow::new(P, "P", "X", None).with_moderation_status("pending"),
        ]))
    }

    #[tokio::test]
    async fn backfill_enqueues_only_accepted_pieces_without_a_marker() {
        let catalog = catalog();
        let q = FakeEnqueuer::default();
        let ids = enqueue_missing_previews(catalog.as_ref(), &q, 100)
            .await
            .unwrap();
        assert_eq!(ids, vec![A.to_string()]);
        let reqs = q.requests();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].name, cymbra_jobs::registry::SCORE_PREVIEW_RENDER);
        assert!(reqs[0].payload_json.contains(A));
        // Once rendered (marked), the piece leaves the work list.
        catalog.set_preview_rendered(A, true).await.unwrap();
        let ids = enqueue_missing_previews(catalog.as_ref(), &q, 100)
            .await
            .unwrap();
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn backfill_honours_the_limit() {
        let catalog = Arc::new(FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new(A, "A", "X", None),
            FakeCatalogRow::new(B, "B", "X", None),
        ]));
        let q = FakeEnqueuer::default();
        let ids = enqueue_missing_previews(catalog.as_ref(), &q, 1)
            .await
            .unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn renderer_is_dormant_without_a_configured_font() {
        let r = ScorePreviewRenderer::new(
            Arc::new(FakeStore::default()),
            Arc::new(FakeStore::default()),
            catalog(),
            Arc::new(FakeSoundFontRepo::default()),
            Arc::new(FixedScorePreviewConfig(ScorePreviewConfig::default())),
        );
        assert!(matches!(
            r.render_and_store(A).await.unwrap(),
            RenderOutcome::Dormant(_)
        ));
        // A configured but unknown font is dormant too (never a failure).
        let r = ScorePreviewRenderer::new(
            Arc::new(FakeStore::default()),
            Arc::new(FakeStore::default()),
            catalog(),
            Arc::new(FakeSoundFontRepo::default()),
            Arc::new(FixedScorePreviewConfig(ScorePreviewConfig {
                max_ms: 1000,
                soundfont_id: "nope".into(),
            })),
        );
        assert!(matches!(
            r.render_and_store(A).await.unwrap(),
            RenderOutcome::Dormant(_)
        ));
    }
}
