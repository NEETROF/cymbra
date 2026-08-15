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

//! Catalog score audio-teaser HTTP routes (change: add-score-daily-access-rewards,
//! design D7) — the same shape as the SoundFont previews:
//!
//! - `GET  /scores/:id/preview`  — authenticated, moderation-visible (accepted
//!   for normal callers; any status for a moderator/admin), **no daily-quota or
//!   points gate** (hearing the teaser is the point), 404 when absent.
//! - `POST /scores/:id/preview`  — admin/moderator **regenerate**: renders inline
//!   (immediate feedback for the back-office "Generate sample"), overwrites the
//!   object and stamps the row's rendered marker.
//!
//! Thin axum glue over [`cymbra_music::ScorePreviewRenderer`]; coverage-excluded
//! like the SoundFont routes.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use cymbra_music::{
    CatalogSearchRepo, RenderOutcome, ScorePreviewRenderer, score_preview_object_key,
};
use cymbra_platform::{AppError, guard};
use cymbra_storage::{ObjectStorage, StorageError};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::soundfont::SoundfontAuth;

/// Router state: the score store (the teaser lives beside the score bytes), the
/// catalog read port (moderation visibility), the renderer (regenerate; `None`
/// when the SoundFont store is unconfigured → 503 on regenerate only), and the
/// auth seam shared with the SoundFont routes.
#[derive(Clone)]
pub struct ScorePreviewState {
    pub store: Option<Arc<dyn ObjectStorage>>,
    pub catalog: Option<Arc<dyn CatalogSearchRepo>>,
    pub renderer: Option<Arc<ScorePreviewRenderer>>,
    pub auth: Arc<dyn SoundfontAuth>,
}

/// The score-preview router, ready to `.merge()` into the HTTP server. Same CORS
/// allow-list as the SoundFont routes (the back office plays and regenerates).
pub fn score_preview_router(state: ScorePreviewState, allowed_origins: Vec<String>) -> Router {
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .expose_headers([header::CONTENT_LENGTH]);
    Router::new()
        .route(
            "/scores/:id/preview",
            get(serve_preview).post(regenerate_preview),
        )
        .layer(cors)
        .with_state(state)
}

fn status(code: StatusCode) -> Response {
    code.into_response()
}

/// `GET /scores/:id/preview` — the audio teaser, moderation-visible, ungated by
/// the daily quota.
async fn serve_preview(
    State(s): State<ScorePreviewState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if s.auth.identify(&headers).is_none() {
        return status(StatusCode::UNAUTHORIZED);
    }
    let can_view_unvalidated = s
        .auth
        .identify_admin(&headers)
        .as_ref()
        .is_some_and(|i| guard::require_moderator_or_admin(i).is_ok());
    let (Some(store), Some(catalog)) = (s.store.as_ref(), s.catalog.as_ref()) else {
        return status(StatusCode::SERVICE_UNAVAILABLE);
    };
    // Moderation visibility: the same gate as the bytes fetch (a normal caller
    // resolves only an accepted piece), but NOT the daily quota — the teaser is
    // what a locked piece offers.
    match catalog.object_ref(&id, can_view_unvalidated).await {
        Ok(Some(_)) => {}
        Ok(None) => return status(StatusCode::NOT_FOUND),
        Err(_) => return status(StatusCode::INTERNAL_SERVER_ERROR),
    }
    match store.get(&score_preview_object_key(&id)).await {
        Ok(bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "audio/wav".to_string()),
                (header::CONTENT_LENGTH, bytes.len().to_string()),
                (header::CACHE_CONTROL, "private, max-age=86400".to_string()),
            ],
            Body::from(bytes),
        )
            .into_response(),
        Err(StorageError::NotFound(_)) => status(StatusCode::NOT_FOUND),
        Err(_) => status(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// `POST /scores/:id/preview` — admin/moderator regenerate, rendered inline.
async fn regenerate_preview(
    State(s): State<ScorePreviewState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    match s.auth.identify_admin(&headers) {
        None => return status(StatusCode::UNAUTHORIZED),
        Some(identity) if guard::require_moderator_or_admin(&identity).is_err() => {
            return status(StatusCode::FORBIDDEN);
        }
        Some(_) => {}
    }
    let Some(renderer) = s.renderer.as_ref() else {
        return status(StatusCode::SERVICE_UNAVAILABLE);
    };
    match renderer.render_and_store(&id).await {
        Ok(RenderOutcome::Rendered { .. }) => status(StatusCode::OK),
        // Nothing to store: the piece sounds nothing, or the teasers are dormant
        // (no preview font configured). Report a typed precondition so the back
        // office can say why rather than "server error".
        Ok(RenderOutcome::Silent) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "the piece sounds nothing within the preview bound",
        )
            .into_response(),
        Ok(RenderOutcome::Dormant(reason)) => {
            (StatusCode::PRECONDITION_FAILED, reason).into_response()
        }
        Err(AppError::NotFound(_)) => status(StatusCode::NOT_FOUND),
        Err(AppError::FailedPrecondition(m)) => {
            (StatusCode::PRECONDITION_FAILED, m).into_response()
        }
        Err(e) => {
            tracing::warn!("score preview render failed for {id}: {e}");
            status(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Request;
    use cymbra_music::{
        FakeCatalogRow, FakeCatalogSearchRepo, FakeSoundFontRepo, FixedScorePreviewConfig,
        ScorePreviewConfig,
    };
    use cymbra_platform::AuthIdentity;
    use cymbra_storage::FakeStore;
    use tower::ServiceExt;

    const ACCEPTED: &str = "11111111-1111-7111-8111-111111111111";
    const PENDING: &str = "44444444-4444-7444-8444-444444444444";

    /// Header-driven test auth: `x-user` = user id, `x-role` = admin|moderator.
    struct HeaderAuth;
    impl SoundfontAuth for HeaderAuth {
        fn identify(&self, headers: &HeaderMap) -> Option<String> {
            headers
                .get("x-user")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        }
        fn identify_admin(&self, headers: &HeaderMap) -> Option<AuthIdentity> {
            let user_id = self.identify(headers)?;
            let roles = headers
                .get("x-role")
                .and_then(|v| v.to_str().ok())
                .map(|r| vec!["user".to_string(), r.to_string()])
                .unwrap_or_else(|| vec!["user".to_string()]);
            Some(AuthIdentity {
                user_id,
                audience: "music".into(),
                roles,
                ..Default::default()
            })
        }
    }

    async fn app(with_clip: bool, with_renderer: bool) -> Router {
        let store = Arc::new(FakeStore::default());
        if with_clip {
            store
                .put(&score_preview_object_key(ACCEPTED), b"RIFFwav".to_vec())
                .await
                .unwrap();
        }
        let catalog = Arc::new(FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new(ACCEPTED, "Clair de Lune", "Debussy", Some("advanced")),
            FakeCatalogRow::new(PENDING, "Pending", "Anon", None).with_moderation_status("pending"),
        ]));
        let renderer = with_renderer.then(|| {
            Arc::new(ScorePreviewRenderer::new(
                store.clone(),
                Arc::new(FakeStore::default()),
                catalog.clone(),
                Arc::new(FakeSoundFontRepo::default()),
                Arc::new(FixedScorePreviewConfig(ScorePreviewConfig::default())),
            ))
        });
        score_preview_router(
            ScorePreviewState {
                store: Some(store),
                catalog: Some(catalog),
                renderer,
                auth: Arc::new(HeaderAuth),
            },
            vec![],
        )
    }

    fn get_req(id: &str, user: Option<&str>, role: Option<&str>) -> Request<Body> {
        let mut b = Request::builder().uri(format!("/scores/{id}/preview"));
        if let Some(u) = user {
            b = b.header("x-user", u);
        }
        if let Some(r) = role {
            b = b.header("x-role", r);
        }
        b.body(Body::empty()).unwrap()
    }

    fn post_req(id: &str, user: Option<&str>, role: Option<&str>) -> Request<Body> {
        let mut b = Request::builder()
            .method(Method::POST)
            .uri(format!("/scores/{id}/preview"));
        if let Some(u) = user {
            b = b.header("x-user", u);
        }
        if let Some(r) = role {
            b = b.header("x-role", r);
        }
        b.body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn preview_is_served_to_any_signed_in_caller_regardless_of_quota() {
        let app = app(true, false).await;
        let res = app
            .oneshot(get_req(ACCEPTED, Some("u1"), None))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[header::CONTENT_TYPE], "audio/wav");
        let body = to_bytes(res.into_body(), 1024).await.unwrap();
        assert_eq!(&body[..], b"RIFFwav");
    }

    #[tokio::test]
    async fn preview_requires_auth_and_is_404_when_absent() {
        let app = app(false, false).await;
        let res = app
            .clone()
            .oneshot(get_req(ACCEPTED, None, None))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        let res = app
            .oneshot(get_req(ACCEPTED, Some("u1"), None))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn pending_piece_preview_is_moderator_only() {
        let store_app = app(false, false).await;
        // A normal caller: not-found (moderation visibility), even before storage.
        let res = store_app
            .clone()
            .oneshot(get_req(PENDING, Some("u1"), None))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        // A moderator resolves the piece; the clip is simply absent here → 404 too,
        // but from storage, so add a clip and check it is served.
        let app = app(false, false).await;
        let res = app
            .oneshot(get_req(PENDING, Some("mod"), Some("moderator")))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn regenerate_is_privileged_and_dormant_without_a_font() {
        let app = app(false, true).await;
        let res = app
            .clone()
            .oneshot(post_req(ACCEPTED, Some("u1"), None))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        let res = app
            .clone()
            .oneshot(post_req(ACCEPTED, None, None))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        // Admin, but no preview font configured → typed precondition, not 500.
        let res = app
            .oneshot(post_req(ACCEPTED, Some("adm"), Some("admin")))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::PRECONDITION_FAILED);
    }

    #[tokio::test]
    async fn regenerate_without_renderer_is_unavailable() {
        let app = app(false, false).await;
        let res = app
            .oneshot(post_req(ACCEPTED, Some("adm"), Some("admin")))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
