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

//! `update-sign` — the CI signing binary (change: add-desktop-auto-update,
//! task 6.1).
//!
//! Reads a manifest JSON document on **stdin**, signs the exact bytes it will
//! ship (the document is re-serialized once into its canonical field order and
//! that result is what gets both signed and encoded — never signed once and
//! serialized again), and writes the envelope to **stdout**.
//!
//! ```text
//! update-sign --key-id 2026-08-a --rollout 0 < manifest.json > envelope.json
//! ```
//!
//! The signing secret comes from `DESKTOP_UPDATE_SIGNING_KEY` (base64, 32
//! bytes) so it never appears in a process argument list.

use std::io::{Read as _, Write as _};

use cymbra_update_manifest::{Manifest, sign, signing_key_from_base64};

fn main() {
    if let Err(msg) = run() {
        eprintln!("update-sign: {msg}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut key_id = None;
    let mut rollout: u8 = 0;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--key-id" => key_id = args.next(),
            "--rollout" => {
                rollout = args
                    .next()
                    .ok_or("--rollout needs a value")?
                    .parse()
                    .map_err(|_| "--rollout must be 0..=100")?;
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let key_id = key_id.ok_or("--key-id is required")?;
    if rollout > 100 {
        return Err("--rollout must be 0..=100".into());
    }
    let secret = std::env::var("DESKTOP_UPDATE_SIGNING_KEY")
        .map_err(|_| "DESKTOP_UPDATE_SIGNING_KEY is not set")?;
    let signing = signing_key_from_base64(&secret).map_err(|e| e.to_string())?;

    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    // Parse then re-serialize once: the bytes we sign ARE the bytes we encode,
    // and a malformed manifest fails here rather than at the client.
    let manifest: Manifest = serde_json::from_str(&input).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(&manifest).map_err(|e| e.to_string())?;
    let envelope = sign(&bytes, &signing, &key_id, rollout);

    let out = serde_json::to_string_pretty(&envelope).map_err(|e| e.to_string())?;
    let mut stdout = std::io::stdout();
    stdout
        .write_all(out.as_bytes())
        .map_err(|e| e.to_string())?;
    stdout.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(())
}
