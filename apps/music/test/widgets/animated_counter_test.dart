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
import 'package:music/widgets/animated_counter.dart';

Widget _host(int value, {int? from, bool reduced = false}) => MediaQuery(
  data: MediaQueryData(disableAnimations: reduced),
  child: Directionality(
    textDirection: TextDirection.ltr,
    child: AnimatedCounter(
      value: value,
      initialFrom: from,
      format: (v) => '+$v pts',
    ),
  ),
);

String _shown(WidgetTester tester) =>
    tester.widget<Text>(find.byType(Text)).data!;

void main() {
  group('animated counter', () {
    testWidgets('counts up from its starting point to the value', (
      tester,
    ) async {
      await tester.pumpWidget(_host(100, from: 0));
      expect(_shown(tester), '+0 pts');
      await tester.pump(Motion.celebrate ~/ 2);
      final mid = int.parse(RegExp(r'\d+').firstMatch(_shown(tester))![0]!);
      expect(mid, inExclusiveRange(0, 100));
      await tester.pumpAndSettle();
      expect(_shown(tester), '+100 pts');
    });

    testWidgets('a later rise counts from the previous value', (tester) async {
      await tester.pumpWidget(_host(10));
      expect(_shown(tester), '+10 pts', reason: 'no start → shown at once');
      await tester.pumpWidget(_host(30));
      await tester.pump();
      expect(_shown(tester), '+10 pts');
      await tester.pumpAndSettle();
      expect(_shown(tester), '+30 pts');
    });

    testWidgets('a decrease shows at once — a spend is not a show', (
      tester,
    ) async {
      await tester.pumpWidget(_host(30));
      await tester.pumpWidget(_host(5));
      await tester.pump();
      expect(_shown(tester), '+5 pts');
    });

    testWidgets('reduced motion shows the final value immediately', (
      tester,
    ) async {
      await tester.pumpWidget(_host(100, from: 0, reduced: true));
      await tester.pump();
      expect(_shown(tester), '+100 pts');
    });
  });
}
