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

use crate::catalog_search::CatalogSearchRepo;
use crate::curation_rewards::CurationRewardsSink;
use crate::module::is_rateable_catalog_score;
use crate::play_module::{PlayModule, RecordInput};
use crate::proto::{
    DayActivity as ProtoDayActivity, GetPlayActivityRequest, GetPlayActivityResponse,
    RecordPlaySessionRequest, RecordPlaySessionResponse,
    play_service_server::{PlayService, PlayServiceServer},
};

/// Wraps the play module as a tonic `PlayService`.
pub struct PlayGrpc {
    module: Arc<PlayModule>,
    rewards: Option<Arc<dyn CurationRewardsSink>>,
    catalog: Option<Arc<dyn CatalogSearchRepo>>,
}

impl PlayGrpc {
    pub fn new(module: Arc<PlayModule>) -> Self {
        Self {
            module,
            rewards: None,
            catalog: None,
        }
    }

    /// Wire the curation-rewards seam so an ingested play session records the
    /// coverage **engagement** signal (change: add-post-play-rating-prompt).
    ///
    /// This lives here, at the composition seam, rather than inside [`PlayModule`]:
    /// play ingest has nothing to do with rewards, and threading a rewards
    /// dependency through the module would couple two unrelated capabilities. It
    /// exists for the offline case — a score opened from the encrypted local cache
    /// never fetches bytes, so the player-open signal never fires for it, and the
    /// session ingest is the only server-observed evidence that it was played.
    ///
    /// `catalog` is required to tell a catalog score from anything else: a session's
    /// `score_id` is whatever the player ranked by, which for a **user upload** is a
    /// UUID from `music.user_scores`. Only a real, rateable catalog score may be
    /// recorded.
    pub fn with_rewards(
        mut self,
        rewards: Arc<dyn CurationRewardsSink>,
        catalog: Arc<dyn CatalogSearchRepo>,
    ) -> Self {
        self.rewards = Some(rewards);
        self.catalog = Some(catalog);
        self
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> PlayServiceServer<Self> {
        PlayServiceServer::new(self)
    }

    /// Record the coverage engagement signal for a just-ingested session,
    /// best-effort.
    ///
    /// Only a **rateable catalog** score counts, and that has to be *resolved*, not
    /// guessed: a session's `score_id` is whatever the player ranked by — a slug for
    /// a bundled piece (`"ode-to-joy"`), a `music.user_scores` UUID for an upload, a
    /// `music.catalog_scores` UUID for a catalog score. Only the last is rateable,
    /// and only it satisfies `score_engagements`' foreign key. Anything else is
    /// skipped silently: playing your own upload is perfectly normal, not an anomaly
    /// worth logging.
    ///
    /// A genuine failure only costs the user coverage points on a later rating — it
    /// never fails the ingest, which the client is waiting on as its persisted-ack.
    async fn record_engagement(&self, user_id: &str, score_id: Option<&str>) {
        let (Some(rewards), Some(catalog), Some(score_id)) =
            (&self.rewards, &self.catalog, score_id)
        else {
            return;
        };
        match is_rateable_catalog_score(catalog.as_ref(), score_id).await {
            Ok(true) => {}
            Ok(false) => return, // a bundled piece or a user upload — not rateable
            Err(e) => {
                tracing::warn!(
                    user_id = %user_id, score_id = %score_id, error = %e,
                    "curation: could not resolve played score, engagement skipped"
                );
                return;
            }
        }
        if let Err(e) = rewards.record_engagement(user_id, score_id).await {
            tracing::warn!(
                user_id = %user_id, catalog_id = %score_id, error = %e,
                "curation: play engagement not recorded"
            );
        }
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
        // Playing a catalog score is genuine engagement for coverage points (change:
        // add-post-play-rating-prompt) — recorded before the ingest so it is not lost
        // if the ingest fails, and idempotent per (user, score) at the repo.
        self.record_engagement(&owner, r.score_id.as_deref()).await;
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
    use crate::catalog_search::{FakeCatalogRow, FakeCatalogSearchRepo};
    use crate::curation_rewards::{CurationRewardsRepo, FakeCurationRewardsRepo};
    use crate::curation_rewards_module::CurationRewardsModule;
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

    /// A catalog holding one accepted score. A `score_id` absent from it models a
    /// bundled piece or a user upload — neither is a catalog score.
    const CATALOG_ID: &str = "11111111-1111-7111-8111-111111111111";
    /// A UUID shaped exactly like a catalog id but living in `music.user_scores`:
    /// the user-upload case that used to slip past a UUID-shape check and violate
    /// `score_engagements`' foreign key.
    const UPLOAD_ID: &str = "22222222-2222-7222-8222-222222222222";

    /// A [`PlayGrpc`] with the rewards + catalog seams wired, plus the fake rewards
    /// repo so a test can assert what engagement the ingest recorded.
    fn grpc_with_rewards() -> (PlayGrpc, Arc<FakeCurationRewardsRepo>) {
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        let rewards = Arc::new(CurationRewardsModule::new(repo.clone()));
        let catalog = Arc::new(FakeCatalogSearchRepo::with(vec![FakeCatalogRow::new(
            CATALOG_ID,
            "Clair de Lune",
            "Claude Debussy",
            Some("advanced"),
        )]));
        (grpc(true).with_rewards(rewards, catalog), repo)
    }

    fn session(score_id: Option<&str>) -> RecordPlaySessionRequest {
        RecordPlaySessionRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            score_id: score_id.map(str::to_string),
            played_at_ms: 1_718_494_200_000,
            tz_offset_minutes: 0,
            overall_sync_pct: 80.0,
            session_result_json: "{}".into(),
        }
    }

    #[tokio::test]
    async fn ingest_records_engagement_for_a_catalog_score() {
        // change: add-post-play-rating-prompt — the offline case: a score played from
        // the local cache never fetches bytes, so the ingest is the only evidence it
        // was played. Without this the post-play rating would earn no coverage.
        let (g, rewards) = grpc_with_rewards();
        g.record_play_session(authed(session(Some(CATALOG_ID)), "u1"))
            .await
            .unwrap();
        assert!(rewards.has_engagement("u1", CATALOG_ID).await.unwrap());
        // Idempotent across repeated sessions on the same score.
        g.record_play_session(authed(session(Some(CATALOG_ID)), "u1"))
            .await
            .unwrap();
        assert!(rewards.has_engagement("u1", CATALOG_ID).await.unwrap());
    }

    #[tokio::test]
    async fn ingest_ignores_a_user_upload_that_merely_looks_like_a_catalog_id() {
        // REGRESSION: a user upload's `score_id` is a UUID too, but it lives in
        // `music.user_scores`. Recording it violated `score_engagements`' foreign key
        // to `music.catalog_scores` — an internal error on every session of an
        // uploaded score. The id must be RESOLVED against the catalog, not sniffed.
        let (g, rewards) = grpc_with_rewards();
        g.record_play_session(authed(session(Some(UPLOAD_ID)), "u1"))
            .await
            .unwrap();
        assert!(!rewards.has_engagement("u1", UPLOAD_ID).await.unwrap());
    }

    #[tokio::test]
    async fn ingest_ignores_bundled_and_missing_score_ids() {
        // A bundled score's id is a slug, not a UUID: it is not in the catalog, is
        // not rateable, and must never reach the rewards store.
        let (g, rewards) = grpc_with_rewards();
        g.record_play_session(authed(session(Some("ode-to-joy")), "u1"))
            .await
            .unwrap();
        assert!(!rewards.has_engagement("u1", "ode-to-joy").await.unwrap());
        // A session with no score id at all is ingested normally and records nothing.
        g.record_play_session(authed(session(None), "u1"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn ingest_ignores_a_rejected_catalog_score() {
        // A rejected score is not rateable, so playing it earns no engagement.
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        let rewards = Arc::new(CurationRewardsModule::new(repo.clone()));
        let catalog = Arc::new(FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new(CATALOG_ID, "Rejected", "Anon", Some("beginner"))
                .with_moderation_status("rejected"),
        ]));
        let g = grpc(true).with_rewards(rewards, catalog);
        g.record_play_session(authed(session(Some(CATALOG_ID)), "u1"))
            .await
            .unwrap();
        assert!(!repo.has_engagement("u1", CATALOG_ID).await.unwrap());
    }

    #[tokio::test]
    async fn ingest_succeeds_without_the_rewards_seam() {
        // The seam is optional (a deployment without rewards wired): ingest — the
        // client's persisted-ack — must not depend on it.
        let g = grpc(true);
        g.record_play_session(authed(session(Some(CATALOG_ID)), "u1"))
            .await
            .unwrap();
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
