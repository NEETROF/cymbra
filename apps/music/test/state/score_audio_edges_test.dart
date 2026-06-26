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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/state/player_data.dart';

void main() {
  // C4 [0,500), D4 [500,1000).
  const notes = [
    TimedNote(pitch: 60, startMs: 0, durationMs: 500),
    TimedNote(pitch: 62, startMs: 500, durationMs: 500),
  ];

  group('scoreNoteEdges', () {
    test('an onset crossed by the span starts that pitch', () {
      final e = scoreNoteEdges(
        visible: notes,
        from: 0,
        to: 50,
        sounding: const {},
      );
      expect(e.starts, [60]);
      expect(e.stops, isEmpty);
    });

    test('half-open span: a frozen onset (from == to) does not pre-sound', () {
      final e = scoreNoteEdges(
        visible: notes,
        from: 0,
        to: 0,
        sounding: const {},
      );
      expect(e.starts, isEmpty);
    });

    test('an onset exactly at the upper bound is not started yet', () {
      // [0,500) excludes 500: D4 sounds only once the playhead passes 500.
      final e = scoreNoteEdges(
        visible: notes,
        from: 0,
        to: 500,
        sounding: const {},
      );
      expect(e.starts, [60]);
      expect(e.starts, isNot(contains(62)));
    });

    test('a sounding note is stopped when its end is passed', () {
      // Crossing 500: C4 [0,500) no longer covers 500 → stop; D4 starts.
      final e = scoreNoteEdges(
        visible: notes,
        from: 480,
        to: 520,
        sounding: const {60},
      );
      expect(e.stops, [60]);
      expect(e.starts, [62]);
    });

    test('a still-covered sounding note is not stopped', () {
      final e = scoreNoteEdges(
        visible: notes,
        from: 100,
        to: 200,
        sounding: const {60},
      );
      expect(e.stops, isEmpty);
      expect(e.starts, isEmpty);
    });

    test('a chord onset starts every pitch at once', () {
      const chord = [
        TimedNote(pitch: 60, startMs: 0, durationMs: 500),
        TimedNote(pitch: 64, startMs: 0, durationMs: 500),
        TimedNote(pitch: 67, startMs: 0, durationMs: 500),
      ];
      final e = scoreNoteEdges(
        visible: chord,
        from: 0,
        to: 10,
        sounding: const {},
      );
      expect(e.starts..sort(), [60, 64, 67]);
    });

    test('hidden-hand notes (absent from visible) are neither started nor '
        'stopped', () {
      // Only the right hand is visible; a left-hand note must not appear.
      const right = [TimedNote(pitch: 72, startMs: 0, durationMs: 500)];
      final e = scoreNoteEdges(
        visible: right,
        from: 0,
        to: 50,
        sounding: const {48}, // a left-hand voice the caller never started here
      );
      expect(e.starts, [72]);
      // 48 isn't covered by any visible note, so it would be released — which is
      // exactly what the player wants after a hand switch hides that staff.
      expect(e.stops, [48]);
    });
  });
}
