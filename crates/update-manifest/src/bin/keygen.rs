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

//! `update-keygen` — generate a desktop-update release keypair
//! (change: add-desktop-auto-update, task 1.6).
//!
//! The **private key never enters the repository**. Run this once, store the
//! secret as the `DESKTOP_UPDATE_SIGNING_KEY` GitHub Actions secret and back it
//! up out of band (losing it means clients stop accepting new releases until a
//! build ships with a new key), then paste the public key into:
//!
//!   * `apps/music/lib/services/update/update_signing_keys.dart` (compiled in), and
//!   * `CYMBRA_UPDATE_TRUSTED_KEYS` in the backend environment.
//!
//! The secret is written to stdout and the public key to stderr, so
//! `update-keygen > key.secret` keeps the secret out of a shared terminal log.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use ed25519_dalek::SigningKey;

fn main() {
    let key_id = std::env::args().nth(1).unwrap_or_else(|| "1".to_string());
    let signing = SigningKey::generate(&mut rand_core::OsRng);
    eprintln!("key_id     : {key_id}");
    eprintln!(
        "public key : {}",
        B64.encode(signing.verifying_key().to_bytes())
    );
    eprintln!(
        "trusted-keys env value : {key_id}={}",
        B64.encode(signing.verifying_key().to_bytes())
    );
    eprintln!("(the secret key is on stdout — redirect it to a file, never paste it)");
    println!("{}", B64.encode(signing.to_bytes()));
}
