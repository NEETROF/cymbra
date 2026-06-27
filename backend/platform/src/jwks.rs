//! JWKS publishing over an Axum HTTP surface (task 2.6 / D9).
//!
//! The internal-token public key(s) are served at `/.well-known/jwks.json` so
//! downstream apps validate Cymbra ID tokens offline. `/healthz` is also exposed
//! here (the rest of the API is gRPC). Multiple keys can be published for rotation.

use crate::error::{AppError, Result};
use axum::{Json, Router, extract::State, routing::get};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use std::sync::Arc;

/// Decode a PEM body to its DER bytes (strips the `-----BEGIN/END-----` lines).
fn pem_to_der(pem: &str) -> Result<Vec<u8>> {
    let body: String = pem
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");
    STANDARD
        .decode(body.trim())
        .map_err(|e| AppError::Config(format!("invalid public key PEM: {e}")))
}

/// Build the JWK for an Ed25519 public key (SPKI PEM). The 32-byte raw key is the
/// trailing 32 bytes of the SPKI DER.
pub fn ed25519_jwk(public_key_pem: &str, kid: &str) -> Result<Value> {
    let der = pem_to_der(public_key_pem)?;
    if der.len() < 32 {
        return Err(AppError::Config("public key shorter than 32 bytes".into()));
    }
    let raw = &der[der.len() - 32..];
    let x = URL_SAFE_NO_PAD.encode(raw);
    Ok(json!({
        "kty": "OKP",
        "crv": "Ed25519",
        "use": "sig",
        "alg": "EdDSA",
        "kid": kid,
        "x": x,
    }))
}

/// Build the full JWK set (one key for now; extend for rotation).
pub fn jwks(public_key_pem: &str, kid: &str) -> Result<Value> {
    Ok(json!({ "keys": [ed25519_jwk(public_key_pem, kid)?] }))
}

/// Axum router exposing `/healthz` and `/.well-known/jwks.json`.
pub fn router(jwks: Value) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/.well-known/jwks.json", get(jwks_handler))
        .with_state(Arc::new(jwks))
}

async fn jwks_handler(State(jwks): State<Arc<Value>>) -> Json<Value> {
    Json((*jwks).clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUB: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAiCcon5VNqPUMVYki6MnxJdscxMozrXbjmdiLGUL8sqA=\n-----END PUBLIC KEY-----\n";

    #[test]
    fn builds_okp_jwk() {
        let jwk = ed25519_jwk(PUB, "k1").unwrap();
        assert_eq!(jwk["kty"], "OKP");
        assert_eq!(jwk["crv"], "Ed25519");
        assert_eq!(jwk["kid"], "k1");
        // 32 bytes base64url (no pad) -> 43 chars.
        assert_eq!(jwk["x"].as_str().unwrap().len(), 43);
    }

    #[test]
    fn jwks_wraps_keys_array() {
        let set = jwks(PUB, "k1").unwrap();
        assert!(set["keys"].is_array());
        assert_eq!(set["keys"].as_array().unwrap().len(), 1);
    }
}
