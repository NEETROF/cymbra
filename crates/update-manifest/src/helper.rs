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

//! The decision the Linux privileged update helper takes before installing
//! anything (change: add-desktop-auto-update, design D5).
//!
//! The helper runs as root behind the system password dialog, and everything it
//! is handed comes from a process running as the user — which is exactly what
//! malware running as the user can also write. So the decision trusts nothing
//! from the caller except the bytes themselves: the envelope is verified against
//! the keys **compiled into the helper**, and the package bytes must match the
//! `linux-x64` `deb` target of that verified manifest. Pure: no I/O, so every
//! refusal is unit-tested on the same golden fixtures as the backend.

use sha2::{Digest, Sha256};

use crate::{Envelope, Manifest, TrustedKeys, VerifyError, verify};

/// The product the helper may install. A correctly signed manifest for another
/// product (Live, one day) must never be installed under Music's package name.
pub const PRODUCT: &str = "music";

/// The manifest target a Debian package is published under.
pub const TARGET: &str = "linux-x64";

/// The install kind of that target.
pub const KIND: &str = "deb";

/// Why the helper refused. Every variant means "nothing was installed".
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HelperRefusal {
    /// The envelope bytes are not an envelope.
    #[error("malformed envelope")]
    MalformedEnvelope,
    /// The envelope failed verification against the helper's own keys.
    #[error("envelope does not verify: {0}")]
    Verify(#[from] VerifyError),
    /// Signed, but for another product.
    #[error("manifest is for another product")]
    WrongProduct,
    /// Signed, but carries no `linux-x64` target of kind `deb`.
    #[error("manifest has no linux-x64 deb target")]
    NoPackageTarget,
    /// The package bytes are not the size the manifest declares.
    #[error("package size does not match the manifest")]
    SizeMismatch,
    /// The package bytes do not hash to the manifest's SHA-256.
    #[error("package hash does not match the manifest")]
    HashMismatch,
    /// A version that cannot be ordered (manifest or installed package).
    #[error("unreadable version")]
    UnreadableVersion,
    /// Not strictly newer than what is installed: a replayed older manifest must
    /// never move a machine backwards.
    #[error("not newer than the installed version")]
    NotNewer,
}

/// Decide whether `package` may be installed over `installed_version`.
///
/// Returns the verified manifest on success. The order is fixed: envelope →
/// signature (inside [`verify`]) → product → target → size → hash → version.
pub fn check_package(
    envelope_json: &[u8],
    package: &[u8],
    trusted: &TrustedKeys,
    installed_version: &str,
) -> Result<Manifest, HelperRefusal> {
    let envelope: Envelope =
        serde_json::from_slice(envelope_json).map_err(|_| HelperRefusal::MalformedEnvelope)?;
    let manifest = verify(&envelope, trusted)?;
    if manifest.product != PRODUCT {
        return Err(HelperRefusal::WrongProduct);
    }
    let target = manifest
        .targets
        .get(TARGET)
        .filter(|t| t.kind == KIND)
        .ok_or(HelperRefusal::NoPackageTarget)?;
    if package.len() as u64 != target.size {
        return Err(HelperRefusal::SizeMismatch);
    }
    let digest = hex(&Sha256::digest(package));
    if !digest.eq_ignore_ascii_case(&target.sha256) {
        return Err(HelperRefusal::HashMismatch);
    }
    let offered = version_key(&manifest.version).ok_or(HelperRefusal::UnreadableVersion)?;
    let current = version_key(installed_version).ok_or(HelperRefusal::UnreadableVersion)?;
    if offered <= current {
        return Err(HelperRefusal::NotNewer);
    }
    Ok(manifest)
}

/// Total order over the app's version format, `major.minor.patch+build` — the
/// same rules as the app's `AppVersion` and the backend's `version_order`: the
/// triple first, then the build number (absent = 0). `None` for anything else.
pub fn version_key(version: &str) -> Option<(u64, u64, u64, u64)> {
    let (triple, build) = match version.trim().split_once('+') {
        Some((t, b)) => (t, b.parse().ok()?),
        None => (version.trim(), 0),
    };
    let mut parts = triple.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch, build))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SCHEMA_VERSION, Target, sign};
    use ed25519_dalek::SigningKey;
    use std::collections::BTreeMap;

    const PACKAGE: &[u8] = b"pretend this is a .deb";

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn trusted() -> TrustedKeys {
        TrustedKeys::from([("k1".to_string(), key().verifying_key())])
    }

    fn manifest(product: &str, version: &str, kind: &str, bytes: &[u8]) -> Manifest {
        Manifest {
            schema: SCHEMA_VERSION,
            product: product.into(),
            channel: "stable".into(),
            version: version.into(),
            released_at: "2026-10-24T00:00:00Z".into(),
            min_supported_version: None,
            notes_url: None,
            targets: BTreeMap::from([(
                TARGET.to_string(),
                Target {
                    kind: kind.into(),
                    url: "https://example.invalid/cymbra-music_1.40.0_amd64.deb".into(),
                    size: bytes.len() as u64,
                    sha256: hex(&Sha256::digest(bytes)),
                },
            )]),
        }
    }

    fn envelope(m: &Manifest, signer: &SigningKey, key_id: &str) -> Vec<u8> {
        let bytes = serde_json::to_vec(m).unwrap();
        serde_json::to_vec(&sign(&bytes, signer, key_id, 0)).unwrap()
    }

    fn valid() -> Vec<u8> {
        envelope(&manifest("music", "1.40.0+50", KIND, PACKAGE), &key(), "k1")
    }

    #[test]
    fn a_signed_newer_matching_package_is_accepted() {
        let m = check_package(&valid(), PACKAGE, &trusted(), "1.39.0+49").unwrap();
        assert_eq!(m.version, "1.40.0+50");
    }

    #[test]
    fn a_substituted_package_is_refused() {
        let swapped = b"pretend this is a .deX";
        assert_eq!(
            check_package(&valid(), swapped, &trusted(), "1.39.0+49"),
            Err(HelperRefusal::HashMismatch)
        );
        assert_eq!(
            check_package(&valid(), b"short", &trusted(), "1.39.0+49"),
            Err(HelperRefusal::SizeMismatch)
        );
    }

    #[test]
    fn keys_from_the_caller_are_never_used() {
        // Signed by a key the helper does not hold, under the id it does hold.
        let forged = envelope(
            &manifest("music", "1.40.0+50", KIND, PACKAGE),
            &SigningKey::from_bytes(&[9u8; 32]),
            "k1",
        );
        assert_eq!(
            check_package(&forged, PACKAGE, &trusted(), "1.39.0+49"),
            Err(HelperRefusal::Verify(VerifyError::BadSignature))
        );
        let unknown = envelope(&manifest("music", "1.40.0+50", KIND, PACKAGE), &key(), "k2");
        assert_eq!(
            check_package(&unknown, PACKAGE, &trusted(), "1.39.0+49"),
            Err(HelperRefusal::Verify(VerifyError::UnknownKeyId))
        );
    }

    #[test]
    fn another_product_or_kind_is_refused() {
        let live = envelope(&manifest("live", "1.40.0+50", KIND, PACKAGE), &key(), "k1");
        assert_eq!(
            check_package(&live, PACKAGE, &trusted(), "1.39.0+49"),
            Err(HelperRefusal::WrongProduct)
        );
        let appimage = envelope(
            &manifest("music", "1.40.0+50", "appimage", PACKAGE),
            &key(),
            "k1",
        );
        assert_eq!(
            check_package(&appimage, PACKAGE, &trusted(), "1.39.0+49"),
            Err(HelperRefusal::NoPackageTarget)
        );
    }

    #[test]
    fn nothing_that_is_not_strictly_newer_is_installed() {
        for installed in ["1.40.0+50", "1.40.0+51", "2.0.0"] {
            assert_eq!(
                check_package(&valid(), PACKAGE, &trusted(), installed),
                Err(HelperRefusal::NotNewer),
                "installed {installed}"
            );
        }
        assert_eq!(
            check_package(&valid(), PACKAGE, &trusted(), "not-a-version"),
            Err(HelperRefusal::UnreadableVersion)
        );
    }

    #[test]
    fn garbage_is_not_an_envelope() {
        assert_eq!(
            check_package(b"{", PACKAGE, &trusted(), "1.0.0"),
            Err(HelperRefusal::MalformedEnvelope)
        );
    }

    #[test]
    fn the_golden_fixture_is_refused_as_it_targets_an_appimage() {
        // The cross-language fixture predates the .deb; it must still verify (the
        // signature path is shared) yet never be installable by the helper.
        let envelope = include_bytes!("../fixtures/envelope_valid.json");
        let keys: BTreeMap<String, String> =
            serde_json::from_str(include_str!("../fixtures/trusted_keys.json")).unwrap();
        let trusted =
            crate::trusted_keys_from_pairs(keys.iter().map(|(k, v)| (k.as_str(), v.as_str())))
                .unwrap();
        assert_eq!(
            check_package(envelope, PACKAGE, &trusted, "1.0.0"),
            Err(HelperRefusal::NoPackageTarget)
        );
    }

    #[test]
    fn versions_order_numerically_with_the_build_as_tiebreaker() {
        assert!(version_key("1.10.0").unwrap() > version_key("1.9.0+99").unwrap());
        assert!(version_key("1.25.0+34").unwrap() > version_key("1.25.0+33").unwrap());
        assert_eq!(version_key("1.25.0"), version_key("1.25.0+0"));
        assert_eq!(version_key("1.2"), None);
        assert_eq!(version_key("1.2.3.4"), None);
        assert_eq!(version_key("1.2.3+x"), None);
    }
}
