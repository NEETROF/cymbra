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

/// The store-capture manifest is declared twice — as the Dart constants the
/// harness runs on, and as `store/manifest.json`, the machine-readable mirror a
/// reviewer (or a non-Dart tool) can read. This keeps the two from drifting.
library;

import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../tool/store_manifest.dart';

void main() {
  test('store/manifest.json mirrors the declared manifest', () {
    final committed =
        jsonDecode(File('store/manifest.json').readAsStringSync())
            as Map<String, dynamic>;
    expect(
      committed,
      captureManifestJson(),
      reason: 'run `dart run tool/store_manifest.dart`',
    );
  });

  test('every declared asset has a unique, well-formed path', () {
    final paths = <String>{};
    for (final target in kCaptureTargets) {
      for (final locale in kCaptureLocales) {
        for (var i = 0; i < kCaptureSurfaces.length; i++) {
          final path = captureAssetPath(target, locale, i);
          expect(paths.add(path), isTrue, reason: 'duplicate $path');
          expect(path, startsWith('store/${target.id}/$locale/'));
          expect(path, endsWith('_${kCaptureSurfaces[i]}.png'));
        }
      }
    }
    expect(
      paths,
      hasLength(
        kCaptureTargets.length *
            kCaptureLocales.length *
            kCaptureSurfaces.length,
      ),
    );
  });

  test('a reported capture name resolves back to its declared asset', () {
    for (final target in kCaptureTargets) {
      for (var i = 0; i < kCaptureSurfaces.length; i++) {
        final name = captureRelativeName(target, 'fr', i);
        final parsed = parseCaptureName(name);
        expect(parsed, isNotNull, reason: name);
        expect(parsed!.target.id, target.id);
        expect(parsed.locale, 'fr');
        expect(parsed.index, i);
        expect(
          captureAssetPath(parsed.target, parsed.locale, parsed.index),
          'store/$name.png',
        );
      }
    }
  });

  test('a name outside the manifest is rejected', () {
    expect(parseCaptureName('ios/iphone_6.7/en/01_library'), isNull);
    expect(parseCaptureName('ios/iphone_6.9/de/01_library'), isNull);
    expect(parseCaptureName('ios/iphone_6.9/en/09_soundfonts'), isNull);
    expect(parseCaptureName('01_library'), isNull);
  });

  test('targets are landscape and cover every shipping locale', () {
    expect(kCaptureLocales, containsAll(['en', 'fr', 'it', 'es']));
    for (final target in kCaptureTargets) {
      expect(
        target.widthPx,
        greaterThan(target.heightPx),
        reason: '${target.id} must be landscape (the app is landscape-locked)',
      );
      // Google Play caps the long side at twice the short side.
      if (target.platform == 'android') {
        expect(target.widthPx, lessThanOrEqualTo(2 * target.heightPx));
      }
    }
  });
}
