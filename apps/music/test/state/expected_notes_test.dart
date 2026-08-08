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
  // A right-hand chord at 0, a left-hand note at 0, and a later right-hand note.
  const data = PlayerData(
    notes: [
      TimedNote(pitch: 60, startMs: 0, durationMs: 400, staff: 1, diatonic: 28),
      TimedNote(pitch: 64, startMs: 0, durationMs: 400, staff: 1, diatonic: 30),
      TimedNote(pitch: 36, startMs: 0, durationMs: 400, staff: 2, diatonic: 14),
      TimedNote(pitch: 67, startMs: 1000, durationMs: 400, staff: 1),
    ],
    songEndMs: 2000,
  );

  group('expectedNotes', () {
    test('agrees with expectedKeys on the pitches it names', () {
      for (final t in [0.0, 200.0, 1000.0]) {
        for (final wait in [true, false]) {
          final s = data.copyWith(elapsedMs: t, waitMode: wait);
          expect(
            s.expectedNotes.map((n) => n.pitch).toSet(),
            s.expectedKeys,
            reason: 'at t=$t waitMode=$wait',
          );
        }
      }
    });

    test('carries the written spelling the pitch set throws away', () {
      final s = data.copyWith(elapsedMs: 0);
      expect(s.expectedNotes.map((n) => n.diatonic), containsAll([28, 30, 14]));
    });

    test('returns the upcoming onset while the playhead travels', () {
      // At 600 ms nothing is attacking; Wait Mode previews the 1000 ms onset.
      final s = data.copyWith(elapsedMs: 600);
      expect(s.expectedNotes.map((n) => n.pitch), [67]);
    });

    test('follows the selected hand', () {
      final right = data.copyWith(elapsedMs: 0, selectedHands: Hand.right);
      expect(right.expectedNotes.map((n) => n.pitch).toSet(), {60, 64});
      final left = data.copyWith(elapsedMs: 0, selectedHands: Hand.left);
      expect(left.expectedNotes.map((n) => n.pitch).toSet(), {36});
    });

    test('outside Wait Mode names the notes sounding under the playhead', () {
      final s = data.copyWith(elapsedMs: 200, waitMode: false);
      expect(s.expectedNotes.map((n) => n.pitch).toSet(), {60, 64, 36});
    });

    test('is empty when no onset is left to point at', () {
      final s = data.copyWith(elapsedMs: 1500);
      expect(s.expectedTimeMs, isNull);
      expect(s.expectedNotes, isEmpty);
      expect(s.expectedKeys, isEmpty);
      expect(s.expectedKeysForHand(rightHand: true), isEmpty);
    });

    test('is empty for a score with no notes', () {
      const empty = PlayerData();
      expect(empty.expectedNotes, isEmpty);
    });
  });

  group('expectedTimeMs', () {
    test('is the playhead itself outside Wait Mode', () {
      expect(
        data.copyWith(elapsedMs: 250, waitMode: false).expectedTimeMs,
        250,
      );
    });

    test('is the onset under the playhead in Wait Mode', () {
      expect(data.copyWith(elapsedMs: 0).expectedTimeMs, 0);
    });

    test('is the upcoming onset while travelling', () {
      expect(data.copyWith(elapsedMs: 600).expectedTimeMs, 1000);
    });
  });

  group('beatDurationMsAt', () {
    test(
      'divides the measure span by the beat count when measures are known',
      () {
        const withMeasures = PlayerData(
          measureStartMs: [0, 2000, 4000],
          songEndMs: 6000,
          beats: 4,
          bpm: 200,
        );
        // A 2000 ms measure of four beats = 500 ms per beat, whatever the bpm says.
        expect(withMeasures.beatDurationMsAt(0), 500);
        expect(withMeasures.beatDurationMsAt(2500), 500);
        // The last measure runs to the end of the song.
        expect(withMeasures.beatDurationMsAt(4500), 500);
      },
    );

    test('falls back to the tempo when there is no measure table', () {
      const demo = PlayerData(bpm: 120, songEndMs: 1000);
      expect(demo.beatDurationMsAt(0), 500);
    });

    test('falls back to the tempo outside the measure table', () {
      const withMeasures = PlayerData(
        measureStartMs: [1000, 2000],
        songEndMs: 3000,
        beats: 4,
        bpm: 60,
      );
      expect(withMeasures.beatDurationMsAt(0), 1000);
    });

    test('returns 0 when neither measures nor tempo are usable', () {
      const none = PlayerData(bpm: 0);
      expect(none.beatDurationMsAt(0), 0);
    });
  });
}
