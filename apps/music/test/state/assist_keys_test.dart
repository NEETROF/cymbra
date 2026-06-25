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
  group('expectedNotesForHand', () {
    // C4 right-hand (staff 1) and C2 left-hand (staff 2) both due at t=0.
    const data = PlayerData(
      notes: [
        TimedNote(pitch: 60, startMs: 0, durationMs: 500, staff: 1),
        TimedNote(pitch: 36, startMs: 0, durationMs: 500, staff: 2),
        TimedNote(pitch: 64, startMs: 500, durationMs: 500, staff: 1),
      ],
    );

    test('right hand returns staff-1 notes due now', () {
      expect(data.expectedNotesForHand(0, rightHand: true), {60});
    });

    test('left hand returns staff-2 notes due now', () {
      expect(data.expectedNotesForHand(0, rightHand: false), {36});
    });

    test('empty when nothing is due for the hand', () {
      // At t=600 only the staff-1 E4 is due; the left hand has nothing.
      expect(data.expectedNotesForHand(600, rightHand: false), isEmpty);
      expect(data.expectedNotesForHand(600, rightHand: true), {64});
    });
  });

  group('nearMissPitch', () {
    test('is near, never equal to an expected note, and in range', () {
      // Deterministic pick (index 0) → first candidate, expected-1 = 59.
      final p = nearMissPitch(
        60,
        lowBound: 21,
        highBound: 108,
        avoid: {60},
        nextRandom: (_) => 0,
      );
      expect(p, isNot(60));
      expect((p - 60).abs(), lessThanOrEqualTo(3));
      expect(p, inInclusiveRange(21, 108));
    });

    test('avoids every expected pitch (chord)', () {
      // Avoid 60 and the immediate neighbours; the picker must skip them.
      for (var i = 0; i < 6; i++) {
        final p = nearMissPitch(
          60,
          lowBound: 21,
          highBound: 108,
          avoid: {59, 60, 61},
          nextRandom: (n) => i % n,
        );
        expect({59, 60, 61}.contains(p), isFalse);
        expect((p - 60).abs(), lessThanOrEqualTo(3));
      }
    });

    test('stays in range at the keyboard edge', () {
      // Expected at the very top: the only near pitches are below it.
      final p = nearMissPitch(
        108,
        lowBound: 21,
        highBound: 108,
        avoid: {108},
        nextRandom: (_) => 0,
      );
      expect(p, lessThan(108));
      expect(p, greaterThanOrEqualTo(105));
    });
  });
}
