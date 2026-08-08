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
import 'package:music/state/note_label.dart';
import 'package:music/state/player_data.dart';

/// Written diatonic degree of a natural pitch name in octave 4 (`octave*7+step`).
const int _c4 = 28; // C4 = MIDI 60
const int _d4 = 29;
const int _e4 = 30;
const int _f4 = 31; // F4 = MIDI 65
const int _g4 = 32;
const int _a4 = 33;
const int _b4 = 34;

String _en(TimedNote n, {int keyFifths = 0}) =>
    noteLabel(n, solfege: false, frenchRe: false, keyFifths: keyFifths);

TimedNote _note(int pitch, {int? diatonic, String? noteType, int dots = 0}) =>
    TimedNote(
      pitch: pitch,
      startMs: 0,
      durationMs: 500,
      diatonic: diatonic,
      noteType: noteType,
      dots: dots,
    );

void main() {
  group('naturalPitchOf', () {
    test('maps the written degree to the MIDI natural', () {
      expect(naturalPitchOf(_c4), 60);
      expect(naturalPitchOf(_d4), 62);
      expect(naturalPitchOf(_e4), 64);
      expect(naturalPitchOf(_f4), 65);
      expect(naturalPitchOf(_g4), 67);
      expect(naturalPitchOf(_a4), 69);
      expect(naturalPitchOf(_b4), 71);
    });

    test('stays correct below C0 (negative degrees floor, not truncate)', () {
      // Diatonic -7 is C(-1) = MIDI 0; -6 is D(-1) = MIDI 2.
      expect(naturalPitchOf(-7), 0);
      expect(naturalPitchOf(-6), 2);
    });
  });

  group('effective alteration', () {
    test('key-signature sharp is named even with no engraved accidental', () {
      // F♯4 written on the F degree, under one sharp: the score engraves no
      // accidental, but the note sounds 66. It must not be named "F".
      expect(_en(_note(66, diatonic: _f4), keyFifths: 1), 'F♯');
    });

    test('key-signature flat is named even with no engraved accidental', () {
      // B♭4 written on the B degree under two flats.
      expect(_en(_note(70, diatonic: _b4), keyFifths: -2), 'B♭');
    });

    test('engraved accidental is named', () {
      expect(_en(_note(61, diatonic: _c4)), 'C♯');
    });

    test('natural cancelling the key signature names the plain degree', () {
      // Under one sharp, an F natural sounds 65 on the F degree.
      expect(_en(_note(65, diatonic: _f4), keyFifths: 1), 'F');
    });

    test('double alterations are named', () {
      expect(_en(_note(67, diatonic: _f4)), 'F♯♯');
      expect(_en(_note(69, diatonic: _b4)), 'B♭♭');
    });

    test('written enharmonic spelling is preserved', () {
      // MIDI 61 written on the D degree is D♭, never C♯.
      expect(_en(_note(61, diatonic: _d4)), 'D♭');
      // …and the same pitch written on the C degree stays C♯.
      expect(_en(_note(61, diatonic: _c4)), 'C♯');
    });
  });

  group('locale conventions', () {
    TimedNote d = _note(62, diatonic: _d4);

    test('English uses letter names', () {
      expect(noteLabel(d, solfege: false, frenchRe: false), 'D');
      expect(
        noteLabel(_note(64, diatonic: _e4), solfege: false, frenchRe: false),
        'E',
      );
    });

    test('French uses solfège with Ré', () {
      expect(noteLabel(d, solfege: true, frenchRe: true), 'Ré');
    });

    test('Spanish and Italian use Re', () {
      expect(noteLabel(d, solfege: true, frenchRe: false), 'Re');
    });

    test('solfège names every degree', () {
      const degrees = [_c4, _d4, _e4, _f4, _g4, _a4, _b4];
      const expected = ['Do', 'Ré', 'Mi', 'Fa', 'Sol', 'La', 'Si'];
      for (var i = 0; i < degrees.length; i++) {
        final n = _note(naturalPitchOf(degrees[i]), diatonic: degrees[i]);
        expect(noteLabel(n, solfege: true, frenchRe: true), expected[i]);
      }
    });

    test('no name carries an octave index', () {
      for (var pitch = 21; pitch <= 108; pitch++) {
        for (final fifths in [-3, 0, 4]) {
          final label = _en(_note(pitch), keyFifths: fifths);
          expect(
            label,
            isNot(matches(RegExp(r'\d'))),
            reason: 'pitch $pitch named "$label" carries a digit',
          );
        }
      }
    });
  });

  group('fallback naming (no written spelling)', () {
    test('flat key spells black keys with flats', () {
      expect(_en(_note(61), keyFifths: -3), 'D♭');
      expect(_en(_note(63), keyFifths: -3), 'E♭');
      expect(_en(_note(66), keyFifths: -1), 'G♭');
      expect(_en(_note(68), keyFifths: -4), 'A♭');
      expect(_en(_note(70), keyFifths: -2), 'B♭');
    });

    test('sharp key spells black keys with sharps', () {
      expect(_en(_note(61), keyFifths: 2), 'C♯');
      expect(_en(_note(63), keyFifths: 2), 'D♯');
      expect(_en(_note(66), keyFifths: 1), 'F♯');
      expect(_en(_note(68), keyFifths: 3), 'G♯');
      expect(_en(_note(70), keyFifths: 5), 'A♯');
    });

    test('no key signature defaults to sharps', () {
      expect(_en(_note(61)), 'C♯');
    });

    test('white keys are named the same either way', () {
      for (final pitch in [60, 62, 64, 65, 67, 69, 71]) {
        expect(
          _en(_note(pitch), keyFifths: -4),
          _en(_note(pitch), keyFifths: 4),
        );
      }
    });

    test('never yields an empty label across the whole keyboard', () {
      for (var pitch = 21; pitch <= 108; pitch++) {
        for (final fifths in [-7, 0, 7]) {
          expect(_en(_note(pitch), keyFifths: fifths), isNotEmpty);
        }
      }
    });
  });

  group('keyName', () {
    // The reference table this function replaced, verbatim, so the migration out
    // of the pre-play modal is provably behaviour-preserving.
    const letters = {
      -7: 'C♭',
      -6: 'G♭',
      -5: 'D♭',
      -4: 'A♭',
      -3: 'E♭',
      -2: 'B♭',
      -1: 'F',
      0: 'C',
      1: 'G',
      2: 'D',
      3: 'A',
      4: 'E',
      5: 'B',
      6: 'F♯',
      7: 'C♯',
    };
    const solfege = {
      -7: 'Do♭',
      -6: 'Sol♭',
      -5: 'Ré♭',
      -4: 'La♭',
      -3: 'Mi♭',
      -2: 'Si♭',
      -1: 'Fa',
      0: 'Do',
      1: 'Sol',
      2: 'Ré',
      3: 'La',
      4: 'Mi',
      5: 'Si',
      6: 'Fa♯',
      7: 'Do♯',
    };

    test('letter names match the previous table', () {
      letters.forEach((fifths, name) {
        expect(keyName(fifths, solfege: false, frenchRe: false), name);
      });
    });

    test('French solfège names match the previous table', () {
      solfege.forEach((fifths, name) {
        expect(keyName(fifths, solfege: true, frenchRe: true), name);
      });
    });

    test('Spanish/Italian use Re', () {
      expect(keyName(2, solfege: true, frenchRe: false), 'Re');
      expect(keyName(-5, solfege: true, frenchRe: false), 'Re♭');
    });

    test('out-of-range fifths clamp instead of throwing', () {
      expect(keyName(99, solfege: false, frenchRe: false), 'C♯');
      expect(keyName(-99, solfege: false, frenchRe: false), 'C♭');
    });
  });

  group('octaveMarkerLabel', () {
    test('uses scientific pitch notation in English', () {
      expect(octaveMarkerLabel(60, solfege: false, frenchRe: false), 'C4');
      expect(octaveMarkerLabel(48, solfege: false, frenchRe: false), 'C3');
      expect(octaveMarkerLabel(21, solfege: false, frenchRe: false), 'A0');
    });

    test('follows the solfège convention so the keyboard and aid agree', () {
      expect(octaveMarkerLabel(60, solfege: true, frenchRe: true), 'Do4');
      expect(octaveMarkerLabel(48, solfege: true, frenchRe: true), 'Do3');
      expect(octaveMarkerLabel(62, solfege: true, frenchRe: true), 'Ré4');
      expect(octaveMarkerLabel(62, solfege: true, frenchRe: false), 'Re4');
    });
  });

  group('figureFor', () {
    test('notated figure is named and quantified in 4/4', () {
      expect(
        figureFor(noteType: 'quarter', beatType: 4),
        const FigureToken(NoteFigure.quarter, 0, 1),
      );
      expect(
        figureFor(noteType: 'half', beatType: 4),
        const FigureToken(NoteFigure.half, 0, 2),
      );
      expect(
        figureFor(noteType: 'whole', beatType: 4),
        const FigureToken(NoteFigure.whole, 0, 4),
      );
    });

    test('dotted half in 4/4 is three beats', () {
      expect(
        figureFor(noteType: 'half', dots: 1, beatType: 4),
        const FigureToken(NoteFigure.half, 1, 3),
      );
    });

    test('half beats are clean values', () {
      expect(figureFor(noteType: 'eighth', beatType: 4)?.beats, 0.5);
    });

    test(
      'a sixteenth in 4/4 does not resolve cleanly and is not quantified',
      () {
        final t = figureFor(noteType: '16th', beatType: 4);
        expect(t?.figure, NoteFigure.sixteenth);
        expect(t?.beats, isNull);
      },
    );

    test('compound meter reports notated beats', () {
      // A dotted quarter in 6/8 is three eighth-beats (not the one felt beat).
      expect(
        figureFor(noteType: 'quarter', dots: 1, beatType: 8),
        const FigureToken(NoteFigure.quarter, 1, 3),
      );
    });

    test('figure is inferred from duration when the type is missing', () {
      // 500 ms at a 500 ms beat in 4/4 = one quarter.
      expect(
        figureFor(beatType: 4, durationMs: 500, beatMs: 500),
        const FigureToken(NoteFigure.quarter, 0, 1),
      );
      // 1500 ms = a dotted half.
      expect(
        figureFor(beatType: 4, durationMs: 1500, beatMs: 500),
        const FigureToken(NoteFigure.half, 1, 3),
      );
    });

    test('a poorly matching duration names the figure but omits the count', () {
      // 620 ms against a 500 ms beat is 24% over a quarter — too far to trust.
      final t = figureFor(beatType: 4, durationMs: 620, beatMs: 500);
      expect(t?.figure, NoteFigure.quarter);
      expect(t?.dots, 0);
      expect(t?.beats, isNull);
    });

    test('returns null when nothing can be determined', () {
      expect(figureFor(beatType: 4), isNull);
      expect(figureFor(beatType: 4, durationMs: 500), isNull);
      expect(figureFor(noteType: 'quarter', beatType: 0), isNull);
      expect(figureFor(beatType: 4, durationMs: 0, beatMs: 500), isNull);
    });

    test('an unknown notated type falls through to inference', () {
      expect(
        figureFor(noteType: '64th', beatType: 4, durationMs: 500, beatMs: 500),
        const FigureToken(NoteFigure.quarter, 0, 1),
      );
    });

    test('every figure has a distinct glyph and dots append', () {
      final glyphs = <String>{};
      for (final f in NoteFigure.values) {
        final token = FigureToken(f, 0, null);
        expect(token.glyph, isNotEmpty);
        glyphs.add(token.glyph);
      }
      expect(glyphs.length, NoteFigure.values.length);
      const dotted = FigureToken(NoteFigure.half, 2, null);
      expect(dotted.glyphWithDots.length, dotted.glyph.length + 2);
    });

    test('dotMultiplier follows the halving rule', () {
      expect(dotMultiplier(0), 1.0);
      expect(dotMultiplier(1), 1.5);
      expect(dotMultiplier(2), 1.75);
    });
  });

  group('NoteName value semantics', () {
    test('equality and hashCode are by degree and alteration', () {
      expect(const NoteName(0, 1), const NoteName(0, 1));
      expect(const NoteName(0, 1).hashCode, const NoteName(0, 1).hashCode);
      expect(const NoteName(0, 1), isNot(const NoteName(1, -1)));
      expect(const NoteName(0, 1).toString(), contains('NoteName'));
    });

    test('alteration is clamped to the notatable range', () {
      // A wildly inconsistent pitch/spelling pair cannot produce a triple sharp.
      final n = noteNameFor(pitch: 70, diatonic: _c4);
      expect(n.alter, 2);
    });

    test('FigureToken value semantics', () {
      expect(
        const FigureToken(NoteFigure.half, 1, 3),
        const FigureToken(NoteFigure.half, 1, 3),
      );
      expect(
        const FigureToken(NoteFigure.half, 1, 3).hashCode,
        const FigureToken(NoteFigure.half, 1, 3).hashCode,
      );
      expect(
        const FigureToken(NoteFigure.half, 1, 3),
        isNot(const FigureToken(NoteFigure.half, 0, 2)),
      );
      expect(
        const FigureToken(NoteFigure.half, 1, 3).toString(),
        contains('FigureToken'),
      );
    });
  });
}
