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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/screens/reward_shop_screen.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/widgets/curator_chip.dart';
import 'package:music/widgets/curator_rewards_section.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import 'curator_rewards_screens_test.mocks.dart';

@GenerateNiceMocks([MockSpec<CuratorRewardsService>()])
CuratorRewardsView _rewards({int balance = 200}) => CuratorRewardsView(
  lifetimePoints: 200,
  spendableBalance: balance,
  level: 2,
  levelFloor: 150,
  nextLevelAt: 350,
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
  recent: const [],
);

RewardShopItemView _item(
  String key, {
  int cost = 50,
  bool owned = false,
  bool redeemable = true,
}) => RewardShopItemView(
  key: key,
  label: 'Grand $key',
  instrument: 'piano',
  license: 'CC0-1.0',
  attribution: '',
  pointCost: cost,
  redeemable: redeemable,
  owned: owned,
);

List<Override> _overrides(CuratorRewardsService service) => [
  curatorRewardsServiceProvider.overrideWithValue(service),
  preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
];

void main() {
  testWidgets('curator chip shows the level/points standing', (tester) async {
    final service = MockCuratorRewardsService();
    when(service.getRewards()).thenAnswer((_) async => _rewards());
    await tester.pumpWidget(
      ProviderScope(
        overrides: _overrides(service),
        child: localizedApp(const Scaffold(body: Center(child: CuratorChip()))),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('curator-chip')), findsOneWidget);
    // The pill carries the level (2) and lifetime points (200).
    expect(find.textContaining('2'), findsWidgets);
    expect(find.textContaining('200'), findsWidgets);
  });

  testWidgets(
    'curator rewards section renders level, points, balance, badges, stats',
    (tester) async {
      final service = MockCuratorRewardsService();
      when(service.getRewards()).thenAnswer((_) async => _rewards());
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(service),
          // The section is a Column; host it in a scroll view like the profile does.
          child: localizedApp(
            const Scaffold(
              body: SingleChildScrollView(child: CuratorRewardsSection()),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Level + lifetime points (from the standing card).
      expect(find.textContaining('2'), findsWidgets);
      expect(find.textContaining('200'), findsWidgets);
      // Reward-shop entry button and the stats (alignment 75%).
      expect(find.byIcon(Icons.card_giftcard), findsWidgets);
      expect(find.textContaining('75'), findsWidgets);
      // Badge grid shows both an earned and a locked badge.
      expect(
        find.byIcon(Icons.military_tech),
        findsWidgets,
      ); // earned first_note
      expect(find.byIcon(Icons.lock_outline), findsWidgets); // locked curator_2
    },
  );

  testWidgets('reward-shop entry from the section opens the shop', (
    tester,
  ) async {
    final service = MockCuratorRewardsService();
    when(service.getRewards()).thenAnswer((_) async => _rewards());
    when(service.listShop()).thenAnswer((_) async => [_item('grand')]);
    await tester.pumpWidget(
      ProviderScope(
        overrides: _overrides(service),
        child: localizedApp(
          const Scaffold(
            body: SingleChildScrollView(child: CuratorRewardsSection()),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.card_giftcard).first);
    await tester.pumpAndSettle();
    expect(find.byType(RewardShopScreen), findsOneWidget);
  });

  testWidgets(
    'reward shop enables an affordable redeem and disables an unaffordable one',
    (tester) async {
      final service = MockCuratorRewardsService();
      when(
        service.getRewards(),
      ).thenAnswer((_) async => _rewards(balance: 100));
      when(service.listShop()).thenAnswer(
        (_) async => [
          _item('cheap', cost: 50), // affordable (balance 100)
          _item('dear', cost: 300), // unaffordable
          _item('soon', cost: 400, redeemable: false), // coming soon
          _item('mine', cost: 20, owned: true), // owned
        ],
      );
      when(service.redeem('cheap')).thenAnswer(
        (_) async => const RedeemResultView(owned: true, newBalance: 50),
      );
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(service),
          child: localizedApp(const RewardShopScreen()),
        ),
      );
      await tester.pumpAndSettle();

      final cheap = tester.widget<FilledButton>(
        find.byKey(const Key('reward-redeem-cheap')),
      );
      final dear = tester.widget<FilledButton>(
        find.byKey(const Key('reward-redeem-dear')),
      );
      expect(cheap.onPressed, isNotNull); // affordable → enabled
      expect(dear.onPressed, isNull); // unaffordable → disabled
      // Coming-soon + owned items render no redeem button.
      expect(find.byKey(const Key('reward-redeem-soon')), findsNothing);
      expect(find.byKey(const Key('reward-redeem-mine')), findsNothing);

      // Redeeming an affordable item calls the service.
      await tester.tap(find.byKey(const Key('reward-redeem-cheap')));
      await tester.pump();
      verify(service.redeem('cheap')).called(1);
    },
  );
}
