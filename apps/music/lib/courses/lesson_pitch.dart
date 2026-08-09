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

/// The written pitch a course exercise talks about (change:
/// add-notation-courses, schema v2).
///
/// A manifest names pitches in compact scientific notation — `"C4"`, `"F#3"`,
/// `"Bb4"` — because course content is authored by hand and a written spelling
/// carries what a MIDI number cannot: the staff degree (a written D♭ is not a
/// C♯ on the staff). Everything else is derived: the sounding MIDI pitch, the
/// staff position under a clef, and the learner-facing name (through the app's
/// single naming module, `note_label.dart`).
library;

import '../state/note_label.dart';

/// Which clef a lesson staff draws. A closed set — courses teach the two piano
/// clefs; a C clef would be a schema evolution, not a content tweak.
enum LessonClef { treble, bass }

/// Diatonic index (`octave * 7 + step`, C4 = 28) of the **bottom staff line**
/// per clef: E4 for treble, G2 for bass.
const int _trebleBottomDiatonic = 4 * 7 + 2; // E4 = 30
const int _bassBottomDiatonic = 2 * 7 + 4; // G2 = 18

/// Degrees altered by sharps (F C G D A E B) and flats (B E A D G C F), in
/// fifths order — the order key signatures accumulate.
const List<int> _sharpDegrees = [3, 0, 4, 1, 5, 2, 6];
const List<int> _flatDegrees = [6, 2, 5, 1, 4, 0, 3];

/// The alteration a key signature of [fifths] imposes on written [degree]
/// (0 = C … 6 = B): +1, −1 or 0. What lets a lesson staff *omit* the engraved
/// accidental the armure already carries — engraving convention, and the whole
/// point of teaching key signatures.
int keySignatureAlter(int fifths, int degree) {
  final f = fifths.clamp(-7, 7);
  if (f > 0) return _sharpDegrees.take(f).contains(degree) ? 1 : 0;
  if (f < 0) return _flatDegrees.take(-f).contains(degree) ? -1 : 0;
  return 0;
}

/// A written pitch: degree 0–6 (C–B), alteration in semitones (−2…+2) and
/// scientific octave (C4 = middle C = MIDI 60). Immutable value type.
class LessonPitch {
  /// Written degree, 0 = C/Do … 6 = B/Si.
  final int step;

  /// Alteration in semitones: −2 = double flat … +2 = double sharp.
  final int alter;

  /// Scientific octave: C4 is middle C.
  final int octave;

  const LessonPitch(this.step, this.alter, this.octave);

  /// Parses compact scientific notation — a letter `A`–`G` (any case), an
  /// optional alteration (`#`, `x`, `##` for sharps; `b`, `bb` for flats), and
  /// an octave `0`–`8` (optionally negative). Returns null on anything else, so
  /// a malformed manifest degrades instead of throwing.
  static LessonPitch? parse(String source) {
    final m = _spn.firstMatch(source.trim());
    if (m == null) return null;
    const letters = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
    final step = letters.indexOf(m.group(1)!.toUpperCase());
    final alter = switch (m.group(2) ?? '') {
      '' => 0,
      '#' => 1,
      '##' || 'x' => 2,
      'b' => -1,
      'bb' => -2,
      _ => null,
    };
    if (alter == null) return null;
    final octave = int.parse(m.group(3)!);
    final p = LessonPitch(step, alter, octave);
    // Keep the sounding pitch on the piano; a spelling off the keyboard is a
    // content error, declined like any other malformed value.
    return p.midi < 21 || p.midi > 108 ? null : p;
  }

  static final RegExp _spn = RegExp(r'^([A-Ga-g])(##|bb|[#xb])?(-?\d)$');

  /// Diatonic staff index (`octave * 7 + step`, C4 = 28) — the scale
  /// `note_label.dart` and the notation painters already speak.
  int get diatonic => octave * 7 + step;

  /// The sounding MIDI pitch (the key that validates the exercise).
  int get midi => naturalPitchOf(diatonic) + alter;

  /// The learner-facing name, through the app's single naming module.
  NoteName get name => NoteName(step, alter);

  /// Position on a [clef]'s staff, in **half staff-spaces above the bottom
  /// line** (0 = on the bottom line, 1 = the space above it, negative = below
  /// the staff — ledger-line territory).
  int staffStep(LessonClef clef) =>
      diatonic -
      (clef == LessonClef.treble ? _trebleBottomDiatonic : _bassBottomDiatonic);

  /// The clef whose staff this pitch sits nearest — how a course exercise
  /// picks a sensible default staff for a drill note. The split is middle C
  /// (the grand staff's seam): C4 and above read in treble, below in bass.
  LessonClef get nearestClef =>
      diatonic >= 4 * 7 ? LessonClef.treble : LessonClef.bass;

  /// The **natural** pitch sitting at [staffStep] half staff-spaces above a
  /// [clef]'s bottom line — the inverse of [LessonPitch.staffStep], used when a
  /// learner taps a staff position (positions name naturals; alterations are a
  /// separate concept).
  static LessonPitch forStaffStep(int staffStep, LessonClef clef) {
    final diatonic =
        staffStep +
        (clef == LessonClef.treble
            ? _trebleBottomDiatonic
            : _bassBottomDiatonic);
    final octave = (diatonic / 7).floor();
    return LessonPitch(diatonic - octave * 7, 0, octave);
  }

  @override
  bool operator ==(Object other) =>
      other is LessonPitch &&
      other.step == step &&
      other.alter == alter &&
      other.octave == octave;

  @override
  int get hashCode => Object.hash(step, alter, octave);

  @override
  String toString() {
    const letters = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
    final alt = switch (alter) {
      -2 => 'bb',
      -1 => 'b',
      1 => '#',
      2 => '##',
      _ => '',
    };
    return '${letters[step]}$alt$octave';
  }
}
