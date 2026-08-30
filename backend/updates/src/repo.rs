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

//! The release store (change: add-desktop-auto-update, task 2.2).
//!
//! A trait seam so the HTTP handlers are testable with `mockall` mocks and never
//! need Postgres, plus the thin `app_updates.releases` adapter. The adapter is
//! coverage-excluded like the other `pg*` adapters; the decision logic it drives
//! lives in [`crate::core`].

use crate::Release;

/// Read/write access to the stored releases.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait::async_trait]
pub trait ReleaseRepo: Send + Sync {
    /// The highest servable release for a product/channel — not paused, rollout
    /// above zero. `None` means the endpoint answers `204`.
    async fn servable(&self, product: &str, channel: &str) -> anyhow::Result<Option<Release>>;

    /// Store (or replace) a release, keyed on `(product, channel, version)`.
    /// Re-ingesting the same version replaces its bytes and policy — that is how
    /// a rollout percentage is raised.
    async fn upsert(&self, release: &Release) -> anyhow::Result<()>;
}

/// Postgres adapter over `app_updates.releases`.
pub struct PgReleaseRepo {
    pool: sqlx::PgPool,
}

impl PgReleaseRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ReleaseRepo for PgReleaseRepo {
    async fn servable(&self, product: &str, channel: &str) -> anyhow::Result<Option<Release>> {
        let row: Option<Release> = sqlx::query_as(
            "SELECT product, channel, version, version_order, manifest, signature, key_id, \
                    rollout_percent, paused \
               FROM app_updates.releases \
              WHERE product = $1 AND channel = $2 AND NOT paused AND rollout_percent > 0 \
              ORDER BY version_order DESC \
              LIMIT 1",
        )
        .bind(product)
        .bind(channel)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn upsert(&self, release: &Release) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO app_updates.releases \
                 (product, channel, version, version_order, manifest, signature, key_id, \
                  rollout_percent, paused) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (product, channel, version) DO UPDATE SET \
                 version_order   = EXCLUDED.version_order, \
                 manifest        = EXCLUDED.manifest, \
                 signature       = EXCLUDED.signature, \
                 key_id          = EXCLUDED.key_id, \
                 rollout_percent = EXCLUDED.rollout_percent, \
                 paused          = EXCLUDED.paused, \
                 updated_at      = now()",
        )
        .bind(&release.product)
        .bind(&release.channel)
        .bind(&release.version)
        .bind(release.version_order)
        .bind(&release.manifest)
        .bind(&release.signature)
        .bind(&release.key_id)
        .bind(release.rollout_percent)
        .bind(release.paused)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
