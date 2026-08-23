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
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/partition_painter.dart';
import 'package:music/painters/staff_hit_index.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/player_data.dart';

import '../support/notation_fakes.dart';
import '../support/test_fonts.dart';

void main() {
  setUpAll(loadBravura);

  test('the scrolling staff engraves and records the repeat notation', () {
    final index = StaffHitIndex();
    const notes = [
      TimedNote(pitch: 72, startMs: 100, durationMs: 400, noteType: 'quarter'),
    ];
    final painter = StaffPainter(
      notes: notes,
      elapsedMs: 0,
      activeNotes: const {},
      bpm: 120,
      songEndMs: 8000,
      lookAheadMs: 9000,
      measureStartMs: const [0, 2000, 4000, 6000],
      measureDecors: const [
        MeasureDecor(repeatForward: true, segno: true),
        MeasureDecor(voltaLabel: '1.', repeatBackward: true),
        MeasureDecor(measureRepeat: true),
        MeasureDecor(coda: true),
      ],
      hitIndex: index,
    );
    final recorder = ui.PictureRecorder();
    painter.paint(Canvas(recorder), const Size(900, 300));
    recorder.endRecording();

    final kinds = index.entries.map((e) => e.descriptor.kind).toSet();
    expect(
      kinds,
      containsAll(<SymbolKind>{
        SymbolKind.repeatBarline,
        SymbolKind.volta,
        SymbolKind.measureRepeat,
        SymbolKind.segno,
        SymbolKind.coda,
      }),
    );
    // Both directions of the repeat barline are present and distinct.
    final barlines = index.entries
        .map((e) => e.descriptor)
        .whereType<RepeatBarlineSymbol>()
        .map((d) => d.forward)
        .toSet();
    expect(barlines, {true, false});
  });

  test('the Partition engraves and records the repeat notation', () {
    NotationMeasure m(int i, {RepeatMarks? repeats}) => NotationMeasure(
      repeats: repeats ?? noRepeats,
      index: i,
      clefs: const [],
      keyFifths: 0,
      minWidth: 160,
      directions: const [],
      notes: [
        noteEvent(
          positionDivisions: 0,
          durationDivisions: 16,
          noteType: 'whole',
          pitch: const Pitch(step: 'C', octave: 5, alter: 0),
        ),
      ],
    );
    final doc = ScoreDocument(
      instruments: const [],
      playOrder: const [],
      meta: const ScoreMeta(title: 'R', composer: 'T'),
      staves: 1,
      attributes: const Attributes(
        divisions: 4,
        clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
        keyFifths: 0,
        time: TimeSignature(beats: 4, beatType: 4),
      ),
      measures: [
        m(0, repeats: repeatMarks(forward: true, segno: true)),
        m(
          1,
          repeats: repeatMarks(
            backwardTimes: 2,
            endingStart: [1],
            endingStop: true,
          ),
        ),
        NotationMeasure(
          repeats: repeatMarks(measureRepeatOf: 1, measureRepeatSlashes: 1),
          index: 2,
          clefs: const [],
          keyFifths: 0,
          minWidth: 160,
          directions: const [],
          notes: const [],
        ),
        m(3, repeats: repeatMarks(coda: true)),
      ],
    );
    final index = StaffHitIndex();
    final painter = PartitionPainter(
      document: doc,
      systems: [
        System(measures: Uint32List.fromList(const [0, 1, 2, 3]), staves: 1),
      ],
      hitIndex: index,
    );
    final recorder = ui.PictureRecorder();
    painter.paint(Canvas(recorder), const Size(900, 400));
    recorder.endRecording();

    final kinds = index.entries.map((e) => e.descriptor.kind).toSet();
    expect(
      kinds,
      containsAll(<SymbolKind>{
        SymbolKind.repeatBarline,
        SymbolKind.volta,
        SymbolKind.measureRepeat,
        SymbolKind.segno,
        SymbolKind.coda,
      }),
    );
    final volta = index.entries
        .map((e) => e.descriptor)
        .whereType<VoltaSymbol>()
        .single;
    expect(volta.label, '1.');
  });

  test(
    'the Partition renders a repeat pass exactly like the first pass',
    () async {
      // Playhead in the SECOND pass of a repeated bar must produce the same
      // engraving as the first pass at the same fraction: cursor on the same
      // written measure, same current-measure wash, same played-line dimming.
      // (The regression: the painting instance missing writtenMeasureOf dimmed
      // the score and drew the cursor a section ahead after the jump.)
      NotationMeasure m(int i) => NotationMeasure(
        repeats: i == 0 ? repeatMarks(backwardTimes: 2) : noRepeats,
        index: i,
        clefs: const [],
        keyFifths: 0,
        minWidth: 200,
        directions: const [],
        notes: [
          noteEvent(
            positionDivisions: 0,
            durationDivisions: 16,
            noteType: 'whole',
            pitch: const Pitch(step: 'C', octave: 5, alter: 0),
          ),
        ],
      );
      final doc = ScoreDocument(
      instruments: const [],
        playOrder: const [
          PlayedMeasure(writtenIndex: 0, pass: 1),
          PlayedMeasure(writtenIndex: 0, pass: 2),
          PlayedMeasure(writtenIndex: 1, pass: 1),
        ],
        meta: const ScoreMeta(title: 'R', composer: 'T'),
        staves: 1,
        attributes: const Attributes(
          divisions: 4,
          clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
          keyFifths: 0,
          time: TimeSignature(beats: 4, beatType: 4),
        ),
        measures: [m(0), m(1)],
      );
      final systems = [
        System(measures: Uint32List.fromList(const [0]), staves: 1),
        System(measures: Uint32List.fromList(const [1]), staves: 1),
      ];

      Future<ByteData> render(double elapsedMs) async {
        final recorder = ui.PictureRecorder();
        PartitionPainter(
          document: doc,
          systems: systems,
          elapsedMs: elapsedMs,
          measureStartMs: const [0, 1000, 2000],
          writtenMeasureOf: const [0, 0, 1],
          songEndMs: 3000,
        ).paint(Canvas(recorder), const Size(700, 500));
        final img = await recorder.endRecording().toImage(700, 500);
        final bytes = await img.toByteData(format: ui.ImageByteFormat.rawRgba);
        img.dispose();
        return bytes!;
      }

      final pass1 = await render(500); // slot 0, halfway
      final pass2 = await render(1500); // slot 1 — the repeat pass, halfway
      expect(pass1.lengthInBytes, pass2.lengthInBytes);
      var identical = true;
      for (var i = 0; i < pass1.lengthInBytes; i++) {
        if (pass1.getUint8(i) != pass2.getUint8(i)) {
          identical = false;
          break;
        }
      }
      expect(
        identical,
        isTrue,
        reason: 'the repeat pass must not dim the line nor move the cursor',
      );
    },
  );
}
