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

//! The music module's gRPC **server** adapter: exposes `ScoreService` by
//! translating each RPC into a [`ScoreModule`] call. The caller's identity comes
//! from the internal-token interceptor (request extension), never the body, and
//! every op is scoped to that `owner_id`.

// tonic's `Status` makes `Result<_, Status>` large; unavoidable on the generated
// service signatures.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::AuthIdentity;
use tonic::{Request, Response, Status};

use crate::catalog_search::{CatalogHit, CatalogQuery, SortKey, is_moderation_sort_field};
use crate::module::{ScoreModule, UploadInput};
use crate::proto::{
    CatalogHit as ProtoCatalogHit, DeleteScoreRequest, DeleteScoreResponse,
    GetCatalogScoreBytesRequest, GetCatalogScoreBytesResponse, GetCatalogScoreRequest,
    GetRatingPreviewBytesRequest, GetRatingPreviewBytesResponse, GetScoreBytesRequest,
    GetScoreBytesResponse, ListMyScoresRequest, ListMyScoresResponse, ListRatingDeckRequest,
    ListRatingDeckResponse, ListSavedCatalogScoresRequest, ListSavedCatalogScoresResponse,
    RemoveSavedCatalogScoreRequest, RemoveSavedCatalogScoreResponse, SaveCatalogScoreRequest,
    SaveCatalogScoreResponse, ScoreRecord, SearchCatalogRequest, SearchCatalogResponse,
    SetModerationStatusRequest, SetModerationStatusResponse, SetScoreFavoriteRequest,
    SetScoreFavoriteResponse, SubmitScoreRatingRequest, SubmitScoreRatingResponse,
    UploadScoreRequest,
    score_service_server::{ScoreService, ScoreServiceServer},
};
use crate::user_scores::UserScore;

/// Wraps the score module as a tonic `ScoreService`.
pub struct ScoreGrpc {
    module: Arc<ScoreModule>,
}

impl ScoreGrpc {
    pub fn new(module: Arc<ScoreModule>) -> Self {
        Self { module }
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> ScoreServiceServer<Self> {
        ScoreServiceServer::new(self)
    }
}

fn owner<T>(req: &Request<T>) -> Result<String, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .map(|id| id.user_id.clone())
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

/// The full authenticated identity (for role checks). Same unauthenticated guard
/// as [`owner`]; cloned so it outlives `req.into_inner()`.
fn identity<T>(req: &Request<T>) -> Result<AuthIdentity, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .cloned()
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

fn to_record(s: UserScore) -> ScoreRecord {
    let m = s.meta;
    ScoreRecord {
        id: s.id,
        title: m.title,
        composer: m.composer,
        level: s.level,
        created_at: s.created_at,
        measure_count: m.measure_count,
        time_sig: m.time_sig,
        key_fifths: m.key_fifths,
        min_note_value: m.facets.min_note_value.map(i32::from),
        tempo_bpm: m.facets.tempo_bpm.map(i32::from),
        note_count: m.facets.note_count as i32,
        lowest_midi: m.facets.lowest_midi.map(i32::from),
        highest_midi: m.facets.highest_midi.map(i32::from),
        favorite: s.favorite,
    }
}

fn to_hit(h: CatalogHit) -> ProtoCatalogHit {
    ProtoCatalogHit {
        id: h.id,
        title: h.title,
        composer: h.composer,
        level: h.level,
        license: h.license,
        source: h.source,
        arranger: h.arranger,
        min_note_value: h.min_note_value,
        tempo_bpm: h.tempo_bpm,
        note_count: h.note_count,
        lowest_midi: h.lowest_midi,
        highest_midi: h.highest_midi,
        time_sig: h.time_sig,
        key_fifths: h.key_fifths,
        needs_review: h.needs_review,
        moderation_status: h.moderation_status,
    }
}

#[tonic::async_trait]
impl ScoreService for ScoreGrpc {
    async fn upload_score(
        &self,
        req: Request<UploadScoreRequest>,
    ) -> Result<Response<ScoreRecord>, Status> {
        let owner_id = owner(&req)?;
        let r = req.into_inner();
        let rec = self
            .module
            .upload(
                &owner_id,
                UploadInput {
                    data: r.data,
                    filename: r.filename,
                    level: r.level,
                    rights_basis: r.rights_basis,
                    rights_ack: r.rights_ack,
                    fallback_title: r.fallback_title,
                    fallback_composer: r.fallback_composer,
                },
            )
            .await?;
        Ok(Response::new(to_record(rec)))
    }

    async fn list_my_scores(
        &self,
        req: Request<ListMyScoresRequest>,
    ) -> Result<Response<ListMyScoresResponse>, Status> {
        let owner_id = owner(&req)?;
        let scores = self.module.list(&owner_id).await?;
        Ok(Response::new(ListMyScoresResponse {
            scores: scores.into_iter().map(to_record).collect(),
        }))
    }

    async fn delete_score(
        &self,
        req: Request<DeleteScoreRequest>,
    ) -> Result<Response<DeleteScoreResponse>, Status> {
        let owner_id = owner(&req)?;
        let id = req.into_inner().id;
        self.module.delete(&owner_id, &id).await?;
        Ok(Response::new(DeleteScoreResponse {}))
    }

    async fn get_score_bytes(
        &self,
        req: Request<GetScoreBytesRequest>,
    ) -> Result<Response<GetScoreBytesResponse>, Status> {
        let owner_id = owner(&req)?;
        let id = req.into_inner().id;
        let data = self.module.get_bytes(&owner_id, &id).await?;
        Ok(Response::new(GetScoreBytesResponse { data }))
    }

    async fn set_score_favorite(
        &self,
        req: Request<SetScoreFavoriteRequest>,
    ) -> Result<Response<SetScoreFavoriteResponse>, Status> {
        let owner_id = owner(&req)?;
        let r = req.into_inner();
        self.module
            .set_favorite(&owner_id, &r.id, r.favorite)
            .await?;
        Ok(Response::new(SetScoreFavoriteResponse {}))
    }

    // --- Score Hub (change: score-hub-search) -------------------------------
    // Every handler asserts an authenticated identity via `owner()` (the strict
    // interceptor already rejects unauthenticated calls; this is defense-in-depth
    // and the identity source for the owner-scoped ops). Search + bytes are
    // catalog-wide, so they require identity but do not scope by owner.

    async fn search_catalog(
        &self,
        req: Request<SearchCatalogRequest>,
    ) -> Result<Response<SearchCatalogResponse>, Status> {
        let id = identity(&req)?; // authenticated-only (catalog is public, not owner-scoped)
        // The moderation-status filter and any moderation-oriented sort key are
        // privileged and back-office-only: when the caller uses either, they MUST be
        // authorised — reject with PERMISSION_DENIED and run no query otherwise.
        // #1 restricted this to `admin`; this change (add-moderation-back-office)
        // widens it to admin-or-(music) moderator.
        let uses_moderation = req.get_ref().moderation_status.is_some()
            || req.get_ref().review_queue.unwrap_or(false)
            || req
                .get_ref()
                .sort
                .iter()
                .any(|k| is_moderation_sort_field(&k.field));
        if uses_moderation {
            cymbra_platform::guard::require_moderator_or_admin(&id)?;
        }
        let r = req.into_inner();
        let offset = r.offset;
        let query = CatalogQuery {
            query: r.query,
            author: r.author,
            level: r.level,
            facets: crate::catalog_search::FacetFilters {
                is_piano: r.is_piano,
                max_note_value: r.max_note_value.map(|v| v.clamp(0, i16::MAX as i32) as i16),
                has_chords: r.has_chords,
                has_tuplets: r.has_tuplets,
                has_dotted: r.has_dotted,
                max_ambitus_semitones: r
                    .max_ambitus_semitones
                    .map(|v| v.clamp(0, i16::MAX as i32) as i16),
                staff_count: r.staff_count.map(|v| v.clamp(0, i16::MAX as i32) as i16),
                min_bpm: r.min_bpm,
                max_bpm: r.max_bpm,
            },
            moderation_status: r.moderation_status,
            review_queue: r.review_queue.unwrap_or(false),
            sort: r
                .sort
                .into_iter()
                .map(|k| SortKey {
                    field: k.field,
                    descending: k.descending,
                })
                .collect(),
            limit: r.limit as i64,
            offset: r.offset as i64,
        };
        let (hits, total) = self.module.search_catalog(query).await?;
        let next_offset = offset.max(0) + hits.len() as i32;
        Ok(Response::new(SearchCatalogResponse {
            hits: hits.into_iter().map(to_hit).collect(),
            next_offset,
            total: total.clamp(0, i32::MAX as i64) as i32,
        }))
    }

    async fn save_catalog_score(
        &self,
        req: Request<SaveCatalogScoreRequest>,
    ) -> Result<Response<SaveCatalogScoreResponse>, Status> {
        let owner_id = owner(&req)?;
        let catalog_id = req.into_inner().catalog_id;
        self.module
            .save_catalog_score(&owner_id, &catalog_id)
            .await?;
        Ok(Response::new(SaveCatalogScoreResponse {}))
    }

    async fn remove_saved_catalog_score(
        &self,
        req: Request<RemoveSavedCatalogScoreRequest>,
    ) -> Result<Response<RemoveSavedCatalogScoreResponse>, Status> {
        let owner_id = owner(&req)?;
        let catalog_id = req.into_inner().catalog_id;
        self.module
            .remove_saved_catalog_score(&owner_id, &catalog_id)
            .await?;
        Ok(Response::new(RemoveSavedCatalogScoreResponse {}))
    }

    async fn list_saved_catalog_scores(
        &self,
        req: Request<ListSavedCatalogScoresRequest>,
    ) -> Result<Response<ListSavedCatalogScoresResponse>, Status> {
        let owner_id = owner(&req)?;
        let hits = self.module.list_saved_catalog_scores(&owner_id).await?;
        Ok(Response::new(ListSavedCatalogScoresResponse {
            hits: hits.into_iter().map(to_hit).collect(),
        }))
    }

    async fn get_catalog_score_bytes(
        &self,
        req: Request<GetCatalogScoreBytesRequest>,
    ) -> Result<Response<GetCatalogScoreBytesResponse>, Status> {
        // Authenticated-only. A moderator/admin may open a score in any moderation
        // status (to review it); a normal caller is served only `accepted` bytes and
        // gets not-found otherwise (change: add-score-moderation-gating, widened to
        // moderator by add-moderation-back-office).
        let id = identity(&req)?;
        let allow_unvalidated = id.is_admin() || id.has_role("moderator");
        let catalog_id = req.into_inner().catalog_id;
        let data = self
            .module
            .get_catalog_bytes(&catalog_id, allow_unvalidated)
            .await?;
        Ok(Response::new(GetCatalogScoreBytesResponse { data }))
    }

    async fn get_rating_preview_bytes(
        &self,
        req: Request<GetRatingPreviewBytesRequest>,
    ) -> Result<Response<GetRatingPreviewBytesResponse>, Status> {
        // Authenticated-only (any signed-in rater). Serves a `pending` or `accepted`
        // score's bytes for the deck's read-only preview; a `rejected`/unknown id is
        // not-found. The player-open bytes path and library save stay accepted-only
        // (change: rate-pending-scores).
        let _ = identity(&req)?;
        let catalog_id = req.into_inner().catalog_id;
        let data = self.module.rating_preview_bytes(&catalog_id).await?;
        Ok(Response::new(GetRatingPreviewBytesResponse { data }))
    }

    async fn get_catalog_score(
        &self,
        req: Request<GetCatalogScoreRequest>,
    ) -> Result<Response<ProtoCatalogHit>, Status> {
        // Same gate as fetch-bytes: a moderator/admin resolves any status; a normal
        // caller only `accepted` (non-`accepted` id → not-found). Lets a detail view
        // load by id without depending on a prior list.
        let id = identity(&req)?;
        let allow_unvalidated = id.is_admin() || id.has_role("moderator");
        let catalog_id = req.into_inner().catalog_id;
        let hit = self
            .module
            .get_catalog_hit(&catalog_id, allow_unvalidated)
            .await?;
        Ok(Response::new(to_hit(hit)))
    }

    async fn set_moderation_status(
        &self,
        req: Request<SetModerationStatusRequest>,
    ) -> Result<Response<SetModerationStatusResponse>, Status> {
        // Evaluate action (change: add-moderation-back-office): moderator/admin only.
        // The reviewer id is the authenticated caller — never the body — and is
        // stamped as `reviewed_by` alongside the status.
        let id = identity(&req)?;
        cymbra_platform::guard::require_moderator_or_admin(&id)?;
        let r = req.into_inner();
        self.module
            .set_moderation_status(&id.user_id, &r.score_id, &r.status)
            .await?;
        Ok(Response::new(SetModerationStatusResponse {}))
    }

    async fn submit_score_rating(
        &self,
        req: Request<SubmitScoreRatingRequest>,
    ) -> Result<Response<SubmitScoreRatingResponse>, Status> {
        // Any signed-in user may rate (change: add-app-score-rating). The rater is
        // the authenticated caller — never the body. The module validates the
        // verdict/stars and rejects a non-`accepted`/unknown target.
        let user_id = owner(&req)?;
        let r = req.into_inner();
        let agg = self
            .module
            .submit_rating(&user_id, &r.catalog_id, &r.verdict, r.stars)
            .await?;
        Ok(Response::new(SubmitScoreRatingResponse {
            rating_avg: agg.avg_effective,
            rating_count: agg.count.clamp(0, i32::MAX as i64) as i32,
            dislike_count: agg.dislike.clamp(0, i32::MAX as i64) as i32,
            like_count: agg.like.clamp(0, i32::MAX as i64) as i32,
            love_count: agg.love.clamp(0, i32::MAX as i64) as i32,
        }))
    }

    async fn list_rating_deck(
        &self,
        req: Request<ListRatingDeckRequest>,
    ) -> Result<Response<ListRatingDeckResponse>, Status> {
        // Authenticated-only; the un-rated exclusion is per caller (change:
        // improve-rating-deck-sourcing).
        let user_id = owner(&req)?;
        let r = req.into_inner();
        let offset = r.offset;
        let hits = self
            .module
            .list_rating_deck(&user_id, r.limit as i64, r.offset as i64)
            .await?;
        let next_offset = offset.max(0) + hits.len() as i32;
        Ok(Response::new(ListRatingDeckResponse {
            hits: hits.into_iter().map(to_hit).collect(),
            next_offset,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_storage::{FakeStore, ObjectStorage};

    use crate::catalog_search::{FakeCatalogRow, FakeCatalogSearchRepo};
    use crate::score_rating::FakeScoreRatingRepo;
    use crate::user_library::FakeUserLibraryRepo;
    use crate::user_scores::FakeUserScoreRepo;

    const DEBUSSY: &str = "11111111-1111-7111-8111-111111111111";
    const SATIE: &str = "22222222-2222-7222-8222-222222222222";
    const PENDING: &str = "44444444-4444-7444-8444-444444444444";
    const REJECTED: &str = "55555555-5555-7555-8555-555555555555";

    /// A `ScoreGrpc` over a seeded catalog + an object store holding the catalog
    /// scores' bytes, so byte fetches resolve. The catalog carries two `accepted`
    /// scores, one `pending` score, and one `rejected` score (change:
    /// add-score-moderation-gating / rate-pending-scores), so the moderation gate and
    /// the rating/preview paths can be exercised at the handler layer.
    async fn grpc() -> ScoreGrpc {
        let store = Arc::new(FakeStore::default());
        for id in [DEBUSSY, SATIE, PENDING, REJECTED] {
            store
                .put(&format!("safe/pdmx/{id}.mxl"), b"<score/>".to_vec())
                .await
                .unwrap();
        }
        let catalog = Arc::new(FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new(DEBUSSY, "Clair de Lune", "Claude Debussy", Some("advanced"))
                .piano(),
            FakeCatalogRow::new(SATIE, "Gymnopédie", "Erik Satie", Some("beginner")).piano(),
            FakeCatalogRow::new(PENDING, "Pending Piece", "Anon", Some("beginner"))
                .with_moderation_status("pending"),
            FakeCatalogRow::new(REJECTED, "Rejected Piece", "Anon", Some("beginner"))
                .with_moderation_status("rejected"),
        ]));
        let module = Arc::new(ScoreModule::new(
            Arc::new(FakeUserScoreRepo::default()),
            catalog,
            Arc::new(FakeUserLibraryRepo::default()),
            Arc::new(FakeScoreRatingRepo::default()),
            store,
            5,
            7,
            8 * 1024 * 1024,
        ));
        ScoreGrpc::new(module)
    }

    /// Attach an authenticated identity to a request (as the interceptor would).
    fn authed<T>(msg: T, user_id: &str) -> Request<T> {
        authed_with(msg, user_id, &["user"])
    }

    /// Attach an authenticated identity carrying the `admin` role.
    fn authed_admin<T>(msg: T, user_id: &str) -> Request<T> {
        authed_with(msg, user_id, &["user", "admin"])
    }

    /// Attach an authenticated identity carrying the `moderator` role.
    fn authed_moderator<T>(msg: T, user_id: &str) -> Request<T> {
        authed_with(msg, user_id, &["user", "moderator"])
    }

    fn authed_with<T>(msg: T, user_id: &str, roles: &[&str]) -> Request<T> {
        let mut req = Request::new(msg);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.into(),
            audience: "music".into(),
            roles: roles.iter().map(|r| (*r).into()).collect(),
        });
        req
    }

    fn search(query: &str, author: Option<&str>, level: Option<&str>) -> SearchCatalogRequest {
        SearchCatalogRequest {
            query: query.into(),
            author: author.map(Into::into),
            level: level.map(Into::into),
            limit: 50,
            offset: 0,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn unauthenticated_requests_are_rejected() {
        let g = grpc().await;
        // No AuthIdentity in the extensions → unauthenticated on every hub RPC.
        let err = g
            .search_catalog(Request::new(search("", None, None)))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
        let err = g
            .save_catalog_score(Request::new(SaveCatalogScoreRequest {
                catalog_id: DEBUSSY.into(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
        let err = g
            .list_saved_catalog_scores(Request::new(ListSavedCatalogScoresRequest {}))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn search_composes_query_author_and_level() {
        let g = grpc().await;
        // Author + level filter narrows to the one advanced Debussy work.
        let resp = g
            .search_catalog(authed(search("", Some("Debussy"), Some("advanced")), "u1"))
            .await
            .unwrap()
            .into_inner();
        let ids: Vec<&str> = resp.hits.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, [DEBUSSY]);
        assert_eq!(resp.next_offset, 1);
        assert_eq!(resp.total, 1); // full match count for the filter, on the response
        assert_eq!(resp.hits[0].license, "CC-BY-4.0");
    }

    #[tokio::test]
    async fn save_list_remove_round_trip_reflects_across_calls() {
        let g = grpc().await;
        // Save two, list returns them newest-first for the SAME owner (the sync
        // source of truth — a later list reflects earlier writes).
        g.save_catalog_score(authed(
            SaveCatalogScoreRequest {
                catalog_id: SATIE.into(),
            },
            "u1",
        ))
        .await
        .unwrap();
        g.save_catalog_score(authed(
            SaveCatalogScoreRequest {
                catalog_id: DEBUSSY.into(),
            },
            "u1",
        ))
        .await
        .unwrap();
        let listed = g
            .list_saved_catalog_scores(authed(ListSavedCatalogScoresRequest {}, "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(
            listed
                .hits
                .iter()
                .map(|h| h.id.as_str())
                .collect::<Vec<_>>(),
            [DEBUSSY, SATIE]
        );
        // Another owner sees none of u1's saves (isolation).
        let other = g
            .list_saved_catalog_scores(authed(ListSavedCatalogScoresRequest {}, "u2"))
            .await
            .unwrap()
            .into_inner();
        assert!(other.hits.is_empty());
        // Remove one; a subsequent list reflects it.
        g.remove_saved_catalog_score(authed(
            RemoveSavedCatalogScoreRequest {
                catalog_id: SATIE.into(),
            },
            "u1",
        ))
        .await
        .unwrap();
        let listed = g
            .list_saved_catalog_scores(authed(ListSavedCatalogScoresRequest {}, "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(
            listed
                .hits
                .iter()
                .map(|h| h.id.as_str())
                .collect::<Vec<_>>(),
            [DEBUSSY]
        );
    }

    #[tokio::test]
    async fn bytes_for_known_and_unknown_id() {
        let g = grpc().await;
        let resp = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: DEBUSSY.into(),
                },
                "u1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.data, b"<score/>");
        // Unknown id → NotFound.
        let err = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: "99999999-9999-7999-8999-999999999999".into(),
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    // --- moderation gating (change: add-score-moderation-gating) -------------

    fn search_status(status: &str) -> SearchCatalogRequest {
        SearchCatalogRequest {
            limit: 50,
            offset: 0,
            moderation_status: Some(status.into()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn normal_search_hides_pending_scores() {
        let g = grpc().await;
        // A normal caller browsing (no status filter) sees only the accepted rows,
        // never the pending one.
        let resp = g
            .search_catalog(authed(search("", None, None), "u1"))
            .await
            .unwrap()
            .into_inner();
        let ids: Vec<&str> = resp.hits.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, [DEBUSSY, SATIE]); // title_norm order; PENDING excluded
        assert!(!ids.contains(&PENDING));
    }

    #[tokio::test]
    async fn non_admin_supplying_status_filter_is_permission_denied() {
        let g = grpc().await;
        // A normal (non-admin) caller that sets the privileged filter is rejected,
        // and no query runs (the request never reaches the search path).
        let err = g
            .search_catalog(authed(search_status("pending"), "u1"))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn admin_supplying_status_filter_is_honoured() {
        let g = grpc().await;
        // An admin caller's `pending` filter returns exactly the pending score.
        let resp = g
            .search_catalog(authed_admin(search_status("pending"), "admin1"))
            .await
            .unwrap()
            .into_inner();
        let ids: Vec<&str> = resp.hits.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, [PENDING]);
    }

    #[tokio::test]
    async fn bytes_of_pending_score_gated_by_role() {
        let g = grpc().await;
        // A normal caller cannot open the pending score's bytes (not found)…
        let err = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: PENDING.into(),
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
        // …but an admin reviewer is served them.
        let resp = g
            .get_catalog_score_bytes(authed_admin(
                GetCatalogScoreBytesRequest {
                    catalog_id: PENDING.into(),
                },
                "admin1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.data, b"<score/>");
    }

    // --- moderator role widening + evaluate + sort (add-moderation-back-office) --

    #[tokio::test]
    async fn moderator_may_use_status_filter_and_open_pending_bytes() {
        let g = grpc().await;
        // A moderator (not admin) can now use the privileged status filter…
        let resp = g
            .search_catalog(authed_moderator(search_status("pending"), "mod1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(
            resp.hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            [PENDING]
        );
        // …and open a pending score's bytes to review it.
        let resp = g
            .get_catalog_score_bytes(authed_moderator(
                GetCatalogScoreBytesRequest {
                    catalog_id: PENDING.into(),
                },
                "mod1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.data, b"<score/>");
    }

    #[tokio::test]
    async fn moderation_sort_key_is_privileged() {
        let g = grpc().await;
        let sorted = || SearchCatalogRequest {
            limit: 50,
            sort: vec![crate::proto::SortKey {
                field: "status_rank".into(),
                descending: true,
            }],
            ..Default::default()
        };
        // A normal caller sorting by a moderation-oriented key is denied…
        let err = g.search_catalog(authed(sorted(), "u1")).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        // …while a moderator may (a substance-only sort stays open to anyone, so
        // this asserts the privileged path specifically).
        assert!(
            g.search_catalog(authed_moderator(sorted(), "mod1"))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn substance_sort_key_is_open_to_normal_callers() {
        let g = grpc().await;
        let req = SearchCatalogRequest {
            limit: 50,
            sort: vec![crate::proto::SortKey {
                field: "note_count".into(),
                descending: true,
            }],
            ..Default::default()
        };
        // No moderation key → no privilege required.
        assert!(g.search_catalog(authed(req, "u1")).await.is_ok());
    }

    #[tokio::test]
    async fn set_moderation_status_requires_moderator_or_admin() {
        let g = grpc().await;
        let req = || SetModerationStatusRequest {
            score_id: PENDING.into(),
            status: "accepted".into(),
        };
        // A normal caller is denied and nothing changes.
        let err = g
            .set_moderation_status(authed(req(), "u1"))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        // A moderator succeeds; the score is now visible to a normal search.
        g.set_moderation_status(authed_moderator(req(), "mod1"))
            .await
            .unwrap();
        let resp = g
            .search_catalog(authed(search("", None, None), "u1"))
            .await
            .unwrap()
            .into_inner();
        assert!(resp.hits.iter().any(|h| h.id == PENDING));
    }

    // --- score ratings (change: add-app-score-rating) ------------------------

    const RATER: &str = "66666666-6666-7666-8666-666666666666";

    fn rating(catalog_id: &str, verdict: &str, stars: Option<i32>) -> SubmitScoreRatingRequest {
        SubmitScoreRatingRequest {
            catalog_id: catalog_id.into(),
            verdict: verdict.into(),
            stars,
        }
    }

    #[tokio::test]
    async fn submit_rating_requires_authentication() {
        let g = grpc().await;
        // No identity in the extensions → unauthenticated.
        let err = g
            .submit_score_rating(Request::new(rating(DEBUSSY, "like", None)))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn submit_rating_records_and_returns_the_aggregate() {
        let g = grpc().await;
        // A signed-in user rates an accepted score with explicit stars.
        let resp = g
            .submit_score_rating(authed(rating(DEBUSSY, "love", Some(5)), RATER))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.rating_count, 1);
        assert_eq!(resp.love_count, 1);
        assert!((resp.rating_avg - 5.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn submit_rating_on_a_pending_score_succeeds() {
        let g = grpc().await;
        // change: rate-pending-scores — a signed-in user CAN rate a pending candidate
        // (the community helps moderate); the rating is recorded.
        let resp = g
            .submit_score_rating(authed(rating(PENDING, "like", None), RATER))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.rating_count, 1);
        assert_eq!(resp.like_count, 1);
    }

    #[tokio::test]
    async fn submit_rating_on_a_rejected_score_is_rejected() {
        let g = grpc().await;
        // A `rejected` score is never rateable.
        let err = g
            .submit_score_rating(authed(rating(REJECTED, "like", None), RATER))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn rating_preview_bytes_serves_pending_and_accepted_refuses_rejected() {
        let g = grpc().await;
        // A signed-in rater previews both an accepted score and a pending candidate.
        for id in [DEBUSSY, PENDING] {
            let resp = g
                .get_rating_preview_bytes(authed(
                    GetRatingPreviewBytesRequest {
                        catalog_id: id.into(),
                    },
                    RATER,
                ))
                .await
                .unwrap()
                .into_inner();
            assert!(!resp.data.is_empty());
        }
        // A rejected score is never previewable.
        let err = g
            .get_rating_preview_bytes(authed(
                GetRatingPreviewBytesRequest {
                    catalog_id: REJECTED.into(),
                },
                RATER,
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
        // Unauthenticated → rejected.
        let err = g
            .get_rating_preview_bytes(Request::new(GetRatingPreviewBytesRequest {
                catalog_id: DEBUSSY.into(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn player_open_bytes_stays_accepted_only_for_a_normal_caller() {
        let g = grpc().await;
        // The player-open path is unchanged by rate-pending-scores: a normal caller
        // still cannot open a pending score there…
        let err = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: PENDING.into(),
                },
                RATER,
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
        // …but the accepted score opens fine.
        let resp = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: DEBUSSY.into(),
                },
                RATER,
            ))
            .await
            .unwrap()
            .into_inner();
        assert!(!resp.data.is_empty());
    }

    #[tokio::test]
    async fn list_rating_deck_requires_auth() {
        let g = grpc().await;
        // Unauthenticated → rejected.
        let err = g
            .list_rating_deck(Request::new(ListRatingDeckRequest {
                limit: 50,
                offset: 0,
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
        // A signed-in caller gets the un-rated piano scores. The pending/rejected rows
        // in this fixture are non-piano, so they are excluded by the deck's piano gate
        // regardless of status (pending sourcing is covered by the module test).
        let resp = g
            .list_rating_deck(authed(
                ListRatingDeckRequest {
                    limit: 50,
                    offset: 0,
                },
                RATER,
            ))
            .await
            .unwrap()
            .into_inner();
        let ids: Vec<&str> = resp.hits.iter().map(|h| h.id.as_str()).collect();
        assert!(ids.contains(&DEBUSSY) && ids.contains(&SATIE));
        assert!(!ids.contains(&PENDING));
    }
}
