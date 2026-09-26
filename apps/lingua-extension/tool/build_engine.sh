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
# WHERE it builds matters. The .wasm embeds the absolute path of its sources — 145 times, in
# assertion messages — and the glue shifts with the data it points into, so the pinned bytes come
# out only when the sources sit at the path CI built them at: $CANONICAL/translations below, a
# GitHub runner's workspace for this repository. Anywhere else (ENGINE_WORK_DIR) the engine
# differs by those path strings alone, and the final check says so. On another machine:
#   sudo mkdir -p /home/runner/work/cymbra/cymbra && sudo chown "$USER" /home/runner/work/cymbra/cymbra
#
# Usage: tool/build_engine.sh [<dir>]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="${1:-$APP/engine}"
CANONICAL=/home/runner/work/cymbra/cymbra
WORK="${ENGINE_WORK_DIR:-$CANONICAL}"
COMMIT=$(node -p "require('$APP/engine-pin.json').translationsCommit")

if [ "$WORK" != "$CANONICAL" ]; then
  echo "warning: building under $WORK, not $CANONICAL — the engine will differ from the pinned bytes by its embedded source paths." >&2
fi
mkdir -p "$WORK" || {
  echo "error: cannot create $WORK. The pinned bytes are reproduced only there — see the header of this script." >&2
  exit 1
}
if [ -e "$WORK/translations" ]; then
  echo "error: $WORK/translations already exists; the engine is built from a fresh checkout." >&2
  exit 1
fi

# The same checkout actions/checkout makes: the pinned commit alone, submodules at depth 1.
git init -q "$WORK/translations"
git -C "$WORK/translations" remote add origin https://github.com/mozilla/translations.git
git -C "$WORK/translations" fetch -q --no-tags --depth 1 origin "$COMMIT"
git -C "$WORK/translations" checkout -q FETCH_HEAD
git -C "$WORK/translations" submodule update -q --init --recursive --depth 1

(cd "$WORK/translations" && ALLOW_RUN_ON_HOST=1 python3 inference/scripts/build-wasm.py)

mkdir -p "$OUT"
cp "$WORK/translations/inference/build-wasm/bergamot-translator.js" \
  "$WORK/translations/inference/build-wasm/bergamot-translator.wasm" "$OUT"/
echo "$COMMIT" >"$OUT/TRANSLATIONS_COMMIT"
node "$SCRIPT_DIR/engine_pin.mjs" "$OUT"
