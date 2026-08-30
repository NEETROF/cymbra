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

//! `cymbra-updates` — the desktop update feed (change: add-desktop-auto-update).
//!
//! Owns the `app_updates` schema: CI-signed release manifests, plus the policy
//! that decides **whether and to whom** a release is offered (rollout, pause).
//! It deliberately cannot decide **what** a release is — the manifest bytes and
//! their Ed25519 signature are produced in CI by a key this service never holds,
//! stored verbatim and served verbatim (design D1/D2). Ingest re-verifies the
//! signature via [`cymbra_update_manifest`], so a stolen ingest credential
//! cannot inject an unsigned or foreign-key manifest either.
//!
//! The feed carries no identity: the check sends no version, no install id and
//! no account, so the response is one cacheable document per product/channel and
//! the update check cannot be used to count or track installs (design D3).

pub mod core;
pub mod repo;

pub use core::{is_servable, select_servable, version_order};
#[cfg(any(test, feature = "mock"))]
pub use repo::MockReleaseRepo;
pub use repo::{PgReleaseRepo, ReleaseRepo};

/// The module's Postgres schema.
pub const SCHEMA: &str = "app_updates";

/// Embedded migrations for the `app_updates` schema.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// One stored release. `manifest` and `signature` are opaque: they are what CI
/// signed and what the client verifies, and nothing here ever re-serializes them.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Release {
    pub product: String,
    pub channel: String,
    /// `major.minor.patch+build`, exactly as it appears inside the signed bytes.
    pub version: String,
    /// Sortable projection of `version`; see [`core::version_order`].
    pub version_order: i64,
    /// Base64 of the exact signed manifest bytes.
    pub manifest: String,
    /// Base64 Ed25519 signature over those bytes.
    pub signature: String,
    pub key_id: String,
    /// Staged rollout, 0..=100. `0` is the kill-switch.
    pub rollout_percent: i16,
    pub paused: bool,
}

impl Release {
    /// Rebuild the wire envelope from the stored row.
    ///
    /// Safe to reconstruct because the two fields that are *not* byte-stable
    /// here — `key_id` and `rollout_percent` — sit outside the signature by
    /// design (design D2): the signed payload (`manifest`) and its `signature`
    /// are passed through untouched.
    pub fn to_envelope(&self) -> cymbra_update_manifest::Envelope {
        cymbra_update_manifest::Envelope {
            manifest: self.manifest.clone(),
            signature: self.signature.clone(),
            key_id: self.key_id.clone(),
            rollout_percent: self.rollout_percent.clamp(0, 100) as u8,
        }
    }
}

/// Connects a Postgres pool pinned to `search_path = app_updates`, so the role
/// records the `_sqlx_migrations` ledger in its own schema (mirroring
/// `cymbra_analytics::connect`).
pub async fn connect(database_url: &str, max_connections: u32) -> anyhow::Result<sqlx::PgPool> {
    use sqlx::Executor;
    Ok(sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET search_path = app_updates").await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_passes_the_signed_bytes_through_untouched() {
        let r = Release {
            product: "music".into(),
            channel: "stable".into(),
            version: "1.25.0+34".into(),
            version_order: version_order("1.25.0+34").unwrap(),
            manifest: "bWFuaWZlc3QtYnl0ZXM=".into(),
            signature: "c2lnbmF0dXJl".into(),
            key_id: "2026-08-a".into(),
            rollout_percent: 25,
            paused: false,
        };
        let env = r.to_envelope();
        assert_eq!(env.manifest, r.manifest);
        assert_eq!(env.signature, r.signature);
        assert_eq!(env.key_id, "2026-08-a");
        assert_eq!(env.rollout_percent, 25);
    }

    #[test]
    fn rollout_is_clamped_into_the_wire_range() {
        let base = Release {
            product: "music".into(),
            channel: "stable".into(),
            version: "1.0.0+1".into(),
            version_order: 0,
            manifest: String::new(),
            signature: String::new(),
            key_id: "k".into(),
            rollout_percent: 250,
            paused: false,
        };
        assert_eq!(base.to_envelope().rollout_percent, 100);
        let negative = Release {
            rollout_percent: -3,
            ..base
        };
        assert_eq!(negative.to_envelope().rollout_percent, 0);
    }
}
