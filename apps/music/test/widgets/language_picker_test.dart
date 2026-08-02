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
import 'package:music/services/preferences_service.dart';
import 'package:music/state/app_language.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/widgets/language_selector.dart';

import '../support/prefs_fakes.dart';

/// Minimal app whose locale is driven by [appLocaleProvider] — mirrors the real
/// `CymbraApp` wiring so a language change hot-swaps localized strings.
class _MiniApp extends ConsumerWidget {
  const _MiniApp({this.home});

  final Widget? home;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp(
      locale: ref.watch(appLocaleProvider),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home:
          home ??
          Builder(
            builder: (context) => Text(AppLocalizations.of(context).settings),
          ),
    );
  }
}

void main() {
  testWidgets('changing the language hot-swaps localized strings', (
    tester,
  ) async {
    final container = ProviderContainer(
      overrides: [
        preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
        deviceLocaleProvider.overrideWithValue(const Locale('en')),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(container: container, child: const _MiniApp()),
    );
    await tester.pumpAndSettle();

    // English by default.
    expect(find.text('Settings'), findsOneWidget);
    expect(find.text('Réglages'), findsNothing);

    // Switch to French — no restart, the visible string updates.
    await container.read(appLocaleProvider.notifier).select(AppLanguage.fr);
    await tester.pumpAndSettle();

    expect(find.text('Réglages'), findsOneWidget);
    expect(find.text('Settings'), findsNothing);
  });

  testWidgets('the language selector button shows flags and selects one', (
    tester,
  ) async {
    // Language now lives outside the player — in the LanguageSelectorButton on the
    // library/entry screens (the in-game settings no longer carry a language menu).
    final prefs = FakePreferencesService();
    final container = ProviderContainer(
      overrides: [
        preferencesServiceProvider.overrideWithValue(prefs),
        deviceLocaleProvider.overrideWithValue(const Locale('en')),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const _MiniApp(
          home: Scaffold(body: Center(child: LanguageSelectorButton())),
        ),
      ),
    );
    await tester.pumpAndSettle();

    // Open the language dialog.
    await tester.tap(find.byType(LanguageSelectorButton));
    await tester.pumpAndSettle();

    // All four flags are listed in the dialog, with exactly one marked active
    // (English). Scope to the dialog since the button itself shows the active flag.
    final dialog = find.byType(SimpleDialog);
    for (final flag in ['🇬🇧', '🇫🇷', '🇮🇹', '🇪🇸']) {
      expect(
        find.descendant(of: dialog, matching: find.text(flag)),
        findsOneWidget,
      );
    }
    expect(find.byIcon(Icons.check_circle), findsOneWidget);

    // Selecting Italian updates the state and persists the code.
    await tester.tap(find.text('🇮🇹'));
    await tester.pumpAndSettle();

    expect(container.read(appLocaleProvider), const Locale('it'));
    expect(prefs.store[AppLocale.prefsKey], 'it');
  });
}
