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
import 'package:music/state/player_data.dart';

import '../support/test_fonts.dart';

const double _w = 600;
const double _h = 300;

/// A tie chain as the player sees it after the playback merge: the chain's
/// first note (playable, its duration already extended over the whole chain)
/// plus the engraved continuation on the render-only channel, anchored back to
/// the first note for the arc.
const _first = TimedNote(
  pitch: 74,
  startMs: 1000,
  durationMs: 2000, // merged: eighth + the tied half it chains into
  noteType: 'eighth',
  diatonic: 5 * 7 + 1,
);

const _continuation = TimedNote(
  pitch: 74,
  startMs: 2000,
  durationMs: 1000,
  noteType: 'half',
  diatonic: 5 * 7 + 1,
  tieFromMs: 1000,
);

StaffPainter _painter({
  List<TimedNote> tieContinuations = const [],
  StaffHitIndex? hitIndex,
}) => StaffPainter(
  notes: const [_first],
  tieContinuations: tieContinuations,
  elapsedMs: 1200,
  activeNotes: const {},
  bpm: 120,
  songEndMs: 4000,
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

  test('a tie continuation is engraved as a note plus its tie arc', () async {
    final index = StaffHitIndex();
    (await _render(
      _painter(tieContinuations: const [_continuation], hitIndex: index),
    )).dispose();

    // The continuation's head is on the staff (two note symbols, not one)…
    final noteEntries = index.entries
        .where((e) => e.descriptor is NoteSymbol)
        .toList();
    expect(noteEntries, hasLength(2));
    expect(
      noteEntries.map((e) => (e.descriptor as NoteSymbol).noteType),
      containsAll(<String>['eighth', 'half']),
    );
    // …joined by a tie arc back to the note it prolongs.
    expect(
      index.entries.map((e) => e.descriptor.kind),
      contains(SymbolKind.tie),
    );
  });

  test(
    'without the channel the continuation is absent (the reported bug)',
    () async {
      final index = StaffHitIndex();
      (await _render(_painter(hitIndex: index))).dispose();

      expect(
        index.entries.where((e) => e.descriptor is NoteSymbol),
        hasLength(1),
      );
      expect(
        index.entries.map((e) => e.descriptor.kind),
        isNot(contains(SymbolKind.tie)),
      );
    },
  );

  test('the continuation actually changes the rendered pixels', () async {
    final without = await _render(_painter());
    final withTie = await _render(
      _painter(tieContinuations: const [_continuation]),
    );
    final a = await without.toByteData(format: ui.ImageByteFormat.rawRgba);
    final b = await withTie.toByteData(format: ui.ImageByteFormat.rawRgba);
    without.dispose();
    withTie.dispose();

    var differs = false;
    for (var i = 0; i < a!.lengthInBytes; i++) {
      if (a.getUint8(i) != b!.getUint8(i)) {
        differs = true;
        break;
      }
    }
    expect(differs, isTrue, reason: 'the engraved continuation must be drawn');
  });
}
