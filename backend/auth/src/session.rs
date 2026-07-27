//! Refresh-token sessions with **rotation + reuse detection** (change:
//! durable-sessions-postgres).
//!
//! A session is a *family* identified by a UUIDv7 `id`. The refresh token handed
//! to a client encodes that id plus a random secret: `"{id}.{secret}"`. Only the
//! SHA-256 hash of the whole token is persisted, so a replayed rotated token still
//! resolves to its family (the id is in the token) while a DB dump leaks no usable
//! token. Rotation issues a new secret; presenting a token whose hash no longer
//! matches the family's `current_rt_hash` is treated as theft and revokes the
//! whole family. Sessions are **audience-bound** (one login per app).
//!
//! [`SessionStore`] is the storage-agnostic seam: [`crate::PgSessionStore`] is the
//! durable Postgres impl; [`FakeSessionStore`] keeps the module unit-testable with
//! no DB. The pure token logic lives in [`session_core`] (host-tested).

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// A rotated session: the new refresh token + the (audience-bound) identity.
pub struct Rotated {
    pub refresh_token: String,
    pub user_id: String,
    pub audience: String,
}

/// Summary of a session family (revoke-all / active-devices listing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub id: String,
    pub audience: String,
}

/// Storage-agnostic refresh-token session store. Consumers depend on this trait;
/// the durable impl is `PgSessionStore`, tests use [`FakeSessionStore`].
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Start a session for `(user_id, audience)`; returns the refresh token.
    async fn create(&self, user_id: &str, audience: &str) -> Result<String>;
    /// Rotate a refresh token; rejects invalid/expired tokens and revokes the
    /// family on replay of an already-rotated token (reuse/theft detection).
    async fn rotate(&self, refresh_token: &str) -> Result<Rotated>;
    /// Revoke the session that owns `refresh_token` (logout).
    async fn revoke(&self, refresh_token: &str) -> Result<()>;
    /// Revoke the session `session_id` **only if it belongs to `user_id`** — so a
    /// caller can end one of their own devices/sessions by id without holding its
    /// refresh token. A foreign, absent, expired, or malformed id is a successful
    /// no-op (it neither errors nor reveals another account's data).
    async fn revoke_by_id(&self, user_id: &str, session_id: &str) -> Result<()>;
    /// Revoke every session for `user_id` — `DELETE FROM sessions WHERE user_id
    /// = $1`. This is the auth module's **erasure path** for account deletion
    /// (it removes all refresh tokens for the user) as well as the
    /// password-reset revoke-all. Idempotent: revoking for a user with no live
    /// sessions is a successful no-op, so a retried deletion converges.
    async fn revoke_all(&self, user_id: &str) -> Result<()>;
    /// The account's non-expired session families.
    async fn list_for_user(&self, user_id: &str) -> Result<Vec<SessionInfo>>;
}

/// Pure, host-testable token logic: encode/parse the `"{id}.{secret}"` refresh
/// token, hash it for at-rest storage, and mint fresh ids/secrets.
pub mod session_core {
    use super::{AppError, Result, Uuid};
    use sha2::{Digest, Sha256};

    /// Encode a refresh token as `"{id}.{secret}"`.
    pub fn encode_token(id: Uuid, secret: &str) -> String {
        format!("{id}.{secret}")
    }

    /// Extract the family id from a refresh token. A malformed token is rejected
    /// as `Unauthenticated` (a garbage token simply fails, not errors).
    pub fn parse_id(token: &str) -> Result<Uuid> {
        let (id, _secret) = token
            .split_once('.')
            .ok_or_else(|| AppError::Unauthenticated("invalid refresh token".into()))?;
        Uuid::parse_str(id).map_err(|_| AppError::Unauthenticated("invalid refresh token".into()))
    }

    /// SHA-256 (lowercase hex) of the refresh token — what is stored at rest.
    pub fn hash_token(token: &str) -> String {
        Sha256::digest(token.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// A new session-family id (time-ordered UUIDv7).
    pub fn new_id() -> Uuid {
        Uuid::now_v7()
    }

    /// A new random token secret.
    pub fn new_secret() -> String {
        Uuid::new_v4().simple().to_string()
    }
}

/// In-memory [`SessionStore`] for unit tests (TTL/expiry ignored). Mirrors the
/// rotation + theft-detection semantics of the durable store.
#[derive(Default)]
pub struct FakeSessionStore {
    fams: Mutex<HashMap<Uuid, FakeFam>>,
}

struct FakeFam {
    user_id: String,
    audience: String,
    current_rt_hash: String,
}

#[async_trait]
impl SessionStore for FakeSessionStore {
    async fn create(&self, user_id: &str, audience: &str) -> Result<String> {
        let id = session_core::new_id();
        let token = session_core::encode_token(id, &session_core::new_secret());
        self.fams.lock().unwrap().insert(
            id,
            FakeFam {
                user_id: user_id.into(),
                audience: audience.into(),
                current_rt_hash: session_core::hash_token(&token),
            },
        );
        Ok(token)
    }

    async fn rotate(&self, refresh_token: &str) -> Result<Rotated> {
        let id = session_core::parse_id(refresh_token)?;
        let mut fams = self.fams.lock().unwrap();
        match fams.get_mut(&id) {
            None => Err(AppError::Unauthenticated("invalid refresh token".into())),
            Some(fam) if fam.current_rt_hash == session_core::hash_token(refresh_token) => {
                let new_token = session_core::encode_token(id, &session_core::new_secret());
                fam.current_rt_hash = session_core::hash_token(&new_token);
                Ok(Rotated {
                    refresh_token: new_token,
                    user_id: fam.user_id.clone(),
                    audience: fam.audience.clone(),
                })
            }
            // A rotated token replayed → theft. Kill the whole family.
            Some(_) => {
                fams.remove(&id);
                Err(AppError::Unauthenticated(
                    "refresh token reuse detected".into(),
                ))
            }
        }
    }

    async fn revoke(&self, refresh_token: &str) -> Result<()> {
        if let Ok(id) = session_core::parse_id(refresh_token) {
            self.fams.lock().unwrap().remove(&id);
        }
        Ok(())
    }

    async fn revoke_by_id(&self, user_id: &str, session_id: &str) -> Result<()> {
        if let Ok(id) = Uuid::parse_str(session_id) {
            let mut fams = self.fams.lock().unwrap();
            // Scoped to the owner: a foreign/absent id removes nothing.
            if fams.get(&id).is_some_and(|f| f.user_id == user_id) {
                fams.remove(&id);
            }
        }
        Ok(())
    }

    async fn revoke_all(&self, user_id: &str) -> Result<()> {
        self.fams
            .lock()
            .unwrap()
            .retain(|_, f| f.user_id != user_id);
        Ok(())
    }

    async fn list_for_user(&self, user_id: &str) -> Result<Vec<SessionInfo>> {
        Ok(self
            .fams
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, f)| f.user_id == user_id)
            .map(|(id, f)| SessionInfo {
                id: id.to_string(),
                audience: f.audience.clone(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_round_trips_and_hash_is_stable() {
        let id = session_core::new_id();
        let token = session_core::encode_token(id, "s3cr3t");
        assert_eq!(token, format!("{id}.s3cr3t"));
        assert_eq!(session_core::parse_id(&token).unwrap(), id);
        // Hash is deterministic and 64 hex chars (SHA-256).
        let h = session_core::hash_token(&token);
        assert_eq!(h, session_core::hash_token(&token));
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn parse_rejects_malformed_tokens() {
        assert!(session_core::parse_id("no-dot").is_err());
        assert!(session_core::parse_id("not-a-uuid.secret").is_err());
        assert!(matches!(
            session_core::parse_id("garbage"),
            Err(AppError::Unauthenticated(_))
        ));
    }

    #[test]
    fn fresh_ids_and_secrets_differ() {
        assert_ne!(session_core::new_id(), session_core::new_id());
        assert_ne!(session_core::new_secret(), session_core::new_secret());
    }

    #[tokio::test]
    async fn fake_rotate_then_reuse_revokes_family() {
        let s = FakeSessionStore::default();
        let rt = s.create("u1", "music").await.unwrap();
        let rot = s.rotate(&rt).await.unwrap();
        assert_eq!(rot.user_id, "u1");
        assert_eq!(rot.audience, "music");
        // Replay the original (now rotated) token → reuse detected.
        assert!(matches!(
            s.rotate(&rt).await,
            Err(AppError::Unauthenticated(_))
        ));
        // The family is revoked, so the rotated token is dead too.
        assert!(matches!(
            s.rotate(&rot.refresh_token).await,
            Err(AppError::Unauthenticated(_))
        ));
    }

    #[tokio::test]
    async fn fake_revoke_and_revoke_all_and_list() {
        let s = FakeSessionStore::default();
        let a = s.create("u1", "music").await.unwrap();
        let _b = s.create("u1", "live").await.unwrap();
        let c = s.create("u2", "music").await.unwrap();

        assert_eq!(s.list_for_user("u1").await.unwrap().len(), 2);

        // logout one session
        s.revoke(&a).await.unwrap();
        assert!(matches!(
            s.rotate(&a).await,
            Err(AppError::Unauthenticated(_))
        ));
        assert_eq!(s.list_for_user("u1").await.unwrap().len(), 1);

        // revoke-all for u1 leaves u2 untouched
        s.revoke_all("u1").await.unwrap();
        assert!(s.list_for_user("u1").await.unwrap().is_empty());
        assert!(s.rotate(&c).await.is_ok());

        // revoking a malformed token is a no-op (no panic/err)
        s.revoke("garbage").await.unwrap();
    }

    #[tokio::test]
    async fn fake_revoke_by_id_is_owner_scoped() {
        let s = FakeSessionStore::default();
        let a = s.create("u1", "music").await.unwrap();
        let b = s.create("u1", "live").await.unwrap();
        let c = s.create("u2", "music").await.unwrap();
        let id_a = session_core::parse_id(&a).unwrap().to_string();

        // A different account cannot revoke u1's session (owner-scoped) → no-op.
        s.revoke_by_id("u2", &id_a).await.unwrap();
        assert_eq!(s.list_for_user("u1").await.unwrap().len(), 2);

        // The owner revokes that session by id: it ends, the other survives.
        s.revoke_by_id("u1", &id_a).await.unwrap();
        assert!(matches!(
            s.rotate(&a).await,
            Err(AppError::Unauthenticated(_))
        ));
        assert!(s.rotate(&b).await.is_ok());

        // A malformed or unknown id is a successful no-op; u2 is unaffected.
        s.revoke_by_id("u1", "not-a-uuid").await.unwrap();
        s.revoke_by_id("u1", &Uuid::new_v4().to_string())
            .await
            .unwrap();
        assert!(s.rotate(&c).await.is_ok());
    }

    #[tokio::test]
    async fn revoke_all_for_user_without_sessions_is_noop_success() {
        // Erasure path (account deletion): revoking a user that has no sessions —
        // an OIDC-only account, or a retried deletion — succeeds as a no-op.
        let s = FakeSessionStore::default();
        s.revoke_all("never-had-a-session").await.unwrap();
        // And after a real revoke-all, re-running still succeeds (idempotent).
        let rt = s.create("u1", "music").await.unwrap();
        s.revoke_all("u1").await.unwrap();
        s.revoke_all("u1").await.unwrap();
        assert!(matches!(
            s.rotate(&rt).await,
            Err(AppError::Unauthenticated(_))
        ));
    }
}
