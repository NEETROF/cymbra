//! Apple channel (design D7): StoreKit 2 signed transactions reported by the
//! app, App Store Server Notifications v2, and the App Store Server API used
//! by the reconciliation job. Everything Apple sends is a JWS whose `x5c`
//! chain must terminate at the pinned Apple Root CA G3 — [`verify_jws`] is the
//! pure verifier (chain + ES256 signature), [`map_transaction`] the pure mapper.
//! No official Rust library exists; the verification is deliberately small and
//! fixture-tested with a self-signed chain (same shape as Apple's).

use crate::billing::{IngestOutcome, ProviderEvent, ingest, payload_digest};
use crate::model::{EntitlementStatus, Source};
use crate::ports::{EntitlementWrite, StorePurchaseVerifier, VerifiedPurchase};
use crate::service::PlanService;
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use chrono::{DateTime, TimeZone, Utc};
use cymbra_platform::{AppError, Result};
use serde::Deserialize;
use std::sync::Arc;
use x509_parser::prelude::*;

/// Apple Root CA - G3 (DER), the anchor of every App Store signed payload.
/// <https://www.apple.com/certificateauthority/AppleRootCA-G3.cer>
pub const APPLE_ROOT_CA_G3: &[u8] = include_bytes!("../../certs/AppleRootCA-G3.cer");

const OID_ECDSA_SHA256: &str = "1.2.840.10045.4.3.2";
const OID_ECDSA_SHA384: &str = "1.2.840.10045.4.3.3";
const OID_EC_PUBLIC_KEY: &str = "1.2.840.10045.2.1";
const OID_P256: &str = "1.2.840.10045.3.1.7";
const OID_P384: &str = "1.3.132.0.34";

fn unauth(msg: &str) -> AppError {
    AppError::Unauthenticated(format!("apple jws: {msg}"))
}

#[derive(Deserialize)]
struct JwsHeader {
    alg: String,
    #[serde(default)]
    x5c: Vec<String>,
}

/// Public key material of one certificate, as the curve it lives on.
enum EcKey {
    P256(p256::ecdsa::VerifyingKey),
    P384(p384::ecdsa::VerifyingKey),
}

fn public_key(cert: &X509Certificate<'_>) -> Result<EcKey> {
    let spki = cert.public_key();
    if spki.algorithm.algorithm.to_id_string() != OID_EC_PUBLIC_KEY {
        return Err(unauth("certificate key is not EC"));
    }
    let curve = spki
        .algorithm
        .parameters
        .as_ref()
        .and_then(|p| p.as_oid().ok())
        .map(|o| o.to_id_string())
        .ok_or_else(|| unauth("EC key without curve"))?;
    let point = spki.subject_public_key.data.as_ref();
    match curve.as_str() {
        OID_P256 => p256::ecdsa::VerifyingKey::from_sec1_bytes(point)
            .map(EcKey::P256)
            .map_err(|_| unauth("bad P-256 key")),
        OID_P384 => p384::ecdsa::VerifyingKey::from_sec1_bytes(point)
            .map(EcKey::P384)
            .map_err(|_| unauth("bad P-384 key")),
        _ => Err(unauth("unsupported EC curve")),
    }
}

/// Verify `child`'s signature with `issuer`'s key (ECDSA over the TBS bytes,
/// hash chosen by the signature algorithm OID).
fn verify_cert_signature(child: &X509Certificate<'_>, issuer: &X509Certificate<'_>) -> Result<()> {
    use p256::ecdsa::signature::Verifier as _;
    let alg = child.signature_algorithm.algorithm.to_id_string();
    let tbs = child.tbs_certificate.as_ref();
    let sig = child.signature_value.data.as_ref();
    match (alg.as_str(), public_key(issuer)?) {
        (OID_ECDSA_SHA256, EcKey::P256(k)) => {
            let s = p256::ecdsa::Signature::from_der(sig).map_err(|_| unauth("bad cert sig"))?;
            k.verify(tbs, &s)
                .map_err(|_| unauth("certificate chain broken"))
        }
        (OID_ECDSA_SHA384, EcKey::P384(k)) => {
            let s = p384::ecdsa::Signature::from_der(sig).map_err(|_| unauth("bad cert sig"))?;
            k.verify(tbs, &s)
                .map_err(|_| unauth("certificate chain broken"))
        }
        // Mixed curves/hashes (e.g. a P-256 intermediate signed by the P-384 root
        // with SHA-384) — Apple's real chain: root P-384/SHA-384 signs the WWDR
        // intermediate, which is P-256 signed with SHA-384. Cover both mixes.
        (OID_ECDSA_SHA384, EcKey::P256(k)) => {
            use p256::ecdsa::signature::hazmat::PrehashVerifier as _;
            use sha2::Digest as _;
            let s = p256::ecdsa::Signature::from_der(sig).map_err(|_| unauth("bad cert sig"))?;
            let digest = sha2::Sha384::digest(tbs);
            k.verify_prehash(&digest, &s)
                .map_err(|_| unauth("certificate chain broken"))
        }
        (OID_ECDSA_SHA256, EcKey::P384(k)) => {
            use p384::ecdsa::signature::hazmat::PrehashVerifier as _;
            use sha2::Digest as _;
            let s = p384::ecdsa::Signature::from_der(sig).map_err(|_| unauth("bad cert sig"))?;
            let digest = sha2::Sha256::digest(tbs);
            k.verify_prehash(&digest, &s)
                .map_err(|_| unauth("certificate chain broken"))
        }
        _ => Err(unauth("unsupported certificate signature algorithm")),
    }
}

fn cert_valid_at(cert: &X509Certificate<'_>, now: DateTime<Utc>) -> bool {
    let v = cert.validity();
    let nb = v.not_before.timestamp();
    let na = v.not_after.timestamp();
    let t = now.timestamp();
    nb <= t && t <= na
}

/// Verify an Apple JWS: ES256, `x5c` chain valid at `now`, each certificate
/// signed by the next, the chain anchored at one of `roots` (byte-equal, or
/// signed by it), and the JWS signature made by the leaf key. Returns the
/// decoded payload bytes.
pub fn verify_jws(jws: &str, roots: &[&[u8]], now: DateTime<Utc>) -> Result<Vec<u8>> {
    let mut parts = jws.split('.');
    let (h, p, s) = match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(h), Some(p), Some(s), None) => (h, p, s),
        _ => return Err(unauth("malformed jws")),
    };
    let header: JwsHeader = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(h)
            .map_err(|_| unauth("bad header encoding"))?,
    )
    .map_err(|_| unauth("bad header"))?;
    if header.alg != "ES256" {
        return Err(unauth("alg must be ES256"));
    }
    if header.x5c.is_empty() {
        return Err(unauth("missing x5c"));
    }
    let ders: Vec<Vec<u8>> = header
        .x5c
        .iter()
        .map(|c| STANDARD.decode(c).map_err(|_| unauth("bad x5c encoding")))
        .collect::<Result<_>>()?;
    let certs: Vec<X509Certificate<'_>> = ders
        .iter()
        .map(|d| {
            X509Certificate::from_der(d)
                .map(|(_, c)| c)
                .map_err(|_| unauth("bad certificate"))
        })
        .collect::<Result<_>>()?;
    for c in &certs {
        if !cert_valid_at(c, now) {
            return Err(unauth("certificate expired or not yet valid"));
        }
    }
    // Each certificate is signed by the next one in the chain.
    for i in 0..certs.len() - 1 {
        verify_cert_signature(&certs[i], &certs[i + 1])?;
    }
    // The last one must be a pinned root, or be signed by one.
    let last_der = ders.last().expect("non-empty");
    let last = certs.last().expect("non-empty");
    let anchored = roots.contains(&last_der.as_slice())
        || roots.iter().any(|r| {
            X509Certificate::from_der(r)
                .ok()
                .is_some_and(|(_, root)| verify_cert_signature(last, &root).is_ok())
        });
    if !anchored {
        return Err(unauth("chain not anchored at a pinned Apple root"));
    }
    // Finally the JWS signature itself, by the leaf key.
    let EcKey::P256(leaf) = public_key(&certs[0])? else {
        return Err(unauth("leaf key must be P-256 for ES256"));
    };
    use p256::ecdsa::signature::Verifier as _;
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|_| unauth("bad signature encoding"))?;
    let sig =
        p256::ecdsa::Signature::from_slice(&sig_bytes).map_err(|_| unauth("bad signature"))?;
    let signing_input = format!("{h}.{p}");
    leaf.verify(signing_input.as_bytes(), &sig)
        .map_err(|_| unauth("signature does not verify"))?;
    URL_SAFE_NO_PAD
        .decode(p)
        .map_err(|_| unauth("bad payload encoding"))
}

// ------------------------------------------------------------------ payloads

/// `JWSTransactionDecodedPayload` (the fields the ledger needs).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfo {
    pub original_transaction_id: String,
    #[serde(default)]
    pub transaction_id: Option<String>,
    pub product_id: String,
    #[serde(default)]
    pub bundle_id: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub purchase_date: Option<i64>,
    #[serde(default)]
    pub expires_date: Option<i64>,
    #[serde(default)]
    pub revocation_date: Option<i64>,
    /// The UUID the app set at purchase — Cymbra sets the account id.
    #[serde(default)]
    pub app_account_token: Option<String>,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
}

/// `JWSRenewalInfoDecodedPayload` (the fields the ledger needs).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RenewalInfo {
    #[serde(default)]
    pub auto_renew_status: Option<i32>,
    #[serde(default)]
    pub is_in_billing_retry_period: Option<bool>,
    #[serde(default)]
    pub grace_period_expires_date: Option<i64>,
    #[serde(default)]
    pub expiration_intent: Option<i32>,
}

/// `responseBodyV2DecodedPayload`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPayload {
    pub notification_type: String,
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(rename = "notificationUUID")]
    pub notification_uuid: String,
    #[serde(default)]
    pub data: Option<NotificationData>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NotificationData {
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub bundle_id: Option<String>,
    #[serde(default)]
    pub signed_transaction_info: Option<String>,
    #[serde(default)]
    pub signed_renewal_info: Option<String>,
}

fn ms(t: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_millis_opt(t).single()
}

/// Pure mapper: a transaction (+ renewal info) → the ledger row it implies, or
/// `None` when the payload describes no subscription state (consumables, TEST).
/// `notification_type` refines the status for notifications; `None` = a
/// transaction reported by the app / read from the Server API.
pub fn map_transaction(
    user_id: &str,
    tx: &TransactionInfo,
    renewal: Option<&RenewalInfo>,
    notification_type: Option<&str>,
    subtype: Option<&str>,
    now: DateTime<Utc>,
) -> Option<EntitlementWrite> {
    if tx
        .kind
        .as_deref()
        .is_some_and(|k| k != "Auto-Renewable Subscription")
        && tx.expires_date.is_none()
    {
        return None;
    }
    let expires = tx.expires_date.and_then(ms);
    let starts_at = tx.purchase_date.and_then(ms).unwrap_or(now);
    let mut ends_at = expires;
    let mut status = match notification_type {
        Some("REFUND") | Some("REVOKE") => EntitlementStatus::Refunded,
        Some("EXPIRED") | Some("GRACE_PERIOD_EXPIRED") => EntitlementStatus::Ended,
        // With or without a grace period Apple is retrying: the row stays active
        // until its end (+ our grace), never longer than Apple's own grace end.
        Some("DID_FAIL_TO_RENEW") => EntitlementStatus::BillingRetry,
        Some("DID_CHANGE_RENEWAL_STATUS") => match subtype {
            Some("AUTO_RENEW_DISABLED") => EntitlementStatus::Cancelled,
            _ => EntitlementStatus::Active,
        },
        Some("TEST") => return None,
        _ => EntitlementStatus::Active,
    };
    if let Some(r) = renewal {
        if status == EntitlementStatus::Active && r.is_in_billing_retry_period == Some(true) {
            status = EntitlementStatus::BillingRetry;
        }
        if status == EntitlementStatus::Active && r.auto_renew_status == Some(0) {
            status = EntitlementStatus::Cancelled;
        }
        if status == EntitlementStatus::BillingRetry
            && let Some(g) = r.grace_period_expires_date.and_then(ms)
            && ends_at.is_some_and(|e| g > e)
        {
            // Apple's grace period: keep the row until it ends.
            ends_at = Some(g);
        }
    }
    if let Some(rev) = tx.revocation_date.and_then(ms) {
        status = EntitlementStatus::Refunded;
        ends_at = Some(rev);
    }
    if status == EntitlementStatus::Refunded && ends_at.is_none() {
        ends_at = Some(now);
    }
    // A subscription with no expiry (should not happen for auto-renewables) is
    // not a state we can reason about.
    ends_at?;
    Some(EntitlementWrite {
        user_id: user_id.to_string(),
        source: Source::Apple,
        provider_ref: tx.original_transaction_id.clone(),
        campaign_id: None,
        starts_at,
        ends_at,
        status,
    })
}

// -------------------------------------------------------------------- config

/// The app-side identity checks every Apple payload must pass.
#[derive(Debug, Clone)]
pub struct AppleConfig {
    pub bundle_id: String,
    /// `Sandbox` payloads are accepted only when true (staging / TestFlight).
    pub allow_sandbox: bool,
    /// Pinned roots (defaults to [`APPLE_ROOT_CA_G3`]).
    pub roots: Vec<Vec<u8>>,
}

impl AppleConfig {
    pub fn new(bundle_id: impl Into<String>, allow_sandbox: bool) -> Self {
        Self {
            bundle_id: bundle_id.into(),
            allow_sandbox,
            roots: vec![APPLE_ROOT_CA_G3.to_vec()],
        }
    }

    fn roots(&self) -> Vec<&[u8]> {
        self.roots.iter().map(|r| r.as_slice()).collect()
    }

    fn check_identity(&self, bundle: Option<&str>, env: Option<&str>) -> Result<()> {
        if bundle.is_some_and(|b| b != self.bundle_id) {
            return Err(unauth("bundle id mismatch"));
        }
        if env == Some("Sandbox") && !self.allow_sandbox {
            return Err(unauth("sandbox payload refused"));
        }
        Ok(())
    }

    /// Verify + decode a signed transaction JWS.
    pub fn transaction(&self, jws: &str, now: DateTime<Utc>) -> Result<TransactionInfo> {
        let bytes = verify_jws(jws, &self.roots(), now)?;
        let tx: TransactionInfo =
            serde_json::from_slice(&bytes).map_err(|_| unauth("bad transaction payload"))?;
        self.check_identity(tx.bundle_id.as_deref(), tx.environment.as_deref())?;
        Ok(tx)
    }

    /// Verify + decode a signed renewal-info JWS.
    pub fn renewal(&self, jws: &str, now: DateTime<Utc>) -> Result<RenewalInfo> {
        let bytes = verify_jws(jws, &self.roots(), now)?;
        serde_json::from_slice(&bytes).map_err(|_| unauth("bad renewal payload"))
    }
}

/// [`StorePurchaseVerifier`] for Apple: the app reports the signed transaction.
pub struct AppleVerifier {
    cfg: AppleConfig,
}

impl AppleVerifier {
    pub fn new(cfg: AppleConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl StorePurchaseVerifier for AppleVerifier {
    async fn verify(
        &self,
        user_id: &str,
        payload: &str,
        _product_id: &str,
    ) -> Result<VerifiedPurchase> {
        let now = Utc::now();
        let tx = self.cfg.transaction(payload, now)?;
        if let Some(token) = tx.app_account_token.as_deref()
            && !token.eq_ignore_ascii_case(user_id)
        {
            return Err(AppError::PermissionDenied(
                "transaction bound to another account".into(),
            ));
        }
        let write = map_transaction(user_id, &tx, None, None, None, now)
            .ok_or_else(|| AppError::InvalidArgument("not a subscription transaction".into()))?;
        Ok(VerifiedPurchase { write })
    }
}

// ------------------------------------------------------- notifications v2

/// Handle one App Store Server Notification v2 body (`{ "signedPayload": … }`).
/// Verifies, resolves the user (app account token, else the existing row), maps
/// and ingests once. Unmappable notifications are acknowledged and logged.
pub async fn handle_notification(
    svc: &PlanService,
    cfg: &AppleConfig,
    body: &[u8],
    now: DateTime<Utc>,
) -> Result<IngestOutcome> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Body {
        signed_payload: String,
    }
    let b: Body = serde_json::from_slice(body).map_err(|_| unauth("bad notification body"))?;
    let bytes = verify_jws(&b.signed_payload, &cfg.roots(), now)?;
    let payload: NotificationPayload =
        serde_json::from_slice(&bytes).map_err(|_| unauth("bad notification payload"))?;
    let data = payload.data.unwrap_or_default();
    cfg.check_identity(data.bundle_id.as_deref(), data.environment.as_deref())?;

    let mut writes = Vec::new();
    if let Some(tx_jws) = data.signed_transaction_info.as_deref() {
        let tx = cfg.transaction(tx_jws, now)?;
        let renewal = match data.signed_renewal_info.as_deref() {
            Some(r) => Some(cfg.renewal(r, now)?),
            None => None,
        };
        let user_id = match tx.app_account_token.clone() {
            Some(t) => Some(t),
            None => {
                svc.user_for_ref(Source::Apple, &tx.original_transaction_id)
                    .await?
            }
        };
        match user_id {
            Some(uid) => {
                if let Some(w) = map_transaction(
                    &uid,
                    &tx,
                    renewal.as_ref(),
                    Some(&payload.notification_type),
                    payload.subtype.as_deref(),
                    now,
                ) {
                    writes.push(w);
                }
            }
            None => tracing::warn!(
                notification = %payload.notification_type,
                original_transaction_id = %tx.original_transaction_id,
                "apple notification for an unknown subscription (no account token, no row) — acknowledged"
            ),
        }
    }
    ingest(
        svc,
        ProviderEvent {
            source: Source::Apple,
            event_id: payload.notification_uuid,
            payload_digest: payload_digest(body),
            writes,
        },
    )
    .await
}

// ------------------------------------------------------- App Store Server API

/// Minimal App Store Server API client for reconciliation: subscription
/// statuses by original transaction id (JWT ES256 signed with the App Store
/// Connect API key).
pub struct AppStoreServerApi {
    cfg: Arc<AppleConfig>,
    key_pem: String,
    key_id: String,
    issuer_id: String,
    sandbox: bool,
    http: reqwest::Client,
}

impl AppStoreServerApi {
    pub fn new(
        cfg: Arc<AppleConfig>,
        key_pem: String,
        key_id: String,
        issuer_id: String,
        sandbox: bool,
    ) -> Self {
        Self {
            cfg,
            key_pem,
            key_id,
            issuer_id,
            sandbox,
            http: reqwest::Client::new(),
        }
    }

    fn base(&self) -> &'static str {
        if self.sandbox {
            "https://api.storekit-sandbox.itunes.apple.com"
        } else {
            "https://api.storekit.itunes.apple.com"
        }
    }

    fn token(&self) -> Result<String> {
        use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
        let now = Utc::now().timestamp();
        let claims = serde_json::json!({
            "iss": self.issuer_id,
            "iat": now,
            "exp": now + 600,
            "aud": "appstoreconnect-v1",
            "bid": self.cfg.bundle_id,
        });
        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());
        header.typ = Some("JWT".into());
        let key = EncodingKey::from_ec_pem(self.key_pem.as_bytes())
            .map_err(|e| AppError::Config(format!("apple api key: {e}")))?;
        encode(&header, &claims, &key)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("apple api token: {e}")))
    }

    /// The latest transaction (+ renewal info) of each subscription group for
    /// `original_transaction_id`, verified.
    pub async fn latest_transactions(
        &self,
        original_transaction_id: &str,
        now: DateTime<Utc>,
    ) -> Result<Vec<(TransactionInfo, Option<RenewalInfo>)>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Last {
            signed_transaction_info: String,
            #[serde(default)]
            signed_renewal_info: Option<String>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Group {
            #[serde(default)]
            last_transactions: Vec<Last>,
        }
        #[derive(Deserialize)]
        struct Resp {
            #[serde(default)]
            data: Vec<Group>,
        }
        let url = format!(
            "{}/inApps/v1/subscriptions/{original_transaction_id}",
            self.base()
        );
        let resp: Resp = self
            .http
            .get(url)
            .bearer_auth(self.token()?)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("apple api: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("apple api status: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("apple api body: {e}")))?;
        let mut out = Vec::new();
        for g in resp.data {
            for l in g.last_transactions {
                let tx = self.cfg.transaction(&l.signed_transaction_info, now)?;
                let renewal = match l.signed_renewal_info.as_deref() {
                    Some(r) => Some(self.cfg.renewal(r, now)?),
                    None => None,
                };
                out.push((tx, renewal));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use chrono::Duration;
    use p256::ecdsa::signature::Signer as _;

    /// A self-signed test chain shaped like Apple's: root (P-384/SHA-384) → leaf
    /// (P-256/SHA-256), plus the leaf's signing key. Returns (root DER, leaf DER,
    /// leaf signing key).
    pub(crate) fn test_chain() -> (Vec<u8>, Vec<u8>, p256::ecdsa::SigningKey) {
        use rcgen::{
            BasicConstraints, CertificateParams, DnType, IsCa, KeyPair, PKCS_ECDSA_P256_SHA256,
            PKCS_ECDSA_P384_SHA384,
        };
        let root_key = KeyPair::generate_for(&PKCS_ECDSA_P384_SHA384).unwrap();
        let mut root_params = CertificateParams::new(Vec::<String>::new()).unwrap();
        root_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        root_params
            .distinguished_name
            .push(DnType::CommonName, "Test Root CA");
        let root = root_params.self_signed(&root_key).unwrap();

        let leaf_key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
        let mut leaf_params = CertificateParams::new(Vec::<String>::new()).unwrap();
        leaf_params
            .distinguished_name
            .push(DnType::CommonName, "Test Leaf");
        // A bounded validity so the "expired chain" path is testable.
        leaf_params.not_before = rcgen::date_time_ymd(2020, 1, 1);
        leaf_params.not_after = rcgen::date_time_ymd(2035, 1, 1);
        let leaf = leaf_params.signed_by(&leaf_key, &root, &root_key).unwrap();

        use p256::pkcs8::DecodePrivateKey as _;
        let signing = p256::ecdsa::SigningKey::from_pkcs8_der(&leaf_key.serialize_der()).unwrap();
        (root.der().to_vec(), leaf.der().to_vec(), signing)
    }

    /// Sign `payload` as an ES256 JWS with `x5c = [leaf, root]`.
    pub(crate) fn sign_jws(
        payload: &serde_json::Value,
        chain: &[&[u8]],
        key: &p256::ecdsa::SigningKey,
    ) -> String {
        let header = serde_json::json!({
            "alg": "ES256",
            "x5c": chain.iter().map(|c| STANDARD.encode(c)).collect::<Vec<_>>(),
        });
        let h = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
        let p = URL_SAFE_NO_PAD.encode(serde_json::to_vec(payload).unwrap());
        let input = format!("{h}.{p}");
        let sig: p256::ecdsa::Signature = key.sign(input.as_bytes());
        format!("{input}.{}", URL_SAFE_NO_PAD.encode(sig.to_bytes()))
    }

    fn tx_json(user: &str, expires_in_days: i64) -> serde_json::Value {
        let now = Utc::now();
        serde_json::json!({
            "originalTransactionId": "otx-1",
            "transactionId": "tx-9",
            "productId": "premium_monthly",
            "bundleId": "com.cymbra.music",
            "environment": "Sandbox",
            "purchaseDate": (now - Duration::days(1)).timestamp_millis(),
            "expiresDate": (now + Duration::days(expires_in_days)).timestamp_millis(),
            "appAccountToken": user,
            "type": "Auto-Renewable Subscription",
        })
    }

    #[test]
    fn valid_chain_and_signature_verify_and_tampering_is_rejected() {
        let (root, leaf, key) = test_chain();
        let now = Utc::now();
        let payload = tx_json("u1", 30);
        let jws = sign_jws(&payload, &[&leaf, &root], &key);
        let bytes = verify_jws(&jws, &[&root], now).unwrap();
        let tx: TransactionInfo = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(tx.original_transaction_id, "otx-1");

        // A chain that omits the root but is signed by it still anchors.
        let jws2 = sign_jws(&payload, &[&leaf], &key);
        assert!(verify_jws(&jws2, &[&root], now).is_ok());

        // Tampered payload → signature fails.
        let mut parts: Vec<&str> = jws.split('.').collect();
        let forged = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&tx_json("u2", 300)).unwrap());
        parts[1] = &forged;
        let tampered = parts.join(".");
        assert!(matches!(
            verify_jws(&tampered, &[&root], now),
            Err(AppError::Unauthenticated(_))
        ));

        // Wrong root → not anchored.
        let (other_root, _, _) = test_chain();
        assert!(verify_jws(&jws, &[&other_root], now).is_err());
        // Expired chain (clock past the leaf's not_after) → refused.
        assert!(
            verify_jws(
                &jws,
                &[&root],
                Utc.with_ymd_and_hms(2036, 1, 1, 0, 0, 0).unwrap()
            )
            .is_err()
        );
        // Malformed / wrong alg.
        assert!(verify_jws("a.b", &[&root], now).is_err());
    }

    #[test]
    fn apple_root_is_the_pinned_g3() {
        let (_, root) = X509Certificate::from_der(APPLE_ROOT_CA_G3).unwrap();
        assert!(root.subject().to_string().contains("Apple Root CA - G3"));
        assert!(matches!(public_key(&root).unwrap(), EcKey::P384(_)));
    }

    #[test]
    fn config_checks_bundle_and_environment() {
        let (root, leaf, key) = test_chain();
        let now = Utc::now();
        let mut cfg = AppleConfig::new("com.cymbra.music", true);
        cfg.roots = vec![root.clone()];
        let jws = sign_jws(&tx_json("u1", 30), &[&leaf, &root], &key);
        assert!(cfg.transaction(&jws, now).is_ok());
        cfg.allow_sandbox = false;
        assert!(cfg.transaction(&jws, now).is_err());
        cfg.allow_sandbox = true;
        cfg.bundle_id = "com.other".into();
        assert!(cfg.transaction(&jws, now).is_err());
    }

    fn tx(expires_in: i64) -> TransactionInfo {
        serde_json::from_value(tx_json("u1", expires_in)).unwrap()
    }

    #[test]
    fn mapper_table() {
        let now = Utc::now();
        let t = tx(30);
        let w = |n: Option<&str>, s: Option<&str>, r: Option<&RenewalInfo>| {
            map_transaction("u1", &t, r, n, s, now).unwrap()
        };
        assert_eq!(w(None, None, None).status, EntitlementStatus::Active);
        assert_eq!(w(None, None, None).provider_ref, "otx-1");
        assert_eq!(
            w(Some("DID_RENEW"), None, None).status,
            EntitlementStatus::Active
        );
        assert_eq!(
            w(
                Some("DID_CHANGE_RENEWAL_STATUS"),
                Some("AUTO_RENEW_DISABLED"),
                None
            )
            .status,
            EntitlementStatus::Cancelled
        );
        assert_eq!(
            w(Some("DID_FAIL_TO_RENEW"), Some("GRACE_PERIOD"), None).status,
            EntitlementStatus::BillingRetry
        );
        assert_eq!(
            w(Some("EXPIRED"), None, None).status,
            EntitlementStatus::Ended
        );
        assert_eq!(
            w(Some("REFUND"), None, None).status,
            EntitlementStatus::Refunded
        );
        // renewal info refines: billing retry + grace extends the end
        let grace = RenewalInfo {
            is_in_billing_retry_period: Some(true),
            grace_period_expires_date: Some((now + Duration::days(46)).timestamp_millis()),
            ..Default::default()
        };
        let g = w(None, None, Some(&grace));
        assert_eq!(g.status, EntitlementStatus::BillingRetry);
        assert!(g.ends_at.unwrap() > now + Duration::days(45));
        let off = RenewalInfo {
            auto_renew_status: Some(0),
            ..Default::default()
        };
        assert_eq!(
            w(None, None, Some(&off)).status,
            EntitlementStatus::Cancelled
        );
        // revocation date ⇒ refunded, ended at the revocation
        let mut revoked = tx(30);
        revoked.revocation_date = Some(now.timestamp_millis());
        assert_eq!(
            map_transaction("u1", &revoked, None, None, None, now)
                .unwrap()
                .status,
            EntitlementStatus::Refunded
        );
        // TEST and consumables map to nothing
        assert!(map_transaction("u1", &t, None, Some("TEST"), None, now).is_none());
        let mut consumable = tx(30);
        consumable.kind = Some("Consumable".into());
        consumable.expires_date = None;
        assert!(map_transaction("u1", &consumable, None, None, None, now).is_none());
    }

    #[tokio::test]
    async fn verifier_binds_the_transaction_to_the_caller() {
        let (root, leaf, key) = test_chain();
        let mut cfg = AppleConfig::new("com.cymbra.music", true);
        cfg.roots = vec![root.clone()];
        let v = AppleVerifier::new(cfg);
        let jws = sign_jws(&tx_json("u1", 30), &[&leaf, &root], &key);
        let ok = v.verify("u1", &jws, "premium_monthly").await.unwrap();
        assert_eq!(ok.write.source, Source::Apple);
        assert!(matches!(
            v.verify("u2", &jws, "premium_monthly").await,
            Err(AppError::PermissionDenied(_))
        ));
        assert!(v.verify("u1", "garbage", "").await.is_err());
    }
}
