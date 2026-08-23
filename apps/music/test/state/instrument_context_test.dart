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

import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/drums_access.dart';
import 'package:music/state/instrument_context.dart';

import '../support/prefs_fakes.dart';

ProviderContainer _container(
  FakePreferencesService prefs, {
  bool drumsVisible = false,
  Override? drumsOverride,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      drumsOverride ?? drumsEnabledProvider.overrideWithValue(drumsVisible),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

/// Lets the async `_restore` (a couple of awaits deep) complete.
Future<void> _settle() async {
  for (var i = 0; i < 4; i++) {
    await Future<void>.delayed(Duration.zero);
  }
}

void main() {
  test('defaults: keyboard context, choice not yet offered', () async {
    final c = _container(FakePreferencesService());
    final s = c.read(instrumentContextProvider);
    expect(s.context, AppInstrument.keyboard);
    expect(s.choiceOffered, isFalse);
    await _settle(); // an empty store must not disturb the defaults
    expect(c.read(instrumentContextProvider).context, AppInstrument.keyboard);
  });

  test('select persists, and a relaunch restores the stored context', () async {
    final prefs = FakePreferencesService();
    final c = _container(prefs);
    c.read(instrumentContextProvider.notifier).select(AppInstrument.drums);
    await _settle();
    expect(prefs.store, contains(InstrumentContext.prefsKey));

    // "Relaunch": a fresh container over the SAME device store.
    final c2 = _container(prefs);
    c2.read(instrumentContextProvider);
    await _settle();
    expect(c2.read(instrumentContextProvider).context, AppInstrument.drums);
  });

  test('the offered marker survives a sign-out/sign-in cycle', () async {
    // The marker is per-installation: signing out tears the session down but
    // never touches device preferences, so a new session over the same store
    // must NOT re-offer (the naive per-session marker re-prompts every time).
    final prefs = FakePreferencesService();
    final c = _container(prefs);
    c.read(instrumentContextProvider.notifier).markChoiceOffered();
    await _settle();

    final c2 = _container(prefs); // new session, same installation
    c2.read(instrumentContextProvider);
    await _settle();
    expect(c2.read(instrumentContextProvider).choiceOffered, isTrue);
    // And a later choice keeps it set.
    c2.read(instrumentContextProvider.notifier).select(AppInstrument.drums);
    expect(c2.read(instrumentContextProvider).choiceOffered, isTrue);
  });

  test('a corrupt stored value falls back to the defaults', () async {
    final prefs = FakePreferencesService({
      InstrumentContext.prefsKey: 'not json{',
    });
    final c = _container(prefs);
    c.read(instrumentContextProvider);
    await _settle();
    expect(c.read(instrumentContextProvider).context, AppInstrument.keyboard);
    expect(c.read(instrumentContextProvider).choiceOffered, isFalse);
  });

  test('effective context falls back to keyboard while drums are invisible — '
      'presentationally, never rewriting the stored choice', () async {
    // A cold start resolves the flag snapshot async, so EVERY launch passes
    // through the invisible state; the stored value must ride it out.
    final prefs = FakePreferencesService({
      InstrumentContext.prefsKey: jsonEncode({
        'context': 'drums',
        'choiceOffered': true,
      }),
    });
    final c = _container(prefs, drumsVisible: false);
    c.read(instrumentContextProvider);
    await _settle();
    expect(c.read(effectiveInstrumentContextProvider), AppInstrument.keyboard);
    expect(c.read(instrumentContextProvider).context, AppInstrument.drums);
    // The store was not rewritten by the fallback.
    expect(prefs.store[InstrumentContext.prefsKey], contains('drums'));
  });

  test('drums reapply the moment visibility returns', () async {
    final visible = StateProvider((ref) => true);
    final prefs = FakePreferencesService();
    final c = _container(
      prefs,
      drumsOverride: drumsEnabledProvider.overrideWith(
        (ref) => ref.watch(visible),
      ),
    );
    c.read(instrumentContextProvider.notifier).select(AppInstrument.drums);
    expect(c.read(effectiveInstrumentContextProvider), AppInstrument.drums);

    // The campaign closes: keyboard presented, drums still stored.
    c.read(visible.notifier).state = false;
    expect(c.read(effectiveInstrumentContextProvider), AppInstrument.keyboard);
    expect(c.read(instrumentContextProvider).context, AppInstrument.drums);

    // It reopens: the stored context reapplies with no user action.
    c.read(visible.notifier).state = true;
    expect(c.read(effectiveInstrumentContextProvider), AppInstrument.drums);
  });
}
