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

import 'dart:async';

import 'support/test_fonts.dart';

/// Suite-wide setup, picked up automatically by `flutter test` for every test
/// under `test/`.
///
/// Loads real fonts before anything runs. Without this the framework draws each
/// glyph as a filled box, which makes every golden an unreadable stack of
/// rectangles — you can see the layout but never what it says. Text metrics also
/// get closer to a real device, so a layout that overflows here is more likely
/// to be one that overflows in the app.
Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  await loadTestFonts();
  await testMain();
}
