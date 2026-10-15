#!/usr/bin/env bash
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# Records the Music showcase video for one locale (change: add-site-music-showcase).
#
#   tool/record_showcase_video.sh fr
#
# 1. `flutter drive` runs integration_test/showcase_test.dart — on macOS by
#    default, laid out as the iPad 13" (the frames come from the app's own layer
#    tree, so no simulator is needed; `CAPTURE_DEVICE="iPad Pro 13-inch (M4)"`
#    runs it there instead): the real app plays a bundled piece in Wait Mode,
#    drawn frame by frame on a virtual clock, every sound command logged on that
#    same clock.
# 2. `showcase-render` (apps/music/rust) replays the log through the app's own
#    mixer and SoundFont into a WAV — the app's sound, to the sample.
# 3. ffmpeg assembles H.264 + AAC (`+faststart`) under the site's 4 MB budget,
#    plus a poster frame.
#
# Writes store/video/<locale>/showcase.mp4 and poster.jpg only when every step
# succeeded: a failed run leaves the previous video in place.
set -euo pipefail

cd "$(dirname "$0")/.."

locale="${1:-}"
if [[ -z "$locale" ]]; then
  echo "usage: $(basename "$0") <locale>   (en, fr, it, es)" >&2
  exit 2
fi

for tool in ffmpeg ffprobe cargo python3; do
  command -v "$tool" >/dev/null || { echo "missing: $tool" >&2; exit 1; }
done

device="${CAPTURE_DEVICE:-macos}"
budget_bytes=$((4 * 1024 * 1024))
max_seconds=30
poster_second=6

work="build/showcase"
rm -rf "$work"
mkdir -p "$work"

if [[ "$device" != "macos" ]]; then
  xcrun simctl boot "$device" 2>/dev/null || true
fi

echo "==> drawing the $locale video on '$device'"
flutter drive \
  --driver=test_driver/showcase_driver.dart \
  --target=integration_test/showcase_test.dart \
  --dart-define="CAPTURE_LOCALE=$locale" \
  -d "$device"

report="$work/report.json"
[[ -f "$report" ]] || { echo "the scenario reported nothing: video left as it was" >&2; exit 1; }

read -r frames_dir fps frames < <(python3 - "$report" "$work/sound.log" <<'PY'
import json, sys
r = json.load(open(sys.argv[1]))
open(sys.argv[2], "w").write("\n".join(r["soundLog"]) + "\n")
print(r["framesDir"], r["fps"], r["frames"])
PY
)
count=$(find "$frames_dir" -name 'frame_*.png' | wc -l | tr -d ' ')
[[ "$count" == "$frames" ]] || { echo "expected $frames frames, found $count in $frames_dir" >&2; exit 1; }

echo "==> rendering the sound with the app's mixer"
cargo run --quiet --release --manifest-path rust/Cargo.toml --bin showcase-render -- \
  "$work/sound.log" assets/soundfonts "$work/sound.wav"

echo "==> encoding"
out="$work/showcase.mp4"
for crf in 24 26 28 30 32 34; do
  ffmpeg -loglevel error -y \
    -framerate "$fps" -i "$frames_dir/frame_%05d.png" -i "$work/sound.wav" \
    -c:v libx264 -preset slow -crf "$crf" -pix_fmt yuv420p \
    -c:a aac -b:a 128k -movflags +faststart -shortest "$out"
  size=$(stat -f%z "$out")
  (( size <= budget_bytes )) && break
done
(( size <= budget_bytes )) || { echo "over the 4 MB budget even at crf $crf ($size bytes)" >&2; exit 1; }

seconds=$(ffprobe -v error -show_entries format=duration -of default=nw=1:nk=1 "$out")
python3 -c "import sys; sys.exit(0 if float('$seconds') <= $max_seconds else 1)" \
  || { echo "video is $seconds s, over $max_seconds s" >&2; exit 1; }
ffprobe -v error -select_streams a -show_entries stream=codec_type -of csv=p=0 "$out" | grep -q audio \
  || { echo "video has no audio track" >&2; exit 1; }

poster_frame=$(printf '%s/frame_%05d.png' "$frames_dir" $((poster_second * fps)))
ffmpeg -loglevel error -y -i "$poster_frame" -q:v 3 "$work/poster.jpg"

dest="store/video/$locale"
mkdir -p "$dest"
mv "$out" "$dest/showcase.mp4"
mv "$work/poster.jpg" "$dest/poster.jpg"
rm -rf "$frames_dir"
echo "wrote $dest/showcase.mp4 (${size} bytes, ${seconds}s, crf $crf) and $dest/poster.jpg"
