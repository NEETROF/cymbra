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
import 'package:music/screens/auth/entry_screen.dart';
import 'package:music/screens/auth/handle_onboarding_screen.dart';
import 'package:music/screens/auth/session_gate.dart';
import 'package:music/screens/library_screen.dart';
import 'package:music/services/token_store.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/session_state.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';
import '../support/localized.dart';

Future<void> _pump(
  WidgetTester tester,
  ProviderContainer c,
  Widget child,
) async {
  await tester.binding.setSurfaceSize(const Size(1200, 900));
  await tester.pumpWidget(
    UncontrolledProviderScope(container: c, child: localizedApp(child)),
  );
}

void main() {
  testWidgets('entry renders the four options', (tester) async {
    final c = authContainer(oidc: FakeOidcTokenSource(appleAvailable: true));
    await _pump(tester, c, const EntryScreen());

    expect(find.byKey(const Key('entry-google')), findsOneWidget);
    expect(find.byKey(const Key('entry-apple')), findsOneWidget);
    expect(find.byKey(const Key('entry-email')), findsOneWidget);
    expect(find.byKey(const Key('entry-guest')), findsOneWidget);
  });

  testWidgets('Apple is hidden where it is not available', (tester) async {
    final c = authContainer(oidc: FakeOidcTokenSource(appleAvailable: false));
    await _pump(tester, c, const EntryScreen());
    expect(find.byKey(const Key('entry-apple')), findsNothing);
  });

  testWidgets('Google is hidden where it is not available', (tester) async {
    // Degrades gracefully on platforms without a google_sign_in impl (e.g.
    // Windows/Linux): the button never renders, so the native plugin is never
    // invoked and cannot crash. Email/guest remain available.
    final c = authContainer(oidc: FakeOidcTokenSource(googleAvailable: false));
    await _pump(tester, c, const EntryScreen());
    expect(find.byKey(const Key('entry-google')), findsNothing);
    expect(find.byKey(const Key('entry-email')), findsOneWidget);
    expect(find.byKey(const Key('entry-guest')), findsOneWidget);
  });

  testWidgets('entry degrades to email + guest with no OIDC providers', (
    tester,
  ) async {
    // Windows/Linux today: neither Google nor Apple is offered.
    final c = authContainer(
      oidc: FakeOidcTokenSource(googleAvailable: false, appleAvailable: false),
    );
    await _pump(tester, c, const EntryScreen());
    expect(find.byKey(const Key('entry-google')), findsNothing);
    expect(find.byKey(const Key('entry-apple')), findsNothing);
    expect(find.byKey(const Key('entry-email')), findsOneWidget);
    expect(find.byKey(const Key('entry-guest')), findsOneWidget);
  });

  testWidgets('entry screen is localized and offers a language switcher', (
    tester,
  ) async {
    final c = authContainer(oidc: FakeOidcTokenSource(appleAvailable: true));
    await tester.binding.setSurfaceSize(const Size(1200, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: c,
        child: localizedApp(const EntryScreen(), locale: const Locale('fr')),
      ),
    );
    await tester.pumpAndSettle();

    // Strings render in French, and the language switcher is present.
    expect(find.text('Continuer sans compte'), findsOneWidget);
    expect(find.text('Continuer avec l\'e-mail'), findsOneWidget);
    expect(find.byTooltip('Langue'), findsOneWidget);
  });

  testWidgets('guest choice persists and enters guest mode', (tester) async {
    final store = FakeTokenStore();
    final c = authContainer(store: store);
    c.read(sessionNotifierProvider);
    await tester.runAsync(() => pumpEventQueue());
    await _pump(tester, c, const EntryScreen());

    await tester.tap(find.byKey(const Key('entry-guest')));
    await tester.pump();
    await tester.pump();

    expect(store.guest, isTrue);
    expect(c.read(sessionNotifierProvider), isA<SessionGuest>());
  });

  group('SessionGate routing (design D2)', () {
    testWidgets('unauthenticated shows the entry screen', (tester) async {
      final c = ProviderContainer(
        overrides: [
          ...authOverrides(store: FakeTokenStore()),
          scoreCatalogProvider.overrideWithValue(const []),
        ],
      );
      addTearDown(c.dispose);
      c.read(sessionNotifierProvider);
      await tester.runAsync(() => pumpEventQueue());
      await _pump(tester, c, const SessionGate());
      await tester.pump();

      expect(find.byType(EntryScreen), findsOneWidget);
    });

    testWidgets('returning guest skips entry and opens the library', (
      tester,
    ) async {
      final c = ProviderContainer(
        overrides: [
          ...authOverrides(store: FakeTokenStore(guest: true)),
          scoreCatalogProvider.overrideWithValue(const []),
        ],
      );
      addTearDown(c.dispose);
      c.read(sessionNotifierProvider);
      await tester.runAsync(() => pumpEventQueue());
      await _pump(tester, c, const SessionGate());
      await tester.pump();

      expect(find.byType(LibraryScreen), findsOneWidget);
      expect(find.byType(EntryScreen), findsNothing);
    });

    testWidgets('signed-in user without a handle sees onboarding', (
      tester,
    ) async {
      final c = ProviderContainer(
        overrides: [
          ...authOverrides(
            store: FakeTokenStore(
              tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
            ),
            account: FakeAccountService(account: fakeAccount(handle: null)),
          ),
          scoreCatalogProvider.overrideWithValue(const []),
        ],
      );
      addTearDown(c.dispose);
      c.read(sessionNotifierProvider);
      await tester.runAsync(() => pumpEventQueue());
      await _pump(tester, c, const SessionGate());
      await tester.pump();

      expect(find.byType(HandleOnboardingScreen), findsOneWidget);
    });
  });
}
