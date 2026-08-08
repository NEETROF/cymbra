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

import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

/// Real fonts for the test environment, which otherwise draws every glyph as a
/// filled box — turning a golden into an unreadable stack of rectangles.
///
/// Loaded once for the whole suite by `test/flutter_test_config.dart`, so no
/// test has to remember to ask for them.

bool _loaded = false;

/// Loads the bundled Bravura SMuFL font so golden tests render real music
/// glyphs instead of the test framework's fallback boxes.
Future<void> loadBravura() => loadTestFonts();

/// Loads the app's music font plus a real text face and the Material icon face.
///
/// Idempotent — safe to call from a `setUpAll` that predates the global hook.
Future<void> loadTestFonts() async {
  if (_loaded) return;
  _loaded = true;
  TestWidgetsFlutterBinding.ensureInitialized();

  // The app's own music font, from its bundled asset.
  await (FontLoader(
    'Bravura',
  )..addFont(rootBundle.load('assets/fonts/Bravura.otf'))).load();

  // Text and icons come from the Flutter SDK's own copies, so this needs no new
  // asset and no network. Found by walking up from the running executable —
  // which is `dart` under some runners and `flutter_tester` under others, at
  // different depths — so it follows whichever SDK the suite runs under.
  final fonts = _findMaterialFonts();
  if (fonts == null) return;

  await _loadFile('Roboto', '${fonts.path}/Roboto-Regular.ttf');
  await _loadFile('MaterialIcons', '${fonts.path}/MaterialIcons-Regular.otf');
}

/// The SDK's `material_fonts` directory, or null when it cannot be located (the
/// suite then keeps the framework's box glyphs rather than failing).
Directory? _findMaterialFonts() {
  var dir = File(Platform.resolvedExecutable).parent;
  for (var up = 0; up < 8; up++) {
    final candidate = Directory('${dir.path}/artifacts/material_fonts');
    if (candidate.existsSync()) return candidate;
    final parent = dir.parent;
    if (parent.path == dir.path) break; // reached the filesystem root
    dir = parent;
  }
  return null;
}

Future<void> _loadFile(String family, String path) async {
  final file = File(path);
  if (!file.existsSync()) return;
  final bytes = await file.readAsBytes();
  await (FontLoader(
    family,
  )..addFont(Future.value(ByteData.view(bytes.buffer)))).load();
}
