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

import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/state/player_data.dart';

import '../support/test_fonts.dart';

// Geometry of the renders below, mirroring the painter's own arithmetic so the
// sampled bands are anchored to the note instead of magic numbers.
//   lineGap   = (300 * 0.10).clamp(8, 18)          = 18
//   playLineX = 600 * 0.25                          = 150
//   pxPerMs   = (600 - 150 - 48) / 4000             = 0.1005
const double _size = 300; // canvas height (width is 600)
const double _lineGap = 18;
const double _noteX = 150 + 2000 * 0.1005; // note at startMs 2000, elapsed 0
// Lone staff: bottom line centred, clamped to keep the stem clearance.
const double _staffBottom = _size / 2 + 2 * _lineGap;
// D5 sits three spaces above the treble bottom line (E4 → diatonic 30, D5 → 36).
const double _noteY = _staffBottom - 6 * (_lineGap / 2);
const double _headLeft = _noteX - 1.18 * _lineGap / 2;

/// A D♯5 eighth, optionally carrying its engraved accidental / stem direction.
List<TimedNote> _notes({String? accidental, bool? stemUp}) => [
  TimedNote(
    pitch: 75, // D♯5
    startMs: 2000,
    durationMs: 375,
    noteType: 'eighth',
    diatonic: 5 * 7 + 1, // written on the D line/space, not E♭
    accidental: accidental,
    stemUp: stemUp,
  ),
];

/// Rasterises [painter] at 600×300 and counts the painted (non-transparent)
/// pixels inside [band].
Future<int> _inkIn(StaffPainter painter, Rect band) async {
  final recorder = ui.PictureRecorder();
  painter.paint(Canvas(recorder), const Size(600, _size));
  final image = await recorder.endRecording().toImage(600, _size.toInt());
  final bytes = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
  var ink = 0;
  for (var y = band.top.floor(); y < band.bottom.ceil(); y++) {
    for (var x = band.left.floor(); x < band.right.ceil(); x++) {
      if (bytes!.getUint8((y * 600 + x) * 4 + 3) > 0) ink++;
    }
  }
  image.dispose();
  return ink;
}

StaffPainter _painter(List<TimedNote> notes) => StaffPainter(
  notes: notes,
  elapsedMs: 0,
  activeNotes: const {},
  bpm: 80,
  songEndMs: 4000,
);

void main() {
  setUpAll(loadBravura);

  // The scrolling Portée used to drop the accidental entirely: a D♯ read as a
  // plain D, contradicting the Partition view, the virtual piano and the sound.
  test('staff engraves a note accidental left of its head', () async {
    // The slot the accidental occupies: one and a half staff spaces left of the
    // head, spanning the glyph's height.
    final slot = Rect.fromLTRB(
      _headLeft - _lineGap * 1.8,
      _noteY - _lineGap * 1.6,
      _headLeft - _lineGap * 0.2,
      _noteY + _lineGap * 1.6,
    );
    final without = await _inkIn(_painter(_notes()), slot);
    final withSharp = await _inkIn(_painter(_notes(accidental: 'sharp')), slot);
    expect(withSharp, greaterThan(without));
  });

  test('staff engraves a natural the same way', () async {
    final slot = Rect.fromLTRB(
      _headLeft - _lineGap * 1.8,
      _noteY - _lineGap * 1.6,
      _headLeft - _lineGap * 0.2,
      _noteY + _lineGap * 1.6,
    );
    final without = await _inkIn(_painter(_notes()), slot);
    final withNatural = await _inkIn(
      _painter(_notes(accidental: 'natural')),
      slot,
    );
    expect(withNatural, greaterThan(without));
  });

  // Stems used to be forced upward, so a run of eighths beamed *above* the staff
  // in the Portée and *below* it in the Partition / back office for the same
  // bar. The notation's own direction now wins in both.
  group('stem direction follows the notation', () {
    // A band below the head, on the down-stem's x (the head's left edge).
    final below = Rect.fromLTRB(
      _headLeft - 2,
      _noteY + _lineGap * 1.5,
      _headLeft + 4,
      _noteY + _lineGap * 3.2,
    );
    // A band above the head, on the up-stem's x (the head's right edge).
    final above = Rect.fromLTRB(
      _headLeft + 1.18 * _lineGap - 3,
      _noteY - _lineGap * 3.2,
      _headLeft + 1.18 * _lineGap + 3,
      _noteY - _lineGap * 1.5,
    );

    test('a down-stemmed note draws its stem below the head', () async {
      final down = await _inkIn(_painter(_notes(stemUp: false)), below);
      final up = await _inkIn(_painter(_notes(stemUp: true)), below);
      expect(down, greaterThan(up));
    });

    test('an up-stemmed note draws its stem above the head', () async {
      final up = await _inkIn(_painter(_notes(stemUp: true)), above);
      final down = await _inkIn(_painter(_notes(stemUp: false)), above);
      expect(up, greaterThan(down));
    });

    // Without a carried direction the painter falls back to the engraving rule
    // (heads below the middle line stem up), so nothing regresses for the demo
    // score and the MIDI-only replay, which carry no stem.
    test('a low note with no carried stem still stems up', () async {
      // G4 (MIDI 67), a space below the treble middle line.
      const low = [
        TimedNote(
          pitch: 67,
          startMs: 2000,
          durationMs: 375,
          noteType: 'eighth',
          diatonic: 4 * 7 + 4,
        ),
      ];
      const lowY = _staffBottom - 2 * (_lineGap / 2);
      final aboveLow = Rect.fromLTRB(
        _headLeft + 1.18 * _lineGap - 3,
        lowY - _lineGap * 3.2,
        _headLeft + 1.18 * _lineGap + 3,
        lowY - _lineGap * 1.5,
      );
      expect(await _inkIn(_painter(low), aboveLow), greaterThan(0));
    });
  });
}
