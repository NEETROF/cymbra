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

//! End-of-session play stats: the `play_sessions` data-access port (change:
//! add-play-activity-profile). gRPC-facing, so it returns the platform
//! [`Result`]/`AppError`. Ingestion is **idempotent by the client session id**
//! (a resent id is a no-op) so the client can retry-until-acked with no double
//! count; reads are **owner-scoped** at the data layer (the visibility gate lives
//! in [`crate::play_module`]).

use std::sync::Mutex;

use async_trait::async_trait;
use chrono::NaiveDate;
use cymbra_platform::Result;

/// One completed play session, as ingested. `session_id` is the client-generated
/// UUID v7 (the idempotency key); `session_result_json` is the full immutable
/// session-result record, stored as-is for future replay/leaderboards.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaySession {
    pub session_id: String,
    pub user_id: String,
    pub score_id: Option<String>,
    /// When the session ended (unix epoch ms, client wall clock).
    pub played_at_ms: i64,
    /// Client UTC offset at `played_at_ms`, for local-day bucketing.
    pub tz_offset_minutes: i32,
    /// The success score 0..100 (summary tier; kept long-term).
    pub overall_sync_pct: f32,
    /// Full immutable record as JSON (heavy tier; pruned after retention). Empty
    /// string ⇒ stored as SQL NULL.
    pub session_result_json: String,
}

/// The minimal per-session data the heatmap aggregation needs (summary tier).
#[derive(Debug, Clone, PartialEq)]
pub struct SessionPoint {
    pub played_at_ms: i64,
    pub tz_offset_minutes: i32,
    pub overall_sync_pct: f32,
}

/// One local day of activity (a heatmap cell): count + average overall sync %.
#[derive(Debug, Clone, PartialEq)]
pub struct DayActivity {
    pub day: NaiveDate,
    pub count: u32,
    pub avg_sync_pct: f32,
}

/// A user's per-day activity plus their songs-played total.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayActivity {
    pub days: Vec<DayActivity>,
    pub total_sessions: u32,
}

/// Storage surface for play sessions.
#[async_trait]
pub trait PlayRepo: Send + Sync {
    /// Persist a session, **idempotently** by `session_id` (`ON CONFLICT DO
    /// NOTHING`): a resent id is a no-op success, never a double-count.
    async fn record(&self, session: &PlaySession) -> Result<()>;

    /// A user's session points (summary tier), for on-demand aggregation.
    async fn session_points(&self, user_id: &str) -> Result<Vec<SessionPoint>>;
}

// --- In-memory fake (tests) -------------------------------------------------

/// In-memory [`PlayRepo`] for unit tests (no Postgres). Idempotent by session id.
#[derive(Default)]
pub struct FakePlayRepo {
    sessions: Mutex<Vec<PlaySession>>,
}

#[async_trait]
impl PlayRepo for FakePlayRepo {
    async fn record(&self, session: &PlaySession) -> Result<()> {
        let mut s = self.sessions.lock().unwrap();
        // Idempotent by session id: a resent id is dropped (mirrors ON CONFLICT).
        if s.iter().any(|e| e.session_id == session.session_id) {
            return Ok(());
        }
        s.push(session.clone());
        Ok(())
    }

    async fn session_points(&self, user_id: &str) -> Result<Vec<SessionPoint>> {
        let s = self.sessions.lock().unwrap();
        Ok(s.iter()
            .filter(|e| e.user_id == user_id)
            .map(|e| SessionPoint {
                played_at_ms: e.played_at_ms,
                tz_offset_minutes: e.tz_offset_minutes,
                overall_sync_pct: e.overall_sync_pct,
            })
            .collect())
    }
}

impl FakePlayRepo {
    /// Test helper: number of stored sessions for a user (to assert no double count).
    pub fn count_for(&self, user_id: &str) -> usize {
        self.sessions
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.user_id == user_id)
            .count()
    }
}
