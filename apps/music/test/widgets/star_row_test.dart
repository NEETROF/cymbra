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
import 'package:music/widgets/star_row.dart';

import '../support/localized.dart';

int _filled(WidgetTester tester) => tester
    .widgetList<Icon>(find.byType(Icon))
    .where((i) => i.icon == Icons.star_rounded)
    .length;

void main() {
  group('star row', () {
    for (final n in [0, 1, 2, 3]) {
      testWidgets('$n of 3 stars filled, announced as such', (tester) async {
        await tester.pumpWidget(
          localizedApp(
            Scaffold(
              body: Center(child: StarRow(stars: n)),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.byType(Icon), findsNWidgets(3));
        expect(_filled(tester), n);
        expect(find.bySemanticsLabel('$n of 3 stars'), findsOneWidget);
      });
    }

    testWidgets('stars gained since before pop in, the others are still', (
      tester,
    ) async {
      await tester.pumpWidget(
        localizedApp(
          const Scaffold(
            body: Center(
              child: StarRow(stars: 3, compact: false, animateFrom: 1),
            ),
          ),
        ),
      );
      await tester.pump();
      // Two Transform.scale wrappers: the second and third star.
      expect(
        find.descendant(
          of: find.byType(StarRow),
          matching: find.byType(Transform),
        ),
        findsNWidgets(2),
      );
      await tester.pumpAndSettle();
      expect(_filled(tester), 3);
    });
  });
}
