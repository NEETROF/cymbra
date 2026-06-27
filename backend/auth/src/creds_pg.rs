//! Postgres-backed [`CredentialRepo`] (task 4.2) — thin I/O glue (integration
//! tested in group 7).

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::creds::{Credential, CredentialRepo};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("auth db: {e}"))
}

pub struct PgCredentialRepo {
    pool: PgPool,
}

impl PgCredentialRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CredentialRepo for PgCredentialRepo {
    async fn insert(&self, email: &str, password_hash: &str) -> Result<()> {
        let res =
            sqlx::query("INSERT INTO local_credentials (email, password_hash) VALUES ($1, $2)")
                .bind(email)
                .bind(password_hash)
                .execute(&self.pool)
                .await;
        match res {
            Ok(_) => Ok(()),
            Err(e)
                if e.as_database_error()
                    .map(|d| d.is_unique_violation())
                    .unwrap_or(false) =>
            {
                Err(AppError::AlreadyExists("email already registered".into()))
            }
            Err(e) => Err(internal(e)),
        }
    }

    async fn get(&self, email: &str) -> Result<Option<Credential>> {
        let row = sqlx::query(
            "SELECT password_hash, email_verified FROM local_credentials WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        Ok(row.map(|r| Credential {
            email: email.to_string(),
            password_hash: r.get("password_hash"),
            email_verified: r.get("email_verified"),
        }))
    }

    async fn set_verification(&self, email: &str, token: &str, expires_at: i64) -> Result<()> {
        sqlx::query(
            "UPDATE local_credentials SET verification_token = $2, verification_expires_at = $3 \
             WHERE email = $1",
        )
        .bind(email)
        .bind(token)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn verify_by_token(&self, token: &str, now: i64) -> Result<Option<String>> {
        let row = sqlx::query(
            "UPDATE local_credentials SET email_verified = true, verification_token = NULL, \
             verification_expires_at = NULL WHERE verification_token = $1 \
             AND verification_expires_at > $2 RETURNING email",
        )
        .bind(token)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        Ok(row.map(|r| r.get::<String, _>("email")))
    }

    async fn set_reset(&self, email: &str, token: &str, expires_at: i64) -> Result<()> {
        sqlx::query(
            "UPDATE local_credentials SET reset_token = $2, reset_expires_at = $3 WHERE email = $1",
        )
        .bind(email)
        .bind(token)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn reset_by_token(
        &self,
        token: &str,
        new_hash: &str,
        now: i64,
    ) -> Result<Option<String>> {
        let row = sqlx::query(
            "UPDATE local_credentials SET password_hash = $2, reset_token = NULL, \
             reset_expires_at = NULL WHERE reset_token = $1 AND reset_expires_at > $3 \
             RETURNING email",
        )
        .bind(token)
        .bind(new_hash)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        Ok(row.map(|r| r.get::<String, _>("email")))
    }
}
