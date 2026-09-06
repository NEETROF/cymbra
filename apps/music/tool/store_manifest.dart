// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// The **declared** store-listing capture manifest (change:
/// add-store-screenshot-harness, D5): per target, the pixel dimensions the store
/// currently requires and the ordered surfaces a set must show.
///
/// This is the single place a future store class change is edited — the capture
/// scenario sizes its viewport from it, the driver validates every produced
/// image against it, and the CI gate re-checks the committed assets against it.
///
/// Plain Dart on purpose (no Flutter imports): the host-side driver, the CI
/// checker and the on-device capture scenario all import this file.
///
/// `dart run tool/store_manifest.dart` rewrites the machine-readable mirror at
/// `store/manifest.json`; `test/store_manifest_test.dart` fails when the two
/// drift apart.
library;

import 'dart:convert';
import 'dart:io';

/// One capture target: a store size class, and the device that stands in for it.
class CaptureTarget {
  const CaptureTarget({
    required this.platform,
    required this.sizeClass,
    required this.widthPx,
    required this.heightPx,
    required this.devicePixelRatio,
    required this.device,
    required this.why,
  });

  /// Store-facing platform directory: `ios`, `macos` or `android`.
  final String platform;

  /// Size-class directory, e.g. `iphone_6.9`.
  final String sizeClass;

  /// Required landscape output dimensions, in pixels.
  final int widthPx;
  final int heightPx;

  /// Pixel density the surface is rendered at. Together with [widthPx] /
  /// [heightPx] it fixes the *logical* viewport, which is what decides the
  /// app's layout class — so a target reproduces its device's layout, not just
  /// its resolution.
  final double devicePixelRatio;

  /// The simulator/emulator this target is captured on.
  final String device;

  /// Why the store requires this class — the note a future editor needs.
  final String why;

  /// `<platform>/<sizeClass>` — the target's id on the command line and the
  /// first two path segments of every image it produces.
  String get id => '$platform/$sizeClass';

  /// Logical (device-independent) viewport the app lays out in.
  double get logicalWidth => widthPx / devicePixelRatio;
  double get logicalHeight => heightPx / devicePixelRatio;

  Map<String, Object?> toJson() => {
    'id': id,
    'platform': platform,
    'sizeClass': sizeClass,
    'widthPx': widthPx,
    'heightPx': heightPx,
    'devicePixelRatio': devicePixelRatio,
    'device': device,
    'why': why,
  };
}

/// The locales the listing ships in (`store/copy/`), and therefore the locales a
/// complete set exists for.
const List<String> kCaptureLocales = ['en', 'fr', 'it', 'es'];

/// The surfaces every set shows, in order. The file name is `NN_<surface>.png`
/// with `NN` the 1-based position — so the set's story is readable from the
/// directory listing, and a shipped-but-uncaptured feature is visible here
/// rather than only inside the images.
const List<String> kCaptureSurfaces = [
  'library',
  'synthesia',
  'staff',
  'courses',
  'measures',
  // The drum surfaces (changes: add-drum-kit-view … add-drum-scoring), appended
  // rather than interleaved so the existing images keep their numbers: the kit
  // cascade mid-run, then the same groove read as engraved percussion.
  'drums',
  'drums_staff',
  // The perspective stage (RenderMode.stage): percussion-only, and the FIRST
  // segment of the mode row because it is the reading a beginner reaches for
  // (lib/screens/player_screen.dart). It is never the default — a fresh run
  // resolves percussion to the cascade — so it went uncaptured until now.
  'drums_stage',
];

/// The targets a full pass regenerates.
const List<CaptureTarget> kCaptureTargets = [
  CaptureTarget(
    platform: 'ios',
    sizeClass: 'iphone_6.9',
    widthPx: 2868,
    heightPx: 1320,
    devicePixelRatio: 3,
    device: 'iPhone 16 Pro Max',
    why:
        'App Store Connect requires the 6.9" iPhone display class; the 6.7" '
        'class it used to list (2796x1290) is retired.',
  ),
  CaptureTarget(
    platform: 'ios',
    sizeClass: 'ipad_13',
    widthPx: 2752,
    heightPx: 2064,
    devicePixelRatio: 2,
    device: 'iPad Pro 13-inch (M4)',
    why:
        'App Store Connect requires the 13" iPad class; 12.9" (2732x2048) is '
        'now merely scaled from it.',
  ),
  CaptureTarget(
    platform: 'macos',
    sizeClass: 'desktop_1440x900',
    widthPx: 1440,
    heightPx: 900,
    devicePixelRatio: 1,
    device: 'macOS desktop',
    why: 'One of the four sizes the Mac App Store accepts.',
  ),
  CaptureTarget(
    platform: 'android',
    sizeClass: 'phone_16x9',
    widthPx: 1920,
    heightPx: 1080,
    devicePixelRatio: 2,
    device: 'Android emulator',
    why:
        'Play asks for 16:9 at >=1080p to be eligible for its recommendation '
        'surfaces; the long side stays within its 2x short-side cap.',
  ),
  // The two tablet slots are **required** on the Play listing (both carry the
  // asterisk) as soon as the bundle does not restrict screen sizes — which ours
  // does not. The pixel sizes are 16:9 because Play demands exactly that on
  // these slots; a real 7" tablet is 16:10, and the store constraint wins over
  // hardware fidelity, exactly as it already does for the phone target.
  //
  // What matters more than the pixel count is the LOGICAL size: the app
  // classifies a viewport by its shortest side (landscape-locked, so that is
  // the height) at 600/900 dp — see lib/layout/device_class.dart. Both targets
  // land in [600, 900), so they render the tablet layout. Get this wrong and
  // the "tablet" screenshots would show the phone layout at a bigger size.
  CaptureTarget(
    platform: 'android',
    sizeClass: 'tablet_7',
    widthPx: 1920,
    heightPx: 1080,
    devicePixelRatio: 1.6,
    device: 'Android emulator',
    why:
        'Play requires 7-inch tablet screenshots in 16:9; dpr 1.6 puts the '
        'viewport at 1200x675 dp — a tablet by the app\'s own breakpoint.',
  ),
  CaptureTarget(
    platform: 'android',
    sizeClass: 'tablet_10',
    widthPx: 2560,
    heightPx: 1440,
    devicePixelRatio: 2,
    device: 'Android emulator',
    why:
        'Play requires 10-inch tablet screenshots in 16:9; dpr 2 puts the '
        'viewport at 1280x720 dp — the same tablet class, more pixels, which '
        'is what a larger tablet actually is.',
  ),
];

/// The target with this [id] (`<platform>/<sizeClass>`), or null.
CaptureTarget? captureTargetById(String id) {
  for (final t in kCaptureTargets) {
    if (t.id == id) return t;
  }
  return null;
}

/// Repository path of one asset, relative to `apps/music/`:
/// `store/<platform>/<class>/<locale>/NN_<surface>.png` (D6).
String captureAssetPath(CaptureTarget target, String locale, int index) =>
    'store/${captureRelativeName(target, locale, index)}.png';

/// The same location without the `store/` prefix or the extension — the name a
/// capture is reported under, so the driver derives the file path from it alone
/// and needs no environment of its own.
String captureRelativeName(CaptureTarget target, String locale, int index) {
  final surface = kCaptureSurfaces[index];
  final nn = (index + 1).toString().padLeft(2, '0');
  return '${target.id}/$locale/${nn}_$surface';
}

/// Resolves a reported capture name back to its target and locale, so the
/// driver can validate the bytes against the declared dimensions. Null when the
/// name does not describe a declared asset.
({CaptureTarget target, String locale, int index})? parseCaptureName(
  String name,
) {
  final parts = name.split('/');
  if (parts.length != 4) return null;
  final target = captureTargetById('${parts[0]}/${parts[1]}');
  if (target == null) return null;
  if (!kCaptureLocales.contains(parts[2])) return null;
  for (var i = 0; i < kCaptureSurfaces.length; i++) {
    if (parts[3] == captureRelativeName(target, parts[2], i).split('/').last) {
      return (target: target, locale: parts[2], index: i);
    }
  }
  return null;
}

/// The manifest as JSON — the machine-readable mirror committed at
/// `store/manifest.json`.
Map<String, Object?> captureManifestJson() => {
  'comment':
      'Generated by `dart run tool/store_manifest.dart` from '
      'tool/store_manifest.dart. Do not edit by hand.',
  'locales': kCaptureLocales,
  'surfaces': kCaptureSurfaces,
  'targets': [for (final t in kCaptureTargets) t.toJson()],
  'assets': [
    for (final t in kCaptureTargets)
      for (final locale in kCaptureLocales)
        for (var i = 0; i < kCaptureSurfaces.length; i++)
          {
            'path': captureAssetPath(t, locale, i),
            'target': t.id,
            'locale': locale,
            'surface': kCaptureSurfaces[i],
            'widthPx': t.widthPx,
            'heightPx': t.heightPx,
          },
  ],
};

/// Rewrites `store/manifest.json` from the constants above.
void main() {
  const path = 'store/manifest.json';
  final json =
      '${const JsonEncoder.withIndent('  ').convert(captureManifestJson())}\n';
  File(path).writeAsStringSync(json);
  stdout.writeln('wrote $path');
}
