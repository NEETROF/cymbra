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

import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/notification_service.dart';
import 'package:music/services/push_service.dart';
import 'package:music/services/timezone_service.dart';
import 'package:music/state/push_registration.dart';

import 'push_registration_test.mocks.dart';

@GenerateNiceMocks([
  MockSpec<PushService>(),
  MockSpec<NotificationRegistryService>(),
  MockSpec<TimezoneService>(),
])
void main() {
  group('push platform resolution', () {
    test('each supported platform has a stable wire name', () {
      expect(PushPlatform.ios.wireName, 'ios');
      expect(PushPlatform.android.wireName, 'android');
      expect(PushPlatform.macos.wireName, 'macos');
      // Windows and Linux are deliberately absent from the enum: there is no
      // value the client could report for them (design D7).
      expect(PushPlatform.values, hasLength(3));
    });

    test('the running platform resolves to a supported one or to none', () {
      // Host-agnostic: the test VM may be macOS (a supported platform) or Linux
      // in CI (unsupported → null). Either way the answer is never a platform
      // outside the FCM-capable set.
      final resolved = currentPushPlatform();
      expect(
        resolved == null || PushPlatform.values.contains(resolved),
        isTrue,
      );
    });
  });

  late MockPushService push;
  late MockNotificationRegistryService registry;
  late MockTimezoneService timezone;

  setUp(() {
    push = MockPushService();
    registry = MockNotificationRegistryService();
    timezone = MockTimezoneService();
    when(push.tokenRefreshes).thenAnswer((_) => const Stream<String>.empty());
    when(timezone.current()).thenAnswer((_) async => 'Europe/Paris');
  });

  ProviderContainer container() {
    final c = ProviderContainer(
      overrides: [
        pushServiceProvider.overrideWithValue(push),
        notificationRegistryServiceProvider.overrideWithValue(registry),
        timezoneServiceProvider.overrideWithValue(timezone),
      ],
    );
    addTearDown(c.dispose);
    return c;
  }

  test(
    'registers the token and reports the timezone once permission is granted',
    () async {
      when(push.platform).thenReturn(PushPlatform.ios);
      when(
        push.requestPermission(),
      ).thenAnswer((_) async => PushPermission.granted);
      when(push.token()).thenAnswer((_) async => 'tok-1');

      final c = container();
      await c.read(pushRegistrationProvider.notifier).registerForCurrentUser();

      verify(registry.registerToken(token: 'tok-1', platform: 'ios')).called(1);
      verify(registry.setTimezone('Europe/Paris')).called(1);
      final state = c.read(pushRegistrationProvider);
      expect(state.supported, isTrue);
      expect(state.registered, isTrue);
      expect(state.token, 'tok-1');
      expect(state.permission, PushPermission.granted);
    },
  );

  test('a refused permission registers nothing', () async {
    when(push.platform).thenReturn(PushPlatform.android);
    when(
      push.requestPermission(),
    ).thenAnswer((_) async => PushPermission.denied);

    final c = container();
    await c.read(pushRegistrationProvider.notifier).registerForCurrentUser();

    verifyNever(
      registry.registerToken(
        token: anyNamed('token'),
        platform: anyNamed('platform'),
      ),
    );
    verifyNever(push.token());
    final state = c.read(pushRegistrationProvider);
    // The platform IS supported — the user simply said no.
    expect(state.supported, isTrue);
    expect(state.registered, isFalse);
    expect(state.permission, PushPermission.denied);
  });

  test(
    'an unsupported platform (Windows/Linux) never asks for permission',
    () async {
      // `platform == null` is exactly what the service reports on Windows/Linux —
      // and on any build with no Firebase configuration.
      when(push.platform).thenReturn(null);

      final c = container();
      await c.read(pushRegistrationProvider.notifier).registerForCurrentUser();

      verifyNever(push.requestPermission());
      verifyNever(push.token());
      verifyNever(
        registry.registerToken(
          token: anyNamed('token'),
          platform: anyNamed('platform'),
        ),
      );
      verifyNever(registry.setTimezone(any));
      expect(c.read(pushRegistrationProvider).supported, isFalse);
    },
  );

  test('each supported platform registers under its own wire name', () async {
    for (final (platform, wire) in [
      (PushPlatform.ios, 'ios'),
      (PushPlatform.android, 'android'),
      (PushPlatform.macos, 'macos'),
    ]) {
      final localRegistry = MockNotificationRegistryService();
      registry = localRegistry;
      when(push.platform).thenReturn(platform);
      when(
        push.requestPermission(),
      ).thenAnswer((_) async => PushPermission.granted);
      when(push.token()).thenAnswer((_) async => 'tok');

      final c = container();
      await c.read(pushRegistrationProvider.notifier).registerForCurrentUser();
      verify(
        localRegistry.registerToken(token: 'tok', platform: wire),
      ).called(1);
    }
  });

  test('a rotated FCM token is re-registered', () async {
    final refreshes = StreamController<String>.broadcast();
    addTearDown(refreshes.close);
    when(push.platform).thenReturn(PushPlatform.android);
    when(
      push.requestPermission(),
    ).thenAnswer((_) async => PushPermission.granted);
    when(push.token()).thenAnswer((_) async => 'tok-old');
    when(push.tokenRefreshes).thenAnswer((_) => refreshes.stream);

    final c = container();
    await c.read(pushRegistrationProvider.notifier).registerForCurrentUser();
    expect(c.read(pushRegistrationProvider).token, 'tok-old');

    refreshes.add('tok-new');
    await pumpEventQueue();
    verify(
      registry.registerToken(token: 'tok-new', platform: 'android'),
    ).called(1);
    expect(c.read(pushRegistrationProvider).token, 'tok-new');
  });

  test(
    'a backend failure leaves the device unregistered so a later launch retries',
    () async {
      when(push.platform).thenReturn(PushPlatform.ios);
      when(
        push.requestPermission(),
      ).thenAnswer((_) async => PushPermission.granted);
      when(push.token()).thenAnswer((_) async => 'tok-1');
      when(
        registry.registerToken(
          token: anyNamed('token'),
          platform: anyNamed('platform'),
        ),
      ).thenThrow(Exception('offline'));

      final c = container();
      await c.read(pushRegistrationProvider.notifier).registerForCurrentUser();

      expect(c.read(pushRegistrationProvider).registered, isFalse);
    },
  );

  test('a missing timezone leaves the stored one alone', () async {
    when(push.platform).thenReturn(PushPlatform.ios);
    when(
      push.requestPermission(),
    ).thenAnswer((_) async => PushPermission.granted);
    when(push.token()).thenAnswer((_) async => 'tok-1');
    when(timezone.current()).thenAnswer((_) async => null);

    final c = container();
    await c.read(pushRegistrationProvider.notifier).registerForCurrentUser();

    verifyNever(registry.setTimezone(any));
  });

  test('unregister drops the token server-side and locally', () async {
    when(push.platform).thenReturn(PushPlatform.ios);
    when(
      push.requestPermission(),
    ).thenAnswer((_) async => PushPermission.granted);
    when(push.token()).thenAnswer((_) async => 'tok-1');

    final c = container();
    final notifier = c.read(pushRegistrationProvider.notifier);
    await notifier.registerForCurrentUser();
    await notifier.unregister();

    verify(registry.unregisterToken('tok-1')).called(1);
    verify(push.deleteToken()).called(1);
    expect(c.read(pushRegistrationProvider).registered, isFalse);
  });

  test('unregistering an unregistered device calls no RPC', () async {
    final c = container();
    await c.read(pushRegistrationProvider.notifier).unregister();
    verifyNever(registry.unregisterToken(any));
  });
}
