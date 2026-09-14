#!/usr/bin/env bash
# Render the extension icons from the Cymbra brand mark (apps/site/public/favicon.svg) into
# icons/ — committed, so builds need no SVG renderer. Rerun when the mark changes; a
# dedicated Lingua icon replaces the source here without touching the manifest.
# Needs rsvg-convert (brew: librsvg).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SOURCE="$APP_DIR/../site/public/favicon.svg"
OUT="$APP_DIR/icons"

command -v rsvg-convert >/dev/null 2>&1 || {
  echo "error: rsvg-convert not found on PATH (brew install librsvg)" >&2
  exit 1
}

mkdir -p "$OUT"
# 16/32: toolbar; 48/96/128: extension lists; 256/512: Safari's extension settings;
# 1024: the host app icon (apps/lingua-apple).
for size in 16 32 48 96 128 256 512 1024; do
  rsvg-convert --width "$size" --height "$size" "$SOURCE" --output "$OUT/icon-$size.png"
done
echo "Rendered icons into icons/ from $(basename "$SOURCE")"
