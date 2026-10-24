#!/usr/bin/env bash
# Build the Cymbra Music Debian package (change: add-desktop-auto-update,
# tasks 5.2–5.3, design D5).
#
#   build_deb.sh <bundle-dir> <update-helper-binary> <version> <out.deb>
#
# Layout:
#   /opt/cymbra-music/               the Flutter bundle, whole — the embedder
#                                    resolves data/ and lib/ relative to the
#                                    executable
#   /opt/cymbra-music/install_method.txt   marker: a package-managed install
#   /usr/bin/cymbra-music            launcher symlink
#   /usr/lib/cymbra-music/cymbra-update-helper   root-owned privileged helper
#   /usr/share/polkit-1/actions/app.cymbra.music.update.policy
#   /usr/share/applications/cymbra-music.desktop + hicolor icon
#
# Everything is owned by root (dpkg-deb --root-owner-group): the helper and its
# policy must not be writable by the user the app runs as.
set -euo pipefail

BUNDLE="${1:?usage: build_deb.sh <bundle-dir> <helper> <version> <out.deb>}"
HELPER="${2:?usage: build_deb.sh <bundle-dir> <helper> <version> <out.deb>}"
VERSION="${3:?usage: build_deb.sh <bundle-dir> <helper> <version> <out.deb>}"
OUT="${4:?usage: build_deb.sh <bundle-dir> <helper> <version> <out.deb>}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

[[ -x "$BUNDLE/music" ]] || { echo "no music binary in $BUNDLE" >&2; exit 1; }
[[ -x "$HELPER" ]] || { echo "no update helper at $HELPER" >&2; exit 1; }
# Debian versions allow `+`; the app's `1.40.0+50` is used as is so the helper's
# installed-version check reads the same string the app was built with.
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(\+[0-9]+)?$ ]] \
  || { echo "unexpected version '$VERSION'" >&2; exit 1; }

ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT

install -d "$ROOT/opt/cymbra-music" "$ROOT/usr/bin" "$ROOT/usr/lib/cymbra-music" \
  "$ROOT/usr/share/applications" "$ROOT/usr/share/icons/hicolor/256x256/apps" \
  "$ROOT/usr/share/polkit-1/actions" "$ROOT/DEBIAN"

cp -a "$BUNDLE/." "$ROOT/opt/cymbra-music/"
printf 'deb\n' > "$ROOT/opt/cymbra-music/install_method.txt"
ln -s /opt/cymbra-music/music "$ROOT/usr/bin/cymbra-music"
install -m 0755 "$HELPER" "$ROOT/usr/lib/cymbra-music/cymbra-update-helper"
install -m 0644 "$HERE/app.cymbra.music.update.policy" \
  "$ROOT/usr/share/polkit-1/actions/app.cymbra.music.update.policy"
install -m 0644 "$HERE/cymbra-music.desktop" "$ROOT/usr/share/applications/cymbra-music.desktop"
install -m 0644 "$HERE/cymbra-music.png" \
  "$ROOT/usr/share/icons/hicolor/256x256/apps/cymbra-music.png"

# Directories 0755, files keep their bundle modes; nothing group/world-writable.
find "$ROOT" -type d -exec chmod 0755 {} +
find "$ROOT" -perm /022 -type f -exec chmod go-w {} +

SIZE_KB="$(du -sk --exclude=DEBIAN "$ROOT" | cut -f1)"
sed -e "s/@VERSION@/$VERSION/" -e "s/@INSTALLED_SIZE@/$SIZE_KB/" \
  "$HERE/control.in" > "$ROOT/DEBIAN/control"

dpkg-deb --root-owner-group --build "$ROOT" "$OUT"
echo "[deb] built $OUT ($VERSION, ${SIZE_KB} KiB installed)"
