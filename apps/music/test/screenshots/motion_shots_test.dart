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

/// Goldens for the drawn motion pieces (change:
/// add-music-motion-and-celebrations): the confetti burst at three progress
/// values and the path curve for an empty, a partial and a complete unit.
/// Tagged `golden` so the cross-platform gate skips them; refresh with
///   flutter test --tags golden test/screenshots/motion_shots_test.dart \
///     --update-goldens
@Tags(['golden'])
library;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/confetti_painter.dart';
import 'package:music/painters/path_curve_painter.dart';
import 'package:music/theme/cymbra_theme.dart';

const _tints = [
  CymbraColors.primary,
  CymbraColors.primaryContainer,
  CymbraColors.secondary,
  Color(0xFFF5D0FE),
];

Future<void> _shoot(
  WidgetTester tester,
  String name,
  CustomPainter painter,
) async {
  await tester.pumpWidget(
    Center(
      child: RepaintBoundary(
        key: const Key('shot'),
        child: Container(
          width: 380,
          height: 240,
          color: CymbraColors.surfaceContainerLow,
          child: CustomPaint(painter: painter),
        ),
      ),
    ),
  );
  await expectLater(
    find.byKey(const Key('shot')),
    matchesGoldenFile('shots/$name.png'),
  );
}

void main() {
  group('confetti burst', () {
    for (final p in [0.25, 0.5, 0.75]) {
      testWidgets('golden at progress $p', (tester) async {
        await _shoot(
          tester,
          'confetti-${(p * 100).round()}',
          ConfettiPainter(
            field: ConfettiField(),
            progress: p,
            colors: _tints,
            origin: const Offset(190, 70),
          ),
        );
      });
    }
  });

  group('path curve', () {
    const centers = [
      Offset(32, 28),
      Offset(60, 84),
      Offset(88, 140),
      Offset(60, 196),
    ];
    for (final (name, fills) in [
      ('empty', <double>[0, 0, 0]),
      ('partial', <double>[1, 0.5, 0]),
      ('complete', <double>[1, 1, 1]),
    ]) {
      testWidgets('golden for a $name unit', (tester) async {
        await _shoot(
          tester,
          'path-curve-$name',
          PathCurvePainter(centers: centers, fills: fills),
        );
      });
    }
  });
}
