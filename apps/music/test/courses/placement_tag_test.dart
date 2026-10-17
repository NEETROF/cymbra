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

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:music/courses/course_manifest.dart';

String _manifest(List<Map<String, Object>> blocks) => jsonEncode({
  'schemaVersion': 2,
  'id': 'placement-x',
  'track': 'placement',
  'blocks': blocks,
});

void main() {
  group('placementUnit tags (change: add-music-placement-test)', () {
    test('kept by block index on tagged blocks, absent on untagged ones', () {
      final m = parseCourseManifest(
        _manifest([
          {
            'type': 'text',
            'text': {'en': 'hook'},
          },
          {
            'type': 'question',
            'placementUnit': 'u1',
            'prompt': {'en': 'q'},
            'options': [
              {'en': 'a'},
              {'en': 'b'},
            ],
            'answerIndex': 0,
          },
          {
            'type': 'question',
            'prompt': {'en': 'untagged'},
            'options': [
              {'en': 'a'},
            ],
            'answerIndex': 0,
          },
          {
            'type': 'question',
            'placementUnit': 'u3',
            'prompt': {'en': 'q3'},
            'options': [
              {'en': 'a'},
            ],
            'answerIndex': 0,
          },
        ]),
      )!;
      expect(m.track, kPlacementTrack);
      expect(m.placementUnits, {1: 'u1', 3: 'u3'});
    });

    test('an unknown block type keeps its tag — it still probes its unit', () {
      final m = parseCourseManifest(
        _manifest([
          {'type': 'hologram', 'placementUnit': 'u5'},
        ]),
      )!;
      expect(m.blocks.single, isA<UnsupportedBlock>());
      expect(m.placementUnits, {0: 'u5'});
    });

    test('an empty or non-string tag is no tag', () {
      final m = parseCourseManifest(
        _manifest([
          {
            'type': 'text',
            'placementUnit': '',
            'text': {'en': 'x'},
          },
          {
            'type': 'text',
            'placementUnit': 3,
            'text': {'en': 'y'},
          },
        ]),
      )!;
      expect(m.placementUnits, isEmpty);
    });
  });
}
