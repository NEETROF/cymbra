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

//! The signed desktop-update manifest contract (change: add-desktop-auto-update,
//! design D2/D7).
//!
//! The signature covers the **exact manifest bytes**, carried base64-encoded
//! inside the envelope and parsed only *after* verification. That is the whole
//! trick: there is no JSON canonicalization to agree on between Rust and Dart,
//! and the bytes that were verified are by construction the bytes that get
//! parsed.
//!
//! `key_id` and `rollout_percent` sit **outside** the signature. `key_id` is a
//! selection hint into a set of trusted keys — an unknown id is rejected, so
//! tampering with it yields a verification failure, never a forgery.
//! `rollout_percent` is backend policy rather than release identity; tampering
//! with it can only offer a legitimately signed update sooner.
//!
//! This crate is the single definition shared by the CI signing binary and the
//! backend ingest handler (which re-verifies before storing, so a stolen ingest
//! credential cannot inject an unsigned or foreign-key manifest). The Dart side
//! re-implements verification, and both run against the same checked-in golden
//! fixture — see `fixtures/`.

use std::collections::BTreeMap;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

/// The only manifest schema this build understands. A client that sees a higher
/// value refuses the manifest rather than guessing at unknown semantics.
pub const SCHEMA_VERSION: u32 = 1;

/// One downloadable artifact, keyed in [`Manifest::targets`] by `<os>-<arch>`
/// (e.g. `windows-x64`, `linux-x64`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    /// Install method the artifact implies: `inno-setup`, `appimage`.
    pub kind: String,
    /// Absolute download URL (a GitHub Release asset).
    pub url: String,
    /// Exact byte length. Enforced as a hard cap while streaming, *before* the
    /// hash can be computed.
    pub size: u64,
    /// Lowercase hex SHA-256 of the artifact bytes.
    pub sha256: String,
}

/// The release description that the CI key signs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Contract version; see [`SCHEMA_VERSION`].
    pub schema: u32,
    /// Product this release belongs to (`music`; `live` later).
    pub product: String,
    /// Release channel (`stable`).
    pub channel: String,
    /// Offered version, in the app's own `major.minor.patch+build` format.
    pub version: String,
    /// RFC 3339 release timestamp.
    pub released_at: String,
    /// Below this version a client cannot talk to the backend and must be
    /// forced to update. Absent means no floor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_supported_version: Option<String>,
    /// Human-readable release notes (the GitHub Release page).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes_url: Option<String>,
    /// Artifacts by `<os>-<arch>`. A `BTreeMap` so serialization is stable —
    /// convenient for fixtures, though nothing depends on it (the signature
    /// covers bytes, not a re-serialization).
    pub targets: BTreeMap<String, Target>,
}

/// What the feed stores and serves: opaque signed bytes plus the two policy
/// fields that are deliberately outside the signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// Base64 of the exact manifest JSON bytes that were signed.
    pub manifest: String,
    /// Base64 Ed25519 signature over those bytes.
    pub signature: String,
    /// Which trusted key signed it. Outside the signature, by design.
    pub key_id: String,
    /// Staged-rollout percentage, evaluated client-side. `0` is the
    /// kill-switch. Outside the signature: backend policy, not release identity.
    pub rollout_percent: u8,
}

/// Why a manifest was refused. Every variant means "nothing is downloaded".
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VerifyError {
    /// `manifest` or `signature` is not valid base64.
    #[error("malformed base64 in the envelope")]
    MalformedBase64,
    /// The signature field does not decode to 64 bytes.
    #[error("malformed signature")]
    MalformedSignature,
    /// No trusted key carries this `key_id`.
    #[error("unknown key id")]
    UnknownKeyId,
    /// The signature does not cover these bytes with that key.
    #[error("signature does not verify")]
    BadSignature,
    /// The verified bytes are not a manifest.
    #[error("malformed manifest json")]
    MalformedManifest,
    /// A schema this build does not implement.
    #[error("unsupported manifest schema")]
    UnsupportedSchema,
}

/// The compiled-in trusted keys, by `key_id`. A **set**, not a single value, so
/// a rotation can ship a build that trusts both the old and the new key before
/// the old one is dropped.
pub type TrustedKeys = BTreeMap<String, VerifyingKey>;

/// Sign the exact `manifest_bytes` — the caller owns the serialization, and the
/// same bytes must be what ends up in the envelope.
pub fn sign(
    manifest_bytes: &[u8],
    signing_key: &SigningKey,
    key_id: &str,
    rollout_percent: u8,
) -> Envelope {
    let signature: Signature = signing_key.sign(manifest_bytes);
    Envelope {
        manifest: B64.encode(manifest_bytes),
        signature: B64.encode(signature.to_bytes()),
        key_id: key_id.to_string(),
        rollout_percent,
    }
}

/// Verify an envelope and return the manifest it carries.
///
/// The order is fixed and load-bearing: decode → resolve the key → verify the
/// signature over the decoded bytes → *only then* parse → guard the schema.
/// Nothing downstream ever sees bytes that failed a step.
pub fn verify(envelope: &Envelope, trusted: &TrustedKeys) -> Result<Manifest, VerifyError> {
    let bytes = B64
        .decode(envelope.manifest.as_bytes())
        .map_err(|_| VerifyError::MalformedBase64)?;
    let sig_bytes = B64
        .decode(envelope.signature.as_bytes())
        .map_err(|_| VerifyError::MalformedBase64)?;
    let sig_array: [u8; 64] = sig_bytes
        .as_slice()
        .try_into()
        .map_err(|_| VerifyError::MalformedSignature)?;
    let key = trusted
        .get(&envelope.key_id)
        .ok_or(VerifyError::UnknownKeyId)?;
    key.verify_strict(&bytes, &Signature::from_bytes(&sig_array))
        .map_err(|_| VerifyError::BadSignature)?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|_| VerifyError::MalformedManifest)?;
    if manifest.schema != SCHEMA_VERSION {
        return Err(VerifyError::UnsupportedSchema);
    }
    Ok(manifest)
}

/// Build a [`TrustedKeys`] map from `key_id=<base64 32-byte public key>` pairs,
/// the shape the backend reads out of its environment.
pub fn trusted_keys_from_pairs<'a>(
    pairs: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<TrustedKeys, VerifyError> {
    let mut out = TrustedKeys::new();
    for (id, b64) in pairs {
        out.insert(id.to_string(), verifying_key_from_base64(b64)?);
    }
    Ok(out)
}

/// Decode a base64 32-byte Ed25519 public key.
pub fn verifying_key_from_base64(b64: &str) -> Result<VerifyingKey, VerifyError> {
    let raw = B64
        .decode(b64.trim().as_bytes())
        .map_err(|_| VerifyError::MalformedBase64)?;
    let array: [u8; 32] = raw
        .as_slice()
        .try_into()
        .map_err(|_| VerifyError::MalformedSignature)?;
    VerifyingKey::from_bytes(&array).map_err(|_| VerifyError::UnknownKeyId)
}

/// Decode a base64 32-byte Ed25519 secret key (the CI signing secret).
pub fn signing_key_from_base64(b64: &str) -> Result<SigningKey, VerifyError> {
    let raw = B64
        .decode(b64.trim().as_bytes())
        .map_err(|_| VerifyError::MalformedBase64)?;
    let array: [u8; 32] = raw
        .as_slice()
        .try_into()
        .map_err(|_| VerifyError::MalformedSignature)?;
    Ok(SigningKey::from_bytes(&array))
}

/// Parse the `CYMBRA_UPDATE_TRUSTED_KEYS` form: `id=<base64>` entries separated
/// by commas. Empty input yields an empty set, which fails every ingest closed.
pub fn trusted_keys_from_env_value(value: &str) -> Result<TrustedKeys, VerifyError> {
    let mut out = TrustedKeys::new();
    for entry in value.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (id, b64) = entry.split_once('=').ok_or(VerifyError::MalformedBase64)?;
        out.insert(id.trim().to_string(), verifying_key_from_base64(b64)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed secret so the fixture and these tests are reproducible. This is a
    /// TEST key: the release key is generated by `update-keygen` and never
    /// enters the repository.
    const TEST_SECRET: [u8; 32] = [7u8; 32];

    fn key() -> SigningKey {
        SigningKey::from_bytes(&TEST_SECRET)
    }

    fn trusted(id: &str, k: &SigningKey) -> TrustedKeys {
        TrustedKeys::from([(id.to_string(), k.verifying_key())])
    }

    fn manifest() -> Manifest {
        Manifest {
            schema: SCHEMA_VERSION,
            product: "music".into(),
            channel: "stable".into(),
            version: "1.25.0+34".into(),
            released_at: "2026-08-21T10:00:00Z".into(),
            min_supported_version: Some("1.20.0+27".into()),
            notes_url: Some("https://example.invalid/notes".into()),
            targets: BTreeMap::from([(
                "windows-x64".to_string(),
                Target {
                    kind: "inno-setup".into(),
                    url: "https://example.invalid/setup.exe".into(),
                    size: 48_123_904,
                    sha256: "a".repeat(64),
                },
            )]),
        }
    }

    fn signed() -> (Envelope, TrustedKeys) {
        let k = key();
        let bytes = serde_json::to_vec(&manifest()).unwrap();
        (sign(&bytes, &k, "test-a", 25), trusted("test-a", &k))
    }

    #[test]
    fn round_trip_verifies() {
        let (env, keys) = signed();
        assert_eq!(verify(&env, &keys).unwrap(), manifest());
    }

    #[test]
    fn tampered_manifest_bytes_are_refused() {
        let (mut env, keys) = signed();
        let mut bytes = B64.decode(&env.manifest).unwrap();
        // Flip the version string's last digit: still valid JSON, wrong bytes.
        let s = String::from_utf8(bytes.clone())
            .unwrap()
            .replace("1.25.0", "9.99.0");
        bytes = s.into_bytes();
        env.manifest = B64.encode(&bytes);
        assert_eq!(verify(&env, &keys), Err(VerifyError::BadSignature));
    }

    #[test]
    fn tampered_signature_is_refused() {
        let (mut env, keys) = signed();
        let mut sig = B64.decode(&env.signature).unwrap();
        sig[0] ^= 0x01;
        env.signature = B64.encode(&sig);
        assert_eq!(verify(&env, &keys), Err(VerifyError::BadSignature));
    }

    #[test]
    fn unknown_key_id_is_refused() {
        let (mut env, keys) = signed();
        env.key_id = "test-b".into();
        assert_eq!(verify(&env, &keys), Err(VerifyError::UnknownKeyId));
    }

    #[test]
    fn a_foreign_key_does_not_verify() {
        let (env, _) = signed();
        let other = SigningKey::from_bytes(&[9u8; 32]);
        assert_eq!(
            verify(&env, &trusted("test-a", &other)),
            Err(VerifyError::BadSignature)
        );
    }

    #[test]
    fn unknown_schema_is_refused() {
        let k = key();
        let mut m = manifest();
        m.schema = SCHEMA_VERSION + 1;
        let env = sign(&serde_json::to_vec(&m).unwrap(), &k, "test-a", 25);
        assert_eq!(
            verify(&env, &trusted("test-a", &k)),
            Err(VerifyError::UnsupportedSchema)
        );
    }

    #[test]
    fn non_manifest_payload_is_refused_after_the_signature_passes() {
        let k = key();
        let env = sign(b"not json at all", &k, "test-a", 0);
        assert_eq!(
            verify(&env, &trusted("test-a", &k)),
            Err(VerifyError::MalformedManifest)
        );
    }

    #[test]
    fn malformed_base64_is_refused() {
        let (mut env, keys) = signed();
        env.manifest = "!!! not base64 !!!".into();
        assert_eq!(verify(&env, &keys), Err(VerifyError::MalformedBase64));

        let (mut env, keys) = signed();
        env.signature = "%%%".into();
        assert_eq!(verify(&env, &keys), Err(VerifyError::MalformedBase64));
    }

    #[test]
    fn short_signature_is_refused() {
        let (mut env, keys) = signed();
        env.signature = B64.encode([0u8; 8]);
        assert_eq!(verify(&env, &keys), Err(VerifyError::MalformedSignature));
    }

    #[test]
    fn env_value_parses_a_key_set() {
        let k = key();
        let value = format!("test-a={}", B64.encode(k.verifying_key().to_bytes()));
        let keys = trusted_keys_from_env_value(&value).unwrap();
        assert_eq!(keys.len(), 1);
        let (env, _) = signed();
        assert!(verify(&env, &keys).is_ok());
    }

    #[test]
    fn env_value_rejects_a_malformed_entry() {
        assert!(trusted_keys_from_env_value("no-equals-sign").is_err());
        assert!(trusted_keys_from_env_value("id=not-a-key").is_err());
        // Empty (and whitespace-only) input is a valid *empty* set: every ingest
        // then fails closed rather than the process failing to start.
        assert!(trusted_keys_from_env_value("").unwrap().is_empty());
    }

    #[test]
    fn signing_key_round_trips_through_base64() {
        let k = key();
        let restored = signing_key_from_base64(&B64.encode(k.to_bytes())).unwrap();
        assert_eq!(restored.to_bytes(), k.to_bytes());
        assert!(signing_key_from_base64("short").is_err());
    }

    #[test]
    fn trusted_keys_from_pairs_builds_the_set() {
        let k = key();
        let b64 = B64.encode(k.verifying_key().to_bytes());
        let keys = trusted_keys_from_pairs([("test-a", b64.as_str())]).unwrap();
        assert!(keys.contains_key("test-a"));
        assert!(trusted_keys_from_pairs([("bad", "zz")]).is_err());
    }
}
