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
import 'package:music/state/onboarding_notifier.dart';

import '../support/prefs_fakes.dart';

ProviderContainer _container(FakePreferencesService prefs) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      deviceLocaleProvider.overrideWithValue(const Locale('en')),
    ],
  );
  // Subscribe to both notifiers so each builds (and restores from storage) now,
  // exactly like the real app: `MaterialApp` watches the locale at startup and
  // the gate watches the onboarding flags.
  final sub = c.listen(onboardingProvider, (_, _) {});
  final localeSub = c.listen(appLocaleProvider, (_, _) {});
  addTearDown(localeSub.close);
  addTearDown(sub.close);
  addTearDown(c.dispose);
  return c;
}

Future<void> _flush() => Future<void>.delayed(Duration.zero);

void main() {
  test('a first launch needs the language step, then the welcome', () async {
    final c = _container(FakePreferencesService());
    // Nothing is decided until the flags are read.
    expect(c.read(onboardingProvider).loaded, isFalse);
    expect(c.read(onboardingProvider).needsLanguage, isFalse);

    await _flush();
    expect(c.read(onboardingProvider).needsLanguage, isTrue);
    // The welcome comes *after* the language step, never before it.
    expect(c.read(onboardingProvider).needsWelcome, isFalse);
  });

  test('choosing a language applies it, persists it, and advances', () async {
    final prefs = FakePreferencesService();
    final c = _container(prefs);
    await _flush();

    await c.read(onboardingProvider.notifier).chooseLanguage(AppLanguage.fr);

    // Applied immediately, so the welcome that follows is already localized.
    expect(c.read(appLocaleProvider), const Locale('fr'));
    expect(prefs.store[AppLocale.prefsKey], 'fr');
    expect(prefs.store[Onboarding.languagePrefsKey], 'true');
    expect(c.read(onboardingProvider).needsLanguage, isFalse);
    expect(c.read(onboardingProvider).needsWelcome, isTrue);
  });

  test('completing the welcome ends the first-run flow for good', () async {
    final prefs = FakePreferencesService();
    final c = _container(prefs);
    await _flush();
    await c.read(onboardingProvider.notifier).chooseLanguage(AppLanguage.en);

    await c.read(onboardingProvider.notifier).completeWelcome();

    expect(c.read(onboardingProvider).needsWelcome, isFalse);
    expect(prefs.store[Onboarding.welcomePrefsKey], 'true');
  });

  test('a returning user sees neither step', () async {
    final c = _container(
      FakePreferencesService({
        Onboarding.languagePrefsKey: 'true',
        Onboarding.welcomePrefsKey: 'true',
      }),
    );
    await _flush();
    expect(c.read(onboardingProvider).needsLanguage, isFalse);
    expect(c.read(onboardingProvider).needsWelcome, isFalse);
  });

  test(
    'an upgrade with a language already chosen is not asked again',
    () async {
      // The device went through the picker before this flow existed.
      final c = _container(FakePreferencesService({AppLocale.prefsKey: 'it'}));
      await _flush();
      expect(c.read(onboardingProvider).needsLanguage, isFalse);
      expect(c.read(onboardingProvider).needsWelcome, isTrue);
    },
  );

  test('the try-run marker is set and cleared by the welcome', () async {
    final c = _container(FakePreferencesService());
    await _flush();
    expect(c.read(onboardingProvider).tryRunFinished, isFalse);

    c.read(onboardingProvider.notifier).markTryRunFinished();
    expect(c.read(onboardingProvider).tryRunFinished, isTrue);

    c.read(onboardingProvider.notifier).clearTryRun();
    expect(c.read(onboardingProvider).tryRunFinished, isFalse);
  });

  test('unusable storage never traps the user in the first-run flow', () async {
    final c = _container(_BrokenPreferences());
    await _flush();
    expect(c.read(onboardingProvider).loaded, isTrue);
    expect(c.read(onboardingProvider).needsLanguage, isFalse);
    expect(c.read(onboardingProvider).needsWelcome, isFalse);
  });
}

/// Preferences that always fail — models a device where storage is unavailable.
class _BrokenPreferences extends FakePreferencesService {
  @override
  Future<String?> getString(String key) async =>
      throw StateError('unavailable');

  @override
  Future<void> setString(String key, String value) async =>
      throw StateError('unavailable');
}
