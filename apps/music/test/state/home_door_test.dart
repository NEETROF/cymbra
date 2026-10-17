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
import 'package:music/state/home_door.dart';
import 'package:music/state/instrument_context.dart';
import 'package:music/state/learn_door_access.dart';

import '../support/prefs_fakes.dart';

ProviderContainer _container({
  FakePreferencesService? prefs,
  bool learnDoor = true,
  bool drumsVisible = false,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      learnDoorEnabledProvider.overrideWithValue(learnDoor),
      drumsEnabledProvider.overrideWithValue(drumsVisible),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

/// Instantiates the persisted notifiers (a provider only runs its `build` on
/// first read) and lets their async restore land.
Future<void> _settle(ProviderContainer c) async {
  c.read(homeDoorChoiceProvider);
  c.read(instrumentContextProvider);
  await Future<void>.delayed(Duration.zero);
  await Future<void>.delayed(Duration.zero);
}

void main() {
  group('HomeDoorChoice', () {
    test('defaults to Play and reports hydration once restored', () async {
      final c = _container();
      expect(c.read(homeDoorChoiceProvider).door, HomeDoor.play);
      expect(c.read(homeDoorChoiceProvider).hydrated, isFalse);
      await _settle(c);
      expect(c.read(homeDoorChoiceProvider).hydrated, isTrue);
      expect(c.read(homeDoorChoiceProvider).door, HomeDoor.play);
    });

    test('restores the remembered door', () async {
      final c = _container(
        prefs: FakePreferencesService({HomeDoorChoice.prefsKey: 'learn'}),
      );
      await _settle(c);
      expect(c.read(homeDoorChoiceProvider).door, HomeDoor.learn);
    });

    test('a corrupt stored value falls back to Play', () async {
      final c = _container(
        prefs: FakePreferencesService({HomeDoorChoice.prefsKey: 'garage'}),
      );
      await _settle(c);
      expect(c.read(homeDoorChoiceProvider).door, HomeDoor.play);
      expect(c.read(homeDoorChoiceProvider).hydrated, isTrue);
    });

    test(
      'selecting persists the choice; re-selecting is a no-op write',
      () async {
        final prefs = FakePreferencesService();
        final c = _container(prefs: prefs);
        await _settle(c);
        c.read(homeDoorChoiceProvider.notifier).select(HomeDoor.learn);
        await _settle(c);
        expect(c.read(homeDoorChoiceProvider).door, HomeDoor.learn);
        expect(prefs.store[HomeDoorChoice.prefsKey], 'learn');
        prefs.store.clear();
        c.read(homeDoorChoiceProvider.notifier).select(HomeDoor.learn);
        await _settle(c);
        expect(prefs.store, isEmpty, reason: 'same door → nothing rewritten');
      },
    );
  });

  group('effectiveHomeDoor', () {
    test('presents the stored door while Learn is available', () async {
      final c = _container(
        prefs: FakePreferencesService({HomeDoorChoice.prefsKey: 'learn'}),
      );
      await _settle(c);
      expect(c.read(effectiveHomeDoorProvider), HomeDoor.learn);
      expect(c.read(learnDoorAvailableProvider), isTrue);
    });

    test(
      'falls back to Play while the flag is off, without rewriting',
      () async {
        final prefs = FakePreferencesService({
          HomeDoorChoice.prefsKey: 'learn',
        });
        final c = _container(prefs: prefs, learnDoor: false);
        await _settle(c);
        expect(c.read(effectiveHomeDoorProvider), HomeDoor.play);
        expect(c.read(learnDoorAvailableProvider), isFalse);
        expect(prefs.store[HomeDoorChoice.prefsKey], 'learn');
      },
    );

    test('falls back to Play under the drums context, and Learn reapplies '
        'when the keyboard context returns', () async {
      final prefs = FakePreferencesService({
        HomeDoorChoice.prefsKey: 'learn',
        InstrumentContext.prefsKey: jsonEncode({
          'context': 'drums',
          'choiceOffered': true,
        }),
      });
      final c = _container(prefs: prefs, drumsVisible: true);
      await _settle(c);
      expect(c.read(effectiveHomeDoorProvider), HomeDoor.play);
      expect(c.read(learnDoorAvailableProvider), isFalse);

      c.read(instrumentContextProvider.notifier).select(AppInstrument.keyboard);
      expect(c.read(effectiveHomeDoorProvider), HomeDoor.learn);
      expect(c.read(learnDoorAvailableProvider), isTrue);
    });
  });
}
