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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/curator_profile_notifier.dart';
import 'package:music/state/reward_shop_notifier.dart';

import '../support/prefs_fakes.dart';
import 'curator_rewards_test.mocks.dart';

@GenerateNiceMocks([MockSpec<CuratorRewardsService>()])
CuratorRewardsView _rewards({
  int lifetime = 200,
  int balance = 200,
  int level = 2,
  int floor = 150,
  int next = 350,
  List<RewardActivityView> recent = const [],
}) => CuratorRewardsView(
  lifetimePoints: lifetime,
  spendableBalance: balance,
  level: level,
  levelFloor: floor,
  nextLevelAt: next,
  totalRatings: 12,
  coverageContribution: 7,
  alignmentRate: 0.75,
  badges: const [
    CuratorBadgeView(
      key: 'first_note',
      metric: 'rating_count',
      threshold: 1,
      earned: true,
    ),
    CuratorBadgeView(
      key: 'curator_2',
      metric: 'rating_count',
      threshold: 100,
      earned: false,
    ),
  ],
  recent: recent,
);

RewardShopItemView _item(String key, {int cost = 50, bool owned = false}) =>
    RewardShopItemView(
      key: key,
      label: 'Font $key',
      instrument: 'piano',
      license: 'CC0-1.0',
      attribution: '',
      pointCost: cost,
      redeemable: true,
      owned: owned,
    );

ProviderContainer _container(
  CuratorRewardsService service, {
  Map<String, String>? prefs,
}) {
  final c = ProviderContainer(
    overrides: [
      curatorRewardsServiceProvider.overrideWithValue(service),
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService(prefs),
      ),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  group('CuratorRewardsView', () {
    test('levelProgress is the fraction into the current band', () {
      // 200 lifetime, band [150, 350] → (200-150)/(350-150) = 0.25.
      expect(_rewards().levelProgress, closeTo(0.25, 1e-9));
    });

    test('levelProgress is full at a degenerate (top) band', () {
      expect(_rewards(floor: 3000, next: 3000).levelProgress, 1.0);
    });

    test('newestActivityAt is the first entry, or null when empty', () {
      expect(_rewards().newestActivityAt, isNull);
      final at = DateTime(2026, 8, 1);
      final r = _rewards(
        recent: [RewardActivityView(kind: 'honesty', amount: 8, createdAt: at)],
      );
      expect(r.newestActivityAt, at);
    });
  });

  group('CuratorProfile notifier', () {
    test('loads the snapshot from the service', () async {
      final service = MockCuratorRewardsService();
      when(service.getRewards()).thenAnswer((_) async => _rewards());
      final c = _container(service);

      final r = await c.read(curatorProfileProvider.future);
      expect(r.level, 2);
      expect(r.lifetimePoints, 200);
    });

    test('refresh reloads from the service', () async {
      final service = MockCuratorRewardsService();
      var lifetime = 200;
      when(
        service.getRewards(),
      ).thenAnswer((_) async => _rewards(lifetime: lifetime));
      final c = _container(service);
      await c.read(curatorProfileProvider.future);

      lifetime = 260;
      await c.read(curatorProfileProvider.notifier).refresh();
      expect(c.read(curatorProfileProvider).value!.lifetimePoints, 260);
    });
  });

  group('curatorHasUnseenAwards', () {
    test(
      'true when a deferred honesty award is newer than last seen',
      () async {
        final service = MockCuratorRewardsService();
        final landed = DateTime(2026, 8, 3);
        when(service.getRewards()).thenAnswer(
          (_) async => _rewards(
            recent: [
              RewardActivityView(
                kind: 'honesty',
                amount: 8,
                source: 'moderator',
                createdAt: landed,
              ),
            ],
          ),
        );
        // Last seen is BEFORE the award → unseen.
        final c = _container(
          service,
          prefs: {
            CuratorActivitySeen.prefsKey: DateTime(
              2026,
              8,
              1,
            ).millisecondsSinceEpoch.toString(),
          },
        );
        await c.read(curatorProfileProvider.future);
        await c.read(curatorActivitySeenProvider.future);
        expect(c.read(curatorHasUnseenAwardsProvider), isTrue);
      },
    );

    test('false when the newest award was already seen', () async {
      final service = MockCuratorRewardsService();
      final landed = DateTime(2026, 8, 1);
      when(service.getRewards()).thenAnswer(
        (_) async => _rewards(
          recent: [
            RewardActivityView(kind: 'honesty', amount: 8, createdAt: landed),
          ],
        ),
      );
      final c = _container(
        service,
        prefs: {
          CuratorActivitySeen.prefsKey: DateTime(
            2026,
            8,
            2,
          ).millisecondsSinceEpoch.toString(),
        },
      );
      await c.read(curatorProfileProvider.future);
      await c.read(curatorActivitySeenProvider.future);
      expect(c.read(curatorHasUnseenAwardsProvider), isFalse);
    });

    test(
      'false when there are no deferred (honesty/adjustment) awards',
      () async {
        final service = MockCuratorRewardsService();
        when(service.getRewards()).thenAnswer(
          (_) async => _rewards(
            recent: [
              // A coverage award is immediate (already shown as "+N"), not deferred.
              RewardActivityView(
                kind: 'coverage',
                amount: 10,
                createdAt: DateTime(2026, 8, 5),
              ),
            ],
          ),
        );
        final c = _container(service);
        await c.read(curatorProfileProvider.future);
        await c.read(curatorActivitySeenProvider.future);
        expect(c.read(curatorHasUnseenAwardsProvider), isFalse);
      },
    );
  });

  group('RewardShop notifier', () {
    test(
      'redeem success refreshes items and signals the profile to refresh',
      () async {
        final service = MockCuratorRewardsService();
        // First list: unowned; after redeem: owned.
        final responses = [
          [_item('grand')],
          [_item('grand', owned: true)],
        ];
        var call = 0;
        when(service.listShop()).thenAnswer((_) async {
          final r = responses[call.clamp(0, 1)];
          call++;
          return r;
        });
        when(service.redeem('grand')).thenAnswer(
          (_) async => const RedeemResultView(owned: true, newBalance: 150),
        );
        final c = _container(service);
        await c.read(rewardShopProvider.future);

        await c.read(rewardShopProvider.notifier).redeem('grand');

        final state = c.read(rewardShopProvider).value!;
        expect(state.lastRedeemedLabel, 'Font grand');
        expect(state.redeemError, isFalse);
        expect(state.redeemSeq, 1);
        expect(state.items.single.owned, isTrue);
        // The profile is told to refresh (revision bumped) — never invalidated directly.
        expect(c.read(rewardRevisionProvider), 1);
        verify(service.redeem('grand')).called(1);
      },
    );

    test('redeem failure surfaces via redeemError, no revision bump', () async {
      final service = MockCuratorRewardsService();
      when(service.listShop()).thenAnswer((_) async => [_item('grand')]);
      when(service.redeem('grand')).thenThrow(Exception('boom'));
      final c = _container(service);
      await c.read(rewardShopProvider.future);

      await c.read(rewardShopProvider.notifier).redeem('grand');

      final state = c.read(rewardShopProvider).value!;
      expect(state.redeemError, isTrue);
      expect(state.lastRedeemedLabel, isNull);
      expect(state.redeemSeq, 1);
      expect(c.read(rewardRevisionProvider), 0); // not bumped on failure
    });

    test('rewardShopItemsByKey indexes loaded items by key', () async {
      final service = MockCuratorRewardsService();
      when(
        service.listShop(),
      ).thenAnswer((_) async => [_item('a'), _item('b', cost: 150)]);
      final c = _container(service);
      await c.read(rewardShopProvider.future);
      final byKey = c.read(rewardShopItemsByKeyProvider);
      expect(byKey.keys, containsAll(['a', 'b']));
      expect(byKey['b']!.pointCost, 150);
    });
  });
}
