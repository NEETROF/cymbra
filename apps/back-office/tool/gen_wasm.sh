#!/usr/bin/env bash
# Build the back-office's WebAssembly modules from Rust and stage the SoundFont:
#   - cymbra-musicxml-wasm  → src/wasm/pkg/       (notation render + playback schedule)
#   - cymbra-audio-wasm      → src/wasm/pkg-audio/ (rustysynth PCM render)
#   - the app's SoundFont    → public/soundfonts/  (fetched on demand for playback)
# All outputs are gitignored (like the gRPC stubs from gen_proto.sh). The `--target
# web` glue fetches the sibling _bg.wasm via new URL(..., import.meta.url), which Vite
# resolves as an asset.
#
# Run via `yarn gen:wasm`. Needs the wasm toolchain: `wasm-pack` and the
# `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`). CI
# builds this in GitHub Actions (the Cloudflare Pages image has no Rust toolchain,
# exactly like `yarn gen`).
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

# --no-pack: skip the npm package.json (we import the glue directly, not as a dep).
build() {
  local crate="$1" out="$2" name="$3"
  rm -rf "$out"
  wasm-pack build "$REPO_ROOT/crates/$crate" \
    --target web --out-dir "$out" --out-name "$name" --no-pack --release
}

build musicxml-wasm "$APP_DIR/src/wasm/pkg" musicxml_wasm
build audio-wasm "$APP_DIR/src/wasm/pkg-audio" audio_wasm

# Stage the app's SoundFont (single committed copy under apps/music/assets — do NOT
# duplicate it in git; copy it at build time). Served same-origin under font/connect
# CSP; fetched on demand + cached when a moderator hits Play.
SF2="UprightPianoKW-20220221.sf2"
mkdir -p "$APP_DIR/public/soundfonts"
cp "$REPO_ROOT/apps/music/assets/soundfonts/$SF2" "$APP_DIR/public/soundfonts/$SF2"

echo "Built wasm modules (notation + audio) and staged the SoundFont"
