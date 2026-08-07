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
import 'package:music/theme/cymbra_theme.dart';
import 'package:music/widgets/coach_mark.dart';

/// Golden coverage of the spotlight visuals (change: add-welcome-onboarding,
/// task 7.2): the scrim cut-out and where the bubble lands — below a high
/// target, above a low one, and beside it when the landscape viewport is too
/// short for either band. Tagged `golden`, so it stays out of the
/// cross-platform gate.
Widget _host(Rect? hole, Size size) => MaterialApp(
  debugShowCheckedModeBanner: false,
  theme: buildCymbraTheme(),
  home: RepaintBoundary(
    key: const Key('golden'),
    child: SizedBox.fromSize(
      size: size,
      child: Stack(
        children: [
          // A plain backdrop, so the golden shows the scrim and the cut-out.
          const Positioned.fill(
            child: ColoredBox(color: CymbraColors.surfaceContainerLow),
          ),
          Positioned.fill(
            child: CoachMarkOverlay(
              hole: hole,
              title: 'Choose your piano sound',
              body: 'Pick the instrument you hear while playing.',
              nextLabel: 'Next',
              skipLabel: 'Skip',
              stepLabel: '1 / 3',
              onNext: _noop,
              onSkip: _noop,
              passThrough: true,
            ),
          ),
        ],
      ),
    ),
  ),
);

Future<void> _pump(WidgetTester tester, Rect? hole, Size size) async {
  await tester.binding.setSurfaceSize(size);
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(_host(hole, size));
  await tester.pumpAndSettle();
}

void main() {
  group('coach-mark spotlight', () {
    testWidgets('bubble below a high target', tags: 'golden', (tester) async {
      await _pump(
        tester,
        const Rect.fromLTWH(120, 90, 260, 56),
        const Size(900, 640),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/coach_mark_below.png'),
      );
    });

    testWidgets('bubble above a low target', tags: 'golden', (tester) async {
      await _pump(
        tester,
        const Rect.fromLTWH(120, 470, 260, 56),
        const Size(900, 640),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/coach_mark_above.png'),
      );
    });

    testWidgets('bubble beside the target in short landscape', tags: 'golden', (
      tester,
    ) async {
      await _pump(
        tester,
        const Rect.fromLTWH(90, 140, 240, 90),
        const Size(880, 360),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/coach_mark_landscape.png'),
      );
    });

    testWidgets('untargeted hint (no cut-out)', tags: 'golden', (tester) async {
      await _pump(tester, null, const Size(900, 640));
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/coach_mark_untargeted.png'),
      );
    });
  });
}

void _noop() {}
