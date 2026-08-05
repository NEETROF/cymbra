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
import 'package:music/screens/curator_activity_screen.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/widgets/curator_chip.dart';
import 'package:music/widgets/curator_rewards_section.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import 'curator_rewards_screens_test.mocks.dart';

@GenerateNiceMocks([MockSpec<CuratorRewardsService>()])
CuratorRewardsView _rewards({List<RewardActivityView> recent = const []}) =>
    CuratorRewardsView(
      lifetimePoints: 200,
      spendableBalance: 200,
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
      recent: recent,
    );

List<Override> _overrides(CuratorRewardsService service) => [
  curatorRewardsServiceProvider.overrideWithValue(service),
  preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
];

Widget _hostSection(CuratorRewardsService service) => ProviderScope(
  overrides: _overrides(service),
  // The section is a Column; host it in a scroll view like the profile does.
  child: localizedApp(
    const Scaffold(body: SingleChildScrollView(child: CuratorRewardsSection())),
  ),
);

void main() {
  testWidgets('curator standing pill shows the level/points', (tester) async {
    final service = MockCuratorRewardsService();
    when(service.getRewards()).thenAnswer((_) async => _rewards());
    await tester.pumpWidget(
      ProviderScope(
        overrides: _overrides(service),
        child: localizedApp(
          const Scaffold(body: Center(child: CuratorStandingPill())),
        ),
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
      await tester.pumpWidget(_hostSection(service));
      await tester.pumpAndSettle();

      // Level + lifetime points (from the standing card).
      expect(find.textContaining('2'), findsWidgets);
      expect(find.textContaining('200'), findsWidgets);
      // Balance CTA into the SoundFont catalog + the stats (alignment 75%).
      expect(find.byIcon(Icons.library_music_outlined), findsWidgets);
      expect(find.textContaining('75'), findsWidgets);
      // Badge grid shows both an earned and a locked badge.
      expect(
        find.byIcon(Icons.military_tech),
        findsWidgets,
      ); // earned first_note
      expect(find.byIcon(Icons.lock_outline), findsWidgets); // locked curator_2
    },
  );

  testWidgets('recent-activity entry opens the activity screen on demand', (
    tester,
  ) async {
    final service = MockCuratorRewardsService();
    when(service.getRewards()).thenAnswer(
      (_) async => _rewards(
        recent: [
          RewardActivityView(
            kind: 'honesty',
            amount: 8,
            source: 'moderator',
            createdAt: DateTime(2026, 8, 3),
          ),
        ],
      ),
    );
    await tester.pumpWidget(_hostSection(service));
    await tester.pumpAndSettle();

    // The profile only shows an entry row — not the full list inline.
    expect(find.byKey(const Key('curator-recent-entry')), findsOneWidget);
    await tester.ensureVisible(find.byKey(const Key('curator-recent-entry')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('curator-recent-entry')));
    await tester.pumpAndSettle();

    // The full activity list opens on its own screen.
    expect(find.byType(CuratorActivityScreen), findsOneWidget);
  });
}
