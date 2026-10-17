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
import 'package:music/theme/motion.dart';

Future<BuildContext> _context(
  WidgetTester tester, {
  bool reduced = false,
}) async {
  late BuildContext ctx;
  await tester.pumpWidget(
    MediaQuery(
      data: MediaQueryData(disableAnimations: reduced),
      child: Builder(
        builder: (c) {
          ctx = c;
          return const SizedBox();
        },
      ),
    ),
  );
  return ctx;
}

void main() {
  group('motion tokens', () {
    test('three durations, short to long', () {
      expect(Motion.quick < Motion.standard, isTrue);
      expect(Motion.standard < Motion.celebrate, isTrue);
      expect(Motion.celebrate, const Duration(milliseconds: 900));
    });

    testWidgets('a duration is honoured when motion is not reduced', (
      tester,
    ) async {
      final ctx = await _context(tester);
      expect(Motion.reduced(ctx), isFalse);
      expect(Motion.of(ctx, Motion.standard), Motion.standard);
    });

    testWidgets('every duration collapses to zero under reduced motion', (
      tester,
    ) async {
      final ctx = await _context(tester, reduced: true);
      expect(Motion.reduced(ctx), isTrue);
      expect(Motion.of(ctx, Motion.quick), Duration.zero);
      expect(Motion.of(ctx, Motion.celebrate), Duration.zero);
    });
  });
}
