#!/usr/bin/env bash
# Build the Lingua wasm bindings from Rust into the extension:
#   crates/lingua-wasm  →  src/wasm/pkg/   (thin wasm-bindgen surface over lingua-core)
#
# Output is gitignored (like the back office's generated wasm). The `--target web`
# glue fetches its sibling _bg.wasm via new URL(..., import.meta.url); the content
# script loads the glue at runtime through chrome.runtime.getURL. Run via
# `yarn gen:wasm`. Needs wasm-pack + the wasm32-unknown-unknown target.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$APP_DIR/../.." && pwd)"

command -v wasm-pack >/dev/null 2>&1 || {
  echo "error: wasm-pack not found on PATH (cargo install wasm-pack)" >&2
  exit 1
}
rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$' || {
  echo "error: wasm32 target missing (rustup target add wasm32-unknown-unknown)" >&2
  exit 1
}

OUT="$APP_DIR/src/wasm/pkg" # absolute: wasm-pack resolves --out-dir relative to the crate
rm -rf "$OUT"
wasm-pack build "$REPO_ROOT/crates/lingua-wasm" \
  --target web --out-dir "$OUT" --out-name lingua_wasm --no-pack --release

echo "Built lingua wasm bindings into src/wasm/pkg"
