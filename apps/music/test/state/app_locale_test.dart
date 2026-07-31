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

import 'dart:ui' show Locale;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/account_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/app_language.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/state/session_notifier.dart';

import '../support/auth_fakes.dart';
import '../support/prefs_fakes.dart';

ProviderContainer makeContainer({
  required FakePreferencesService prefs,
  required Locale device,
  // Defaults model a signed-out session: no account push happens.
  bool online = false,
  FakeAccountService? account,
}) {
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      deviceLocaleProvider.overrideWithValue(device),
      // Override the derived bool directly (the established test seam) so the real
      // session graph is never built.
      canUseOnlineServicesProvider.overrideWithValue(online),
      accountServiceProvider.overrideWithValue(
        account ?? FakeAccountService(),
      ),
    ],
  );
  addTearDown(container.dispose);
  container.listen(appLocaleProvider, (_, _) {}, fireImmediately: true);
  return container;
}

void main() {
  test('seeds synchronously from the device locale before restore', () {
    final container = makeContainer(
      prefs: FakePreferencesService(),
      device: const Locale('it'),
    );
    // Read before pumping the event queue: the async restore has not run yet.
    expect(container.read(appLocaleProvider), const Locale('it'));
  });

  test('an unsupported device locale seeds English', () {
    final container = makeContainer(
      prefs: FakePreferencesService(),
      device: const Locale('de'),
    );
    expect(container.read(appLocaleProvider), const Locale('en'));
  });

  test(
    'a persisted choice is restored and wins over the device locale',
    () async {
      final container = makeContainer(
        prefs: FakePreferencesService({AppLocale.prefsKey: 'es'}),
        device: const Locale('fr'),
      );
      await pumpEventQueue();
      expect(container.read(appLocaleProvider), const Locale('es'));
    },
  );

  test('selecting a language switches state and persists the code', () async {
    final prefs = FakePreferencesService();
    final container = makeContainer(prefs: prefs, device: const Locale('en'));
    await pumpEventQueue();

    await container.read(appLocaleProvider.notifier).select(AppLanguage.fr);

    expect(container.read(appLocaleProvider), const Locale('fr'));
    expect(prefs.store[AppLocale.prefsKey], 'fr');
  });

  test(
    'an unsupported persisted choice falls back and is re-persisted',
    () async {
      final prefs = FakePreferencesService({AppLocale.prefsKey: 'de'});
      final container = makeContainer(prefs: prefs, device: const Locale('it'));
      await pumpEventQueue();

      // Falls back to the (supported) device language...
      expect(container.read(appLocaleProvider), const Locale('it'));
      // ...and storage self-heals to that fallback.
      expect(prefs.store[AppLocale.prefsKey], 'it');
    },
  );

  // --- Account sync (change: sync-account-language-preference) ---------------

  test('select pushes the choice to the account when signed in', () async {
    final account = FakeAccountService();
    final container = makeContainer(
      prefs: FakePreferencesService(),
      device: const Locale('en'),
      online: true,
      account: account,
    );
    await pumpEventQueue();

    await container.read(appLocaleProvider.notifier).select(AppLanguage.fr);

    expect(container.read(appLocaleProvider), const Locale('fr'));
    expect(account.lastSetLocale, 'fr'); // recorded on the account
  });

  test('select stays local and pushes nothing when signed out', () async {
    final account = FakeAccountService();
    final prefs = FakePreferencesService();
    final container = makeContainer(
      prefs: prefs,
      device: const Locale('en'),
      // online defaults to false (signed out)
      account: account,
    );
    await pumpEventQueue();

    await container.read(appLocaleProvider.notifier).select(AppLanguage.fr);

    expect(container.read(appLocaleProvider), const Locale('fr'));
    expect(prefs.store[AppLocale.prefsKey], 'fr'); // persisted locally
    expect(account.lastSetLocale, isNull); // but never pushed
    expect(account.calls, isEmpty);
  });

  test(
    'syncOnLogin: a displayable server locale wins over the local choice '
    'and is persisted without echoing a push',
    () async {
      final account = FakeAccountService();
      final prefs = FakePreferencesService({AppLocale.prefsKey: 'en'});
      final container = makeContainer(
        prefs: prefs,
        device: const Locale('en'),
        online: true,
        account: account,
      );
      await pumpEventQueue();

      await container.read(appLocaleProvider.notifier).syncOnLogin('fr');

      expect(container.read(appLocaleProvider), const Locale('fr'));
      expect(prefs.store[AppLocale.prefsKey], 'fr'); // persisted locally
      expect(account.lastSetLocale, isNull); // server-wins path never pushes
    },
  );

  test(
    'syncOnLogin: an unset server locale keeps the local choice and pushes it up',
    () async {
      final account = FakeAccountService();
      final container = makeContainer(
        prefs: FakePreferencesService({AppLocale.prefsKey: 'es'}),
        device: const Locale('en'),
        online: true,
        account: account,
      );
      await pumpEventQueue();
      expect(container.read(appLocaleProvider), const Locale('es'));

      await container.read(appLocaleProvider.notifier).syncOnLogin(null);

      // The UI stays on the local choice, which is adopted by the account.
      expect(container.read(appLocaleProvider), const Locale('es'));
      expect(account.lastSetLocale, 'es');
    },
  );

  test(
    'syncOnLogin: an undisplayable server locale leaves the UI and stored value',
    () async {
      final account = FakeAccountService();
      final prefs = FakePreferencesService({AppLocale.prefsKey: 'en'});
      final container = makeContainer(
        prefs: prefs,
        device: const Locale('en'),
        online: true,
        account: account,
      );
      await pumpEventQueue();

      // `de` is not one of en/fr/it/es — the client cannot display it.
      await container.read(appLocaleProvider.notifier).syncOnLogin('de');

      expect(container.read(appLocaleProvider), const Locale('en'));
      expect(prefs.store[AppLocale.prefsKey], 'en'); // unchanged
      expect(account.lastSetLocale, isNull); // stored value untouched
    },
  );

  test('startup performs no pre-auth account read', () async {
    final account = FakeAccountService();
    final container = makeContainer(
      prefs: FakePreferencesService({AppLocale.prefsKey: 'fr'}),
      device: const Locale('en'),
      // online false: not authenticated
      account: account,
    );
    await pumpEventQueue();

    // Resolving the language at startup never touches the account service.
    expect(container.read(appLocaleProvider), const Locale('fr'));
    expect(account.calls, isEmpty);
  });
}
