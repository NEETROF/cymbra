//! Postgres-backed [`SessionStore`] (change: durable-sessions-postgres) — thin I/O
//! over `auth.sessions`, integration-tested against a live DB. The pure token
//! logic it drives lives in [`crate::session::session_core`].
//!
//! Durability: session families are rows, so a cache (Redis) outage no longer logs
//! anyone out. Rotation is a single **guarded** UPDATE on `(id, current_rt_hash)`
//! so two concurrent refreshes cannot both win; the loser is treated as a replay
//! and revokes the family (theft detection). Expiry is enforced on read
//! (`expires_at > now()`), so correctness never depends on the reap cadence.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};
use std::time::Duration;

use crate::session::{Rotated, SessionInfo, SessionStore, session_core};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("auth session db: {e}"))
}

/// Interval as a Postgres-friendly seconds count for `now() + $ttl`.
fn ttl_secs(ttl: Duration) -> f64 {
    ttl.as_secs() as f64
}

pub struct PgSessionStore {
    pool: PgPool,
    ttl: Duration,
}

impl PgSessionStore {
    pub fn new(pool: PgPool, ttl: Duration) -> Self {
        Self { pool, ttl }
    }
}

#[async_trait]
impl SessionStore for PgSessionStore {
    async fn create(&self, user_id: &str, audience: &str) -> Result<String> {
        let id = session_core::new_id();
        let token = session_core::encode_token(id, &session_core::new_secret());
        let hash = session_core::hash_token(&token);
        sqlx::query(
            "INSERT INTO sessions (id, user_id, audience, current_rt_hash, expires_at) \
             VALUES ($1, $2, $3, $4, now() + make_interval(secs => $5))",
        )
        .bind(id)
        .bind(user_id)
        .bind(audience)
        .bind(&hash)
        .bind(ttl_secs(self.ttl))
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(token)
    }

    async fn rotate(&self, refresh_token: &str) -> Result<Rotated> {
        let id = session_core::parse_id(refresh_token)?;
        let old_hash = session_core::hash_token(refresh_token);
        let new_token = session_core::encode_token(id, &session_core::new_secret());
        let new_hash = session_core::hash_token(&new_token);

        let mut tx = self.pool.begin().await.map_err(internal)?;

        // Atomic check-and-rotate: exactly one concurrent caller matches
        // (id, old_hash) on a live row and slides the expiry.
        let rotated = sqlx::query(
            "UPDATE sessions SET current_rt_hash = $1, \
                 expires_at = now() + make_interval(secs => $2) \
             WHERE id = $3 AND current_rt_hash = $4 AND expires_at > now() \
             RETURNING user_id, audience",
        )
        .bind(&new_hash)
        .bind(ttl_secs(self.ttl))
        .bind(id)
        .bind(&old_hash)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal)?;

        if let Some(row) = rotated {
            tx.commit().await.map_err(internal)?;
            return Ok(Rotated {
                refresh_token: new_token,
                user_id: row.get("user_id"),
                audience: row.get("audience"),
            });
        }

        // No row matched. If the family still exists (and is live), the presented
        // token is a replay of a rotated token → theft: revoke the whole family.
        let live: Option<i32> =
            sqlx::query_scalar("SELECT 1 FROM sessions WHERE id = $1 AND expires_at > now()")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(internal)?;
        if live.is_some() {
            sqlx::query("DELETE FROM sessions WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(internal)?;
            tx.commit().await.map_err(internal)?;
            return Err(AppError::Unauthenticated(
                "refresh token reuse detected".into(),
            ));
        }

        tx.commit().await.map_err(internal)?;
        Err(AppError::Unauthenticated("invalid refresh token".into()))
    }

    async fn revoke(&self, refresh_token: &str) -> Result<()> {
        // Lenient: a malformed token has nothing to revoke.
        if let Ok(id) = session_core::parse_id(refresh_token) {
            sqlx::query("DELETE FROM sessions WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(internal)?;
        }
        Ok(())
    }

    async fn revoke_by_id(&self, user_id: &str, session_id: &str) -> Result<()> {
        // Malformed id → nothing to revoke. The `user_id` guard scopes the delete to
        // the owner, so a foreign id affects zero rows (no-op, no enumeration).
        let Ok(id) = uuid::Uuid::parse_str(session_id) else {
            return Ok(());
        };
        sqlx::query("DELETE FROM sessions WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }

    async fn revoke_all(&self, user_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }

    async fn list_for_user(&self, user_id: &str) -> Result<Vec<SessionInfo>> {
        let rows = sqlx::query(
            "SELECT id, audience FROM sessions WHERE user_id = $1 AND expires_at > now()",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| SessionInfo {
                id: r.get::<uuid::Uuid, _>("id").to_string(),
                audience: r.get("audience"),
            })
            .collect())
    }
}

/// Delete every expired session row (table hygiene). Idempotent; the correctness
/// of expiry is enforced on read, so this only bounds table growth. Run by
/// `cymbra-worker`'s `session_reap` job on an auth-scoped pool.
pub async fn reap_expired_sessions(pool: &PgPool) -> Result<u64> {
    let res = sqlx::query("DELETE FROM sessions WHERE expires_at <= now()")
        .execute(pool)
        .await
        .map_err(internal)?;
    Ok(res.rows_affected())
}
