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
import 'package:music/services/app_platform.dart';
import 'package:music/services/legal_links.dart';
import 'package:music/services/plan_service.dart';
import 'package:music/services/store_client.dart';
import 'package:music/state/plan_notifier.dart';
import 'package:music/state/plan_withdrawal.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/state/session_notifier.dart';
import 'package:music/services/curator_rewards_service.dart';

import 'plan_notifier_test.mocks.dart';

@GenerateNiceMocks([MockSpec<PlanService>(), MockSpec<StoreClient>()])
class _RecordingLauncher implements LegalLinkLauncher {
  final List<Uri> opened = [];
  @override
  Future<void> open(Uri url) async => opened.add(url);
}

PlanSnapshotView _premium({
  DateTime? endsAt,
  bool trial = false,
  bool canPurchase = false,
  PlanChannel? managedOn,
}) => PlanSnapshotView(
  plan: 'premium',
  source: trial ? 'code' : 'apple',
  endsAt: endsAt,
  trialCampaignKey: trial ? 'beta-premium' : null,
  trialCampaignName: trial ? 'Beta premium' : null,
  trialEndsAt: trial ? endsAt : null,
  managedOn: managedOn,
  canPurchaseHere: canPurchase,
  purchaseChannel: canPurchase ? PlanChannel.apple : null,
  products: canPurchase ? const ['premium_monthly'] : const [],
  unlocks: const ['catalog.unlimited', 'offline.cache'],
);

ProviderContainer _container({
  required PlanService service,
  StoreClient? store,
  AppPlatform platform = AppPlatform.ios,
  bool online = true,
  bool plansEnabled = true,
  DateTime? now,
  LegalLinkLauncher? launcher,
  String? userId = 'u1',
}) {
  final c = ProviderContainer(
    overrides: [
      planServiceProvider.overrideWithValue(service),
      storeClientProvider.overrideWithValue(store ?? const NoopStoreClient()),
      appPlatformProvider.overrideWithValue(platform),
      canUseOnlineServicesProvider.overrideWithValue(online),
      currentUserIdProvider.overrideWithValue(userId),
      plansEnabledProvider.overrideWithValue(plansEnabled),
      nowFnProvider.overrideWithValue(() => now ?? DateTime.utc(2026, 3, 1)),
      legalLinkLauncherProvider.overrideWithValue(
        launcher ?? _RecordingLauncher(),
      ),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  group('Plan', () {
    test(
      'signed out ⇒ free without a call; online ⇒ server snapshot',
      () async {
        final svc = MockPlanService();
        when(svc.getMyPlan(any)).thenAnswer((_) async => _premium());
        final offline = _container(service: svc, online: false);
        expect(await offline.read(planProvider.future), PlanSnapshotView.free);
        verifyNever(svc.getMyPlan(any));

        final online = _container(service: svc, platform: AppPlatform.android);
        final snap = await online.read(planProvider.future);
        expect(snap.isPremium, isTrue);
        verify(svc.getMyPlan(AppPlatform.android)).called(1);
      },
    );

    test(
      'a failed first read is free; a failed refresh keeps last-known',
      () async {
        final svc = MockPlanService();
        when(svc.getMyPlan(any)).thenThrow(Exception('down'));
        final c = _container(service: svc);
        expect(await c.read(planProvider.future), PlanSnapshotView.free);

        when(svc.getMyPlan(any)).thenAnswer((_) async => _premium());
        await c.read(planProvider.notifier).refresh();
        expect(c.read(planProvider).valueOrNull?.isPremium, isTrue);
        when(svc.getMyPlan(any)).thenThrow(Exception('down again'));
        await c.read(planProvider.notifier).refresh();
        expect(
          c.read(planProvider).valueOrNull?.isPremium,
          isTrue,
          reason: 'offline keeps the last-known plan',
        );
      },
    );

    test(
      'local belt: past endsAt + grace the effective plan is free',
      () async {
        final svc = MockPlanService();
        final ends = DateTime.utc(2026, 3, 10);
        when(
          svc.getMyPlan(any),
        ).thenAnswer((_) async => _premium(endsAt: ends));
        final fresh = _container(service: svc, now: DateTime.utc(2026, 3, 12));
        await fresh.read(planProvider.future);
        expect(fresh.read(effectivePlanProvider).isPremium, isTrue);
        expect(fresh.read(catalogOfflineCacheAllowedProvider), isTrue);

        final stale = _container(service: svc, now: DateTime.utc(2026, 3, 14));
        await stale.read(planProvider.future);
        expect(stale.read(effectivePlanProvider).isPremium, isFalse);
        expect(stale.read(catalogOfflineCacheAllowedProvider), isFalse);
      },
    );

    test('plans disabled ⇒ catalog offline caching stays as before', () async {
      final svc = MockPlanService();
      when(svc.getMyPlan(any)).thenAnswer((_) async => PlanSnapshotView.free);
      final c = _container(service: svc, plansEnabled: false);
      await c.read(planProvider.future);
      expect(c.read(catalogOfflineCacheAllowedProvider), isTrue);
    });
  });

  group('PurchaseFlow', () {
    test('desktop buy opens the web checkout in the browser', () async {
      final svc = MockPlanService();
      when(svc.getMyPlan(any)).thenAnswer(
        (_) async => _premium(canPurchase: true).copyWith(
          plan: 'free',
          source: null,
          unlocks: const [],
          purchaseChannel: PlanChannel.web,
        ),
      );
      when(
        svc.createWebCheckout('premium_monthly'),
      ).thenAnswer((_) async => Uri.parse('https://pay.example/c?_ptxn=1'));
      final launcher = _RecordingLauncher();
      final c = _container(
        service: svc,
        platform: AppPlatform.windows,
        launcher: launcher,
      );
      await c.read(planProvider.future);
      await c.read(purchaseFlowProvider.notifier).buy('premium_monthly');
      expect(launcher.opened.single.host, 'pay.example');
      final s = c.read(purchaseFlowProvider);
      expect(s.outcome, PurchaseOutcome.checkoutOpened);
      expect(s.busy, isFalse);
      expect(s.seq, 1);
    });

    test(
      'store buy binds the account; the store receipt triggers a plan sync',
      () async {
        final svc = MockPlanService();
        when(svc.getMyPlan(any)).thenAnswer(
          (_) async => _premium(
            canPurchase: true,
          ).copyWith(plan: 'free', source: null, unlocks: const []),
        );
        when(
          svc.syncStorePlan(any),
        ).thenAnswer((_) async => PurchaseReportView(plan: _premium()));
        final store = MockStoreClient();
        final events = StreamController<StoreEvent>.broadcast();
        when(store.events).thenAnswer((_) => events.stream);
        when(store.setAccount(any)).thenAnswer((_) async {});
        when(
          store.buy(any, accountToken: anyNamed('accountToken')),
        ).thenAnswer((_) async {});
        when(store.complete(any)).thenAnswer((_) async {});
        final c = _container(service: svc, store: store);
        await c.read(planProvider.future);
        // Subscribe the flow (build) before events arrive; build binds the
        // SDK to the signed-in account.
        c.read(purchaseFlowProvider);
        await Future<void>.delayed(Duration.zero);
        verify(store.setAccount('u1')).called(1);
        await c.read(purchaseFlowProvider.notifier).buy('premium_monthly');
        verify(store.buy('premium_monthly', accountToken: 'u1')).called(1);
        expect(c.read(purchaseFlowProvider).busy, isTrue);

        events.add(
          const StoreEvent.receipt(StoreReceipt(productId: 'premium_monthly')),
        );
        await Future<void>.delayed(Duration.zero);
        await Future<void>.delayed(Duration.zero);
        // No payload ever crosses: the server re-reads the aggregator.
        verify(svc.syncStorePlan(AppPlatform.ios)).called(1);
        verify(store.complete(any)).called(1);
        expect(c.read(purchaseFlowProvider).outcome, PurchaseOutcome.purchased);
        expect(c.read(planProvider).valueOrNull?.isPremium, isTrue);
        await events.close();
      },
    );

    test(
      'cancel / error / pending land as outcomes; claim fires once',
      () async {
        final svc = MockPlanService();
        when(svc.getMyPlan(any)).thenAnswer((_) async => PlanSnapshotView.free);
        final store = MockStoreClient();
        final events = StreamController<StoreEvent>.broadcast();
        when(store.events).thenAnswer((_) => events.stream);
        when(store.setAccount(any)).thenAnswer((_) async {});
        final c = _container(service: svc, store: store);
        c.read(purchaseFlowProvider);
        events.add(const StoreEvent.cancelled());
        await Future<void>.delayed(Duration.zero);
        expect(c.read(purchaseFlowProvider).outcome, PurchaseOutcome.cancelled);
        final n = c.read(purchaseFlowProvider.notifier);
        expect(n.claim(1), isTrue);
        expect(n.claim(1), isFalse);
        events.add(const StoreEvent.error('boom'));
        await Future<void>.delayed(Duration.zero);
        expect(c.read(purchaseFlowProvider).outcome, PurchaseOutcome.failed);
        events.add(const StoreEvent.pending());
        await Future<void>.delayed(Duration.zero);
        expect(c.read(purchaseFlowProvider).outcome, PurchaseOutcome.pending);
        await events.close();
      },
    );

    test(
      'restore settles from the store events: found ⇒ sync + restored, '
      'none ⇒ nothingToRestore, another account ⇒ otherAccount (no sync)',
      () async {
        final svc = MockPlanService();
        when(svc.getMyPlan(any)).thenAnswer((_) async => PlanSnapshotView.free);
        when(
          svc.syncStorePlan(any),
        ).thenAnswer((_) async => PurchaseReportView(plan: _premium()));
        final store = MockStoreClient();
        final events = StreamController<StoreEvent>.broadcast();
        when(store.events).thenAnswer((_) => events.stream);
        when(store.setAccount(any)).thenAnswer((_) async {});
        when(store.complete(any)).thenAnswer((_) async {});
        when(
          store.restore(accountToken: anyNamed('accountToken')),
        ).thenAnswer((_) async {});
        final c = _container(service: svc, store: store);
        await c.read(planProvider.future);
        c.read(purchaseFlowProvider);

        // The aggregator refused (receipt kept with its original account):
        // the outcome is explicit and the server is not asked to sync.
        await c.read(purchaseFlowProvider.notifier).restore();
        verify(store.restore(accountToken: 'u1')).called(1);
        events.add(const StoreEvent.otherAccount());
        await Future<void>.delayed(Duration.zero);
        expect(
          c.read(purchaseFlowProvider).outcome,
          PurchaseOutcome.otherAccount,
        );
        verifyNever(svc.syncStorePlan(any));
        expect(c.read(planProvider).valueOrNull?.isPremium, isFalse);

        // Nothing to restore: settles immediately, no timed wait.
        await c.read(purchaseFlowProvider.notifier).restore();
        events.add(const StoreEvent.nothingToRestore());
        await Future<void>.delayed(Duration.zero);
        expect(
          c.read(purchaseFlowProvider).outcome,
          PurchaseOutcome.nothingToRestore,
        );
        expect(c.read(purchaseFlowProvider).busy, isFalse);

        // Found: sync then restored.
        await c.read(purchaseFlowProvider.notifier).restore();
        events.add(
          const StoreEvent.receipt(StoreReceipt(productId: '', restored: true)),
        );
        await Future<void>.delayed(Duration.zero);
        await Future<void>.delayed(Duration.zero);
        verify(svc.syncStorePlan(any)).called(1);
        expect(c.read(purchaseFlowProvider).outcome, PurchaseOutcome.restored);
        expect(c.read(planProvider).valueOrNull?.isPremium, isTrue);
        await events.close();
      },
    );

    test('a failed sync after a store success is "pending", never "failed", '
        'and keeps the last-known plan', () async {
      final svc = MockPlanService();
      when(svc.getMyPlan(any)).thenAnswer((_) async => PlanSnapshotView.free);
      when(svc.syncStorePlan(any)).thenThrow(Exception('offline'));
      final store = MockStoreClient();
      final events = StreamController<StoreEvent>.broadcast();
      when(store.events).thenAnswer((_) => events.stream);
      when(store.setAccount(any)).thenAnswer((_) async {});
      final c = _container(service: svc, store: store);
      await c.read(planProvider.future);
      c.read(purchaseFlowProvider);
      events.add(
        const StoreEvent.receipt(StoreReceipt(productId: 'premium_monthly')),
      );
      await Future<void>.delayed(Duration.zero);
      await Future<void>.delayed(Duration.zero);
      expect(c.read(purchaseFlowProvider).outcome, PurchaseOutcome.syncPending);
      expect(c.read(planProvider).valueOrNull, PlanSnapshotView.free);
      await events.close();
    });

    test(
      'signed out ⇒ the store SDK is unbound and no restore starts',
      () async {
        final svc = MockPlanService();
        when(svc.getMyPlan(any)).thenAnswer((_) async => PlanSnapshotView.free);
        final store = MockStoreClient();
        when(store.events).thenAnswer((_) => const Stream.empty());
        when(store.setAccount(any)).thenAnswer((_) async {});
        final c = _container(service: svc, store: store, userId: null);
        c.read(purchaseFlowProvider);
        await Future<void>.delayed(Duration.zero);
        verify(store.setAccount(null)).called(1);
        await c.read(purchaseFlowProvider.notifier).restore();
        verifyNever(store.restore(accountToken: anyNamed('accountToken')));
      },
    );

    test('buy is refused when the server says no purchase here', () async {
      final svc = MockPlanService();
      when(
        svc.getMyPlan(any),
      ).thenAnswer((_) async => _premium(managedOn: PlanChannel.google));
      final store = MockStoreClient();
      when(store.events).thenAnswer((_) => const Stream.empty());
      final c = _container(service: svc, store: store);
      await c.read(planProvider.future);
      await c.read(purchaseFlowProvider.notifier).buy('premium_monthly');
      verifyNever(store.buy(any, accountToken: anyNamed('accountToken')));
      expect(c.read(purchaseFlowProvider).seq, 0);
    });
  });

  group('withdrawal', () {
    test('lockedFontIds keeps only costed, unowned catalog fonts', () {
      RewardShopItemView item(String k, int cost, bool owned) =>
          RewardShopItemView(
            key: k,
            label: k,
            instrument: 'piano',
            license: 'CC0',
            attribution: '',
            pointCost: cost,
            redeemable: true,
            owned: owned,
          );
      expect(
        lockedFontIds([
          item('free', 0, false),
          item('owned', 50, true),
          item('locked', 50, false),
        ]),
        ['locked'],
      );
    });
  });
}
