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

//! The global leaderboard's gRPC **server** adapter (change: add-global-
//! leaderboard): exposes `GlobalLeaderboardService` by translating each RPC into a
//! [`GlobalLeaderboardModule`] call. The caller's identity comes from the
//! internal-token interceptor (request extension), never the body; viewing is open
//! to any authenticated caller, while the public/eligible listing gate + own-rank
//! are enforced inside the module.

// tonic's `Status` makes `Result<_, Status>` large; unavoidable on the generated
// service signatures.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::AuthIdentity;
use tonic::{Request, Response, Status};

use crate::global_leaderboard_module::{GlobalBoard, GlobalEntry, GlobalLeaderboardModule, Page};
use crate::leaderboard::Mode;
use crate::proto::{
    GetGlobalLeaderboardRequest, GetGlobalLeaderboardResponse,
    GlobalLeaderboardEntry as ProtoEntry, ListGlobalSeasonsRequest, ListGlobalSeasonsResponse,
    global_leaderboard_service_server::{GlobalLeaderboardService, GlobalLeaderboardServiceServer},
};

/// Wraps the global-leaderboard module as a tonic `GlobalLeaderboardService`.
pub struct GlobalLeaderboardGrpc {
    module: Arc<GlobalLeaderboardModule>,
}

impl GlobalLeaderboardGrpc {
    pub fn new(module: Arc<GlobalLeaderboardModule>) -> Self {
        Self { module }
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> GlobalLeaderboardServiceServer<Self> {
        GlobalLeaderboardServiceServer::new(self)
    }
}

fn caller<T>(req: &Request<T>) -> Result<String, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .map(|id| id.user_id.clone())
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

fn proto_entry(e: GlobalEntry) -> ProtoEntry {
    ProtoEntry {
        rank: e.rank,
        user_id: e.user_id,
        handle: e.handle.unwrap_or_default(),
        display_name: e.display_name.unwrap_or_default(),
        score: e.score as f32,
        contributing_pieces: e.contributing_pieces,
        reached_at_ms: e.reached_at_ms,
    }
}

fn to_response(board: GlobalBoard) -> GetGlobalLeaderboardResponse {
    GetGlobalLeaderboardResponse {
        season_id: board.season_id,
        entries: board.entries.into_iter().map(proto_entry).collect(),
        total: board.total,
        own: board.own.map(proto_entry),
    }
}

#[tonic::async_trait]
impl GlobalLeaderboardService for GlobalLeaderboardGrpc {
    async fn get_global_leaderboard(
        &self,
        req: Request<GetGlobalLeaderboardRequest>,
    ) -> Result<Response<GetGlobalLeaderboardResponse>, Status> {
        let viewer = caller(&req)?;
        let r = req.into_inner();
        let mode = Mode::parse(&r.mode).map_err(Status::from)?;
        let now = chrono::Utc::now();
        // An empty `season_id` means "the current season" (the client's default).
        let season = if r.season_id.is_empty() {
            None
        } else {
            Some(r.season_id.as_str())
        };
        let board = self
            .module
            .get_board(
                &viewer,
                mode,
                season,
                Page {
                    offset: r.offset as i64,
                    limit: r.limit as i64,
                },
                now,
            )
            .await?;
        Ok(Response::new(to_response(board)))
    }

    async fn list_global_seasons(
        &self,
        req: Request<ListGlobalSeasonsRequest>,
    ) -> Result<Response<ListGlobalSeasonsResponse>, Status> {
        // Identity is still required (viewing is open to any AUTHENTICATED user).
        caller(&req)?;
        let seasons = self
            .module
            .seasons(chrono::Utc::now().timestamp_millis())
            .await?;
        Ok(Response::new(ListGlobalSeasonsResponse {
            current_season_id: seasons.current_season_id,
            past_season_ids: seasons.past_season_ids,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_leaderboard::FakeGlobalLeaderboardRepo;
    use crate::global_leaderboard_core::GlobalConfig;
    use crate::leaderboard::BestCandidate;
    use cymbra_user_port::{MockUserPort, PlayerProfile, Visibility};

    fn grpc(
        repo: Arc<FakeGlobalLeaderboardRepo>,
        public: &'static [&'static str],
    ) -> GlobalLeaderboardGrpc {
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
        let module = Arc::new(GlobalLeaderboardModule::new(repo, Arc::new(user)));
        GlobalLeaderboardGrpc::new(module)
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

    /// Seed a season best in the CURRENT season (so the default-season read sees it).
    async fn play(g: &GlobalLeaderboardGrpc, user: &str, piece: &str, sub: f32) {
        let now = chrono::Utc::now().timestamp_millis();
        g.module
            .maintain_from_candidates(
                user,
                piece,
                &[BestCandidate {
                    mode: Mode::Tempo,
                    subscore: sub,
                    tiebreak_metric: 1.0,
                    achieved_at_ms: now,
                }],
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_global_leaderboard_returns_ranked_public_entries() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let g = grpc(repo.clone(), &["a1", "a2"]);
        play(&g, "a1", "p1", 95.0).await;
        play(&g, "a2", "p2", 80.0).await;
        play(&g, "priv", "p3", 99.0).await;

        let resp = g
            .get_global_leaderboard(authed(
                GetGlobalLeaderboardRequest {
                    mode: "tempo".into(),
                    season_id: String::new(),
                    offset: 0,
                    limit: 50,
                },
                "a1",
            ))
            .await
            .unwrap()
            .into_inner();
        // The empty season id echoes back the CURRENT season.
        let now = chrono::Utc::now().timestamp_millis();
        assert_eq!(resp.season_id, GlobalConfig::default().season_at(now).id);
        assert_eq!(resp.total, 2);
        assert_eq!(resp.entries[0].user_id, "a1");
        assert_eq!(resp.entries[0].handle, "@a1");
        assert_eq!(resp.entries[0].contributing_pieces, 1);
        // The private player is never listed to others.
        assert!(resp.entries.iter().all(|e| e.user_id != "priv"));
        assert_eq!(resp.own.expect("own standing").rank, 1);
    }

    #[tokio::test]
    async fn get_global_leaderboard_rejects_unknown_mode() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let g = grpc(repo, &[]);
        let err = g
            .get_global_leaderboard(authed(
                GetGlobalLeaderboardRequest {
                    mode: "bogus".into(),
                    season_id: String::new(),
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
    async fn rpcs_reject_unauthenticated_callers() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let g = grpc(repo, &[]);
        let err = g
            .get_global_leaderboard(Request::new(GetGlobalLeaderboardRequest {
                mode: "tempo".into(),
                season_id: String::new(),
                offset: 0,
                limit: 50,
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
        let err = g
            .list_global_seasons(Request::new(ListGlobalSeasonsRequest {}))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn global_boards_stay_open_to_an_ineligible_caller_and_name_no_piece() {
        // The global reads are deliberately OUTSIDE the drum-audience gate
        // (change: add-drum-scoring): their entries disclose players, scores and a
        // contributing-piece COUNT — never a piece identity — so gating them would
        // protect nothing and break a community surface. This service holds no
        // eligibility seam at all, which is the point: there is nothing to gate.
        const DRUM_PIECE: &str = "44444444-4444-7444-8444-444444444444";
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let g = grpc(repo.clone(), &["drummer"]);
        // A season standing fed entirely by a percussion piece…
        play(&g, "drummer", DRUM_PIECE, 95.0).await;
        // …read by a caller outside the drum audience.
        let resp = g
            .get_global_leaderboard(authed(
                GetGlobalLeaderboardRequest {
                    mode: "tempo".into(),
                    season_id: String::new(),
                    offset: 0,
                    limit: 50,
                },
                "keyboardist",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.total, 1, "the standing is still readable");
        assert_eq!(resp.entries[0].user_id, "drummer");
        assert_eq!(resp.entries[0].contributing_pieces, 1);
        assert!(
            !format!("{resp:?}").contains(DRUM_PIECE),
            "no piece identity leaves the global board"
        );
        // The season history is equally open.
        let seasons = g
            .list_global_seasons(authed(ListGlobalSeasonsRequest {}, "keyboardist"))
            .await
            .unwrap()
            .into_inner();
        assert!(!seasons.current_season_id.is_empty());
    }

    #[tokio::test]
    async fn list_global_seasons_returns_the_current_season() {
        let repo = Arc::new(FakeGlobalLeaderboardRepo::default());
        let g = grpc(repo, &[]);
        let resp = g
            .list_global_seasons(authed(ListGlobalSeasonsRequest {}, "a1"))
            .await
            .unwrap()
            .into_inner();
        let now = chrono::Utc::now().timestamp_millis();
        assert_eq!(
            resp.current_season_id,
            GlobalConfig::default().season_at(now).id
        );
        assert!(resp.past_season_ids.is_empty());
    }
}
