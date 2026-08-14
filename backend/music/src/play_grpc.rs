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
use crate::play_core::local_day;
use crate::play_module::{PlayModule, RecordInput, RecordPracticeInput};
use crate::proto::{
    DayActivity as ProtoDayActivity, GetPlayActivityRequest, GetPlayActivityResponse,
    RecordPlaySessionRequest, RecordPlaySessionResponse, RecordPracticeRequest,
    RecordPracticeResponse,
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

    /// The piece's catalog difficulty, for the play award's weighting (change:
    /// add-play-rewards, task 4.2). Resolved through the SAME `CatalogSearchRepo`
    /// seam that already tells a catalog score from an upload here.
    ///
    /// `None` for a bundled piece, a user upload, an unleveled catalog row, or a
    /// catalog read that failed — every one of which weighs **neutrally**, never
    /// zero: a missing level is a metadata gap, not a reason to tell a player their
    /// run was worthless (design D7).
    async fn piece_level(&self, score_id: &str) -> Option<String> {
        let catalog = self.catalog.as_ref()?;
        match catalog.hit_by_id(score_id, true).await {
            Ok(hit) => hit.and_then(|h| h.level),
            Err(e) => {
                tracing::warn!(
                    score_id = %score_id, error = %e,
                    "play rewards: could not resolve piece difficulty, weighing it neutrally"
                );
                None
            }
        }
    }

    /// Award performance points for a just-stored session, best-effort (change:
    /// add-play-rewards, design D5).
    ///
    /// **Never fails the ingest.** The client's outbox treats a failed call as
    /// undelivered and retries the whole session, so a player would see their run
    /// "not saved" because points could not be written. A lost award is a handful
    /// of points; a failed ack is a user-visible bug. Returns the points awarded,
    /// which the ack carries back (0 when the seam is not wired).
    ///
    /// A session with no `score_id` awards nothing: the per-piece diminishing curve
    /// has nothing to count against, and paying an unidentifiable piece would be
    /// precisely the infinite well the curve exists to close.
    async fn award_performance(
        &self,
        user_id: &str,
        score_id: Option<&str>,
        accuracy_pct: f32,
        session_id: &str,
    ) -> i32 {
        let (Some(rewards), Some(score_id)) = (&self.rewards, score_id) else {
            return 0;
        };
        let level = self.piece_level(score_id).await;
        match rewards
            .award_performance(user_id, score_id, accuracy_pct, level, session_id)
            .await
        {
            Ok(points) => points as i32,
            Err(e) => {
                tracing::warn!(
                    user_id = %user_id, score_id = %score_id, error = %e,
                    "play rewards: performance award not recorded"
                );
                0
            }
        }
    }

    /// Award the once-a-day practice award for a just-stored practice session,
    /// best-effort — same ack contract as [`Self::award_performance`].
    async fn award_practice(&self, user_id: &str, practiced_at_ms: i64, tz_offset: i32) -> i32 {
        let Some(rewards) = &self.rewards else {
            return 0;
        };
        // The PLAYER's local day, not the server's: the same bucketing the heatmap
        // and the streak badges use, so midnight UTC falling mid-evening never
        // costs someone their daily award (design D3).
        let day = local_day(practiced_at_ms, tz_offset).to_string();
        match rewards.award_practice(user_id, &day).await {
            Ok(points) => points as i32,
            Err(e) => {
                tracing::warn!(
                    user_id = %user_id, local_day = %day, error = %e,
                    "play rewards: practice award not recorded"
                );
                0
            }
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
        let session_id = r.session_id.clone();
        let score_id = r.score_id.clone();
        let accuracy = r.overall_sync_pct;
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
        // The run is stored; now pay for it (change: add-play-rewards). Keyed on the
        // session id, so this is exactly-once even though the ingest is
        // at-least-once — and best-effort, so a failure to pay never fails the ack.
        let points_awarded = self
            .award_performance(&owner, score_id.as_deref(), accuracy, &session_id)
            .await;
        Ok(Response::new(RecordPlaySessionResponse { points_awarded }))
    }

    async fn record_practice(
        &self,
        req: Request<RecordPracticeRequest>,
    ) -> Result<Response<RecordPracticeResponse>, Status> {
        let owner = caller(&req)?;
        let r = req.into_inner();
        let practiced_at_ms = r.practiced_at_ms;
        let tz_offset_minutes = r.tz_offset_minutes;
        // A successful return IS the persisted-ack the client's outbox waits for.
        self.module
            .record_practice(
                &owner,
                RecordPracticeInput {
                    session_id: r.session_id,
                    score_id: r.score_id,
                    practiced_at_ms: r.practiced_at_ms,
                    tz_offset_minutes: r.tz_offset_minutes,
                },
            )
            .await?;
        // Showing up is worth acknowledging, once for the player's local day
        // (change: add-play-rewards). Best-effort, exactly like the scored path.
        let points_awarded = self
            .award_practice(&owner, practiced_at_ms, tz_offset_minutes)
            .await;
        Ok(Response::new(RecordPracticeResponse { points_awarded }))
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
                    practice_count: d.practice_count as i32,
                })
                .collect(),
            total_sessions: activity.total_sessions as i32,
            total_practices: activity.total_practices as i32,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_search::{FakeCatalogRow, FakeCatalogSearchRepo};
    use crate::curation_rewards::{
        CurationRewardsRepo, FakeCurationRewardsRepo, MockCurationRewardsSink,
    };
    use crate::curation_rewards_module::CurationRewardsModule;
    use crate::play::FakePlayRepo;
    use crate::play_module::PlayModule;
    use cymbra_platform::AppError;
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

    // --- play rewards (change: add-play-rewards) ---------------------------

    fn practice(at_ms: i64, tz_offset_minutes: i32) -> RecordPracticeRequest {
        RecordPracticeRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            score_id: Some(CATALOG_ID.into()),
            practiced_at_ms: at_ms,
            tz_offset_minutes,
        }
    }

    #[tokio::test]
    async fn the_ack_carries_what_the_session_earned() {
        let (g, _rewards) = grpc_with_rewards();
        // A good run of an ADVANCED catalog piece: the first band, difficulty-weighted.
        let resp = g
            .record_play_session(authed(session(Some(CATALOG_ID)), "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.points_awarded, 16);
    }

    #[tokio::test]
    async fn a_sub_floor_session_is_stored_and_reports_zero() {
        let (g, _rewards) = grpc_with_rewards();
        let mut req = session(Some(CATALOG_ID));
        req.overall_sync_pct = 20.0; // keys mashed / walked away
        let resp = g
            .record_play_session(authed(req, "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.points_awarded, 0);
        // Still recorded as activity — the run happened.
        let activity = g
            .get_play_activity(authed(
                GetPlayActivityRequest {
                    user_id: String::new(),
                },
                "u1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(activity.total_sessions, 1);
    }

    #[tokio::test]
    async fn a_redelivered_session_reports_zero_the_second_time() {
        // The client's outbox retries until acked, so the SAME request arrives
        // twice. It must pay once and report 0 on the retry.
        let (g, _rewards) = grpc_with_rewards();
        let req = session(Some(CATALOG_ID));
        let first = g
            .record_play_session(authed(req.clone(), "u1"))
            .await
            .unwrap()
            .into_inner();
        let second = g
            .record_play_session(authed(req, "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(first.points_awarded, 16);
        assert_eq!(second.points_awarded, 0);
    }

    #[tokio::test]
    async fn an_unleveled_piece_still_pays_neutrally() {
        // A user upload carries no catalog level: neutral weight, never zero.
        let (g, _rewards) = grpc_with_rewards();
        let resp = g
            .record_play_session(authed(session(Some(UPLOAD_ID)), "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.points_awarded, 8);
    }

    #[tokio::test]
    async fn practice_pays_once_for_the_players_local_day() {
        let (g, _rewards) = grpc_with_rewards();
        // 2024-06-15T23:30Z at +60min is already the 16th LOCALLY...
        const JUN15_2330Z: i64 = 1_718_494_200_000;
        let first = g
            .record_practice(authed(practice(JUN15_2330Z, 60), "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(first.points_awarded, 3);
        // ...so an hour later — a different UTC day, the SAME local day — pays
        // nothing more.
        let same_local_day = g
            .record_practice(authed(practice(JUN15_2330Z + 3_600_000, 60), "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(same_local_day.points_awarded, 0);
        // The next local day pays again.
        let next_day = g
            .record_practice(authed(practice(JUN15_2330Z + 86_400_000, 60), "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(next_day.points_awarded, 3);
    }

    #[tokio::test]
    async fn a_failing_award_still_stores_and_acks_the_session() {
        // The contract that protects the player (design D5): the outbox treats a
        // failed call as undelivered and retries the whole session, so an award
        // failure must NEVER fail the ack — the player would see their run "not
        // saved" over a handful of points.
        let mut sink = MockCurationRewardsSink::new();
        sink.expect_record_engagement().returning(|_, _| Ok(()));
        sink.expect_award_performance()
            .returning(|_, _, _, _, _| Err(AppError::Internal(anyhow::anyhow!("ledger down"))));
        sink.expect_award_practice()
            .returning(|_, _| Err(AppError::Internal(anyhow::anyhow!("ledger down"))));
        let catalog = Arc::new(FakeCatalogSearchRepo::with(vec![FakeCatalogRow::new(
            CATALOG_ID,
            "Clair de Lune",
            "Claude Debussy",
            Some("advanced"),
        )]));
        let g = grpc(true).with_rewards(Arc::new(sink), catalog);

        let played = g
            .record_play_session(authed(session(Some(CATALOG_ID)), "u1"))
            .await
            .expect("a failed award must not fail the ingest")
            .into_inner();
        assert_eq!(played.points_awarded, 0);
        let practised = g
            .record_practice(authed(practice(1_718_494_200_000, 0), "u1"))
            .await
            .expect("a failed award must not fail the ingest")
            .into_inner();
        assert_eq!(practised.points_awarded, 0);

        // Both runs are stored and readable, which is what the client is waiting on.
        let activity = g
            .get_play_activity(authed(
                GetPlayActivityRequest {
                    user_id: String::new(),
                },
                "u1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(activity.total_sessions, 1);
        assert_eq!(activity.total_practices, 1);
    }

    #[tokio::test]
    async fn an_unwired_rewards_seam_reports_zero() {
        // A deployment without rewards wired: ingest works, the ack simply says 0.
        let g = grpc(true);
        let resp = g
            .record_play_session(authed(session(Some(CATALOG_ID)), "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.points_awarded, 0);
        let p = g
            .record_practice(authed(practice(1_718_494_200_000, 0), "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(p.points_awarded, 0);
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
    async fn record_practice_shows_up_as_activity_but_not_as_a_play() {
        let g = grpc(true);
        g.record_practice(authed(
            RecordPracticeRequest {
                session_id: uuid::Uuid::now_v7().to_string(),
                score_id: Some("s".into()),
                practiced_at_ms: 1_718_494_200_000,
                tz_offset_minutes: 0,
            },
            "u1",
        ))
        .await
        .unwrap();
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
        assert_eq!(resp.total_sessions, 0);
        assert_eq!(resp.total_practices, 1);
        assert_eq!(resp.days.len(), 1);
        assert_eq!(resp.days[0].count, 0);
        assert_eq!(resp.days[0].practice_count, 1);
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
