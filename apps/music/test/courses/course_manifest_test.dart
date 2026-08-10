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

String _json(Object o) => jsonEncode(o);

void main() {
  group('resolveInline', () {
    test('prefers the requested language, falls back to en then anything', () {
      const t = {'en': 'Rest', 'fr': 'Silence'};
      expect(resolveInline(t, 'fr'), 'Silence');
      expect(resolveInline(t, 'de'), 'Rest'); // → en
      expect(resolveInline(const {'it': 'Pausa'}, 'fr'), 'Pausa'); // → any
      expect(resolveInline(const {}, 'fr'), '');
    });
  });

  group('parseCourseManifest', () {
    test('parses a valid manifest into ordered typed blocks', () {
      final m = parseCourseManifest(
        _json({
          'schemaVersion': 1,
          'id': 'reading-the-staff',
          'instrument': 'piano',
          'track': 'solfege',
          'level': 'beginner',
          'title': {'en': 'Reading the staff', 'fr': 'Lire la portée'},
          'blocks': [
            {
              'type': 'text',
              'text': {'en': 'The staff has 5 lines.'},
            },
            {'type': 'diagram', 'id': 'treble-clef'},
            {
              'type': 'question',
              'prompt': {'en': 'What does a sharp do?'},
              'options': [
                {'en': 'Raises a semitone'},
                {'en': 'Lowers a semitone'},
              ],
              'answerIndex': 0,
            },
            {
              'type': 'playKey',
              'notes': [60],
            },
            {'type': 'score', 'musicXml': '<score/>', 'playable': true},
          ],
        }),
      );
      expect(m, isNotNull);
      expect(m!.id, 'reading-the-staff');
      expect(m.track, 'solfege');
      expect(resolveInline(m.title, 'fr'), 'Lire la portée');
      expect(m.blocks, hasLength(5));
      expect(m.blocks[0], isA<TextBlock>());
      expect(m.blocks[1], isA<DiagramBlock>());
      expect(m.blocks[2], isA<QuestionBlock>());
      expect(m.blocks[3], isA<PlayKeyBlock>());
      expect(m.blocks[4], isA<ScoreBlock>());
      expect((m.blocks[4] as ScoreBlock).playable, isTrue);
    });

    test(
      'an unknown block type degrades to unsupported, course still parses',
      () {
        final m = parseCourseManifest(
          _json({
            'schemaVersion': 1,
            'id': 'c1',
            'blocks': [
              {
                'type': 'text',
                'text': {'en': 'hi'},
              },
              {'type': 'hologram', 'payload': 42},
              {'type': 'diagram', 'id': 'bass-clef'},
            ],
          }),
        );
        expect(m, isNotNull);
        expect(m!.blocks, hasLength(3));
        expect(m.blocks[1], isA<UnsupportedBlock>());
        expect((m.blocks[1] as UnsupportedBlock).type, 'hologram');
        // The blocks around the unsupported one are intact.
        expect(m.blocks[0], isA<TextBlock>());
        expect(m.blocks[2], isA<DiagramBlock>());
      },
    );

    test('a malformed known block degrades to unsupported, never throws', () {
      final m = parseCourseManifest(
        _json({
          'schemaVersion': 1,
          'id': 'c1',
          'blocks': [
            {'type': 'diagram'}, // missing id
            {'type': 'question', 'options': []}, // empty options
            {'type': 'playKey', 'notes': []}, // no notes
          ],
        }),
      );
      expect(m, isNotNull);
      expect(m!.blocks.every((b) => b is UnsupportedBlock), isTrue);
    });

    test('a schemaVersion above this build is declined', () {
      final m = parseCourseManifest(
        _json({'schemaVersion': 999, 'id': 'c1', 'blocks': []}),
      );
      expect(m, isNull);
    });

    test('missing/invalid schemaVersion or id is declined', () {
      expect(parseCourseManifest(_json({'id': 'c1', 'blocks': []})), isNull);
      expect(
        parseCourseManifest(_json({'schemaVersion': 1, 'blocks': []})),
        isNull,
      );
      expect(
        parseCourseManifest(
          _json({'schemaVersion': 1, 'id': '', 'blocks': []}),
        ),
        isNull,
      );
    });

    test('malformed JSON is declined, not thrown', () {
      expect(parseCourseManifest('{not json'), isNull);
      expect(parseCourseManifest('[]'), isNull); // not an object
    });

    test('a higher maxSchemaVersion accepts a newer manifest', () {
      final m = parseCourseManifest(
        _json({'schemaVersion': 2, 'id': 'c1', 'blocks': []}),
        maxSchemaVersion: 2,
      );
      expect(m, isNotNull);
      expect(m!.schemaVersion, 2);
    });

    test('metadata falls back to defaults when absent', () {
      final m = parseCourseManifest(_json({'schemaVersion': 1, 'id': 'c1'}));
      expect(m!.instrument, 'piano');
      expect(m.track, 'solfege');
      expect(m.level, 'beginner');
      expect(m.blocks, isEmpty);
    });
  });
}
