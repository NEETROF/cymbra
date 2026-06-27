//! `IdentityVerifier` for OIDC providers (task 4.3).
//!
//! [`OidcVerifier`] turns a provider ID token into a normalized
//! [`ExternalIdentity`]. [`RealOidcVerifier`] supports Google + Apple (multi-issuer,
//! selected by `iss`) with JWKS fetch + RS256 verification — thin I/O glue,
//! integration-tested. [`FakeOidcVerifier`] keeps the auth module unit-testable.

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use cymbra_platform::oidc::{ExternalIdentity, oidc_core};
use cymbra_platform::{AppError, Result};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;

#[async_trait]
pub trait OidcVerifier: Send + Sync {
    /// Verify a provider ID token, returning the external identity it asserts.
    async fn verify(&self, id_token: &str) -> Result<ExternalIdentity>;
}

/// Per-provider OIDC config.
#[derive(Clone)]
pub struct OidcProviderCfg {
    pub provider: String, // google | apple
    pub issuer: String,
    pub audience: String,
    pub jwks_uri: String,
}

#[derive(Deserialize)]
struct IssOnly {
    iss: String,
}

#[derive(Deserialize)]
struct OidcClaims {
    sub: String,
    #[serde(default)]
    email: Option<String>,
    exp: u64,
}

/// Production verifier: validates Google/Apple tokens against their JWKS.
pub struct RealOidcVerifier {
    providers: Vec<OidcProviderCfg>,
    http: reqwest::Client,
    jwks_cache: Mutex<HashMap<String, serde_json::Value>>,
}

impl RealOidcVerifier {
    pub fn new(providers: Vec<OidcProviderCfg>) -> Self {
        Self {
            providers,
            http: reqwest::Client::new(),
            jwks_cache: Mutex::new(HashMap::new()),
        }
    }

    fn unverified_issuer(id_token: &str) -> Result<String> {
        let payload = id_token
            .split('.')
            .nth(1)
            .ok_or_else(|| AppError::Unauthenticated("malformed token".into()))?;
        let bytes = URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| AppError::Unauthenticated("malformed token".into()))?;
        let iss: IssOnly = serde_json::from_slice(&bytes)
            .map_err(|_| AppError::Unauthenticated("token missing iss".into()))?;
        Ok(iss.iss)
    }

    async fn jwks(&self, p: &OidcProviderCfg) -> Result<serde_json::Value> {
        if let Some(v) = self.jwks_cache.lock().unwrap().get(&p.issuer).cloned() {
            return Ok(v);
        }
        let v: serde_json::Value = self
            .http
            .get(&p.jwks_uri)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("jwks fetch: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("jwks parse: {e}")))?;
        self.jwks_cache
            .lock()
            .unwrap()
            .insert(p.issuer.clone(), v.clone());
        Ok(v)
    }
}

#[async_trait]
impl OidcVerifier for RealOidcVerifier {
    async fn verify(&self, id_token: &str) -> Result<ExternalIdentity> {
        let iss = Self::unverified_issuer(id_token)?;
        let provider = self
            .providers
            .iter()
            .find(|p| p.issuer == iss)
            .ok_or_else(|| AppError::Unauthenticated("untrusted issuer".into()))?;

        let kid = decode_header(id_token)
            .ok()
            .and_then(|h| h.kid)
            .ok_or_else(|| AppError::Unauthenticated("token missing kid".into()))?;

        let jwks = self.jwks(provider).await?;
        let jwk = jwks["keys"]
            .as_array()
            .and_then(|ks| ks.iter().find(|k| k["kid"] == kid))
            .ok_or_else(|| AppError::Unauthenticated("unknown provider key".into()))?;
        let n = jwk["n"]
            .as_str()
            .ok_or_else(|| AppError::Unauthenticated("malformed jwk".into()))?;
        let e = jwk["e"]
            .as_str()
            .ok_or_else(|| AppError::Unauthenticated("malformed jwk".into()))?;
        let key = DecodingKey::from_rsa_components(n, e)
            .map_err(|_| AppError::Unauthenticated("malformed jwk".into()))?;

        let mut v = Validation::new(Algorithm::RS256);
        v.set_audience(&[&provider.audience]);
        v.set_issuer(&[&provider.issuer]);
        let claims = decode::<OidcClaims>(id_token, &key, &v)
            .map_err(|_| AppError::Unauthenticated("provider token rejected".into()))?
            .claims;

        // Defensive re-check of the core claims (issuer/aud already enforced).
        oidc_core::validate(&provider.issuer, &provider.issuer, true, claims.exp, 0)?;

        Ok(ExternalIdentity {
            provider: provider.provider.clone(),
            subject: claims.sub,
            email: claims.email,
        })
    }
}

/// Test verifier: treats the token string as the subject (provider configurable).
pub struct FakeOidcVerifier {
    pub provider: String,
}

impl Default for FakeOidcVerifier {
    fn default() -> Self {
        Self {
            provider: "google".into(),
        }
    }
}

#[async_trait]
impl OidcVerifier for FakeOidcVerifier {
    async fn verify(&self, id_token: &str) -> Result<ExternalIdentity> {
        Ok(ExternalIdentity {
            provider: self.provider.clone(),
            subject: id_token.to_string(),
            email: None,
        })
    }
}
