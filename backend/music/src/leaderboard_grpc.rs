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

//! The leaderboard module's gRPC **server** adapter (change: add-play-
//! leaderboards): exposes `LeaderboardService` by translating each RPC into a
//! [`LeaderboardModule`] call. The caller's identity comes from the internal-token
//! interceptor (request extension), never the body; the public/private listing
//! gate + own-rank are enforced inside the module.

// tonic's `Status` makes `Result<_, Status>` large; unavoidable on the generated
// service signatures.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::AuthIdentity;
use tonic::{Request, Response, Status};

use crate::leaderboard::Mode;
use crate::leaderboard_module::{Board, BoardEntry, LeaderboardModule};
use crate::proto::{
    GetLeaderboardRequest, GetLeaderboardResponse, GetMyStandingsRequest, GetMyStandingsResponse,
    LeaderboardEntry as ProtoEntry, MyStanding as ProtoMyStanding,
    leaderboard_service_server::{LeaderboardService, LeaderboardServiceServer},
};

/// Wraps the leaderboard module as a tonic `LeaderboardService`.
pub struct LeaderboardGrpc {
    module: Arc<LeaderboardModule>,
}

impl LeaderboardGrpc {
    pub fn new(module: Arc<LeaderboardModule>) -> Self {
        Self { module }
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> LeaderboardServiceServer<Self> {
        LeaderboardServiceServer::new(self)
    }
}

fn caller<T>(req: &Request<T>) -> Result<String, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .map(|id| id.user_id.clone())
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

fn proto_entry(e: BoardEntry) -> ProtoEntry {
    ProtoEntry {
        rank: e.rank,
        user_id: e.user_id,
        handle: e.handle.unwrap_or_default(),
        display_name: e.display_name.unwrap_or_default(),
        subscore: e.subscore,
        tiebreak_metric: e.tiebreak_metric,
        achieved_at_ms: e.achieved_at_ms,
    }
}

fn to_response(board: Board) -> GetLeaderboardResponse {
    GetLeaderboardResponse {
        entries: board.entries.into_iter().map(proto_entry).collect(),
        total: board.total,
        own: board.own.map(proto_entry),
    }
}

#[tonic::async_trait]
impl LeaderboardService for LeaderboardGrpc {
    async fn get_leaderboard(
        &self,
        req: Request<GetLeaderboardRequest>,
    ) -> Result<Response<GetLeaderboardResponse>, Status> {
        let viewer = caller(&req)?;
        let r = req.into_inner();
        let mode = Mode::parse(&r.mode).map_err(Status::from)?;
        let today = chrono::Utc::now().date_naive();
        let board = self
            .module
            .get_board(
                &viewer,
                &r.score_id,
                mode,
                r.offset as i64,
                r.limit as i64,
                today,
            )
            .await?;
        Ok(Response::new(to_response(board)))
    }

    async fn get_my_standings(
        &self,
        req: Request<GetMyStandingsRequest>,
    ) -> Result<Response<GetMyStandingsResponse>, Status> {
        let viewer = caller(&req)?;
        let r = req.into_inner();
        let today = chrono::Utc::now().date_naive();
        let standings = self
            .module
            .my_standings(&viewer, &r.score_ids, today)
            .await?;
        Ok(Response::new(GetMyStandingsResponse {
            standings: standings
                .into_iter()
                .map(|s| ProtoMyStanding {
                    score_id: s.score_id,
                    rank: s.rank,
                    subscore: s.subscore,
                    mode: s.mode.as_str().to_string(),
                })
                .collect(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leaderboard::FakeLeaderboardRepo;
    use crate::play::PlaySession;
    use cymbra_user_port::{MockUserPort, PlayerProfile, Visibility};

    fn session(user: &str, score: &str, sub: f64) -> PlaySession {
        PlaySession {
            session_id: uuid::Uuid::now_v7().to_string(),
            user_id: user.into(),
            score_id: Some(score.into()),
            played_at_ms: 1,
            tz_offset_minutes: 0,
            overall_sync_pct: sub as f32,
            session_result_json: format!(
                r#"{{"freeSyncPct": {sub}, "freeOnsetCount": 6, "avgFreeOffsetMs": 5.0}}"#
            ),
        }
    }

    fn grpc(repo: Arc<FakeLeaderboardRepo>, public: &'static [&'static str]) -> LeaderboardGrpc {
        let mut user = MockUserPort::new();
        user.expect_listable_profiles().returning(move |ids, _| {
            Ok(ids
                .iter()
                .filter(|id| public.contains(&id.as_str()))
                .map(|id| PlayerProfile {
                    user_id: id.clone(),
                    handle: Some(format!("@{id}")),
                    display_name: Some(id.clone()),
                    visibility: Visibility::Public,
                })
                .collect())
        });
        user.expect_get_player_profile().returning(|_, target, _| {
            Ok(PlayerProfile {
                user_id: target.to_string(),
                handle: Some(format!("@{target}")),
                display_name: Some(target.to_string()),
                visibility: Visibility::Private,
            })
        });
        let module = Arc::new(LeaderboardModule::new(repo, Arc::new(user)));
        LeaderboardGrpc::new(module)
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
    async fn get_leaderboard_returns_ranked_public_entries() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        repo.accept("p");
        let g = grpc(repo.clone(), &["a1", "a2"]);
        for (u, sub) in [("a1", 95.0), ("a2", 80.0), ("priv", 99.0)] {
            g.module
                .maintain_from_session(&session(u, "p", sub))
                .await
                .unwrap();
        }
        let resp = g
            .get_leaderboard(authed(
                GetLeaderboardRequest {
                    score_id: "p".into(),
                    mode: "tempo".into(),
                    offset: 0,
                    limit: 50,
                },
                "a1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.total, 2);
        assert_eq!(resp.entries.len(), 2);
        assert_eq!(resp.entries[0].user_id, "a1");
        assert_eq!(resp.entries[0].handle, "@a1");
        assert!(resp.own.is_some());
        assert_eq!(resp.own.unwrap().rank, 1);
    }

    #[tokio::test]
    async fn get_leaderboard_rejects_unknown_mode() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        let g = grpc(repo, &[]);
        let err = g
            .get_leaderboard(authed(
                GetLeaderboardRequest {
                    score_id: "p".into(),
                    mode: "bogus".into(),
                    offset: 0,
                    limit: 50,
                },
                "a1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn get_leaderboard_rejects_unauthenticated() {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        let g = grpc(repo, &[]);
        let err = g
            .get_leaderboard(Request::new(GetLeaderboardRequest {
                score_id: "p".into(),
                mode: "tempo".into(),
                offset: 0,
                limit: 50,
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }
}
