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

  testWidgets('partition tie/slur golden', (tester) async {
    final document = sampleTieSlurDocument();
    final painter = PartitionPainter(
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
