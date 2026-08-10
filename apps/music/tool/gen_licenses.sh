#!/usr/bin/env bash
# Generate the third-party Rust license notices bundled by the app (change:
# add-oss-license-attributions). Walks the dependency tree rooted at
# apps/music/rust's Cargo.toml (NOT the whole workspace, so backend-only
# crates never leak in) via `cargo about`, then reshapes its verbose JSON
# into the compact { licenses: [{id, name, text, packages}] } asset the app
# parses at startup (see lib/services/license_notices.dart).
#
# Run via `melos run gen-licenses`. Requires `cargo about` on PATH:
#   cargo install cargo-about --locked --features cli
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$APP_DIR/rust"
OUT_FILE="$APP_DIR/assets/generated/rust_licenses.json"

command -v cargo-about >/dev/null 2>&1 || {
  echo "error: cargo-about not found on PATH (cargo install cargo-about --locked --features cli)" >&2
  exit 1
}

RAW_JSON="$(mktemp)"
trap 'rm -f "$RAW_JSON"' EXIT

(cd "$RUST_DIR" && cargo about generate --format json -o "$RAW_JSON")

python3 "$SCRIPT_DIR/reshape_licenses.py" "$RAW_JSON" "$OUT_FILE"

echo "Generated Rust license notices into $OUT_FILE"
