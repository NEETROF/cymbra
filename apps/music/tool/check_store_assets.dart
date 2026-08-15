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

/// Checks the committed store-listing assets against the declared manifest
/// (change: add-store-screenshot-harness, D7).
///
/// Reads PNG headers only — no simulator, no image decoding, no dependency — so
/// it is cheap enough to gate every CI run: it catches an off-size, missing or
/// hand-edited asset, and a screenshot carrying the alpha channel Apple
/// rejects, without booting anything.
///
/// ```bash
/// cd apps/music && dart run tool/check_store_assets.dart              # everything
/// cd apps/music && dart run tool/check_store_assets.dart ios/ipad_13
/// cd apps/music && dart run tool/check_store_assets.dart ios/ipad_13 --locale=fr
/// ```
///
/// A target that has not been captured yet is reported as missing, so the gate
/// also answers "which sets are stale?".
library;

import 'dart:io';
import 'dart:typed_data';

import 'store_manifest.dart';

Future<void> main(List<String> args) async {
  const localeFlag = '--locale=';
  final localeArgs = <String>{
    for (final arg in args)
      if (arg.startsWith(localeFlag)) arg.substring(localeFlag.length),
  };
  final targetArgs = <String>{
    for (final arg in args)
      if (!arg.startsWith('--')) arg,
  };

  for (final id in targetArgs) {
    if (captureTargetById(id) == null) {
      stderr.writeln('unknown target: $id');
      exit(2);
    }
  }
  for (final locale in localeArgs) {
    if (!kCaptureLocales.contains(locale)) {
      stderr.writeln('unknown locale: $locale');
      exit(2);
    }
  }

  final targets = targetArgs.isEmpty ? null : targetArgs;
  final locales = localeArgs.isEmpty ? kCaptureLocales : localeArgs.toList();

  final problems = <String>[];
  var checked = 0;

  for (final target in kCaptureTargets) {
    if (targets != null && !targets.contains(target.id)) continue;
    for (final locale in locales) {
      for (var i = 0; i < kCaptureSurfaces.length; i++) {
        final path = captureAssetPath(target, locale, i);
        final file = File(path);
        if (!file.existsSync()) {
          problems.add('$path: missing');
          continue;
        }
        checked++;
        final problem = _inspect(file.readAsBytesSync(), target);
        if (problem != null) problems.add('$path: $problem');
      }
    }
  }

  if (problems.isNotEmpty) {
    stderr.writeln('Store assets do not match store/manifest.json:');
    for (final problem in problems) {
      stderr.writeln('  $problem');
    }
    stderr.writeln(
      '\nRegenerate with `tool/capture_store_screenshots.sh <target>`.',
    );
    exit(1);
  }

  stdout.writeln('$checked store assets match the manifest.');
}

/// What is wrong with these PNG bytes for [target], or null when they are fine.
String? _inspect(Uint8List bytes, CaptureTarget target) {
  // 8-byte signature, then the IHDR chunk: length(4) type(4) width(4) height(4)
  // bitDepth(1) colourType(1).
  const signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
  if (bytes.length < 26) return 'truncated';
  for (var i = 0; i < signature.length; i++) {
    if (bytes[i] != signature[i]) return 'not a PNG';
  }
  final header = ByteData.sublistView(bytes);
  if (String.fromCharCodes(bytes.sublist(12, 16)) != 'IHDR') {
    return 'malformed PNG (no IHDR)';
  }

  final width = header.getUint32(16);
  final height = header.getUint32(20);
  if (width != target.widthPx || height != target.heightPx) {
    return '${width}x$height, expected ${target.widthPx}x${target.heightPx}';
  }

  // Colour types 4 (grey+alpha) and 6 (RGBA) carry an alpha channel; App Store
  // Connect rejects those even when every pixel is opaque.
  final colourType = header.getUint8(25);
  if (colourType == 4 || colourType == 6) {
    return 'carries an alpha channel (PNG colour type $colourType)';
  }
  return null;
}
