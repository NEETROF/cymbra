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
import 'package:music/painters/partition_painter.dart';

import '../support/notation_fakes.dart';

void main() {
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
}
