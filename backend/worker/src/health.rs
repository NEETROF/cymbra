//! Worker health surface (task 3.3). Pure readiness logic + a small Axum router
//! (`/healthz` liveness, `/readyz` readiness — pings the queue DB).

use axum::http::StatusCode;
use axum::{Router, extract::State, routing::get};
use cymbra_platform::db;
use sqlx::PgPool;

/// Ready only when the queue database is reachable.
pub fn ready(db_ok: bool) -> bool {
    db_ok
}

#[derive(Clone)]
struct HealthState {
    pool: PgPool,
}

/// `/healthz` (liveness) + `/readyz` (readiness — pings Postgres).
pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(readyz))
        .with_state(HealthState { pool })
}

async fn readyz(State(s): State<HealthState>) -> StatusCode {
    if ready(db::ping(&s.pool).await) {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_requires_db() {
        assert!(ready(true));
        assert!(!ready(false));
    }
}
