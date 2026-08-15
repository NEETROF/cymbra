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
# Regenerates the store listing screenshots (change: add-store-screenshot-harness).
#
#   tool/capture_store_screenshots.sh <target> [locale ...]
#   tool/capture_store_screenshots.sh ios/iphone_6.9          # every locale
#   tool/capture_store_screenshots.sh macos/desktop_1440x900 fr
#
# Targets and their required dimensions are declared in tool/store_manifest.dart
# (mirrored to store/manifest.json). One `flutter drive` run per locale writes
# the full set for that locale; the driver refuses any image that does not match
# the manifest.
#
# The device is derived from the target; override it with CAPTURE_DEVICE when
# your simulator/emulator is named differently:
#
#   CAPTURE_DEVICE="iPhone 17 Pro Max" tool/capture_store_screenshots.sh ios/iphone_6.9
set -euo pipefail

cd "$(dirname "$0")/.."

target="${1:-}"
if [[ -z "$target" ]]; then
  echo "usage: $(basename "$0") <target> [locale ...]" >&2
  echo "targets:" >&2
  sed -n 's/.*"id": "\(.*\)",/  \1/p' store/manifest.json | sort -u >&2
  exit 2
fi
shift

locales=("$@")
if [[ ${#locales[@]} -eq 0 ]]; then
  locales=(en fr it es)
fi

# Simulators/emulators the declared targets are captured on. Apple retires and
# renames device classes regularly — when that happens, edit the dimensions in
# tool/store_manifest.dart and the stand-in device here.
device="${CAPTURE_DEVICE:-}"
if [[ -z "$device" ]]; then
  case "$target" in
    ios/iphone_6.9) device="iPhone 16 Pro Max" ;;
    ios/ipad_13) device="iPad Pro 13-inch (M4)" ;;
    macos/*) device="macos" ;;
    android/*) device="emulator-5554" ;;
    *)
      echo "unknown target: $target" >&2
      exit 2
      ;;
  esac
fi

echo "==> capturing $target on '$device' for: ${locales[*]}"
for locale in "${locales[@]}"; do
  echo "--> $target / $locale"
  flutter drive \
    --driver=test_driver/integration_test.dart \
    --target=integration_test/capture_test.dart \
    --dart-define="CAPTURE_TARGET=$target" \
    --dart-define="CAPTURE_LOCALE=$locale" \
    -d "$device"
done

echo "==> verifying the captured assets against the manifest"
dart run tool/check_store_assets.dart "$target" \
  "${locales[@]/#/--locale=}"
