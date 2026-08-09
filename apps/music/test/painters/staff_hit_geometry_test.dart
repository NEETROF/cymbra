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
import 'package:music/painters/staff_hit_index.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/src/rust/api/musicxml.dart' show BeamState;
import 'package:music/state/player_data.dart';

import '../support/test_fonts.dart';

const double _w = 600;
const double _h = 300;

/// A grand-staff fixture rich enough to exercise the whole glyph vocabulary the
/// hit index must cover: two staves (→ two clefs), a key signature, a beamed
/// eighth pair, a note carrying an accidental, a dotted note, a high note needing
/// ledger lines, and a rest.
List<TimedNote> _notes() => [
  // Beamed eighth pair on the treble staff.
  const TimedNote(
    pitch: 72,
    startMs: 1000,
    durationMs: 250,
    noteType: 'eighth',
    diatonic: 5 * 7 + 0,
    beams: [BeamState.begin],
  ),
  const TimedNote(
    pitch: 74,
    startMs: 1250,
    durationMs: 250,
    noteType: 'eighth',
    diatonic: 5 * 7 + 1,
    beams: [BeamState.end],
  ),
  // A note with an engraved accidental.
  const TimedNote(
    pitch: 75,
    startMs: 2000,
    durationMs: 375,
    noteType: 'eighth',
    diatonic: 5 * 7 + 1,
    accidental: 'sharp',
  ),
  // A dotted quarter high above the staff (ledger lines).
  const TimedNote(
    pitch: 88,
    startMs: 2500,
    durationMs: 750,
    noteType: 'quarter',
    diatonic: 6 * 7 + 4,
    dots: 1,
  ),
  // A bass-staff note so the grand staff (and its bass clef) is drawn.
  const TimedNote(
    pitch: 48,
    startMs: 1000,
    durationMs: 500,
    staff: 2,
    noteType: 'half',
    clefSign: 'F',
    clefLine: 4,
    diatonic: 3 * 7 + 0,
  ),
];

List<TimedRest> _rests() => [
  const TimedRest(startMs: 1500, durationMs: 500, noteType: 'quarter', dots: 1),
];

StaffPainter _painter({StaffHitIndex? hitIndex}) => StaffPainter(
  notes: _notes(),
  rests: _rests(),
  elapsedMs: 1200,
  activeNotes: const {},
  bpm: 120,
  songEndMs: 4000,
  keyFifths: 2,
  beats: 3,
  beatType: 4,
  measureStartMs: const [0, 2000],
  hitIndex: hitIndex,
);

Future<ui.Image> _render(StaffPainter painter) async {
  final recorder = ui.PictureRecorder();
  painter.paint(Canvas(recorder), const Size(_w, _h));
  return recorder.endRecording().toImage(_w.toInt(), _h.toInt());
}

void main() {
  setUpAll(loadBravura);

  test('the hit index does not change what is rendered', () async {
    final plain = await _render(_painter());
    final instrumented = await _render(_painter(hitIndex: StaffHitIndex()));

    final a = await plain.toByteData(format: ui.ImageByteFormat.rawRgba);
    final b = await instrumented.toByteData(format: ui.ImageByteFormat.rawRgba);
    plain.dispose();
    instrumented.dispose();

    expect(a!.lengthInBytes, b!.lengthInBytes);
    // Byte-for-byte identical: recording geometry is a pure side channel.
    var identical = true;
    for (var i = 0; i < a.lengthInBytes; i++) {
      if (a.getUint8(i) != b.getUint8(i)) {
        identical = false;
        break;
      }
    }
    expect(identical, isTrue, reason: 'render must be byte-identical');
  });

  test('every drawn symbol family is recorded (totality)', () async {
    final index = StaffHitIndex();
    (await _render(_painter(hitIndex: index))).dispose();

    final kinds = index.entries.map((e) => e.descriptor.kind).toSet();
    // The fixture draws all of these; each must resolve to a descriptor so no
    // rendered symbol is a dead long-press.
    expect(
      kinds,
      containsAll(<SymbolKind>{
        SymbolKind.note,
        SymbolKind.rest,
        SymbolKind.accidental,
        SymbolKind.clef,
        SymbolKind.keySignature,
        SymbolKind.timeSignature,
        SymbolKind.augmentationDot,
        SymbolKind.beam,
        SymbolKind.barLine,
        SymbolKind.ledgerLine,
      }),
    );
  });

  test('a press on a note head resolves to that note', () async {
    final index = StaffHitIndex();
    (await _render(_painter(hitIndex: index))).dispose();

    // Find a recorded note and press its centre.
    final noteEntry = index.entries.firstWhere(
      (e) => e.descriptor is NoteSymbol,
    );
    final hit = index.hitTest(noteEntry.region.center);
    expect(hit, isA<NoteSymbol>());
  });

  test('the index is refilled (not appended) across paints', () async {
    final index = StaffHitIndex();
    (await _render(_painter(hitIndex: index))).dispose();
    final first = index.length;
    (await _render(_painter(hitIndex: index))).dispose();
    // A second paint clears and rebuilds — no accumulation.
    expect(index.length, first);
  });
}
