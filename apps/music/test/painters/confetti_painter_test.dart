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
import 'package:music/painters/confetti_painter.dart';

const _colors = [Colors.red, Colors.green, Colors.blue, Colors.yellow];

/// The drawRRect calls a paint at [progress] records — one per piece drawn.
List<Invocation> _pieces(ConfettiPainter painter) {
  final canvas = TestRecordingCanvas();
  painter.paint(canvas, const Size(380, 300));
  return canvas.invocations
      .map((i) => i.invocation)
      .where((i) => i.memberName == #drawRRect)
      .toList();
}

void main() {
  group('confetti field', () {
    test('a burst has the one tuned particle count by default', () {
      expect(ConfettiField().count, kConfettiParticleCount);
      expect(kConfettiParticleCount, 40);
      expect(ConfettiField(count: 12).count, 12);
    });
  });

  group('confetti painter', () {
    test('draws every piece mid-flight, and nothing before or after', () {
      final field = ConfettiField();
      ConfettiPainter at(double p) =>
          ConfettiPainter(field: field, progress: p, colors: _colors);
      expect(_pieces(at(0.5)), hasLength(kConfettiParticleCount));
      expect(_pieces(at(0)), isEmpty);
      expect(_pieces(at(1)), isEmpty);
    });

    test('the same seed paints the same burst — a golden stays stable', () {
      final a = ConfettiPainter(
        field: ConfettiField(seed: 3),
        progress: 0.4,
        colors: _colors,
      );
      final b = ConfettiPainter(
        field: ConfettiField(seed: 3),
        progress: 0.4,
        colors: _colors,
      );
      final pa = _pieces(a).map((i) => i.positionalArguments.first.toString());
      final pb = _pieces(b).map((i) => i.positionalArguments.first.toString());
      expect(pa, orderedEquals(pb));
    });

    test('repaints only when the progress or the field moves', () {
      final field = ConfettiField();
      final p1 = ConfettiPainter(field: field, progress: 0.2, colors: _colors);
      final p2 = ConfettiPainter(field: field, progress: 0.3, colors: _colors);
      final p3 = ConfettiPainter(field: field, progress: 0.3, colors: _colors);
      expect(p2.shouldRepaint(p1), isTrue);
      expect(p3.shouldRepaint(p2), isFalse);
    });

    testWidgets('paints inside a widget without throwing', (tester) async {
      await tester.pumpWidget(
        SizedBox(
          width: 300,
          height: 300,
          child: CustomPaint(
            painter: ConfettiPainter(
              field: ConfettiField(),
              progress: 0.6,
              colors: _colors,
              origin: const Offset(150, 60),
            ),
          ),
        ),
      );
      expect(tester.takeException(), isNull);
    });
  });
}
