//! The module's HTTP surface, as one router (change: harden-module-boundaries,
//! group 5).
//!
//! The composition root used to build two states, call two `*_router` functions and
//! merge both, passing the CORS allow-list twice. That made the split between the
//! delivery routes and the preview routes part of `main.rs`'s vocabulary — a detail
//! of this module that the root had no reason to know. It mounts one router now, and
//! a route added here needs no change there.

use axum::Router;

use crate::score_preview_http::{ScorePreviewState, score_preview_router};
use crate::soundfont_http::{SoundfontState, soundfont_router};

/// Every music HTTP route: SoundFont delivery and upload, and the score preview
/// teasers. `allowed_origins` is the credentialed CORS allow-list, applied
/// identically to both — passing it once is what keeps them identical.
pub fn router(
    soundfont: SoundfontState,
    score_preview: ScorePreviewState,
    allowed_origins: Vec<String>,
) -> Router {
    soundfont_router(soundfont, allowed_origins.clone())
        .merge(score_preview_router(score_preview, allowed_origins))
}
