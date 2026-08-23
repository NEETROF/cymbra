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

import 'package:flutter/painting.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/staff_hit_index.dart';

void main() {
  group('StaffHitIndex.hitTest', () {
    test('a press inside a glyph resolves to it', () {
      final index = StaffHitIndex()
        ..add(
          const Rect.fromLTWH(0, 0, 20, 20),
          const SymbolDescriptor.note(pitch: 60),
        )
        ..add(
          const Rect.fromLTWH(100, 0, 20, 20),
          const SymbolDescriptor.clef(sign: 'G'),
        );

      final hit = index.hitTest(const Offset(10, 10));
      expect(hit, isA<NoteSymbol>());
      expect((hit! as NoteSymbol).pitch, 60);
    });

    test('when regions overlap, the closest centre wins', () {
      // A note head sitting under a wide beam that also covers the press point.
      final index = StaffHitIndex()
        ..add(const Rect.fromLTWH(0, 0, 200, 8), const SymbolDescriptor.beam())
        ..add(
          const Rect.fromLTWH(46, 0, 12, 12),
          const SymbolDescriptor.note(pitch: 64),
        );

      // Press on the note head (also within the beam's horizontal span).
      expect(index.hitTest(const Offset(52, 6)), isA<NoteSymbol>());
    });

    test('nearest symbol wins when the press is between two glyphs', () {
      final index = StaffHitIndex()
        ..add(
          const Rect.fromLTWH(0, 0, 10, 10),
          const SymbolDescriptor.note(pitch: 60),
        )
        ..add(
          const Rect.fromLTWH(40, 0, 10, 10),
          const SymbolDescriptor.note(pitch: 72),
        );

      // Just right of the first note, still within tolerance of it.
      final hit = index.hitTest(const Offset(16, 5));
      expect((hit! as NoteSymbol).pitch, 60);
    });

    test('empty staff area (beyond tolerance) resolves to null', () {
      final index = StaffHitIndex()
        ..add(const Rect.fromLTWH(0, 0, 10, 10), const SymbolDescriptor.rest());

      expect(index.hitTest(const Offset(300, 300)), isNull);
    });

    test('an empty index resolves to null', () {
      expect(StaffHitIndex().hitTest(Offset.zero), isNull);
    });

    test('empty rects are not recorded', () {
      final index = StaffHitIndex()
        ..add(Rect.zero, const SymbolDescriptor.barLine());
      expect(index.isEmpty, isTrue);
      expect(index.length, 0);
    });

    test('clear drops all entries', () {
      final index = StaffHitIndex()
        ..add(
          const Rect.fromLTWH(0, 0, 10, 10),
          const SymbolDescriptor.clef(sign: 'F'),
        );
      expect(index.isNotEmpty, isTrue);
      index.clear();
      expect(index.isEmpty, isTrue);
    });
  });

  group('SymbolDescriptor.kind', () {
    test('every variant maps to a distinct, total kind', () {
      const descriptors = <SymbolDescriptor>[
        SymbolDescriptor.note(pitch: 60),
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
      // One descriptor per kind, and each kind covered exactly once.
      final kinds = descriptors.map((d) => d.kind).toSet();
      expect(kinds.length, descriptors.length);
      expect(kinds.length, SymbolKind.values.length);
    });
  });
}
