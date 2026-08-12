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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/push_service.dart';
import 'package:music/state/foreground_notification.dart';
import 'package:music/widgets/foreground_notification_banner.dart';

/// Task 5.3 (add-foreground-notifications): the banner renders the message,
/// dismisses, and hands taps to the notifier.
void main() {
  Future<ProviderContainer> pump(WidgetTester tester) async {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(
          home: Stack(
            children: [
              Scaffold(body: Text('UNDER')),
              ForegroundNotificationLayer(),
            ],
          ),
        ),
      ),
    );
    return container;
  }

  const surfaced = PushForegroundMessage(
    title: 'Score beaten',
    body: 'u2 just topped your best on Für Elise',
    data: {kPushForegroundDataKey: 'true'},
  );

  testWidgets('renders the message title and body as-is', (tester) async {
    final container = await pump(tester);

    container.read(foregroundNotificationProvider.notifier).show(surfaced);
    await tester.pumpAndSettle();

    expect(find.text('Score beaten'), findsOneWidget);
    expect(find.text('u2 just topped your best on Für Elise'), findsOneWidget);
  });

  testWidgets('an empty state renders nothing and blocks nothing', (
    tester,
  ) async {
    await pump(tester);
    expect(find.byKey(const Key('foreground-banner')), findsNothing);
    // The screen under the layer stays tappable/visible.
    expect(find.text('UNDER'), findsOneWidget);
  });

  testWidgets('the close button dismisses the banner', (tester) async {
    final container = await pump(tester);

    container.read(foregroundNotificationProvider.notifier).show(surfaced);
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('foreground-banner-dismiss')));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('foreground-banner')), findsNothing);
    expect(container.read(foregroundNotificationProvider).banner, isNull);
  });

  testWidgets('a tap hands the routing payload to the notifier', (
    tester,
  ) async {
    final container = await pump(tester);

    container
        .read(foregroundNotificationProvider.notifier)
        .show(
          const PushForegroundMessage(
            title: 'T',
            body: 'B',
            data: {
              kPushForegroundDataKey: 'true',
              kPushRouteDataKey: '/practice',
            },
          ),
        );
    await tester.pumpAndSettle();
    // Tap the message body: the card's centre can land on the close button.
    await tester.tap(find.text('B'));
    await tester.pumpAndSettle();

    final state = container.read(foregroundNotificationProvider);
    expect(state.banner, isNull);
    expect(state.tappedRoute, '/practice');
  });

  testWidgets('a tap without a payload just dismisses', (tester) async {
    final container = await pump(tester);

    container.read(foregroundNotificationProvider.notifier).show(surfaced);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Score beaten'));
    await tester.pumpAndSettle();

    final state = container.read(foregroundNotificationProvider);
    expect(state.banner, isNull);
    expect(state.tappedRoute, isNull);
    expect(find.byKey(const Key('foreground-banner')), findsNothing);
  });
}
