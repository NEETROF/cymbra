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

import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/account_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/app_language.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/state/language_sync_listener.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/session_state.dart';

import '../support/auth_fakes.dart';
import '../support/prefs_fakes.dart';

/// A drivable [SessionNotifier]: skips the real hydrate and lets a test emit
/// session states so the listener's `ref.listen` fires.
class _TestSessionNotifier extends SessionNotifier {
  @override
  SessionState build() => const SessionState.unauthenticated();

  void emit(SessionState next) => state = next;
}

Account _acct(String userId, {String? locale}) =>
    Account(userId: userId, version: 1, handle: 'h', locale: locale);

void main() {
  testWidgets(
    'reconciles a displayable server locale into the UI on sign-in, once per user',
    (tester) async {
      final prefs = FakePreferencesService();
      final container = ProviderContainer(
        overrides: [
          preferencesServiceProvider.overrideWithValue(prefs),
          deviceLocaleProvider.overrideWithValue(const Locale('en')),
          canUseOnlineServicesProvider.overrideWithValue(true),
          accountServiceProvider.overrideWithValue(FakeAccountService()),
          sessionNotifierProvider.overrideWith(_TestSessionNotifier.new),
        ],
      );
      addTearDown(container.dispose);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: const LanguageSyncListener(child: SizedBox()),
        ),
      );
      // Pre-warm the locale provider so its async restore settles before sign-in
      // (mirrors real startup, where the locale resolves long before login).
      container.listen(appLocaleProvider, (_, _) {}, fireImmediately: true);
      await tester.pumpAndSettle();
      final session =
          container.read(sessionNotifierProvider.notifier)
              as _TestSessionNotifier;

      // Sign in as u1 whose account language is French → UI switches to French.
      session.emit(
        SessionState.authenticated(account: _acct('u1', locale: 'fr')),
      );
      await tester.pumpAndSettle();
      expect(container.read(appLocaleProvider), const Locale('fr'));

      // The user then switches to English locally.
      await container.read(appLocaleProvider.notifier).select(AppLanguage.en);
      expect(container.read(appLocaleProvider), const Locale('en'));

      // Re-resolving the SAME account (e.g. after handle onboarding) must NOT
      // re-reconcile — the UI stays on the user's later local choice.
      session.emit(
        SessionState.authenticated(account: _acct('u1', locale: 'fr')),
      );
      await tester.pumpAndSettle();
      expect(container.read(appLocaleProvider), const Locale('en'));

      // A different user (u2 = Italian) signs in → reconciles again.
      session.emit(const SessionState.unauthenticated());
      session.emit(
        SessionState.authenticated(account: _acct('u2', locale: 'it')),
      );
      await tester.pumpAndSettle();
      expect(container.read(appLocaleProvider), const Locale('it'));
    },
  );

  testWidgets('does not reconcile before authentication', (tester) async {
    final account = FakeAccountService();
    final container = ProviderContainer(
      overrides: [
        preferencesServiceProvider.overrideWithValue(
          FakePreferencesService({AppLocale.prefsKey: 'fr'}),
        ),
        deviceLocaleProvider.overrideWithValue(const Locale('en')),
        canUseOnlineServicesProvider.overrideWithValue(false),
        accountServiceProvider.overrideWithValue(account),
        sessionNotifierProvider.overrideWith(_TestSessionNotifier.new),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const LanguageSyncListener(child: SizedBox()),
      ),
    );
    container.listen(appLocaleProvider, (_, _) {}, fireImmediately: true);
    await tester.pumpAndSettle();

    // Still unauthenticated: the persisted choice stands and the account service
    // is never consulted.
    expect(container.read(appLocaleProvider), const Locale('fr'));
    expect(account.calls, isEmpty);
  });
}
