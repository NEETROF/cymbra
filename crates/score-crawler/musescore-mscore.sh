#!/bin/sh
# `mscore` shim for the score-crawler's docker converter backend.
#
# The crawler always calls:  mscore -o <OUT>.mxl <IN>.mscx
# but MuseScore 4.7 headless is fussy:
#   - it ignores QT_QPA_PLATFORM=offscreen (still tries xcb → aborts, no display);
#   - under a virtual display (xvfb) the CLI conversion hangs (first-run GUI);
#   - `-platform offscreen` on argv wins, but MuseScore's own file parser then
#     mis-reads the leftover `offscreen` token as an input file.
# The one combination that converts headlessly AND exits cleanly is the offscreen
# platform + a batch *job file* (`-j`), which takes no positional inputs. So this
# shim rewrites the crawler's `-o OUT IN` into a one-entry job file.
#
# Any other invocation (e.g. `mscore --version`) is passed through under
# offscreen unchanged.
set -eu

APPRUN=/opt/musescore/squashfs-root/AppRun

# Is this the crawler's convert call (contains a `-o`)?
has_o=0
for a in "$@"; do
    [ "$a" = "-o" ] && has_o=1
done

if [ "$has_o" -eq 0 ]; then
    exec "$APPRUN" -platform offscreen "$@"
fi

# Extract OUT (the arg right after -o) and IN (a non-flag arg elsewhere).
out=""
in=""
prev=""
for a in "$@"; do
    if [ "$prev" = "-o" ]; then
        out="$a"
    elif [ "${a#-}" = "$a" ]; then
        in="$a"
    fi
    prev="$a"
done

if [ -z "$in" ] || [ -z "$out" ]; then
    echo "mscore shim: expected '-o <out> <in>', got: $*" >&2
    exit 2
fi

job="$(mktemp)"
# MuseScore infers the export format from the OUT extension (.mxl → compressed
# MusicXML), exactly as `-o OUT` would.
printf '[{"in": "%s", "out": "%s"}]\n' "$in" "$out" >"$job"
exec "$APPRUN" -platform offscreen -j "$job"
