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
import 'package:music/src/rust/api/musicxml.dart' show BeamState;
import 'package:music/state/note_density_core.dart';
import 'package:music/state/player_data.dart';

import '../support/test_fonts.dart';

// Geometry of the renders below, mirroring the painter's own arithmetic.
//   lineGap     = min((300 * 0.10).clamp(8, 12), 300 / 12)  = 12
//   playLineX   = 600 * 0.25                                = 150
//   trackPx     = 600 - 150 - 48                            = 402
const double _height = 300;
const double _width = 600;
const double _lineGap = 12;
const double _playLineX = 150;
const double _trackPx = _width - _playLineX - 48;
// Lone staff: bottom line centred, clamped to keep the stem clearance.
const double _staffBottom = _height / 2 + 2 * _lineGap;

/// Sixteenth-note motion at ♩=120: eight onsets 125 ms apart, all on F4 (the
/// first space above the bottom line, so a horizontal band through that space
/// crosses the noteheads but no staff line). Beamed, like a real sixteenth
/// run — an unbeamed one would hang a flag past the last stem and blur where
/// the notation actually ends.
const int _gapMs = 125;
const int _onsets = 8;
final List<TimedNote> _run = List.generate(_onsets, (i) {
  return TimedNote(
    pitch: 65,
    startMs: i * _gapMs,
    durationMs: _gapMs,
    noteType: '16th',
    diatonic: 4 * 7 + 3, // F4: one step above the E4 bottom line
    beams: [
      if (i == 0)
        BeamState.begin
      else if (i == _onsets - 1)
        BeamState.end
      else
        BeamState.continue_,
    ],
  );
});

StaffPainter _painter({double? onsetGapMs}) => StaffPainter(
  notes: _run,
  elapsedMs: 0,
  activeNotes: const {},
  bpm: 120,
  // Both bar lines land far off-canvas, so the only ink in the sampled band is
  // the notation itself.
  measureStartMs: const [0, 100000],
  songEndMs: 100000,
  onsetGapMs: onsetGapMs,
);

/// The x of the rightmost painted pixel inside the staff space the run sits in
/// — i.e. the right edge of the last note of the run.
Future<int> _rightmostNoteX(StaffPainter painter) async {
  final recorder = ui.PictureRecorder();
  painter.paint(Canvas(recorder), const Size(_width, _height));
  final image = await recorder.endRecording().toImage(
    _width.toInt(),
    _height.toInt(),
  );
  final bytes = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
  image.dispose();
  // Strictly inside the space: clear of the E4 and G4 lines above and below.
  final top = (_staffBottom - _lineGap + 2.5).round();
  final bottom = (_staffBottom - 2.5).round();
  var rightmost = 0;
  for (var y = top; y < bottom; y++) {
    for (var x = _width.toInt() - 1; x > rightmost; x--) {
      if (bytes!.getUint8((y * _width.toInt() + x) * 4 + 3) > 0) {
        rightmost = x;
        break;
      }
    }
  }
  return rightmost;
}

void main() {
  setUpAll(loadBravura);

  // The Portée spreads a fixed 4 s over the width whatever the piece, so a fast
  // dense score was engraved as tight as its metre happened to make it — Für
  // Elise's sixteenths advanced ~13 px where the glyphs need ~32.
  group('StaffPainter density cap', () {
    // Where the last onset of the run must reach for every gap in it to clear
    // the painter's glyph budget.
    const legibleX =
        _playLineX + (_onsets - 1) * StaffPainter.minOnsetSpaces * _lineGap;

    test(
      'without the score\'s density the run is engraved too tight',
      () async {
        // 402 px / 4000 ms = 0.1005 px/ms → a sixteenth advances 12.6 px.
        expect(await _rightmostNoteX(_painter()), lessThan(legibleX));
      },
    );

    test('the score\'s density spreads the run until its glyphs fit', () async {
      final rightmost = await _rightmostNoteX(
        _painter(onsetGapMs: _gapMs.toDouble()),
      );
      expect(rightmost, greaterThanOrEqualTo(legibleX));
      // The window narrowed rather than the notation shrinking: the run still
      // ends inside the track, it just uses more of it.
      expect(rightmost, lessThan(_playLineX + _trackPx));
    });

    test('a sparse score is left exactly as it was', () async {
      // Half notes at ♩=120 (1 s apart) already clear the budget at the full
      // window, so the cap must not move a single pixel.
      final capped = await _rightmostNoteX(_painter(onsetGapMs: 1000));
      expect(capped, await _rightmostNoteX(_painter()));
    });

    test('the floor bounds how far the cap will go', () async {
      // Absurdly dense input cannot collapse the window: the run is spread by
      // the floor, not by the (unreachable) glyph budget.
      final painter = _painter(onsetGapMs: 1);
      final window = densityCappedLookAheadMs(
        requestedMs: StaffPainter.defaultLookAheadMs,
        trackPx: _trackPx,
        lineGap: _lineGap,
        minSpaces: StaffPainter.minOnsetSpaces,
        gapMs: 1,
      );
      expect(window, kMinLookAheadMs);
      const expectedX =
          _playLineX + (_onsets - 1) * _gapMs * _trackPx / kMinLookAheadMs;
      expect(await _rightmostNoteX(painter), closeTo(expectedX, 10));
    });
  });
}
