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
import 'package:music/main.dart';
import 'package:music/screens/auth/entry_screen.dart';
import 'package:music/screens/library_screen.dart';
import 'package:music/screens/onboarding/language_step_screen.dart';
import 'package:music/screens/onboarding/onboarding_gate.dart';
import 'package:music/screens/onboarding/welcome_screen.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/oidc_token_source.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/services/token_store.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/state/onboarding_notifier.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/session_state.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';
import '../support/fakes.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

ProviderContainer _container({
  required FakePreferencesService prefs,
  Locale device = const Locale('en'),
  FakeTokenStore? store,
}) {
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      deviceLocaleProvider.overrideWithValue(device),
      tokenStoreProvider.overrideWithValue(store ?? FakeTokenStore()),
      authServiceProvider.overrideWithValue(FakeAuthService()),
      accountServiceProvider.overrideWithValue(FakeAccountService()),
      oidcTokenSourceProvider.overrideWithValue(FakeOidcTokenSource()),
      curatorRewardsServiceProvider.overrideWithValue(
        const FakeCuratorRewardsService(),
      ),
      // The no-account try plays a bundled score through the real player, with
      // the native seams faked.
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
    ],
  );
  addTearDown(container.dispose);
  return container;
}

/// The real app wiring in miniature: the locale drives `MaterialApp`, whose home
/// is the first-run gate.
class _App extends ConsumerWidget {
  const _App();

  @override
  Widget build(BuildContext context, WidgetRef ref) => MaterialApp(
    locale: ref.watch(appLocaleProvider),
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    home: const OnboardingGate(),
  );
}

Future<void> _pump(WidgetTester tester, ProviderContainer container) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    UncontrolledProviderScope(container: container, child: const _App()),
  );
  await tester.pumpAndSettle();
}

/// Pumps a fixed number of frames. The player drives a continuous ticker, so
/// once it is on screen the tree never "settles".
Future<void> _frames(WidgetTester tester, [int count = 10]) async {
  for (var i = 0; i < count; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

/// Walks the welcome to its last page, where the first action lives.
Future<void> _toLastWelcomePage(WidgetTester tester) async {
  for (var i = 0; i < 2; i++) {
    await tester.tap(find.byKey(const Key('welcome-next')));
    await tester.pumpAndSettle();
  }
}

void main() {
  testWidgets('the language step comes first, defaulted to the device locale', (
    tester,
  ) async {
    final container = _container(
      prefs: FakePreferencesService(),
      device: const Locale('fr'),
    );
    await _pump(tester, container);

    // First surface of a first launch — before the welcome and any sign-in.
    expect(find.byType(LanguageStepScreen), findsOneWidget);
    expect(find.byType(WelcomeScreen), findsNothing);
    expect(find.byType(EntryScreen), findsNothing);
    // The device language is pre-applied, so the step itself is already in it.
    expect(container.read(appLocaleProvider), const Locale('fr'));
    expect(find.text('Choisissez votre langue'), findsOneWidget);
  });

  testWidgets('choosing a language leads to the welcome, with no account', (
    tester,
  ) async {
    final prefs = FakePreferencesService();
    final container = _container(prefs: prefs);
    await _pump(tester, container);

    // Pick a different language than the default, then continue.
    await tester.tap(find.byKey(const Key('onboarding-language-it')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('onboarding-language-continue')));
    await tester.pumpAndSettle();

    expect(container.read(appLocaleProvider), const Locale('it'));
    expect(find.byType(WelcomeScreen), findsOneWidget);
    // No account was needed to get here, and none is being asked for.
    expect(
      container.read(sessionNotifierProvider),
      isA<SessionUnauthenticated>(),
    );
    expect(find.byType(EntryScreen), findsNothing);
  });

  testWidgets('the welcome is skippable and never forces sign-up', (
    tester,
  ) async {
    final prefs = FakePreferencesService({Onboarding.languagePrefsKey: 'true'});
    final container = _container(prefs: prefs);
    await _pump(tester, container);
    expect(find.byType(WelcomeScreen), findsOneWidget);

    await tester.tap(find.byKey(const Key('welcome-skip')));
    await tester.pumpAndSettle();

    // Skipping lands on the entry screen — which still offers continuing
    // without an account, so nothing is gated behind a mandatory sign-up.
    expect(find.byType(WelcomeScreen), findsNothing);
    expect(find.byKey(const Key('entry-guest')), findsOneWidget);
    expect(prefs.store[Onboarding.welcomePrefsKey], 'true');
  });

  testWidgets('the shipped CymbraApp opens on the first-run step', (
    tester,
  ) async {
    // Guards what the (slow, CI-only) integration test drives: the *shipped*
    // root widget — gate, coach layer and all — not the gate in isolation. A
    // resumed guest session does not skip the language step, which is why the
    // integration test has to walk it.
    await tester.binding.setSurfaceSize(const Size(1400, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    final container = _container(
      prefs: FakePreferencesService(),
      store: FakeTokenStore(guest: true),
    );

    await tester.pumpWidget(
      UncontrolledProviderScope(container: container, child: const CymbraApp()),
    );
    await tester.pumpAndSettle();

    expect(find.byType(LanguageStepScreen), findsOneWidget);
    expect(find.byType(LibraryScreen), findsNothing);

    // Continuing hands over to the session routing: a guest skips the welcome.
    await tester.tap(find.byKey(const Key('onboarding-language-continue')));
    await tester.pumpAndSettle();
    expect(find.byType(WelcomeScreen), findsNothing);
    expect(find.byType(LibraryScreen), findsOneWidget);

    // Unmount here: the library warms providers that own timers, cancelled on
    // container dispose — which the binding checks before tear-downs run.
    await tester.pumpWidget(const SizedBox());
    await tester.pump();
    container.dispose();
  });

  testWidgets('a returning user sees neither first-run step', (tester) async {
    final container = _container(
      prefs: FakePreferencesService({
        Onboarding.languagePrefsKey: 'true',
        Onboarding.welcomePrefsKey: 'true',
      }),
      store: FakeTokenStore(guest: true),
    );
    await _pump(tester, container);

    expect(find.byType(LanguageStepScreen), findsNothing);
    expect(find.byType(WelcomeScreen), findsNothing);
    expect(find.byType(LibraryScreen), findsOneWidget);
  });

  testWidgets('the try plays a bundled score without signing in, then offers '
      'sign-in after the summary', (tester) async {
    final container = _container(
      prefs: FakePreferencesService({Onboarding.languagePrefsKey: 'true'}),
    );
    await _pump(tester, container);
    await _toLastWelcomePage(tester);

    // Pick one of the included scores.
    await tester.tap(find.byKey(const Key('welcome-try')));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('welcome-try-picker')), findsOneWidget);
    await tester.tap(find.byKey(const Key('welcome-try-sample')));
    // The player runs a ticker, so the tree never "settles" — pump frames.
    await _frames(tester, 20);

    // The real player opened, with no account in sight.
    expect(find.byType(PlayerScreen), findsOneWidget);
    expect(
      container.read(sessionNotifierProvider),
      isA<SessionUnauthenticated>(),
    );
    expect(find.byType(EntryScreen), findsNothing);

    // Close the pre-play setup, then play a scored run to its summary.
    await tester.tap(find.byIcon(Icons.close));
    await _frames(tester);
    container.read(performanceScorerProvider.notifier)
      ..startRun(
        pieceId: _entry.id,
        title: _entry.title,
        hands: 'both',
        speed: 1,
        notes: const <TimedNote>[],
      )
      ..finishRun(1000, waitMode: false);
    await _frames(tester, 20);
    // The end-of-session summary is up (the pre-play modal is already closed,
    // so this dialog and its close control are the summary's).
    expect(find.text('Overall sync'), findsOneWidget);

    // Leaving the summary quits the player and returns to the welcome.
    await tester.tap(find.byIcon(Icons.close));
    await _frames(tester, 30);

    // Sign-in is *offered* there, with its benefit named.
    expect(find.byKey(const Key('sign-in-invitation')), findsOneWidget);
    expect(
      find.text('Sign in to keep your progress and build your library.'),
      findsOneWidget,
    );

    // Declining returns to the welcome — no dead end, still no account.
    await tester.tap(find.byKey(const Key('sign-in-invitation-decline')));
    await tester.pumpAndSettle();
    expect(find.byType(WelcomeScreen), findsOneWidget);
    expect(
      container.read(sessionNotifierProvider),
      isA<SessionUnauthenticated>(),
    );
  });
}
