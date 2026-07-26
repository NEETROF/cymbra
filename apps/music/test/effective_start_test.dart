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

TimedNote _note(
  int pitch,
  int startMs, {
  int durationMs = 500,
  int staff = 1,
}) => TimedNote(
  pitch: pitch,
  startMs: startMs,
  durationMs: durationMs,
  staff: staff,
);

void main() {
  group('effectiveStartMs (trim leading silence)', () {
    test('no notes → 0 (nothing to trim)', () {
      expect(effectiveStartMs(const []), 0);
    });

    test('first onset well past the lead-in → onset − lead-in', () {
      // First note in the 3rd measure (80bpm 4/4 → 3000ms/measure).
      final notes = [_note(60, 6000), _note(62, 6500)];
      expect(effectiveStartMs(notes, leadInMs: 1000), 5000);
    });

    test('first onset within the lead-in budget → clamped to 0', () {
      expect(effectiveStartMs([_note(60, 400)], leadInMs: 1000), 0);
    });

    test('first onset exactly at the lead-in → 0 (boundary)', () {
      expect(effectiveStartMs([_note(60, 1000)], leadInMs: 1000), 0);
    });

    test('a piece starting at time zero is unchanged (→ 0)', () {
      expect(effectiveStartMs([_note(60, 0), _note(62, 500)]), 0);
    });

    test('uses the minimum onset even when unsorted', () {
      final notes = [_note(64, 9000), _note(60, 4000), _note(62, 6000)];
      expect(effectiveStartMs(notes, leadInMs: 1000), 3000);
    });

    test('defaults to the shared kStartLeadInMs constant', () {
      expect(effectiveStartMs([_note(60, 8000)]), 8000 - kStartLeadInMs);
    });
  });

  group('PlayerData.startMs is selection-scoped', () {
    // Right hand = staff 1, left hand = staff 2+. A late-entering hand trims to
    // its own first note.
    final data = PlayerData(
      notes: [
        _note(48, 6000, staff: 2), // left hand enters in measure 3
        _note(60, 12000, staff: 1), // right hand enters even later
      ],
    );

    test('both hands → earliest onset of either hand', () {
      expect(data.copyWith(selectedHands: Hand.both).startMs, 5000);
    });

    test('left hand → that hand’s first note', () {
      expect(data.copyWith(selectedHands: Hand.left).startMs, 5000);
    });

    test('right hand → that hand’s (later) first note', () {
      expect(data.copyWith(selectedHands: Hand.right).startMs, 11000);
    });

    test('a selection with no notes → 0', () {
      final rightOnly = PlayerData(notes: [_note(48, 6000, staff: 2)]);
      expect(rightOnly.copyWith(selectedHands: Hand.right).startMs, 0);
    });
  });
}
