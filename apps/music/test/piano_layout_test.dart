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
  });
}
