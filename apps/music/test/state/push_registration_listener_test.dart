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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/account_service.dart';
import 'package:music/services/notification_service.dart';
import 'package:music/services/push_service.dart';
import 'package:music/services/timezone_service.dart';
import 'package:music/state/push_registration_listener.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/session_state.dart';

import 'push_registration_listener_test.mocks.dart';

@GenerateNiceMocks([
  MockSpec<PushService>(),
  MockSpec<NotificationRegistryService>(),
  MockSpec<TimezoneService>(),
])
/// A drivable [SessionNotifier]: skips the real hydrate so a test can emit
/// session states and make the listener's `ref.listen` fire.
class _TestSessionNotifier extends SessionNotifier {
  @override
  SessionState build() => const SessionState.unknown();

  void emit(SessionState next) => state = next;
}

Account _acct(String userId) =>
    Account(userId: userId, version: 1, handle: 'h');

void main() {
  late MockPushService push;
  late MockNotificationRegistryService registry;
  late MockTimezoneService timezone;

  setUp(() {
    push = MockPushService();
    registry = MockNotificationRegistryService();
    timezone = MockTimezoneService();
    when(push.platform).thenReturn(PushPlatform.ios);
    when(push.tokenRefreshes).thenAnswer((_) => const Stream<String>.empty());
    when(
      push.requestPermission(),
    ).thenAnswer((_) async => PushPermission.granted);
    when(push.token()).thenAnswer((_) async => 'tok-1');
    when(timezone.current()).thenAnswer((_) async => 'Europe/Paris');
    when(
      registry.settings(),
    ).thenAnswer((_) async => NotificationSettings.empty);
  });

  Future<_TestSessionNotifier> pump(WidgetTester tester) async {
    final container = ProviderContainer(
      overrides: [
        pushServiceProvider.overrideWithValue(push),
        notificationRegistryServiceProvider.overrideWithValue(registry),
        timezoneServiceProvider.overrideWithValue(timezone),
        sessionNotifierProvider.overrideWith(_TestSessionNotifier.new),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const PushRegistrationListener(child: SizedBox()),
      ),
    );
    await tester.pumpAndSettle();
    return container.read(sessionNotifierProvider.notifier)
        as _TestSessionNotifier;
  }

  testWidgets('registers once per signed-in account', (tester) async {
    final session = await pump(tester);

    session.emit(SessionState.authenticated(account: _acct('u1')));
    await tester.pumpAndSettle();
    verify(registry.registerToken(token: 'tok-1', platform: 'ios')).called(1);

    // Re-resolving the SAME account (e.g. after handle onboarding) must not
    // re-register.
    session.emit(SessionState.authenticated(account: _acct('u1')));
    await tester.pumpAndSettle();
    verifyNever(
      registry.registerToken(
        token: anyNamed('token'),
        platform: anyNamed('platform'),
      ),
    );
  });

  testWidgets('signing out unregisters the device', (tester) async {
    final session = await pump(tester);

    session.emit(SessionState.authenticated(account: _acct('u1')));
    await tester.pumpAndSettle();
    session.emit(const SessionState.unauthenticated());
    await tester.pumpAndSettle();

    verify(registry.unregisterToken('tok-1')).called(1);
  });

  testWidgets('the transient startup state never tears a registration down', (
    tester,
  ) async {
    final session = await pump(tester);

    session.emit(SessionState.authenticated(account: _acct('u1')));
    await tester.pumpAndSettle();
    session.emit(const SessionState.unknown());
    await tester.pumpAndSettle();

    verifyNever(registry.unregisterToken(any));
  });

  testWidgets('a guest never registers', (tester) async {
    final session = await pump(tester);

    session.emit(const SessionState.guest());
    await tester.pumpAndSettle();

    verifyNever(push.requestPermission());
    verifyNever(
      registry.registerToken(
        token: anyNamed('token'),
        platform: anyNamed('platform'),
      ),
    );
  });

  testWidgets('a different account re-registers this device', (tester) async {
    final session = await pump(tester);

    session.emit(SessionState.authenticated(account: _acct('u1')));
    await tester.pumpAndSettle();
    session.emit(SessionState.authenticated(account: _acct('u2')));
    await tester.pumpAndSettle();

    verify(registry.registerToken(token: 'tok-1', platform: 'ios')).called(2);
  });
}
