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
//!
//! This adapter also resolves the caller's **drum audience** (change:
//! add-drum-scoring) through the same [`DrumsEligibility`] seam the score RPCs and
//! the HTTP preview route use — identity + memberships, never a request field —
//! and hands it to the module, which withholds a percussion piece's board from a
//! caller outside the audience.

// tonic's `Status` makes `Result<_, Status>` large; unavoidable on the generated
// service signatures.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::AuthIdentity;
use tonic::{Request, Response, Status};

use crate::leaderboard::Mode;
use crate::leaderboard_module::{Board, BoardEntry, BoardViewer, LeaderboardModule};
use crate::module::DrumsEligibility;
use crate::proto::{
    GetLeaderboardRequest, GetLeaderboardResponse, GetMyStandingsRequest, GetMyStandingsResponse,
    LeaderboardEntry as ProtoEntry, MyStanding as ProtoMyStanding,
    leaderboard_service_server::{LeaderboardService, LeaderboardServiceServer},
};

/// Wraps the leaderboard module as a tonic `LeaderboardService`.
pub struct LeaderboardGrpc {
    module: Arc<LeaderboardModule>,
    /// The drum-audience predicate (change: add-drum-scoring) — the SAME
    /// `ScoreModule::caller_may_see_percussion` every other gated surface uses,
    /// behind its trait seam, so no second implementation can drift. `None`
    /// (unwired) means "not eligible", exactly like an unwired flag service:
    /// percussion boards then read as boardless for everyone, staff included.
    drums: Option<Arc<dyn DrumsEligibility>>,
}

impl LeaderboardGrpc {
    pub fn new(module: Arc<LeaderboardModule>) -> Self {
        Self {
            module,
            drums: None,
        }
    }

    /// Wire the drum-audience seam (change: add-drum-scoring).
    pub fn with_drums(mut self, drums: Arc<dyn DrumsEligibility>) -> Self {
        self.drums = Some(drums);
        self
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> LeaderboardServiceServer<Self> {
        LeaderboardServiceServer::new(self)
    }

    /// The authenticated reader, with their drum audience resolved from their own
    /// identity and memberships — never from the request.
    async fn viewer<T>(&self, req: &Request<T>) -> Result<BoardViewer, Status> {
        let (user_id, staff) = req
            .extensions()
            .get::<AuthIdentity>()
            .map(|id| (id.user_id.clone(), is_staff(id)))
            .ok_or_else(|| Status::unauthenticated("missing identity"))?;
        let eligible_for_percussion = match &self.drums {
            Some(drums) => drums.eligible_for_percussion(&user_id, staff).await,
            None => false,
        };
        Ok(BoardViewer {
            user_id,
            eligible_for_percussion,
        })
    }
}

/// Staff = a moderator/admin role in any scope — the same test the score RPCs
/// apply before resolving the drum audience (`grpc.rs`).
fn is_staff(id: &AuthIdentity) -> bool {
    id.roles.iter().any(|r| r == "admin" || r == "moderator")
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
        let viewer = self.viewer(&req).await?;
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
        let viewer = self.viewer(&req).await?;
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
    use crate::module::MockDrumsEligibility;
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
        let err = g
            .get_my_standings(Request::new(GetMyStandingsRequest {
                score_ids: vec!["p".into()],
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    // --- the drum audience on the read RPCs (change: add-drum-scoring) ------
    //
    // One ineligible case per gated read path, plus the eligible counterpart:
    // the module's gate proves nothing about whether these RPCs consult it.

    /// The same service with the drum seam wired to `eligible`, over a percussion
    /// piece ("drum") and a keyboard one ("key") the caller is ranked on.
    async fn drum_grpc(eligible: bool) -> LeaderboardGrpc {
        let repo = Arc::new(FakeLeaderboardRepo::default());
        repo.accept("drum");
        repo.accept("key");
        repo.mark_percussion("drum");
        let g = grpc(repo, &["a1"]);
        for piece in ["drum", "key"] {
            for (u, sub) in [("a1", 95.0), ("me", 88.0)] {
                g.module
                    .maintain_from_session(&session(u, piece, sub))
                    .await
                    .unwrap();
            }
        }
        let mut drums = MockDrumsEligibility::new();
        drums
            .expect_eligible_for_percussion()
            .returning(move |_, _| eligible);
        g.with_drums(Arc::new(drums))
    }

    async fn board_of(g: &LeaderboardGrpc, score_id: &str) -> GetLeaderboardResponse {
        g.get_leaderboard(authed(
            GetLeaderboardRequest {
                score_id: score_id.into(),
                mode: "tempo".into(),
                offset: 0,
                limit: 50,
            },
            "me",
        ))
        .await
        .unwrap()
        .into_inner()
    }

    #[tokio::test]
    async fn get_leaderboard_hides_a_percussion_board_from_an_ineligible_caller() {
        let g = drum_grpc(false).await;
        let hidden = board_of(&g, "drum").await;
        // Exactly the answer for a piece that has no board — no entries, no
        // total, no own standing, though the caller is ranked on it.
        let boardless = board_of(&g, "never-played").await;
        assert_eq!(hidden.total, boardless.total);
        assert!(hidden.entries.is_empty() && hidden.own.is_none());
        // The keyboard board is untouched by the gate.
        let key = board_of(&g, "key").await;
        assert_eq!(key.total, 1);
        assert!(key.own.is_some());
    }

    #[tokio::test]
    async fn get_leaderboard_serves_a_percussion_board_to_an_eligible_caller() {
        let g = drum_grpc(true).await;
        let seen = board_of(&g, "drum").await;
        assert_eq!(seen.total, 1);
        assert_eq!(seen.entries[0].user_id, "a1");
        assert_eq!(seen.own.expect("own standing").subscore, 88.0);
    }

    #[tokio::test]
    async fn get_my_standings_skips_percussion_for_an_ineligible_caller() {
        let ids = vec!["key".to_string(), "drum".to_string()];
        let ineligible = drum_grpc(false).await;
        let resp = ineligible
            .get_my_standings(authed(
                GetMyStandingsRequest {
                    score_ids: ids.clone(),
                },
                "me",
            ))
            .await
            .unwrap()
            .into_inner();
        // The percussion piece contributes nothing — indistinguishable from a
        // boardless piece in the same batch.
        assert_eq!(resp.standings.len(), 1);
        assert_eq!(resp.standings[0].score_id, "key");

        let eligible = drum_grpc(true).await;
        let resp = eligible
            .get_my_standings(authed(GetMyStandingsRequest { score_ids: ids }, "me"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.standings.len(), 2);
        assert!(resp.standings.iter().any(|s| s.score_id == "drum"));
    }

    #[tokio::test]
    async fn an_unwired_drum_seam_withholds_percussion_boards() {
        // Fail-closed like an unwired flag service: no seam ⇒ nobody is eligible,
        // so a percussion board discloses nothing rather than everything.
        let repo = Arc::new(FakeLeaderboardRepo::default());
        repo.accept("drum");
        repo.mark_percussion("drum");
        let g = grpc(repo, &["a1"]);
        g.module
            .maintain_from_session(&session("a1", "drum", 95.0))
            .await
            .unwrap();
        let resp = board_of(&g, "drum").await;
        assert_eq!(resp.total, 0);
        assert!(resp.entries.is_empty());
    }
}
