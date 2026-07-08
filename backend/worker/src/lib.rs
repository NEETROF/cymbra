//! Library surface of `cymbra-worker` — the testable job logic the binary
//! (`main.rs`) wires into the sqlxmq runner. Thin DB I/O over live Postgres, so
//! it is exercised by the `#[ignore]` integration tests in `tests/` (and
//! coverage-excluded like the other I/O glue).

use sqlx::PgPool;

/// Completely erase a deleted user's data across the `user_account` and `auth`
/// schemas in ONE `admin_svc` transaction (change: complete-account-deletion).
///
/// `admin_svc` is the only actor that may write both schemas, which is what
/// makes the erasure **atomic** (all-or-nothing). The steps:
/// 1. Resolve the user's email from the `local` identity (the `local_credentials`
///    primary key). An OIDC-only account has no local identity → `None` → the
///    credentials delete is skipped (OIDC-only-safe).
/// 2. `DELETE` the `auth.local_credentials` row by that email (if any).
/// 3. `DELETE` every `auth.sessions` row for the user (refresh-token erasure).
/// 4. `DELETE` the `user_account.users` row (cascades identities + roles).
///
/// Every delete is a no-op when its rows are already gone, so the whole function
/// is **idempotent**: re-running it for an already-purged user commits nothing to
/// delete and succeeds. Callers (the `purge_user` job handler) get at-least-once
/// delivery, so this idempotency is required.
pub async fn purge_user(admin_pool: &PgPool, user_id: &str) -> anyhow::Result<()> {
    let uid = uuid::Uuid::parse_str(user_id)
        .map_err(|_| anyhow::anyhow!("purge_user: invalid user_id {user_id:?}"))?;

    let mut tx = admin_pool.begin().await?;

    // Resolve the local-identity email (the `local_credentials` PK). OIDC-only
    // accounts have no `local` identity → `None` → skip the credentials delete.
    let email: Option<String> = sqlx::query_scalar(
        "SELECT subject FROM user_account.user_identities \
         WHERE user_id = $1 AND provider = 'local'",
    )
    .bind(uid)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(email) = email {
        sqlx::query("DELETE FROM auth.local_credentials WHERE email = $1")
            .bind(&email)
            .execute(&mut *tx)
            .await?;
    }

    // Refresh-token sessions are keyed by the user_id string (auth.sessions.user_id
    // is TEXT), so bind the raw id, not the parsed UUID.
    sqlx::query("DELETE FROM auth.sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // Finally the account row; the FKs cascade `user_identities` + `user_roles`.
    sqlx::query("DELETE FROM user_account.users WHERE id = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}
