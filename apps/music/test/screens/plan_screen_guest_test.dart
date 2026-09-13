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

// The paywall seen by a guest (change: open-app-without-sign-in-wall, spec
// `music-premium-paywall`, "Guests see the offer and are invited to sign in to
// subscribe").

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:music/screens/plan_screen.dart';
import 'package:music/services/app_platform.dart';
import 'package:music/services/plan_service.dart';
import 'package:music/services/store_client.dart';
import 'package:music/state/plan_notifier.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/state/session_notifier.dart';

import '../state/plan_notifier_test.mocks.dart';
import '../support/localized.dart';

/// Counts the store calls a guest must never cause: availability and listing.
/// Binding the SDK to "no account" is session plumbing, not a purchase call, so
/// it is not counted.
class _SpyStore extends Fake implements StoreClient {
  int purchaseCalls = 0;
  @override
  Future<bool> isAvailable() async {
    purchaseCalls++;
    return true;
  }

  @override
  Future<List<StoreProduct>> products(Set<String> ids) async {
    purchaseCalls++;
    return const [];
  }

  @override
  Future<void> setAccount(String? userId) async {}
  @override
  Stream<StoreEvent> get events => const Stream.empty();
}

Future<({MockPlanService service, _SpyStore store})> _pumpGuest(
  WidgetTester tester,
) async {
  final service = MockPlanService();
  final store = _SpyStore();
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        planServiceProvider.overrideWithValue(service),
        storeClientProvider.overrideWithValue(store),
        appPlatformProvider.overrideWithValue(AppPlatform.ios),
        canUseOnlineServicesProvider.overrideWithValue(false),
        currentUserIdProvider.overrideWithValue(null),
        plansEnabledProvider.overrideWithValue(true),
        nowFnProvider.overrideWithValue(() => DateTime.utc(2026, 3, 1)),
      ],
      child: localizedApp(const PlanScreen()),
    ),
  );
  await tester.pumpAndSettle();
  return (service: service, store: store);
}

bool _isBuyButton(Widget w) =>
    w.key is ValueKey<String> &&
    (w.key! as ValueKey<String>).value.startsWith('plan-buy-');

void main() {
  testWidgets('a guest sees the offer, why it needs an account, and sign-in', (
    tester,
  ) async {
    final guest = await _pumpGuest(tester);

    expect(find.byKey(const Key('plan-benefits')), findsOneWidget);
    expect(find.byKey(const Key('plan-guest')), findsOneWidget);
    expect(find.byKey(const Key('plan-guest-sign-in')), findsOneWidget);
    expect(
      find.textContaining('Premium is attached to your Cymbra account'),
      findsOneWidget,
    );

    // Nothing that cannot work without an account.
    expect(find.byKey(const Key('plan-restore')), findsNothing);
    expect(find.byWidgetPredicate(_isBuyButton), findsNothing);

    // And nothing reached the backend or the store to draw it.
    verifyNever(guest.service.getMyPlan(any));
    expect(guest.store.purchaseCalls, 0);
  });

  testWidgets('signing in from the guest paywall names subscribing', (
    tester,
  ) async {
    await _pumpGuest(tester);

    await tester.tap(find.byKey(const Key('plan-guest-sign-in')));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('sign-in-invitation')), findsOneWidget);
    expect(
      find.text(
        'Sign in to subscribe. Premium is attached to your Cymbra account, '
        'which is what makes it follow you on every device.',
      ),
      findsOneWidget,
    );
  });

  testWidgets('declining the invitation leaves the guest on the paywall', (
    tester,
  ) async {
    final guest = await _pumpGuest(tester);

    await tester.tap(find.byKey(const Key('plan-guest-sign-in')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('sign-in-invitation-decline')));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('sign-in-invitation')), findsNothing);
    expect(find.byKey(const Key('plan-guest')), findsOneWidget);
    verifyNever(guest.service.getMyPlan(any));
  });
}
