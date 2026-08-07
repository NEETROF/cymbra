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

import 'dart:ui' show Tristate;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/onboarding/language_step_screen.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/state/coaching_notifier.dart';
import 'package:music/widgets/coach_mark.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';

/// Accessibility of the onboarding surfaces (task 6.2): everything is
/// announceable and reachable without relying on one specific gesture.
void main() {
  testWidgets('language choices are announced by name and selection state', (
    tester,
  ) async {
    final handle = tester.ensureSemantics();
    final container = ProviderContainer(
      overrides: [
        preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
        deviceLocaleProvider.overrideWithValue(const Locale('fr')),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(
          const LanguageStepScreen(),
          locale: const Locale('en'),
        ),
      ),
    );
    await tester.pumpAndSettle();

    // The flag alone would be meaningless to a screen reader: each tile carries
    // the language name and whether it is the current choice.
    final french = tester.getSemantics(
      find.byKey(const Key('onboarding-language-fr')),
    );
    expect(french.label, contains('French'));
    expect(french.flagsCollection.isSelected, Tristate.isTrue);
    expect(french.flagsCollection.isButton, isTrue);

    final english = tester.getSemantics(
      find.byKey(const Key('onboarding-language-en')),
    );
    expect(english.label, contains('English'));
    expect(english.flagsCollection.isSelected, Tristate.isFalse);
    handle.dispose();
  });

  testWidgets('a coaching hint is announced and dismissed by a real button', (
    tester,
  ) async {
    final handle = tester.ensureSemantics();
    final prefs = FakePreferencesService();
    final container = ProviderContainer(
      overrides: [preferencesServiceProvider.overrideWithValue(prefs)],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(
          const Scaffold(body: CoachHintCallout(hint: CoachHint.rewards)),
        ),
      ),
    );
    await tester.pumpAndSettle();

    // Announced on appearance rather than silently painted…
    expect(
      find.byWidgetPredicate(
        (w) => w is Semantics && w.properties.liveRegion == true,
      ),
      findsWidgets,
    );
    // …and dismissible by activating a button — no swipe or tap-anywhere needed,
    // so assistive input can get rid of it.
    final dismiss = find.byKey(const Key('coach-hint-dismiss-rewards'));
    expect(tester.getSemantics(dismiss).flagsCollection.isButton, isTrue);
    await tester.tap(dismiss);
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('coach-hint-rewards')), findsNothing);
    handle.dispose();
  });

  testWidgets(
    'the spotlight offers Next/Skip as buttons, not only a scrim tap',
    (tester) async {
      final handle = tester.ensureSemantics();
      await tester.pumpWidget(
        localizedApp(
          Scaffold(
            body: Stack(
              children: [
                Positioned.fill(
                  child: CoachMarkOverlay(
                    hole: const Rect.fromLTWH(80, 100, 200, 60),
                    title: 'Choose your piano sound',
                    body: 'Pick the instrument you hear while playing.',
                    nextLabel: 'Next',
                    skipLabel: 'Skip',
                    onNext: () {},
                    onSkip: () {},
                    passThrough: true,
                  ),
                ),
              ],
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      for (final key in const [
        Key('coach-mark-next'),
        Key('coach-mark-skip'),
      ]) {
        expect(
          tester.getSemantics(find.byKey(key)).flagsCollection.isButton,
          isTrue,
          reason: '\$key is not announced as a button',
        );
      }
      handle.dispose();
    },
  );
}
