#!/usr/bin/env bash
# Assemble the Cymbra Music AppDir from a built Flutter Linux bundle
# (change: add-desktop-auto-update, task 5.2).
#
#   build_appdir.sh <bundle-dir> <appdir-out>
#
# Layout: the Flutter bundle goes in whole under usr/bin, because the embedder
# resolves `data/` and `lib/` relative to the executable. AppRun then adds
# usr/bin/lib and usr/lib to LD_LIBRARY_PATH.
#
# Library bundling uses a BLOCK-list, the way linuxdeploy does: copy every
# resolved `ldd` dependency of the binary and of each bundled plugin .so, EXCEPT
# the core platform stack. Bundling glibc, the GL/X11/Wayland stack or GTK is not
# "safer" — it is the classic way to make an AppImage crash on a host whose
# drivers or theme engine do not match the copies you shipped.
set -euo pipefail

BUNDLE="${1:?usage: build_appdir.sh <bundle-dir> <appdir-out>}"
APPDIR="${2:?usage: build_appdir.sh <bundle-dir> <appdir-out>}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

[[ -x "$BUNDLE/music" ]] || { echo "no music binary in $BUNDLE" >&2; exit 1; }

rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/lib" \
         "$APPDIR/usr/share/applications" \
         "$APPDIR/usr/share/icons/hicolor/256x256/apps"

cp -a "$BUNDLE/." "$APPDIR/usr/bin/"

# AppImage requires the .desktop and the icon at the AppDir ROOT; the copies
# under usr/share are what a host desktop picks up once the file is integrated.
install -m 0755 "$HERE/AppRun"                 "$APPDIR/AppRun"
install -m 0644 "$HERE/cymbra-music.desktop"   "$APPDIR/cymbra-music.desktop"
install -m 0644 "$HERE/cymbra-music.desktop"   "$APPDIR/usr/share/applications/cymbra-music.desktop"
install -m 0644 "$HERE/cymbra-music.png"       "$APPDIR/cymbra-music.png"
install -m 0644 "$HERE/cymbra-music.png"       "$APPDIR/usr/share/icons/hicolor/256x256/apps/cymbra-music.png"

# Never bundle these: they must come from the host or the AppImage breaks on
# machines whose kernel, GPU driver, display server or theme engine differ.
EXCLUDE_RE='^(ld-linux|libc|libm|libdl|libpthread|librt|libresolv|libnsl|libutil|libanl|libgcc_s|libstdc\+\+|libGL|libGLX|libGLdispatch|libEGL|libGLESv2|libOpenGL|libdrm|libgbm|libX|libxcb|libxkbcommon|libwayland|libgtk-3|libgdk-3|libgdk_pixbuf|libglib-2|libgobject-2|libgio-2|libgmodule-2|libpango|libcairo|libatk|libharfbuzz|libfontconfig|libfreetype|libselinux|libsystemd|libudev|libdbus-1)'

copy_deps() {
  local obj="$1"
  ldd "$obj" 2>/dev/null | awk '/=> \//{print $3}' | while read -r lib; do
    local base
    base="$(basename "$lib")"
    [[ "$base" =~ $EXCLUDE_RE ]] && continue
    # Already shipped inside the Flutter bundle (libapp.so, plugins…).
    [[ -e "$APPDIR/usr/bin/lib/$base" ]] && continue
    [[ -e "$APPDIR/usr/lib/$base" ]] && continue
    cp -L "$lib" "$APPDIR/usr/lib/$base"
    echo "  bundled $base"
  done
}

echo "[appdir] bundling runtime libraries"
copy_deps "$APPDIR/usr/bin/music"
if [[ -d "$APPDIR/usr/bin/lib" ]]; then
  for so in "$APPDIR/usr/bin/lib"/*.so*; do
    [[ -e "$so" ]] || continue
    copy_deps "$so"
  done
fi

echo "[appdir] ready: $APPDIR"
