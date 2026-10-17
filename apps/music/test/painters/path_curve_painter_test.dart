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

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/path_curve_painter.dart';

const _centers = [Offset(32, 28), Offset(60, 84), Offset(88, 140)];

List<Paint> _strokes(PathCurvePainter painter) {
  final canvas = TestRecordingCanvas();
  painter.paint(canvas, const Size(300, 200));
  return canvas.invocations
      .map((i) => i.invocation)
      .where((i) => i.memberName == #drawPath)
      .map((i) => i.positionalArguments[1] as Paint)
      .toList();
}

void main() {
  group('path curve painter', () {
    test('a segment runs from one centre to the next', () {
      final bounds = PathCurvePainter.segment(
        _centers[0],
        _centers[1],
      ).getBounds();
      expect(bounds.top, _centers[0].dy);
      expect(bounds.bottom, _centers[1].dy);
      expect(bounds.left, _centers[0].dx);
      expect(bounds.right, _centers[1].dx);
    });

    test('an unfilled path is all dashes, a filled one all ribbon', () {
      final dashed = _strokes(
        const PathCurvePainter(centers: _centers, fills: [0, 0]),
      );
      expect(dashed, isNotEmpty);
      expect(dashed.every((p) => p.strokeWidth == 3), isTrue);

      final filled = _strokes(
        const PathCurvePainter(centers: _centers, fills: [1, 1]),
      );
      expect(filled, hasLength(2), reason: 'one ribbon per segment, no dash');
      expect(filled.every((p) => p.strokeWidth == 4), isTrue);
    });

    test('a half-filled segment is ribbon then dashes', () {
      final strokes = _strokes(
        const PathCurvePainter(centers: _centers, fills: [0.5]),
      );
      expect(strokes.first.strokeWidth, 4);
      expect(strokes.where((p) => p.strokeWidth == 3), isNotEmpty);
    });

    test('a missing fill reads as unfilled, never throws', () {
      expect(
        () => _strokes(const PathCurvePainter(centers: _centers, fills: [])),
        returnsNormally,
      );
    });

    test('repaints when a fill or a centre moves, not otherwise', () {
      const a = PathCurvePainter(centers: _centers, fills: [0, 0]);
      const b = PathCurvePainter(centers: _centers, fills: [0.5, 0]);
      const c = PathCurvePainter(centers: _centers, fills: [0.5, 0]);
      expect(b.shouldRepaint(a), isTrue);
      expect(c.shouldRepaint(b), isFalse);
      expect(
        const PathCurvePainter(
          centers: [Offset(0, 0), Offset(1, 1)],
          fills: [0.5, 0],
        ).shouldRepaint(b),
        isTrue,
      );
    });
  });
}
