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
import 'package:music/courses/lesson_pitch.dart';
import 'package:music/state/note_label.dart';

void main() {
  group('LessonPitch.parse', () {
    test('parses naturals, sharps and flats with the right MIDI pitch', () {
      expect(LessonPitch.parse('C4')!.midi, 60);
      expect(LessonPitch.parse('A0')!.midi, 21);
      expect(LessonPitch.parse('C8')!.midi, 108);
      expect(LessonPitch.parse('F#4')!.midi, 66);
      expect(LessonPitch.parse('Bb3')!.midi, 58);
      expect(LessonPitch.parse('g4')!.midi, 67); // case-insensitive letter
      expect(LessonPitch.parse(' E5 ')!.midi, 76); // tolerant of spacing
    });

    test('preserves the written spelling, not just the sounding pitch', () {
      final dFlat = LessonPitch.parse('Db4')!;
      final cSharp = LessonPitch.parse('C#4')!;
      expect(dFlat.midi, cSharp.midi); // enharmonic — same key…
      expect(dFlat.step, 1); // …but D♭ is written on the D degree
      expect(cSharp.step, 0);
      expect(dFlat.name, const NoteName(1, -1));
      expect(cSharp.name, const NoteName(0, 1));
    });

    test('parses double alterations in both notations', () {
      expect(LessonPitch.parse('F##4')!.midi, 67);
      expect(LessonPitch.parse('Fx4')!.midi, 67);
      expect(LessonPitch.parse('Ebb3')!.midi, 50);
    });

    test('declines malformed input rather than throwing', () {
      expect(LessonPitch.parse(''), isNull);
      expect(LessonPitch.parse('H4'), isNull); // no H in this convention
      expect(LessonPitch.parse('C'), isNull); // octave required
      expect(LessonPitch.parse('C#'), isNull);
      expect(LessonPitch.parse('4C'), isNull);
      expect(LessonPitch.parse('C44'), isNull);
      expect(LessonPitch.parse('do4'), isNull); // letters only in manifests
    });

    test('declines pitches that fall off the 88-key piano', () {
      expect(LessonPitch.parse('G0'), isNull); // below A0
      expect(LessonPitch.parse('D8'), isNull); // above C8
      expect(LessonPitch.parse('Ab0'), isNull); // A♭0 sounds below A0
      expect(LessonPitch.parse('A0'), isNotNull); // the exact edges stay in
      expect(LessonPitch.parse('C8'), isNotNull);
    });
  });

  group('staffStep', () {
    test('places treble-staff anchors on their lines and spaces', () {
      // Bottom line E4 = 0; each half staff-space is one step.
      expect(LessonPitch.parse('E4')!.staffStep(LessonClef.treble), 0);
      expect(LessonPitch.parse('F4')!.staffStep(LessonClef.treble), 1);
      expect(LessonPitch.parse('B4')!.staffStep(LessonClef.treble), 4);
      expect(LessonPitch.parse('F5')!.staffStep(LessonClef.treble), 8);
      // Middle C sits on its ledger line below the treble staff.
      expect(LessonPitch.parse('C4')!.staffStep(LessonClef.treble), -2);
    });

    test('places bass-staff anchors on their lines and spaces', () {
      // Bottom line G2 = 0; the F line (the clef line) is step 6.
      expect(LessonPitch.parse('G2')!.staffStep(LessonClef.bass), 0);
      expect(LessonPitch.parse('F3')!.staffStep(LessonClef.bass), 6);
      expect(LessonPitch.parse('A3')!.staffStep(LessonClef.bass), 8);
      // Middle C sits on its ledger line above the bass staff.
      expect(LessonPitch.parse('C4')!.staffStep(LessonClef.bass), 10);
    });

    test('an alteration never moves the staff position', () {
      expect(
        LessonPitch.parse('F#4')!.staffStep(LessonClef.treble),
        LessonPitch.parse('F4')!.staffStep(LessonClef.treble),
      );
    });
  });

  test('nearestClef splits around the grand-staff middle', () {
    expect(LessonPitch.parse('G4')!.nearestClef, LessonClef.treble);
    expect(LessonPitch.parse('C4')!.nearestClef, LessonClef.treble);
    expect(LessonPitch.parse('A3')!.nearestClef, LessonClef.bass);
    expect(LessonPitch.parse('F2')!.nearestClef, LessonClef.bass);
  });

  test('keySignatureAlter follows the fifths order', () {
    expect(keySignatureAlter(0, 3), 0); // C major: F stays natural
    expect(keySignatureAlter(1, 3), 1); // G major: F♯
    expect(keySignatureAlter(1, 0), 0); // …but not C
    expect(keySignatureAlter(2, 0), 1); // D major adds C♯
    expect(keySignatureAlter(-1, 6), -1); // F major: B♭
    expect(keySignatureAlter(-2, 2), -1); // B♭ major adds E♭
    expect(keySignatureAlter(-1, 2), 0);
  });

  test('round-trips through toString', () {
    for (final s in ['C4', 'F#3', 'Bb4', 'Ebb3', 'A0', 'C8']) {
      expect(LessonPitch.parse(s).toString(), s);
    }
  });
}
