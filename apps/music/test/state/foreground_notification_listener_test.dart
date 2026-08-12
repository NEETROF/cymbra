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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/push_service.dart';
import 'package:music/state/foreground_notification.dart';
import 'package:music/state/foreground_notification_listener.dart';
import 'package:music/widgets/foreground_notification_banner.dart';

import 'foreground_notification_listener_test.mocks.dart';

@GenerateNiceMocks([MockSpec<PushService>()])
/// Tasks 5.2/5.4 (add-foreground-notifications): the listener bridges the
/// mocked [PushService] stream to the notifier, navigates on a tapped route,
/// and the platform's shipped state (nothing configured) stays inert.
void main() {
  late MockPushService push;
  late StreamController<PushForegroundMessage> messages;

  setUp(() {
    push = MockPushService();
    messages = StreamController<PushForegroundMessage>();
    // NOT `addTearDown(messages.close)`: close() of a single-subscription
    // controller nobody ever listened to (the regression test below) never
    // completes, and an awaited tear-down would hang the test.
    addTearDown(() => unawaited(messages.close()));
    when(push.foregroundMessages).thenAnswer((_) => messages.stream);
  });

  Future<ProviderContainer> pump(
    WidgetTester tester, {
    Map<String, WidgetBuilder> routes = const {},
  }) async {
    final container = ProviderContainer(
      overrides: [
        pushServiceProvider.overrideWithValue(push),
        pushRouteBuildersProvider.overrideWithValue(routes),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        // Production shape: the banner layer above the navigator, the listener
        // inside it.
        child: MaterialApp(
          builder: (context, child) =>
              Stack(children: [?child, const ForegroundNotificationLayer()]),
          home: const ForegroundNotificationListener(
            child: Scaffold(body: Text('HOME')),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    return container;
  }

  testWidgets('a delivered message reaches the notifier', (tester) async {
    final container = await pump(tester);

    messages.add(
      const PushForegroundMessage(
        title: 'T',
        body: 'B',
        data: {kPushForegroundDataKey: 'true'},
      ),
    );
    await tester.pumpAndSettle();

    expect(container.read(foregroundNotificationProvider).banner, isNotNull);
    expect(find.byKey(const Key('foreground-banner')), findsOneWidget);
  });

  testWidgets(
    'a category this build has never declared still surfaces when the '
    'message says to',
    (tester) async {
      // `pushCategories` is empty by design and the notifier never consults it:
      // the message's own indication is the whole decision.
      final container = await pump(tester);

      messages.add(
        const PushForegroundMessage(
          title: 'New thing',
          body: 'from a future feature',
          data: {kPushForegroundDataKey: 'true', 'category': 'never_declared'},
        ),
      );
      await tester.pumpAndSettle();

      expect(container.read(foregroundNotificationProvider).banner, isNotNull);
    },
  );

  testWidgets('an indication-less message surfaces nothing', (tester) async {
    final container = await pump(tester);

    messages.add(const PushForegroundMessage(title: 'T', body: 'B'));
    await tester.pumpAndSettle();

    expect(container.read(foregroundNotificationProvider).banner, isNull);
    expect(find.byKey(const Key('foreground-banner')), findsNothing);
  });

  testWidgets('a tapped banner routes through its payload', (tester) async {
    await pump(
      tester,
      routes: {'/dest': (_) => const Scaffold(body: Text('DEST'))},
    );

    messages.add(
      const PushForegroundMessage(
        title: 'T',
        body: 'B',
        data: {kPushForegroundDataKey: 'true', kPushRouteDataKey: '/dest'},
      ),
    );
    await tester.pumpAndSettle();
    // Tap the message body: the card's centre can land on the close button.
    await tester.tap(find.text('B'));
    await tester.pumpAndSettle();

    expect(find.text('DEST'), findsOneWidget);
    expect(find.byKey(const Key('foreground-banner')), findsNothing);
  });

  testWidgets('an unknown route dismisses without navigating', (tester) async {
    await pump(tester);

    messages.add(
      const PushForegroundMessage(
        title: 'T',
        body: 'B',
        data: {kPushForegroundDataKey: 'true', kPushRouteDataKey: '/nowhere'},
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('B'));
    await tester.pumpAndSettle();

    expect(find.text('HOME'), findsOneWidget);
    expect(find.byKey(const Key('foreground-banner')), findsNothing);
  });

  testWidgets(
    'regression: the shipped state (no Firebase, no categories) renders '
    'nothing and never reaches a real SDK',
    (tester) async {
      // No overrides at all: the REAL FirebasePushService backs the stream. Its
      // guard must degrade to an empty stream — no banner, no subscription to
      // the native SDK, no crash (task 5.4).
      final container = ProviderContainer();
      addTearDown(container.dispose);
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: MaterialApp(
            builder: (context, child) =>
                Stack(children: [?child, const ForegroundNotificationLayer()]),
            home: const ForegroundNotificationListener(
              child: Scaffold(body: Text('HOME')),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('HOME'), findsOneWidget);
      expect(find.byKey(const Key('foreground-banner')), findsNothing);
      expect(container.read(foregroundNotificationProvider).banner, isNull);
      expect(tester.takeException(), isNull);
    },
  );
}
