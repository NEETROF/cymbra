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

//! The Rust half of the cross-language golden-fixture test (tasks 1.7, 9.7).
//!
//! `apps/music/test/services/update/update_manifest_test.dart` reads exactly
//! these files and asserts exactly these outcomes. Two independent Ed25519
//! implementations that disagree fail here, not on a user's machine.

use std::collections::BTreeMap;
use std::path::PathBuf;

use cymbra_update_manifest::{Envelope, VerifyError, trusted_keys_from_pairs, verify};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn envelope(name: &str) -> Envelope {
    let raw = std::fs::read_to_string(fixtures().join(name)).expect("fixture present");
    serde_json::from_str(&raw).expect("fixture parses")
}

fn trusted() -> cymbra_update_manifest::TrustedKeys {
    let raw = std::fs::read_to_string(fixtures().join("trusted_keys.json")).unwrap();
    let map: BTreeMap<String, String> = serde_json::from_str(&raw).unwrap();
    trusted_keys_from_pairs(map.iter().map(|(k, v)| (k.as_str(), v.as_str()))).unwrap()
}

#[test]
fn valid_envelope_verifies_and_carries_both_targets() {
    let manifest = verify(&envelope("envelope_valid.json"), &trusted()).unwrap();
    assert_eq!(manifest.product, "music");
    assert_eq!(manifest.channel, "stable");
    assert_eq!(manifest.version, "1.25.0+34");
    assert_eq!(manifest.min_supported_version.as_deref(), Some("1.20.0+27"));
    assert_eq!(manifest.targets.len(), 2);
    assert_eq!(manifest.targets["windows-x64"].kind, "inno-setup");
    assert_eq!(manifest.targets["linux-x64"].kind, "appimage");
    assert_eq!(manifest.targets["windows-x64"].size, 48_123_904);
}

#[test]
fn tampered_manifest_bytes_are_refused() {
    assert_eq!(
        verify(&envelope("envelope_tampered_manifest.json"), &trusted()),
        Err(VerifyError::BadSignature)
    );
}

#[test]
fn tampered_signature_is_refused() {
    assert_eq!(
        verify(&envelope("envelope_tampered_signature.json"), &trusted()),
        Err(VerifyError::BadSignature)
    );
}

#[test]
fn unknown_key_id_is_refused() {
    assert_eq!(
        verify(&envelope("envelope_unknown_key_id.json"), &trusted()),
        Err(VerifyError::UnknownKeyId)
    );
}

#[test]
fn unknown_schema_is_refused_even_though_the_signature_is_good() {
    assert_eq!(
        verify(&envelope("envelope_unknown_schema.json"), &trusted()),
        Err(VerifyError::UnsupportedSchema)
    );
}

#[test]
fn rollout_zero_still_verifies_the_gate_is_client_side() {
    let env = envelope("envelope_rollout_zero.json");
    assert_eq!(env.rollout_percent, 0);
    // Verification says nothing about whether the update is *offered*: rollout is
    // outside the signature and evaluated by the client.
    assert!(verify(&env, &trusted()).is_ok());
}
