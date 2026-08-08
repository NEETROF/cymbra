//! The user module's **direct adapter** (task 3.4): implements [`UserPort`] with
//! the account invariants on top of a [`UserRepo`].

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;
use cymbra_jobs::{EnqueueRequest, Enqueuer, PURGE_USER};
use cymbra_platform::{AppError, Result};
use cymbra_user_port::{Account, AccountPage, Identity, PlayerProfile, UserPort, Visibility};
use serde::Serialize;

use crate::profile_core;
use crate::repo::UserRepo;

/// Default minimum age to make a profile public (change: add-play-activity-
/// profile, D6) — 16, the strictest EU digital-consent age. Overridable from
/// config via [`UserModule::with_min_public_sharing_age`].
const DEFAULT_MIN_PUBLIC_SHARING_AGE: u32 = 16;

/// Recognized role values (change: add-moderation-back-office). `moderator` is
/// added here so a grant of `music/moderator` is accepted and flows into the token
/// via `effective_roles`.
const ROLES: [&str; 3] = ["user", "admin", "moderator"];
/// Recognized authorization scopes (module audiences + the `global` break-glass).
/// `pub(crate)` so the gRPC layer can compute a caller's authorized scopes.
pub(crate) const SCOPES: [&str; 3] = ["global", "music", "live"];
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
    /// Minimum age (years) required to make a profile public (change: add-play-
    /// activity-profile). From config; defaults to 16.
    min_public_sharing_age: u32,
}

impl<R: UserRepo> UserModule<R> {
    pub fn new(repo: R) -> Self {
        Self {
            repo,
            enqueuer: None,
            min_public_sharing_age: DEFAULT_MIN_PUBLIC_SHARING_AGE,
        }
    }

    /// Wire the enqueue seam so `delete_account` performs a complete erasure via
    /// the `purge_user` job (change: complete-account-deletion). Used by the
    /// server composition root; the worker keeps the enqueuer unset.
    pub fn with_enqueuer(mut self, enqueuer: Arc<dyn Enqueuer>) -> Self {
        self.enqueuer = Some(enqueuer);
        self
    }

    /// Override the minimum public-sharing age from config (change: add-play-
    /// activity-profile). The server composition root threads
    /// `CYMBRA_MIN_PUBLIC_SHARING_AGE` here.
    pub fn with_min_public_sharing_age(mut self, min_age: u32) -> Self {
        self.min_public_sharing_age = min_age;
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

    async fn set_locale(&self, user_id: &str, locale: &str) -> Result<()> {
        // No-op on empty input (change: persist-user-locale, D3): a locale-less
        // call must never clear a previously recorded preference.
        if locale.is_empty() {
            return Ok(());
        }
        self.repo.set_locale(user_id, locale).await
    }

    async fn locale(&self, user_id: &str) -> Result<Option<String>> {
        self.repo.locale(user_id).await
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

    async fn scoped_effective_roles(
        &self,
        user_id: &str,
        scopes: &[String],
    ) -> Result<BTreeMap<String, Vec<String>>> {
        let pairs = self.repo.roles_by_scope(user_id, scopes).await?;
        let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (scope, role) in pairs {
            out.entry(scope).or_default().push(role);
        }
        Ok(out)
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

    async fn list_accounts(
        &self,
        query: &str,
        limit: i64,
        offset: i64,
        scopes: &[String],
    ) -> Result<AccountPage> {
        // Clamp paging (default when unset, capped) so a caller can't drain the
        // table; authorization (admin-only, per authorized scope) is enforced at the
        // gRPC layer, which also picks the `scopes` this directory may expose.
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
            .list_accounts(query, &handle_key, limit, offset, scopes)
            .await
    }

    async fn get_player_profile(
        &self,
        viewer_id: &str,
        target_id: &str,
        today: NaiveDate,
    ) -> Result<PlayerProfile> {
        let row = self.repo.profile_row(target_id).await?;
        let visibility = Visibility::parse(&row.visibility)?;
        // The owner always sees their own profile in full, whatever the setting.
        if viewer_id == target_id {
            return Ok(PlayerProfile {
                user_id: target_id.to_string(),
                handle: row.handle,
                display_name: row.display_name,
                visibility,
            });
        }
        // Another player: expose only a public AND age-eligible profile; anything
        // else is `NotFound` (fail-closed — never reveal a private profile).
        if visibility == Visibility::Public && self.eligible_now(&row.share_eligible_from, today) {
            Ok(PlayerProfile {
                user_id: target_id.to_string(),
                handle: row.handle,
                display_name: row.display_name,
                visibility,
            })
        } else {
            Err(AppError::NotFound("profile".into()))
        }
    }

    async fn set_profile_visibility(
        &self,
        user_id: &str,
        visibility: Visibility,
        date_of_birth: Option<NaiveDate>,
        today: NaiveDate,
    ) -> Result<Visibility> {
        // Private is always allowed and never touches the age data.
        if visibility != Visibility::Public {
            self.repo
                .update_visibility(user_id, visibility.as_str(), None)
                .await?;
            return Ok(visibility);
        }

        // Going public: resolve the eligibility date. A **supplied DOB always
        // wins** — it re-derives and overrides any stored date, so a user who
        // mis-entered their birth date once can correct it (the DOB itself is
        // still discarded; only the derived date is stored). With no DOB, fall
        // back to the stored date (the "already established my age, just flip me
        // public" path). The check is fail-closed: refuse when not yet eligible.
        let eligible_from = match date_of_birth {
            Some(dob) => {
                if !profile_core::dob_is_plausible(dob, today) {
                    return Err(AppError::InvalidArgument(
                        "date of birth cannot be in the future".into(),
                    ));
                }
                profile_core::derive_eligible_from(dob, self.min_public_sharing_age)
            }
            None => self
                .repo
                .profile_row(user_id)
                .await?
                .share_eligible_from
                .ok_or_else(|| {
                    AppError::FailedPrecondition(
                        "date of birth is required to make a profile public".into(),
                    )
                })?,
        };

        if profile_core::is_eligible(today, eligible_from) {
            self.repo
                .update_visibility(user_id, Visibility::Public.as_str(), Some(eligible_from))
                .await?;
            Ok(Visibility::Public)
        } else {
            // Persist the derived date (so a later attempt needs no DOB and can
            // succeed once the date passes) but keep the profile private and refuse.
            self.repo
                .update_visibility(user_id, Visibility::Private.as_str(), Some(eligible_from))
                .await?;
            Err(AppError::FailedPrecondition(
                "not old enough to make the profile public".into(),
            ))
        }
    }

    async fn activity_visible_to(
        &self,
        owner_id: &str,
        viewer_id: &str,
        today: NaiveDate,
    ) -> Result<bool> {
        if owner_id == viewer_id {
            return Ok(true);
        }
        let row = self.repo.profile_row(owner_id).await?;
        let visibility = Visibility::parse(&row.visibility)?;
        Ok(visibility == Visibility::Public && self.eligible_now(&row.share_eligible_from, today))
    }

    async fn listable_profiles(
        &self,
        user_ids: &[String],
        today: NaiveDate,
    ) -> Result<Vec<PlayerProfile>> {
        let mut out = Vec::new();
        for id in user_ids {
            // A deleted/unknown id is simply not listed (fail-closed); a transient
            // read error still propagates rather than silently dropping a player.
            let row = match self.repo.profile_row(id).await {
                Ok(row) => row,
                Err(AppError::NotFound(_)) => continue,
                Err(e) => return Err(e),
            };
            let visibility = Visibility::parse(&row.visibility)?;
            if visibility == Visibility::Public
                && self.eligible_now(&row.share_eligible_from, today)
            {
                out.push(PlayerProfile {
                    user_id: id.clone(),
                    handle: row.handle,
                    display_name: row.display_name,
                    visibility,
                });
            }
        }
        Ok(out)
    }

    async fn age_eligible_profiles(
        &self,
        user_ids: &[String],
        today: NaiveDate,
    ) -> Result<Vec<PlayerProfile>> {
        let mut out = Vec::new();
        for id in user_ids {
            // Same fail-closed handling as `listable_profiles`: an unknown id is
            // simply absent, a transient read error still propagates.
            let row = match self.repo.profile_row(id).await {
                Ok(row) => row,
                Err(AppError::NotFound(_)) => continue,
                Err(e) => return Err(e),
            };
            // The AGE safeguard only — visibility is deliberately not consulted, so
            // a now-private player still resolves. The caller must already hold a
            // recorded past consent (see the port docs).
            if self.eligible_now(&row.share_eligible_from, today) {
                out.push(PlayerProfile {
                    user_id: id.clone(),
                    handle: row.handle,
                    display_name: row.display_name,
                    visibility: Visibility::parse(&row.visibility)?,
                });
            }
        }
        Ok(out)
    }
}

impl<R: UserRepo> UserModule<R> {
    /// Age-eligible today iff an eligibility date exists and `today` is strictly
    /// past it (the one-day margin). No date ⇒ never eligible (fail-closed).
    fn eligible_now(&self, share_eligible_from: &Option<NaiveDate>, today: NaiveDate) -> bool {
        share_eligible_from
            .map(|d| profile_core::is_eligible(today, d))
            .unwrap_or(false)
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

    /// Owned scope list for the `&[String]` directory/role APIs.
    fn sc(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
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
        m.update_account(&admin, None, Some("bossadmin".into()), "{}", 1)
            .await
            .unwrap();
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
        // The audit resolves the acting admin's handle (shown instead of the UUID).
        assert_eq!(grants[0].acting_admin_handle.as_deref(), Some("bossadmin"));
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
        let page = m.list_accounts("", 2, 0, &sc(&["music"])).await.unwrap();
        assert_eq!(page.total, 4);
        assert_eq!(page.entries.len(), 2);
        let page2 = m.list_accounts("", 2, 2, &sc(&["music"])).await.unwrap();
        assert_eq!(page2.entries.len(), 2);
        let first_ids: Vec<&String> = page.entries.iter().map(|e| &e.user_id).collect();
        assert!(
            page2
                .entries
                .iter()
                .all(|e| !first_ids.contains(&&e.user_id))
        );

        // Filter by handle prefix (case-insensitive) + role aggregation per scope.
        let by_handle = m.list_accounts("AD", 25, 0, &sc(&["music"])).await.unwrap();
        assert_eq!(by_handle.total, 1);
        assert_eq!(by_handle.entries[0].handle.as_deref(), Some("ada"));
        let music = by_handle.entries[0]
            .roles_by_scope
            .iter()
            .find(|sr| sr.scope == "music")
            .unwrap();
        assert!(music.roles.contains(&"moderator".to_string()));

        // Filter by the email of a local identity.
        let by_email = m
            .list_accounts("ada@x.dev", 25, 0, &sc(&["music"]))
            .await
            .unwrap();
        assert_eq!(by_email.total, 1);
        assert_eq!(by_email.entries[0].user_id, a);

        // No match → empty page, total 0 (not an error).
        let none = m
            .list_accounts("nobody", 25, 0, &sc(&["music"]))
            .await
            .unwrap();
        assert_eq!(none.total, 0);
        assert!(none.entries.is_empty());
    }

    #[tokio::test]
    async fn list_accounts_clamps_paging() {
        let m = module();
        m.resolve_or_provision("google", "x").await.unwrap();
        // limit <= 0 falls back to the default window rather than returning nothing.
        let page = m.list_accounts("", 0, 0, &sc(&["music"])).await.unwrap();
        assert_eq!(page.entries.len(), 1);
    }

    // --- Preferred locale (change: persist-user-locale) -----------------------

    #[tokio::test]
    async fn set_locale_writes_reads_back_and_is_last_writer_wins() {
        // State-based round-trip: FakeUserRepo is a behavioural in-memory adapter
        // (rust-testing skill special case), so write-then-read reads clearer here
        // than mock expectation chains.
        let m = module();
        let u = m.resolve_or_provision("google", "g1").await.unwrap();
        // No locale recorded yet → None (callers treat this as English).
        assert_eq!(m.locale(&u).await.unwrap(), None);
        // First write records it; a later non-empty write overwrites it.
        m.set_locale(&u, "fr").await.unwrap();
        assert_eq!(m.locale(&u).await.unwrap().as_deref(), Some("fr"));
        m.set_locale(&u, "es").await.unwrap();
        assert_eq!(m.locale(&u).await.unwrap().as_deref(), Some("es"));
    }

    #[tokio::test]
    async fn set_locale_is_a_noop_on_empty_input() {
        let m = module();
        let u = m.resolve_or_provision("google", "g1").await.unwrap();
        m.set_locale(&u, "fr").await.unwrap();
        // An empty locale must never clear an already-stored preference.
        m.set_locale(&u, "").await.unwrap();
        assert_eq!(m.locale(&u).await.unwrap().as_deref(), Some("fr"));
    }

    #[tokio::test]
    async fn scoped_effective_roles_groups_by_scope() {
        let m = module();
        let a = m.resolve_or_provision("google", "g1").await.unwrap();
        // Default global/user, plus a music/admin and a live/moderator.
        m.repo.grant_role(&a, "music", "admin").await.unwrap();
        m.repo.grant_role(&a, "live", "moderator").await.unwrap();
        let by_scope = m
            .scoped_effective_roles(&a, &sc(&["global", "music", "live"]))
            .await
            .unwrap();
        assert_eq!(by_scope["global"], vec!["user".to_string()]);
        assert_eq!(by_scope["music"], vec!["admin".to_string()]);
        assert_eq!(by_scope["live"], vec!["moderator".to_string()]);
        // A scope with no role is omitted entirely.
        let only_music = m.scoped_effective_roles(&a, &sc(&["music"])).await.unwrap();
        assert_eq!(only_music.len(), 1);
        assert!(only_music.contains_key("music"));
    }

    // --- Public profile / visibility (change: add-play-activity-profile) ------

    fn ymd(y: i32, mo: u32, d: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(y, mo, d).unwrap()
    }
    // A fixed "today" (UTC) for deterministic eligibility checks.
    fn today() -> chrono::NaiveDate {
        ymd(2026, 7, 28)
    }

    #[tokio::test]
    async fn new_profile_is_private_and_owner_sees_it() {
        let m = module();
        let u = m.resolve_or_provision("google", "g1").await.unwrap();
        m.update_account(&u, None, Some("ada".into()), "{}", 1)
            .await
            .unwrap();
        // Owner viewing self sees the (private) profile in full.
        let p = m.get_player_profile(&u, &u, today()).await.unwrap();
        assert_eq!(p.visibility, Visibility::Private);
        assert_eq!(p.handle.as_deref(), Some("ada"));
        // Another player is refused (fail-closed: NotFound, not a private stub).
        let other = m.resolve_or_provision("google", "g2").await.unwrap();
        assert!(matches!(
            m.get_player_profile(&other, &u, today()).await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn eligible_user_can_go_public_and_is_visible_to_others() {
        let m = module();
        let u = m.resolve_or_provision("google", "g1").await.unwrap();
        m.update_account(&u, None, Some("ada".into()), "{}", 1)
            .await
            .unwrap();
        // Born 2000 → eligible date 2016 → today (2026) is past it.
        let now = m
            .set_profile_visibility(&u, Visibility::Public, Some(ymd(2000, 1, 1)), today())
            .await
            .unwrap();
        assert_eq!(now, Visibility::Public);
        // Another player now sees the allow-listed public profile.
        let other = m.resolve_or_provision("google", "g2").await.unwrap();
        let p = m.get_player_profile(&other, &u, today()).await.unwrap();
        assert_eq!(p.visibility, Visibility::Public);
        assert_eq!(p.handle.as_deref(), Some("ada"));
        // And activity is visible to others.
        assert!(m.activity_visible_to(&u, &other, today()).await.unwrap());
    }

    #[tokio::test]
    async fn under_age_user_is_refused_fail_closed_and_dob_not_stored() {
        let m = module();
        let u = m.resolve_or_provision("google", "g1").await.unwrap();
        // Born 2015 → eligible 2031 → not eligible in 2026: refused, stays private.
        assert!(matches!(
            m.set_profile_visibility(&u, Visibility::Public, Some(ymd(2015, 1, 1)), today())
                .await,
            Err(AppError::FailedPrecondition(_))
        ));
        // Fail-closed: still private, and neither the owner-view nor others expose
        // it as public. Only the DERIVED eligibility date is kept (2031-01-01) —
        // never the DOB — so a later attempt needs no DOB and still fails until 2031.
        let row = m.repo.profile_row(&u).await.unwrap();
        assert_eq!(row.visibility, "private");
        assert_eq!(row.share_eligible_from, Some(ymd(2031, 1, 1)));
        let other = m.resolve_or_provision("google", "g2").await.unwrap();
        assert!(!m.activity_visible_to(&u, &other, today()).await.unwrap());
        // A retry without DOB uses the stored date and is still refused.
        assert!(matches!(
            m.set_profile_visibility(&u, Visibility::Public, None, today())
                .await,
            Err(AppError::FailedPrecondition(_))
        ));
    }

    #[tokio::test]
    async fn a_fresh_dob_overrides_a_previously_stored_eligibility_date() {
        // Regression: a mis-entered birth date must be correctable. A first attempt
        // with a too-young DOB stores a future eligibility date and is refused...
        let m = module();
        let u = m.resolve_or_provision("google", "g1").await.unwrap();
        assert!(matches!(
            m.set_profile_visibility(&u, Visibility::Public, Some(ymd(2010, 7, 28)), today())
                .await,
            Err(AppError::FailedPrecondition(_))
        ));
        assert_eq!(
            m.repo.profile_row(&u).await.unwrap().share_eligible_from,
            Some(ymd(2026, 7, 28))
        );
        // ...but re-entering a correct, eligible DOB now succeeds: the fresh DOB
        // wins over the stored date (previously it was ignored → stuck forever).
        let now = m
            .set_profile_visibility(&u, Visibility::Public, Some(ymd(1982, 7, 28)), today())
            .await
            .unwrap();
        assert_eq!(now, Visibility::Public);
        assert_eq!(
            m.repo.profile_row(&u).await.unwrap().share_eligible_from,
            Some(ymd(1998, 7, 28))
        );
    }

    #[tokio::test]
    async fn going_public_requires_dob_when_no_eligibility_yet() {
        let m = module();
        let u = m.resolve_or_provision("google", "g1").await.unwrap();
        assert!(matches!(
            m.set_profile_visibility(&u, Visibility::Public, None, today())
                .await,
            Err(AppError::FailedPrecondition(_))
        ));
    }

    #[tokio::test]
    async fn private_toggle_needs_no_dob_and_preserves_eligibility() {
        let m = module();
        let u = m.resolve_or_provision("google", "g1").await.unwrap();
        // Establish eligibility + go public.
        m.set_profile_visibility(&u, Visibility::Public, Some(ymd(2000, 1, 1)), today())
            .await
            .unwrap();
        // Back to private without a DOB; the derived date is preserved (COALESCE).
        assert_eq!(
            m.set_profile_visibility(&u, Visibility::Private, None, today())
                .await
                .unwrap(),
            Visibility::Private
        );
        assert_eq!(
            m.repo.profile_row(&u).await.unwrap().share_eligible_from,
            Some(ymd(2016, 1, 1))
        );
        // Re-going public now needs no DOB (date already stored) and succeeds.
        assert_eq!(
            m.set_profile_visibility(&u, Visibility::Public, None, today())
                .await
                .unwrap(),
            Visibility::Public
        );
    }

    #[tokio::test]
    async fn public_but_not_yet_eligible_is_hidden_from_others_fail_closed() {
        // Directly craft the state a stale/modified client might try to exploit:
        // visibility=public but the eligibility date is in the future.
        let m = module();
        let u = m.resolve_or_provision("google", "g1").await.unwrap();
        m.repo
            .update_visibility(&u, "public", Some(ymd(2031, 1, 1)))
            .await
            .unwrap();
        let other = m.resolve_or_provision("google", "g2").await.unwrap();
        // Read + activity gate both refuse (server-enforced eligibility).
        assert!(matches!(
            m.get_player_profile(&other, &u, today()).await,
            Err(AppError::NotFound(_))
        ));
        assert!(!m.activity_visible_to(&u, &other, today()).await.unwrap());
    }

    #[tokio::test]
    async fn min_age_override_changes_the_derived_date() {
        // With min age 18, a 2009 birth is eligible from 2027 → still under-age in 2026.
        let m = UserModule::new(FakeUserRepo::default()).with_min_public_sharing_age(18);
        let u = m.resolve_or_provision("google", "g1").await.unwrap();
        assert!(matches!(
            m.set_profile_visibility(&u, Visibility::Public, Some(ymd(2009, 1, 1)), today())
                .await,
            Err(AppError::FailedPrecondition(_))
        ));
        assert_eq!(
            m.repo.profile_row(&u).await.unwrap().share_eligible_from,
            Some(ymd(2027, 1, 1))
        );
    }

    /// `age_eligible_profiles` is the AGE half of the listing gate on its own
    /// (change: add-global-leaderboard) — it backs the frozen-history archive read,
    /// where consent was already recorded at snapshot time.
    #[tokio::test]
    async fn age_eligible_profiles_is_visibility_blind_but_fails_closed_on_age() {
        let m = module();
        // An adult who went public, then back to private.
        let adult = m.resolve_or_provision("google", "adult").await.unwrap();
        m.set_profile_visibility(&adult, Visibility::Public, Some(ymd(1990, 1, 1)), today())
            .await
            .unwrap();
        m.set_profile_visibility(&adult, Visibility::Private, None, today())
            .await
            .unwrap();
        // An account that never established an eligibility date.
        let unknown_age = m.resolve_or_provision("google", "noage").await.unwrap();

        let ids = vec![adult.clone(), unknown_age.clone(), "ghost".to_string()];
        let out = m.age_eligible_profiles(&ids, today()).await.unwrap();
        let got: Vec<&str> = out.iter().map(|p| p.user_id.as_str()).collect();

        // Visibility-blind: the now-PRIVATE adult still resolves…
        assert_eq!(got, vec![adult.as_str()]);
        // …while no eligibility date is fail-closed, and an unknown id is absent.
        assert!(!got.contains(&unknown_age.as_str()));
        assert!(!got.contains(&"ghost"));

        // Contrast: the full listing gate drops the now-private adult.
        assert!(m.listable_profiles(&ids, today()).await.unwrap().is_empty());
    }
}
