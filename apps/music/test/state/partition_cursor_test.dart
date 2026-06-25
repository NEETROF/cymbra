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
import 'package:music/state/notation_playback.dart';
import 'package:music/state/player_data.dart';

import '../support/notation_fakes.dart';

void main() {
  group('measureStartMs derivation', () {
    test('first measure starts at 0 and later measures are spaced by time', () {
      // Two 4/4 measures, divisions=4, default tempo 90 bpm.
      // ms/division = (60000/90)/4 = 166.67; a full measure = 16 divisions.
      final derived = notationToTimedNotes(sampleClefChangeDocument());
      expect(derived.measureStartMs.length, 2);
      expect(derived.measureStartMs.first, 0);
      expect(derived.measureStartMs[1], closeTo(2667, 2));
    });
  });

  group('PlayerData.measureAt', () {
    const data = PlayerData(measureStartMs: [0, 1000, 2000], songEndMs: 3000);

    test('maps a time inside a measure to index + fraction', () {
      expect(data.measureAt(500)?.index, 0);
      expect(data.measureAt(500)?.fraction, closeTo(0.5, 1e-9));
      expect(data.measureAt(1000)?.index, 1);
      expect(data.measureAt(1000)?.fraction, closeTo(0.0, 1e-9));
      expect(data.measureAt(2500)?.index, 2);
      expect(data.measureAt(2500)?.fraction, closeTo(0.5, 1e-9));
    });

    test('returns null before the start and at/after the end', () {
      expect(data.measureAt(-1), isNull);
      expect(data.measureAt(3000), isNull);
      expect(data.measureAt(4000), isNull);
    });

    test('returns null when there is no timing', () {
      const empty = PlayerData();
      expect(empty.measureAt(0), isNull);
    });
  });
}
