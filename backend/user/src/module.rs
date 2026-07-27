//! The user module's **direct adapter** (task 3.4): implements [`UserPort`] with
//! the account invariants on top of a [`UserRepo`].

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_jobs::{EnqueueRequest, Enqueuer, PURGE_USER};
use cymbra_platform::{AppError, Result};
use cymbra_user_port::{Account, AccountPage, Identity, UserPort};
use serde::Serialize;

use crate::repo::UserRepo;

/// Recognized role values (change: add-moderation-back-office). `moderator` is
/// added here so a grant of `music/moderator` is accepted and flows into the token
/// via `effective_roles`.
const ROLES: [&str; 3] = ["user", "admin", "moderator"];
/// Recognized authorization scopes (module audiences + the `global` break-glass).
const SCOPES: [&str; 3] = ["global", "music", "live"];
/// Account-directory paging bounds (change: add-admin-account-directory): a `0`/
/// negative request means "default page", capped so a caller can't pull the whole
/// table in one call.
const DEFAULT_PAGE: i64 = 25;
const MAX_PAGE: i64 = 100;

/// Payload for the `purge_user` job: only the user id (the erasure worker resolves
/// the email inside its own transaction, so it never transits the queue).
#[derive(Serialize)]
struct PurgeUserPayload<'a> {
    user_id: &'a str,
}

/// In-process implementation of the user port over any [`UserRepo`].
pub struct UserModule<R: UserRepo> {
    repo: R,
    /// When set, `delete_account` enqueues a `purge_user` job (complete
    /// cross-schema erasure by the worker) instead of the direct `user_account`
    /// delete. Absent in contexts with no queue (unit tests, the worker's own
    /// reaper wiring), where the direct repo delete is used.
    enqueuer: Option<Arc<dyn Enqueuer>>,
}

impl<R: UserRepo> UserModule<R> {
    pub fn new(repo: R) -> Self {
        Self {
            repo,
            enqueuer: None,
        }
    }

    /// Wire the enqueue seam so `delete_account` performs a complete erasure via
    /// the `purge_user` job (change: complete-account-deletion). Used by the
    /// server composition root; the worker keeps the enqueuer unset.
    pub fn with_enqueuer(mut self, enqueuer: Arc<dyn Enqueuer>) -> Self {
        self.enqueuer = Some(enqueuer);
        self
    }

    /// Purge handle-less accounts older than `grace_secs` (orphans left by
    /// abandoned onboarding). `now_unix` is injected so the policy is testable.
    /// Returns the number of accounts purged.
    pub async fn reap_orphans(&self, now_unix: i64, grace_secs: i64) -> Result<u64> {
        let cutoff = crate::reaper_core::cutoff(now_unix, grace_secs);
        self.repo.delete_orphans_before(cutoff).await
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
        handle: Option<String>,
        preferences: &str,
        expected_version: i64,
    ) -> Result<Account> {
        // Validate + normalize the handle here (business invariant); the repo
        // stores the display form and enforces uniqueness on the normalized key.
        let handle_key = match &handle {
            Some(h) => {
                crate::handle_core::validate(h)?;
                Some(crate::handle_core::normalize(h))
            }
            None => None,
        };
        self.repo
            .update_account(
                user_id,
                display_name,
                handle,
                handle_key,
                preferences,
                expected_version,
            )
            .await
    }

    async fn check_handle_availability(&self, handle: &str) -> Result<bool> {
        crate::handle_core::validate(handle)?;
        let key = crate::handle_core::normalize(handle);
        Ok(self.repo.handle_owner(&key).await?.is_none())
    }

    async fn delete_account(&self, user_id: &str) -> Result<()> {
        match &self.enqueuer {
            // Enqueue the complete cross-schema erasure. The `purge_user` job
            // (run by the worker as `admin_svc`) owns the `user_account` delete
            // too, so it all happens in one atomic transaction — we do NOT delete
            // here. Returns once enqueued (async erasure; the RPC stays thin).
            Some(enqueuer) => {
                let spec = cymbra_jobs::spec(PURGE_USER).ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!("purge_user job spec missing from registry"))
                })?;
                let req = EnqueueRequest::for_job(&spec, &PurgeUserPayload { user_id }, None)
                    .map_err(|e| {
                        AppError::Internal(anyhow::anyhow!("build purge_user job: {e}"))
                    })?;
                enqueuer
                    .enqueue(req)
                    .await
                    .map_err(|e| AppError::Internal(anyhow::anyhow!("enqueue purge_user: {e}")))?;
                Ok(())
            }
            // No queue wired (unit tests / non-server contexts): fall back to the
            // direct `user_account` delete.
            None => self.repo.delete_account(user_id).await,
        }
    }

    async fn effective_roles(&self, user_id: &str, scope: &str) -> Result<Vec<String>> {
        self.repo.roles_for_scope(user_id, &["global", scope]).await
    }

    async fn grant_role(
        &self,
        acting_admin: &str,
        user_id: &str,
        scope: &str,
        role: &str,
    ) -> Result<()> {
        validate_scope_role(scope, role)?;
        // Idempotent role write, then the audit entry (change: add-moderation-back-
        // office). Authorization (admin-only) is enforced at the gRPC layer.
        self.repo.grant_role(user_id, scope, role).await?;
        self.repo
            .record_role_grant(user_id, scope, role, "grant", acting_admin)
            .await
    }

    async fn revoke_role(
        &self,
        acting_admin: &str,
        user_id: &str,
        scope: &str,
        role: &str,
    ) -> Result<()> {
        validate_scope_role(scope, role)?;
        self.repo.revoke_role(user_id, scope, role).await?;
        self.repo
            .record_role_grant(user_id, scope, role, "revoke", acting_admin)
            .await
    }

    async fn list_role_grants(&self, user_id: &str) -> Result<Vec<cymbra_user_port::RoleGrant>> {
        self.repo.list_role_grants(user_id).await
    }

    async fn list_accounts(&self, query: &str, limit: i64, offset: i64) -> Result<AccountPage> {
        // Clamp paging (default when unset, capped) so a caller can't drain the
        // table; authorization (admin-only) is enforced at the gRPC layer.
        let limit = if limit <= 0 {
            DEFAULT_PAGE
        } else {
            limit.min(MAX_PAGE)
        };
        let offset = offset.max(0);
        // Normalize the query into a handle key for the prefix match; handles carry
        // no '@', so an email query just won't hit the handle branch (it matches the
        // repo's email branch instead). Empty query → empty key → list all.
        let handle_key = if query.is_empty() {
            String::new()
        } else {
            crate::handle_core::normalize(query)
        };
        self.repo
            .list_accounts(query, &handle_key, limit, offset)
            .await
    }
}

/// Reject an unrecognized scope or role before any write (change: add-moderation-
/// back-office), so a typo or an arbitrary role value can never be persisted.
fn validate_scope_role(scope: &str, role: &str) -> Result<()> {
    if !SCOPES.contains(&scope) {
        return Err(AppError::InvalidArgument(format!(
            "unknown scope {scope:?}"
        )));
    }
    if !ROLES.contains(&role) {
        return Err(AppError::InvalidArgument(format!("unknown role {role:?}")));
    }
    Ok(())
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
    async fn delete_account_enqueues_purge_user_when_wired() {
        use cymbra_jobs::FakeEnqueuer;
        let enq = Arc::new(FakeEnqueuer::default());
        let m = UserModule::new(FakeUserRepo::default()).with_enqueuer(enq.clone());
        let a = m.resolve_or_provision("google", "g1").await.unwrap();

        m.delete_account(&a).await.unwrap();

        // The direct delete is NOT performed — the enqueued job owns the
        // `user_account` erasure so it stays in the single atomic transaction.
        assert!(m.get_account(&a).await.is_ok());
        // Exactly one `purge_user` job was enqueued, carrying the user id.
        let reqs = enq.requests();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].name, PURGE_USER);
        assert!(reqs[0].payload_json.contains(&a));
    }

    #[tokio::test]
    async fn reap_purges_only_old_handle_less_accounts() {
        let m = module();
        // Old, no handle → reaped.
        let old = m.resolve_or_provision("google", "old").await.unwrap();
        m.repo.set_created_at(&old, 100);
        // Recent, no handle → kept.
        let recent = m.resolve_or_provision("google", "recent").await.unwrap();
        m.repo.set_created_at(&recent, 5_000);
        // Old but onboarded (has a handle) → kept.
        let onboarded = m.resolve_or_provision("google", "named").await.unwrap();
        m.repo.set_created_at(&onboarded, 100);
        m.update_account(&onboarded, None, Some("Alice".into()), "{}", 1)
            .await
            .unwrap();

        // now=2000, grace=1000 → cutoff=1000; only `old` (created 100) qualifies.
        let purged = m.reap_orphans(2_000, 1_000).await.unwrap();
        assert_eq!(purged, 1);
        assert!(matches!(
            m.get_account(&old).await,
            Err(AppError::NotFound(_))
        ));
        assert!(m.get_account(&recent).await.is_ok());
        assert!(m.get_account(&onboarded).await.is_ok());
    }

    #[tokio::test]
    async fn grant_role_adds_moderator_and_audits() {
        let m = module();
        let admin = m.resolve_or_provision("google", "admin").await.unwrap();
        let target = m.resolve_or_provision("google", "target").await.unwrap();
        // Grant music/moderator → it appears in the music-audience effective roles.
        m.grant_role(&admin, &target, "music", "moderator")
            .await
            .unwrap();
        assert!(
            m.effective_roles(&target, "music")
                .await
                .unwrap()
                .contains(&"moderator".to_string())
        );
        // Idempotent: a second grant is a no-op success (but still audits).
        m.grant_role(&admin, &target, "music", "moderator")
            .await
            .unwrap();
        // The audit records the grants, most recent first, attributed to the admin.
        let grants = m.list_role_grants(&target).await.unwrap();
        assert_eq!(grants.len(), 2);
        assert_eq!(grants[0].action, "grant");
        assert_eq!(grants[0].role, "moderator");
        assert_eq!(grants[0].scope, "music");
        assert_eq!(grants[0].acting_admin, admin);
    }

    #[tokio::test]
    async fn revoke_role_removes_and_audits() {
        let m = module();
        let admin = m.resolve_or_provision("google", "admin").await.unwrap();
        let target = m.resolve_or_provision("google", "target").await.unwrap();
        m.grant_role(&admin, &target, "music", "moderator")
            .await
            .unwrap();
        m.revoke_role(&admin, &target, "music", "moderator")
            .await
            .unwrap();
        assert!(
            !m.effective_roles(&target, "music")
                .await
                .unwrap()
                .contains(&"moderator".to_string())
        );
        // The most recent audit entry is the revoke.
        let grants = m.list_role_grants(&target).await.unwrap();
        assert_eq!(grants[0].action, "revoke");
    }

    #[tokio::test]
    async fn grant_role_rejects_unknown_scope_or_role() {
        let m = module();
        let admin = m.resolve_or_provision("google", "admin").await.unwrap();
        let t = m.resolve_or_provision("google", "t").await.unwrap();
        assert!(matches!(
            m.grant_role(&admin, &t, "galaxy", "moderator").await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            m.grant_role(&admin, &t, "music", "wizard").await,
            Err(AppError::InvalidArgument(_))
        ));
        // A rejected grant writes neither the role nor an audit entry (the default
        // `user` role is still present, but no `moderator` was added).
        assert!(
            !m.effective_roles(&t, "music")
                .await
                .unwrap()
                .contains(&"moderator".to_string())
        );
        assert!(m.list_role_grants(&t).await.unwrap().is_empty());
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
            .update_account(&a, Some("Ada".into()), None, "{\"theme\":\"dark\"}", 1)
            .await
            .unwrap();
        assert_eq!(acc.version, 2);
        // stale write rejected
        assert!(matches!(
            m.update_account(&a, None, None, "{}", 1).await,
            Err(AppError::Aborted(_))
        ));
    }

    #[tokio::test]
    async fn handle_is_set_and_reported_on_account() {
        let m = module();
        let a = m.resolve_or_provision("google", "g1").await.unwrap();
        assert_eq!(m.get_account(&a).await.unwrap().handle, None);

        let acc = m
            .update_account(&a, None, Some("Alice".into()), "{}", 1)
            .await
            .unwrap();
        assert_eq!(acc.handle.as_deref(), Some("Alice")); // display form preserved
        assert_eq!(
            m.get_account(&a).await.unwrap().handle.as_deref(),
            Some("Alice")
        );
    }

    #[tokio::test]
    async fn handle_uniqueness_is_case_insensitive() {
        let m = module();
        let a = m.resolve_or_provision("google", "g1").await.unwrap();
        let b = m.resolve_or_provision("google", "g2").await.unwrap();

        m.update_account(&a, None, Some("Alice".into()), "{}", 1)
            .await
            .unwrap();
        // Differs only by case → conflict.
        assert!(matches!(
            m.update_account(&b, None, Some("alice".into()), "{}", 1)
                .await,
            Err(AppError::AlreadyExists(_))
        ));
    }

    #[tokio::test]
    async fn invalid_handle_is_rejected_on_update() {
        let m = module();
        let a = m.resolve_or_provision("google", "g1").await.unwrap();
        assert!(matches!(
            m.update_account(&a, None, Some("no spaces".into()), "{}", 1)
                .await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn availability_reflects_taken_and_rejects_invalid() {
        let m = module();
        let a = m.resolve_or_provision("google", "g1").await.unwrap();
        assert!(m.check_handle_availability("Alice").await.unwrap());

        m.update_account(&a, None, Some("Alice".into()), "{}", 1)
            .await
            .unwrap();
        // Taken, case-insensitively.
        assert!(!m.check_handle_availability("alice").await.unwrap());
        assert!(!m.check_handle_availability("Alice").await.unwrap());
        // Invalid handles error rather than reporting availability.
        assert!(matches!(
            m.check_handle_availability("bad handle!").await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn list_accounts_paginates_filters_and_aggregates_roles() {
        let m = module();
        let admin = m.resolve_or_provision("google", "admin").await.unwrap();
        // Three handled accounts; `ada` also has a `local` (email) identity + role.
        let a = m.resolve_or_provision("local", "ada@x.dev").await.unwrap();
        m.update_account(&a, None, Some("ada".into()), "{}", 1)
            .await
            .unwrap();
        let b = m.resolve_or_provision("google", "b").await.unwrap();
        m.update_account(&b, None, Some("bob".into()), "{}", 1)
            .await
            .unwrap();
        let c = m.resolve_or_provision("google", "c").await.unwrap();
        m.update_account(&c, None, Some("carol".into()), "{}", 1)
            .await
            .unwrap();
        m.grant_role(&admin, &a, "music", "moderator")
            .await
            .unwrap();

        // Unfiltered: all four accounts, total independent of the page window.
        let page = m.list_accounts("", 2, 0).await.unwrap();
        assert_eq!(page.total, 4);
        assert_eq!(page.entries.len(), 2);
        let page2 = m.list_accounts("", 2, 2).await.unwrap();
        assert_eq!(page2.entries.len(), 2);
        let first_ids: Vec<&String> = page.entries.iter().map(|e| &e.user_id).collect();
        assert!(
            page2
                .entries
                .iter()
                .all(|e| !first_ids.contains(&&e.user_id))
        );

        // Filter by handle prefix (case-insensitive) + role aggregation.
        let by_handle = m.list_accounts("AD", 25, 0).await.unwrap();
        assert_eq!(by_handle.total, 1);
        assert_eq!(by_handle.entries[0].handle.as_deref(), Some("ada"));
        assert!(
            by_handle.entries[0]
                .roles
                .contains(&"moderator".to_string())
        );

        // Filter by the email of a local identity.
        let by_email = m.list_accounts("ada@x.dev", 25, 0).await.unwrap();
        assert_eq!(by_email.total, 1);
        assert_eq!(by_email.entries[0].user_id, a);

        // No match → empty page, total 0 (not an error).
        let none = m.list_accounts("nobody", 25, 0).await.unwrap();
        assert_eq!(none.total, 0);
        assert!(none.entries.is_empty());
    }

    #[tokio::test]
    async fn list_accounts_clamps_paging() {
        let m = module();
        m.resolve_or_provision("google", "x").await.unwrap();
        // limit <= 0 falls back to the default window rather than returning nothing.
        let page = m.list_accounts("", 0, 0).await.unwrap();
        assert_eq!(page.entries.len(), 1);
    }
}
