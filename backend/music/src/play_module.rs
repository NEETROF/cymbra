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

//! Play-activity domain logic (change: add-play-activity-profile): idempotent
//! ingestion of end-of-session stats and the per-day activity read that drives
//! the profile heatmap.
//!
//! The activity read is **gated fail-closed** on the target's profile visibility.
//! Visibility lives in the `user_account` schema, which this (`music`) module
//! cannot read directly (role isolation), so the gate is delegated in-process to
//! the [`UserPort`] — the same seam `cymbra-auth` uses — via
//! [`UserPort::activity_visible_to`]. No cross-schema access, no bypass: a
//! modified client calling the activity read directly still can't see a private
//! user's heatmap.

use std::sync::Arc;

use chrono::NaiveDate;
use cymbra_platform::{AppError, Result};
use cymbra_user_port::UserPort;

use crate::play::{PlayActivity, PlayRepo, PlaySession};
use crate::play_core;

/// Max plausible client UTC offset (±14h) — anything larger is a bad client.
const MAX_TZ_OFFSET_MINUTES: i32 = 14 * 60;

/// The caller-supplied fields of a session to record. `user_id` is NOT here — it
/// is always the authenticated caller, set by the module (a client cannot record
/// under another account).
#[derive(Debug, Clone)]
pub struct RecordInput {
    pub session_id: String,
    pub score_id: Option<String>,
    pub played_at_ms: i64,
    pub tz_offset_minutes: i32,
    pub overall_sync_pct: f32,
    pub session_result_json: String,
}

/// Ingestion + per-day activity, over an owner-scoped [`PlayRepo`] and the
/// [`UserPort`] visibility gate.
pub struct PlayModule {
    repo: Arc<dyn PlayRepo>,
    user: Arc<dyn UserPort>,
}

impl PlayModule {
    pub fn new(repo: Arc<dyn PlayRepo>, user: Arc<dyn UserPort>) -> Self {
        Self { repo, user }
    }

    /// Record one completed session for `owner_id`, idempotently by session id.
    /// Validates the id + summary fields; `overall_sync_pct` is clamped to
    /// `0..=100` (the score is a percentage). Retried deliveries of the same id
    /// are no-ops (no double-count).
    pub async fn record_session(&self, owner_id: &str, input: RecordInput) -> Result<()> {
        // A well-formed client session id (UUID v7) is the idempotency key.
        uuid::Uuid::parse_str(&input.session_id)
            .map_err(|_| AppError::InvalidArgument("invalid session id".into()))?;
        if !input.overall_sync_pct.is_finite() {
            return Err(AppError::InvalidArgument(
                "overall_sync_pct must be a finite number".into(),
            ));
        }
        if input.tz_offset_minutes.abs() > MAX_TZ_OFFSET_MINUTES {
            return Err(AppError::InvalidArgument(
                "implausible timezone offset".into(),
            ));
        }
        let session = PlaySession {
            session_id: input.session_id,
            user_id: owner_id.to_string(),
            score_id: input.score_id,
            played_at_ms: input.played_at_ms,
            tz_offset_minutes: input.tz_offset_minutes,
            overall_sync_pct: input.overall_sync_pct.clamp(0.0, 100.0),
            session_result_json: input.session_result_json,
        };
        self.repo.record(&session).await
    }

    /// Read `target_id`'s per-day activity as seen by `viewer_id`. Fail-closed: if
    /// the viewer may not see the target's activity (private / not age-eligible,
    /// and not the owner), this is `NotFound` — never revealing a private heatmap.
    /// `today` (UTC) is injected for the deterministic eligibility check.
    pub async fn play_activity(
        &self,
        viewer_id: &str,
        target_id: &str,
        today: NaiveDate,
    ) -> Result<PlayActivity> {
        if !self
            .user
            .activity_visible_to(target_id, viewer_id, today)
            .await?
        {
            return Err(AppError::NotFound("profile".into()));
        }
        let points = self.repo.session_points(target_id).await?;
        Ok(play_core::aggregate(&points))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::FakePlayRepo;
    use cymbra_user_port::MockUserPort;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 7, 28).unwrap()
    }

    fn input(session_id: &str, sync: f32) -> RecordInput {
        RecordInput {
            session_id: session_id.into(),
            score_id: Some("score-1".into()),
            played_at_ms: 1_718_494_200_000,
            tz_offset_minutes: 0,
            overall_sync_pct: sync,
            session_result_json: "{}".into(),
        }
    }

    /// A module whose visibility gate is stubbed to `allow`.
    fn module(repo: Arc<FakePlayRepo>, allow: bool) -> PlayModule {
        let mut user = MockUserPort::new();
        user.expect_activity_visible_to()
            .returning(move |_, _, _| Ok(allow));
        PlayModule::new(repo, Arc::new(user))
    }

    #[tokio::test]
    async fn record_is_idempotent_by_session_id() {
        let repo = Arc::new(FakePlayRepo::default());
        let m = module(repo.clone(), true);
        let sid = uuid::Uuid::now_v7().to_string();
        m.record_session("u1", input(&sid, 80.0)).await.unwrap();
        // Same id again → no-op (no double-count).
        m.record_session("u1", input(&sid, 80.0)).await.unwrap();
        assert_eq!(repo.count_for("u1"), 1);
    }

    #[tokio::test]
    async fn record_rejects_bad_session_id_and_clamps_sync() {
        let repo = Arc::new(FakePlayRepo::default());
        let m = module(repo.clone(), true);
        assert!(matches!(
            m.record_session("u1", input("not-a-uuid", 50.0)).await,
            Err(AppError::InvalidArgument(_))
        ));
        // Out-of-range sync is clamped to 0..=100.
        let sid = uuid::Uuid::now_v7().to_string();
        m.record_session("u1", input(&sid, 150.0)).await.unwrap();
        let pts = repo.session_points("u1").await.unwrap();
        assert_eq!(pts[0].overall_sync_pct, 100.0);
    }

    #[tokio::test]
    async fn activity_aggregates_when_visible() {
        let repo = Arc::new(FakePlayRepo::default());
        let m = module(repo.clone(), true);
        let day = 24 * 3600 * 1000i64;
        for (i, sync) in [70.0f32, 90.0].iter().enumerate() {
            let mut inp = input(&uuid::Uuid::now_v7().to_string(), *sync);
            inp.played_at_ms = 1_718_494_200_000 + i as i64 * day;
            m.record_session("owner", inp).await.unwrap();
        }
        let a = m.play_activity("viewer", "owner", today()).await.unwrap();
        assert_eq!(a.total_sessions, 2);
        assert_eq!(a.days.len(), 2);
    }

    #[tokio::test]
    async fn activity_is_not_found_when_gate_refuses() {
        let repo = Arc::new(FakePlayRepo::default());
        // Even with recorded sessions, a refused gate hides them (fail-closed).
        let m = module(repo.clone(), false);
        m.record_session("owner", input(&uuid::Uuid::now_v7().to_string(), 80.0))
            .await
            .unwrap();
        assert!(matches!(
            m.play_activity("viewer", "owner", today()).await,
            Err(AppError::NotFound(_))
        ));
    }
}
