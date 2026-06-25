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
  // C4 + E4 as a chord at t=0, then D4 at t=500.
  const data = PlayerData(
    notes: [
      TimedNote(pitch: 60, startMs: 0, durationMs: 500),
      TimedNote(pitch: 64, startMs: 0, durationMs: 500),
      TimedNote(pitch: 62, startMs: 500, durationMs: 500),
    ],
    songEndMs: 1000,
    waitMode: true,
  );

  group('onsetPitchesAt', () {
    test('returns all pitches whose onset is at the time', () {
      expect(data.onsetPitchesAt(0), {60, 64});
      expect(data.onsetPitchesAt(500), {62});
    });

    test('is empty between onsets', () {
      expect(data.onsetPitchesAt(250), isEmpty);
    });
  });

  group('nextOnsetAfter', () {
    test('finds the next onset strictly after t', () {
      expect(data.nextOnsetAfter(0), 500);
      expect(data.nextOnsetAfter(250), 500);
    });

    test('is null past the last onset', () {
      expect(data.nextOnsetAfter(500), isNull);
    });
  });

  group('expectedKeys', () {
    test('shows the onset at the playhead when sitting on one', () {
      expect(data.onsetPitchesAt(0), {60, 64});
      expect(data.expectedKeys, {60, 64});
    });

    test('shows the upcoming onset while travelling between onsets', () {
      // At t=250 no note starts, so the preview points at the next note (D4).
      expect(data.copyWith(elapsedMs: 250).expectedKeys, {62});
    });

    test('falls back to the sounding notes when Wait Mode is off', () {
      // Window-based: at t=250 the chord [0,500) is still sounding.
      final free = data.copyWith(waitMode: false, elapsedMs: 250);
      expect(free.expectedKeys, {60, 64});
    });
  });
}
