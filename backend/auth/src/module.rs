//! The auth module's **direct adapter** (tasks 4.4–4.9): implements [`AuthPort`]
//! over the credential store, the user port, the session store, the email sender,
//! the cache (rate-limit), and an [`OidcVerifier`].

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use cymbra_auth_port::{AuthPort, TokenPair};
use cymbra_platform::cache::Cache;
use cymbra_platform::email::EmailSender;
use cymbra_platform::email_template::{self, SupportedLocale};
use cymbra_platform::{AppError, Result, password, ratelimit, token};
use cymbra_user_port::UserPort;
use jsonwebtoken::EncodingKey;

use crate::creds::CredentialRepo;
use crate::pending_setpw::{PendingCredentialStore, PendingLocalCredential};
use crate::session::SessionStore;
use crate::verifier::OidcVerifier;

/// Tunables for the auth module (sourced from [`cymbra_platform::config::Config`]).
#[derive(Clone)]
pub struct AuthConfig {
    pub access_ttl: Duration,
    pub refresh_ttl: Duration,
    pub allowed_audiences: Vec<String>,
    pub password_min_length: usize,
    pub signin_max_attempts: u32,
    pub signin_lockout: Duration,
    pub email_max: u32,
    pub email_window: Duration,
    pub verify_ttl: Duration,
    pub reset_ttl: Duration,
    /// Hosted "Cymbra ID" logo URL for branded emails; `None` = text wordmark only.
    pub email_logo_url: Option<String>,
}

/// In-process auth implementation.
pub struct AuthModule {
    user: Arc<dyn UserPort>,
    creds: Arc<dyn CredentialRepo>,
    cache: Arc<dyn Cache>,
    /// Parked set-password submissions awaiting email verification (design D1).
    pending: Arc<dyn PendingCredentialStore>,
    email: Arc<dyn EmailSender>,
    oidc: Arc<dyn OidcVerifier>,
    sessions: Arc<dyn SessionStore>,
    signing_key: EncodingKey,
    kid: String,
    cfg: AuthConfig,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Build the enqueue request for a verification email. The producer renders the
/// branded multipart email here, so the worker's `verification_email` handler just
/// transports the `{to, subject, html, text}` payload (design D2). Idempotency is
/// provided by the single-use token: a re-delivered job re-sends the same code.
fn verification_email_job(
    email: &str,
    token: &str,
    locale: SupportedLocale,
    logo_url: Option<&str>,
) -> Result<cymbra_jobs::EnqueueRequest> {
    let spec = cymbra_jobs::registry::spec(cymbra_jobs::VERIFICATION_EMAIL).ok_or_else(|| {
        AppError::Internal(anyhow::anyhow!("missing verification_email job spec"))
    })?;
    let rendered = email_template::verification_email(token, locale, logo_url);
    let payload = serde_json::json!({
        "to": email,
        "subject": rendered.subject,
        "html": rendered.html,
        "text": rendered.text,
    });
    cymbra_jobs::EnqueueRequest::for_job(&spec, &payload, None)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("build verification email job: {e}")))
}

impl AuthModule {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user: Arc<dyn UserPort>,
        creds: Arc<dyn CredentialRepo>,
        cache: Arc<dyn Cache>,
        pending: Arc<dyn PendingCredentialStore>,
        email: Arc<dyn EmailSender>,
        oidc: Arc<dyn OidcVerifier>,
        sessions: Arc<dyn SessionStore>,
        signing_key_pem: &str,
        kid: &str,
        cfg: AuthConfig,
    ) -> Result<Self> {
        let signing_key = token::encoding_key(signing_key_pem)?;
        Ok(Self {
            user,
            creds,
            cache,
            pending,
            email,
            oidc,
            sessions,
            signing_key,
            kid: kid.to_string(),
            cfg,
        })
    }

    fn check_audience(&self, audience: &str) -> Result<()> {
        if self.cfg.allowed_audiences.iter().any(|a| a == audience) {
            Ok(())
        } else {
            Err(AppError::InvalidArgument(format!(
                "unknown app audience `{audience}`"
            )))
        }
    }

    /// Mint an access (signed) + refresh (session) token pair for `audience`.
    async fn issue(&self, user_id: &str, audience: &str) -> Result<TokenPair> {
        let roles = self.user.effective_roles(user_id, audience).await?;
        let claims = token::new_claims(user_id, audience, roles, self.cfg.access_ttl);
        let access = token::sign(&claims, &self.kid, &self.signing_key)?;
        let refresh = self.sessions.create(user_id, audience).await?;
        Ok(TokenPair {
            access_token: access,
            refresh_token: refresh,
        })
    }
}

impl AuthConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        access_ttl: Duration,
        refresh_ttl: Duration,
        allowed_audiences: Vec<String>,
        password_min_length: usize,
        signin_max_attempts: u32,
        signin_lockout: Duration,
        email_max: u32,
        email_window: Duration,
        verify_ttl: Duration,
        reset_ttl: Duration,
        email_logo_url: Option<String>,
    ) -> Self {
        Self {
            access_ttl,
            refresh_ttl,
            allowed_audiences,
            password_min_length,
            signin_max_attempts,
            signin_lockout,
            email_max,
            email_window,
            verify_ttl,
            reset_ttl,
            email_logo_url,
        }
    }
}

#[async_trait]
impl AuthPort for AuthModule {
    async fn sign_up_local(&self, email: &str, password: &str, locale: &str) -> Result<()> {
        password::check_policy(password, self.cfg.password_min_length)?;
        let hash = password::hash(password)?;
        self.creds.insert(email, &hash).await?; // AlreadyExists if taken
        // Provision the shared account + its `local` identity.
        self.user.resolve_or_provision("local", email).await?;
        let tok = uuid::Uuid::new_v4().to_string();
        let exp = now_secs() + self.cfg.verify_ttl.as_secs() as i64;
        // Enqueue the verification email as a job in the same transaction as the
        // verification write — SMTP is now off the request path, so a mail
        // failure can never fail an account that already exists (design D10).
        let job = verification_email_job(
            email,
            &tok,
            SupportedLocale::parse(Some(locale)),
            self.cfg.email_logo_url.as_deref(),
        )?;
        self.creds
            .set_verification_with_job(email, &tok, exp, &job)
            .await?;
        Ok(())
    }

    async fn verify_email(&self, token: &str) -> Result<()> {
        // Pending set-password path (change: verify-before-local-credential-link):
        // a token parked by `set_local_credential` binds the credential + `local`
        // identity only now, once ownership of the email is proven.
        if let Some(p) = self.pending.take(token).await? {
            // A prior pending for the same account may have bound a local identity
            // already — keep the one-local-credential-per-account invariant.
            if self
                .user
                .list_identities(&p.user_id)
                .await?
                .iter()
                .any(|i| i.provider == "local")
            {
                return Err(AppError::AlreadyExists(
                    "account already has a password".into(),
                ));
            }
            // The email was free when submitted; bind it now (verified). If it was
            // taken meanwhile, `insert_verified`/`link_identity` fails cleanly —
            // nothing was ever reserved.
            self.creds
                .insert_verified(&p.email, &p.password_hash)
                .await?;
            if let Err(e) = self.user.link_identity(&p.user_id, "local", &p.email).await {
                let _ = self.creds.delete_credentials(&p.email).await;
                return Err(e);
            }
            return Ok(());
        }
        // Sign-up path (unchanged): mark an existing unverified credential verified.
        match self.creds.verify_by_token(token, now_secs()).await? {
            Some(_) => Ok(()),
            None => Err(AppError::InvalidArgument(
                "invalid or expired verification token".into(),
            )),
        }
    }

    async fn resend_verification(&self, email: &str, locale: &str) -> Result<()> {
        ratelimit::check(
            self.cache.as_ref(),
            "verify_email",
            email,
            self.cfg.email_max,
            self.cfg.email_window,
        )
        .await?;
        if let Some(cred) = self.creds.get(email).await?
            && !cred.email_verified
        {
            let tok = uuid::Uuid::new_v4().to_string();
            let exp = now_secs() + self.cfg.verify_ttl.as_secs() as i64;
            self.creds.set_verification(email, &tok, exp).await?;
            let rendered = email_template::verification_email(
                &tok,
                SupportedLocale::parse(Some(locale)),
                self.cfg.email_logo_url.as_deref(),
            );
            self.email.send(email, &rendered).await?;
        }
        Ok(()) // never reveals whether the email exists / is verified
    }

    async fn sign_in_local(
        &self,
        email: &str,
        password: &str,
        audience: &str,
    ) -> Result<TokenPair> {
        self.check_audience(audience)?;
        let lock_key = format!("signin:{email}");
        let attempts: u32 = self
            .cache
            .get(&lock_key)
            .await?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        if attempts >= self.cfg.signin_max_attempts {
            return Err(AppError::ResourceExhausted(
                "too many sign-in attempts, try again later".into(),
            ));
        }

        let ok = match self.creds.get(email).await? {
            Some(cred) if password::verify(password, &cred.password_hash) => Some(cred),
            _ => None,
        };
        let cred = match ok {
            Some(c) => c,
            None => {
                self.cache
                    .incr_with_ttl(&lock_key, self.cfg.signin_lockout)
                    .await?;
                return Err(AppError::Unauthenticated("invalid credentials".into()));
            }
        };
        if !cred.email_verified {
            return Err(AppError::FailedPrecondition("email not verified".into()));
        }
        self.cache.del(&lock_key).await?; // clear failures on success
        let user_id = self.user.resolve_or_provision("local", email).await?;
        self.issue(&user_id, audience).await
    }

    async fn sign_in_oidc(&self, id_token: &str, audience: &str) -> Result<TokenPair> {
        self.check_audience(audience)?;
        let ext = self.oidc.verify(id_token).await?;
        let user_id = self
            .user
            .resolve_or_provision(&ext.provider, &ext.subject)
            .await?;
        self.issue(&user_id, audience).await
    }

    async fn refresh(&self, refresh_token: &str) -> Result<TokenPair> {
        let rot = self.sessions.rotate(refresh_token).await?;
        let roles = self
            .user
            .effective_roles(&rot.user_id, &rot.audience)
            .await?;
        let claims = token::new_claims(&rot.user_id, &rot.audience, roles, self.cfg.access_ttl);
        let access = token::sign(&claims, &self.kid, &self.signing_key)?;
        Ok(TokenPair {
            access_token: access,
            refresh_token: rot.refresh_token,
        })
    }

    async fn logout(&self, refresh_token: &str) -> Result<()> {
        self.sessions.revoke(refresh_token).await
    }

    async fn revoke_all_sessions(&self, user_id: &str) -> Result<()> {
        self.sessions.revoke_all(user_id).await
    }

    async fn revoke_account_sessions(
        &self,
        acting_admin: &str,
        target_user_id: &str,
        audience: &str,
    ) -> Result<()> {
        // One transactional, audience-scoped delete + audit in the store: the count is
        // exact (from the delete) and the trail can't be lost on a partial failure.
        self.sessions
            .revoke_account_sessions_audited(target_user_id, acting_admin, audience)
            .await?;
        Ok(())
    }

    async fn request_password_reset(&self, email: &str, locale: &str) -> Result<()> {
        ratelimit::check(
            self.cache.as_ref(),
            "reset_email",
            email,
            self.cfg.email_max,
            self.cfg.email_window,
        )
        .await?;
        if let Some(_cred) = self.creds.get(email).await? {
            let tok = uuid::Uuid::new_v4().to_string();
            let exp = now_secs() + self.cfg.reset_ttl.as_secs() as i64;
            self.creds.set_reset(email, &tok, exp).await?;
            let rendered = email_template::password_reset_email(
                &tok,
                SupportedLocale::parse(Some(locale)),
                self.cfg.email_logo_url.as_deref(),
            );
            self.email.send(email, &rendered).await?;
        }
        Ok(()) // uniform response — no account enumeration
    }

    async fn reset_password(&self, token: &str, new_password: &str) -> Result<()> {
        password::check_policy(new_password, self.cfg.password_min_length)?;
        let new_hash = password::hash(new_password)?;
        let email = self
            .creds
            .reset_by_token(token, &new_hash, now_secs())
            .await?
            .ok_or_else(|| AppError::InvalidArgument("invalid or expired reset token".into()))?;
        // Invalidate every session for this account.
        let user_id = self.user.resolve_or_provision("local", &email).await?;
        self.sessions.revoke_all(&user_id).await?;
        Ok(())
    }

    async fn link_identity(&self, user_id: &str, id_token: &str) -> Result<()> {
        let ext = self.oidc.verify(id_token).await?;
        self.user
            .link_identity(user_id, &ext.provider, &ext.subject)
            .await
    }

    async fn unlink_identity(&self, user_id: &str, provider: &str, subject: &str) -> Result<()> {
        self.user.unlink_identity(user_id, provider, subject).await
    }

    async fn set_local_credential(
        &self,
        user_id: &str,
        email: &str,
        password: &str,
        locale: &str,
    ) -> Result<()> {
        password::check_policy(password, self.cfg.password_min_length)?;
        // One local credential per account: refuse if a password is already bound.
        if self
            .user
            .list_identities(user_id)
            .await?
            .iter()
            .any(|i| i.provider == "local")
        {
            return Err(AppError::AlreadyExists(
                "account already has a password".into(),
            ));
        }
        // Friendly up-front rejection of an email that already has a credential. The
        // authoritative free-email check is at verify time (design D3); this does
        // NOT reserve the email.
        if self.creds.get(email).await?.is_some() {
            return Err(AppError::AlreadyExists("email already registered".into()));
        }
        // Do NOT bind yet (change: verify-before-local-credential-link). Park the
        // submission keyed by the token with a TTL; the credential + `local` identity
        // are created only when the emailed code is confirmed, so an unverified email
        // is never reserved and an abandoned set-password self-expires.
        let hash = password::hash(password)?;
        let tok = uuid::Uuid::new_v4().to_string();
        self.pending
            .put(
                &tok,
                &PendingLocalCredential {
                    user_id: user_id.to_string(),
                    email: email.to_string(),
                    password_hash: hash,
                },
                self.cfg.verify_ttl,
            )
            .await?;
        // Send the branded/localized verification email inline (like resend): there
        // is no DB row to enqueue a job transactionally against.
        let rendered = email_template::verification_email(
            &tok,
            SupportedLocale::parse(Some(locale)),
            self.cfg.email_logo_url.as_deref(),
        );
        self.email.send(email, &rendered).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creds::FakeCredentialRepo;
    use crate::verifier::FakeOidcVerifier;
    use cymbra_platform::cache::FakeCache;
    use cymbra_platform::email::FakeEmail;
    use cymbra_platform::token as ptoken;
    use cymbra_user::{FakeUserRepo, UserModule};
    use jsonwebtoken::DecodingKey;
    use std::collections::HashMap;

    const PRIV: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIPlT7JHCc7NTTIZVmlCgVeNNEkqsENhAZoscpnG+jSSw\n-----END PRIVATE KEY-----\n";
    const PUB: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAiCcon5VNqPUMVYki6MnxJdscxMozrXbjmdiLGUL8sqA=\n-----END PUBLIC KEY-----\n";
    const PW: &str = "a-strong-passphrase";

    struct Harness {
        m: AuthModule,
        creds: Arc<FakeCredentialRepo>,
        email: Arc<FakeEmail>,
        pending: Arc<crate::pending_setpw::FakePendingStore>,
        sessions: Arc<crate::session::FakeSessionStore>,
    }

    fn harness() -> Harness {
        let user: Arc<dyn UserPort> = Arc::new(UserModule::new(FakeUserRepo::default()));
        let creds = Arc::new(FakeCredentialRepo::default());
        let cache: Arc<dyn Cache> = Arc::new(FakeCache::default());
        let pending = Arc::new(crate::pending_setpw::FakePendingStore::default());
        let pending_dyn: Arc<dyn PendingCredentialStore> = pending.clone();
        let email = Arc::new(FakeEmail::default());
        let email_dyn: Arc<dyn EmailSender> = email.clone();
        let oidc = Arc::new(FakeOidcVerifier::default());
        let sessions = Arc::new(crate::session::FakeSessionStore::default());
        let sessions_dyn: Arc<dyn SessionStore> = sessions.clone();
        let cfg = AuthConfig::new(
            Duration::from_secs(900),
            Duration::from_secs(2_592_000),
            vec!["music".into(), "live".into()],
            12,
            3,
            Duration::from_secs(60),
            5,
            Duration::from_secs(3600),
            Duration::from_secs(86_400),
            Duration::from_secs(3600),
            None,
        );
        let m = AuthModule::new(
            user,
            creds.clone(),
            cache,
            pending_dyn,
            email_dyn,
            oidc,
            sessions_dyn,
            PRIV,
            "k1",
            cfg,
        )
        .unwrap();
        Harness {
            m,
            creds,
            email,
            pending,
            sessions,
        }
    }

    fn keys() -> HashMap<String, DecodingKey> {
        HashMap::from([("k1".to_string(), ptoken::decoding_key(PUB).unwrap())])
    }

    fn sub_of(access: &str, aud: &str) -> String {
        ptoken::verify(access, &keys(), &[aud]).unwrap().sub
    }

    #[tokio::test]
    async fn signup_verify_signin_issues_scoped_token() {
        let h = harness();
        h.m.sign_up_local("a@x.dev", PW, "").await.unwrap();
        let tok = h.creds.peek_verification_token("a@x.dev").unwrap();
        h.m.verify_email(&tok).await.unwrap();
        let pair = h.m.sign_in_local("a@x.dev", PW, "music").await.unwrap();
        let claims = ptoken::verify(&pair.access_token, &keys(), &["music"]).unwrap();
        assert_eq!(claims.aud, "music");
        assert!(claims.roles.contains(&"user".to_string()));
    }

    #[tokio::test]
    async fn signup_enqueues_email_off_request_path() {
        let h = harness();
        h.m.sign_up_local("a@x.dev", PW, "").await.unwrap();
        // SMTP is no longer on the sign-up request path (design D10).
        assert!(
            h.email.sent.lock().unwrap().is_empty(),
            "sign-up must not send email inline"
        );
        // Exactly one verification_email job was enqueued (in the verification tx).
        let jobs = h.creds.enqueued_jobs();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, cymbra_jobs::VERIFICATION_EMAIL);
        assert_eq!(jobs[0].channel_name, "auth.email");
        assert!(jobs[0].payload_json.contains("a@x.dev"));
        // The verification token is still set, so the verify flow is unaffected.
        let tok = h.creds.peek_verification_token("a@x.dev").unwrap();
        h.m.verify_email(&tok).await.unwrap();
    }

    #[tokio::test]
    async fn duplicate_and_weak_password() {
        let h = harness();
        h.m.sign_up_local("a@x.dev", PW, "").await.unwrap();
        assert!(matches!(
            h.m.sign_up_local("a@x.dev", PW, "").await,
            Err(AppError::AlreadyExists(_))
        ));
        assert!(matches!(
            h.m.sign_up_local("b@x.dev", "short", "").await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn unverified_blocked_then_wrong_password_then_lockout() {
        let h = harness();
        h.m.sign_up_local("a@x.dev", PW, "").await.unwrap();
        // unverified
        assert!(matches!(
            h.m.sign_in_local("a@x.dev", PW, "music").await,
            Err(AppError::FailedPrecondition(_))
        ));
        let tok = h.creds.peek_verification_token("a@x.dev").unwrap();
        h.m.verify_email(&tok).await.unwrap();
        // three wrong attempts -> Unauthenticated, then lockout
        for _ in 0..3 {
            assert!(matches!(
                h.m.sign_in_local("a@x.dev", "nope", "music").await,
                Err(AppError::Unauthenticated(_))
            ));
        }
        assert!(matches!(
            h.m.sign_in_local("a@x.dev", "nope", "music").await,
            Err(AppError::ResourceExhausted(_))
        ));
    }

    #[tokio::test]
    async fn oidc_signin_and_unknown_audience() {
        let h = harness();
        let pair = h.m.sign_in_oidc("g-sub-1", "live").await.unwrap();
        assert_eq!(
            ptoken::verify(&pair.access_token, &keys(), &["live"])
                .unwrap()
                .aud,
            "live"
        );
        assert!(matches!(
            h.m.sign_in_oidc("g-sub-1", "bogus").await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn refresh_rotates_then_reuse_revokes_family() {
        let h = harness();
        let pair = h.m.sign_in_oidc("g1", "music").await.unwrap();
        let p2 = h.m.refresh(&pair.refresh_token).await.unwrap();
        // replay the old refresh -> reuse detected
        assert!(matches!(
            h.m.refresh(&pair.refresh_token).await,
            Err(AppError::Unauthenticated(_))
        ));
        // the family is now revoked: the rotated token is dead too
        assert!(matches!(
            h.m.refresh(&p2.refresh_token).await,
            Err(AppError::Unauthenticated(_))
        ));
    }

    #[tokio::test]
    async fn sign_out_everywhere_revokes_all_the_users_sessions() {
        let h = harness();
        let a = h.m.sign_in_oidc("g1", "music").await.unwrap();
        let b = h.m.sign_in_oidc("g1", "live").await.unwrap();
        let uid = sub_of(&a.access_token, "music");

        h.m.revoke_all_sessions(&uid).await.unwrap();

        // Both of the account's sessions are dead; neither refresh token can rotate.
        assert!(matches!(
            h.m.refresh(&a.refresh_token).await,
            Err(AppError::Unauthenticated(_))
        ));
        assert!(matches!(
            h.m.refresh(&b.refresh_token).await,
            Err(AppError::Unauthenticated(_))
        ));
    }

    #[tokio::test]
    async fn admin_revoke_account_sessions_is_audience_scoped_and_audited() {
        let h = harness();
        let music = h.m.sign_in_oidc("target", "music").await.unwrap();
        let live = h.m.sign_in_oidc("target", "live").await.unwrap();
        let target = sub_of(&music.access_token, "music");

        // A music-scoped admin revoke cuts ONLY the target's music sessions.
        h.m.revoke_account_sessions("admin-9", &target, "music")
            .await
            .unwrap();

        // The music session is gone; the live session survives (out of scope).
        assert!(matches!(
            h.m.refresh(&music.refresh_token).await,
            Err(AppError::Unauthenticated(_))
        ));
        assert!(h.m.refresh(&live.refresh_token).await.is_ok());

        // A durable audit entry records admin, target, audience, and the exact count.
        let audit = h.sessions.admin_revocations();
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].acting_admin, "admin-9");
        assert_eq!(audit[0].target_user_id, target);
        assert_eq!(audit[0].audience, "music");
        assert_eq!(audit[0].revoked_count, 1);
    }

    #[tokio::test]
    async fn logout_revokes_session() {
        let h = harness();
        let pair = h.m.sign_in_oidc("g1", "music").await.unwrap();
        h.m.logout(&pair.refresh_token).await.unwrap();
        assert!(matches!(
            h.m.refresh(&pair.refresh_token).await,
            Err(AppError::Unauthenticated(_))
        ));
    }

    #[tokio::test]
    async fn password_reset_invalidates_sessions() {
        let h = harness();
        h.m.sign_up_local("b@x.dev", PW, "").await.unwrap();
        let vt = h.creds.peek_verification_token("b@x.dev").unwrap();
        h.m.verify_email(&vt).await.unwrap();
        let pair = h.m.sign_in_local("b@x.dev", PW, "music").await.unwrap();

        h.m.request_password_reset("b@x.dev", "").await.unwrap();
        let rt = h.creds.peek_reset_token("b@x.dev").unwrap();
        let new_pw = format!("Pw-{}-Aa1!", uuid::Uuid::new_v4());
        h.m.reset_password(&rt, &new_pw).await.unwrap();

        // old session revoked
        assert!(matches!(
            h.m.refresh(&pair.refresh_token).await,
            Err(AppError::Unauthenticated(_))
        ));
        // new password works
        h.m.sign_in_local("b@x.dev", &new_pw, "music")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn set_local_credential_defers_binding_until_verify_then_enables_signin() {
        let h = harness();
        // Start from a Google-only account.
        let g = h.m.sign_in_oidc("g-sub", "music").await.unwrap();
        let uid = sub_of(&g.access_token, "music");

        // Submit a password: the branded verification email is sent inline, but
        // NOTHING is bound yet — no credential, no local identity (design A).
        h.m.set_local_credential(&uid, "me@x.dev", PW, "")
            .await
            .unwrap();
        assert_eq!(h.email.sent.lock().unwrap().len(), 1);
        assert!(
            h.creds.get("me@x.dev").await.unwrap().is_none(),
            "email must not be reserved before verification"
        );

        // Sign-in fails as "no such credential" (unauthenticated), not "unverified".
        assert!(matches!(
            h.m.sign_in_local("me@x.dev", PW, "music").await,
            Err(AppError::Unauthenticated(_))
        ));

        // Verifying the emailed code binds the credential (already verified) + the
        // local identity, resolving to the SAME account (no second account).
        let tok = h.pending.only_token();
        h.m.verify_email(&tok).await.unwrap();
        let pair = h.m.sign_in_local("me@x.dev", PW, "music").await.unwrap();
        assert_eq!(sub_of(&pair.access_token, "music"), uid);
    }

    #[tokio::test]
    async fn unconfirmed_set_password_reserves_no_email() {
        let h = harness();
        let g = h.m.sign_in_oidc("g-sub", "music").await.unwrap();
        let uid = sub_of(&g.access_token, "music");
        // Submit a set-password for an email the caller does not own …
        h.m.set_local_credential(&uid, "victim@x.dev", PW, "")
            .await
            .unwrap();
        // … the email is NOT reserved: a fresh sign-up can still register it.
        h.m.sign_up_local("victim@x.dev", PW, "").await.unwrap();
        assert!(h.creds.get("victim@x.dev").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn abandoned_pending_set_password_binds_nothing() {
        let h = harness();
        let g = h.m.sign_in_oidc("g-sub", "music").await.unwrap();
        let uid = sub_of(&g.access_token, "music");
        h.m.set_local_credential(&uid, "me@x.dev", PW, "")
            .await
            .unwrap();
        // Simulate the TTL expiring: drop the pending record without verifying.
        let tok = h.pending.only_token();
        let _ = h.pending.take(&tok).await.unwrap();
        // Verifying the now-expired token binds nothing and reports invalid.
        assert!(matches!(
            h.m.verify_email(&tok).await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(h.creds.get("me@x.dev").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn contended_email_first_to_verify_wins() {
        let h = harness();
        let a = h.m.sign_in_oidc("g-a", "music").await.unwrap();
        let uid_a = sub_of(&a.access_token, "music");
        let b = h.m.sign_in_oidc("g-b", "music").await.unwrap();
        let _uid_b = sub_of(&b.access_token, "music");

        // Both submit a set-password for the same currently-free email.
        h.m.set_local_credential(&uid_a, "shared@x.dev", PW, "")
            .await
            .unwrap();
        let tok_a = h.pending.only_token();
        h.m.set_local_credential(&_uid_b, "shared@x.dev", PW, "")
            .await
            .unwrap();
        let tok_b = h
            .pending
            .tokens()
            .into_iter()
            .find(|t| *t != tok_a)
            .unwrap();

        // A verifies first → binds the email to A; B's later verify fails cleanly.
        h.m.verify_email(&tok_a).await.unwrap();
        assert!(matches!(
            h.m.verify_email(&tok_b).await,
            Err(AppError::AlreadyExists(_))
        ));
        // The email resolves to A's account.
        let pair =
            h.m.sign_in_local("shared@x.dev", PW, "music")
                .await
                .unwrap();
        assert_eq!(sub_of(&pair.access_token, "music"), uid_a);
    }

    #[tokio::test]
    async fn set_local_credential_rejects_second_password_and_weak_password() {
        let h = harness();
        let g = h.m.sign_in_oidc("g-sub", "music").await.unwrap();
        let uid = sub_of(&g.access_token, "music");
        // Bind a first password (submit then verify).
        h.m.set_local_credential(&uid, "me@x.dev", PW, "")
            .await
            .unwrap();
        h.m.verify_email(&h.pending.only_token()).await.unwrap();

        // A second password on the now-bound account is refused up front.
        assert!(matches!(
            h.m.set_local_credential(&uid, "other@x.dev", PW, "").await,
            Err(AppError::AlreadyExists(_))
        ));
        // Weak passwords are rejected up front (nothing parked).
        let g2 = h.m.sign_in_oidc("g-sub-2", "music").await.unwrap();
        let uid2 = sub_of(&g2.access_token, "music");
        assert!(matches!(
            h.m.set_local_credential(&uid2, "n@x.dev", "short", "")
                .await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn set_local_credential_rejects_email_owned_by_another_account() {
        let h = harness();
        // Account A owns me@x.dev via a verified local credential.
        h.m.sign_up_local("me@x.dev", PW, "").await.unwrap();
        let vt = h.creds.peek_verification_token("me@x.dev").unwrap();
        h.m.verify_email(&vt).await.unwrap();

        // Account B (Google) tries to claim the same email — rejected, and the
        // compensating erase must NOT wipe A's credential.
        let g = h.m.sign_in_oidc("g-sub", "music").await.unwrap();
        let uid_b = sub_of(&g.access_token, "music");
        assert!(matches!(
            h.m.set_local_credential(&uid_b, "me@x.dev", PW, "").await,
            Err(AppError::AlreadyExists(_))
        ));
        // A can still sign in — its credential survived the failed claim.
        h.m.sign_in_local("me@x.dev", PW, "music").await.unwrap();
    }

    #[tokio::test]
    async fn link_rejects_bound_elsewhere_and_unlink_guards_last() {
        let h = harness();
        let a = h.m.sign_in_oidc("g1", "music").await.unwrap();
        let uid_a = sub_of(&a.access_token, "music");
        let b = h.m.sign_in_oidc("g2", "music").await.unwrap();
        let uid_b = sub_of(&b.access_token, "music");

        // account B tries to link account A's identity "g1"
        assert!(matches!(
            h.m.link_identity(&uid_b, "g1").await,
            Err(AppError::AlreadyExists(_))
        ));
        // A has a single identity -> cannot unlink it
        assert!(matches!(
            h.m.unlink_identity(&uid_a, "google", "g1").await,
            Err(AppError::FailedPrecondition(_))
        ));
    }

    #[tokio::test]
    async fn resend_verification_sends_branded_multipart_email() {
        let h = harness();
        h.m.sign_up_local("v@x.dev", PW, "").await.unwrap();
        // Sign-up enqueues a job (no direct send yet).
        assert!(h.email.sent.lock().unwrap().is_empty());

        h.m.resend_verification("v@x.dev", "").await.unwrap();
        let sent = h.email.sent.lock().unwrap();
        let msg = sent.first().expect("resend should send one email");
        let code = h.creds.peek_verification_token("v@x.dev").unwrap();
        assert_eq!(msg.to, "v@x.dev");
        assert_eq!(msg.subject, "Verify your Cymbra account");
        // Branded HTML + plain-text alternative, both carrying the code.
        assert!(msg.html.contains("Cymbra ID"));
        assert!(msg.html.contains("#7C3AED"));
        assert!(msg.html.contains(&code));
        assert!(msg.text.contains(&code));
    }

    #[tokio::test]
    async fn password_reset_sends_branded_localized_email() {
        let h = harness();
        h.m.sign_up_local("r@x.dev", PW, "").await.unwrap();
        let vt = h.creds.peek_verification_token("r@x.dev").unwrap();
        h.m.verify_email(&vt).await.unwrap();

        // French locale -> French subject + French legal links.
        h.m.request_password_reset("r@x.dev", "fr-FR")
            .await
            .unwrap();
        let sent = h.email.sent.lock().unwrap();
        let msg = sent.first().expect("reset should send one email");
        let code = h.creds.peek_reset_token("r@x.dev").unwrap();
        assert_eq!(msg.subject, "Réinitialisez votre mot de passe Cymbra");
        assert!(msg.html.contains("https://cymbra.app/cgu/"));
        assert!(msg.html.contains(&code));
        assert!(msg.text.contains(&code));
    }
}
