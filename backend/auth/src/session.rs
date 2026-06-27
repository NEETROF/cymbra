//! Refresh-token sessions in Redis (task 4.6) with **rotation + reuse detection**.
//!
//! A session is a *family*: `sess:fam:{fid}` holds `{user_id, audience,
//! current_rt}`; `sess:rt:{rt}` maps a refresh token to its family. Rotation
//! issues a new `current_rt`; presenting a previously-rotated token (whose
//! mapping still exists but no longer equals `current_rt`) is treated as theft
//! and revokes the whole family. Sessions are **audience-bound** (one login per
//! app).

use cymbra_platform::cache::Cache;
use cymbra_platform::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct Family {
    user_id: String,
    audience: String,
    current_rt: String,
}

/// A rotated session: the new refresh token + the (audience-bound) identity.
pub struct Rotated {
    pub refresh_token: String,
    pub user_id: String,
    pub audience: String,
}

pub struct SessionStore {
    cache: Arc<dyn Cache>,
    ttl: Duration,
}

impl SessionStore {
    pub fn new(cache: Arc<dyn Cache>, ttl: Duration) -> Self {
        Self { cache, ttl }
    }

    fn fam_key(fid: &str) -> String {
        format!("sess:fam:{fid}")
    }
    fn rt_key(rt: &str) -> String {
        format!("sess:rt:{rt}")
    }
    fn userfam_key(uid: &str) -> String {
        format!("sess:userfam:{uid}")
    }

    fn encode(fam: &Family) -> String {
        serde_json::to_string(fam).expect("family serializes")
    }

    /// Start a session for `(user_id, audience)`; returns the refresh token.
    pub async fn create(&self, user_id: &str, audience: &str) -> Result<String> {
        let fid = uuid::Uuid::now_v7().to_string();
        let rt = uuid::Uuid::new_v4().to_string();
        let fam = Family {
            user_id: user_id.into(),
            audience: audience.into(),
            current_rt: rt.clone(),
        };
        self.cache
            .set_ex(&Self::fam_key(&fid), &Self::encode(&fam), self.ttl)
            .await?;
        self.cache
            .set_ex(&Self::rt_key(&rt), &fid, self.ttl)
            .await?;
        // Track the family under the user for revoke-all.
        let ukey = Self::userfam_key(user_id);
        let mut list = self.cache.get(&ukey).await?.unwrap_or_default();
        if !list.is_empty() {
            list.push(',');
        }
        list.push_str(&fid);
        self.cache.set_ex(&ukey, &list, self.ttl).await?;
        Ok(rt)
    }

    /// Rotate a refresh token; detects reuse (revoking the family).
    pub async fn rotate(&self, rt: &str) -> Result<Rotated> {
        let fid = self
            .cache
            .get(&Self::rt_key(rt))
            .await?
            .ok_or_else(|| AppError::Unauthenticated("invalid refresh token".into()))?;
        let fam_json = self
            .cache
            .get(&Self::fam_key(&fid))
            .await?
            .ok_or_else(|| AppError::Unauthenticated("session revoked".into()))?;
        let mut fam: Family = serde_json::from_str(&fam_json)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("session parse: {e}")))?;

        if fam.current_rt != rt {
            // A rotated token replayed → theft. Kill the whole family.
            self.cache.del(&Self::fam_key(&fid)).await?;
            return Err(AppError::Unauthenticated(
                "refresh token reuse detected".into(),
            ));
        }

        let new_rt = uuid::Uuid::new_v4().to_string();
        fam.current_rt = new_rt.clone();
        let out = Rotated {
            refresh_token: new_rt.clone(),
            user_id: fam.user_id.clone(),
            audience: fam.audience.clone(),
        };
        self.cache
            .set_ex(&Self::fam_key(&fid), &Self::encode(&fam), self.ttl)
            .await?;
        self.cache
            .set_ex(&Self::rt_key(&new_rt), &fid, self.ttl)
            .await?;
        Ok(out)
    }

    /// Revoke the session that owns `rt` (logout).
    pub async fn revoke(&self, rt: &str) -> Result<()> {
        if let Some(fid) = self.cache.get(&Self::rt_key(rt)).await? {
            self.cache.del(&Self::fam_key(&fid)).await?;
        }
        Ok(())
    }

    /// Revoke every session for `user_id` (e.g. after a password reset).
    pub async fn revoke_all(&self, user_id: &str) -> Result<()> {
        let ukey = Self::userfam_key(user_id);
        if let Some(list) = self.cache.get(&ukey).await? {
            for fid in list.split(',').filter(|s| !s.is_empty()) {
                self.cache.del(&Self::fam_key(fid)).await?;
            }
            self.cache.del(&ukey).await?;
        }
        Ok(())
    }
}
