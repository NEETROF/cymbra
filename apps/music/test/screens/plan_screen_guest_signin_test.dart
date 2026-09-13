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

// From guest paywall to live offer through one sign-in (change:
// open-app-without-sign-in-wall, design D4). Driven through a real session
// transition rather than an overridden flag, because what is under test is that
// the identity-scoped plan provider re-reads the plan on its own.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:music/screens/onboarding/sign_in_invitation.dart';
import 'package:music/screens/plan_screen.dart';
import 'package:music/services/app_platform.dart';
import 'package:music/services/plan_service.dart';
import 'package:music/services/store_client.dart';
import 'package:music/state/plan_notifier.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/state/session_notifier.dart';

import '../state/plan_notifier_test.mocks.dart';
import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';
import '../support/localized.dart';

class _Store extends Fake implements StoreClient {
  @override
  Future<bool> isAvailable() async => true;
  @override
  Future<void> setAccount(String? userId) async {}
  @override
  Future<List<StoreProduct>> products(Set<String> ids) async => [
    for (final id in ids)
      StoreProduct(id: id, title: id, description: '', price: '4,99 €'),
  ];
  @override
  Stream<StoreEvent> get events => const Stream.empty();
}

/// Frames rather than `pumpAndSettle`: the store listing loads behind a spinner.
Future<void> _frames(WidgetTester tester, [int count = 16]) async {
  for (var i = 0; i < count; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  testWidgets(
    'a guest who signs in from the paywall gets the purchase actions',
    (tester) async {
      await tester.binding.setSurfaceSize(const Size(1200, 900));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      final service = MockPlanService();
      when(service.getMyPlan(any)).thenAnswer(
        (_) async => const PlanSnapshotView(
          plan: 'free',
          canPurchaseHere: true,
          purchaseChannel: PlanChannel.apple,
          products: ['premium_monthly', 'premium_yearly'],
        ),
      );
      final container = ProviderContainer(
        overrides: [
          ...authOverrides(
            store: FakeTokenStore(guest: true),
            account: FakeAccountService(account: fakeAccount(handle: 'bob')),
          ),
          planServiceProvider.overrideWithValue(service),
          storeClientProvider.overrideWithValue(_Store()),
          appPlatformProvider.overrideWithValue(AppPlatform.ios),
          plansEnabledProvider.overrideWithValue(true),
          nowFnProvider.overrideWithValue(() => DateTime.utc(2026, 3, 1)),
        ],
      );
      addTearDown(container.dispose);
      container.read(sessionNotifierProvider);
      await tester.runAsync(() => pumpEventQueue());

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(const PlanScreen()),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('plan-guest')), findsOneWidget);
      verifyNever(service.getMyPlan(any));

      await tester.tap(find.byKey(const Key('plan-guest-sign-in')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('sign-in-invitation-accept')));
      await tester.pumpAndSettle();
      expect(find.byType(SignInInvitationScreen), findsOneWidget);

      await tester.tap(find.byKey(const Key('invite-google')));
      await _frames(tester);

      // Back on the paywall: the plan was read for the new session and the offer
      // is live — with nothing but the provider's own identity scoping.
      expect(find.byType(SignInInvitationScreen), findsNothing);
      expect(find.byKey(const Key('plan-guest')), findsNothing);
      verify(service.getMyPlan(any)).called(greaterThanOrEqualTo(1));
      expect(find.byKey(const Key('plan-buy-premium_monthly')), findsOneWidget);
    },
  );
}
