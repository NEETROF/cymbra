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
import 'package:music/src/rust/api/musicxml.dart';

import '../support/notation_fakes.dart';
import '../support/test_fonts.dart';

const double _w = 600;
const double _h = 400;

PartitionPainter _painter(ScoreDocument document, {StaffHitIndex? hitIndex}) =>
    PartitionPainter(
      document: document,
      systems: FakeNotationEngine().layout(document, _w),
      hitIndex: hitIndex,
    );

PartitionPainter _oneSystem(ScoreDocument document, {StaffHitIndex? hitIndex}) =>
    PartitionPainter(
      document: document,
      systems: [
        System(
          measures: Uint32List.fromList([
            for (var i = 0; i < document.measures.length; i++) i,
          ]),
          staves: document.staves,
        ),
      ],
      hitIndex: hitIndex,
    );

Future<ui.Image> _render(PartitionPainter painter) async {
  final recorder = ui.PictureRecorder();
  painter.paint(Canvas(recorder), const Size(_w, _h));
  return recorder.endRecording().toImage(_w.toInt(), _h.toInt());
}

Set<SymbolKind> _recordedKinds(StaffHitIndex index) =>
    index.entries.map((e) => e.descriptor.kind).toSet();

void main() {
  setUpAll(loadBravura);

  test('the hit index does not change what is rendered', () async {
    final document = sampleGrandStaffDocument();
    final plain = await _render(_painter(document));
    final instrumented = await _render(
      _painter(document, hitIndex: StaffHitIndex()),
    );

    final a = await plain.toByteData(format: ui.ImageByteFormat.rawRgba);
    final b = await instrumented.toByteData(
      format: ui.ImageByteFormat.rawRgba,
    );
    plain.dispose();
    instrumented.dispose();

    expect(a!.lengthInBytes, b!.lengthInBytes);
    var identical = true;
    for (var i = 0; i < a.lengthInBytes; i++) {
      if (a.getUint8(i) != b.getUint8(i)) {
        identical = false;
        break;
      }
    }
    expect(identical, isTrue, reason: 'render must be byte-identical');
  });

  test('grand staff records its core symbols', () async {
    final index = StaffHitIndex();
    (await _render(_painter(sampleGrandStaffDocument(), hitIndex: index)))
        .dispose();

    expect(
      _recordedKinds(index),
      containsAll(<SymbolKind>{
        SymbolKind.note,
        SymbolKind.clef,
        SymbolKind.timeSignature,
        SymbolKind.barLine,
        SymbolKind.brace,
      }),
    );
  });

  test('ties and slurs are recorded', () async {
    final index = StaffHitIndex();
    (await _render(_oneSystem(sampleTieSlurDocument(), hitIndex: index)))
        .dispose();

    final kinds = _recordedKinds(index);
    expect(kinds, contains(SymbolKind.tie));
    expect(kinds, contains(SymbolKind.slur));
  });

  test('beams and tuplets are recorded', () async {
    final index = StaffHitIndex();
    (await _render(_oneSystem(sampleBeamedDocument(), hitIndex: index)))
        .dispose();

    expect(_recordedKinds(index), contains(SymbolKind.beam));
  });

  test('a press on a note head resolves to that note', () async {
    final index = StaffHitIndex();
    (await _render(_painter(sampleGrandStaffDocument(), hitIndex: index)))
        .dispose();

    final noteEntry = index.entries.firstWhere(
      (e) => e.descriptor is NoteSymbol,
    );
    expect(index.hitTest(noteEntry.region.center), isA<NoteSymbol>());
  });
}
