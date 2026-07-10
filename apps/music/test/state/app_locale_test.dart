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
import 'package:music/services/preferences_service.dart';
import 'package:music/state/app_language.dart';
import 'package:music/state/app_locale.dart';

import '../support/prefs_fakes.dart';

ProviderContainer makeContainer({
  required FakePreferencesService prefs,
  required Locale device,
}) {
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      deviceLocaleProvider.overrideWithValue(device),
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
}
