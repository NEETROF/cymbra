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
  group('effectiveEndMs (trim trailing silence)', () {
    test('no notes → falls back to songEndMs (nothing to trim)', () {
      expect(effectiveEndMs(const [], songEndMs: 12000), 12000);
    });

    test('last note resolves before songEndMs → last note resolution', () {
      // Last note ends at 6500; the piece runs on to 12000 with trailing rests.
      final notes = [_note(60, 4000), _note(62, 6000, durationMs: 500)];
      expect(effectiveEndMs(notes, songEndMs: 12000), 6500);
    });

    test('uses the maximum resolution even when unsorted', () {
      final notes = [
        _note(64, 9000, durationMs: 500), // resolves at 9500
        _note(60, 4000),
        _note(62, 6000),
      ];
      expect(effectiveEndMs(notes, songEndMs: 20000), 9500);
    });

    test('duration is included, not just the onset', () {
      // A single held note: end is onset + duration, not the onset.
      expect(
        effectiveEndMs([_note(60, 4000, durationMs: 2000)], songEndMs: 20000),
        6000,
      );
    });

    test('a note held past songEndMs is clamped to songEndMs', () {
      expect(
        effectiveEndMs([_note(60, 4000, durationMs: 5000)], songEndMs: 8000),
        8000,
      );
    });

    test('earlier rests do not matter — only the last note counts', () {
      // Rests live in a separate channel; effectiveEndMs sees notes only, so a
      // gap partway through never pulls the end earlier than the last note.
      final notes = [_note(60, 0), _note(62, 8000, durationMs: 500)];
      expect(effectiveEndMs(notes, songEndMs: 12000), 8500);
    });
  });

  group('PlayerData.endMs is selection-scoped', () {
    // Right hand = staff 1, left hand = staff 2+. Each hand trims to its own
    // last note; songEndMs (with trailing rests) is 20000.
    final data = PlayerData(
      songEndMs: 20000,
      notes: [
        _note(48, 6000, staff: 2), // left hand's last note → resolves 6500
        _note(60, 12000, staff: 1), // right hand's last note → resolves 12500
      ],
    );

    test('both hands → latest resolution of either hand', () {
      expect(data.copyWith(selectedHands: Hand.both).endMs, 12500);
    });

    test('right hand → that hand’s (later) last note', () {
      expect(data.copyWith(selectedHands: Hand.right).endMs, 12500);
    });

    test('left hand → that hand’s (earlier) last note', () {
      expect(data.copyWith(selectedHands: Hand.left).endMs, 6500);
    });

    test('a selection with no notes → falls back to songEndMs', () {
      final leftOnly = PlayerData(
        songEndMs: 20000,
        notes: [_note(48, 6000, staff: 2)],
      );
      expect(leftOnly.copyWith(selectedHands: Hand.right).endMs, 20000);
    });

    test('endMs is greater than startMs when a note exists', () {
      final d = data.copyWith(selectedHands: Hand.both);
      expect(d.endMs, greaterThan(d.startMs));
    });
  });
}
