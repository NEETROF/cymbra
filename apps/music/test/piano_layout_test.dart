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
import 'package:music/painters/piano_layout.dart';

void main() {
  group('PianoLayout', () {
    // Default range C4..C6 has 15 white keys.
    const layout = PianoLayout(width: 150);

    test('classifies black and white keys', () {
      expect(PianoLayout.isBlack(60), isFalse); // C4
      expect(PianoLayout.isBlack(61), isTrue); // C#4
      expect(PianoLayout.isBlack(64), isFalse); // E4
      expect(PianoLayout.isBlack(66), isTrue); // F#4
    });

    test('white key width divides the available width by white count', () {
      // 15 white keys across 150px → 10px each.
      expect(layout.whiteWidth, closeTo(10.0, 1e-9));
      expect(layout.blackWidth, closeTo(6.2, 1e-9));
    });

    test('first white key starts at the left edge', () {
      final r = layout.keyRect(60);
      expect(r.left, 0);
      expect(r.width, closeTo(10.0, 1e-9));
    });

    test('second white key is offset by one white width', () {
      expect(layout.keyRect(62).left, closeTo(10.0, 1e-9)); // D4
    });

    test('black key is centered on the boundary between white keys', () {
      final r = layout.keyRect(61); // C#4
      // boundary at 10px, black width 6.2 → left = 10 - 3.1.
      expect(r.left, closeTo(10 - 3.1, 1e-9));
      expect(r.width, closeTo(6.2, 1e-9));
    });

    test('centerX returns the horizontal middle of a key', () {
      expect(layout.centerX(60), closeTo(5.0, 1e-9));
    });

    test('contains respects the pitch range', () {
      expect(layout.contains(60), isTrue);
      expect(layout.contains(84), isTrue);
      expect(layout.contains(59), isFalse);
      expect(layout.contains(85), isFalse);
    });

    group('pitchAt', () {
      // 150px height: black keys occupy the upper 62% (y < 93), white keys the
      // full height. whiteWidth 10, blackWidth 6.2.
      const h = 150.0;
      const whiteY = 120.0; // below the black band → white-only region
      const blackY = 30.0; // inside the black band

      test('white key hit in the white-only region', () {
        // C4 is the first white key [0,10); its center is x=5.
        expect(layout.pitchAt(const Offset(5, whiteY), h), 60);
        // D4 is the second white key [10,20).
        expect(layout.pitchAt(const Offset(15, whiteY), h), 62);
      });

      test('black key takes priority in the overlap band', () {
        // C#4 is centered on the C4/D4 boundary (x=10), rect [6.9,13.1). In the
        // black band a hit there is the black key, not a white one.
        expect(layout.pitchAt(const Offset(10, blackY), h), 61);
      });

      test('same x below the black band hits the white key', () {
        // At x=10 but in the white-only region, the black key is not present, so
        // the hit falls through to the white key starting at that boundary (D4).
        expect(layout.pitchAt(const Offset(10, whiteY), h), 62);
      });

      test('out of range returns null', () {
        expect(layout.pitchAt(const Offset(-1, whiteY), h), isNull);
        expect(layout.pitchAt(const Offset(200, whiteY), h), isNull);
        expect(layout.pitchAt(const Offset(5, -1), h), isNull);
        expect(layout.pitchAt(const Offset(5, h + 1), h), isNull);
      });

      test('left edge of the keyboard hits the first key', () {
        expect(layout.pitchAt(const Offset(0, whiteY), h), 60);
      });
    });
  });
}
