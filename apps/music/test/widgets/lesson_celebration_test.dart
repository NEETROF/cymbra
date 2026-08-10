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
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/widgets/lesson_celebration.dart';

Future<void> _open(WidgetTester tester) async {
  await tester.pumpWidget(
    MaterialApp(
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      locale: const Locale('fr'),
      home: Builder(
        builder: (context) => Scaffold(
          body: Center(
            child: FilledButton(
              key: const Key('go'),
              onPressed: () => showLessonCelebration(
                context,
                lessonTitle: 'Majeur ou mineur',
                flawless: 3,
                gated: 4,
                nextLessonTitle: 'Do, fa, sol : les trois piliers',
              ),
              child: const Text('go'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.byKey(const Key('go')));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('never overflows on a short landscape-phone viewport', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(852, 393); // iPhone landscape pts
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await _open(tester);
    // The dialog scrolls instead of overflowing.
    expect(tester.takeException(), isNull);
    expect(find.text('Leçon terminée !'), findsOneWidget);
    expect(find.text('3 sur 4 du premier coup'), findsOneWidget);
    // Both actions stay reachable (the close is scrolled to if needed).
    await tester.scrollUntilVisible(
      find.byKey(const Key('lesson-celebration-close')),
      60,
    );
    expect(find.byKey(const Key('lesson-celebration-close')), findsOneWidget);
  });
}
