#!/usr/bin/env bash
# Build the WebAssembly notation renderer from the Rust crate `cymbra-musicxml-wasm`
# into src/wasm/pkg/ (gitignored, like the gRPC stubs from gen_proto.sh). The crate
# is a thin wrapper over cymbra-musicxml-core exposing `render(bytes, width)`; this
# emits the `--target web` ES glue + optimized .wasm that Vite loads on demand.
#
# Run via `yarn gen:wasm`. Needs the wasm toolchain: `wasm-pack` and the
# `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`). CI
# builds this in GitHub Actions (the Cloudflare Pages image has no Rust toolchain,
# exactly like `yarn gen`).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$APP_DIR/../.." && pwd)"

CRATE_DIR="$REPO_ROOT/crates/musicxml-wasm"
OUT_DIR="$APP_DIR/src/wasm/pkg"

command -v wasm-pack >/dev/null 2>&1 || {
  echo "error: wasm-pack not found on PATH (cargo install wasm-pack)" >&2
  exit 1
}
rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$' || {
  echo "error: wasm32 target missing (rustup target add wasm32-unknown-unknown)" >&2
  exit 1
}

rm -rf "$OUT_DIR"

# --target web: ES module glue that fetches the sibling _bg.wasm via
#   new URL('..._bg.wasm', import.meta.url) — Vite resolves that as an asset.
# --no-pack: skip the npm package.json (we import the glue directly, not as a dep).
wasm-pack build "$CRATE_DIR" \
  --target web \
  --out-dir "$OUT_DIR" \
  --out-name musicxml_wasm \
  --no-pack \
  --release

echo "Built wasm notation renderer into $OUT_DIR"
