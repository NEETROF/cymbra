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
import 'package:music/state/player_data.dart' show Hand;

import '../support/notation_fakes.dart';
import '../support/test_fonts.dart';

void main() {
  setUpAll(loadBravura);

  test('paints a two-staff grand staff without error', () {
    final document = sampleGrandStaffDocument();
    final painter = PartitionPainter(
      document: document,
      systems: FakeNotationEngine().layout(document, 600),
    );
    final recorder = ui.PictureRecorder();
    final canvas = Canvas(recorder);
    // Exercises both staves, directions, a lyric, a hollow (whole) note head.
    painter.paint(canvas, const Size(600, 400));
    recorder.endRecording();

    expect(painter.heightFor(600), greaterThan(0));
    expect(painter.document.staves, 2);
  });

  test('paints ties, slurs, tuplets and a clef change without error', () {
    for (final document in [
      sampleTieSlurDocument(),
      sampleClefChangeDocument(),
      sampleBeamedDocument(),
    ]) {
      final painter = PartitionPainter(
        document: document,
        systems: [
          System(
            measures: Uint32List.fromList([
              for (var i = 0; i < document.measures.length; i++) i,
            ]),
            staves: document.staves,
          ),
        ],
      );
      final recorder = ui.PictureRecorder();
      painter.paint(Canvas(recorder), const Size(600, 400));
      recorder.endRecording();
    }
  });

  test('single-hand selection collapses to one staff', () {
    final document = sampleGrandStaffDocument();
    final systems = FakeNotationEngine().layout(document, 600);
    final both = PartitionPainter(document: document, systems: systems);
    final right = PartitionPainter(
      document: document,
      systems: systems,
      selectedHands: Hand.right,
    );
    // Collapsing a staff shortens the system (no bass staff + inter-staff gap).
    expect(right.heightFor(600), lessThan(both.heightFor(600)));
    expect(right.shouldRepaint(both), isTrue); // selection drives a repaint

    // Both single-hand selections paint without error.
    for (final hand in [Hand.left, Hand.right]) {
      final painter = PartitionPainter(
        document: document,
        systems: systems,
        selectedHands: hand,
      );
      final recorder = ui.PictureRecorder();
      painter.paint(Canvas(recorder), const Size(600, 400));
      recorder.endRecording();
    }
  });

  test('shouldRepaint reflects document/systems changes', () {
    final doc = sampleGrandStaffDocument();
    final systems = FakeNotationEngine().layout(doc, 600);
    final a = PartitionPainter(document: doc, systems: systems);
    final same = PartitionPainter(document: doc, systems: systems);
    final other = PartitionPainter(
      document: doc,
      systems: FakeNotationEngine().layout(doc, 300),
    );
    expect(a.shouldRepaint(same), isFalse);
    expect(a.shouldRepaint(other), isTrue);
  });

  test('shouldRepaint reflects the visible window (viewport cull)', () {
    final doc = sampleGrandStaffDocument();
    final systems = FakeNotationEngine().layout(doc, 600);
    final a = PartitionPainter(
      document: doc,
      systems: systems,
      viewTop: 0,
      viewBottom: 200,
    );
    final sameWindow = PartitionPainter(
      document: doc,
      systems: systems,
      viewTop: 0,
      viewBottom: 200,
    );
    final scrolled = PartitionPainter(
      document: doc,
      systems: systems,
      viewTop: 500,
      viewBottom: 700,
    );
    expect(a.shouldRepaint(sameWindow), isFalse);
    expect(a.shouldRepaint(scrolled), isTrue);
  });

  test('viewport cull skips systems outside the visible window', () async {
    // Stack many systems so most fall off-screen, then paint once with the whole
    // score visible (no window) and once with only the top window; a band over a
    // far-down system must carry ink in the first and none in the second.
    final doc = sampleBeamedDocument();
    final allMeasures = Uint32List.fromList([
      for (var i = 0; i < doc.measures.length; i++) i,
    ]);
    const systemCount = 8;
    final systems = [
      for (var i = 0; i < systemCount; i++)
        System(measures: allMeasures, staves: doc.staves),
    ];
    const width = 600.0;

    final full = PartitionPainter(document: doc, systems: systems);
    final height = full.heightFor(width);
    // Window shows only the first system; systems further down are culled.
    final windowed = PartitionPainter(
      document: doc,
      systems: systems,
      viewTop: 0,
      viewBottom: full.systemStride,
    );

    // A thin horizontal band across the last (far-down) system.
    final bandY = full.systemTopY(systemCount - 1) + full.systemStride / 2;

    Future<bool> hasInkInBand(PartitionPainter painter) async {
      final recorder = ui.PictureRecorder();
      painter.paint(Canvas(recorder), Size(width, height));
      final image = await recorder.endRecording().toImage(
        width.toInt(),
        height.toInt(),
      );
      final bytes = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
      final w = image.width;
      final y = bandY.toInt().clamp(0, image.height - 1);
      for (var x = 0; x < w; x++) {
        if (bytes!.getUint8((y * w + x) * 4 + 3) != 0) return true; // alpha > 0
      }
      return false;
    }

    expect(await hasInkInBand(full), isTrue); // whole score → last system drawn
    expect(await hasInkInBand(windowed), isFalse); // culled → nothing there
  });

  test('staff space scales every system dimension (score size)', () {
    final doc = tallDocument(2);
    final systems = [
      System(measures: Uint32List.fromList([0]), staves: 1),
      System(measures: Uint32List.fromList([1]), staves: 1),
    ];
    final base = PartitionPainter(document: doc, systems: systems);
    final large = PartitionPainter(
      document: doc,
      systems: systems,
      staffSpace: 12 * 1.2,
    );
    expect(large.systemStride, closeTo(base.systemStride * 1.2, 1e-6));
    expect(large.heightFor(600), closeTo(base.heightFor(600) * 1.2, 1e-6));
    expect(large.systemTopY(1), closeTo(base.systemTopY(1) * 1.2, 1e-6));
    expect(base.shouldRepaint(large), isTrue); // size drives a repaint
  });

  test('systems before the playhead render dimmed, none without one', () async {
    final doc = tallDocument(2);
    final systems = [
      System(measures: Uint32List.fromList([0]), staves: 1),
      System(measures: Uint32List.fromList([1]), staves: 1),
    ];
    const width = 600.0;

    // Max alpha along the first system's bottom staff line.
    Future<int> maxAlphaOnFirstStaffLine(PartitionPainter painter) async {
      final height = painter.heightFor(width);
      final recorder = ui.PictureRecorder();
      painter.paint(Canvas(recorder), Size(width, height));
      final image = await recorder.endRecording().toImage(
        width.toInt(),
        height.toInt(),
      );
      final bytes = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
      // systemTop(0) + topPad(5s) + staffHeight(4s) = gap(3s) + 9s.
      final y = (painter.staffSpace * 12).round().clamp(0, image.height - 1);
      var maxAlpha = 0;
      for (var x = 0; x < image.width; x++) {
        final a = bytes!.getUint8((y * image.width + x) * 4 + 3);
        if (a > maxAlpha) maxAlpha = a;
      }
      return maxAlpha;
    }

    // Playhead in measure 1 (system 1) → system 0 is fully played → dimmed.
    final playing = PartitionPainter(
      document: doc,
      systems: systems,
      elapsedMs: 1500,
      measureStartMs: const [0, 1000],
      songEndMs: 2000,
    );
    // No playhead → full opacity everywhere.
    final idle = PartitionPainter(document: doc, systems: systems);

    final dimmed = await maxAlphaOnFirstStaffLine(playing);
    final full = await maxAlphaOnFirstStaffLine(idle);
    expect(full, greaterThan(150)); // staff line at its normal 0.7 alpha
    expect(dimmed, greaterThan(0)); // still drawn…
    expect(dimmed, lessThan(120)); // …but visibly faded (~0.45×)
  });

  test('the active measure carries a background wash', () async {
    final doc = tallDocument(2);
    final systems = [
      System(measures: Uint32List.fromList([0]), staves: 1),
      System(measures: Uint32List.fromList([1]), staves: 1),
    ];
    const width = 600.0;

    // Sample just below the first system's bottom staff line, at the right
    // edge: inside the wash rect, clear of staff lines, glyphs and cursor.
    Future<int> alphaBelowFirstStaff(PartitionPainter painter) async {
      final height = painter.heightFor(width);
      final recorder = ui.PictureRecorder();
      painter.paint(Canvas(recorder), Size(width, height));
      final image = await recorder.endRecording().toImage(
        width.toInt(),
        height.toInt(),
      );
      final bytes = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
      final y = (painter.staffSpace * 12.5).round().clamp(0, image.height - 1);
      final x = image.width - 3;
      return bytes!.getUint8((y * image.width + x) * 4 + 3);
    }

    // Playhead mid-measure 0 → its measure is washed.
    final playing = PartitionPainter(
      document: doc,
      systems: systems,
      elapsedMs: 500,
      measureStartMs: const [0, 1000],
      songEndMs: 2000,
    );
    final idle = PartitionPainter(document: doc, systems: systems);

    expect(await alphaBelowFirstStaff(playing), greaterThan(0));
    expect(await alphaBelowFirstStaff(idle), 0); // no wash when stopped
  });

  testWidgets('partition tie/slur golden', (tester) async {
    final document = sampleTieSlurDocument();
    final painter = PartitionPainter(
      // Name the loaded test face so engraved words (tempo marks, lyrics)
      // render as words instead of the framework's box glyphs.
      textFontFamily: 'Roboto',
      document: document,
      systems: FakeNotationEngine().layout(document, 600),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: CustomPaint(
              painter: painter,
              size: Size(600, painter.heightFor(600)),
            ),
          ),
        ),
      ),
    );
    await expectLater(
      find.byType(CustomPaint).first,
      matchesGoldenFile('goldens/partition_tie_slur.png'),
    );
  }, tags: 'golden');

  testWidgets('partition clef-change golden', (tester) async {
    final document = sampleClefChangeDocument();
    // Both measures in one system so the mid-system clef change is visible.
    final painter = PartitionPainter(
      // Name the loaded test face so engraved words (tempo marks, lyrics)
      // render as words instead of the framework's box glyphs.
      textFontFamily: 'Roboto',
      document: document,
      systems: [
        System(measures: Uint32List.fromList([0, 1]), staves: 2),
      ],
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: CustomPaint(
              painter: painter,
              size: Size(600, painter.heightFor(600)),
            ),
          ),
        ),
      ),
    );
    await expectLater(
      find.byType(CustomPaint).first,
      matchesGoldenFile('goldens/partition_clef_change.png'),
    );
  }, tags: 'golden');

  testWidgets('partition beamed golden', (tester) async {
    final document = sampleBeamedDocument();
    final painter = PartitionPainter(
      // Name the loaded test face so engraved words (tempo marks, lyrics)
      // render as words instead of the framework's box glyphs.
      textFontFamily: 'Roboto',
      document: document,
      systems: FakeNotationEngine().layout(document, 600),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: CustomPaint(
              painter: painter,
              size: Size(600, painter.heightFor(600)),
            ),
          ),
        ),
      ),
    );
    await expectLater(
      find.byType(CustomPaint).first,
      matchesGoldenFile('goldens/partition_beamed.png'),
    );
  }, tags: 'golden');

  testWidgets('partition golden', (tester) async {
    final document = sampleGrandStaffDocument();
    final painter = PartitionPainter(
      // Name the loaded test face so engraved words (tempo marks, lyrics)
      // render as words instead of the framework's box glyphs.
      textFontFamily: 'Roboto',
      document: document,
      systems: FakeNotationEngine().layout(document, 600),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: CustomPaint(
              painter: painter,
              size: Size(600, painter.heightFor(600)),
            ),
          ),
        ),
      ),
    );
    await expectLater(
      find.byType(CustomPaint).first,
      matchesGoldenFile('goldens/partition.png'),
    );
  }, tags: 'golden');

  // Single-hand collapsed layouts: only the kept staff (treble for Right, bass
  // for Left) is engraved; the other staff and its glyphs are gone.
  for (final (hand, name) in [(Hand.right, 'right'), (Hand.left, 'left')]) {
    testWidgets('partition $name-hand only golden', (tester) async {
      final document = sampleGrandStaffDocument();
      final painter = PartitionPainter(
        // Name the loaded test face so engraved words (tempo marks, lyrics)
        // render as words instead of the framework's box glyphs.
        textFontFamily: 'Roboto',
        document: document,
        systems: FakeNotationEngine().layout(document, 600),
        selectedHands: hand,
      );
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Center(
              child: CustomPaint(
                painter: painter,
                size: Size(600, painter.heightFor(600)),
              ),
            ),
          ),
        ),
      );
      await expectLater(
        find.byType(CustomPaint).first,
        matchesGoldenFile('goldens/partition_${name}_only.png'),
      );
    }, tags: 'golden');
  }
}
