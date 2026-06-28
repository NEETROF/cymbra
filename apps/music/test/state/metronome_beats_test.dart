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

/// Convenience: run the helper with sensible defaults so each test names only the
/// fields it cares about.
List<MetronomeBeat> beats({
  List<int> measureStartMs = const [],
  int beatsPerMeasure = 4,
  int bpm = 120,
  double songEndMs = 0,
  required double from,
  required double to,
}) => metronomeBeatsCrossed(
  measureStartMs: measureStartMs,
  beats: beatsPerMeasure,
  bpm: bpm,
  songEndMs: songEndMs,
  from: from,
  to: to,
);

void main() {
  group('metronomeBeatsCrossed — measure-based', () {
    // Three 2000ms measures in 4/4 → 500ms per beat.
    const measures = [0, 2000, 4000];
    const songEnd = 6000.0;

    test('one tick per beat across the whole piece, downbeats accented', () {
      final result = beats(
        measureStartMs: measures,
        songEndMs: songEnd,
        from: 0,
        to: 6000,
      );
      expect(result.length, 12); // 3 measures × 4 beats
      expect(result.map((b) => b.timeMs).toList(), [
        0, 500, 1000, 1500, // measure 0
        2000, 2500, 3000, 3500, // measure 1
        4000, 4500, 5000, 5500, // measure 2
      ]);
      // Accent only on each measure start (the downbeat).
      expect(result.where((b) => b.accent).map((b) => b.timeMs).toList(), [
        0,
        2000,
        4000,
      ]);
    });

    test('number of ticks per measure follows the time signature (3/4)', () {
      // Same 2000ms measures but in 3/4 → 3 beats each, ~666.7ms apart.
      final result = beats(
        measureStartMs: measures,
        beatsPerMeasure: 3,
        songEndMs: songEnd,
        from: 0,
        to: 2000,
      );
      expect(result.length, 3);
      expect(result.first.accent, isTrue); // downbeat
      expect(result.skip(1).every((b) => !b.accent), isTrue);
    });

    test('a frozen playhead (from == to) yields no beats', () {
      expect(
        beats(measureStartMs: measures, songEndMs: songEnd, from: 500, to: 500),
        isEmpty,
      );
    });

    test('multiple boundaries in one span each fire exactly once', () {
      final result = beats(
        measureStartMs: measures,
        songEndMs: songEnd,
        from: 0,
        to: 1600,
      );
      expect(result.map((b) => b.timeMs).toList(), [0, 500, 1000, 1500]);
    });

    test('half-open span: includes from, excludes to', () {
      // [500, 1000): the beat at 500 is included, the one at 1000 is not.
      final result = beats(
        measureStartMs: measures,
        songEndMs: songEnd,
        from: 500,
        to: 1000,
      );
      expect(result.map((b) => b.timeMs).toList(), [500]);
    });

    test('a mid-measure downbeat crossing is accented', () {
      final result = beats(
        measureStartMs: measures,
        songEndMs: songEnd,
        from: 1900,
        to: 2100,
      );
      expect(result.length, 1);
      expect(result.single.timeMs, 2000);
      expect(result.single.accent, isTrue);
    });
  });

  group('metronomeBeatsCrossed — tempo fallback (no measure table)', () {
    test('derives a steady beat from bpm, accenting every Nth', () {
      // 120 bpm → 500ms per beat; 4/4 accents beats 0, 4, 8, …
      final result = beats(bpm: 120, beatsPerMeasure: 4, from: 0, to: 2000);
      expect(result.map((b) => b.timeMs).toList(), [0, 500, 1000, 1500]);
      expect(result.first.accent, isTrue);
      expect(result.skip(1).every((b) => !b.accent), isTrue);
    });

    test('accents land every N beats from the origin', () {
      // Span covering beats 4..7 (2000..3500): beat 4 (2000ms) is a downbeat.
      final result = beats(bpm: 120, beatsPerMeasure: 4, from: 2000, to: 4000);
      expect(result.map((b) => b.timeMs).toList(), [2000, 2500, 3000, 3500]);
      expect(result.first.accent, isTrue); // beat index 4 → 4 % 4 == 0
      expect(result.skip(1).every((b) => !b.accent), isTrue);
    });
  });

  group('metronomeBeatsCrossed — guards', () {
    test('beats < 1 yields nothing', () {
      expect(beats(beatsPerMeasure: 0, from: 0, to: 5000), isEmpty);
    });

    test('non-positive bpm with no measures yields nothing', () {
      expect(beats(bpm: 0, from: 0, to: 5000), isEmpty);
    });

    test('reversed/empty span yields nothing', () {
      expect(
        beats(measureStartMs: const [0, 2000], from: 1000, to: 1000),
        isEmpty,
      );
    });
  });
}
