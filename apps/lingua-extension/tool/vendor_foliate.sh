#!/usr/bin/env bash
# Refresh the vendored copy of foliate-js (add-lingua-reader D2) at a given commit:
#   tool/vendor_foliate.sh [<commit>]    (default: the commit vendor/VENDOR.md records)
#
# foliate-js has no release and declares its API unstable, so it is vendored at a pinned
# commit rather than taken as a submodule or a git dependency. Only the modules an EPUB needs
# are copied; the other formats and features (PDF, MOBI, FB2, CBZ, search, TTS) are refused at
# build time by build.mjs, and foliate's own minified zip build is replaced by the npm
# package it is built from (see vendor/foliate-js/vendor/zip.js). After a refresh: update the
# commit in vendor/VENDOR.md, rebuild, and re-run the reader's device passes (tasks 1.3, 6.2).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEST="$APP_DIR/vendor/foliate-js"
PINNED="$(sed -n 's/^- \*\*Commit\*\*: `\([0-9a-f]\{40\}\)`.*/\1/p' "$APP_DIR/vendor/VENDOR.md")"
COMMIT="${1:-$PINNED}"
[ -n "$COMMIT" ] || { echo "error: no commit given and none recorded in vendor/VENDOR.md" >&2; exit 1; }

FILES=(LICENSE view.js epub.js epubcfi.js paginator.js fixed-layout.js progress.js overlayer.js text-walker.js)

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
git -C "$WORK" init -q
git -C "$WORK" fetch -q --depth 1 https://github.com/johnfactotum/foliate-js "$COMMIT"
git -C "$WORK" checkout -q FETCH_HEAD

mkdir -p "$DEST/vendor"
for f in "${FILES[@]}"; do cp "$WORK/$f" "$DEST/$f"; done
# The zip reader: foliate's rollup input (rollup/zip.js), pointed at the npm package instead of
# the minified bundle foliate builds from it. Kept by the refresh, never overwritten.
[ -f "$DEST/vendor/zip.js" ] || {
  echo "error: $DEST/vendor/zip.js is missing — restore it from git" >&2
  exit 1
}
echo "Vendored foliate-js $COMMIT into vendor/foliate-js (${#FILES[@]} files)"
