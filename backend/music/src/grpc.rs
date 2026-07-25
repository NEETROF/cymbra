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

use crate::catalog_search::{CatalogHit, CatalogQuery};
use crate::module::{ScoreModule, UploadInput};
use crate::proto::{
    CatalogHit as ProtoCatalogHit, DeleteScoreRequest, DeleteScoreResponse,
    GetCatalogScoreBytesRequest, GetCatalogScoreBytesResponse, GetScoreBytesRequest,
    GetScoreBytesResponse, ListMyScoresRequest, ListMyScoresResponse,
    ListSavedCatalogScoresRequest, ListSavedCatalogScoresResponse, RemoveSavedCatalogScoreRequest,
    RemoveSavedCatalogScoreResponse, SaveCatalogScoreRequest, SaveCatalogScoreResponse,
    ScoreRecord, SearchCatalogRequest, SearchCatalogResponse, SetScoreFavoriteRequest,
    SetScoreFavoriteResponse, UploadScoreRequest,
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
        owner(&req)?; // authenticated-only (catalog is public, not owner-scoped)
        let r = req.into_inner();
        let offset = r.offset;
        let query = CatalogQuery {
            query: r.query,
            author: r.author,
            level: r.level,
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
        owner(&req)?; // authenticated-only
        let catalog_id = req.into_inner().catalog_id;
        let data = self.module.get_catalog_bytes(&catalog_id).await?;
        Ok(Response::new(GetCatalogScoreBytesResponse { data }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_storage::{FakeStore, ObjectStorage};

    use crate::catalog_search::{FakeCatalogRow, FakeCatalogSearchRepo};
    use crate::user_library::FakeUserLibraryRepo;
    use crate::user_scores::FakeUserScoreRepo;

    const DEBUSSY: &str = "11111111-1111-7111-8111-111111111111";
    const SATIE: &str = "22222222-2222-7222-8222-222222222222";

    /// A `ScoreGrpc` over a seeded catalog + an object store holding the catalog
    /// scores' bytes, so byte fetches resolve.
    async fn grpc() -> ScoreGrpc {
        let store = Arc::new(FakeStore::default());
        for id in [DEBUSSY, SATIE] {
            store
                .put(&format!("safe/pdmx/{id}.mxl"), b"<score/>".to_vec())
                .await
                .unwrap();
        }
        let catalog = Arc::new(FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new(DEBUSSY, "Clair de Lune", "Claude Debussy", Some("advanced")),
            FakeCatalogRow::new(SATIE, "Gymnopédie", "Erik Satie", Some("beginner")),
        ]));
        let module = Arc::new(ScoreModule::new(
            Arc::new(FakeUserScoreRepo::default()),
            catalog,
            Arc::new(FakeUserLibraryRepo::default()),
            store,
            5,
            7,
            8 * 1024 * 1024,
        ));
        ScoreGrpc::new(module)
    }

    /// Attach an authenticated identity to a request (as the interceptor would).
    fn authed<T>(msg: T, user_id: &str) -> Request<T> {
        let mut req = Request::new(msg);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.into(),
            audience: "music".into(),
            roles: vec!["user".into()],
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
}
