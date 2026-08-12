//! Postgres-backed [`PushRegistry`] (change: add-push-notifications, task 1.2) —
//! thin I/O glue (excluded from the coverage gate; the behaviour it serves is
//! covered through the mocked trait).
//!
//! Every statement uses **fully-qualified** `user_account.*` names so the same
//! repo works from the server's `user_svc` pool (search_path pinned to
//! `user_account`) and from the worker's `admin_svc` pool (which is not).

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::repo::{Audience, Candidate, CategoryPref, Platform, PushRegistry};

/// Maps a sqlx error to an internal `AppError` (no detail leaked).
fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("notifications db: {e}"))
}

fn parse_uuid(id: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(id).map_err(|_| AppError::InvalidArgument("invalid user id".into()))
}

/// Postgres implementation over the push tables in the `user_account` schema.
pub struct PgPushRegistry {
    pool: PgPool,
}

impl PgPushRegistry {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PushRegistry for PgPushRegistry {
    async fn register_token(&self, user_id: &str, token: &str, platform: Platform) -> Result<()> {
        // `token` is the PK: the conflict path re-points the install at the
        // current signed-in user and refreshes last-seen.
        sqlx::query(
            "INSERT INTO user_account.push_tokens (token, user_id, platform) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (token) DO UPDATE \
               SET user_id = EXCLUDED.user_id, \
                   platform = EXCLUDED.platform, \
                   last_seen_at = now()",
        )
        .bind(token)
        .bind(parse_uuid(user_id)?)
        .bind(platform.as_str())
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn unregister_token(&self, user_id: &str, token: &str) -> Result<()> {
        // Owner-scoped: a caller can only drop its own device's token.
        sqlx::query("DELETE FROM user_account.push_tokens WHERE token = $1 AND user_id = $2")
            .bind(token)
            .bind(parse_uuid(user_id)?)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }

    async fn prune_token(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM user_account.push_tokens WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }

    async fn has_device(&self, user_id: &str) -> Result<bool> {
        let row = sqlx::query("SELECT 1 FROM user_account.push_tokens WHERE user_id = $1 LIMIT 1")
            .bind(parse_uuid(user_id)?)
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        Ok(row.is_some())
    }

    async fn set_pref(&self, user_id: &str, category: &str, enabled: bool) -> Result<()> {
        sqlx::query(
            "INSERT INTO user_account.notification_prefs (user_id, category, enabled) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (user_id, category) DO UPDATE \
               SET enabled = EXCLUDED.enabled, updated_at = now()",
        )
        .bind(parse_uuid(user_id)?)
        .bind(category)
        .bind(enabled)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn prefs(&self, user_id: &str) -> Result<Vec<CategoryPref>> {
        let rows = sqlx::query(
            "SELECT category, enabled FROM user_account.notification_prefs \
             WHERE user_id = $1 ORDER BY category",
        )
        .bind(parse_uuid(user_id)?)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| CategoryPref {
                category: r.get("category"),
                enabled: r.get("enabled"),
            })
            .collect())
    }

    async fn set_timezone(&self, user_id: &str, timezone: &str) -> Result<()> {
        let res = sqlx::query(
            "UPDATE user_account.users SET timezone = $2, updated_at = now() WHERE id = $1",
        )
        .bind(parse_uuid(user_id)?)
        .bind(timezone)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("account".into()));
        }
        Ok(())
    }

    async fn timezone(&self, user_id: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT timezone FROM user_account.users WHERE id = $1")
            .bind(parse_uuid(user_id)?)
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        Ok(row.and_then(|r| r.get::<Option<String>, _>("timezone")))
    }

    async fn candidates(&self, category: &str, audience: &Audience) -> Result<Vec<Candidate>> {
        // One join per candidate row: the device token, the owner's timezone and
        // the owner's *explicit* choice for this category (NULL when never set —
        // the selection core turns that into the category default).
        let base = "SELECT t.user_id, t.token, t.platform, u.timezone, p.enabled \
             FROM user_account.push_tokens t \
             JOIN user_account.users u ON u.id = t.user_id \
             LEFT JOIN user_account.notification_prefs p \
               ON p.user_id = t.user_id AND p.category = $1";

        let rows = match audience {
            Audience::All => sqlx::query(base)
                .bind(category)
                .fetch_all(&self.pool)
                .await
                .map_err(internal)?,
            Audience::Users(ids) if ids.is_empty() => Vec::new(),
            Audience::Users(ids) => {
                let uuids = ids
                    .iter()
                    .map(|id| parse_uuid(id))
                    .collect::<Result<Vec<_>>>()?;
                let sql = format!("{base} WHERE t.user_id = ANY($2)");
                sqlx::query(&sql)
                    .bind(category)
                    .bind(uuids)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(internal)?
            }
        };

        Ok(rows
            .into_iter()
            .map(|r| Candidate {
                user_id: r.get::<uuid::Uuid, _>("user_id").to_string(),
                token: r.get("token"),
                platform: r.get("platform"),
                timezone: r.get::<Option<String>, _>("timezone"),
                pref: r.get::<Option<bool>, _>("enabled"),
            })
            .collect())
    }
}
