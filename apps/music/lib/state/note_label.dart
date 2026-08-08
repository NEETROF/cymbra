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

/// Naming a pitch, a key signature and a rhythmic figure for a learner.
///
/// The single naming implementation in the app: the reading aid and the pre-play
/// modal's key-signature chip both go through here. Deliberately pure (no
/// `BuildContext`, no `AppLocalizations`) so it is host-testable — the locale is
/// passed in as two flags, and the rhythmic figure comes back as a *token* the
/// widget renders into localized prose.
library;

import 'dart:math' as math;

import '../painters/smufl.dart';
import 'player_data.dart' show TimedNote;

/// Semitones above the octave's C for each written degree (C D E F G A B).
const List<int> _degreeSemitones = [0, 2, 4, 5, 7, 9, 11];

/// Letter names per written degree.
const List<String> _letters = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];

/// Solfège names per written degree. Index 1 (`re`) is filled in per language —
/// French writes `Ré`, Spanish and Italian write `Re`.
const List<String> _solfege = ['Do', '', 'Mi', 'Fa', 'Sol', 'La', 'Si'];

/// Written degree of each major tonic, indexed by `fifths % 7` — the circle of
/// fifths C G D A E B F. Paired with a flat/sharp shift for the outer keys.
const List<int> _fifthsDegrees = [0, 4, 1, 5, 2, 6, 3];

/// Written degree + alteration of each pitch class when spelling with sharps.
const List<(int, int)> _sharpSpelling = [
  (0, 0), // C
  (0, 1), // C♯
  (1, 0), // D
  (1, 1), // D♯
  (2, 0), // E
  (3, 0), // F
  (3, 1), // F♯
  (4, 0), // G
  (4, 1), // G♯
  (5, 0), // A
  (5, 1), // A♯
  (6, 0), // B
];

/// Written degree + alteration of each pitch class when spelling with flats.
const List<(int, int)> _flatSpelling = [
  (0, 0), // C
  (1, -1), // D♭
  (1, 0), // D
  (2, -1), // E♭
  (2, 0), // E
  (3, 0), // F
  (4, -1), // G♭
  (4, 0), // G
  (5, -1), // A♭
  (5, 0), // A
  (6, -1), // B♭
  (6, 0), // B
];

/// The MIDI pitch of the **natural** of a written staff degree.
///
/// [diatonic] is the `octave * 7 + step` scale carried by [TimedNote.diatonic]
/// (C=0…B=6, octave from `pitch ~/ 12 - 1`, so MIDI 60 = C4 = diatonic 28).
/// Uses floor division so degrees below C0 (negative [diatonic]) stay correct.
int naturalPitchOf(int diatonic) {
  final octave = (diatonic / 7).floor();
  final step = diatonic - octave * 7;
  return (octave + 1) * 12 + _degreeSemitones[step];
}

/// A note's written degree (0 = C/Do … 6 = B/Si) and the alteration in force on
/// it, in semitones (−2…+2). Alteration is *effective*: it covers the key
/// signature and any accidental, whether or not the score engraves a symbol.
class NoteName {
  /// Written degree, 0 = C/Do … 6 = B/Si.
  final int degree;

  /// Alteration in semitones: −2 = double flat … +2 = double sharp.
  final int alter;

  const NoteName(this.degree, this.alter);

  /// The alteration symbol, or the empty string for a natural degree.
  ///
  /// Doubles are written as a repeated symbol (`♯♯`) rather than the dedicated
  /// U+1D12A/U+1D12B code points, which most UI fonts do not carry.
  String get alterSymbol => switch (alter) {
    -2 => '♭♭',
    -1 => '♭',
    1 => '♯',
    2 => '♯♯',
    _ => '',
  };

  /// The name in the reader's convention: letters (C, D, E…) when [solfege] is
  /// false, solfège (Do, Ré/Re, Mi…) otherwise, with the alteration appended.
  /// Never carries an octave index — the keyboard already shows which key.
  String label({required bool solfege, required bool frenchRe}) {
    final base = solfege
        ? (degree == 1 ? (frenchRe ? 'Ré' : 'Re') : _solfege[degree])
        : _letters[degree];
    return '$base$alterSymbol';
  }

  @override
  bool operator ==(Object other) =>
      other is NoteName && other.degree == degree && other.alter == alter;

  @override
  int get hashCode => Object.hash(degree, alter);

  @override
  String toString() => 'NoteName($degree, $alter)';
}

/// The name of a sounding [pitch], spelled from its written degree when the
/// notation carries one.
///
/// When [diatonic] is known the alteration is exact: `pitch` is the pitch the
/// parser already resolved (key signature and in-measure accidentals applied)
/// and `diatonic` is the unresolved written degree, so their difference *is* the
/// alteration in force. This is what names an F♯ carried by the key signature
/// alone as F♯ rather than F, and what preserves an enharmonic spelling (a
/// written D♭ never comes back as C♯).
///
/// When [diatonic] is null — the built-in demo score and any MIDI-only source —
/// the name is derived from [pitch] alone, spelled with flats under a flat key
/// signature ([keyFifths] < 0) and with sharps otherwise. That cannot recover a
/// spelling the source never had, but it never contradicts the sounding pitch.
NoteName noteNameFor({required int pitch, int? diatonic, int keyFifths = 0}) {
  if (diatonic != null) {
    final alter = (pitch - naturalPitchOf(diatonic)).clamp(-2, 2);
    final octave = (diatonic / 7).floor();
    return NoteName(diatonic - octave * 7, alter);
  }
  final table = keyFifths < 0 ? _flatSpelling : _sharpSpelling;
  final (degree, alter) = table[pitch % 12];
  return NoteName(degree, alter);
}

/// The reading-aid label for [note] — its written degree plus the alteration in
/// force, in the reader's naming convention. [keyFifths] is only consulted for
/// notes that carry no written spelling.
String noteLabel(
  TimedNote note, {
  required bool solfege,
  required bool frenchRe,
  int keyFifths = 0,
}) => noteNameFor(
  pitch: note.pitch,
  diatonic: note.diatonic,
  keyFifths: keyFifths,
).label(solfege: solfege, frenchRe: frenchRe);

/// The tonic name of a key from its number of sharps (+) / flats (−).
///
/// Mode is unknown from the key signature alone, so this is the conventional
/// major-tonic reading. [fifths] outside the notatable −7…7 range is clamped.
String keyName(int fifths, {required bool solfege, required bool frenchRe}) {
  final f = fifths.clamp(-7, 7);
  final degree = _fifthsDegrees[f % 7];
  final alter = ((f + 1) / 7).floor();
  return NoteName(degree, alter).label(solfege: solfege, frenchRe: frenchRe);
}

/// The keyboard's octave marker for [pitch] — the note name followed by its
/// octave in scientific pitch notation (MIDI 60 = C4 = `Do4`).
///
/// Unlike the reading aid, these anchors *do* carry the octave: their whole job
/// is to say *which* C you are looking at. They go through the same naming
/// convention so the keyboard cannot say "C4" while the aid says "Do♯".
String octaveMarkerLabel(
  int pitch, {
  required bool solfege,
  required bool frenchRe,
}) {
  final name = noteNameFor(
    pitch: pitch,
  ).label(solfege: solfege, frenchRe: frenchRe);
  return '$name${pitch ~/ 12 - 1}';
}

/// A rhythmic figure, from the double whole (breve) down to the thirty-second.
enum NoteFigure {
  doubleWhole,
  whole,
  half,
  quarter,
  eighth,
  sixteenth,
  thirtySecond,
}

/// Fraction of a whole note each bare figure lasts.
const Map<NoteFigure, double> _figureFractions = {
  NoteFigure.doubleWhole: 2.0,
  NoteFigure.whole: 1.0,
  NoteFigure.half: 0.5,
  NoteFigure.quarter: 0.25,
  NoteFigure.eighth: 0.125,
  NoteFigure.sixteenth: 0.0625,
  NoteFigure.thirtySecond: 0.03125,
};

/// MusicXML `<type>` tokens mapped to the figures we name. Anything else (64th
/// and below, `long`) falls through to duration-based inference.
const Map<String, NoteFigure> _typeTokens = {
  'breve': NoteFigure.doubleWhole,
  'whole': NoteFigure.whole,
  'half': NoteFigure.half,
  'quarter': NoteFigure.quarter,
  'eighth': NoteFigure.eighth,
  '16th': NoteFigure.sixteenth,
  '32nd': NoteFigure.thirtySecond,
};

/// The multiplier augmentation [dots] apply to a figure: 1 dot = ×1.5, 2 = ×1.75.
double dotMultiplier(int dots) =>
    2 - math.pow(0.5, dots.clamp(0, 4)).toDouble();

/// A named rhythmic figure: the figure itself, its augmentation dots, and how
/// long it lasts in beats of the current time signature.
///
/// [beats] is null when the duration does not land on a clean value (halves or
/// better) — the aid then names the figure without quantifying it rather than
/// printing a misleading number.
class FigureToken {
  final NoteFigure figure;
  final int dots;
  final double? beats;

  const FigureToken(this.figure, this.dots, this.beats);

  /// The complete Bravura note glyph (head, stem and flag) for this figure, to
  /// be rendered in [Smufl.fontFamily].
  String get glyph => switch (figure) {
    NoteFigure.doubleWhole => Smufl.noteDoubleWhole,
    NoteFigure.whole => Smufl.noteWhole,
    NoteFigure.half => Smufl.noteHalfUp,
    NoteFigure.quarter => Smufl.noteQuarterUp,
    NoteFigure.eighth => Smufl.note8thUp,
    NoteFigure.sixteenth => Smufl.note16thUp,
    NoteFigure.thirtySecond => Smufl.note32ndUp,
  };

  /// The glyph followed by one augmentation dot per [dots].
  String get glyphWithDots => glyph + Smufl.augmentationDot * dots;

  @override
  bool operator ==(Object other) =>
      other is FigureToken &&
      other.figure == figure &&
      other.dots == dots &&
      other.beats == beats;

  @override
  int get hashCode => Object.hash(figure, dots, beats);

  @override
  String toString() => 'FigureToken($figure, dots: $dots, beats: $beats)';
}

/// Rounds [beats] to a clean value (a multiple of a half beat), or null when it
/// is not one — the caller then shows the figure name without a beat count.
double? _cleanBeats(double beats) {
  if (!beats.isFinite || beats <= 0 || beats > 64) return null;
  final halves = beats * 2;
  final rounded = halves.roundToDouble();
  if ((halves - rounded).abs() > 1e-6) return null;
  return rounded / 2;
}

/// The rhythmic figure of a note, named and quantified.
///
/// Prefers the notated [noteType] + [dots]. When the notation carries no type
/// (the demo score and MIDI-only sources) the figure is inferred from
/// [durationMs] against [beatMs] — the duration of one beat at that point in the
/// piece. Returns null when nothing can be determined.
///
/// The beat count is expressed in beats of the notated [beatType]: a dotted half
/// in 4/4 is three beats. Compound meters are reported the same way (a dotted
/// quarter in 6/8 is three eighth-beats, not the one beat a musician counts) —
/// a known limitation of naming from notation alone.
FigureToken? figureFor({
  String? noteType,
  int dots = 0,
  required int beatType,
  double durationMs = 0,
  double? beatMs,
}) {
  if (beatType <= 0) return null;
  final safeDots = dots.clamp(0, 4);

  final notated = noteType == null ? null : _typeTokens[noteType];
  if (notated != null) {
    final fraction = _figureFractions[notated]! * dotMultiplier(safeDots);
    return FigureToken(notated, safeDots, _cleanBeats(fraction * beatType));
  }

  // No usable notated type: infer from the sounding duration.
  if (beatMs == null || beatMs <= 0 || durationMs <= 0) return null;
  final wholeMs = beatMs * beatType;
  final fraction = durationMs / wholeMs;
  if (fraction <= 0) return null;

  NoteFigure? bestFigure;
  var bestDots = 0;
  var bestError = double.infinity;
  for (final entry in _figureFractions.entries) {
    for (var d = 0; d <= 2; d++) {
      final candidate = entry.value * dotMultiplier(d);
      final error = (math.log(fraction) - math.log(candidate)).abs();
      if (error < bestError) {
        bestError = error;
        bestFigure = entry.key;
        bestDots = d;
      }
    }
  }
  if (bestFigure == null) return null;
  // A poor match (>10% off) still names the nearest figure, but refuses to
  // quantify it — a wrong beat count is worse than none.
  final poor = bestError > 0.0953; // ln(1.1)
  final matched = _figureFractions[bestFigure]! * dotMultiplier(bestDots);
  return FigureToken(
    bestFigure,
    poor ? 0 : bestDots,
    poor ? null : _cleanBeats(matched * beatType),
  );
}
