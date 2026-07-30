//! The override store seam (the DB source of truth) + its Postgres adapter.
//!
//! The store holds only **overrides** — declared keys an admin has changed. The
//! service composes them over the code registry. The trait is the seam unit tests
//! mock ([`MockFlagStore`]); [`PgFlagStore`] is the coverage-excluded I/O glue.

use crate::context::RolloutScope;
use crate::value::{FlagValue, ValueType};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cymbra_platform::error::{AppError, Result};

/// A stored override for a declared `(app, key)`.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredOverride {
    pub app: String,
    pub key: String,
    pub value_type: ValueType,
    pub value: FlagValue,
    pub rollout: RolloutScope,
    pub sensitive: bool,
    pub updated_by: String,
    pub updated_at: DateTime<Utc>,
}

/// A row of the change audit.
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeRecord {
    pub app: String,
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub actor: String,
    pub at: DateTime<Utc>,
}

/// A write of a single override, carried with the actor + the prior display value
/// so the store can upsert the override and append the audit atomically.
#[derive(Debug, Clone)]
pub struct OverrideWrite {
    pub app: String,
    pub key: String,
    pub value_type: ValueType,
    pub value: FlagValue,
    pub rollout: RolloutScope,
    pub sensitive: bool,
    pub actor: String,
    /// Display of the previous override, or `None` when there was none.
    pub old_display: Option<String>,
}

/// The override store: load-all for the snapshot, plus audited upsert/clear and an
/// audit read.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait FlagStore: Send + Sync {
    /// Load every override (the whole set is tiny — dozens of keys).
    async fn load_all(&self) -> Result<Vec<StoredOverride>>;
    /// Upsert an override and append its change audit in one transaction.
    async fn upsert(&self, write: &OverrideWrite) -> Result<()>;
    /// Remove an override (revert to code default) and append the audit.
    async fn clear(&self, app: &str, key: &str, actor: &str, old_display: &str) -> Result<()>;
    /// Recent changes, newest first. An empty `app_filter`/`key_filter` means "any".
    async fn recent_changes(
        &self,
        app_filter: &str,
        key_filter: &str,
        limit: i64,
    ) -> Result<Vec<ChangeRecord>>;
}

fn parse_uuid(s: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(s).map_err(|_| AppError::InvalidArgument(format!("invalid uuid: {s}")))
}

/// Postgres-backed [`FlagStore`] over the `feature_flags` schema (role
/// `flags_svc`). Values are stored as `jsonb` but bound/read as text (via
/// `$n::jsonb` / `value::text`) so no extra sqlx feature is needed.
#[derive(Clone)]
pub struct PgFlagStore {
    pool: sqlx::PgPool,
}

impl PgFlagStore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FlagStore for PgFlagStore {
    async fn load_all(&self) -> Result<Vec<StoredOverride>> {
        use sqlx::Row;
        let rows = sqlx::query(
            "SELECT app, key, value_type, value::text AS value, rollout_scope, \
             sensitive, updated_by, updated_at FROM flag_overrides",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("load flags: {e}")))?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let vt = ValueType::parse(row.get::<String, _>("value_type").as_str())
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("bad value_type in store")))?;
            let raw: String = row.get("value");
            let json: serde_json::Value = serde_json::from_str(&raw)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("bad json in store: {e}")))?;
            let rollout = RolloutScope::parse(row.get::<String, _>("rollout_scope").as_str())
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("bad rollout_scope in store")))?;
            out.push(StoredOverride {
                app: row.get("app"),
                key: row.get("key"),
                value_type: vt,
                value: FlagValue::from_json(vt, &json)?,
                rollout,
                sensitive: row.get("sensitive"),
                updated_by: row.get::<uuid::Uuid, _>("updated_by").to_string(),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(out)
    }

    async fn upsert(&self, w: &OverrideWrite) -> Result<()> {
        let actor = parse_uuid(&w.actor)?;
        let json = serde_json::to_string(&w.value.to_json())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("encode value: {e}")))?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("begin: {e}")))?;
        sqlx::query(
            "INSERT INTO flag_overrides \
               (app, key, value_type, value, rollout_scope, sensitive, updated_by, updated_at) \
             VALUES ($1, $2, $3, $4::jsonb, $5, $6, $7, now()) \
             ON CONFLICT (app, key) DO UPDATE SET \
               value_type = EXCLUDED.value_type, value = EXCLUDED.value, \
               rollout_scope = EXCLUDED.rollout_scope, sensitive = EXCLUDED.sensitive, \
               updated_by = EXCLUDED.updated_by, updated_at = now()",
        )
        .bind(&w.app)
        .bind(&w.key)
        .bind(w.value_type.as_str())
        .bind(&json)
        .bind(w.rollout.as_str())
        .bind(w.sensitive)
        .bind(actor)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("upsert override: {e}")))?;
        sqlx::query(
            "INSERT INTO feature_flag_changes (app, key, old_value, new_value, actor) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&w.app)
        .bind(&w.key)
        .bind(w.old_display.as_deref())
        .bind(w.value.display())
        .bind(actor)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("audit: {e}")))?;
        tx.commit()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("commit: {e}")))
    }

    async fn clear(&self, app: &str, key: &str, actor: &str, old_display: &str) -> Result<()> {
        let actor = parse_uuid(actor)?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("begin: {e}")))?;
        sqlx::query("DELETE FROM flag_overrides WHERE app = $1 AND key = $2")
            .bind(app)
            .bind(key)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("clear override: {e}")))?;
        sqlx::query(
            "INSERT INTO feature_flag_changes (app, key, old_value, new_value, actor) \
             VALUES ($1, $2, $3, '(default)', $4)",
        )
        .bind(app)
        .bind(key)
        .bind(old_display)
        .bind(actor)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("audit: {e}")))?;
        tx.commit()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("commit: {e}")))
    }

    async fn recent_changes(
        &self,
        app_filter: &str,
        key_filter: &str,
        limit: i64,
    ) -> Result<Vec<ChangeRecord>> {
        use sqlx::Row;
        let app = (!app_filter.is_empty()).then_some(app_filter);
        let key = (!key_filter.is_empty()).then_some(key_filter);
        let rows = sqlx::query(
            "SELECT app, key, old_value, new_value, actor, at FROM feature_flag_changes \
             WHERE ($1::text IS NULL OR app = $1) AND ($2::text IS NULL OR key = $2) \
             ORDER BY at DESC LIMIT $3",
        )
        .bind(app)
        .bind(key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("list changes: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|row| ChangeRecord {
                app: row.get("app"),
                key: row.get("key"),
                old_value: row.get("old_value"),
                new_value: row.get("new_value"),
                actor: row.get::<uuid::Uuid, _>("actor").to_string(),
                at: row.get("at"),
            })
            .collect())
    }
}
