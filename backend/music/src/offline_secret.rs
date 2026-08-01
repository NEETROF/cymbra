// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Per-user offline-cache secret: the `offline_cache_secrets` data-access port
//! (change: add-offline-score-cache, spec `backend-offline-key`).
//!
//! The secret is a high-entropy value (>= 32 bytes) the app uses as ONE input to
//! its local offline-cache key derivation. It is **created on first request and
//! returned unchanged thereafter** (so the same favorites decrypt across a user's
//! devices), and **rotated** on account deletion so residual cache files become
//! undecryptable. Every method is owner-scoped by `user_id`; there is no
//! cross-user read path at the data layer, and the value is never logged.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_platform::Result;

/// Length of a freshly generated offline-cache secret (bytes). The spec requires
/// at least 32 bytes of entropy; 32 CSPRNG bytes is the whole value.
pub const OFFLINE_SECRET_LEN: usize = 32;

/// Generate a fresh high-entropy offline-cache secret via the OS CSPRNG.
///
/// Kept as a free function (not a store method) so the store trait stays pure I/O
/// and the generation policy is unit-testable on its own.
pub fn generate_offline_secret() -> Vec<u8> {
    let mut buf = vec![0u8; OFFLINE_SECRET_LEN];
    // `getrandom` reads the OS CSPRNG; a failure here means the platform has no
    // entropy source, which is unrecoverable — surface it rather than emit a weak
    // secret.
    getrandom::getrandom(&mut buf).expect("OS CSPRNG unavailable for offline secret");
    buf
}

/// Owner-scoped storage surface for the per-user offline-cache secret.
///
/// The value is sensitive: implementations MUST NOT log it and callers only ever
/// receive their own (the `user_id` argument is the authenticated caller).
#[async_trait]
pub trait OfflineSecretRepo: Send + Sync {
    /// The user's stored secret, or `None` if none has been created yet.
    async fn get(&self, user_id: &str) -> Result<Option<Vec<u8>>>;

    /// Atomically get-or-create: store `candidate` for `user_id` only if no secret
    /// exists yet, then return the value now stored. On a concurrent race the
    /// **existing** value wins (so every device converges on one secret) — the
    /// candidate is discarded. This is the create-on-first-request primitive.
    async fn create_if_absent(&self, user_id: &str, candidate: &[u8]) -> Result<Vec<u8>>;

    /// Replace the user's secret with `secret` (insert or overwrite). Used by the
    /// rotation / kill-switch lever; after this the next `get` returns the new value
    /// and prior offline caches stop decrypting once devices re-derive.
    async fn rotate(&self, user_id: &str, secret: &[u8]) -> Result<()>;
}

/// In-memory [`OfflineSecretRepo`] for unit tests.
#[derive(Default)]
pub struct FakeOfflineSecretRepo {
    secrets: Mutex<HashMap<String, Vec<u8>>>,
}

impl FakeOfflineSecretRepo {
    /// The raw stored secret for a user (test introspection).
    pub fn stored(&self, user_id: &str) -> Option<Vec<u8>> {
        self.secrets
            .lock()
            .expect("offline secret fake lock")
            .get(user_id)
            .cloned()
    }
}

#[async_trait]
impl OfflineSecretRepo for FakeOfflineSecretRepo {
    async fn get(&self, user_id: &str) -> Result<Option<Vec<u8>>> {
        Ok(self
            .secrets
            .lock()
            .expect("offline secret fake lock")
            .get(user_id)
            .cloned())
    }

    async fn create_if_absent(&self, user_id: &str, candidate: &[u8]) -> Result<Vec<u8>> {
        let mut map = self.secrets.lock().expect("offline secret fake lock");
        let stored = map
            .entry(user_id.to_string())
            .or_insert_with(|| candidate.to_vec());
        Ok(stored.clone())
    }

    async fn rotate(&self, user_id: &str, secret: &[u8]) -> Result<()> {
        self.secrets
            .lock()
            .expect("offline secret fake lock")
            .insert(user_id.to_string(), secret.to_vec());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_secret_has_full_entropy_length_and_varies() {
        let a = generate_offline_secret();
        let b = generate_offline_secret();
        assert_eq!(a.len(), OFFLINE_SECRET_LEN);
        assert!(a.len() >= 32, "secret must be at least 32 bytes");
        // Two draws colliding would be a broken RNG (2^-256).
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn create_if_absent_is_get_or_create() {
        let r = FakeOfflineSecretRepo::default();
        assert!(r.get("u1").await.unwrap().is_none());
        let first = r
            .create_if_absent("u1", b"first-secret-value")
            .await
            .unwrap();
        assert_eq!(first, b"first-secret-value");
        // A second create returns the EXISTING value, never the new candidate.
        let again = r
            .create_if_absent("u1", b"different-candidate")
            .await
            .unwrap();
        assert_eq!(again, b"first-secret-value");
        assert_eq!(r.get("u1").await.unwrap().unwrap(), b"first-secret-value");
    }

    #[tokio::test]
    async fn rotate_replaces_and_is_owner_scoped() {
        let r = FakeOfflineSecretRepo::default();
        r.create_if_absent("u1", b"old").await.unwrap();
        r.create_if_absent("u2", b"other").await.unwrap();
        r.rotate("u1", b"new").await.unwrap();
        assert_eq!(r.get("u1").await.unwrap().unwrap(), b"new");
        // Rotation is scoped to the one user; u2 is untouched.
        assert_eq!(r.get("u2").await.unwrap().unwrap(), b"other");
    }
}
