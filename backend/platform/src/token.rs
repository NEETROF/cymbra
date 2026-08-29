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
use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Internal access-token claims.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    /// Internal account id (UUID v7).
    pub sub: String,
    /// App audience the token is scoped to.
    pub aud: String,
    /// Effective role names for the audience — the union across every scope in
    /// `roles_by_scope` (kept for the coarse `has_role`/`require_admin` guards and
    /// backward compatibility).
    pub roles: Vec<String>,
    /// Roles grouped by the scope they are held in (`global` / `music` / `live`),
    /// so authorization can ask "is the caller admin **in scope S**" — not just
    /// "does the caller hold admin somewhere". Empty on legacy/flat tokens; absent
    /// entries mean the account holds no role in that scope (change:
    /// scope-aware-role-admin).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub roles_by_scope: BTreeMap<String, Vec<String>>,
    /// Issued-at (unix seconds).
    pub iat: usize,
    /// Expiry (unix seconds).
    pub exp: usize,
    /// Unique token id (for logging / future deny-lists).
    pub jti: String,
}

/// Build a fresh claim set expiring `access_ttl` from now, carrying a flat role
/// set only (no per-scope breakdown). Retained for callers/tests that don't need
/// scope-matched authorization.
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
        roles_by_scope: BTreeMap::new(),
        iat: now,
        exp: now + access_ttl.as_secs() as usize,
        jti: uuid::Uuid::now_v7().to_string(),
    }
}

/// Build a fresh claim set from roles **grouped by scope**. The flat `roles` field
/// is derived as the de-duplicated union across scopes, so the coarse guards keep
/// working while `roles_by_scope` powers scope-matched checks (change:
/// scope-aware-role-admin).
pub fn new_claims_scoped(
    user_id: &str,
    audience: &str,
    roles_by_scope: BTreeMap<String, Vec<String>>,
    access_ttl: Duration,
) -> Claims {
    let now = unix_now();
    let mut roles: Vec<String> = Vec::new();
    for scope_roles in roles_by_scope.values() {
        for r in scope_roles {
            if !roles.contains(r) {
                roles.push(r.clone());
            }
        }
    }
    Claims {
        sub: user_id.to_string(),
        aud: audience.to_string(),
        roles,
        roles_by_scope,
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
pub(crate) mod tests {
    use super::*;

    // Throwaway Ed25519 keypair for tests only.
    const PRIV: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIPlT7JHCc7NTTIZVmlCgVeNNEkqsENhAZoscpnG+jSSw\n-----END PRIVATE KEY-----\n";
    const PUB: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAiCcon5VNqPUMVYki6MnxJdscxMozrXbjmdiLGUL8sqA=\n-----END PUBLIC KEY-----\n";

    /// The signing half of the throwaway pair, for tests in sibling modules.
    ///
    /// Exposed as a **function** rather than the PEM: one fixture, in one file.
    /// A second copy of the literal elsewhere is a second thing to rotate if it
    /// ever turned out to matter, and reads to any secret scanner as a fresh
    /// private key committed to the repository.
    pub(crate) fn signing_key() -> EncodingKey {
        encoding_key(PRIV).unwrap()
    }

    /// The verifying half, keyed under `k1` as every test signs with.
    pub(crate) fn verifying_keys() -> HashMap<String, DecodingKey> {
        HashMap::from([("k1".to_string(), decoding_key(PUB).unwrap())])
    }

    fn keys() -> HashMap<String, DecodingKey> {
        verifying_keys()
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
    fn scoped_claims_derive_flat_union_and_roundtrip() {
        let ek = encoding_key(PRIV).unwrap();
        let by_scope = BTreeMap::from([
            ("global".to_string(), vec!["user".to_string()]),
            ("music".to_string(), vec!["admin".to_string()]),
            ("live".to_string(), vec!["moderator".to_string()]),
        ]);
        let claims = new_claims_scoped("u1", "back-office", by_scope, Duration::from_secs(900));
        // Flat roles are the de-duplicated union across scopes.
        assert!(claims.roles.contains(&"user".to_string()));
        assert!(claims.roles.contains(&"admin".to_string()));
        assert!(claims.roles.contains(&"moderator".to_string()));
        // Per-scope breakdown survives the JWT round-trip.
        let tok = sign(&claims, "k1", &ek).unwrap();
        let got = verify(&tok, &keys(), &["back-office"]).unwrap();
        assert_eq!(got.roles_by_scope["music"], vec!["admin".to_string()]);
        assert_eq!(got.roles_by_scope["live"], vec!["moderator".to_string()]);
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
            roles_by_scope: BTreeMap::new(),
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
