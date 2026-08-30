# Golden update-manifest fixture

Cross-language contract test (change: add-desktop-auto-update, decision D7).
The Rust verifier (`tests/golden.rs`) and the Dart verifier
(`apps/music/test/services/update/update_manifest_test.dart`) both read these
files and assert the same outcomes, so a drift between the two independent
Ed25519 implementations fails a unit test rather than an install.

Signed with a **fixed test secret** (`[42u8; 32]`), never the release key — safe
to commit, and byte-reproducible.

Regenerate with:

    cargo run -p cymbra-update-manifest --example gen_fixtures

| File | Expected outcome |
|---|---|
| `manifest.json` | the plaintext manifest, for reference |
| `trusted_keys.json` | `key_id` → base64 public key, the trusted set both tests load |
| `envelope_valid.json` | verifies |
| `envelope_tampered_manifest.json` | bad signature |
| `envelope_tampered_signature.json` | bad signature |
| `envelope_unknown_key_id.json` | unknown key id |
| `envelope_unknown_schema.json` | unsupported schema (signature is good) |
| `envelope_rollout_zero.json` | verifies; the rollout gate is client-side |
