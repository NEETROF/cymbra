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
import 'package:music/courses/lesson_rhythm.dart';
import 'package:music/state/note_label.dart';

void main() {
  group('RhythmFigure', () {
    test('parses figures, dots and rests', () {
      expect(
        RhythmFigure.parse({'fig': 'quarter'}),
        const RhythmFigure(NoteFigure.quarter),
      );
      expect(
        RhythmFigure.parse({'fig': 'half', 'dots': 1}),
        const RhythmFigure(NoteFigure.half, dots: 1),
      );
      expect(
        RhythmFigure.parse({'fig': 'eighth', 'rest': true}),
        const RhythmFigure(NoteFigure.eighth, rest: true),
      );
      expect(RhythmFigure.parse({'fig': '16th'})!.figure, NoteFigure.sixteenth);
    });

    test('declines unknown figures and malformed shapes', () {
      expect(RhythmFigure.parse({'fig': 'breve'}), isNull);
      expect(RhythmFigure.parse({'fig': 42}), isNull);
      expect(RhythmFigure.parse('quarter'), isNull);
      expect(RhythmFigure.parse(null), isNull);
    });

    test('beats follow the meter denominator', () {
      const quarter = RhythmFigure(NoteFigure.quarter);
      expect(quarter.beats(4), 1.0); // one beat in x/4
      expect(quarter.beats(8), 2.0); // two eighth-beats in x/8
      expect(const RhythmFigure(NoteFigure.half, dots: 1).beats(4), 3.0);
      expect(const RhythmFigure(NoteFigure.eighth).beats(4), 0.5);
    });
  });

  group('rhythmOnsets', () {
    test('lays out onsets at the tempo, skipping rests', () {
      // ♩ ♪♪ 𝄽 ♩ at 60 bpm in x/4 → beat = 1000 ms.
      final r = rhythmOnsets(
        pattern: const [
          RhythmFigure(NoteFigure.quarter),
          RhythmFigure(NoteFigure.eighth),
          RhythmFigure(NoteFigure.eighth),
          RhythmFigure(NoteFigure.quarter, rest: true),
          RhythmFigure(NoteFigure.quarter),
        ],
        bpm: 60,
        beatType: 4,
      );
      expect(r.onsetsMs, [0, 1000, 1500, 3000]);
      expect(r.totalMs, 4000);
    });
  });

  group('gradeRhythmTaps', () {
    test('marks onsets hit within the window and counts strays', () {
      final g = gradeRhythmTaps(
        onsetsMs: [0, 1000, 2000],
        tapsMs: [40, 960, 2500, 3200],
        windowMs: 150,
      );
      expect(g.hit, [true, true, false]);
      expect(g.extras, 2);
    });

    test('one tap can never satisfy two onsets', () {
      final g = gradeRhythmTaps(
        onsetsMs: [1000, 1100],
        tapsMs: [1050],
        windowMs: 200,
      );
      expect(g.hit.where((h) => h).length, 1);
      expect(g.extras, 0);
    });

    test('perfect pass', () {
      final g = gradeRhythmTaps(
        onsetsMs: [0, 500, 1000],
        tapsMs: [0, 500, 1000],
        windowMs: 120,
      );
      expect(g.hit, everyElement(isTrue));
      expect(g.extras, 0);
    });
  });
}
