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
import 'package:music/painters/keyboard_range.dart';

void main() {
  group('computeKeyboardRange — auto', () {
    test('empty score falls back to C4..C6', () {
      final r = computeKeyboardRange(KeyboardRangeMode.auto, const []);
      expect(r.low, 60);
      expect(r.high, 84);
    });

    test('covers the piece and snaps to octave boundaries', () {
      // D3 (50) .. F5 (77).
      final r = computeKeyboardRange(KeyboardRangeMode.auto, const [50, 77]);
      expect(r.low, lessThanOrEqualTo(50));
      expect(r.high, greaterThanOrEqualTo(77));
      expect(r.low % 12, 0, reason: 'low snaps down to a C');
    });

    test('enforces a minimum two-octave span for a sparse piece', () {
      final r = computeKeyboardRange(KeyboardRangeMode.auto, const [60, 61]);
      expect(r.high - r.low, greaterThanOrEqualTo(24));
      expect(r.low, lessThanOrEqualTo(60));
      expect(r.high, greaterThanOrEqualTo(61));
    });

    test('clamps to the lowest piano key (A0)', () {
      final r = computeKeyboardRange(KeyboardRangeMode.auto, const [21]);
      expect(r.low, 21);
      expect(r.low, greaterThanOrEqualTo(kPianoLowest));
    });

    test('clamps to the highest piano key (C8)', () {
      final r = computeKeyboardRange(KeyboardRangeMode.auto, const [108]);
      expect(r.high, 108);
      expect(r.high, lessThanOrEqualTo(kPianoHighest));
      expect(r.low, lessThanOrEqualTo(108));
    });

    test('full A0..C8 piece stays within bounds and covers everything', () {
      final r = computeKeyboardRange(KeyboardRangeMode.auto, const [21, 108]);
      expect(r.low, 21);
      expect(r.high, 108);
    });
  });

  group('computeKeyboardRange — presets', () {
    test('25-key preset with no notes uses its anchor window', () {
      final r = computeKeyboardRange(KeyboardRangeMode.keys25, const []);
      expect(r.low, 48); // C3
      expect(r.high, 72); // C5
    });

    test('88-key preset is the full keyboard regardless of notes', () {
      final r = computeKeyboardRange(KeyboardRangeMode.keys88, const [60]);
      expect(r.low, kPianoLowest);
      expect(r.high, kPianoHighest);
    });

    test('window shifts up to include a high-register piece', () {
      // 49-key preset (span 48, anchor C2..C6) with notes up at C7 (96).
      final r = computeKeyboardRange(KeyboardRangeMode.keys49, const [72, 96]);
      expect(r.low, lessThanOrEqualTo(72));
      expect(r.high, greaterThanOrEqualTo(96));
      expect(r.high, lessThanOrEqualTo(kPianoHighest));
    });

    test('widens when the piece is wider than the preset', () {
      // 25-key preset (span 24) but the piece spans 40 semitones.
      final r = computeKeyboardRange(KeyboardRangeMode.keys25, const [50, 90]);
      expect(r.low, lessThanOrEqualTo(50));
      expect(r.high, greaterThanOrEqualTo(90));
    });

    test('stays clamped to the 88-key bounds', () {
      final r = computeKeyboardRange(KeyboardRangeMode.keys76, const [21, 108]);
      expect(r.low, greaterThanOrEqualTo(kPianoLowest));
      expect(r.high, lessThanOrEqualTo(kPianoHighest));
    });
  });

  group('metadata helpers', () {
    test('presetKeyCount maps each mode', () {
      expect(presetKeyCount(KeyboardRangeMode.auto), isNull);
      expect(presetKeyCount(KeyboardRangeMode.keys25), 25);
      expect(presetKeyCount(KeyboardRangeMode.keys37), 37);
      expect(presetKeyCount(KeyboardRangeMode.keys49), 49);
      expect(presetKeyCount(KeyboardRangeMode.keys61), 61);
      expect(presetKeyCount(KeyboardRangeMode.keys76), 76);
      expect(presetKeyCount(KeyboardRangeMode.keys88), 88);
    });

    test('label is set for every mode', () {
      expect(KeyboardRangeMode.auto.label, 'Auto');
      expect(KeyboardRangeMode.keys25.label, '25');
      expect(KeyboardRangeMode.keys88.label, '88');
      for (final m in KeyboardRangeMode.values) {
        expect(m.label, isNotEmpty);
      }
    });
  });
}
