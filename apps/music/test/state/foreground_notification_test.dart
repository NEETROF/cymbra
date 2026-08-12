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
import 'package:music/services/push_service.dart';
import 'package:music/state/foreground_notification.dart';

/// Task 5.1 (add-foreground-notifications): the notifier surfaces a message
/// only when the message itself says so, and never consults the category list.
void main() {
  ProviderContainer container() {
    final c = ProviderContainer();
    addTearDown(c.dispose);
    // Keep the autoDispose notifier alive across the test body.
    c.listen(foregroundNotificationProvider, (_, _) {});
    return c;
  }

  PushForegroundMessage msg({Map<String, String> data = const {}}) =>
      PushForegroundMessage(
        title: 'Beaten!',
        body: 'u2 topped you',
        data: data,
      );

  test('a message carrying the foreground indication is surfaced', () {
    final c = container();
    final m = msg(data: const {kPushForegroundDataKey: 'true'});
    c.read(foregroundNotificationProvider.notifier).show(m);
    expect(c.read(foregroundNotificationProvider).banner, same(m));
  });

  test('a message saying not to surface stays silent', () {
    final c = container();
    c
        .read(foregroundNotificationProvider.notifier)
        .show(msg(data: const {kPushForegroundDataKey: 'false'}));
    expect(c.read(foregroundNotificationProvider).banner, isNull);
  });

  test('an indication-less message stays silent (absence means silence)', () {
    final c = container();
    c.read(foregroundNotificationProvider.notifier).show(msg());
    expect(c.read(foregroundNotificationProvider).banner, isNull);
  });

  test('dismissal clears the banner', () {
    final c = container();
    final n = c.read(foregroundNotificationProvider.notifier);
    n.show(msg(data: const {kPushForegroundDataKey: 'true'}));
    n.dismiss();
    expect(c.read(foregroundNotificationProvider).banner, isNull);
  });

  test('a tap dismisses and surfaces the routing payload', () {
    final c = container();
    final n = c.read(foregroundNotificationProvider.notifier);
    n.show(
      msg(
        data: const {
          kPushForegroundDataKey: 'true',
          kPushRouteDataKey: '/practice',
        },
      ),
    );
    n.tap();
    final s = c.read(foregroundNotificationProvider);
    expect(s.banner, isNull);
    expect(s.tappedRoute, '/practice');
    n.routeHandled();
    expect(c.read(foregroundNotificationProvider).tappedRoute, isNull);
  });

  test('a tap on a payload-less message dismisses without a route', () {
    final c = container();
    final n = c.read(foregroundNotificationProvider.notifier);
    n.show(msg(data: const {kPushForegroundDataKey: 'true'}));
    n.tap();
    final s = c.read(foregroundNotificationProvider);
    expect(s.banner, isNull);
    expect(s.tappedRoute, isNull);
  });

  test('a new surfaced message replaces the current banner', () {
    final c = container();
    final n = c.read(foregroundNotificationProvider.notifier);
    n.show(msg(data: const {kPushForegroundDataKey: 'true'}));
    final second = PushForegroundMessage(
      title: 'Second',
      body: 'b',
      data: const {kPushForegroundDataKey: 'true'},
    );
    n.show(second);
    expect(c.read(foregroundNotificationProvider).banner, same(second));
  });
}
