//! Google channel (design D8): the app reports a Play purchase token, the server
//! reads the subscription from the Play Developer API (`subscriptionsv2.get`),
//! acknowledges it, and maps the state; Real-Time Developer Notifications arrive
//! by Pub/Sub push, are authenticated by their Google-signed OIDC token, and
//! trigger a fresh API read (the notification body is never trusted for state).
//! [`map_subscription`] is the pure mapper.

use crate::billing::{IngestOutcome, ProviderEvent, ingest, payload_digest};
use crate::model::{EntitlementStatus, Source};
use crate::ports::{EntitlementWrite, StorePurchaseVerifier, VerifiedPurchase};
use crate::service::PlanService;
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use cymbra_platform::{AppError, Result};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

// ------------------------------------------------------------ API responses

/// The subset of `SubscriptionPurchaseV2` the ledger needs.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionV2 {
    #[serde(default)]
    pub subscription_state: Option<String>,
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub line_items: Vec<LineItem>,
    #[serde(default)]
    pub linked_purchase_token: Option<String>,
    #[serde(default)]
    pub acknowledgement_state: Option<String>,
    #[serde(default)]
    pub external_account_identifiers: Option<ExternalAccountIdentifiers>,
    #[serde(default)]
    pub canceled_state_context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct LineItem {
    #[serde(default)]
    pub product_id: Option<String>,
    #[serde(default)]
    pub expiry_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAccountIdentifiers {
    /// What the app set at purchase (`setObfuscatedAccountId`) — the account id.
    #[serde(default)]
    pub obfuscated_external_account_id: Option<String>,
}

/// Pure mapper: a Play subscription (keyed by its purchase token) → the ledger
/// write, or `None` for states that carry no plan (pending, unspecified).
pub fn map_subscription(
    user_id: &str,
    purchase_token: &str,
    sub: &SubscriptionV2,
    now: DateTime<Utc>,
) -> Option<EntitlementWrite> {
    let expiry = sub.line_items.iter().filter_map(|l| l.expiry_time).max();
    let status = match sub.subscription_state.as_deref() {
        Some("SUBSCRIPTION_STATE_ACTIVE") => EntitlementStatus::Active,
        Some("SUBSCRIPTION_STATE_CANCELED") => EntitlementStatus::Cancelled,
        Some("SUBSCRIPTION_STATE_IN_GRACE_PERIOD") => EntitlementStatus::BillingRetry,
        // On hold / paused / expired: Play has withdrawn access.
        Some("SUBSCRIPTION_STATE_ON_HOLD")
        | Some("SUBSCRIPTION_STATE_PAUSED")
        | Some("SUBSCRIPTION_STATE_EXPIRED") => EntitlementStatus::Ended,
        Some("SUBSCRIPTION_STATE_PENDING") | Some("SUBSCRIPTION_STATE_UNSPECIFIED") | None => {
            return None;
        }
        Some(_) => EntitlementStatus::Ended,
    };
    let ends_at = match status {
        EntitlementStatus::Ended if expiry.is_none_or(|e| e > now) => Some(now),
        _ => expiry.or(Some(now)),
    };
    Some(EntitlementWrite {
        user_id: user_id.to_string(),
        source: Source::Google,
        provider_ref: purchase_token.to_string(),
        campaign_id: None,
        starts_at: sub.start_time.unwrap_or(now),
        ends_at,
        status,
    })
}

/// A voided / refunded purchase (from the voided-purchases feed or a refund
/// notification) ends the row now.
pub fn map_voided(user_id: &str, purchase_token: &str, now: DateTime<Utc>) -> EntitlementWrite {
    EntitlementWrite {
        user_id: user_id.to_string(),
        source: Source::Google,
        provider_ref: purchase_token.to_string(),
        campaign_id: None,
        starts_at: now,
        ends_at: Some(now),
        status: EntitlementStatus::Refunded,
    }
}

// --------------------------------------------------------- Play Developer API

/// What the Play API adapter exposes (mockable for the handler tests).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait PlayApi: Send + Sync {
    async fn get_subscription(&self, purchase_token: &str) -> Result<SubscriptionV2>;
    async fn acknowledge(&self, subscription_id: &str, purchase_token: &str) -> Result<()>;
}

/// Service-account credentials for the Play Developer API.
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub package_name: String,
    /// Service-account client email (`iss`).
    pub client_email: String,
    /// Service-account private key (PEM, RS256).
    pub private_key_pem: String,
}

/// reqwest-backed [`PlayApi`] with a cached OAuth2 access token.
pub struct PlayDeveloperApi {
    cfg: GoogleConfig,
    http: reqwest::Client,
    token: Mutex<Option<(String, DateTime<Utc>)>>,
}

impl PlayDeveloperApi {
    pub fn new(cfg: GoogleConfig) -> Self {
        Self {
            cfg,
            http: reqwest::Client::new(),
            token: Mutex::new(None),
        }
    }

    async fn access_token(&self) -> Result<String> {
        use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
        let now = Utc::now();
        if let Some((t, exp)) = self.token.lock().await.as_ref()
            && *exp > now + chrono::Duration::seconds(60)
        {
            return Ok(t.clone());
        }
        let claims = serde_json::json!({
            "iss": self.cfg.client_email,
            "scope": "https://www.googleapis.com/auth/androidpublisher",
            "aud": "https://oauth2.googleapis.com/token",
            "iat": now.timestamp(),
            "exp": now.timestamp() + 3600,
        });
        let key = EncodingKey::from_rsa_pem(self.cfg.private_key_pem.as_bytes())
            .map_err(|e| AppError::Config(format!("google service account key: {e}")))?;
        let assertion = encode(&Header::new(Algorithm::RS256), &claims, &key)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("google assertion: {e}")))?;
        #[derive(Deserialize)]
        struct Tok {
            access_token: String,
            expires_in: i64,
        }
        let tok: Tok = self
            .http
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", assertion.as_str()),
            ])
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("google token: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("google token status: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("google token body: {e}")))?;
        let exp = now + chrono::Duration::seconds(tok.expires_in);
        *self.token.lock().await = Some((tok.access_token.clone(), exp));
        Ok(tok.access_token)
    }
}

#[async_trait]
impl PlayApi for PlayDeveloperApi {
    async fn get_subscription(&self, purchase_token: &str) -> Result<SubscriptionV2> {
        let url = format!(
            "https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{}/purchases/subscriptionsv2/tokens/{purchase_token}",
            self.cfg.package_name
        );
        self.http
            .get(url)
            .bearer_auth(self.access_token().await?)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("play api: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("play api status: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("play api body: {e}")))
    }

    async fn acknowledge(&self, subscription_id: &str, purchase_token: &str) -> Result<()> {
        let url = format!(
            "https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{}/purchases/subscriptions/{subscription_id}/tokens/{purchase_token}:acknowledge",
            self.cfg.package_name
        );
        self.http
            .post(url)
            .bearer_auth(self.access_token().await?)
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("play ack: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("play ack status: {e}")))?;
        Ok(())
    }
}

/// [`StorePurchaseVerifier`] for Google: validate the token through the API,
/// bind it to the caller, acknowledge server-side, map.
pub struct GoogleVerifier {
    api: Arc<dyn PlayApi>,
}

impl GoogleVerifier {
    pub fn new(api: Arc<dyn PlayApi>) -> Self {
        Self { api }
    }
}

#[async_trait]
impl StorePurchaseVerifier for GoogleVerifier {
    async fn verify(
        &self,
        user_id: &str,
        payload: &str,
        product_id: &str,
    ) -> Result<VerifiedPurchase> {
        let now = Utc::now();
        let sub = self.api.get_subscription(payload).await?;
        if let Some(acc) = sub
            .external_account_identifiers
            .as_ref()
            .and_then(|e| e.obfuscated_external_account_id.as_deref())
            && !acc.eq_ignore_ascii_case(user_id)
        {
            return Err(AppError::PermissionDenied(
                "purchase bound to another account".into(),
            ));
        }
        let write = map_subscription(user_id, payload, &sub, now)
            .ok_or_else(|| AppError::FailedPrecondition("purchase not active yet".into()))?;
        if sub.acknowledgement_state.as_deref() == Some("ACKNOWLEDGEMENT_STATE_PENDING") {
            let sid = sub
                .line_items
                .first()
                .and_then(|l| l.product_id.clone())
                .unwrap_or_else(|| product_id.to_string());
            self.api.acknowledge(&sid, payload).await?;
        }
        Ok(VerifiedPurchase { write })
    }
}

// ------------------------------------------------------------- RTDN (Pub/Sub)

/// Authenticates the Pub/Sub push (a Google-signed OIDC token in the
/// `Authorization: Bearer …` header). Mockable for the handler tests.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait PushAuth: Send + Sync {
    async fn verify(&self, bearer: &str) -> Result<()>;
}

/// The Google OIDC verifier for Pub/Sub push: RS256 against Google's JWKS,
/// `iss` accounts.google.com, `aud` = our endpoint URL, `email` = the configured
/// push service account.
pub struct GooglePushAuth {
    audience: String,
    service_account_email: String,
    http: reqwest::Client,
    jwks: Mutex<Option<(serde_json::Value, DateTime<Utc>)>>,
}

impl GooglePushAuth {
    pub fn new(audience: String, service_account_email: String) -> Self {
        Self {
            audience,
            service_account_email,
            http: reqwest::Client::new(),
            jwks: Mutex::new(None),
        }
    }

    async fn jwks(&self) -> Result<serde_json::Value> {
        let now = Utc::now();
        if let Some((v, fetched)) = self.jwks.lock().await.as_ref()
            && now - *fetched < chrono::Duration::hours(6)
        {
            return Ok(v.clone());
        }
        let v: serde_json::Value = self
            .http
            .get("https://www.googleapis.com/oauth2/v3/certs")
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("google jwks: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("google jwks body: {e}")))?;
        *self.jwks.lock().await = Some((v.clone(), now));
        Ok(v)
    }
}

#[async_trait]
impl PushAuth for GooglePushAuth {
    async fn verify(&self, bearer: &str) -> Result<()> {
        use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
        #[derive(Deserialize)]
        struct Claims {
            #[serde(default)]
            email: Option<String>,
            #[serde(default)]
            email_verified: Option<bool>,
        }
        let kid = decode_header(bearer)
            .ok()
            .and_then(|h| h.kid)
            .ok_or_else(|| AppError::Unauthenticated("push token missing kid".into()))?;
        let jwks = self.jwks().await?;
        let jwk = jwks["keys"]
            .as_array()
            .and_then(|ks| ks.iter().find(|k| k["kid"] == kid))
            .ok_or_else(|| AppError::Unauthenticated("unknown google key".into()))?;
        let key = DecodingKey::from_rsa_components(
            jwk["n"].as_str().unwrap_or_default(),
            jwk["e"].as_str().unwrap_or_default(),
        )
        .map_err(|_| AppError::Unauthenticated("malformed google jwk".into()))?;
        let mut v = Validation::new(Algorithm::RS256);
        v.set_audience(&[self.audience.as_str()]);
        v.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);
        let claims = decode::<Claims>(bearer, &key, &v)
            .map_err(|_| AppError::Unauthenticated("push token rejected".into()))?
            .claims;
        if claims.email.as_deref() != Some(self.service_account_email.as_str())
            || claims.email_verified != Some(true)
        {
            return Err(AppError::Unauthenticated(
                "push token from an unexpected account".into(),
            ));
        }
        Ok(())
    }
}

/// The RTDN envelope: `{ message: { data: base64(json), messageId }, subscription }`.
#[derive(Debug, Deserialize)]
struct PushEnvelope {
    message: PushMessage,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PushMessage {
    #[serde(default)]
    data: String,
    message_id: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeveloperNotification {
    #[serde(default)]
    package_name: Option<String>,
    #[serde(default)]
    subscription_notification: Option<SubscriptionNotification>,
    #[serde(default)]
    voided_purchase_notification: Option<VoidedPurchaseNotification>,
    #[serde(default)]
    test_notification: Option<serde_json::Value>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubscriptionNotification {
    #[serde(default)]
    notification_type: i32,
    purchase_token: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VoidedPurchaseNotification {
    purchase_token: String,
}

/// Handle one Pub/Sub push (already authenticated by the route through
/// [`PushAuth`]): decode, re-read the subscription from the API, map, ingest once.
pub async fn handle_rtdn(
    svc: &PlanService,
    api: &dyn PlayApi,
    package_name: &str,
    body: &[u8],
    now: DateTime<Utc>,
) -> Result<IngestOutcome> {
    let env: PushEnvelope = serde_json::from_slice(body)
        .map_err(|_| AppError::InvalidArgument("bad pub/sub envelope".into()))?;
    let event_id = env.message.message_id.clone();
    let digest = payload_digest(body);
    let data = STANDARD
        .decode(env.message.data.as_bytes())
        .or_else(|_| URL_SAFE_NO_PAD.decode(env.message.data.as_bytes()))
        .map_err(|_| AppError::InvalidArgument("bad pub/sub data".into()))?;
    let n: DeveloperNotification = serde_json::from_slice(&data)
        .map_err(|_| AppError::InvalidArgument("bad developer notification".into()))?;
    if n.package_name.as_deref().is_some_and(|p| p != package_name) {
        return Err(AppError::Unauthenticated("rtdn for another package".into()));
    }
    let mut writes = Vec::new();
    if n.test_notification.is_some() {
        // fallthrough: informational, ingested as a no-op
    } else if let Some(v) = n.voided_purchase_notification {
        if let Some(uid) = svc.user_for_ref(Source::Google, &v.purchase_token).await? {
            writes.push(map_voided(&uid, &v.purchase_token, now));
        }
    } else if let Some(s) = n.subscription_notification {
        // Never trust the notification body for state: re-read the subscription.
        let sub = api.get_subscription(&s.purchase_token).await?;
        let uid = match sub
            .external_account_identifiers
            .as_ref()
            .and_then(|e| e.obfuscated_external_account_id.clone())
        {
            Some(u) => Some(u),
            None => svc.user_for_ref(Source::Google, &s.purchase_token).await?,
        };
        match uid {
            Some(uid) => {
                if let Some(w) = map_subscription(&uid, &s.purchase_token, &sub, now) {
                    // A superseded token (upgrade / resubscribe) ends the linked row.
                    if let Some(old) = sub.linked_purchase_token.as_deref()
                        && old != s.purchase_token
                    {
                        writes.push(EntitlementWrite {
                            user_id: uid.clone(),
                            source: Source::Google,
                            provider_ref: old.to_string(),
                            campaign_id: None,
                            starts_at: now,
                            ends_at: Some(now),
                            status: EntitlementStatus::Ended,
                        });
                    }
                    writes.push(w);
                }
                let _ = s.notification_type;
            }
            None => tracing::warn!(
                "google rtdn for an unknown purchase (no account id, no row) — acknowledged"
            ),
        }
    }
    ingest(
        svc,
        ProviderEvent {
            source: Source::Google,
            event_id,
            payload_digest: digest,
            writes,
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn sub(state: &str, days: i64, acc: Option<&str>) -> SubscriptionV2 {
        SubscriptionV2 {
            subscription_state: Some(state.into()),
            start_time: Some(Utc::now() - Duration::days(1)),
            line_items: vec![LineItem {
                product_id: Some("premium_monthly".into()),
                expiry_time: Some(Utc::now() + Duration::days(days)),
            }],
            linked_purchase_token: None,
            acknowledgement_state: Some("ACKNOWLEDGEMENT_STATE_ACKNOWLEDGED".into()),
            external_account_identifiers: acc.map(|a| ExternalAccountIdentifiers {
                obfuscated_external_account_id: Some(a.into()),
            }),
            canceled_state_context: None,
        }
    }

    #[test]
    fn mapper_table() {
        let now = Utc::now();
        let m = |st: &str| map_subscription("u1", "tok", &sub(st, 30, None), now);
        assert_eq!(
            m("SUBSCRIPTION_STATE_ACTIVE").unwrap().status,
            EntitlementStatus::Active
        );
        assert_eq!(
            m("SUBSCRIPTION_STATE_CANCELED").unwrap().status,
            EntitlementStatus::Cancelled
        );
        assert_eq!(
            m("SUBSCRIPTION_STATE_IN_GRACE_PERIOD").unwrap().status,
            EntitlementStatus::BillingRetry
        );
        let held = m("SUBSCRIPTION_STATE_ON_HOLD").unwrap();
        assert_eq!(held.status, EntitlementStatus::Ended);
        assert!(held.ends_at.unwrap() <= now + Duration::seconds(1));
        assert_eq!(
            m("SUBSCRIPTION_STATE_EXPIRED").unwrap().status,
            EntitlementStatus::Ended
        );
        assert!(m("SUBSCRIPTION_STATE_PENDING").is_none());
        assert_eq!(m("SUBSCRIPTION_STATE_ACTIVE").unwrap().provider_ref, "tok");
        assert_eq!(
            map_voided("u1", "tok", now).status,
            EntitlementStatus::Refunded
        );
    }

    #[tokio::test]
    async fn verifier_validates_binds_and_acknowledges() {
        let mut api = MockPlayApi::new();
        api.expect_get_subscription().returning(|tok| {
            Ok(match tok {
                "pending-ack" => SubscriptionV2 {
                    acknowledgement_state: Some("ACKNOWLEDGEMENT_STATE_PENDING".into()),
                    ..sub("SUBSCRIPTION_STATE_ACTIVE", 30, Some("u1"))
                },
                "other" => sub("SUBSCRIPTION_STATE_ACTIVE", 30, Some("u2")),
                "pending" => sub("SUBSCRIPTION_STATE_PENDING", 30, None),
                _ => sub("SUBSCRIPTION_STATE_ACTIVE", 30, None),
            })
        });
        api.expect_acknowledge()
            .withf(|sid, tok| sid == "premium_monthly" && tok == "pending-ack")
            .times(1)
            .returning(|_, _| Ok(()));
        let v = GoogleVerifier::new(Arc::new(api));
        assert!(
            v.verify("u1", "pending-ack", "premium_monthly")
                .await
                .is_ok()
        );
        assert!(v.verify("u1", "acked", "premium_monthly").await.is_ok());
        assert!(matches!(
            v.verify("u1", "other", "premium_monthly").await,
            Err(AppError::PermissionDenied(_))
        ));
        assert!(matches!(
            v.verify("u1", "pending", "premium_monthly").await,
            Err(AppError::FailedPrecondition(_))
        ));
    }
}
