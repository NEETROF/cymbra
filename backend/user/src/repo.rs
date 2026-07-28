//! Data-access port for the user module (task 3.3).
//!
//! [`UserRepo`] is the storage primitive surface; the direct adapter
//! ([`crate::module::UserModule`]) layers the business invariants on top. A
//! [`FakeUserRepo`] lets the module be unit-tested without Postgres.

use async_trait::async_trait;
use chrono::NaiveDate;
use cymbra_platform::{AppError, Result};
use cymbra_user_port::{Account, AccountPage, AccountSummary, Identity, RoleGrant};
use std::collections::HashMap;
use std::sync::Mutex;

/// The profile fields the public-profile read + visibility control need (change:
/// add-play-activity-profile). Internal to the user crate (repo ↔ module).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileRow {
    pub handle: Option<String>,
    pub display_name: Option<String>,
    /// Stored visibility (`private` | `public`).
    pub visibility: String,
    /// Derived eligibility date (`None` until an age has been established).
    pub share_eligible_from: Option<NaiveDate>,
}

#[async_trait]
pub trait UserRepo: Send + Sync {
    /// `user_id` owning `(provider, subject)`, if any.
    async fn identity_owner(&self, provider: &str, subject: &str) -> Result<Option<String>>;
    /// Create a user + its first identity + the default `(global, user)` role.
    async fn create_account(&self, provider: &str, subject: &str) -> Result<String>;
    /// Insert an identity for `user_id` (caller has checked ownership).
    async fn add_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()>;
    /// Remove an identity from `user_id`.
    async fn remove_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()>;
    async fn count_identities(&self, user_id: &str) -> Result<usize>;
    async fn list_identities(&self, user_id: &str) -> Result<Vec<Identity>>;
    async fn get_account(&self, user_id: &str) -> Result<Account>;
    /// `user_id` whose normalized handle key is `handle_key`, if any.
    async fn handle_owner(&self, handle_key: &str) -> Result<Option<String>>;
    /// Conditional update: applies only if the stored version == `expected_version`.
    /// When `handle`/`handle_key` are `Some`, the handle is (re)assigned and the
    /// key's uniqueness is enforced; when `None`, the stored handle is unchanged.
    async fn update_account(
        &self,
        user_id: &str,
        display_name: Option<String>,
        handle: Option<String>,
        handle_key: Option<String>,
        preferences: &str,
        expected_version: i64,
    ) -> Result<Account>;
    async fn delete_account(&self, user_id: &str) -> Result<()>;
    /// Delete every handle-less account created strictly before `cutoff_unix`
    /// (orphans from abandoned onboarding). Returns the number purged.
    async fn delete_orphans_before(&self, cutoff_unix: i64) -> Result<u64>;
    /// Roles whose scope is in `scopes` (e.g. `["global", "live"]`).
    async fn roles_for_scope(&self, user_id: &str, scopes: &[&str]) -> Result<Vec<String>>;
    async fn grant_role(&self, user_id: &str, scope: &str, role: &str) -> Result<()>;
    /// Remove `(user_id, scope, role)` if present (idempotent; change:
    /// add-moderation-back-office).
    async fn revoke_role(&self, user_id: &str, scope: &str, role: &str) -> Result<()>;
    /// Append a role-grant audit entry (change: add-moderation-back-office). The
    /// store stamps the time; `action` is `grant` or `revoke`.
    async fn record_role_grant(
        &self,
        target_user_id: &str,
        scope: &str,
        role: &str,
        action: &str,
        acting_admin: &str,
    ) -> Result<()>;
    /// The audit history for `user_id`, most recent first.
    async fn list_role_grants(&self, user_id: &str) -> Result<Vec<RoleGrant>>;
    /// A page of the account directory (change: add-admin-account-directory):
    /// accounts (with their `music`-scope roles) matching `query` — empty matches
    /// all; otherwise a `handle_key` prefix OR a `local` identity whose email equals
    /// `query` — ordered by handle (nulls last) then creation, plus the total count.
    async fn list_accounts(
        &self,
        query: &str,
        handle_key: &str,
        limit: i64,
        offset: i64,
    ) -> Result<AccountPage>;

    /// Read the profile row (identity + visibility + eligibility) for `user_id`
    /// (change: add-play-activity-profile). `NotFound` when the account is gone.
    async fn profile_row(&self, user_id: &str) -> Result<ProfileRow>;

    /// Set `user_id`'s visibility. When `share_eligible_from` is `Some`, it is
    /// (over)written; when `None`, the stored eligibility date is left unchanged
    /// (COALESCE semantics), so toggling private↔public never loses the derived
    /// date. `NotFound` when the account is gone.
    async fn update_visibility(
        &self,
        user_id: &str,
        visibility: &str,
        share_eligible_from: Option<NaiveDate>,
    ) -> Result<()>;
}

// --- In-memory fake (tests) -------------------------------------------------

struct AccountRow {
    display_name: Option<String>,
    preferences: String,
    version: i64,
    handle: Option<String>,
    handle_key: Option<String>,
    created_at: i64,
    /// Profile visibility (change: add-play-activity-profile); private by default.
    visibility: String,
    share_eligible_from: Option<NaiveDate>,
}

impl Default for AccountRow {
    fn default() -> Self {
        Self {
            display_name: None,
            preferences: String::new(),
            version: 0,
            handle: None,
            handle_key: None,
            created_at: 0,
            // Mirror the migration default: profiles are private until opt-in.
            visibility: "private".into(),
            share_eligible_from: None,
        }
    }
}

#[derive(Default)]
struct State {
    users: HashMap<String, AccountRow>,
    identities: Vec<(String, String, String)>, // (user_id, provider, subject)
    roles: Vec<(String, String, String)>,      // (user_id, scope, role)
    role_grants: Vec<RoleGrant>,               // append-only audit (oldest first)
}

/// In-memory [`UserRepo`] for unit tests (no Postgres; `updated_at` is fixed).
#[derive(Default)]
pub struct FakeUserRepo {
    state: Mutex<State>,
}

impl FakeUserRepo {
    /// Test helper: stamp an account's creation time (the real repo sets it from
    /// `now()` on insert; the fake defaults to 0).
    pub fn set_created_at(&self, user_id: &str, created_at_unix: i64) {
        let mut s = self.state.lock().unwrap();
        if let Some(row) = s.users.get_mut(user_id) {
            row.created_at = created_at_unix;
        }
    }

    fn account(row: &AccountRow, user_id: &str) -> Account {
        Account {
            user_id: user_id.to_string(),
            display_name: row.display_name.clone(),
            preferences: if row.preferences.is_empty() {
                "{}".into()
            } else {
                row.preferences.clone()
            },
            version: row.version,
            updated_at: 0,
            handle: row.handle.clone(),
        }
    }
}

#[async_trait]
impl UserRepo for FakeUserRepo {
    async fn identity_owner(&self, provider: &str, subject: &str) -> Result<Option<String>> {
        let s = self.state.lock().unwrap();
        Ok(s.identities
            .iter()
            .find(|(_, p, sub)| p == provider && sub == subject)
            .map(|(uid, _, _)| uid.clone()))
    }

    async fn create_account(&self, provider: &str, subject: &str) -> Result<String> {
        let mut s = self.state.lock().unwrap();
        let uid = uuid::Uuid::now_v7().to_string();
        s.users.insert(
            uid.clone(),
            AccountRow {
                preferences: "{}".into(),
                version: 1,
                ..Default::default()
            },
        );
        s.identities
            .push((uid.clone(), provider.into(), subject.into()));
        s.roles.push((uid.clone(), "global".into(), "user".into()));
        Ok(uid)
    }

    async fn add_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.identities
            .push((user_id.into(), provider.into(), subject.into()));
        Ok(())
    }

    async fn remove_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.identities
            .retain(|(u, p, sub)| !(u == user_id && p == provider && sub == subject));
        Ok(())
    }

    async fn count_identities(&self, user_id: &str) -> Result<usize> {
        let s = self.state.lock().unwrap();
        Ok(s.identities.iter().filter(|(u, _, _)| u == user_id).count())
    }

    async fn list_identities(&self, user_id: &str) -> Result<Vec<Identity>> {
        let s = self.state.lock().unwrap();
        Ok(s.identities
            .iter()
            .filter(|(u, _, _)| u == user_id)
            .map(|(_, p, sub)| Identity {
                provider: p.clone(),
                subject: sub.clone(),
                linked_at: 0,
            })
            .collect())
    }

    async fn get_account(&self, user_id: &str) -> Result<Account> {
        let s = self.state.lock().unwrap();
        s.users
            .get(user_id)
            .map(|row| Self::account(row, user_id))
            .ok_or_else(|| AppError::NotFound("account".into()))
    }

    async fn handle_owner(&self, handle_key: &str) -> Result<Option<String>> {
        let s = self.state.lock().unwrap();
        Ok(s.users
            .iter()
            .find(|(_, row)| row.handle_key.as_deref() == Some(handle_key))
            .map(|(uid, _)| uid.clone()))
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
        let mut s = self.state.lock().unwrap();
        // Enforce the unique key against other accounts before mutating (mirrors
        // the Postgres unique index).
        if let Some(key) = &handle_key
            && s.users
                .iter()
                .any(|(uid, row)| uid != user_id && row.handle_key.as_deref() == Some(key))
        {
            return Err(AppError::AlreadyExists("handle already taken".into()));
        }
        let row = s
            .users
            .get_mut(user_id)
            .ok_or_else(|| AppError::NotFound("account".into()))?;
        crate::version_core::check(row.version, expected_version)?;
        row.display_name = display_name;
        row.preferences = preferences.to_string();
        // A `None` handle leaves the stored handle untouched (COALESCE semantics).
        if handle.is_some() {
            row.handle = handle;
            row.handle_key = handle_key;
        }
        row.version = crate::version_core::next(row.version);
        Ok(Self::account(row, user_id))
    }

    async fn delete_account(&self, user_id: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.users.remove(user_id);
        s.identities.retain(|(u, _, _)| u != user_id);
        s.roles.retain(|(u, _, _)| u != user_id);
        Ok(())
    }

    async fn delete_orphans_before(&self, cutoff_unix: i64) -> Result<u64> {
        let mut s = self.state.lock().unwrap();
        let victims: Vec<String> = s
            .users
            .iter()
            .filter(|(_, row)| {
                crate::reaper_core::reapable(row.handle.as_deref(), row.created_at, cutoff_unix)
            })
            .map(|(uid, _)| uid.clone())
            .collect();
        for uid in &victims {
            s.users.remove(uid);
            s.identities.retain(|(u, _, _)| u != uid);
            s.roles.retain(|(u, _, _)| u != uid);
        }
        Ok(victims.len() as u64)
    }

    async fn roles_for_scope(&self, user_id: &str, scopes: &[&str]) -> Result<Vec<String>> {
        let s = self.state.lock().unwrap();
        Ok(s.roles
            .iter()
            .filter(|(u, sc, _)| u == user_id && scopes.contains(&sc.as_str()))
            .map(|(_, _, r)| r.clone())
            .collect())
    }

    async fn grant_role(&self, user_id: &str, scope: &str, role: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        let tuple = (user_id.to_string(), scope.to_string(), role.to_string());
        if !s.roles.contains(&tuple) {
            s.roles.push(tuple);
        }
        Ok(())
    }

    async fn revoke_role(&self, user_id: &str, scope: &str, role: &str) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        s.roles
            .retain(|(u, sc, r)| !(u == user_id && sc == scope && r == role));
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
        let mut s = self.state.lock().unwrap();
        // Monotonic `at` from the append position, so `list_role_grants` can return
        // a deterministic most-recent-first order without a clock.
        let at = s.role_grants.len() as i64;
        s.role_grants.push(RoleGrant {
            target_user_id: target_user_id.to_string(),
            scope: scope.to_string(),
            role: role.to_string(),
            action: action.to_string(),
            acting_admin: acting_admin.to_string(),
            at,
            acting_admin_handle: None,
        });
        Ok(())
    }

    async fn list_role_grants(&self, user_id: &str) -> Result<Vec<RoleGrant>> {
        let s = self.state.lock().unwrap();
        Ok(s.role_grants
            .iter()
            .filter(|g| g.target_user_id == user_id)
            .rev() // most recent first
            .map(|g| RoleGrant {
                // Resolve the acting admin's handle at read time (mirrors the SQL join).
                acting_admin_handle: s.users.get(&g.acting_admin).and_then(|r| r.handle.clone()),
                ..g.clone()
            })
            .collect())
    }

    async fn list_accounts(
        &self,
        query: &str,
        handle_key: &str,
        limit: i64,
        offset: i64,
    ) -> Result<AccountPage> {
        let s = self.state.lock().unwrap();
        let email = query.to_lowercase();
        // Filter: empty query = all; else a handle-key prefix OR a `local` identity
        // whose email equals the query (case-insensitive) — mirrors the SQL.
        let mut matched: Vec<(&String, &AccountRow)> =
            s.users
                .iter()
                .filter(|(uid, row)| {
                    if query.is_empty() {
                        return true;
                    }
                    let handle_hit = !handle_key.is_empty()
                        && row
                            .handle_key
                            .as_deref()
                            .map(|k| k.starts_with(handle_key))
                            .unwrap_or(false);
                    let email_hit = s.identities.iter().any(|(u, p, sub)| {
                        u == *uid && p == "local" && sub.to_lowercase() == email
                    });
                    handle_hit || email_hit
                })
                .collect();
        // Order by handle (nulls last, case-insensitive), then creation, then id.
        matched.sort_by(|(ua, a), (ub, b)| {
            match (a.handle.as_deref(), b.handle.as_deref()) {
                (Some(x), Some(y)) => x.to_lowercase().cmp(&y.to_lowercase()),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
            .then(a.created_at.cmp(&b.created_at))
            .then(ua.cmp(ub))
        });
        let total = matched.len() as i64;
        let entries = matched
            .into_iter()
            .skip(offset.max(0) as usize)
            .take(limit.max(0) as usize)
            .map(|(uid, row)| AccountSummary {
                user_id: uid.clone(),
                handle: row.handle.clone(),
                display_name: row.display_name.clone(),
                roles: s
                    .roles
                    .iter()
                    .filter(|(u, sc, _)| u == uid && sc == "music")
                    .map(|(_, _, r)| r.clone())
                    .collect(),
            })
            .collect();
        Ok(AccountPage { entries, total })
    }

    async fn profile_row(&self, user_id: &str) -> Result<ProfileRow> {
        let s = self.state.lock().unwrap();
        s.users
            .get(user_id)
            .map(|row| ProfileRow {
                handle: row.handle.clone(),
                display_name: row.display_name.clone(),
                visibility: row.visibility.clone(),
                share_eligible_from: row.share_eligible_from,
            })
            .ok_or_else(|| AppError::NotFound("account".into()))
    }

    async fn update_visibility(
        &self,
        user_id: &str,
        visibility: &str,
        share_eligible_from: Option<NaiveDate>,
    ) -> Result<()> {
        let mut s = self.state.lock().unwrap();
        let row = s
            .users
            .get_mut(user_id)
            .ok_or_else(|| AppError::NotFound("account".into()))?;
        row.visibility = visibility.to_string();
        // COALESCE: only overwrite the eligibility date when a new one is derived.
        if share_eligible_from.is_some() {
            row.share_eligible_from = share_eligible_from;
        }
        Ok(())
    }
}
