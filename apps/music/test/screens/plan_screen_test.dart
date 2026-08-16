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
import 'package:mockito/mockito.dart';
import 'package:music/screens/plan_screen.dart';
import 'package:music/services/app_platform.dart';
import 'package:music/services/legal_links.dart';
import 'package:music/services/plan_service.dart';
import 'package:music/services/store_client.dart';
import 'package:music/state/plan_notifier.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/state/session_notifier.dart';

import '../state/plan_notifier_test.mocks.dart';
import '../support/localized.dart';

class _RecordingLauncher implements LegalLinkLauncher {
  final List<Uri> opened = [];
  @override
  Future<void> open(Uri url) async => opened.add(url);
}

class _FakeStore extends Fake implements StoreClient {
  _FakeStore({this.available = true});
  final bool available;
  @override
  Future<bool> isAvailable() async => available;
  @override
  Future<List<StoreProduct>> products(Set<String> ids) async => [
    for (final id in ids)
      StoreProduct(
        id: id,
        title: id,
        description: '',
        price: id.contains('year') ? '39,99 €' : '4,99 €',
      ),
  ];
  @override
  Stream<StoreEvent> get events => const Stream.empty();
}

Future<void> _pump(
  WidgetTester tester, {
  required PlanSnapshotView snapshot,
  required AppPlatform platform,
  StoreClient? store,
  LegalLinkLauncher? launcher,
}) async {
  final svc = MockPlanService();
  when(svc.getMyPlan(any)).thenAnswer((_) async => snapshot);
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        planServiceProvider.overrideWithValue(svc),
        storeClientProvider.overrideWithValue(store ?? _FakeStore()),
        appPlatformProvider.overrideWithValue(platform),
        canUseOnlineServicesProvider.overrideWithValue(true),
        currentUserIdProvider.overrideWithValue('u1'),
        plansEnabledProvider.overrideWithValue(true),
        nowFnProvider.overrideWithValue(() => DateTime.utc(2026, 3, 1)),
        legalLinkLauncherProvider.overrideWithValue(
          launcher ?? _RecordingLauncher(),
        ),
      ],
      child: localizedApp(const PlanScreen()),
    ),
  );
  await tester.pumpAndSettle();
}

const _free = PlanSnapshotView(plan: 'free');

PlanSnapshotView _freeBuyable(PlanChannel channel) => PlanSnapshotView(
  plan: 'free',
  canPurchaseHere: true,
  purchaseChannel: channel,
  products: const ['premium_monthly', 'premium_yearly'],
);

void main() {
  testWidgets(
    'iOS: store products with prices, restore, no web link/code field',
    (tester) async {
      await _pump(
        tester,
        snapshot: _freeBuyable(PlanChannel.apple),
        platform: AppPlatform.ios,
      );
      expect(find.byKey(const Key('plan-status')), findsOneWidget);
      expect(find.text('Free plan'), findsOneWidget);
      expect(find.byKey(const Key('plan-buy-premium_monthly')), findsOneWidget);
      expect(find.textContaining('4,99 €'), findsOneWidget);
      expect(find.byKey(const Key('plan-restore')), findsOneWidget);
      // Store builds: no web checkout button, no "I've paid" refresh, no code
      // field anywhere.
      expect(find.byKey(const Key('plan-refresh')), findsNothing);
      expect(find.byType(TextField), findsNothing);
      expect(find.textContaining('browser'), findsNothing);
    },
  );

  testWidgets('Windows: web checkout buttons + refresh, no restore', (
    tester,
  ) async {
    await _pump(
      tester,
      snapshot: _freeBuyable(PlanChannel.web),
      platform: AppPlatform.windows,
    );
    expect(find.byKey(const Key('plan-buy-premium_monthly')), findsOneWidget);
    expect(find.byKey(const Key('plan-buy-premium_yearly')), findsOneWidget);
    expect(find.byKey(const Key('plan-refresh')), findsOneWidget);
    expect(find.byKey(const Key('plan-restore')), findsNothing);
    expect(find.textContaining('browser'), findsWidgets);
  });

  testWidgets('managed elsewhere: no purchase button, says where', (
    tester,
  ) async {
    await _pump(
      tester,
      snapshot: const PlanSnapshotView(
        plan: 'premium',
        source: 'google',
        managedOn: PlanChannel.google,
        unlocks: ['catalog.unlimited'],
      ),
      platform: AppPlatform.ios,
    );
    expect(find.byKey(const Key('plan-managed-elsewhere')), findsOneWidget);
    expect(find.textContaining('Google Play'), findsWidgets);
    expect(find.byKey(const Key('plan-buy-premium_monthly')), findsNothing);
    expect(find.byKey(const Key('plan-manage')), findsOneWidget);
  });

  testWidgets('manage opens the store subscription page via the launcher', (
    tester,
  ) async {
    final launcher = _RecordingLauncher();
    await _pump(
      tester,
      snapshot: const PlanSnapshotView(
        plan: 'premium',
        source: 'apple',
        managedOn: PlanChannel.apple,
        endsAt: null,
      ),
      platform: AppPlatform.ios,
      launcher: launcher,
    );
    await tester.tap(find.byKey(const Key('plan-manage')));
    await tester.pump();
    expect(launcher.opened.single.host, 'apps.apple.com');
  });

  testWidgets('trial: "trial until", rights-end warning, can still subscribe', (
    tester,
  ) async {
    await _pump(
      tester,
      snapshot: PlanSnapshotView(
        plan: 'premium',
        source: 'code',
        endsAt: DateTime.utc(2026, 5, 30),
        endsWithoutRenewal: true,
        trialCampaignKey: 'beta-premium',
        trialCampaignName: 'Beta premium',
        trialEndsAt: DateTime.utc(2026, 5, 30),
        canPurchaseHere: true,
        purchaseChannel: PlanChannel.apple,
        products: const ['premium_monthly'],
        unlocks: const ['catalog.unlimited', 'offline.cache'],
        betas: [
          BetaMembershipView(
            campaignKey: 'beta-premium',
            campaignName: 'Beta premium',
            kind: 'premium_trial',
            joinedAt: DateTime.utc(2026, 3, 1),
            endsAt: DateTime.utc(2026, 5, 30),
          ),
        ],
      ),
      platform: AppPlatform.ios,
    );
    expect(find.text('Premium trial'), findsOneWidget);
    expect(find.textContaining('Trial until'), findsOneWidget);
    expect(find.textContaining('Rights end on'), findsOneWidget);
    expect(find.textContaining('removed from this device'), findsOneWidget);
    expect(find.byKey(const Key('plan-betas')), findsOneWidget);
    expect(find.text('Beta premium'), findsWidgets);
    // The purchase card sits below the fold of the test viewport.
    await tester.drag(
      find.byKey(const Key('plan-screen')),
      const Offset(0, -700),
    );
    await tester.pumpAndSettle();
    expect(find.text('Keep Premium after your trial'), findsOneWidget);
    expect(find.byKey(const Key('plan-buy-premium_monthly')), findsOneWidget);
  });

  testWidgets(
    'feature-beta member on free lists the beta and the free paywall',
    (tester) async {
      await _pump(
        tester,
        snapshot: _freeBuyable(PlanChannel.apple).copyWith(
          betas: [
            BetaMembershipView(
              campaignKey: 'midi-drums',
              campaignName: 'MIDI drums',
              kind: 'feature',
              joinedAt: DateTime.utc(2026, 3, 1),
            ),
          ],
        ),
        platform: AppPlatform.ios,
      );
      expect(find.text('Free plan'), findsOneWidget);
      expect(find.text('MIDI drums'), findsOneWidget);
      expect(find.text('Go Premium'), findsOneWidget);
    },
  );

  testWidgets(
    'store unavailable shows a localized message, never a raw error',
    (tester) async {
      await _pump(
        tester,
        snapshot: _freeBuyable(PlanChannel.apple),
        platform: AppPlatform.ios,
        store: _FakeStore(available: false),
      );
      expect(find.byKey(const Key('plan-store-unavailable')), findsOneWidget);
      expect(find.byKey(const Key('plan-buy-premium_monthly')), findsNothing);
    },
  );

  testWidgets('plans disabled / free with no channel: status only', (
    tester,
  ) async {
    await _pump(tester, snapshot: _free, platform: AppPlatform.linux);
    expect(find.text('Free plan'), findsOneWidget);
    expect(find.byKey(const Key('plan-purchase')), findsNothing);
    expect(find.byKey(const Key('plan-managed-elsewhere')), findsNothing);
    expect(find.byKey(const Key('plan-restore')), findsNothing);
  });
}
