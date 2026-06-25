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
  // A two-hand timeline: right hand (staff 1) and left hand (staff 2), with a
  // left-only region at [1000, 1500) so the gate's hidden-hand behaviour can be
  // checked (a left note there must not be required/awaited when Right is shown).
  const notes = [
    TimedNote(pitch: 60, startMs: 0, durationMs: 500), // R: C4
    TimedNote(pitch: 48, startMs: 0, durationMs: 500, staff: 2), // L: C3
    TimedNote(pitch: 64, startMs: 500, durationMs: 500), // R: E4
    TimedNote(
      pitch: 50,
      startMs: 1000,
      durationMs: 500,
      staff: 2,
    ), // L only: D3
  ];

  PlayerData withHand(Hand hand) =>
      const PlayerData(notes: notes).copyWith(selectedHands: hand);

  group('showsStaff', () {
    test('both shows every staff', () {
      final d = withHand(Hand.both);
      expect(d.showsStaff(1), isTrue);
      expect(d.showsStaff(2), isTrue);
    });

    test('right shows only staff 1', () {
      final d = withHand(Hand.right);
      expect(d.showsStaff(1), isTrue);
      expect(d.showsStaff(2), isFalse);
    });

    test('left shows only staff 2+', () {
      final d = withHand(Hand.left);
      expect(d.showsStaff(1), isFalse);
      expect(d.showsStaff(2), isTrue);
      expect(d.showsStaff(3), isTrue);
    });
  });

  group('visibleNotes', () {
    test('both keeps every note', () {
      expect(withHand(Hand.both).visibleNotes, hasLength(4));
    });

    test('right keeps only staff-1 notes', () {
      final v = withHand(Hand.right).visibleNotes;
      expect(v.map((n) => n.pitch), [60, 64]);
      expect(v.every((n) => n.staff == 1), isTrue);
    });

    test('left keeps only staff-2 notes', () {
      final v = withHand(Hand.left).visibleNotes;
      expect(v.map((n) => n.pitch), [48, 50]);
      expect(v.every((n) => n.staff >= 2), isTrue);
    });
  });

  group('requiredNotesAt respects the selection', () {
    test('both awaits both hands at the shared onset', () {
      expect(withHand(Hand.both).requiredNotesAt(0), {60, 48});
    });

    test('right awaits only the right hand', () {
      expect(withHand(Hand.right).requiredNotesAt(0), {60});
    });

    test('left awaits only the left hand', () {
      expect(withHand(Hand.left).requiredNotesAt(0), {48});
    });

    test('a hidden-hand note is absent from the required set', () {
      // At t=1000 only the left-hand D3 sounds; with Right selected the gate is
      // empty, so Wait Mode has nothing to wait for and advances.
      final right = withHand(Hand.right);
      expect(right.requiredNotesAt(1000), isEmpty);
      expect(right.onsetPitchesAt(1000), isEmpty);
      // The same instant still gates the left hand when Left is shown.
      expect(withHand(Hand.left).requiredNotesAt(1000), {50});
    });

    test('nextOnsetAfter skips the hidden hand', () {
      // After the last right-hand onset (500) there is only a left onset (1000),
      // which Right must not pause on.
      expect(withHand(Hand.right).nextOnsetAfter(600), isNull);
      expect(withHand(Hand.left).nextOnsetAfter(600), 1000);
      expect(withHand(Hand.both).nextOnsetAfter(600), 1000);
    });
  });

  group('hasMultipleStaves', () {
    test('true when a left-hand note exists', () {
      expect(const PlayerData(notes: notes).hasMultipleStaves, isTrue);
    });

    test('false for a single-staff piece', () {
      const single = PlayerData(
        notes: [TimedNote(pitch: 60, startMs: 0, durationMs: 500)],
      );
      expect(single.hasMultipleStaves, isFalse);
    });
  });
}
