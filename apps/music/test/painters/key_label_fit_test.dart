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
import 'package:music/painters/piano_keyboard_painter.dart';
import 'package:music/painters/piano_layout.dart';

/// Width-per-font-unit of a typical two-to-three character label ("Do♯"): each
/// glyph runs a bit over half an em.
const double _wpfu = 1.7;

void main() {
  group('fitKeyLabel', () {
    test('stays upright when the label fits across the key', () {
      // An auto-fit keyboard: ~40 px per white key, plenty of room.
      final fit = fitKeyLabel(
        widthPerFontUnit: _wpfu,
        keyWidth: 40,
        availableHeight: 36,
      );
      expect(fit, isNotNull);
      expect(fit!.vertical, isFalse);
    });

    test('turns on its side when the key is too narrow to read across', () {
      // 88 keys on a phone: ~15 px per white key.
      final fit = fitKeyLabel(
        widthPerFontUnit: _wpfu,
        keyWidth: 15,
        availableHeight: 36,
      );
      expect(fit, isNotNull);
      expect(fit!.vertical, isTrue);
    });

    test('never exceeds the key box, upright or sideways', () {
      for (final keyWidth in [9.0, 12.0, 15.0, 22.0, 40.0, 60.0]) {
        for (final height in [24.0, 30.0, 36.0, 60.0]) {
          final fit = fitKeyLabel(
            widthPerFontUnit: _wpfu,
            keyWidth: keyWidth,
            availableHeight: height,
          );
          if (fit == null) continue;
          final textLength = fit.fontSize * _wpfu;
          final across = fit.vertical ? fit.fontSize : textLength;
          final along = fit.vertical ? textLength : fit.fontSize;
          expect(
            across,
            lessThanOrEqualTo(keyWidth - 2 + 1e-9),
            reason: 'overflows the key width at $keyWidth × $height',
          );
          expect(
            along,
            lessThanOrEqualTo(height - 2 + 1e-9),
            reason: 'overflows the key height at $keyWidth × $height',
          );
        }
      }
    });

    test('drops the label rather than drawing it illegibly small', () {
      // A hair-thin key with almost no band: nothing legible fits.
      expect(
        fitKeyLabel(widthPerFontUnit: _wpfu, keyWidth: 4, availableHeight: 6),
        isNull,
      );
    });

    test('caps the font size on a very wide key', () {
      final fit = fitKeyLabel(
        widthPerFontUnit: _wpfu,
        keyWidth: 400,
        availableHeight: 300,
        maxFontSize: 15,
      );
      expect(fit!.fontSize, 15);
      expect(fit.vertical, isFalse);
    });

    test('rejects degenerate boxes', () {
      expect(
        fitKeyLabel(widthPerFontUnit: 0, keyWidth: 40, availableHeight: 40),
        isNull,
      );
      expect(
        fitKeyLabel(widthPerFontUnit: _wpfu, keyWidth: 0, availableHeight: 40),
        isNull,
      );
      expect(
        fitKeyLabel(widthPerFontUnit: _wpfu, keyWidth: 40, availableHeight: 0),
        isNull,
      );
      expect(
        fitKeyLabel(widthPerFontUnit: _wpfu, keyWidth: 1, availableHeight: 1),
        isNull,
      );
    });

    test('a real 88-key phone keyboard still gets a label on every key', () {
      // The worst realistic case: the full 88 keys across a 780 px phone in
      // landscape, with the short phone keyboard band.
      const layout = PianoLayout(width: 780, lowPitch: 21, highPitch: 108);
      const bandHeight = 78 * 0.38; // white key band below the black keys
      final white = fitKeyLabel(
        widthPerFontUnit: _wpfu,
        keyWidth: layout.whiteWidth,
        availableHeight: bandHeight,
      );
      final black = fitKeyLabel(
        widthPerFontUnit: _wpfu,
        keyWidth: layout.blackWidth,
        availableHeight: 78 * 0.62 * 0.55,
      );
      expect(white, isNotNull, reason: 'white keys must stay labelled');
      expect(white!.vertical, isTrue);
      expect(black, isNotNull, reason: 'black keys must stay labelled');
    });

    test('KeyLabelFit value semantics', () {
      expect(
        const KeyLabelFit(12, vertical: true),
        const KeyLabelFit(12, vertical: true),
      );
      expect(
        const KeyLabelFit(12, vertical: true).hashCode,
        const KeyLabelFit(12, vertical: true).hashCode,
      );
      expect(
        const KeyLabelFit(12, vertical: true),
        isNot(const KeyLabelFit(12, vertical: false)),
      );
      expect(
        const KeyLabelFit(12, vertical: true).toString(),
        contains('KeyLabelFit'),
      );
    });
  });
}
