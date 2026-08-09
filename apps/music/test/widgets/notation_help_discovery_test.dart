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
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/notation/notation_help_content.dart';
import 'package:music/screens/notation_glossary_screen.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/coaching_notifier.dart';
import 'package:music/widgets/coach_mark.dart';

import '../support/prefs_fakes.dart';

Future<void> _pumpApp(
  WidgetTester tester,
  Widget home,
  FakePreferencesService prefs,
) {
  final container = ProviderContainer(
    overrides: [preferencesServiceProvider.overrideWithValue(prefs)],
  );
  addTearDown(container.dispose);
  return tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        locale: const Locale('en'),
        home: home,
      ),
    ),
  );
}

void main() {
  testWidgets('the notation-help hint shows once, then not again', (
    tester,
  ) async {
    final prefs = FakePreferencesService();
    await _pumpApp(
      tester,
      const Scaffold(body: CoachHintCallout(hint: CoachHint.notationHelp)),
      prefs,
    );
    // Let the coaching "seen" flags restore.
    await tester.pumpAndSettle();

    expect(find.text('Tap and hold a symbol for help'), findsOneWidget);

    // Dismiss it.
    await tester.tap(find.byKey(const Key('coach-hint-dismiss-notationHelp')));
    await tester.pumpAndSettle();
    expect(find.text('Tap and hold a symbol for help'), findsNothing);
    expect(prefs.store['coach_notation_help_seen'], 'true');
  });

  testWidgets('an already-seen hint does not appear', (tester) async {
    final prefs = FakePreferencesService({'coach_notation_help_seen': 'true'});
    await _pumpApp(
      tester,
      const Scaffold(body: CoachHintCallout(hint: CoachHint.notationHelp)),
      prefs,
    );
    await tester.pumpAndSettle();
    expect(find.text('Tap and hold a symbol for help'), findsNothing);
  });

  testWidgets('the glossary lists every symbol kind with its help', (
    tester,
  ) async {
    // A tall surface so the whole list builds (no lazy off-screen rows to miss).
    tester.view.physicalSize = const Size(1000, 6000);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    final prefs = FakePreferencesService();
    await _pumpApp(tester, const NotationGlossaryScreen(), prefs);
    await tester.pumpAndSettle();

    final l10n = await AppLocalizations.delegate.load(const Locale('en'));

    // Every glossary sample's title is rendered, and it matches the exact copy
    // the on-staff bubble would show (same lookup) — so glossary and bubbles
    // can't diverge.
    for (final sample in notationGlossarySamples) {
      final help = notationHelpFor(
        l10n,
        sample,
        solfege: false,
        frenchRe: false,
      );
      expect(
        find.text(help.title),
        findsWidgets,
        reason: 'missing glossary row: ${help.title}',
      );
    }
  });
}
