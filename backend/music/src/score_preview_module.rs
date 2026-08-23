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
    ScorePreviewConfig, ScorePreviewConfigSource, render_score_preview_wav,
    score_preview_object_key,
};
use crate::soundfont::SoundFontRepo;

/// What one render attempt did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderOutcome {
    /// The clip was rendered, stored, and the row marked.
    Rendered { bytes: usize },
    /// The row's family has no usable configured font — unset, unknown, not
    /// accepted, or (for a percussion row) not percussion-family: that family's
    /// teasers are dormant. Nothing stored, nothing marked.
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
    ///
    /// The font is chosen by the **row's instrument** (change:
    /// add-drum-audio-channel): a keyboard (or `unknown`) row reads the
    /// existing keyboard key unchanged; a percussion row reads the kit key and
    /// renders on the drum channel. A percussion row whose kit font is unset,
    /// unknown, unaccepted or not percussion-family is `Dormant` — nothing
    /// stored, the row left unmarked, so the standard backfill covers it once a
    /// kit is configured — and a percussion piece is never rendered with a
    /// keyboard-family font.
    pub async fn render_and_store(&self, catalog_id: &str) -> Result<RenderOutcome> {
        let cfg = self.config.score_preview_config();
        let obj = self
            .catalog
            .object_ref(catalog_id, true)
            .await?
            .ok_or_else(|| AppError::NotFound("catalog score not found".into()))?;
        let percussion = obj.instrument == crate::repo::Instrument::Percussion;
        let (font_id, channel) = preview_font_selection(percussion, &cfg);
        if font_id.is_empty() {
            return Ok(RenderOutcome::Dormant(if percussion {
                "catalog.preview.drum_soundfont_id is unset (percussion previews dormant)".into()
            } else {
                "catalog.preview.soundfont_id is unset".into()
            }));
        }
        let Some(font) = self.fonts.lookup(&font_id).await? else {
            return Ok(RenderOutcome::Dormant(format!(
                "preview font {font_id} not found"
            )));
        };
        if !font.is_accepted() {
            return Ok(RenderOutcome::Dormant(format!(
                "preview font {font_id} is not accepted"
            )));
        }
        // Never a keyboard-font clip of a drum part: a mis-configured kit key is
        // dormant, exactly like an unset one.
        if percussion && font.instrument != crate::soundfont::PERCUSSION_FAMILY {
            return Ok(RenderOutcome::Dormant(format!(
                "preview font {font_id} is not percussion-family"
            )));
        }
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
            render_score_preview_wav(&font_bytes, &score_bytes, max_ms, channel)
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("render task: {e}")))?
        .map_err(|e| AppError::Internal(anyhow::anyhow!("render preview: {e}")))?;
        let Some(wav) = wav else {
            return Ok(RenderOutcome::Silent);
        };
        let bytes = wav.len();
        // A NEW key per render (the store's warm cache treats a key as immutable):
        // store, stamp the same instant on the row, then evict the previous clip.
        let rendered_at = chrono::Utc::now();
        self.score_store
            .put(&score_preview_object_key(catalog_id, rendered_at), wav)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("store preview: {e}")))?;
        self.catalog
            .set_preview_rendered(catalog_id, Some(rendered_at))
            .await?;
        if let Some(previous) = obj.preview_rendered_at {
            // Best-effort: a leftover object costs storage, never correctness.
            let _ = self
                .score_store
                .delete(&score_preview_object_key(catalog_id, previous))
                .await;
        }
        Ok(RenderOutcome::Rendered { bytes })
    }
}

/// The per-family font key + synthesis channel a catalog row's teaser uses
/// (change: add-drum-audio-channel) — pure, so the routing is host-testable:
/// a percussion row reads the kit key and renders on the shared drum channel;
/// every other row reads the pre-existing keyboard key on the melodic channel,
/// byte-identically to before the kit key existed.
pub fn preview_font_selection(percussion: bool, cfg: &ScorePreviewConfig) -> (String, i32) {
    if percussion {
        (
            cfg.drum_soundfont_id.trim().to_string(),
            cymbra_musicxml_core::DRUM_CHANNEL,
        )
    } else {
        (
            cfg.soundfont_id.trim().to_string(),
            cymbra_musicxml_core::MELODIC_CHANNEL,
        )
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
        catalog
            .set_preview_rendered(A, Some(chrono::Utc::now()))
            .await
            .unwrap();
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
                ..Default::default()
            })),
        );
        assert!(matches!(
            r.render_and_store(A).await.unwrap(),
            RenderOutcome::Dormant(_)
        ));
    }

    // --- Percussion previews (change: add-drum-audio-channel) -------------

    /// A percussion piece id, seeded accepted and unmarked.
    const D: &str = "55555555-5555-7555-8555-555555555555";

    /// A catalog holding one accepted keyboard piece and one accepted
    /// percussion piece, both unmarked.
    fn mixed_catalog() -> Arc<FakeCatalogSearchRepo> {
        Arc::new(FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new(A, "A", "X", None).piano(),
            FakeCatalogRow::new(D, "D", "X", None).percussion(),
        ]))
    }

    /// A catalog font entry with the given family and moderation status.
    fn font(id: &str, family: &str, status: &str) -> crate::soundfont::FontEntry {
        crate::soundfont::FontEntry {
            id: id.into(),
            label: id.into(),
            object_key: format!("{id}.sf2"),
            instrument: family.into(),
            license: "CC0-1.0".into(),
            attribution: None,
            size_bytes: None,
            moderation_status: status.into(),
            reviewed_by: None,
            reviewed_at: None,
            uploaded_by: None,
            content_sha256: None,
            point_cost: 0,
            redeemable: true,
            review_reason: None,
            resubmission_note: None,
        }
    }

    fn kit_config(drum_soundfont_id: &str) -> Arc<FixedScorePreviewConfig> {
        Arc::new(FixedScorePreviewConfig(ScorePreviewConfig {
            max_ms: 1000,
            // The keyboard key is set throughout: the kit key alone governs
            // percussion rows.
            soundfont_id: "grand".into(),
            drum_soundfont_id: drum_soundfont_id.into(),
        }))
    }

    /// A percussion row is `Dormant` — nothing stored, row unmarked — for every
    /// unusable kit-font state: unset, unknown, unaccepted, wrong family.
    #[tokio::test]
    async fn percussion_is_dormant_until_a_kit_font_is_usable() {
        let cases: Vec<(Arc<FakeSoundFontRepo>, Arc<FixedScorePreviewConfig>)> = vec![
            // Unset kit key (the keyboard key being set must not leak over).
            (Arc::new(FakeSoundFontRepo::default()), kit_config("")),
            // Configured but unknown kit id.
            (Arc::new(FakeSoundFontRepo::default()), kit_config("kit")),
            // Known but not accepted.
            (
                Arc::new(FakeSoundFontRepo::with(vec![font(
                    "kit",
                    "percussion",
                    "pending",
                )])),
                kit_config("kit"),
            ),
            // Accepted but not percussion-family: never a keyboard-font clip of
            // a drum part.
            (
                Arc::new(FakeSoundFontRepo::with(vec![font(
                    "kit", "keyboard", "accepted",
                )])),
                kit_config("kit"),
            ),
        ];
        for (fonts, config) in cases {
            let catalog = mixed_catalog();
            let r = ScorePreviewRenderer::new(
                Arc::new(FakeStore::default()),
                Arc::new(FakeStore::default()),
                catalog.clone(),
                fonts,
                config,
            );
            assert!(matches!(
                r.render_and_store(D).await.unwrap(),
                RenderOutcome::Dormant(_)
            ));
            // The row stays unmarked, so the standard backfill picks it up once
            // a kit font is configured.
            let missing = catalog.accepted_ids_missing_preview(100).await.unwrap();
            assert!(missing.contains(&D.to_string()));
        }
    }

    /// A keyboard row's plan is byte-identical to before the kit key existed:
    /// the same `catalog.preview.soundfont_id` font, on the melodic channel —
    /// which is pinned to 0, the value the retired hardcoded `PREVIEW_CHANNEL`
    /// held, so the render call chain is unchanged for keyboards.
    #[test]
    fn keyboard_selection_is_byte_identical_to_before() {
        let cfg = ScorePreviewConfig {
            max_ms: 1000,
            soundfont_id: "grand".into(),
            drum_soundfont_id: "kit".into(),
        };
        assert_eq!(
            preview_font_selection(false, &cfg),
            ("grand".to_string(), cymbra_musicxml_core::MELODIC_CHANNEL)
        );
        assert_eq!(cymbra_musicxml_core::MELODIC_CHANNEL, 0);
    }

    #[test]
    fn percussion_selection_targets_the_kit_on_the_drum_channel() {
        let cfg = ScorePreviewConfig {
            max_ms: 1000,
            soundfont_id: "grand".into(),
            drum_soundfont_id: "kit".into(),
        };
        assert_eq!(
            preview_font_selection(true, &cfg),
            ("kit".to_string(), cymbra_musicxml_core::DRUM_CHANNEL)
        );
        assert_eq!(cymbra_musicxml_core::DRUM_CHANNEL, 9);
    }

    /// Task 5.4: with the `add-drums-access` skips lifted, the standard backfill
    /// over unmarked accepted pieces enqueues percussion rows like any other —
    /// no new mechanism.
    #[tokio::test]
    async fn backfill_includes_unmarked_percussion_rows() {
        let catalog = mixed_catalog();
        let q = FakeEnqueuer::default();
        let ids = enqueue_missing_previews(catalog.as_ref(), &q, 100)
            .await
            .unwrap();
        assert!(ids.contains(&A.to_string()));
        assert!(ids.contains(&D.to_string()));
    }
}
