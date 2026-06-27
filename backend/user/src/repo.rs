//! Data-access port for the user module (task 3.3).
//!
//! [`UserRepo`] is the storage primitive surface; the direct adapter
//! ([`crate::module::UserModule`]) layers the business invariants on top. A
//! [`FakeUserRepo`] lets the module be unit-tested without Postgres.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use cymbra_user_port::{Account, Identity};
use std::collections::HashMap;
use std::sync::Mutex;

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
    /// Conditional update: applies only if the stored version == `expected_version`.
    async fn update_account(
        &self,
        user_id: &str,
        display_name: Option<String>,
        preferences: &str,
        expected_version: i64,
    ) -> Result<Account>;
    async fn delete_account(&self, user_id: &str) -> Result<()>;
    /// Roles whose scope is in `scopes` (e.g. `["global", "live"]`).
    async fn roles_for_scope(&self, user_id: &str, scopes: &[&str]) -> Result<Vec<String>>;
    async fn grant_role(&self, user_id: &str, scope: &str, role: &str) -> Result<()>;
}

// --- In-memory fake (tests) -------------------------------------------------

#[derive(Default)]
struct AccountRow {
    display_name: Option<String>,
    preferences: String,
    version: i64,
}

#[derive(Default)]
struct State {
    users: HashMap<String, AccountRow>,
    identities: Vec<(String, String, String)>, // (user_id, provider, subject)
    roles: Vec<(String, String, String)>,      // (user_id, scope, role)
}

/// In-memory [`UserRepo`] for unit tests (no Postgres; `updated_at` is fixed).
#[derive(Default)]
pub struct FakeUserRepo {
    state: Mutex<State>,
}

impl FakeUserRepo {
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
                display_name: None,
                preferences: "{}".into(),
                version: 1,
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

    async fn update_account(
        &self,
        user_id: &str,
        display_name: Option<String>,
        preferences: &str,
        expected_version: i64,
    ) -> Result<Account> {
        let mut s = self.state.lock().unwrap();
        let row = s
            .users
            .get_mut(user_id)
            .ok_or_else(|| AppError::NotFound("account".into()))?;
        crate::version_core::check(row.version, expected_version)?;
        row.display_name = display_name;
        row.preferences = preferences.to_string();
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
}
