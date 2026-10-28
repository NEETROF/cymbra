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

//! The download page's view of a release (change: add-music-download-page,
//! task 2.2): what a person needs to pick and check a file — version, date,
//! notes link, and per target the kind, URL, size and SHA-256.
//!
//! Projected **only** from a manifest that has just been re-verified (design
//! D4): a compromised backend can withhold a release from the page, not invent
//! one. Nothing here carries the signature, the key id or the rollout — those
//! belong to the updater's feed, not to a page.

use std::collections::BTreeMap;

use cymbra_update_manifest::{Manifest, TrustedKeys};
use serde::Serialize;

use crate::Release;

/// The `200` body of `GET /updates/desktop/latest`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DownloadView {
    /// `major.minor.patch+build`, as signed.
    pub version: String,
    /// RFC 3339, as signed.
    pub released_at: String,
    /// The GitHub Release page, when the manifest names one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes_url: Option<String>,
    /// By `<os>-<arch>`, in the manifest's (sorted) order.
    pub targets: BTreeMap<String, DownloadTarget>,
}

/// One downloadable file of a [`DownloadView`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DownloadTarget {
    pub kind: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
}

impl From<Manifest> for DownloadView {
    fn from(m: Manifest) -> Self {
        Self {
            version: m.version,
            released_at: m.released_at,
            notes_url: m.notes_url,
            targets: m
                .targets
                .into_iter()
                .map(|(key, t)| {
                    (
                        key,
                        DownloadTarget {
                            kind: t.kind,
                            url: t.url,
                            size: t.size,
                            sha256: t.sha256,
                        },
                    )
                })
                .collect(),
        }
    }
}

/// Re-verify a stored row and project it, or `None` when it no longer verifies
/// against any trusted key — or verifies but describes another product or
/// channel than the one it is stored under, which only a tampered table could
/// produce. Either way the page falls back; no field of the row is served.
pub fn verified_view(
    release: &Release,
    trusted: &TrustedKeys,
    product: &str,
    channel: &str,
) -> Option<DownloadView> {
    let manifest = cymbra_update_manifest::verify(&release.to_envelope(), trusted).ok()?;
    if manifest.product != product || manifest.channel != channel {
        return None;
    }
    Some(manifest.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version_order;
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD as B64;
    use cymbra_update_manifest::{SCHEMA_VERSION, Target, sign};
    use ed25519_dalek::SigningKey;

    const KEY_ID: &str = "test-a";

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[3u8; 32])
    }

    fn trusted() -> TrustedKeys {
        TrustedKeys::from([(KEY_ID.to_string(), key().verifying_key())])
    }

    fn manifest() -> Manifest {
        Manifest {
            schema: SCHEMA_VERSION,
            product: "music".into(),
            channel: "stable".into(),
            version: "1.37.0+51".into(),
            released_at: "2026-10-28T10:00:00Z".into(),
            min_supported_version: Some("1.30.0+40".into()),
            notes_url: Some(
                "https://github.com/NEETROF/cymbra/releases/tag/music-v1.37.0".into(),
            ),
            targets: BTreeMap::from([(
                "linux-x64".to_string(),
                Target {
                    kind: "appimage".into(),
                    url: "https://github.com/NEETROF/cymbra/releases/download/music-v1.37.0/Cymbra-Music-x86_64.AppImage".into(),
                    size: 98_765_432,
                    sha256: "ab".repeat(32),
                },
            )]),
        }
    }

    fn stored(m: &Manifest, signer: &SigningKey) -> Release {
        let env = sign(&serde_json::to_vec(m).unwrap(), signer, KEY_ID, 0);
        Release {
            product: "music".into(),
            channel: "stable".into(),
            version: m.version.clone(),
            version_order: version_order(&m.version).unwrap(),
            manifest: env.manifest,
            signature: env.signature,
            key_id: env.key_id,
            rollout_percent: 0,
            paused: false,
        }
    }

    #[test]
    fn a_verified_row_is_projected() {
        let view =
            verified_view(&stored(&manifest(), &key()), &trusted(), "music", "stable").unwrap();
        assert_eq!(view.version, "1.37.0+51");
        assert_eq!(view.released_at, "2026-10-28T10:00:00Z");
        let t = &view.targets["linux-x64"];
        assert_eq!(t.kind, "appimage");
        assert_eq!(t.size, 98_765_432);
        assert_eq!(t.sha256, "ab".repeat(32));
    }

    #[test]
    fn no_signature_key_or_rollout_field_leaks() {
        let view =
            verified_view(&stored(&manifest(), &key()), &trusted(), "music", "stable").unwrap();
        let json: serde_json::Value = serde_json::to_value(&view).unwrap();
        let keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["notes_url", "released_at", "targets", "version"]);
        let target: Vec<&str> = json["targets"]["linux-x64"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(target, ["kind", "sha256", "size", "url"]);
    }

    #[test]
    fn absent_notes_are_omitted_not_null() {
        let mut m = manifest();
        m.notes_url = None;
        let view = verified_view(&stored(&m, &key()), &trusted(), "music", "stable").unwrap();
        let json = serde_json::to_value(&view).unwrap();
        assert!(json.get("notes_url").is_none());
    }

    #[test]
    fn a_row_signed_by_a_foreign_key_is_not_served() {
        let other = SigningKey::from_bytes(&[8u8; 32]);
        assert!(
            verified_view(&stored(&manifest(), &other), &trusted(), "music", "stable").is_none()
        );
    }

    #[test]
    fn a_tampered_row_is_not_served() {
        let mut row = stored(&manifest(), &key());
        let swapped = String::from_utf8(B64.decode(&row.manifest).unwrap())
            .unwrap()
            .replace("NEETROF/cymbra", "evil/cymbra");
        row.manifest = B64.encode(swapped);
        assert!(verified_view(&row, &trusted(), "music", "stable").is_none());
    }

    #[test]
    fn a_row_filed_under_another_product_is_not_served() {
        let row = stored(&manifest(), &key());
        assert!(verified_view(&row, &trusted(), "live", "stable").is_none());
        assert!(verified_view(&row, &trusted(), "music", "beta").is_none());
    }
}
