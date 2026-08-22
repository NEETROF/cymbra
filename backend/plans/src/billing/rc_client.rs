//! RevenueCat v1 customer API client — `GET /v1/subscribers/{app_user_id}` and
//! `DELETE /v1/subscribers/{app_user_id}` with the secret API key. Thin
//! reqwest glue (coverage-excluded); the JSON → [`StoreSubscription`] projection
//! it feeds is in [`crate::billing::revenuecat`] and tested there.

use crate::billing::revenuecat::{SubscriberResponse, project_subscriber};
use crate::ports::{StoreCustomerEraser, StoreCustomerSource, StoreSubscription};
use async_trait::async_trait;
use cymbra_platform::{AppError, Result};

pub struct RcClient {
    http: reqwest::Client,
    base: String,
    api_key: String,
}

impl RcClient {
    pub fn new(api_key: String) -> Self {
        Self::with_base(api_key, "https://api.revenuecat.com/v1".into())
    }

    pub fn with_base(api_key: String, base: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base,
            api_key,
        }
    }
}

#[async_trait]
impl StoreCustomerSource for RcClient {
    async fn subscriptions(&self, user_id: &str) -> Result<Vec<StoreSubscription>> {
        let url = format!("{}/subscribers/{}", self.base, path_segment(user_id));
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("revenuecat get subscriber: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "revenuecat get subscriber: HTTP {}",
                resp.status()
            )));
        }
        let body: SubscriberResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("revenuecat subscriber body: {e}")))?;
        Ok(project_subscriber(&body.subscriber))
    }
}

#[async_trait]
impl StoreCustomerEraser for RcClient {
    async fn delete_customer(&self, user_id: &str) -> Result<()> {
        let url = format!("{}/subscribers/{}", self.base, path_segment(user_id));
        let resp = self
            .http
            .delete(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("revenuecat delete subscriber: {e}"))
            })?;
        // 404 = already gone: erasure is idempotent.
        if resp.status().is_success() || resp.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(())
        } else {
            Err(AppError::Internal(anyhow::anyhow!(
                "revenuecat delete subscriber: HTTP {}",
                resp.status()
            )))
        }
    }
}

/// Percent-encode a path segment (account ids are UUIDs; anything else is
/// escaped rather than trusted).
fn path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
