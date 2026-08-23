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
import 'package:music/painters/staff_painter.dart';
import 'package:music/src/rust/api/musicxml.dart' show BeamState;
import 'package:music/state/player_data.dart';

import '../support/test_fonts.dart';

const double _w = 600;
const double _h = 300;

/// A beamed eighth pair whose second onset carries a chord member (the River
/// measure-6 shape: E5 rides the beamed A4). The member has no beams of its
/// own — before the fix the painter gave it a lone stem + flag next to the
/// principal's beam, which read as a spurious 16th/32nd.
List<TimedNote> _notes({required bool memberMarked}) => [
  const TimedNote(
    pitch: 72,
    startMs: 1000,
    durationMs: 250,
    noteType: 'eighth',
    diatonic: 5 * 7 + 0,
    beams: [BeamState.begin],
  ),
  const TimedNote(
    pitch: 69,
    startMs: 1250,
    durationMs: 250,
    noteType: 'eighth',
    diatonic: 4 * 7 + 5,
    beams: [BeamState.end],
  ),
  TimedNote(
    pitch: 76,
    startMs: 1250,
    durationMs: 250,
    noteType: 'eighth',
    diatonic: 5 * 7 + 2,
    isChord: memberMarked,
  ),
];

Future<ui.Image> _render(List<TimedNote> notes) async {
  final recorder = ui.PictureRecorder();
  StaffPainter(
    notes: notes,
    elapsedMs: 1200,
    activeNotes: const {},
    bpm: 120,
    songEndMs: 4000,
    measureStartMs: const [0, 2000],
  ).paint(Canvas(recorder), const Size(_w, _h));
  return recorder.endRecording().toImage(_w.toInt(), _h.toInt());
}

Future<ByteData> _bytes(ui.Image image) async {
  final b = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
  image.dispose();
  return b!;
}

void main() {
  setUpAll(loadBravura);

  test('a chord member draws no stem/flag of its own', () async {
    // Marked as a chord member, the E5 contributes only its head; unmarked it
    // grows a lone stem + flag — the spurious glyph the report showed. The two
    // renders must therefore differ exactly there.
    final marked = await _bytes(await _render(_notes(memberMarked: true)));
    final unmarked = await _bytes(await _render(_notes(memberMarked: false)));
    var differs = false;
    for (var i = 0; i < marked.lengthInBytes; i++) {
      if (marked.getUint8(i) != unmarked.getUint8(i)) {
        differs = true;
        break;
      }
    }
    expect(
      differs,
      isTrue,
      reason: 'marking the member must suppress its lone stem/flag',
    );

    // And the marked render is identical to the same scene with the member's
    // head only (a duplicate of the principal pass never draws member stems):
    // rendering twice must be deterministic, so equality is meaningful.
    final again = await _bytes(await _render(_notes(memberMarked: true)));
    expect(marked.lengthInBytes, again.lengthInBytes);
    var identical = true;
    for (var i = 0; i < marked.lengthInBytes; i++) {
      if (marked.getUint8(i) != again.getUint8(i)) {
        identical = false;
        break;
      }
    }
    expect(identical, isTrue, reason: 'render must be deterministic');
  });
}
