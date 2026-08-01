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

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/partition_painter.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/notation_playback.dart';

import '../support/notation_fakes.dart';

/// A modulating single-staff score: 4 flats (A♭ major) for the first two
/// measures, then 1 flat — like Haydn's canzonet. Measure 1 carries an A♭ so the
/// Staff painter exercises its written-diatonic placement (A♭ on the A line, not
/// collapsed onto G via the MIDI number).
ScoreDocument _modulatingDoc() => ScoreDocument(
  meta: const ScoreMeta(title: 'Mod', composer: null),
  staves: 1,
  attributes: const Attributes(
    divisions: 4,
    clefs: [Clef(staff: 1, sign: 'G', line: 2)],
    keyFifths: -1,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    NotationMeasure(
      index: 0,
      clefs: const [],
      keyFifths: -4,
      minWidth: 100,
      directions: const [],
      notes: [
        noteEvent(
          durationDivisions: 16,
          pitch: const Pitch(step: 'C', octave: 5, alter: 0),
        ),
      ],
    ),
    NotationMeasure(
      index: 1,
      clefs: const [],
      keyFifths: -4,
      minWidth: 100,
      directions: const [],
      notes: [
        noteEvent(
          durationDivisions: 16,
          pitch: const Pitch(step: 'A', octave: 4, alter: -1),
        ),
      ],
    ),
    NotationMeasure(
      index: 2,
      clefs: const [],
      keyFifths: -1,
      minWidth: 100,
      directions: const [],
      notes: [
        noteEvent(
          durationDivisions: 16,
          pitch: const Pitch(step: 'C', octave: 5, alter: 0),
        ),
      ],
    ),
  ],
);

Widget _host(CustomPainter painter, Size size) => Directionality(
  textDirection: TextDirection.ltr,
  child: Center(
    child: RepaintBoundary(
      child: SizedBox.fromSize(
        size: size,
        child: CustomPaint(size: size, painter: painter),
      ),
    ),
  ),
);

void main() {
  final doc = _modulatingDoc();

  group('Partition key signature on a modulating score', () {
    testWidgets('draws a mid-system key change without error', (tester) async {
      // Measure 2's key change (−4 → −1) lands mid-system → drawKeyChange inline.
      await tester.pumpWidget(
        _host(
          PartitionPainter(
            document: doc,
            systems: [
              System(measures: Uint32List.fromList([0]), staves: 1),
              System(measures: Uint32List.fromList([1, 2]), staves: 1),
            ],
          ),
          const Size(1200, 400),
        ),
      );
      expect(tester.takeException(), isNull);
    });

    testWidgets('draws a key change in the system header without error', (
      tester,
    ) async {
      // The change falls on a system boundary → the header shows the change.
      await tester.pumpWidget(
        _host(
          PartitionPainter(
            document: doc,
            systems: [
              System(measures: Uint32List.fromList([0, 1]), staves: 1),
              System(measures: Uint32List.fromList([2]), staves: 1),
            ],
          ),
          const Size(1200, 400),
        ),
      );
      expect(tester.takeException(), isNull);
    });
  });

  group('Scrolling staff armure follows the playhead', () {
    final derived = notationToTimedNotes(doc);

    testWidgets('shows the opening key and an A♭ on the A line', (
      tester,
    ) async {
      // Playhead in the 4-flat section; the A♭ note carries a written diatonic.
      final aFlat = derived.notes.firstWhere((n) => n.pitch == 68);
      expect(aFlat.diatonic, 4 * 7 + 5); // A4 line, not the G collapse
      await tester.pumpWidget(
        _host(
          StaffPainter(
            notes: derived.notes,
            elapsedMs: 0,
            activeNotes: const {},
            bpm: derived.bpm,
            songEndMs: derived.songEndMs,
            keyFifths: -1,
            measureKeyFifths: derived.measureKeyFifths,
            measureStartMs: derived.measureStartMs,
          ),
          const Size(1200, 320),
        ),
      );
      expect(tester.takeException(), isNull);
    });

    testWidgets('switches the head armure after the modulation', (
      tester,
    ) async {
      // Playhead in the last measure → _keyFifthsAtPlayhead returns the new key.
      final lastStart = derived.measureStartMs.last.toDouble();
      await tester.pumpWidget(
        _host(
          StaffPainter(
            notes: derived.notes,
            elapsedMs: lastStart + 1,
            activeNotes: const {},
            bpm: derived.bpm,
            songEndMs: derived.songEndMs,
            keyFifths: -1,
            measureKeyFifths: derived.measureKeyFifths,
            measureStartMs: derived.measureStartMs,
          ),
          const Size(1200, 320),
        ),
      );
      expect(tester.takeException(), isNull);
    });
  });
}
