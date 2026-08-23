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

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/notation/notation_help_content.dart';
import 'package:music/painters/staff_hit_index.dart';

/// One descriptor per SymbolKind, so the totality assertion covers every kind a
/// painter can record.
const _oneOfEachKind = <SymbolDescriptor>[
  SymbolDescriptor.note(pitch: 60),
  SymbolDescriptor.note(pitch: 71, noteType: 'eighth', isGrace: true),
  SymbolDescriptor.rest(),
  SymbolDescriptor.accidental(token: 'sharp'),
  SymbolDescriptor.clef(sign: 'G'),
  SymbolDescriptor.keySignature(fifths: 2),
  SymbolDescriptor.timeSignature(beats: 4, beatType: 4),
  SymbolDescriptor.augmentationDot(),
  SymbolDescriptor.stem(),
  SymbolDescriptor.flag(),
  SymbolDescriptor.beam(),
  SymbolDescriptor.ledgerLine(),
  SymbolDescriptor.barLine(),
  SymbolDescriptor.tie(),
  SymbolDescriptor.slur(),
  SymbolDescriptor.tuplet(actual: 3),
  SymbolDescriptor.brace(),
  SymbolDescriptor.dynamics(token: 'mf'),
  SymbolDescriptor.repeatBarline(forward: true),
  SymbolDescriptor.volta(label: '1.'),
  SymbolDescriptor.measureRepeat(),
  SymbolDescriptor.segno(),
  SymbolDescriptor.coda(),
  SymbolDescriptor.jump(words: 'D.C. al Fine'),
];

void main() {
  test('_oneOfEachKind truly covers every SymbolKind', () {
    expect(
      _oneOfEachKind.map((d) => d.kind).toSet().length,
      SymbolKind.values.length,
    );
  });

  for (final code in AppLocalizations.supportedLocales.map(
    (l) => l.languageCode,
  )) {
    group('locale "$code"', () {
      late AppLocalizations l10n;
      late bool solfege;
      late bool frenchRe;

      setUp(() async {
        final locale = Locale(code);
        l10n = await AppLocalizations.delegate.load(locale);
        final style = notationNameStyle(locale);
        solfege = style.solfege;
        frenchRe = style.frenchRe;
      });

      test('every symbol kind has non-empty title and body', () {
        for (final d in _oneOfEachKind) {
          final help = notationHelpFor(
            l10n,
            d,
            solfege: solfege,
            frenchRe: frenchRe,
          );
          expect(help.title.trim(), isNotEmpty, reason: '${d.kind} title');
          expect(help.body.trim(), isNotEmpty, reason: '${d.kind} body');
        }
      });

      test('each accidental token has distinct help', () {
        String title(String token) => notationHelpFor(
          l10n,
          SymbolDescriptor.accidental(token: token),
          solfege: solfege,
          frenchRe: frenchRe,
        ).title;
        final titles = {
          title('sharp'),
          title('flat'),
          title('natural'),
          title('double-sharp'),
          title('flat-flat'),
        };
        expect(titles.length, 5);
      });

      test('an empty key signature reads differently from a keyed one', () {
        String body(int fifths) => notationHelpFor(
          l10n,
          SymbolDescriptor.keySignature(fifths: fifths),
          solfege: solfege,
          frenchRe: frenchRe,
        ).body;
        expect(body(0), isNot(equals(body(3))));
      });

      test('a grace note reads as an ornament, not as a held note', () {
        NotationHelp helpOf(SymbolDescriptor d) =>
            notationHelpFor(l10n, d, solfege: solfege, frenchRe: frenchRe);
        final grace = helpOf(
          const SymbolDescriptor.note(
            pitch: 71,
            noteType: 'eighth',
            isGrace: true,
          ),
        );
        final plain = helpOf(
          const SymbolDescriptor.note(pitch: 71, noteType: 'eighth'),
        );
        // Dedicated copy, and it never quotes the eighth's hold duration.
        expect(grace.title, isNot(equals(plain.title)));
        expect(grace.body, isNot(equals(plain.body)));
        // The grace symbol is browsable from the glossary too.
        expect(
          notationGlossarySamples.any((d) => d is NoteSymbol && d.isGrace),
          isTrue,
        );
      });

      test('the glossary shares the on-staff help for the same symbol', () {
        const sample = SymbolDescriptor.clef(sign: 'F');
        final direct = notationHelpFor(
          l10n,
          sample,
          solfege: solfege,
          frenchRe: frenchRe,
        );
        // notationGlossarySamples feeds the glossary; the same lookup is used.
        expect(notationGlossarySamples, contains(sample));
        final viaGlossary = notationHelpFor(
          l10n,
          notationGlossarySamples.firstWhere((d) => d == sample),
          solfege: solfege,
          frenchRe: frenchRe,
        );
        expect(viaGlossary, equals(direct));
      });
    });
  }

  group('duration & rest naming (English)', () {
    late AppLocalizations l10n;
    setUp(() async {
      l10n = await AppLocalizations.delegate.load(const Locale('en'));
    });

    NotationHelp help(SymbolDescriptor d) =>
        notationHelpFor(l10n, d, solfege: false, frenchRe: false);

    test('a note ends with its pitch and duration in beats', () {
      // C6 (MIDI 84), a quarter note → "1 beat".
      final h = help(
        const SymbolDescriptor.note(pitch: 84, noteType: 'quarter'),
      );
      expect(h.title, contains('C6'));
      expect(h.body, contains('C6'));
      expect(h.body, contains('1 beat'));
    });

    test('a dotted half note reads three beats', () {
      final h = help(
        const SymbolDescriptor.note(pitch: 60, noteType: 'half', dots: 1),
      );
      expect(h.body, contains('3 beats'));
    });

    test('a note without a known type shows no duration suffix', () {
      final h = help(const SymbolDescriptor.note(pitch: 60));
      expect(h.body, equals(l10n.notationHelpNoteBody));
    });

    test('rests are named and dated by their value', () {
      final half = help(const SymbolDescriptor.rest(noteType: 'half'));
      expect(half.title, 'Half rest');
      expect(half.body, contains('2 beats'));
      final quarter = help(const SymbolDescriptor.rest(noteType: 'quarter'));
      expect(quarter.title, 'Quarter rest');
      expect(quarter.body, contains('1 beat'));
      // The half rest and the quarter rest are clearly distinguished.
      expect(half.title, isNot(equals(quarter.title)));
    });

    test('an unknown rest falls back to the generic title/body', () {
      final h = help(const SymbolDescriptor.rest());
      expect(h.title, l10n.notationHelpRestTitle);
      expect(h.body, l10n.notationHelpRestBody);
    });

    test('the time signature names the note the bottom number stands for', () {
      final h = help(
        const SymbolDescriptor.timeSignature(beats: 4, beatType: 4),
      );
      expect(h.body, contains('quarter note'));
      final sixEight = help(
        const SymbolDescriptor.timeSignature(beats: 6, beatType: 8),
      );
      expect(sixEight.body, contains('eighth note'));
    });
  });

  test('note names use letters in English and solfège elsewhere', () {
    // C4 is MIDI 60.
    expect(notationPitchName(60, solfege: false, frenchRe: false), 'C4');
    expect(notationPitchName(60, solfege: true, frenchRe: true), 'Do4');
    // A sharp/black key spells with ♯.
    expect(notationPitchName(61, solfege: false, frenchRe: false), 'C♯4');
  });
}
