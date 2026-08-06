//! FCM HTTP v1 [`PushSender`] (change: add-push-notifications, task 2.2, design
//! D1) — coverage-excluded glue: the only code here that is not a trait call is
//! the HTTP conversation with Google.
//!
//! One provider reaches all three supported platforms: Android natively, and the
//! Apple ones (iOS, macOS) by FCM bridging to APNs with the auth key uploaded to
//! the Firebase project. The client therefore reports **FCM tokens** everywhere
//! and the server stores one token shape.
//!
//! Auth is service-account OAuth: a short-lived RS256 JWT assertion is exchanged
//! for an access token at Google's token endpoint, cached until shortly before it
//! expires. The classification of a send response into
//! delivered / retryable / invalid is in [`classify`], which *is* covered.

use std::sync::Arc;

use anyhow::{Context, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::sender::{PushMessage, PushSender, SendOutcome};

const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const FCM_SCOPE: &str = "https://www.googleapis.com/auth/firebase.messaging";
/// Renew the access token this long before it actually expires.
const RENEW_MARGIN_SECS: i64 = 120;

/// The Firebase service-account fields the sender needs. Parsed from the JSON key
/// file Google issues (`CYMBRA_FCM_SERVICE_ACCOUNT_JSON`).
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccount {
    pub project_id: String,
    pub client_email: String,
    pub private_key: String,
}

impl ServiceAccount {
    /// Parse a service-account key file's JSON.
    pub fn from_json(raw: &str) -> anyhow::Result<Self> {
        serde_json::from_str(raw).context("invalid FCM service-account JSON")
    }
}

/// Classify an FCM send response into the three outcomes callers act on.
///
/// FCM reports a dead token as `404 NOT_FOUND` (`UNREGISTERED`) or, for a
/// malformed one, `400 INVALID_ARGUMENT`; `401/403` mean *our* credentials are
/// wrong, which is not the device's fault, so those are retryable rather than a
/// reason to delete a user's token. Everything else transient (429, 5xx,
/// network) is retryable too.
pub fn classify(status: u16, body: &str) -> SendOutcome {
    match status {
        200..=299 => SendOutcome::Delivered,
        // A dead registration, or a token FCM will never accept: prune it.
        404 => SendOutcome::Invalid(format!("UNREGISTERED ({body})")),
        400 if body.contains("UNREGISTERED") || body.contains("INVALID_ARGUMENT") => {
            SendOutcome::Invalid(format!("INVALID_ARGUMENT ({body})"))
        }
        other => SendOutcome::Retryable(format!("fcm http {other} ({body})")),
    }
}

/// Build the FCM HTTP v1 `message` body for one token.
pub fn message_body(token: &str, msg: &PushMessage) -> serde_json::Value {
    let data: serde_json::Map<String, serde_json::Value> = msg
        .data
        .iter()
        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        .collect();
    serde_json::json!({
        "message": {
            "token": token,
            "notification": { "title": msg.title, "body": msg.body },
            "data": data,
        }
    })
}

#[derive(Clone)]
struct CachedToken {
    value: String,
    expires_at: i64,
}

/// FCM HTTP v1 sender.
pub struct FcmSender {
    account: ServiceAccount,
    http: reqwest::Client,
    cached: Arc<Mutex<Option<CachedToken>>>,
}

impl FcmSender {
    pub fn new(account: ServiceAccount) -> Self {
        Self {
            account,
            http: reqwest::Client::new(),
            cached: Arc::new(Mutex::new(None)),
        }
    }

    /// Build the sender from the raw service-account JSON.
    pub fn from_service_account_json(raw: &str) -> anyhow::Result<Self> {
        Ok(Self::new(ServiceAccount::from_json(raw)?))
    }

    fn send_url(&self) -> String {
        format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.account.project_id
        )
    }

    /// A valid OAuth access token, minted (and cached) on demand.
    async fn access_token(&self) -> anyhow::Result<String> {
        let now = chrono::Utc::now().timestamp();
        {
            let cached = self.cached.lock().await;
            if let Some(t) = cached.as_ref()
                && t.expires_at - RENEW_MARGIN_SECS > now
            {
                return Ok(t.value.clone());
            }
        }

        #[derive(serde::Serialize)]
        struct Claims<'a> {
            iss: &'a str,
            scope: &'a str,
            aud: &'a str,
            iat: i64,
            exp: i64,
        }
        let claims = Claims {
            iss: &self.account.client_email,
            scope: FCM_SCOPE,
            aud: TOKEN_URL,
            iat: now,
            exp: now + 3600,
        };
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(self.account.private_key.as_bytes())
            .context("FCM service-account private key is not a valid RSA PEM")?;
        let assertion = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &key,
        )?;

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            expires_in: i64,
        }
        let res = self
            .http
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &assertion),
            ])
            .send()
            .await?;
        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(anyhow!("FCM token exchange failed: {status} {body}"));
        }
        let token: TokenResponse = res.json().await?;

        let mut cached = self.cached.lock().await;
        *cached = Some(CachedToken {
            value: token.access_token.clone(),
            expires_at: now + token.expires_in,
        });
        Ok(token.access_token)
    }
}

#[async_trait]
impl PushSender for FcmSender {
    async fn send(&self, token: &str, msg: &PushMessage) -> anyhow::Result<SendOutcome> {
        // A credentials failure is an `Err` (it aborts the whole dispatch); a
        // per-token failure is a `SendOutcome`.
        let access = self.access_token().await?;
        let res = self
            .http
            .post(self.send_url())
            .bearer_auth(access)
            .json(&message_body(token, msg))
            .send()
            .await;
        Ok(match res {
            Ok(res) => {
                let status = res.status().as_u16();
                let body = res.text().await.unwrap_or_default();
                classify(status, &body)
            }
            // Network-level failure: never a reason to delete a user's token.
            Err(e) => SendOutcome::Retryable(format!("fcm request failed: {e}")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_is_delivered() {
        assert_eq!(classify(200, "{}"), SendOutcome::Delivered);
    }

    #[test]
    fn dead_registrations_are_invalid_so_the_token_is_pruned() {
        assert!(classify(404, r#"{"error":{"status":"NOT_FOUND"}}"#).is_invalid());
        assert!(classify(400, r#"{"error":{"status":"INVALID_ARGUMENT"}}"#).is_invalid());
        assert!(
            classify(
                400,
                r#"{"error":{"details":[{"errorCode":"UNREGISTERED"}]}}"#
            )
            .is_invalid()
        );
    }

    #[test]
    fn our_own_credential_and_transient_failures_never_delete_a_token() {
        for (status, body) in [
            (401, "invalid credentials"),
            (403, "SenderId mismatch"),
            (429, "quota"),
            (500, "internal"),
            (503, "unavailable"),
            // A 400 that is not about the token itself stays retryable.
            (400, r#"{"error":{"status":"FAILED_PRECONDITION"}}"#),
        ] {
            let outcome = classify(status, body);
            assert!(
                outcome.is_retryable(),
                "http {status} should be retryable, got {outcome:?}"
            );
        }
    }

    #[test]
    fn body_carries_the_token_notification_and_data() {
        let msg = PushMessage::new("Streak", "Play today").with_data("route", "/practice");
        let body = message_body("tok-1", &msg);
        assert_eq!(body["message"]["token"], "tok-1");
        assert_eq!(body["message"]["notification"]["title"], "Streak");
        assert_eq!(body["message"]["notification"]["body"], "Play today");
        assert_eq!(body["message"]["data"]["route"], "/practice");
    }

    #[test]
    fn service_account_parses_the_key_file_shape() {
        let sa = ServiceAccount::from_json(
            r#"{"type":"service_account","project_id":"cymbra","client_email":"x@cymbra.iam.gserviceaccount.com","private_key":"-----BEGIN PRIVATE KEY-----\n-----END PRIVATE KEY-----\n"}"#,
        )
        .unwrap();
        assert_eq!(sa.project_id, "cymbra");
        assert_eq!(
            FcmSender::new(sa).send_url(),
            "https://fcm.googleapis.com/v1/projects/cymbra/messages:send"
        );
        assert!(ServiceAccount::from_json("not json").is_err());
    }
}
