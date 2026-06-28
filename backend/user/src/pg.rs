//! Postgres-backed [`UserRepo`] (task 3.3) — thin I/O glue (excluded from the
//! coverage gate; exercised by the integration tests in group 7).

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use cymbra_user_port::{Account, Identity};
use sqlx::{PgPool, Row};

use crate::repo::UserRepo;

/// Maps a sqlx error to an internal `AppError` (no detail leaked).
fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("user db: {e}"))
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .map(|d| d.is_unique_violation())
        .unwrap_or(false)
}

/// Postgres implementation. The pool uses the `user_svc` role whose `search_path`
/// is pinned to `user_account`, so unqualified tables resolve there.
pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepo for PgUserRepo {
    async fn identity_owner(&self, provider: &str, subject: &str) -> Result<Option<String>> {
        let row =
            sqlx::query("SELECT user_id FROM user_identities WHERE provider = $1 AND subject = $2")
                .bind(provider)
                .bind(subject)
                .fetch_optional(&self.pool)
                .await
                .map_err(internal)?;
        Ok(row.map(|r| r.get::<uuid::Uuid, _>("user_id").to_string()))
    }

    async fn create_account(&self, provider: &str, subject: &str) -> Result<String> {
        let uid = uuid::Uuid::now_v7();
        let iid = uuid::Uuid::now_v7();
        let mut tx = self.pool.begin().await.map_err(internal)?;
        sqlx::query("INSERT INTO users (id) VALUES ($1)")
            .bind(uid)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
        sqlx::query(
            "INSERT INTO user_identities (id, user_id, provider, subject) VALUES ($1, $2, $3, $4)",
        )
        .bind(iid)
        .bind(uid)
        .bind(provider)
        .bind(subject)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
        sqlx::query("INSERT INTO user_roles (user_id, scope, role) VALUES ($1, 'global', 'user')")
            .bind(uid)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
        tx.commit().await.map_err(internal)?;
        Ok(uid.to_string())
    }

    async fn add_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()> {
        let uid = parse_uuid(user_id)?;
        let res = sqlx::query(
            "INSERT INTO user_identities (id, user_id, provider, subject) VALUES ($1, $2, $3, $4)",
        )
        .bind(uuid::Uuid::now_v7())
        .bind(uid)
        .bind(provider)
        .bind(subject)
        .execute(&self.pool)
        .await;
        match res {
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => Err(AppError::AlreadyExists(
                "identity already linked to another account".into(),
            )),
            Err(e) => Err(internal(e)),
        }
    }

    async fn remove_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM user_identities WHERE user_id = $1 AND provider = $2 AND subject = $3",
        )
        .bind(parse_uuid(user_id)?)
        .bind(provider)
        .bind(subject)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn count_identities(&self, user_id: &str) -> Result<usize> {
        let row = sqlx::query("SELECT count(*) AS n FROM user_identities WHERE user_id = $1")
            .bind(parse_uuid(user_id)?)
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;
        Ok(row.get::<i64, _>("n") as usize)
    }

    async fn list_identities(&self, user_id: &str) -> Result<Vec<Identity>> {
        let rows = sqlx::query(
            "SELECT provider, subject, extract(epoch FROM linked_at)::bigint AS linked_at \
             FROM user_identities WHERE user_id = $1 ORDER BY linked_at",
        )
        .bind(parse_uuid(user_id)?)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| Identity {
                provider: r.get("provider"),
                subject: r.get("subject"),
                linked_at: r.get("linked_at"),
            })
            .collect())
    }

    async fn get_account(&self, user_id: &str) -> Result<Account> {
        let row = sqlx::query(
            "SELECT display_name, handle, preferences::text AS preferences, version, \
             extract(epoch FROM updated_at)::bigint AS updated_at FROM users WHERE id = $1",
        )
        .bind(parse_uuid(user_id)?)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?
        .ok_or_else(|| AppError::NotFound("account".into()))?;
        Ok(Account {
            user_id: user_id.to_string(),
            display_name: row.get("display_name"),
            preferences: row.get("preferences"),
            version: row.get("version"),
            updated_at: row.get("updated_at"),
            handle: row.get("handle"),
        })
    }

    async fn handle_owner(&self, handle_key: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT id FROM users WHERE handle_key = $1")
            .bind(handle_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        Ok(row.map(|r| r.get::<uuid::Uuid, _>("id").to_string()))
    }

    async fn update_account(
        &self,
        user_id: &str,
        display_name: Option<String>,
        handle: Option<String>,
        handle_key: Option<String>,
        preferences: &str,
        expected_version: i64,
    ) -> Result<Account> {
        let uid = parse_uuid(user_id)?;
        // COALESCE keeps the stored handle when none is supplied; a non-null
        // handle_key that collides trips the unique index → AlreadyExists.
        let res = sqlx::query(
            "UPDATE users SET display_name = $2, preferences = $3::jsonb, version = version + 1, \
             updated_at = now(), handle = COALESCE($5, handle), \
             handle_key = COALESCE($6, handle_key) WHERE id = $1 AND version = $4 \
             RETURNING display_name, handle, preferences::text AS preferences, version, \
             extract(epoch FROM updated_at)::bigint AS updated_at",
        )
        .bind(uid)
        .bind(&display_name)
        .bind(preferences)
        .bind(expected_version)
        .bind(&handle)
        .bind(&handle_key)
        .fetch_optional(&self.pool)
        .await;

        let updated = match res {
            Ok(row) => row,
            Err(e) if is_unique_violation(&e) => {
                return Err(AppError::AlreadyExists("handle already taken".into()));
            }
            Err(e) => return Err(internal(e)),
        };

        match updated {
            Some(row) => Ok(Account {
                user_id: user_id.to_string(),
                display_name: row.get("display_name"),
                preferences: row.get("preferences"),
                version: row.get("version"),
                updated_at: row.get("updated_at"),
                handle: row.get("handle"),
            }),
            None => {
                // Distinguish a stale write from a missing account.
                let exists = sqlx::query("SELECT version FROM users WHERE id = $1")
                    .bind(uid)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(internal)?;
                match exists {
                    Some(r) => Err(AppError::Aborted(format!(
                        "version conflict: expected {expected_version}, server has {}",
                        r.get::<i64, _>("version")
                    ))),
                    None => Err(AppError::NotFound("account".into())),
                }
            }
        }
    }

    async fn delete_account(&self, user_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(parse_uuid(user_id)?)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }

    async fn roles_for_scope(&self, user_id: &str, scopes: &[&str]) -> Result<Vec<String>> {
        let scope_vec: Vec<String> = scopes.iter().map(|s| s.to_string()).collect();
        let rows =
            sqlx::query("SELECT role FROM user_roles WHERE user_id = $1 AND scope = ANY($2)")
                .bind(parse_uuid(user_id)?)
                .bind(&scope_vec)
                .fetch_all(&self.pool)
                .await
                .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>("role"))
            .collect())
    }

    async fn grant_role(&self, user_id: &str, scope: &str, role: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO user_roles (user_id, scope, role) VALUES ($1, $2, $3) \
             ON CONFLICT DO NOTHING",
        )
        .bind(parse_uuid(user_id)?)
        .bind(scope)
        .bind(role)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }
}

fn parse_uuid(s: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(s).map_err(|_| AppError::InvalidArgument("invalid user id".into()))
}
