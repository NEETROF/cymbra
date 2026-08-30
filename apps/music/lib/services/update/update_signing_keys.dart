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

/// The desktop-update signing keys this build trusts (change:
/// add-desktop-auto-update, design D2).
///
/// A **set keyed by `key_id`, not a single value** — that is what makes a key
/// rotation survivable. A rotation ships a build trusting both the outgoing and
/// the incoming key, waits for it to spread, and only then drops the old one.
/// With a single compiled-in key there is no overlap window and every client
/// still on the previous build stops accepting releases the moment CI switches.
///
/// These are **public** keys: they verify, they cannot sign. The private half
/// exists only as the `DESKTOP_UPDATE_SIGNING_KEY` GitHub Actions secret and an
/// out-of-band backup — deliberately not on the backend, so a compromised
/// backend can withhold or stall an update but never fabricate one.
///
/// Keep in sync with the backend's `CYMBRA_UPDATE_TRUSTED_KEYS` and with
/// `apps/music/config/desktop_release.json`'s `key_id`.
library;

/// `key_id` → base64 32-byte Ed25519 public key.
const Map<String, String> kUpdateTrustedKeys = {
  '2026-08-a': 'CPn7pYhTnCufY1/ptXO2l1ICbdXRnEUchv37/NMBSjY=',
};
