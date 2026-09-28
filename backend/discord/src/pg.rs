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

//! Postgres adapters (change: add-discord-notifications, design D4, D5) — thin
//! I/O glue, excluded from the coverage gate and exercised by the `#[ignore]`d
//! integration tests. **The role is the pool's**: read the constructor docs
//! before wiring one.

use async_trait::async_trait;
use sqlx::{Connection, PgConnection, PgPool, Row};

use crate::event::AnnouncementEvent;
use crate::ports::{AnnouncementLedger, AnnouncementSource, Claim};
use crate::render::{ScoreCard, SoundFontCard};

/// Enqueue `discord_notify` for `event` inside the caller's transaction.
///
/// The job exists iff the caller's write commits (a rollback takes it away), and
/// it is **best-effort**: it runs in a savepoint, so a failed enqueue (a missing
/// grant, a full queue) is logged and rolled back alone, and never aborts the
/// domain write — an announcement is never worth losing a moderation decision.
pub async fn enqueue_notify(conn: &mut PgConnection, event: &AnnouncementEvent) {
    if let Err(e) = try_enqueue(conn, event).await {
        tracing::warn!(
            key = %event.dedup_key(), error = %e,
            "discord announcement not enqueued"
        );
    }
}

async fn try_enqueue(conn: &mut PgConnection, event: &AnnouncementEvent) -> anyhow::Result<()> {
    let request = crate::notify_request(event)?;
    let mut savepoint = conn.begin().await?;
    match cymbra_jobs::transactional_enqueue(&mut savepoint, &request).await {
        Ok(_) => savepoint.commit().await?,
        Err(e) => {
            savepoint.rollback().await?;
            return Err(e.into());
        }
    }
    Ok(())
}

/// The idempotency ledger over `jobs.discord_announcements`. Takes the
/// **worker's queue pool** (`worker_svc`, which owns the `jobs` schema).
pub struct PgAnnouncementLedger {
    pool: PgPool,
}

impl PgAnnouncementLedger {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AnnouncementLedger for PgAnnouncementLedger {
    async fn claim(&self, key: &str, stale_after_secs: i64) -> anyhow::Result<Claim> {
        // Insert the claim, or take over one left unsettled past the stale window.
        let won = sqlx::query(
            "INSERT INTO jobs.discord_announcements (dedup_key, status) VALUES ($1, 'claimed') \
             ON CONFLICT (dedup_key) DO UPDATE SET claimed_at = now() \
             WHERE jobs.discord_announcements.status = 'claimed' \
               AND jobs.discord_announcements.claimed_at \
                   < now() - make_interval(secs => $2::double precision) \
             RETURNING dedup_key",
        )
        .bind(key)
        .bind(stale_after_secs as f64)
        .fetch_optional(&self.pool)
        .await?;
        if won.is_some() {
            return Ok(Claim::Claimed);
        }
        let status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM jobs.discord_announcements WHERE dedup_key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(match status.as_deref() {
            Some("posted" | "failed") => Claim::Settled,
            // Still claimed by another attempt, or released in between: retry.
            _ => Claim::InFlight,
        })
    }

    async fn mark_posted(&self, key: &str) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE jobs.discord_announcements \
             SET status = 'posted', settled_at = now(), reason = NULL WHERE dedup_key = $1",
        )
        .bind(key)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_failed(&self, key: &str, reason: &str) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE jobs.discord_announcements \
             SET status = 'failed', settled_at = now(), reason = $2 WHERE dedup_key = $1",
        )
        .bind(key)
        .bind(reason)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn release(&self, key: &str) -> anyhow::Result<()> {
        sqlx::query(
            "DELETE FROM jobs.discord_announcements WHERE dedup_key = $1 AND status = 'claimed'",
        )
        .bind(key)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

/// Reads accepted catalog items from the `music` schema. Takes the worker's
/// **`admin_svc` pool** — the worker-only cross-schema reader; names are
/// schema-qualified so they do not depend on its `search_path`.
pub struct PgAnnouncementSource {
    pool: PgPool,
}

impl PgAnnouncementSource {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AnnouncementSource for PgAnnouncementSource {
    async fn accepted_soundfont(&self, id: &str) -> anyhow::Result<Option<SoundFontCard>> {
        let row = sqlx::query(
            "SELECT label, instrument, license, attribution FROM music.soundfonts \
             WHERE id = $1 AND moderation_status = 'accepted'",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| SoundFontCard {
            label: r.get("label"),
            instrument: r.get("instrument"),
            license: r.get("license"),
            attribution: r.get("attribution"),
        }))
    }

    async fn accepted_score(&self, id: &str) -> anyhow::Result<Option<ScoreCard>> {
        let Ok(id) = uuid::Uuid::parse_str(id) else {
            return Ok(None);
        };
        let row = sqlx::query(
            "SELECT title, composer FROM music.catalog_scores \
             WHERE id = $1 AND moderation_status = 'accepted'",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| ScoreCard {
            title: r.get("title"),
            composer: r.get("composer"),
        }))
    }
}
