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
import 'package:music/courses/lesson_pitch.dart';
import 'package:music/state/note_label.dart';

CourseBlock parseOne(Map<String, Object?> block) {
  final manifest = parseCourseManifest(
    jsonEncode({
      'schemaVersion': 2,
      'id': 'c',
      'blocks': [block],
    }),
  );
  return manifest!.blocks.single;
}

void main() {
  test('schemaVersion 2 is accepted and 3 is declined', () {
    expect(parseCourseManifest('{"schemaVersion":2,"id":"a"}'), isNotNull);
    expect(parseCourseManifest('{"schemaVersion":3,"id":"a"}'), isNull);
  });

  group('staff block', () {
    test('parses clef, key, time, notes and rests', () {
      final block = parseOne({
        'type': 'staff',
        'clef': 'bass',
        'keyFifths': -2,
        'time': {'beats': 3, 'beatType': 4},
        'labels': true,
        'elements': [
          {'p': 'F3', 'fig': 'quarter'},
          {'rest': true, 'fig': 'quarter'},
          {'p': 'C3', 'fig': 'half', 'dots': 1},
        ],
        'caption': {'en': 'A staff', 'fr': 'Une portée'},
      });
      final staff = block as StaffBlock;
      expect(staff.clef, LessonClef.bass);
      expect(staff.keyFifths, -2);
      expect(staff.time, const LessonTimeSig(3, 4));
      expect(staff.labels, isTrue);
      expect(staff.elements, hasLength(3));
      expect(staff.elements[0].pitch, LessonPitch.parse('F3'));
      expect(staff.elements[1].fig.rest, isTrue);
      expect(staff.elements[1].pitch, isNull);
      expect(staff.elements[2].fig.dots, 1);
      expect(resolveInline(staff.caption, 'fr'), 'Une portée');
    });

    test('declines a malformed element or a missing clef', () {
      expect(
        parseOne({
          'type': 'staff',
          'clef': 'treble',
          'elements': [
            {'p': 'H4', 'fig': 'quarter'},
          ],
        }),
        isA<UnsupportedBlock>(),
      );
      expect(
        parseOne({
          'type': 'staff',
          'elements': [
            {'p': 'C4', 'fig': 'quarter'},
          ],
        }),
        isA<UnsupportedBlock>(),
      );
    });
  });

  group('readPlay block', () {
    test('parses notes, mode, labels and key', () {
      final block = parseOne({
        'type': 'readPlay',
        'notes': ['C4', 'E4', 'G4'],
        'mode': 'melody',
        'clef': 'treble',
        'keyFifths': 1,
        'labels': 'never',
        'prompt': {'en': 'Play it'},
      });
      final rp = block as ReadPlayBlock;
      expect(rp.notes.map((p) => p.midi), [60, 64, 67]);
      expect(rp.mode, ReadPlayMode.melody);
      expect(rp.clef, LessonClef.treble);
      expect(rp.keyFifths, 1);
      expect(rp.labels, LessonLabelMode.never);
    });

    test('defaults: drill mode, afterMiss labels, no explicit clef', () {
      final rp =
          parseOne({
                'type': 'readPlay',
                'notes': ['A3'],
              })
              as ReadPlayBlock;
      expect(rp.mode, ReadPlayMode.drill);
      expect(rp.labels, LessonLabelMode.afterMiss);
      expect(rp.clef, isNull);
    });

    test('one bad spelling declines the whole block', () {
      expect(
        parseOne({
          'type': 'readPlay',
          'notes': ['C4', 'X2'],
        }),
        isA<UnsupportedBlock>(),
      );
    });
  });

  group('nameNote block', () {
    test('parses and clamps the choice count', () {
      final block =
          parseOne({
                'type': 'nameNote',
                'notes': ['F#4'],
                'choiceCount': 9,
              })
              as NameNoteBlock;
      expect(block.notes.single.name, const NoteName(3, 1));
      expect(block.choiceCount, 4);
    });
  });

  group('placeNote block', () {
    test('parses natural targets', () {
      final block =
          parseOne({
                'type': 'placeNote',
                'clef': 'treble',
                'targets': ['E4', 'G4'],
              })
              as PlaceNoteBlock;
      expect(block.targets, hasLength(2));
      expect(block.clef, LessonClef.treble);
    });

    test('declines altered targets and a missing clef', () {
      expect(
        parseOne({
          'type': 'placeNote',
          'clef': 'treble',
          'targets': ['F#4'],
        }),
        isA<UnsupportedBlock>(),
      );
      expect(
        parseOne({
          'type': 'placeNote',
          'targets': ['F4'],
        }),
        isA<UnsupportedBlock>(),
      );
    });
  });

  group('rhythmTap block', () {
    test('parses the pattern with clamped envelope', () {
      final block =
          parseOne({
                'type': 'rhythmTap',
                'beats': 4,
                'beatType': 4,
                'bpm': 300,
                'passRatio': 2,
                'pattern': [
                  {'fig': 'quarter'},
                  {'fig': 'eighth'},
                  {'fig': 'eighth'},
                  {'rest': true, 'fig': 'quarter'},
                  {'fig': 'half'},
                ],
              })
              as RhythmTapBlock;
      expect(block.pattern, hasLength(5));
      expect(block.pattern[3].rest, isTrue);
      expect(block.bpm, 240);
      expect(block.passRatio, 1.0);
    });

    test('declines an unknown figure and an all-rest pattern', () {
      expect(
        parseOne({
          'type': 'rhythmTap',
          'pattern': [
            {'fig': 'breve'},
          ],
        }),
        isA<UnsupportedBlock>(),
      );
      expect(
        parseOne({
          'type': 'rhythmTap',
          'pattern': [
            {'rest': true, 'fig': 'quarter'},
          ],
        }),
        isA<UnsupportedBlock>(),
      );
    });
  });

  group('earChoice block', () {
    Map<String, Object?> ear({Object? answerId = 'up', Object? choices}) => {
      'type': 'earChoice',
      'notes': ['C4', 'G4'],
      'answerId': answerId,
      'choices':
          choices ??
          [
            {
              'id': 'up',
              'label': {'en': 'Higher'},
            },
            {
              'id': 'down',
              'label': {'en': 'Lower'},
            },
          ],
    };

    test('parses choices and validates the answer id', () {
      final block = parseOne(ear()) as EarChoiceBlock;
      expect(block.choices.map((c) => c.id), ['up', 'down']);
      expect(block.answerId, 'up');
      expect(block.reveal, isTrue);
      expect(block.harmonic, isFalse);
      expect(block.gapMs, 700);
    });

    test('declines an answer id that matches no choice', () {
      expect(parseOne(ear(answerId: 'sideways')), isA<UnsupportedBlock>());
    });

    test('declines fewer than two choices', () {
      expect(
        parseOne(
          ear(
            choices: [
              {
                'id': 'up',
                'label': {'en': 'Higher'},
              },
            ],
          ),
        ),
        isA<UnsupportedBlock>(),
      );
    });
  });

  group('buildChord block', () {
    test('parses a triad', () {
      final block =
          parseOne({
                'type': 'buildChord',
                'notes': ['C4', 'E4', 'G4'],
              })
              as BuildChordBlock;
      expect(block.notes.map((p) => p.midi), [60, 64, 67]);
    });

    test('declines fewer than 2 or more than 5 notes', () {
      expect(
        parseOne({
          'type': 'buildChord',
          'notes': ['C4'],
        }),
        isA<UnsupportedBlock>(),
      );
      expect(
        parseOne({
          'type': 'buildChord',
          'notes': ['C4', 'D4', 'E4', 'F4', 'G4', 'A4'],
        }),
        isA<UnsupportedBlock>(),
      );
    });
  });

  test('a v1 manifest with v1 blocks still parses under v2', () {
    final manifest = parseCourseManifest(
      jsonEncode({
        'schemaVersion': 1,
        'id': 'legacy',
        'blocks': [
          {
            'type': 'text',
            'text': {'en': 'hello'},
          },
          {
            'type': 'playKey',
            'notes': [60],
          },
        ],
      }),
    );
    expect(manifest!.blocks, hasLength(2));
    expect(manifest.blocks[0], isA<TextBlock>());
    expect(manifest.blocks[1], isA<PlayKeyBlock>());
  });
}
