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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/notification_service.dart';
import 'package:music/state/notification_prefs.dart';
import 'package:music/state/push_categories.dart';

import 'notification_prefs_test.mocks.dart';

@GenerateNiceMocks([MockSpec<NotificationRegistryService>()])
/// A stand-in for the category a feature will declare — the platform itself
/// ships none, so the mechanism is exercised with a test-owned one.
final _streak = PushCategory(
  id: 'practice_streak',
  label: (_) => 'Streak reminder',
);

/// A category whose product default is opt-out.
final _optIn = PushCategory(
  id: 'newsletter',
  label: (_) => 'News',
  defaultEnabled: false,
);

void main() {
  late MockNotificationRegistryService registry;

  setUp(() {
    registry = MockNotificationRegistryService();
    when(
      registry.settings(),
    ).thenAnswer((_) async => NotificationSettings.empty);
  });

  ProviderContainer container() {
    final c = ProviderContainer(
      overrides: [
        notificationRegistryServiceProvider.overrideWithValue(registry),
      ],
    );
    addTearDown(c.dispose);
    return c;
  }

  test('loads the stored preferences from the server', () async {
    when(registry.settings()).thenAnswer(
      (_) async => const NotificationSettings(
        prefs: {'practice_streak': false},
        timezone: 'Europe/Paris',
        hasRegisteredDevice: true,
      ),
    );

    final c = container();
    final settings = await c.read(notificationPrefsProvider.future);
    expect(settings.prefs['practice_streak'], isFalse);
    expect(settings.timezone, 'Europe/Paris');
    expect(settings.hasRegisteredDevice, isTrue);
    expect(
      c.read(notificationPrefsProvider.notifier).isEnabled(_streak),
      isFalse,
    );
  });

  test('an absent preference falls back to the category default', () async {
    final c = container();
    await c.read(notificationPrefsProvider.future);
    final notifier = c.read(notificationPrefsProvider.notifier);
    expect(notifier.isEnabled(_streak), isTrue); // defaultEnabled: true
    expect(notifier.isEnabled(_optIn), isFalse); // defaultEnabled: false
  });

  test(
    'toggling records the choice server-side and flips immediately',
    () async {
      final c = container();
      await c.read(notificationPrefsProvider.future);
      final notifier = c.read(notificationPrefsProvider.notifier);

      await notifier.setEnabled(_streak, false);

      verify(
        registry.setPref(category: 'practice_streak', enabled: false),
      ).called(1);
      expect(notifier.isEnabled(_streak), isFalse);
    },
  );

  test('a refused write reverts the switch', () async {
    when(
      registry.setPref(
        category: anyNamed('category'),
        enabled: anyNamed('enabled'),
      ),
    ).thenThrow(Exception('offline'));

    final c = container();
    await c.read(notificationPrefsProvider.future);
    final notifier = c.read(notificationPrefsProvider.notifier);

    await notifier.setEnabled(_streak, false);
    expect(notifier.isEnabled(_streak), isTrue); // reverted to the default
  });

  test('a failing load shows the defaults rather than an error', () async {
    when(registry.settings()).thenThrow(Exception('offline'));

    final c = container();
    final settings = await c.read(notificationPrefsProvider.future);
    expect(settings.prefs, isEmpty);
    expect(c.read(notificationPrefsProvider).hasError, isFalse);
    expect(
      c.read(notificationPrefsProvider.notifier).isEnabled(_streak),
      isTrue,
    );
  });

  test('the platform declares no category of its own', () {
    // Design D6: a category ships with the feature that owns its notification
    // type, never with the platform.
    final c = ProviderContainer();
    addTearDown(c.dispose);
    expect(c.read(pushCategoriesProvider), isEmpty);
  });
}
