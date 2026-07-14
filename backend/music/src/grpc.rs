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

use crate::module::{ScoreModule, UploadInput};
use crate::proto::{
    DeleteScoreRequest, DeleteScoreResponse, GetScoreBytesRequest, GetScoreBytesResponse,
    ListMyScoresRequest, ListMyScoresResponse, ScoreRecord, UploadScoreRequest,
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
    ScoreRecord {
        id: s.id,
        title: s.title,
        composer: s.composer,
        level: s.level,
        created_at: s.created_at,
        measure_count: s.measure_count,
        time_sig: s.time_sig,
        key_fifths: s.key_fifths,
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
}
