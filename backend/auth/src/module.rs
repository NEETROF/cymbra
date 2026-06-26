//! The auth module's **direct adapter** (tasks 4.4–4.9): implements [`AuthPort`]
//! over the credential store, the user port, the session store, the email sender,
//! the cache (rate-limit), and an [`OidcVerifier`].

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use cymbra_auth_port::{AuthPort, TokenPair};
use cymbra_platform::cache::Cache;
use cymbra_platform::email::EmailSender;
use cymbra_platform::{AppError, Result, password, ratelimit, token};
use cymbra_user_port::UserPort;
use jsonwebtoken::EncodingKey;

use crate::creds::CredentialRepo;
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
}

/// In-process auth implementation.
pub struct AuthModule {
    user: Arc<dyn UserPort>,
    creds: Arc<dyn CredentialRepo>,
    cache: Arc<dyn Cache>,
    email: Arc<dyn EmailSender>,
    oidc: Arc<dyn OidcVerifier>,
    sessions: SessionStore,
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

impl AuthModule {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user: Arc<dyn UserPort>,
        creds: Arc<dyn CredentialRepo>,
        cache: Arc<dyn Cache>,
        email: Arc<dyn EmailSender>,
        oidc: Arc<dyn OidcVerifier>,
        signing_key_pem: &str,
        kid: &str,
        cfg: AuthConfig,
    ) -> Result<Self> {
        let signing_key = token::encoding_key(signing_key_pem)?;
        let sessions = SessionStore::new(cache.clone(), cfg.refresh_ttl);
        Ok(Self {
            user,
            creds,
            cache,
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
        }
    }
}

#[async_trait]
impl AuthPort for AuthModule {
    async fn sign_up_local(&self, email: &str, password: &str) -> Result<()> {
        password::check_policy(password, self.cfg.password_min_length)?;
        let hash = password::hash(password)?;
        self.creds.insert(email, &hash).await?; // AlreadyExists if taken
        // Provision the shared account + its `local` identity.
        self.user.resolve_or_provision("local", email).await?;
        let tok = uuid::Uuid::new_v4().to_string();
        let exp = now_secs() + self.cfg.verify_ttl.as_secs() as i64;
        self.creds.set_verification(email, &tok, exp).await?;
        self.email
            .send(
                email,
                "Verify your Cymbra account",
                &format!("Confirm your email with this code: {tok}"),
            )
            .await?;
        Ok(())
    }

    async fn verify_email(&self, token: &str) -> Result<()> {
        match self.creds.verify_by_token(token, now_secs()).await? {
            Some(_) => Ok(()),
            None => Err(AppError::InvalidArgument(
                "invalid or expired verification token".into(),
            )),
        }
    }

    async fn resend_verification(&self, email: &str) -> Result<()> {
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
            self.email
                .send(
                    email,
                    "Verify your Cymbra account",
                    &format!("Confirm your email with this code: {tok}"),
                )
                .await?;
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

    async fn request_password_reset(&self, email: &str) -> Result<()> {
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
            self.email
                .send(
                    email,
                    "Reset your Cymbra password",
                    &format!("Reset your password with this code: {tok}"),
                )
                .await?;
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
    }

    fn harness() -> Harness {
        let user: Arc<dyn UserPort> = Arc::new(UserModule::new(FakeUserRepo::default()));
        let creds = Arc::new(FakeCredentialRepo::default());
        let cache: Arc<dyn Cache> = Arc::new(FakeCache::default());
        let email: Arc<dyn EmailSender> = Arc::new(FakeEmail::default());
        let oidc = Arc::new(FakeOidcVerifier::default());
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
        );
        let m = AuthModule::new(user, creds.clone(), cache, email, oidc, PRIV, "k1", cfg).unwrap();
        Harness { m, creds }
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
        h.m.sign_up_local("a@x.dev", PW).await.unwrap();
        let tok = h.creds.peek_verification_token("a@x.dev").unwrap();
        h.m.verify_email(&tok).await.unwrap();
        let pair = h.m.sign_in_local("a@x.dev", PW, "music").await.unwrap();
        let claims = ptoken::verify(&pair.access_token, &keys(), &["music"]).unwrap();
        assert_eq!(claims.aud, "music");
        assert!(claims.roles.contains(&"user".to_string()));
    }

    #[tokio::test]
    async fn duplicate_and_weak_password() {
        let h = harness();
        h.m.sign_up_local("a@x.dev", PW).await.unwrap();
        assert!(matches!(
            h.m.sign_up_local("a@x.dev", PW).await,
            Err(AppError::AlreadyExists(_))
        ));
        assert!(matches!(
            h.m.sign_up_local("b@x.dev", "short").await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn unverified_blocked_then_wrong_password_then_lockout() {
        let h = harness();
        h.m.sign_up_local("a@x.dev", PW).await.unwrap();
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
        h.m.sign_up_local("b@x.dev", PW).await.unwrap();
        let vt = h.creds.peek_verification_token("b@x.dev").unwrap();
        h.m.verify_email(&vt).await.unwrap();
        let pair = h.m.sign_in_local("b@x.dev", PW, "music").await.unwrap();

        h.m.request_password_reset("b@x.dev").await.unwrap();
        let rt = h.creds.peek_reset_token("b@x.dev").unwrap();
        h.m.reset_password(&rt, "a-new-strong-pass").await.unwrap();

        // old session revoked
        assert!(matches!(
            h.m.refresh(&pair.refresh_token).await,
            Err(AppError::Unauthenticated(_))
        ));
        // new password works
        h.m.sign_in_local("b@x.dev", "a-new-strong-pass", "music")
            .await
            .unwrap();
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
}
