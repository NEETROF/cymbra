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
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/platform_info.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/app_language.dart';
import 'package:music/state/app_locale.dart';

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';

/// Minimal app whose locale is driven by [appLocaleProvider] — mirrors the real
/// `CymbraApp` wiring so a language change hot-swaps localized strings.
class _MiniApp extends ConsumerWidget {
  const _MiniApp();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp(
      locale: ref.watch(appLocaleProvider),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Builder(
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

  testWidgets('the settings drawer language picker shows flags and selects one', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(1600, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));

    final prefs = FakePreferencesService();
    final midi = FakeMidiService(ports: const ['Piano'], connected: 'Piano');
    final container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
        isAndroidProvider.overrideWithValue(false),
        preferencesServiceProvider.overrideWithValue(prefs),
        deviceLocaleProvider.overrideWithValue(const Locale('en')),
      ],
    );

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp(
          locale: container.read(appLocaleProvider),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: const PlayerScreen(),
        ),
      ),
    );
    await tester.pump();

    // Open the settings drawer and drill into the Language category. The player
    // screen runs a continuous Ticker, so settle with timed pumps rather than
    // pumpAndSettle (which would never converge).
    await tester.tap(find.byTooltip('Settings'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300)); // drawer slide-in
    await tester.tap(find.text('Language'));
    await tester.pump();

    // All four flags are listed, with exactly one marked active (English).
    for (final flag in ['🇬🇧', '🇫🇷', '🇮🇹', '🇪🇸']) {
      expect(find.text(flag), findsOneWidget);
    }
    expect(find.byIcon(Icons.check_circle), findsOneWidget);

    // Selecting Italian updates the state and persists the code.
    await tester.tap(find.text('🇮🇹'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    expect(container.read(appLocaleProvider), const Locale('it'));
    expect(prefs.store[AppLocale.prefsKey], 'it');

    // Unmount and dispose within the body so the player's status timer is
    // cancelled before the test framework's pending-timer check.
    await tester.pumpWidget(const SizedBox());
    await tester.pump();
    container.dispose();
    await midi.close();
  });
}
