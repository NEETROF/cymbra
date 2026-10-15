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

/// Driver for the showcase video (change: add-site-music-showcase).
///
/// The scenario reports where it wrote the frames and the sound log it kept.
/// This moves the frames into `build/showcase/frames/` — now, while the app is
/// still installed: `flutter drive` uninstalls it on exit, and its temporary
/// directory with it — and writes the report to `build/showcase/report.json`
/// for `tool/record_showcase_video.sh`. A failed scenario reports nothing, so
/// the script stops before touching the committed video.
library;

import 'dart:convert';
import 'dart:io';

import 'package:integration_test/integration_test_driver.dart';

Future<void> main() => integrationDriver(
  responseDataCallback: (data) async {
    final showcase = data?['showcase'];
    if (showcase is! Map<String, dynamic>) return;
    final frames = Directory('build/showcase/frames');
    await frames.create(recursive: true);
    final source = Directory(showcase['framesDir'] as String);
    await for (final entry in source.list()) {
      if (entry is File) {
        await _copy(entry, '${frames.path}/${entry.uri.pathSegments.last}');
      }
    }
    showcase['framesDir'] = frames.absolute.path;
    await File(
      'build/showcase/report.json',
    ).writeAsString(jsonEncode(showcase));
  },
);

/// Copies [file] to [to], retrying when a signal interrupts the copy
/// (`EINTR`, errno 4 — seen on macOS while `flutter drive` winds the app down).
Future<void> _copy(File file, String to) async {
  for (var attempt = 1; ; attempt++) {
    try {
      await file.copy(to);
      return;
    } on FileSystemException catch (e) {
      if (e.osError?.errorCode != 4 || attempt == 5) rethrow;
    }
  }
}
