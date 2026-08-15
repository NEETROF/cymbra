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

/// Host side of `flutter drive`. Two scenarios share it:
///
/// - `integration_test/app_test.dart` (`melos run integration-watch`) takes no
///   screenshot, so [onScreenshot] is simply never called;
/// - `integration_test/capture_test.dart` (`melos run screenshots`) reports one
///   capture per listing surface, which this driver validates against the
///   declared manifest and writes into the versioned store assets tree.
///
/// The capture's *name* carries its whole destination
/// (`<platform>/<class>/<locale>/NN_<surface>`), so this process needs no
/// arguments of its own — see `tool/store_manifest.dart`.
// This is a CLI-style driver; printing is how it reports progress.
// ignore_for_file: avoid_print
library;

import 'dart:io';
import 'dart:typed_data';

import 'package:image/image.dart' as img;
import 'package:integration_test/integration_test_driver_extended.dart';

import '../tool/store_manifest.dart';

Future<void> main() => integrationDriver(onScreenshot: _writeStoreAsset);

/// Writes one reported capture to `store/<name>.png`, failing the run rather
/// than emitting an asset the stores would reject days later:
/// the name must describe a declared asset, its dimensions must match the
/// manifest exactly, and the file is re-encoded without an alpha channel.
Future<bool> _writeStoreAsset(
  String name,
  List<int> bytes, [
  Map<String, Object?>? args,
]) async {
  final declared = parseCaptureName(name);
  if (declared == null) {
    print(
      'Capture "$name" is not a declared store asset (see store/manifest.json).',
    );
    return false;
  }

  final decoded = img.decodePng(Uint8List.fromList(bytes));
  if (decoded == null) {
    print('Capture "$name" is not decodable as PNG.');
    return false;
  }

  final target = declared.target;
  if (decoded.width != target.widthPx || decoded.height != target.heightPx) {
    print(
      'Capture "$name" is ${decoded.width}x${decoded.height}, but '
      '${target.id} requires ${target.widthPx}x${target.heightPx}.',
    );
    return false;
  }

  // Flatten to opaque RGB: the engine always encodes RGBA, and App Store
  // Connect rejects a screenshot that carries transparency even when every
  // pixel is opaque.
  final opaque = decoded.convert(numChannels: 3);

  final file = File(captureAssetPath(target, declared.locale, declared.index));
  await file.parent.create(recursive: true);
  await file.writeAsBytes(img.encodePng(opaque));
  print('wrote ${file.path} (${target.widthPx}x${target.heightPx})');
  return true;
}
