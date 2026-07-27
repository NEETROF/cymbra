//! Internal-token JWT codec — `token_core` (task 2.5).
//!
//! Access tokens are signed with an **asymmetric** Ed25519 key (`kid` in the
//! header); downstream apps verify them offline against the published JWKS (D9).
//! This module is host-testable: keys are passed in, no global state.

use crate::error::{AppError, Result};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, decode_header, encode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Internal access-token claims.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    /// Internal account id (UUID v7).
    pub sub: String,
    /// App audience the token is scoped to.
    pub aud: String,
    /// Effective role names for the audience.
    pub roles: Vec<String>,
    /// Issued-at (unix seconds).
    pub iat: usize,
    /// Expiry (unix seconds).
    pub exp: usize,
    /// Unique token id (for logging / future deny-lists).
    pub jti: String,
}

/// Build a fresh claim set expiring `access_ttl` from now.
pub fn new_claims(
    user_id: &str,
    audience: &str,
    roles: Vec<String>,
    access_ttl: Duration,
) -> Claims {
    let now = unix_now();
    Claims {
        sub: user_id.to_string(),
        aud: audience.to_string(),
        roles,
        iat: now,
        exp: now + access_ttl.as_secs() as usize,
        jti: uuid::Uuid::now_v7().to_string(),
    }
}

/// Parse an Ed25519 private key (PKCS#8 PEM) for signing.
pub fn encoding_key(pem: &str) -> Result<EncodingKey> {
    EncodingKey::from_ed_pem(pem.as_bytes())
        .map_err(|e| AppError::Config(format!("invalid signing key: {e}")))
}

/// Parse an Ed25519 public key (SPKI PEM) for verification.
pub fn decoding_key(pem: &str) -> Result<DecodingKey> {
    DecodingKey::from_ed_pem(pem.as_bytes())
        .map_err(|e| AppError::Config(format!("invalid public key: {e}")))
}

/// Sign `claims` as an EdDSA JWT, stamping `kid` into the header.
pub fn sign(claims: &Claims, kid: &str, key: &EncodingKey) -> Result<String> {
    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = Some(kid.to_string());
    encode(&header, claims, key).map_err(|e| AppError::Internal(e.into()))
}

/// Verify an EdDSA JWT: select the key by header `kid`, check signature + `exp`,
/// and require `aud` to be one of `allowed_auds`. Any failure is `Unauthenticated`.
pub fn verify(
    token: &str,
    keys: &HashMap<String, DecodingKey>,
    allowed_auds: &[&str],
) -> Result<Claims> {
    let header =
        decode_header(token).map_err(|_| AppError::Unauthenticated("malformed token".into()))?;
    let kid = header
        .kid
        .ok_or_else(|| AppError::Unauthenticated("token missing kid".into()))?;
    let key = keys
        .get(&kid)
        .ok_or_else(|| AppError::Unauthenticated("unknown signing key".into()))?;

    let mut v = Validation::new(Algorithm::EdDSA);
    v.set_audience(allowed_auds);
    v.set_required_spec_claims(&["exp", "aud"]);

    decode::<Claims>(token, key, &v)
        .map(|d| d.claims)
        .map_err(|e| AppError::Unauthenticated(format!("token rejected: {}", e.kind_str())))
}

fn unix_now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize
}

/// Stable kind string for an error (avoids leaking detail in messages).
trait KindStr {
    fn kind_str(&self) -> &'static str;
}
impl KindStr for jsonwebtoken::errors::Error {
    fn kind_str(&self) -> &'static str {
        use jsonwebtoken::errors::ErrorKind::*;
        match self.kind() {
            ExpiredSignature => "expired",
            InvalidAudience => "wrong audience",
            InvalidSignature => "bad signature",
            _ => "invalid",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Throwaway Ed25519 keypair for tests only.
    const PRIV: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIPlT7JHCc7NTTIZVmlCgVeNNEkqsENhAZoscpnG+jSSw\n-----END PRIVATE KEY-----\n";
    const PUB: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAiCcon5VNqPUMVYki6MnxJdscxMozrXbjmdiLGUL8sqA=\n-----END PUBLIC KEY-----\n";

    fn keys() -> HashMap<String, DecodingKey> {
        HashMap::from([("k1".to_string(), decoding_key(PUB).unwrap())])
    }

    #[test]
    fn sign_then_verify_roundtrips() {
        let ek = encoding_key(PRIV).unwrap();
        let claims = new_claims("u1", "music", vec!["user".into()], Duration::from_secs(900));
        let tok = sign(&claims, "k1", &ek).unwrap();
        let got = verify(&tok, &keys(), &["music"]).unwrap();
        assert_eq!(got.sub, "u1");
        assert_eq!(got.roles, vec!["user"]);
    }

    #[test]
    fn wrong_audience_rejected() {
        let ek = encoding_key(PRIV).unwrap();
        let claims = new_claims("u1", "music", vec![], Duration::from_secs(900));
        let tok = sign(&claims, "k1", &ek).unwrap();
        assert!(matches!(
            verify(&tok, &keys(), &["live"]),
            Err(AppError::Unauthenticated(_))
        ));
    }

    #[test]
    fn expired_rejected() {
        let ek = encoding_key(PRIV).unwrap();
        let now = unix_now();
        let claims = Claims {
            sub: "u1".into(),
            aud: "music".into(),
            roles: vec![],
            iat: now - 1000,
            exp: now - 600, // past, beyond default leeway
            jti: "j".into(),
        };
        let tok = sign(&claims, "k1", &ek).unwrap();
        assert!(matches!(
            verify(&tok, &keys(), &["music"]),
            Err(AppError::Unauthenticated(_))
        ));
    }

    #[test]
    fn unknown_kid_rejected() {
        let ek = encoding_key(PRIV).unwrap();
        let claims = new_claims("u1", "music", vec![], Duration::from_secs(900));
        let tok = sign(&claims, "other", &ek).unwrap();
        assert!(matches!(
            verify(&tok, &keys(), &["music"]),
            Err(AppError::Unauthenticated(_))
        ));
    }
}
