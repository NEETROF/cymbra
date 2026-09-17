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

# The host app's iOS icon: iOS draws its own rounded mask, and App Store Connect refuses an
# icon with transparency or an alpha channel — so the mark goes full-bleed on an opaque square.
# The macOS sizes keep the rounded mark (macOS draws icons as they are).
command -v magick >/dev/null 2>&1 || {
  echo "error: magick not found on PATH (brew install imagemagick)" >&2
  exit 1
}
IOS_ICON="$APP_DIR/../lingua-apple/Shared (App)/Assets.xcassets/AppIcon.appiconset/universal-icon-1024@1x.png"
sed -E 's/rx="[0-9.]+" ry="[0-9.]+"/rx="0" ry="0"/' "$SOURCE" | rsvg-convert --width 1024 --height 1024 | magick - -alpha off "PNG24:$IOS_ICON"
echo "Rendered the opaque iOS app icon into apps/lingua-apple"
