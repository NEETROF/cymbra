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

// The guest account menu (change: open-app-without-sign-in-wall, spec
// `account-access`, "Guest mode is fully offline"): a guest reaches the
// subscription, help, language and legal pages without signing in, and never
// the entries that need an account.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:music/screens/auth/account_menu.dart';
import 'package:music/services/app_platform.dart';
import 'package:music/services/legal_links.dart';
import 'package:music/services/plan_service.dart';
import 'package:music/services/store_client.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/state/plan_notifier.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;

import '../state/plan_notifier_test.mocks.dart';
import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';
import '../support/localized.dart';

class _FixedLocale extends AppLocale {
  @override
  Locale build() => const Locale('en');
}

class _RecordingLauncher implements LegalLinkLauncher {
  final List<Uri> opened = [];
  @override
  Future<void> open(Uri url) async => opened.add(url);
}

/// A guest must never cause a store call; binding "no account" is plumbing.
class _NoStore extends Fake implements StoreClient {
  @override
  Future<void> setAccount(String? userId) async {}
  @override
  Stream<StoreEvent> get events => const Stream.empty();
}

Future<({MockPlanService service, _RecordingLauncher launcher})> _pumpGuestMenu(
  WidgetTester tester,
) async {
  final service = MockPlanService();
  final launcher = _RecordingLauncher();
  await tester.binding.setSurfaceSize(const Size(1200, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        ...authOverrides(store: FakeTokenStore(guest: true)),
        appLocaleProvider.overrideWith(_FixedLocale.new),
        legalLinkLauncherProvider.overrideWithValue(launcher),
        planServiceProvider.overrideWithValue(service),
        storeClientProvider.overrideWithValue(_NoStore()),
        appPlatformProvider.overrideWithValue(AppPlatform.ios),
        plansEnabledProvider.overrideWithValue(true),
        nowFnProvider.overrideWithValue(() => DateTime.utc(2026, 3, 1)),
      ],
      child: localizedApp(const Scaffold(body: Center(child: AccountMenu()))),
    ),
  );
  await tester.pumpAndSettle();
  await tester.tap(find.byKey(const Key('account-guest-menu')));
  await tester.pumpAndSettle();
  return (service: service, launcher: launcher);
}

void main() {
  testWidgets('a guest menu offers everything that needs no account', (
    tester,
  ) async {
    await _pumpGuestMenu(tester);

    for (final key in [
      'account-signin',
      'account-plan',
      'account-help',
      'account-language',
      'account-legal-terms',
      'account-legal-privacy',
      'account-legal-licenses',
    ]) {
      expect(find.byKey(Key(key)), findsOneWidget, reason: key);
    }
    // The account-bound entries stay out of a guest's reach.
    expect(find.byKey(const Key('account-profile')), findsNothing);
    expect(find.byKey(const Key('account-connected')), findsNothing);
    expect(find.byKey(const Key('account-signout-all')), findsNothing);
    expect(find.text('Sign out'), findsNothing);
    expect(find.text('Delete account'), findsNothing);
  });

  testWidgets('a guest reaches the paywall, in its guest state', (
    tester,
  ) async {
    final guest = await _pumpGuestMenu(tester);

    await tester.tap(find.byKey(const Key('account-plan')));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('plan-guest')), findsOneWidget);
    expect(find.byKey(const Key('plan-guest-sign-in')), findsOneWidget);
    verifyNever(guest.service.getMyPlan(any));
  });

  testWidgets('a guest opens the terms through the launcher', (tester) async {
    final guest = await _pumpGuestMenu(tester);

    await tester.tap(find.byKey(const Key('account-legal-terms')));
    await tester.pumpAndSettle();

    expect(guest.launcher.opened, [legalLinksFor('en').terms]);
  });
}
