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

//! The play module's gRPC **server** adapter (change: add-play-activity-profile):
//! exposes `PlayService` by translating each RPC into a [`PlayModule`] call. The
//! caller's identity comes from the internal-token interceptor (request
//! extension), never the body; ingestion is scoped to that caller and the
//! activity read is gated fail-closed inside the module.

// tonic's `Status` makes `Result<_, Status>` large; unavoidable on the generated
// service signatures.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::AuthIdentity;
use tonic::{Request, Response, Status};

use crate::play_module::{PlayModule, RecordInput};
use crate::proto::{
    DayActivity as ProtoDayActivity, GetPlayActivityRequest, GetPlayActivityResponse,
    RecordPlaySessionRequest, RecordPlaySessionResponse,
    play_service_server::{PlayService, PlayServiceServer},
};

/// Wraps the play module as a tonic `PlayService`.
pub struct PlayGrpc {
    module: Arc<PlayModule>,
}

impl PlayGrpc {
    pub fn new(module: Arc<PlayModule>) -> Self {
        Self { module }
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> PlayServiceServer<Self> {
        PlayServiceServer::new(self)
    }
}

fn caller<T>(req: &Request<T>) -> Result<String, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .map(|id| id.user_id.clone())
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

#[tonic::async_trait]
impl PlayService for PlayGrpc {
    async fn record_play_session(
        &self,
        req: Request<RecordPlaySessionRequest>,
    ) -> Result<Response<RecordPlaySessionResponse>, Status> {
        let owner = caller(&req)?;
        let r = req.into_inner();
        // A successful return IS the persisted-ack the client waits for.
        self.module
            .record_session(
                &owner,
                RecordInput {
                    session_id: r.session_id,
                    score_id: r.score_id,
                    played_at_ms: r.played_at_ms,
                    tz_offset_minutes: r.tz_offset_minutes,
                    overall_sync_pct: r.overall_sync_pct,
                    session_result_json: r.session_result_json,
                },
            )
            .await?;
        Ok(Response::new(RecordPlaySessionResponse {}))
    }

    async fn get_play_activity(
        &self,
        req: Request<GetPlayActivityRequest>,
    ) -> Result<Response<GetPlayActivityResponse>, Status> {
        let viewer = caller(&req)?;
        let r = req.into_inner();
        // An empty target means "my own activity" (convenience for the self view).
        let target = if r.user_id.is_empty() {
            viewer.clone()
        } else {
            r.user_id
        };
        let today = chrono::Utc::now().date_naive();
        let activity = self.module.play_activity(&viewer, &target, today).await?;
        Ok(Response::new(GetPlayActivityResponse {
            days: activity
                .days
                .into_iter()
                .map(|d| ProtoDayActivity {
                    day: d.day.to_string(),
                    count: d.count as i32,
                    avg_sync_pct: d.avg_sync_pct,
                })
                .collect(),
            total_sessions: activity.total_sessions as i32,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::FakePlayRepo;
    use crate::play_module::PlayModule;
    use cymbra_user_port::MockUserPort;

    fn grpc(allow: bool) -> PlayGrpc {
        let mut user = MockUserPort::new();
        user.expect_activity_visible_to()
            .returning(move |_, _, _| Ok(allow));
        let module = Arc::new(PlayModule::new(
            Arc::new(FakePlayRepo::default()),
            Arc::new(user),
        ));
        PlayGrpc::new(module)
    }

    fn authed<T>(msg: T, user_id: &str) -> Request<T> {
        let mut req = Request::new(msg);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.into(),
            audience: "music".into(),
            roles: vec!["user".into()],
            ..Default::default()
        });
        req
    }

    #[tokio::test]
    async fn record_then_read_own_activity() {
        let g = grpc(true);
        let sid = uuid::Uuid::now_v7().to_string();
        g.record_play_session(authed(
            RecordPlaySessionRequest {
                session_id: sid,
                score_id: Some("s".into()),
                played_at_ms: 1_718_494_200_000,
                tz_offset_minutes: 0,
                overall_sync_pct: 80.0,
                session_result_json: "{}".into(),
            },
            "u1",
        ))
        .await
        .unwrap();
        // Empty user_id → own activity.
        let resp = g
            .get_play_activity(authed(
                GetPlayActivityRequest {
                    user_id: String::new(),
                },
                "u1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.total_sessions, 1);
        assert_eq!(resp.days.len(), 1);
    }

    #[tokio::test]
    async fn rpcs_reject_unauthenticated() {
        let g = grpc(true);
        let err = g
            .get_play_activity(Request::new(GetPlayActivityRequest {
                user_id: "u1".into(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }
}
