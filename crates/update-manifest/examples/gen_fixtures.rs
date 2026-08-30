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

//! Regenerates the checked-in golden fixture (task 1.7):
//! `cargo run -p cymbra-update-manifest --example gen_fixtures`.
//!
//! The fixture is signed with a **fixed test secret**, never the release key, so
//! the output is byte-reproducible and safe to commit. Both the Rust tests and
//! the Dart tests read these files: an Ed25519/base64/schema disagreement
//! between the two implementations then fails a unit test instead of an install.

use std::collections::BTreeMap;
use std::path::PathBuf;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use cymbra_update_manifest::{Envelope, Manifest, SCHEMA_VERSION, Target, sign};
use ed25519_dalek::SigningKey;

/// Fixed, non-secret test key. NOT the release key.
const FIXTURE_SECRET: [u8; 32] = [42u8; 32];
const KEY_ID: &str = "golden-1";

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    std::fs::create_dir_all(&dir).unwrap();
    let key = SigningKey::from_bytes(&FIXTURE_SECRET);

    let manifest = Manifest {
        schema: SCHEMA_VERSION,
        product: "music".into(),
        channel: "stable".into(),
        version: "1.25.0+34".into(),
        released_at: "2026-08-21T10:00:00Z".into(),
        min_supported_version: Some("1.20.0+27".into()),
        notes_url: Some("https://github.com/NEETROF/cymbra/releases/tag/music-v1.25.0".into()),
        targets: BTreeMap::from([
            (
                "windows-x64".to_string(),
                Target {
                    kind: "inno-setup".into(),
                    url: "https://example.invalid/cymbra-music-1.25.0-setup.exe".into(),
                    size: 48_123_904,
                    sha256: "3b1f2c9d4e5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f90a1b2"
                        .into(),
                },
            ),
            (
                "linux-x64".to_string(),
                Target {
                    kind: "appimage".into(),
                    url: "https://example.invalid/CymbraMusic-1.25.0-x86_64.AppImage".into(),
                    size: 52_428_800,
                    sha256: "c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f9"
                        .into(),
                },
            ),
        ]),
    };

    let bytes = serde_json::to_vec(&manifest).unwrap();
    let valid = sign(&bytes, &key, KEY_ID, 25);

    write(
        &dir,
        "manifest.json",
        &serde_json::to_string_pretty(&manifest).unwrap(),
    );
    write(
        &dir,
        "trusted_keys.json",
        &serde_json::to_string_pretty(&BTreeMap::from([(
            KEY_ID,
            B64.encode(key.verifying_key().to_bytes()),
        )]))
        .unwrap(),
    );
    write_env(&dir, "envelope_valid.json", &valid);

    // Tampered manifest bytes: valid JSON, different version, same signature.
    let swapped = String::from_utf8(bytes.clone())
        .unwrap()
        .replace("1.25.0+34", "9.99.9+99");
    let mut tampered_manifest = valid.clone();
    tampered_manifest.manifest = B64.encode(swapped.as_bytes());
    write_env(&dir, "envelope_tampered_manifest.json", &tampered_manifest);

    // Tampered signature: one flipped bit.
    let mut sig = B64.decode(&valid.signature).unwrap();
    sig[0] ^= 0x01;
    let mut tampered_signature = valid.clone();
    tampered_signature.signature = B64.encode(&sig);
    write_env(
        &dir,
        "envelope_tampered_signature.json",
        &tampered_signature,
    );

    // A key id nothing trusts.
    let mut unknown_key = valid.clone();
    unknown_key.key_id = "not-a-trusted-key".into();
    write_env(&dir, "envelope_unknown_key_id.json", &unknown_key);

    // A correctly signed manifest from a future schema.
    let mut future = manifest.clone();
    future.schema = SCHEMA_VERSION + 1;
    let future_bytes = serde_json::to_vec(&future).unwrap();
    write_env(
        &dir,
        "envelope_unknown_schema.json",
        &sign(&future_bytes, &key, KEY_ID, 25),
    );

    // Rollout 0 — the kill-switch shape the client must refuse to offer.
    let mut killed = valid.clone();
    killed.rollout_percent = 0;
    write_env(&dir, "envelope_rollout_zero.json", &killed);

    eprintln!("fixtures written to {}", dir.display());
}

fn write_env(dir: &std::path::Path, name: &str, env: &Envelope) {
    write(dir, name, &serde_json::to_string_pretty(env).unwrap());
}

fn write(dir: &std::path::Path, name: &str, body: &str) {
    std::fs::write(dir.join(name), format!("{body}\n")).unwrap();
}
