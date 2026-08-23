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
import 'package:music/src/rust/api/musicxml.dart';

import '../support/notation_fakes.dart';
import '../support/test_fonts.dart';

const _w = 900;
const _h = 400;

/// One 4/4 measure of four quarters on the beat (divisions 4 → 16 per measure).
ScoreDocument _fourQuarters() => ScoreDocument(
      instruments: const [],
  playOrder: const [],
  meta: const ScoreMeta(title: 'P', composer: 'C'),
  staves: 1,
  attributes: const Attributes(
    divisions: 4,
    clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    NotationMeasure(
      repeats: noRepeats,
      index: 0,
      clefs: const [],
      keyFifths: 0,
      minWidth: 200,
      directions: const [],
      notes: [
        for (var i = 0; i < 4; i++)
          noteEvent(
            positionDivisions: i * 4,
            pitch: const Pitch(step: 'B', octave: 4, alter: 0),
          ),
      ],
    ),
  ],
);

/// Renders the measure at [elapsedMs] and returns the playhead's x (the teal
/// stroke's ink centre) together with every note head's ink centre on the B4
/// row, both in painter pixels.
Future<({double cursorX, List<double> headXs})> _probe(double elapsedMs) async {
  final painter = PartitionPainter(
    document: _fourQuarters(),
    systems: [
      System(measures: Uint32List.fromList([0]), staves: 1),
    ],
    elapsedMs: elapsedMs,
    measureStartMs: const [0],
    songEndMs: 4000,
  );
  final rec = ui.PictureRecorder();
  painter.paint(Canvas(rec), const Size(_w * 1.0, _h * 1.0));
  final img = await rec.endRecording().toImage(_w, _h);
  final b = (await img.toByteData(format: ui.ImageByteFormat.rawRgba))!;

  // The playhead is the only teal (CymbraColors.secondary, 0xFF44E2CD) ink.
  var sum = 0.0, n = 0;
  for (var x = 0; x < _w; x++) {
    for (var y = 60; y < 220; y++) {
      final i = (y * _w + x) * 4;
      if (b.getUint8(i) < 120 &&
          b.getUint8(i + 1) > 190 &&
          b.getUint8(i + 2) > 170 &&
          b.getUint8(i + 3) > 200) {
        sum += x;
        n++;
      }
    }
  }

  // Note heads: the widest opaque row is the B4 line; each run of ink on it
  // that is at least half a head wide is a head, taken at its mid-point.
  var maxRow = 0, headY = 0;
  for (var y = 60; y < 220; y++) {
    var row = 0;
    for (var x = 0; x < _w; x++) {
      if (b.getUint8((y * _w + x) * 4 + 3) > 200) row++;
    }
    if (row > maxRow) {
      maxRow = row;
      headY = y;
    }
  }
  final heads = <double>[];
  var runStart = -1;
  for (var x = 0; x < _w; x++) {
    final on = b.getUint8((headY * _w + x) * 4 + 3) > 200;
    if (on && runStart < 0) runStart = x;
    if (!on && runStart >= 0) {
      if (x - runStart >= 8) heads.add((runStart + x - 1) / 2);
      runStart = -1;
    }
  }
  img.dispose();
  return (cursorX: n > 0 ? sum / n : -1.0, headXs: heads);
}

void main() {
  setUpAll(loadBravura);

  // The playhead used to be placed by the raw measure fraction
  // (measureX + fraction * measureWidth) while the heads were placed on note
  // columns inset from the bar line — so it sat ~13 px *behind* the downbeat,
  // caught up mid-measure and overshot by ~9 px on the last beat. It now rides
  // the same column mapping, so it lands on the head it points at.
  test('the playhead sits on the note it points at, every beat', () async {
    for (var beat = 0; beat < 4; beat++) {
      final r = await _probe(beat * 1000.0);
      // The head columns are the last four runs (the clef makes the first ones).
      final heads = r.headXs.sublist(r.headXs.length - 4);
      final target = heads[beat];
      expect(
        (r.cursorX - target).abs(),
        lessThan(3),
        reason:
            'beat $beat: playhead at ${r.cursorX}, head at $target '
            '(heads: ${r.headXs})',
      );
    }
  });
}
