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

import 'dart:ui' show Locale;

import '../l10n/gen/app_localizations.dart';
import '../painters/staff_hit_index.dart';

/// A localized explanation for a staff symbol: a short [title] and a one- or
/// two-sentence [body], both ready to render in a bubble or the glossary
/// (change: add-notation-help).
typedef NotationHelp = ({String title, String body});

/// Whether note names read as solfège (Do, Ré, Mi…) rather than letters
/// (C, D, E…), and whether "Ré" takes its French accent — derived from the app
/// language, matching the convention already used by the pre-play setup. English
/// uses letters; every other supported language uses solfège.
({bool solfege, bool frenchRe}) notationNameStyle(Locale locale) => (
  solfege: locale.languageCode != 'en',
  frenchRe: locale.languageCode == 'fr',
);

/// Names a MIDI pitch for a beginner: a letter (or solfège) plus its octave,
/// spelling black keys with a sharp (e.g. `C♯4` / `Do♯4`). The exact enharmonic
/// spelling is not needed here — the goal is to let a novice find the key.
String notationPitchName(
  int midi, {
  required bool solfege,
  required bool frenchRe,
}) {
  const letters = [
    'C',
    'C♯',
    'D',
    'D♯',
    'E',
    'F',
    'F♯',
    'G',
    'G♯',
    'A',
    'A♯',
    'B',
  ];
  final re = frenchRe ? 'Ré' : 'Re';
  final solfege0 = [
    'Do',
    'Do♯',
    re,
    '$re♯',
    'Mi',
    'Fa',
    'Fa♯',
    'Sol',
    'Sol♯',
    'La',
    'La♯',
    'Si',
  ];
  final pc = ((midi % 12) + 12) % 12;
  final octave = (midi ~/ 12) - 1;
  return '${(solfege ? solfege0 : letters)[pc]}$octave';
}

/// The tonic a key signature points to, from its number of sharps (+) / flats
/// (−), as a major-key reading (mode is unknown from the signature alone).
String notationKeyTonic(
  int fifths, {
  required bool solfege,
  required bool frenchRe,
}) {
  if (!solfege) {
    return switch (fifths) {
      -7 => 'C♭',
      -6 => 'G♭',
      -5 => 'D♭',
      -4 => 'A♭',
      -3 => 'E♭',
      -2 => 'B♭',
      -1 => 'F',
      1 => 'G',
      2 => 'D',
      3 => 'A',
      4 => 'E',
      5 => 'B',
      6 => 'F♯',
      7 => 'C♯',
      _ => 'C',
    };
  }
  final re = frenchRe ? 'Ré' : 'Re';
  return switch (fifths) {
    -7 => 'Do♭',
    -6 => 'Sol♭',
    -5 => '$re♭',
    -4 => 'La♭',
    -3 => 'Mi♭',
    -2 => 'Si♭',
    -1 => 'Fa',
    1 => 'Sol',
    2 => re,
    3 => 'La',
    4 => 'Mi',
    5 => 'Si',
    6 => 'Fa♯',
    7 => 'Do♯',
    _ => 'Do',
  };
}

/// The note the time-signature bottom number stands for (1 = whole … 16 =
/// sixteenth), localized, with its symbol where it renders reliably.
String notationBeatUnitName(AppLocalizations l10n, int beatType) =>
    switch (beatType) {
      1 => l10n.notationBeatUnitWhole,
      2 => l10n.notationBeatUnitHalf,
      4 => l10n.notationBeatUnitQuarter,
      8 => l10n.notationBeatUnitEighth,
      16 => l10n.notationBeatUnitSixteenth,
      _ => l10n.notationBeatUnitOther,
    };

/// The localized rest name for a note-value type, or null for the unknown/
/// generic case (MIDI-only sources carry no note type).
String? notationRestTitle(AppLocalizations l10n, String? noteType) =>
    switch (noteType) {
      'whole' => l10n.notationRestWhole,
      'half' => l10n.notationRestHalf,
      'quarter' => l10n.notationRestQuarter,
      'eighth' => l10n.notationRestEighth,
      '16th' => l10n.notationRestSixteenth,
      _ => null,
    };

/// A localized "how long it lasts" label in beats (quarter = one beat, the
/// common simple-meter reading), for a note/rest value plus its dots. Returns
/// null when the type is unknown or lands on an uncommon fraction, so the bubble
/// simply omits the duration rather than showing something confusing.
String? notationDurationBeats(
  AppLocalizations l10n,
  String? noteType,
  int dots,
) {
  final base = switch (noteType) {
    'whole' => 4.0,
    'half' => 2.0,
    'quarter' => 1.0,
    'eighth' => 0.5,
    '16th' => 0.25,
    _ => null,
  };
  if (base == null) return null;
  final beats = dots >= 1 ? base * 1.5 : base;
  return switch (beats) {
    4.0 => l10n.notationBeats4,
    3.0 => l10n.notationBeats3,
    2.0 => l10n.notationBeats2,
    1.5 => l10n.notationBeats1half,
    1.0 => l10n.notationBeats1,
    0.5 => l10n.notationBeatsHalf,
    0.25 => l10n.notationBeatsQuarter,
    _ => null,
  };
}

/// The localized help for a resolved staff [symbol]. Exhaustive over the
/// [SymbolDescriptor] union, so **every** symbol a painter can record has an
/// explanation — the compiler enforces it (a new variant fails to compile until
/// its help is written), which is the content side of the totality guarantee.
NotationHelp notationHelpFor(
  AppLocalizations l10n,
  SymbolDescriptor symbol, {
  required bool solfege,
  required bool frenchRe,
}) {
  switch (symbol) {
    case NoteSymbol(:final pitch, :final noteType, :final dots):
      final name = notationPitchName(
        pitch,
        solfege: solfege,
        frenchRe: frenchRe,
      );
      final beats = notationDurationBeats(l10n, noteType, dots);
      final body = beats == null
          ? l10n.notationHelpNoteBody
          : '${l10n.notationHelpNoteBody} ${l10n.notationHelpNoteDuration(name, beats)}';
      return (title: l10n.notationHelpNoteTitle(name), body: body);
    case RestSymbol(:final noteType):
      final beats = notationDurationBeats(l10n, noteType, 0);
      final body = beats == null
          ? l10n.notationHelpRestBody
          : '${l10n.notationHelpRestBody} ${l10n.notationHelpRestDuration(beats)}';
      return (
        title: notationRestTitle(l10n, noteType) ?? l10n.notationHelpRestTitle,
        body: body,
      );
    case AccidentalSymbol(:final token):
      return switch (token) {
        'flat' => (
          title: l10n.notationHelpFlatTitle,
          body: l10n.notationHelpFlatBody,
        ),
        'natural' => (
          title: l10n.notationHelpNaturalTitle,
          body: l10n.notationHelpNaturalBody,
        ),
        'double-sharp' || 'sharp-sharp' => (
          title: l10n.notationHelpDoubleSharpTitle,
          body: l10n.notationHelpDoubleSharpBody,
        ),
        'flat-flat' => (
          title: l10n.notationHelpDoubleFlatTitle,
          body: l10n.notationHelpDoubleFlatBody,
        ),
        _ => (
          title: l10n.notationHelpSharpTitle,
          body: l10n.notationHelpSharpBody,
        ),
      };
    case ClefSymbol(:final sign):
      return switch (sign) {
        'F' => (
          title: l10n.notationHelpBassClefTitle,
          body: l10n.notationHelpBassClefBody,
        ),
        'C' => (
          title: l10n.notationHelpCClefTitle,
          body: l10n.notationHelpCClefBody,
        ),
        _ => (
          title: l10n.notationHelpTrebleClefTitle,
          body: l10n.notationHelpTrebleClefBody,
        ),
      };
    case KeySignatureSymbol(:final fifths):
      return (
        title: l10n.notationHelpKeySignatureTitle,
        body: fifths == 0
            ? l10n.notationHelpKeySignatureNone
            : l10n.notationHelpKeySignatureBody(
                notationKeyTonic(fifths, solfege: solfege, frenchRe: frenchRe),
              ),
      );
    case TimeSignatureSymbol(:final beats, :final beatType):
      return (
        title: l10n.notationHelpTimeSignatureTitle,
        body: l10n.notationHelpTimeSignatureBody(
          beats,
          beatType,
          notationBeatUnitName(l10n, beatType),
        ),
      );
    case AugmentationDotSymbol():
      return (title: l10n.notationHelpDotTitle, body: l10n.notationHelpDotBody);
    case StemSymbol():
      return (
        title: l10n.notationHelpStemTitle,
        body: l10n.notationHelpStemBody,
      );
    case FlagSymbol():
      return (
        title: l10n.notationHelpFlagTitle,
        body: l10n.notationHelpFlagBody,
      );
    case BeamSymbol():
      return (
        title: l10n.notationHelpBeamTitle,
        body: l10n.notationHelpBeamBody,
      );
    case LedgerLineSymbol():
      return (
        title: l10n.notationHelpLedgerLineTitle,
        body: l10n.notationHelpLedgerLineBody,
      );
    case BarLineSymbol():
      return (
        title: l10n.notationHelpBarLineTitle,
        body: l10n.notationHelpBarLineBody,
      );
    case TieSymbol():
      return (title: l10n.notationHelpTieTitle, body: l10n.notationHelpTieBody);
    case SlurSymbol():
      return (
        title: l10n.notationHelpSlurTitle,
        body: l10n.notationHelpSlurBody,
      );
    case TupletSymbol(:final actual):
      return (
        title: l10n.notationHelpTupletTitle,
        body: l10n.notationHelpTupletBody(actual),
      );
    case BraceSymbol():
      return (
        title: l10n.notationHelpBraceTitle,
        body: l10n.notationHelpBraceBody,
      );
    case DynamicsSymbol():
      return (
        title: l10n.notationHelpDynamicsTitle,
        body: l10n.notationHelpDynamicsBody,
      );
  }
}

/// One glossary row: the symbol kind's help, plus the concrete descriptor used
/// to render an example.
typedef GlossaryEntry = ({SymbolDescriptor sample, NotationHelp help});

/// A representative descriptor per [SymbolKind], used to build the browsable
/// glossary so it covers exactly the same symbols as the on-staff bubbles.
const List<SymbolDescriptor> notationGlossarySamples = <SymbolDescriptor>[
  SymbolDescriptor.note(pitch: 60),
  SymbolDescriptor.rest(),
  SymbolDescriptor.accidental(token: 'sharp'),
  SymbolDescriptor.accidental(token: 'flat'),
  SymbolDescriptor.accidental(token: 'natural'),
  SymbolDescriptor.clef(sign: 'G'),
  SymbolDescriptor.clef(sign: 'F'),
  SymbolDescriptor.keySignature(fifths: 1),
  SymbolDescriptor.timeSignature(beats: 4, beatType: 4),
  SymbolDescriptor.augmentationDot(),
  SymbolDescriptor.beam(),
  SymbolDescriptor.flag(),
  SymbolDescriptor.stem(),
  SymbolDescriptor.ledgerLine(),
  SymbolDescriptor.barLine(),
  SymbolDescriptor.tie(),
  SymbolDescriptor.slur(),
  SymbolDescriptor.tuplet(actual: 3),
  SymbolDescriptor.brace(),
  SymbolDescriptor.dynamics(token: 'mf'),
];
