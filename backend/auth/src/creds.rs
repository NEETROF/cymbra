//! Local-credential store (task 4.2): the email/password secret + single-use
//! verification/reset tokens. A [`FakeCredentialRepo`] keeps the module testable.

use async_trait::async_trait;
use cymbra_jobs::EnqueueRequest;
use cymbra_platform::{AppError, Result};
use std::collections::HashMap;
use std::sync::Mutex;

/// A local credential record.
#[derive(Debug, Clone)]
pub struct Credential {
    pub email: String,
    pub password_hash: String,
    pub email_verified: bool,
}

#[async_trait]
pub trait CredentialRepo: Send + Sync {
    /// Create a credential (unverified). `AlreadyExists` if the email is taken.
    async fn insert(&self, email: &str, password_hash: &str) -> Result<()>;
    async fn get(&self, email: &str) -> Result<Option<Credential>>;
    /// Store a single-use verification token + expiry (unix seconds).
    async fn set_verification(&self, email: &str, token: &str, expires_at: i64) -> Result<()>;
    /// Store the verification token **and** enqueue `job` atomically in one
    /// transaction (transactional enqueue, design D1): the email job exists iff
    /// the verification write commits. Used by sign-up so SMTP is off the request
    /// path and an email failure can never fail an account that was created.
    async fn set_verification_with_job(
        &self,
        email: &str,
        token: &str,
        expires_at: i64,
        job: &EnqueueRequest,
    ) -> Result<()>;
    /// Consume a valid verification token: mark verified, clear it; return the email.
    async fn verify_by_token(&self, token: &str, now: i64) -> Result<Option<String>>;
    async fn set_reset(&self, email: &str, token: &str, expires_at: i64) -> Result<()>;
    /// Consume a valid reset token: set the new hash, clear it; return the email.
    async fn reset_by_token(&self, token: &str, new_hash: &str, now: i64)
    -> Result<Option<String>>;
    /// Erase the credential for `email` (GDPR account deletion / complete erasure).
    /// Idempotent: deleting an absent email is a successful no-op, so a retried
    /// deletion converges. Nothing that could re-identify the user (password hash,
    /// verification/reset token) survives.
    async fn delete_credentials(&self, email: &str) -> Result<()>;
}

#[derive(Clone, Default)]
struct Row {
    hash: String,
    verified: bool,
    verify_token: Option<(String, i64)>,
    reset_token: Option<(String, i64)>,
}

/// In-memory [`CredentialRepo`] for unit tests.
#[derive(Default)]
pub struct FakeCredentialRepo {
    rows: Mutex<HashMap<String, Row>>,
    enqueued: Mutex<Vec<EnqueueRequest>>,
}

impl FakeCredentialRepo {
    /// Test helper: jobs enqueued via [`CredentialRepo::set_verification_with_job`].
    pub fn enqueued_jobs(&self) -> Vec<EnqueueRequest> {
        self.enqueued.lock().unwrap().clone()
    }

    /// Test helper: read the pending verification token for `email`.
    pub fn peek_verification_token(&self, email: &str) -> Option<String> {
        self.rows
            .lock()
            .unwrap()
            .get(email)
            .and_then(|r| r.verify_token.as_ref().map(|(t, _)| t.clone()))
    }

    /// Test helper: read the pending reset token for `email`.
    pub fn peek_reset_token(&self, email: &str) -> Option<String> {
        self.rows
            .lock()
            .unwrap()
            .get(email)
            .and_then(|r| r.reset_token.as_ref().map(|(t, _)| t.clone()))
    }
}

#[async_trait]
impl CredentialRepo for FakeCredentialRepo {
    async fn insert(&self, email: &str, password_hash: &str) -> Result<()> {
        let mut rows = self.rows.lock().unwrap();
        if rows.contains_key(email) {
            return Err(AppError::AlreadyExists("email already registered".into()));
        }
        rows.insert(
            email.into(),
            Row {
                hash: password_hash.into(),
                ..Default::default()
            },
        );
        Ok(())
    }

    async fn get(&self, email: &str) -> Result<Option<Credential>> {
        Ok(self.rows.lock().unwrap().get(email).map(|r| Credential {
            email: email.into(),
            password_hash: r.hash.clone(),
            email_verified: r.verified,
        }))
    }

    async fn set_verification(&self, email: &str, token: &str, expires_at: i64) -> Result<()> {
        let mut rows = self.rows.lock().unwrap();
        let row = rows
            .get_mut(email)
            .ok_or_else(|| AppError::NotFound("credential".into()))?;
        row.verify_token = Some((token.into(), expires_at));
        Ok(())
    }

    async fn set_verification_with_job(
        &self,
        email: &str,
        token: &str,
        expires_at: i64,
        job: &EnqueueRequest,
    ) -> Result<()> {
        // Records the write and the enqueue together, mirroring the Pg impl's
        // single transaction.
        self.set_verification(email, token, expires_at).await?;
        self.enqueued.lock().unwrap().push(job.clone());
        Ok(())
    }

    async fn verify_by_token(&self, token: &str, now: i64) -> Result<Option<String>> {
        let mut rows = self.rows.lock().unwrap();
        for (email, row) in rows.iter_mut() {
            if let Some((t, exp)) = &row.verify_token
                && t == token
                && *exp > now
            {
                row.verified = true;
                row.verify_token = None;
                return Ok(Some(email.clone()));
            }
        }
        Ok(None)
    }

    async fn set_reset(&self, email: &str, token: &str, expires_at: i64) -> Result<()> {
        let mut rows = self.rows.lock().unwrap();
        let row = rows
            .get_mut(email)
            .ok_or_else(|| AppError::NotFound("credential".into()))?;
        row.reset_token = Some((token.into(), expires_at));
        Ok(())
    }

    async fn reset_by_token(
        &self,
        token: &str,
        new_hash: &str,
        now: i64,
    ) -> Result<Option<String>> {
        let mut rows = self.rows.lock().unwrap();
        for (email, row) in rows.iter_mut() {
            if let Some((t, exp)) = &row.reset_token
                && t == token
                && *exp > now
            {
                row.hash = new_hash.into();
                row.reset_token = None;
                return Ok(Some(email.clone()));
            }
        }
        Ok(None)
    }

    async fn delete_credentials(&self, email: &str) -> Result<()> {
        // Idempotent: `remove` on an absent key is a no-op.
        self.rows.lock().unwrap().remove(email);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn delete_credentials_erases_then_frees_the_email() {
        let repo = FakeCredentialRepo::default();
        repo.insert("a@x.dev", "hash").await.unwrap();
        assert!(repo.get("a@x.dev").await.unwrap().is_some());

        repo.delete_credentials("a@x.dev").await.unwrap();
        // The row is gone …
        assert!(repo.get("a@x.dev").await.unwrap().is_none());
        // … and the email is registrable again (no AlreadyExists).
        repo.insert("a@x.dev", "hash2").await.unwrap();
    }

    #[tokio::test]
    async fn delete_credentials_absent_email_is_noop_success() {
        let repo = FakeCredentialRepo::default();
        // Deleting a never-seen email succeeds (idempotent erasure).
        repo.delete_credentials("nobody@x.dev").await.unwrap();
        // Re-running after a real delete also succeeds.
        repo.insert("a@x.dev", "hash").await.unwrap();
        repo.delete_credentials("a@x.dev").await.unwrap();
        repo.delete_credentials("a@x.dev").await.unwrap();
    }
}
