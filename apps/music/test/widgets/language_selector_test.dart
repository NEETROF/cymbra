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
import 'package:music/state/app_locale.dart';
import 'package:music/widgets/language_selector.dart';

import '../support/prefs_fakes.dart';

/// A tiny app that hosts the [LanguageSelectorButton] and re-reads its own
/// locale from the provider, so a selection visibly updates the button too.
class _Host extends ConsumerWidget {
  const _Host();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp(
      locale: ref.watch(appLocaleProvider),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: const Scaffold(
        appBar: null,
        body: Center(child: LanguageSelectorButton()),
      ),
    );
  }
}

void main() {
  testWidgets('button opens a flag dialog and a selection persists', (
    tester,
  ) async {
    final prefs = FakePreferencesService();
    final container = ProviderContainer(
      overrides: [
        preferencesServiceProvider.overrideWithValue(prefs),
        deviceLocaleProvider.overrideWithValue(const Locale('en')),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(container: container, child: const _Host()),
    );
    await tester.pumpAndSettle();

    // Opens via the localized "Language" tooltip.
    await tester.tap(find.byTooltip('Language'));
    await tester.pumpAndSettle();

    // The dialog lists all four flags.
    for (final flag in ['🇬🇧', '🇫🇷', '🇮🇹', '🇪🇸']) {
      expect(find.text(flag), findsWidgets);
    }

    // Selecting Spanish updates state, persists, and closes the dialog.
    await tester.tap(find.text('🇪🇸'));
    await tester.pumpAndSettle();

    expect(container.read(appLocaleProvider), const Locale('es'));
    expect(prefs.store[AppLocale.prefsKey], 'es');
    expect(find.byType(SimpleDialog), findsNothing);
  });
}
