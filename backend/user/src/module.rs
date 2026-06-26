//! The user module's **direct adapter** (task 3.4): implements [`UserPort`] with
//! the account invariants on top of a [`UserRepo`].

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use cymbra_user_port::{Account, Identity, UserPort};

use crate::repo::UserRepo;

/// In-process implementation of the user port over any [`UserRepo`].
pub struct UserModule<R: UserRepo> {
    repo: R,
}

impl<R: UserRepo> UserModule<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R: UserRepo> UserPort for UserModule<R> {
    async fn resolve_or_provision(&self, provider: &str, subject: &str) -> Result<String> {
        match self.repo.identity_owner(provider, subject).await? {
            Some(uid) => Ok(uid),
            None => self.repo.create_account(provider, subject).await,
        }
    }

    async fn link_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()> {
        match self.repo.identity_owner(provider, subject).await? {
            Some(owner) if owner == user_id => Ok(()), // already linked to this account
            Some(_) => Err(AppError::AlreadyExists(
                "identity already linked to another account".into(),
            )),
            None => self.repo.add_identity(user_id, provider, subject).await,
        }
    }

    async fn unlink_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()> {
        if self.repo.count_identities(user_id).await? <= 1 {
            return Err(AppError::FailedPrecondition(
                "cannot unlink the last identity".into(),
            ));
        }
        match self.repo.identity_owner(provider, subject).await? {
            Some(owner) if owner == user_id => {
                self.repo.remove_identity(user_id, provider, subject).await
            }
            _ => Err(AppError::NotFound("identity".into())),
        }
    }

    async fn list_identities(&self, user_id: &str) -> Result<Vec<Identity>> {
        self.repo.list_identities(user_id).await
    }

    async fn get_account(&self, user_id: &str) -> Result<Account> {
        self.repo.get_account(user_id).await
    }

    async fn update_account(
        &self,
        user_id: &str,
        display_name: Option<String>,
        preferences: &str,
        expected_version: i64,
    ) -> Result<Account> {
        self.repo
            .update_account(user_id, display_name, preferences, expected_version)
            .await
    }

    async fn delete_account(&self, user_id: &str) -> Result<()> {
        self.repo.delete_account(user_id).await
    }

    async fn effective_roles(&self, user_id: &str, scope: &str) -> Result<Vec<String>> {
        self.repo.roles_for_scope(user_id, &["global", scope]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::FakeUserRepo;

    fn module() -> UserModule<FakeUserRepo> {
        UserModule::new(FakeUserRepo::default())
    }

    #[tokio::test]
    async fn provisions_with_default_role_then_reuses() {
        let m = module();
        let uid = m.resolve_or_provision("google", "sub-1").await.unwrap();
        // default (global, user)
        assert_eq!(
            m.effective_roles(&uid, "music").await.unwrap(),
            vec!["user"]
        );
        // same identity resolves to the same account
        let again = m.resolve_or_provision("google", "sub-1").await.unwrap();
        assert_eq!(uid, again);
    }

    #[tokio::test]
    async fn link_attaches_and_rejects_bound_elsewhere() {
        let m = module();
        let a = m.resolve_or_provision("google", "g1").await.unwrap();
        let b = m.resolve_or_provision("google", "g2").await.unwrap();

        m.link_identity(&a, "local", "a@x.dev").await.unwrap();
        assert_eq!(m.list_identities(&a).await.unwrap().len(), 2);

        // b cannot claim a's local identity
        assert!(matches!(
            m.link_identity(&b, "local", "a@x.dev").await,
            Err(AppError::AlreadyExists(_))
        ));
    }

    #[tokio::test]
    async fn unlink_guards_last_identity() {
        let m = module();
        let a = m.resolve_or_provision("google", "g1").await.unwrap();
        // only one identity -> cannot unlink
        assert!(matches!(
            m.unlink_identity(&a, "google", "g1").await,
            Err(AppError::FailedPrecondition(_))
        ));
        // add a second, then unlink the first is allowed
        m.link_identity(&a, "apple", "ap1").await.unwrap();
        m.unlink_identity(&a, "google", "g1").await.unwrap();
        assert_eq!(m.list_identities(&a).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn delete_erases_account_and_roles() {
        let m = module();
        let a = m.resolve_or_provision("google", "g1").await.unwrap();
        m.delete_account(&a).await.unwrap();
        assert!(matches!(
            m.get_account(&a).await,
            Err(AppError::NotFound(_))
        ));
        assert!(m.effective_roles(&a, "music").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn roles_are_scoped_per_app() {
        let m = module();
        let a = m.resolve_or_provision("google", "g1").await.unwrap();
        m.repo.grant_role(&a, "live", "broadcaster").await.unwrap();
        let live = m.effective_roles(&a, "live").await.unwrap();
        assert!(live.contains(&"broadcaster".to_string()));
        assert!(live.contains(&"user".to_string()));
        let music = m.effective_roles(&a, "music").await.unwrap();
        assert!(!music.contains(&"broadcaster".to_string())); // other app's scope excluded
    }

    #[tokio::test]
    async fn update_uses_optimistic_concurrency() {
        let m = module();
        let a = m.resolve_or_provision("google", "g1").await.unwrap();
        let acc = m
            .update_account(&a, Some("Ada".into()), "{\"theme\":\"dark\"}", 1)
            .await
            .unwrap();
        assert_eq!(acc.version, 2);
        // stale write rejected
        assert!(matches!(
            m.update_account(&a, None, "{}", 1).await,
            Err(AppError::Aborted(_))
        ));
    }
}
