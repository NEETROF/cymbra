#!/usr/bin/env bash
# Build the translation engine from source into <dir> (default: apps/lingua-extension/engine),
# exactly as the lingua-engine-build workflow does — it runs this script — and check the result
# against engine-pin.json. This is the recipe a reviewer follows to reproduce
# engine/bergamot-translator.wasm and engine/bergamot-translator.js from the packaged add-on.
#
# What it builds: mozilla/translations at the commit pinned in engine-pin.json (Bergamot v0.6.0),
# with the upstream script inference/scripts/build-wasm.py, which fetches the Emscripten SDK
# version its submodules pin (3.1.8). Nothing is patched.
#
# Requires Linux x86-64 (the upstream script warns that it breaks on macOS AArch64), git,
# Python 3.11, cmake and a C++ toolchain. About eight minutes on four cores. The upstream script
# refuses to run outside its Docker image unless ALLOW_RUN_ON_HOST=1, which this sets: run it in
# a throwaway machine or container.
#
# Usage: tool/build_engine.sh [<dir>]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="${1:-$APP/engine}"
WORK="${ENGINE_WORK_DIR:-$(mktemp -d)}"
COMMIT=$(node -p "require('$APP/engine-pin.json').translationsCommit")

mkdir -p "$WORK"
git init -q "$WORK/translations"
git -C "$WORK/translations" remote add origin https://github.com/mozilla/translations.git
git -C "$WORK/translations" fetch -q --depth 1 origin "$COMMIT"
git -C "$WORK/translations" checkout -q FETCH_HEAD
git -C "$WORK/translations" submodule update -q --init --recursive

(cd "$WORK/translations" && ALLOW_RUN_ON_HOST=1 python3 inference/scripts/build-wasm.py)

mkdir -p "$OUT"
cp "$WORK/translations/inference/build-wasm/bergamot-translator.js" \
  "$WORK/translations/inference/build-wasm/bergamot-translator.wasm" "$OUT"/
echo "$COMMIT" >"$OUT/TRANSLATIONS_COMMIT"
node "$SCRIPT_DIR/engine_pin.mjs" "$OUT"
