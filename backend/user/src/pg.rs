//! Postgres-backed [`UserRepo`] (task 3.3) — thin I/O glue (excluded from the
//! coverage gate; exercised by the integration tests in group 7).

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use cymbra_user_port::{Account, AccountPage, AccountSummary, Identity, RoleGrant, ScopeRoles};
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
            "SELECT display_name, handle, locale, preferences::text AS preferences, version, \
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
            locale: row.get("locale"),
        })
    }

    async fn set_locale(&self, user_id: &str, locale: &str) -> Result<()> {
        let res = sqlx::query("UPDATE users SET locale = $2, updated_at = now() WHERE id = $1")
            .bind(parse_uuid(user_id)?)
            .bind(locale)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("account".into()));
        }
        Ok(())
    }

    async fn locale(&self, user_id: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT locale FROM users WHERE id = $1")
            .bind(parse_uuid(user_id)?)
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        Ok(row.and_then(|r| r.get::<Option<String>, _>("locale")))
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
             RETURNING display_name, handle, locale, preferences::text AS preferences, version, \
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
                locale: row.get("locale"),
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

    async fn delete_orphans_before(&self, cutoff_unix: i64) -> Result<u64> {
        // Mirrors reaper_core::reapable: no handle AND created before the cutoff.
        let res =
            sqlx::query("DELETE FROM users WHERE handle IS NULL AND created_at < to_timestamp($1)")
                .bind(cutoff_unix)
                .execute(&self.pool)
                .await
                .map_err(internal)?;
        Ok(res.rows_affected())
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

    async fn roles_by_scope(
        &self,
        user_id: &str,
        scopes: &[String],
    ) -> Result<Vec<(String, String)>> {
        let scope_vec: Vec<String> = scopes.to_vec();
        let rows = sqlx::query(
            "SELECT scope, role FROM user_roles WHERE user_id = $1 AND scope = ANY($2)",
        )
        .bind(parse_uuid(user_id)?)
        .bind(&scope_vec)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| (r.get::<String, _>("scope"), r.get::<String, _>("role")))
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

    async fn revoke_role(&self, user_id: &str, scope: &str, role: &str) -> Result<()> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND scope = $2 AND role = $3")
            .bind(parse_uuid(user_id)?)
            .bind(scope)
            .bind(role)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }

    async fn record_role_grant(
        &self,
        target_user_id: &str,
        scope: &str,
        role: &str,
        action: &str,
        acting_admin: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO role_grants (target_user_id, scope, role, action, acting_admin) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(parse_uuid(target_user_id)?)
        .bind(scope)
        .bind(role)
        .bind(action)
        .bind(parse_uuid(acting_admin)?)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn list_role_grants(&self, user_id: &str) -> Result<Vec<RoleGrant>> {
        let rows = sqlx::query(
            "SELECT rg.target_user_id, rg.scope, rg.role, rg.action, rg.acting_admin, \
                    u.handle AS acting_admin_handle, \
                    EXTRACT(EPOCH FROM rg.created_at)::bigint AS at \
             FROM role_grants rg \
             LEFT JOIN users u ON u.id = rg.acting_admin \
             WHERE rg.target_user_id = $1 \
             ORDER BY rg.created_at DESC, rg.id DESC",
        )
        .bind(parse_uuid(user_id)?)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| RoleGrant {
                target_user_id: r.get::<uuid::Uuid, _>("target_user_id").to_string(),
                scope: r.get("scope"),
                role: r.get("role"),
                action: r.get("action"),
                acting_admin: r.get::<uuid::Uuid, _>("acting_admin").to_string(),
                at: r.get("at"),
                acting_admin_handle: r.get("acting_admin_handle"),
            })
            .collect())
    }

    async fn list_accounts(
        &self,
        query: &str,
        handle_key: &str,
        limit: i64,
        offset: i64,
        scopes: &[String],
    ) -> Result<AccountPage> {
        // Filter predicate (shared by count + page): empty query = all; else a
        // handle-key prefix OR a `local` identity email equal to the query.
        const WHERE: &str = "($1 = '' \
            OR ($2 <> '' AND u.handle_key LIKE $2 || '%') \
            OR EXISTS (SELECT 1 FROM user_identities i \
                       WHERE i.user_id = u.id AND i.provider = 'local' \
                         AND lower(i.subject) = lower($1)))";

        let total: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM users u WHERE {WHERE}"))
            .bind(query)
            .bind(handle_key)
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;

        // Page the matching accounts first (identity only) …
        let rows = sqlx::query(&format!(
            "SELECT u.id, u.handle, u.display_name \
             FROM users u \
             WHERE {WHERE} \
             ORDER BY u.handle ASC NULLS LAST, u.created_at, u.id \
             LIMIT $3 OFFSET $4"
        ))
        .bind(query)
        .bind(handle_key)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;

        let ids: Vec<uuid::Uuid> = rows.iter().map(|r| r.get::<uuid::Uuid, _>("id")).collect();
        let scope_vec: Vec<String> = scopes.to_vec();

        // … then fetch just this page's roles, restricted to the authorized scopes,
        // and group them per (user, scope) in memory. Only the caller's authorized
        // scopes are queried, so the directory can never leak another scope's roles.
        use std::collections::HashMap;
        let mut roles_map: HashMap<uuid::Uuid, HashMap<String, Vec<String>>> = HashMap::new();
        if !ids.is_empty() && !scope_vec.is_empty() {
            let role_rows = sqlx::query(
                "SELECT user_id, scope, role FROM user_roles \
                 WHERE user_id = ANY($1) AND scope = ANY($2)",
            )
            .bind(&ids)
            .bind(&scope_vec)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
            for r in role_rows {
                let uid: uuid::Uuid = r.get("user_id");
                let scope: String = r.get("scope");
                let role: String = r.get("role");
                roles_map
                    .entry(uid)
                    .or_default()
                    .entry(scope)
                    .or_default()
                    .push(role);
            }
        }

        let entries = rows
            .into_iter()
            .map(|r| {
                let uid: uuid::Uuid = r.get("id");
                let per_scope = roles_map.remove(&uid).unwrap_or_default();
                // Emit an entry for every authorized scope (empty when no role), in
                // the caller's scope order, so the UI renders columns consistently.
                let roles_by_scope = scopes
                    .iter()
                    .map(|sc| ScopeRoles {
                        scope: sc.clone(),
                        roles: per_scope.get(sc).cloned().unwrap_or_default(),
                    })
                    .collect();
                AccountSummary {
                    user_id: uid.to_string(),
                    handle: r.get("handle"),
                    display_name: r.get("display_name"),
                    roles_by_scope,
                }
            })
            .collect();
        Ok(AccountPage { entries, total })
    }

    async fn profile_row(&self, user_id: &str) -> Result<crate::repo::ProfileRow> {
        let row = sqlx::query(
            "SELECT handle, display_name, profile_visibility, share_eligible_from \
             FROM users WHERE id = $1",
        )
        .bind(parse_uuid(user_id)?)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?
        .ok_or_else(|| AppError::NotFound("account".into()))?;
        Ok(crate::repo::ProfileRow {
            handle: row.get("handle"),
            display_name: row.get("display_name"),
            visibility: row.get("profile_visibility"),
            share_eligible_from: row.get("share_eligible_from"),
        })
    }

    async fn update_visibility(
        &self,
        user_id: &str,
        visibility: &str,
        share_eligible_from: Option<chrono::NaiveDate>,
    ) -> Result<()> {
        // COALESCE keeps the stored eligibility date when the caller passes `None`
        // (a private toggle never loses the derived date).
        let res = sqlx::query(
            "UPDATE users SET profile_visibility = $2, \
             share_eligible_from = COALESCE($3, share_eligible_from) WHERE id = $1",
        )
        .bind(parse_uuid(user_id)?)
        .bind(visibility)
        .bind(share_eligible_from)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("account".into()));
        }
        Ok(())
    }
}

fn parse_uuid(s: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(s).map_err(|_| AppError::InvalidArgument("invalid user id".into()))
}
