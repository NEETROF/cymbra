//! Postgres connection-pool factory (task 2.3).
//!
//! Each module builds its **own** pool from its own role's URL, so its queries
//! are confined to its schema by Postgres privileges (design D0). Migrations are
//! owned by each module crate (via `sqlx::migrate!`).

use crate::error::{AppError, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};

/// Connect a pooled Postgres client for one module's role.
pub async fn connect(url: &str, max_connections: u32) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(url)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("db connect failed: {e}")))
}

/// Lightweight readiness probe used by the health check (task 5.3).
pub async fn ping(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1").execute(pool).await.is_ok()
}
